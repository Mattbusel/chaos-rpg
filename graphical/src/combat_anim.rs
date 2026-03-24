// Combat animation sequencer — CHAOS RPG "Every Action Tells a Story"
//
// Manages multi-phase visual overlays for all combat actions.
// Phases: Windup → Travel → Impact → Recovery → Done
//
// Coordinate conventions (160×80 tile grid):
//   Enemy sprite center:  x ≈ 38,  y ≈ 18
//   Player sprite center: x ≈ 120, y ≈ 18
//   Trail travels from player side (x≈78) to enemy side (x≈40)
//   Telegraph travels from enemy side (x≈42) toward player side (x≈82)

use bracket_lib::prelude::*;

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AnimPhase { Windup, Travel, Impact, Recovery, Done }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WeaponKind { Sword, Axe, Dagger, Fist, Staff }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpellElement { Fire, Ice, Lightning, Arcane, Necro, Heal }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StatusKind { Burn, Freeze, Poison, Stun, Bleed, Blessed, Curse }

#[derive(Clone, PartialEq, Debug)]
pub enum AnimKind {
    PlayerMelee  { damage: i64, is_crit: bool, weapon: WeaponKind },
    PlayerHeavy  { damage: i64, is_crit: bool, weapon: WeaponKind },
    PlayerSpell  { damage: i64, is_crit: bool, element: SpellElement },
    PlayerDefend,
    PlayerFlee   { success: bool },
    EnemyMelee   { damage: i64, is_crit: bool },
    EnemySpecial { name: String },
    StatusApply  { status: StatusKind, on_enemy: bool },
    LevelUpPillar,
    None,
}

// ── Trail character ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct TrailChar {
    x: i32,
    y: i32,
    ch: &'static str,
    reveal_at: u32,   // phase_frame when this char appears
    brightness: f32,
}

// ── Status particle emission helper (returned as a list) ──────────────────────

pub struct EmitRequest {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ch: &'static str,
    pub col: (u8, u8, u8),
    pub lifetime: u32,
}

// ── CombatAnim ────────────────────────────────────────────────────────────────

pub struct CombatAnim {
    pub active: bool,
    pub phase:  AnimPhase,
    pub kind:   AnimKind,

    phase_frame:     u32,
    windup_frames:   u32,
    travel_frames:   u32,
    impact_frames:   u32,
    recovery_frames: u32,

    trail:            Vec<TrailChar>,
    // Enemy special ability text (typewriter)
    ability_text:     String,
    ability_text_len: usize,
    ability_tick:     u32,
    // Status application ring state
    status_ring_radius: f32,
    // Queued particle emissions from this frame's update
    pub pending_particles: Vec<EmitRequest>,
}

impl CombatAnim {
    pub fn new() -> Self {
        Self {
            active: false, phase: AnimPhase::Done, kind: AnimKind::None,
            phase_frame: 0,
            windup_frames: 1, travel_frames: 1, impact_frames: 1, recovery_frames: 1,
            trail: Vec::new(),
            ability_text: String::new(), ability_text_len: 0, ability_tick: 0,
            status_ring_radius: 0.0,
            pending_particles: Vec::new(),
        }
    }

    pub fn is_done(&self) -> bool { !self.active || self.phase == AnimPhase::Done }

    // ── Start methods ──────────────────────────────────────────────────────────

    pub fn start_player_melee(&mut self, damage: i64, is_crit: bool, weapon: WeaponKind, speed: f32) {
        let (w, t, i, r) = if is_crit { (14, 12, 18, 8) } else { (10, 9, 14, 6) };
        self.reset(w, t, i, r, speed);
        self.kind = AnimKind::PlayerMelee { damage, is_crit, weapon };
        self.build_melee_trail(weapon, false);
    }

    pub fn start_player_heavy(&mut self, damage: i64, is_crit: bool, weapon: WeaponKind, speed: f32) {
        let (w, t, i, r) = if is_crit { (18, 14, 22, 10) } else { (14, 11, 18, 8) };
        self.reset(w, t, i, r, speed);
        self.kind = AnimKind::PlayerHeavy { damage, is_crit, weapon };
        self.build_melee_trail(weapon, true);
    }

    pub fn start_player_spell(&mut self, damage: i64, is_crit: bool, element: SpellElement, speed: f32) {
        let (w, t, i, r) = if is_crit { (20, 14, 20, 8) } else { (16, 12, 16, 6) };
        self.reset(w, t, i, r, speed);
        self.kind = AnimKind::PlayerSpell { damage, is_crit, element };
        self.build_spell_trail(element);
    }

    pub fn start_enemy_attack(&mut self, damage: i64, is_crit: bool, speed: f32) {
        let (w, t, i, r) = if is_crit { (16, 11, 15, 6) } else { (13, 9, 12, 5) };
        self.reset(w, t, i, r, speed);
        self.kind = AnimKind::EnemyMelee { damage, is_crit };
    }

    pub fn start_enemy_special(&mut self, name: &str, speed: f32) {
        self.reset(32, 16, 22, 10, speed);
        self.kind = AnimKind::EnemySpecial { name: name.to_string() };
        self.ability_text = name.to_string();
        self.ability_text_len = 0;
        self.ability_tick = 0;
    }

    pub fn start_defend(&mut self, speed: f32) {
        self.reset(8, 10, 14, 8, speed);
        self.kind = AnimKind::PlayerDefend;
    }

    pub fn start_flee(&mut self, success: bool, speed: f32) {
        self.reset(18, 12, 14, 8, speed);
        self.kind = AnimKind::PlayerFlee { success };
    }

    pub fn start_status_apply(&mut self, status: StatusKind, on_enemy: bool, speed: f32) {
        self.reset(6, 8, 14, 6, speed);
        self.kind = AnimKind::StatusApply { status, on_enemy };
        self.status_ring_radius = 0.0;
    }

    pub fn start_level_up_pillar(&mut self, speed: f32) {
        self.reset(0, 0, 36, 10, speed);
        self.phase = AnimPhase::Impact;   // skip windup/travel, go straight to pillar
        self.kind = AnimKind::LevelUpPillar;
    }

    // ── Reset ─────────────────────────────────────────────────────────────────

    fn reset(&mut self, windup: u32, travel: u32, impact: u32, recovery: u32, speed: f32) {
        self.active = true;
        self.phase = AnimPhase::Windup;
        self.phase_frame = 0;
        self.trail.clear();
        self.pending_particles.clear();
        self.ability_text.clear();
        self.ability_text_len = 0;
        self.status_ring_radius = 0.0;
        let s = speed.max(0.1);
        self.windup_frames   = ((windup   as f32 / s) as u32).max(1);
        self.travel_frames   = ((travel   as f32 / s) as u32).max(1);
        self.impact_frames   = ((impact   as f32 / s) as u32).max(1);
        self.recovery_frames = ((recovery as f32 / s) as u32).max(1);
    }

    // ── Trail builders ────────────────────────────────────────────────────────

    fn build_melee_trail(&mut self, weapon: WeaponKind, heavy: bool) {
        let chars: &[&'static str] = match weapon {
            WeaponKind::Sword  => &["/", "─", "\\", "│", "/", "─", "\\"],
            WeaponKind::Axe    => &[")", "─", "(", "═", ")", "─", "("],
            WeaponKind::Dagger => &["·", "─", "►", "·", "─", "►", "·"],
            WeaponKind::Fist   => &["○", "●", "◉", "●", "○", "·", "●"],
            WeaponKind::Staff  => &["~", "≈", "~", "≈", "∿", "~", "≈"],
        };
        // Trail: x from 78 down to 40 at y=18 (enemy sprite area)
        let y_base = 18i32;
        let x_start = 78i32;
        let x_end   = 40i32;
        let total = (x_start - x_end) as u32;
        let rows: i32 = if heavy { 2 } else { 1 };
        for (i, x) in (x_end..=x_start).rev().enumerate() {
            let reveal = i as u32 * self.travel_frames / total.max(1);
            for dy in 0..rows {
                self.trail.push(TrailChar {
                    x, y: y_base + dy,
                    ch: chars[i % chars.len()],
                    reveal_at: reveal,
                    brightness: 1.0,
                });
            }
        }
    }

    fn build_spell_trail(&mut self, element: SpellElement) {
        let chars: &[&'static str] = match element {
            SpellElement::Fire      => &["~", "≈", "✦", "·", "~", "≈"],
            SpellElement::Ice       => &["✧", "·", "*", "❄", "✧", "·"],
            SpellElement::Lightning => &["⚡", "/", "\\", "|", "─", "⚡"],
            SpellElement::Arcane    => &["·", "●", "◉", "⊕", "✦", "·"],
            SpellElement::Necro     => &["☠", "~", "≈", "·", "☠", "~"],
            SpellElement::Heal      => &["✚", "+", "·", "✚", "+", "·"],
        };
        let y_base  = 18i32;
        let x_start = 78i32;
        let x_end   = 40i32;
        let total = (x_start - x_end) as u32;
        for (i, x) in (x_end..=x_start).rev().enumerate() {
            let y_off: i32 = match element {
                SpellElement::Lightning => {
                    let seed = (i as i64 * 7 + 13) % 5;
                    (seed - 2) as i32
                }
                SpellElement::Arcane => ((i as f32 * 0.45).sin() * 2.0).round() as i32,
                SpellElement::Necro  => if i % 4 == 0 { 1 } else { 0 },
                _ => 0,
            };
            let reveal = i as u32 * self.travel_frames / total.max(1);
            self.trail.push(TrailChar {
                x, y: y_base + y_off,
                ch: chars[i % chars.len()],
                reveal_at: reveal,
                brightness: 1.0,
            });
            // Lightning branches: every 8th tile, add fork
            if element == SpellElement::Lightning && i % 8 == 4 && i + 1 < total as usize {
                self.trail.push(TrailChar {
                    x, y: y_base + y_off + 1,
                    ch: "╱",
                    reveal_at: reveal,
                    brightness: 0.6,
                });
            }
        }
    }

    // ── Update ────────────────────────────────────────────────────────────────

    pub fn update(&mut self, frame: u64) {
        if !self.active { return; }
        self.pending_particles.clear();
        self.phase_frame += 1;

        // Fade trail characters during Recovery
        if self.phase == AnimPhase::Recovery || self.phase == AnimPhase::Impact {
            let fade_rate = 1.0 / self.recovery_frames.max(1) as f32;
            for tc in &mut self.trail {
                tc.brightness = (tc.brightness - fade_rate * 1.5).max(0.0);
            }
        }

        // Enemy special ability typewriter
        if let AnimKind::EnemySpecial { .. } = &self.kind {
            self.ability_tick += 1;
            if self.ability_tick >= 2 {
                self.ability_tick = 0;
                if self.ability_text_len < self.ability_text.len() {
                    self.ability_text_len += 1;
                }
            }
        }

        // Status ring expands during impact
        if matches!(self.kind, AnimKind::StatusApply { .. }) && self.phase == AnimPhase::Impact {
            self.status_ring_radius += 0.6;
        }

        // Level up pillar: emit upward particles
        if self.phase == AnimPhase::Impact {
            if let AnimKind::LevelUpPillar = &self.kind {
                let intensity = (self.phase_frame as f32 / self.impact_frames as f32).min(1.0);
                if frame % 2 == 0 {
                    let col = (255u8, 215u8, 0u8);
                    let pillar_chars = ["│", "║", "✦", "·", "│"];
                    let ch = pillar_chars[(frame as usize / 3) % pillar_chars.len()];
                    // Pillar from player panel center upward
                    let px = 120.0f32;
                    let py = 20.0f32 - (intensity * 15.0);
                    self.pending_particles.push(EmitRequest {
                        x: px + ((frame % 5) as f32 - 2.0), y: py,
                        vx: 0.0, vy: -0.15,
                        ch, col, lifetime: 20,
                    });
                }
            }
        }

        // Phase transition
        let limit = match self.phase {
            AnimPhase::Windup   => self.windup_frames,
            AnimPhase::Travel   => self.travel_frames,
            AnimPhase::Impact   => self.impact_frames,
            AnimPhase::Recovery => self.recovery_frames,
            AnimPhase::Done     => { self.active = false; return; }
        };
        if self.phase_frame >= limit {
            self.phase_frame = 0;
            self.phase = match self.phase {
                AnimPhase::Windup   => {
                    if self.travel_frames <= 1 { AnimPhase::Impact } else { AnimPhase::Travel }
                }
                AnimPhase::Travel   => AnimPhase::Impact,
                AnimPhase::Impact   => AnimPhase::Recovery,
                AnimPhase::Recovery => { self.active = false; AnimPhase::Done }
                AnimPhase::Done     => AnimPhase::Done,
            };
        }
    }

    // ── Draw ──────────────────────────────────────────────────────────────────

    pub fn draw(&self, ctx: &mut BTerm, bg: RGB, frame: u64) {
        if !self.active { return; }
        match &self.kind {
            AnimKind::PlayerMelee { is_crit, weapon, .. }
            | AnimKind::PlayerHeavy { is_crit, weapon, .. } => {
                self.draw_player_melee(ctx, bg, *is_crit, *weapon, frame);
            }
            AnimKind::PlayerSpell { is_crit, element, .. } => {
                self.draw_player_spell(ctx, bg, *is_crit, *element, frame);
            }
            AnimKind::EnemyMelee { is_crit, .. } => {
                self.draw_enemy_melee(ctx, bg, *is_crit, frame);
            }
            AnimKind::EnemySpecial { .. } => {
                self.draw_enemy_special(ctx, bg, frame);
            }
            AnimKind::PlayerDefend => {
                self.draw_defend(ctx, bg, frame);
            }
            AnimKind::PlayerFlee { success } => {
                self.draw_flee(ctx, bg, *success, frame);
            }
            AnimKind::StatusApply { status, on_enemy } => {
                self.draw_status_apply(ctx, bg, *status, *on_enemy, frame);
            }
            AnimKind::LevelUpPillar => {
                self.draw_level_up_pillar(ctx, bg, frame);
            }
            AnimKind::None => {}
        }
    }

    // ── Player melee ──────────────────────────────────────────────────────────

    fn draw_player_melee(&self, ctx: &mut BTerm, bg: RGB, is_crit: bool, weapon: WeaponKind, frame: u64) {
        let base_col: (u8, u8, u8) = if is_crit {
            (255, 200, 50)
        } else {
            (160, 220, 160)
        };

        // Windup: weapon charging indicator at player side
        if self.phase == AnimPhase::Windup {
            let t = self.phase_frame as f32 / self.windup_frames as f32;
            let v = (t * 200.0) as u8;
            let glyph = match weapon {
                WeaponKind::Sword  => "⚔",
                WeaponKind::Axe    => "⚒",
                WeaponKind::Dagger => "►",
                WeaponKind::Fist   => "◉",
                WeaponKind::Staff  => "~",
            };
            ctx.print_color(79, 17, RGB::from_u8(v, v/2, v/4), bg, glyph);
            // Windup whoosh trail (retreating dots)
            for i in 0..3i32 {
                let wx = 82 + i * 2;
                if wx < 160 {
                    let bv = (v as f32 * (1.0 - i as f32 * 0.3)) as u8;
                    ctx.print_color(wx, 18, RGB::from_u8(bv/3, bv/3, bv/3), bg, "·");
                }
            }
        }

        // Travel + Recovery: draw slash trail
        if matches!(self.phase, AnimPhase::Travel | AnimPhase::Impact | AnimPhase::Recovery) {
            for tc in &self.trail {
                if tc.brightness <= 0.02 { continue; }
                if self.phase == AnimPhase::Travel && self.phase_frame < tc.reveal_at { continue; }
                let r = (base_col.0 as f32 * tc.brightness) as u8;
                let g = (base_col.1 as f32 * tc.brightness) as u8;
                let b_c = (base_col.2 as f32 * tc.brightness) as u8;
                ctx.print_color(tc.x, tc.y, RGB::from_u8(r.max(4), g.max(4), b_c.max(4)), bg, tc.ch);
            }
        }

        // Impact: flash at enemy position
        if self.phase == AnimPhase::Impact && self.phase_frame < 5 {
            let iv = (255.0 * (1.0 - self.phase_frame as f32 / 5.0)) as u8;
            let impact_ch = if is_crit { "✦" } else { "*" };
            ctx.print_color(40, 17, RGB::from_u8(iv, iv, iv/2), bg, impact_ch);
            ctx.print_color(39, 18, RGB::from_u8(iv, iv/2, 0), bg, impact_ch);
            ctx.print_color(41, 18, RGB::from_u8(iv, iv/2, 0), bg, impact_ch);
        }
    }

    // ── Player spell ──────────────────────────────────────────────────────────

    fn draw_player_spell(&self, ctx: &mut BTerm, bg: RGB, is_crit: bool, element: SpellElement, frame: u64) {
        let (r, g, b_c) = spell_color(element);

        // Windup/channel: glyph at player position with growing intensity
        if self.phase == AnimPhase::Windup {
            let t = self.phase_frame as f32 / self.windup_frames as f32;
            let glyph = spell_channel_glyph(element, frame);
            let vr = (r as f32 * t) as u8;
            let vg = (g as f32 * t) as u8;
            let vb = (b_c as f32 * t) as u8;
            ctx.print_color(82, 17, RGB::from_u8(vr.max(8), vg.max(5), vb.max(5)), bg, glyph);
            // Mana flow: small dots converging from MP bar area (~x=83, y=8) to cast point
            if t > 0.3 {
                let flow_x = 83 + ((1.0 - t) * 8.0) as i32;
                let flow_y = 8 + ((17.0 - 8.0) * t) as i32;
                if flow_x < 160 && flow_y < 80 {
                    ctx.print_color(flow_x, flow_y, RGB::from_u8(vr/2, vg/2, vb.max(30)), bg, "·");
                }
            }
        }

        // Travel: draw spell trail
        if matches!(self.phase, AnimPhase::Travel | AnimPhase::Impact | AnimPhase::Recovery) {
            for tc in &self.trail {
                if tc.brightness <= 0.02 { continue; }
                if self.phase == AnimPhase::Travel && self.phase_frame < tc.reveal_at { continue; }
                let vr = (r as f32 * tc.brightness) as u8;
                let vg = (g as f32 * tc.brightness) as u8;
                let vb = (b_c as f32 * tc.brightness) as u8;
                ctx.print_color(tc.x, tc.y, RGB::from_u8(vr.max(4), vg.max(4), vb.max(4)), bg, tc.ch);
            }
        }

        // Impact: element-specific flash at enemy position
        if self.phase == AnimPhase::Impact && self.phase_frame < 8 {
            let iv = (1.0 - self.phase_frame as f32 / 8.0).powi(2);
            let vr = (r as f32 * iv * 1.2).min(255.0) as u8;
            let vg = (g as f32 * iv * 1.2).min(255.0) as u8;
            let vb = (b_c as f32 * iv * 1.2).min(255.0) as u8;
            let impact_ch = spell_impact_glyph(element);
            ctx.print_color(38, 17, RGB::from_u8(vr, vg, vb), bg, impact_ch);
            ctx.print_color(37, 18, RGB::from_u8(vr/2, vg/2, vb/2), bg, impact_ch);
            ctx.print_color(39, 18, RGB::from_u8(vr/2, vg/2, vb/2), bg, impact_ch);
            ctx.print_color(38, 19, RGB::from_u8(vr/2, vg/2, vb/2), bg, impact_ch);
            if element == SpellElement::Lightning && self.phase_frame < 3 {
                // Bright flash
                for fy in 15..=21i32 {
                    ctx.print_color(37, fy, RGB::from_u8(220, 220, 255), bg, " ");
                }
            }
        }
    }

    // ── Enemy melee ───────────────────────────────────────────────────────────

    fn draw_enemy_melee(&self, ctx: &mut BTerm, bg: RGB, is_crit: bool, frame: u64) {
        // Windup: telegraph dashes from enemy toward player
        if self.phase == AnimPhase::Windup {
            let t = self.phase_frame as f32 / self.windup_frames as f32;
            let base_v = (t * 140.0) as u8;
            let col = if is_crit {
                RGB::from_u8(base_v.max(30), 8, 8)
            } else {
                RGB::from_u8(base_v, base_v / 3, base_v / 3)
            };
            // Dashed telegraph line from x=45 to x=80
            let mut x = 45i32;
            let mut dash_i = 0usize;
            while x <= 80 {
                let ch = if x >= 80 { "►" } else if dash_i % 3 == 2 { " " } else { "─" };
                ctx.print_color(x, 18, col, bg, ch);
                x += 1;
                dash_i += 1;
            }
            // Pulse at enemy position
            if is_crit && (frame / 3) % 2 == 0 {
                ctx.print_color(40, 17, RGB::from_u8(base_v, 20, 20), bg, "⚔");
            }
        }

        // Travel: attack projectile moving toward player
        if self.phase == AnimPhase::Travel {
            let progress = self.phase_frame as f32 / self.travel_frames as f32;
            let head_x = 45 + (progress * 38.0) as i32;
            let ch = if is_crit { "⚔" } else { "►" };
            let col = if is_crit {
                RGB::from_u8(255, 40, 40)
            } else {
                RGB::from_u8(200, 80, 80)
            };
            if head_x >= 0 && head_x < 160 {
                ctx.print_color(head_x, 18, col, bg, ch);
                // Trail behind projectile
                for trail_dist in 1..4i32 {
                    let tx = head_x - trail_dist;
                    if tx > 44 {
                        let tv = (120.0 * (1.0 - trail_dist as f32 / 4.0)) as u8;
                        ctx.print_color(tx, 18, RGB::from_u8(tv, tv/4, tv/4), bg, "─");
                    }
                }
            }
        }

        // Impact: hit burst at player position
        if self.phase == AnimPhase::Impact && self.phase_frame < 6 {
            let iv = (1.0 - self.phase_frame as f32 / 6.0) * 255.0;
            let impact_ch = if is_crit { "✦" } else { "*" };
            let vr = iv as u8;
            let vg = (iv * 0.2) as u8;
            ctx.print_color(118, 17, RGB::from_u8(vr, vg, vg), bg, impact_ch);
            ctx.print_color(117, 18, RGB::from_u8(vr/2, vg/2, vg/2), bg, impact_ch);
            ctx.print_color(119, 18, RGB::from_u8(vr/2, vg/2, vg/2), bg, impact_ch);
        }
    }

    // ── Enemy special ─────────────────────────────────────────────────────────

    fn draw_enemy_special(&self, ctx: &mut BTerm, bg: RGB, frame: u64) {
        // Typewriter ability name above enemy
        if self.ability_text_len > 0 {
            let shown: String = self.ability_text.chars().take(self.ability_text_len).collect();
            let x = (40 - shown.len() as i32 / 2).max(3);
            let pulse = (frame / 4) % 2 == 0;
            let v: u8 = if pulse { 240 } else { 160 };
            ctx.print_color(x, 8, RGB::from_u8(v, v / 4, v / 4), bg, &shown);
        }

        // Windup: convergence brackets
        if self.phase == AnimPhase::Windup {
            let t = self.phase_frame as f32 / self.windup_frames as f32;
            let glow = (t * 200.0) as u8;
            let dx = ((1.0 - t) * 14.0) as i32;
            ctx.print_color((38 - dx).max(3), 16, RGB::from_u8(glow, glow/4, glow/4), bg, "◄");
            ctx.print_color((38 + dx).min(76), 16, RGB::from_u8(glow, glow/4, glow/4), bg, "►");
            // Gathering particle dots at enemy
            let chaos_chars = ["·", "*", "+", "·"];
            for i in 0..4i32 {
                let angle = (frame as f32 * 0.12 + i as f32 * 1.57).sin();
                let ex = 38 + (angle * (3.0 + t * 4.0)) as i32;
                let ey = 18 + ((frame as f32 * 0.1 + i as f32 * 1.57).cos() * 2.5) as i32;
                if ex > 2 && ex < 78 && ey > 3 && ey < 36 {
                    ctx.print_color(ex, ey, RGB::from_u8(glow/2, glow/8, glow/8), bg,
                        chaos_chars[i as usize % chaos_chars.len()]);
                }
            }
        }

        // Execute (Travel+Impact): ability flash on enemy panel
        if self.phase == AnimPhase::Travel || self.phase == AnimPhase::Impact {
            let t = match self.phase {
                AnimPhase::Travel => self.phase_frame as f32 / self.travel_frames as f32,
                AnimPhase::Impact => 1.0 - self.phase_frame as f32 / self.impact_frames as f32,
                _ => 1.0,
            };
            let v = (t * 200.0) as u8;
            // Expanding ring around enemy
            let r = (t * 8.0) as i32;
            for i in 0..=(r * 2) {
                let rx = 38 - r + i;
                if rx > 2 && rx < 78 {
                    ctx.print_color(rx, 16, RGB::from_u8(v, v/4, v/4), bg, "─");
                    ctx.print_color(rx, 20, RGB::from_u8(v, v/4, v/4), bg, "─");
                }
            }
            for i in 0..=(r * 1) {
                let ry = 17 - (r/2) + i;
                if ry > 3 && ry < 36 {
                    ctx.print_color(38 - r/2, ry, RGB::from_u8(v, v/4, v/4), bg, "│");
                    ctx.print_color(38 + r/2, ry, RGB::from_u8(v, v/4, v/4), bg, "│");
                }
            }
        }
    }

    // ── Player defend ─────────────────────────────────────────────────────────

    fn draw_defend(&self, ctx: &mut BTerm, bg: RGB, frame: u64) {
        let t = match self.phase {
            AnimPhase::Windup   => self.phase_frame as f32 / self.windup_frames as f32,
            AnimPhase::Travel
            | AnimPhase::Impact => 1.0,
            AnimPhase::Recovery => 1.0 - self.phase_frame as f32 / self.recovery_frames as f32,
            _ => 0.0,
        };
        let v = (t * 200.0) as u8;
        let bv = (t * 180.0) as u8;

        // Shield glyph at player front
        let shield_chars = ["▓", "█", "◊", "█", "▓", "◊"];
        let ch = shield_chars[(frame / 5) as usize % shield_chars.len()];
        ctx.print_color(84, 17, RGB::from_u8(v/3, v/3, v), bg, ch);
        ctx.print_color(84, 18, RGB::from_u8(v/4, v/4, bv), bg, "│");
        ctx.print_color(84, 19, RGB::from_u8(v/3, v/3, v), bg, ch);

        // Barrier line at midpoint
        if t > 0.5 {
            let bline_v = (bv as f32 * (t - 0.5) / 0.5) as u8;
            for by in 14..=22i32 {
                ctx.print_color(82, by, RGB::from_u8(bline_v/4, bline_v/4, bline_v), bg, "│");
            }
            // Defense value hint
            if t > 0.8 {
                let dv = ((t - 0.8) / 0.2 * 160.0) as u8;
                ctx.print_color(86, 17, RGB::from_u8(dv/4, dv/4, dv), bg, "+DEF");
            }
        }
    }

    // ── Player flee ───────────────────────────────────────────────────────────

    fn draw_flee(&self, ctx: &mut BTerm, bg: RGB, success: bool, frame: u64) {
        // Windup: spinning chaos indicator
        if self.phase == AnimPhase::Windup {
            let spin = ["|", "/", "─", "\\"];
            let ch = spin[(self.phase_frame as usize / 3) % spin.len()];
            ctx.print_color(118, 17, RGB::from_u8(180, 180, 180), bg, ch);
            // Speed lines
            let t = self.phase_frame as f32 / self.windup_frames as f32;
            if t > 0.4 {
                for i in 0..5i32 {
                    let sx = 120 + i * 4;
                    if sx < 158 {
                        let sv = (t * 80.0) as u8;
                        ctx.print_color(sx, 17 + i % 2, RGB::from_u8(sv, sv, sv), bg, "─");
                    }
                }
            }
        }

        // Travel + Impact: outcome reveal
        if matches!(self.phase, AnimPhase::Travel | AnimPhase::Impact | AnimPhase::Recovery) {
            let t = match self.phase {
                AnimPhase::Travel   => self.phase_frame as f32 / self.travel_frames as f32,
                AnimPhase::Impact   => 1.0,
                AnimPhase::Recovery => 1.0 - self.phase_frame as f32 / self.recovery_frames as f32,
                _ => 0.0,
            };
            let v = (t * 220.0) as u8;

            // Result check mark / X
            let (result_ch, result_col) = if success {
                ("✓", RGB::from_u8(v/4, v, v/4))
            } else {
                ("✗", RGB::from_u8(v, v/4, v/4))
            };
            ctx.print_color(118, 17, result_col, bg, result_ch);

            // Outcome text
            let label = if success { "ESCAPED" } else { "FAILED" };
            let lx = 114i32;
            let label_col = if success {
                RGB::from_u8(v/3, v, v/3)
            } else {
                RGB::from_u8(v, v/3, v/3)
            };
            ctx.print_color(lx, 19, label_col, bg, label);

            // Speed lines on success
            if success && t > 0.3 {
                for i in 0..6i32 {
                    let sx = 122 + i * 5;
                    if sx < 158 {
                        let sv = (v as f32 * (1.0 - i as f32 * 0.15)) as u8;
                        ctx.print_color(sx, 17 + (i % 3) - 1, RGB::from_u8(sv, sv, sv), bg, "─");
                    }
                }
            }
        }
    }

    // ── Status apply ──────────────────────────────────────────────────────────

    fn draw_status_apply(&self, ctx: &mut BTerm, bg: RGB, status: StatusKind, on_enemy: bool, frame: u64) {
        let (cx, cy) = if on_enemy { (38i32, 18i32) } else { (118i32, 18i32) };
        let col = status_color(status);
        let r_int = self.status_ring_radius as i32;
        let label = status_label(status);

        // Expanding ring of status characters
        if r_int > 0 {
            let ring_chars = status_ring_chars(status);
            let fade = (1.0 - self.status_ring_radius / 10.0).max(0.0);
            let vr = (col.0 as f32 * fade) as u8;
            let vg = (col.1 as f32 * fade) as u8;
            let vb = (col.2 as f32 * fade) as u8;
            if vr > 4 || vg > 4 || vb > 4 {
                for i in 0..=r_int * 2 {
                    let rx = cx - r_int + i;
                    let ry_top = cy - r_int / 2;
                    let ry_bot = cy + r_int / 2;
                    let ch = ring_chars[(i as usize) % ring_chars.len()];
                    if rx > 0 && rx < 159 {
                        if ry_top > 0 && ry_top < 79 {
                            ctx.print_color(rx, ry_top, RGB::from_u8(vr, vg, vb), bg, ch);
                        }
                        if ry_bot > 0 && ry_bot < 79 && ry_bot != ry_top {
                            ctx.print_color(rx, ry_bot, RGB::from_u8(vr, vg, vb), bg, ch);
                        }
                    }
                }
            }
        }

        // Status label fades in during impact
        if self.phase == AnimPhase::Impact {
            let t = self.phase_frame as f32 / self.impact_frames as f32;
            if t > 0.3 {
                let lv = ((t - 0.3) / 0.7 * 200.0) as u8;
                let lx = cx - label.len() as i32 / 2;
                let ly = cy - 3;
                if lx > 0 && ly > 0 {
                    ctx.print_color(lx, ly, RGB::from_u8(
                        (col.0 as f32 * lv as f32 / 200.0) as u8,
                        (col.1 as f32 * lv as f32 / 200.0) as u8,
                        (col.2 as f32 * lv as f32 / 200.0) as u8,
                    ), bg, label);
                }
            }
        }
    }

    // ── Level up pillar ───────────────────────────────────────────────────────

    fn draw_level_up_pillar(&self, ctx: &mut BTerm, bg: RGB, frame: u64) {
        if self.phase != AnimPhase::Impact && self.phase != AnimPhase::Recovery { return; }

        let t = match self.phase {
            AnimPhase::Impact   => self.phase_frame as f32 / self.impact_frames as f32,
            AnimPhase::Recovery => 1.0 - self.phase_frame as f32 / self.recovery_frames as f32,
            _ => 0.0,
        };
        let v = (t * 255.0) as u8;
        let gv = (t * 215.0) as u8;

        // Vertical pillar of gold chars from player panel up
        let px = 120i32;
        let top_y = (18.0 - t * 16.0) as i32;
        let pillar_chars = ["│", "║", "✦", "·", "║", "│"];
        for y in top_y.max(2)..=18i32 {
            let dist = 18 - y;
            let bv = (v as f32 * (1.0 - dist as f32 / 18.0).powi(2)) as u8;
            let bgv = (gv as f32 * (1.0 - dist as f32 / 18.0).powi(2)) as u8;
            let ch = pillar_chars[(y as usize + frame as usize / 3) % pillar_chars.len()];
            ctx.print_color(px, y, RGB::from_u8(bv, bgv, 0), bg, ch);
        }

        // "LEVEL UP" text briefly at peak
        if t > 0.5 && t < 0.95 {
            let label_v = ((t - 0.5) / 0.45 * 200.0) as u8;
            let label_gv = (label_v as f32 * 215.0 / 200.0) as u8;
            ctx.print_color(110, 14, RGB::from_u8(label_v, label_gv, 0), bg, "LEVEL UP");
        }

        // Spotlight: dim surrounding elements
        if t > 0.3 {
            let dim_v = ((1.0 - t) * 40.0) as u8;
            for x in 82..84i32 {
                for y in 4..8i32 {
                    ctx.print_color(x, y, RGB::from_u8(dim_v, dim_v, dim_v), bg, " ");
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn spell_color(element: SpellElement) -> (u8, u8, u8) {
    match element {
        SpellElement::Fire      => (255, 100,  20),
        SpellElement::Ice       => (100, 180, 255),
        SpellElement::Lightning => (200, 220, 255),
        SpellElement::Arcane    => (180,  80, 255),
        SpellElement::Necro     => ( 80, 200,  80),
        SpellElement::Heal      => ( 50, 220, 100),
    }
}

fn spell_channel_glyph(element: SpellElement, frame: u64) -> &'static str {
    let i = (frame / 3) as usize;
    match element {
        SpellElement::Fire      => ["☀","✦","☀","·"][i % 4],
        SpellElement::Ice       => ["❄","✧","❄","·"][i % 4],
        SpellElement::Lightning => ["⚡","│","⚡","─"][i % 4],
        SpellElement::Arcane    => ["◉","⊗","⊕","◉"][i % 4],
        SpellElement::Necro     => ["☠","·","☠","~"][i % 4],
        SpellElement::Heal      => ["✚","+","✚","·"][i % 4],
    }
}

fn spell_impact_glyph(element: SpellElement) -> &'static str {
    match element {
        SpellElement::Fire      => "☀",
        SpellElement::Ice       => "❄",
        SpellElement::Lightning => "⚡",
        SpellElement::Arcane    => "⊕",
        SpellElement::Necro     => "☠",
        SpellElement::Heal      => "✚",
    }
}

fn status_color(status: StatusKind) -> (u8, u8, u8) {
    match status {
        StatusKind::Burn    => (255, 100,  20),
        StatusKind::Freeze  => ( 80, 160, 255),
        StatusKind::Poison  => ( 40, 200,  40),
        StatusKind::Stun    => (220, 220,  80),
        StatusKind::Bleed   => (200,  20,  20),
        StatusKind::Blessed => (255, 220,  60),
        StatusKind::Curse   => (160,  40, 200),
    }
}

fn status_label(status: StatusKind) -> &'static str {
    match status {
        StatusKind::Burn    => "BURNING",
        StatusKind::Freeze  => "FROZEN",
        StatusKind::Poison  => "POISONED",
        StatusKind::Stun    => "STUNNED",
        StatusKind::Bleed   => "BLEEDING",
        StatusKind::Blessed => "BLESSED",
        StatusKind::Curse   => "CURSED",
    }
}

fn status_ring_chars(status: StatusKind) -> &'static [&'static str] {
    match status {
        StatusKind::Burn    => &["~", "≈", "·", "~"],
        StatusKind::Freeze  => &["*", "·", "❄", "·"],
        StatusKind::Poison  => &["·", "°", "·", "°"],
        StatusKind::Stun    => &["✦", "★", "·", "★"],
        StatusKind::Bleed   => &["/", "\\", "·", "/"],
        StatusKind::Blessed => &["✦", "+", "·", "+"],
        StatusKind::Curse   => &["~", "·", "×", "·"],
    }
}

/// Infer weapon kind from equipped weapon name keywords.
pub fn weapon_kind_from_name(name: &str) -> WeaponKind {
    let lower = name.to_lowercase();
    if lower.contains("axe") || lower.contains("hatchet") || lower.contains("cleaver") {
        WeaponKind::Axe
    } else if lower.contains("dagger") || lower.contains("knife") || lower.contains("shiv") {
        WeaponKind::Dagger
    } else if lower.contains("staff") || lower.contains("wand") || lower.contains("rod") || lower.contains("scepter") {
        WeaponKind::Staff
    } else if lower.contains("fist") || lower.contains("gauntlet") || lower.contains("knuckle") {
        WeaponKind::Fist
    } else {
        WeaponKind::Sword
    }
}

/// Infer spell element from spell name keywords.
pub fn spell_element_from_name(name: &str) -> SpellElement {
    let lower = name.to_lowercase();
    if lower.contains("fire") || lower.contains("flame") || lower.contains("burn") || lower.contains("ignite") || lower.contains("inferno") {
        SpellElement::Fire
    } else if lower.contains("ice") || lower.contains("frost") || lower.contains("freeze") || lower.contains("cryo") || lower.contains("cold") {
        SpellElement::Ice
    } else if lower.contains("lightning") || lower.contains("thunder") || lower.contains("spark") || lower.contains("bolt") || lower.contains("shock") {
        SpellElement::Lightning
    } else if lower.contains("heal") || lower.contains("restore") || lower.contains("regen") || lower.contains("mend") {
        SpellElement::Heal
    } else if lower.contains("death") || lower.contains("necro") || lower.contains("drain") || lower.contains("soul") || lower.contains("decay") {
        SpellElement::Necro
    } else {
        SpellElement::Arcane
    }
}

/// Infer status kind from status name.
pub fn status_kind_from_name(name: &str) -> StatusKind {
    let lower = name.to_lowercase();
    if lower.contains("burn") || lower.contains("fire") || lower.contains("immolat") { StatusKind::Burn }
    else if lower.contains("freeze") || lower.contains("frost") || lower.contains("ice") { StatusKind::Freeze }
    else if lower.contains("poison") || lower.contains("venom") { StatusKind::Poison }
    else if lower.contains("stun") || lower.contains("daze") { StatusKind::Stun }
    else if lower.contains("bleed") || lower.contains("lacerate") { StatusKind::Bleed }
    else if lower.contains("bless") || lower.contains("sacred") { StatusKind::Blessed }
    else { StatusKind::Curse }
}
