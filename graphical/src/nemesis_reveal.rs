// Nemesis reveal overlay — CHAOS RPG "Every Action Tells a Story"
//
// When the player encounters their Nemesis enemy, a full announcement panel
// assembles character-by-character before combat begins.
//
// Phases (all in frames):
//   0   Inactive
//   1   Darkening (0-15): screen fades dark
//   2   Assembly (15-75): panel border draws, text types in
//   3   Hold    (75-120): full display, particles active
//   4   Done    (120+): caller transitions to combat

use bracket_lib::prelude::*;

pub const NEMESIS_REVEAL_DONE_FRAME: u32 = 120;

pub struct NemesisReveal {
    pub active:       bool,
    pub frame:        u32,
    pub enemy_name:   String,
    pub enemy_floor:  u32,
    pub unique_ability: String,
    pub hp_bonus_pct:   u32,
    pub dmg_bonus_pct:  u32,
    pub player_name:  String,

    // Typewriter tracking
    chars_shown:  usize,
    text_timer:   u32,
}

impl NemesisReveal {
    pub fn new() -> Self {
        Self {
            active: false, frame: 0,
            enemy_name: String::new(), enemy_floor: 1,
            unique_ability: String::new(),
            hp_bonus_pct: 150, dmg_bonus_pct: 25,
            player_name: String::new(),
            chars_shown: 0, text_timer: 0,
        }
    }

    pub fn start(
        &mut self,
        enemy_name: &str,
        enemy_floor: u32,
        unique_ability: &str,
        hp_bonus_pct: u32,
        dmg_bonus_pct: u32,
        player_name: &str,
    ) {
        self.active = true;
        self.frame  = 0;
        self.enemy_name     = enemy_name.chars().take(30).collect();
        self.enemy_floor    = enemy_floor;
        self.unique_ability = unique_ability.chars().take(24).collect();
        self.hp_bonus_pct   = hp_bonus_pct;
        self.dmg_bonus_pct  = dmg_bonus_pct;
        self.player_name    = player_name.chars().take(20).collect();
        self.chars_shown    = 0;
        self.text_timer     = 0;
    }

    pub fn is_done(&self) -> bool { self.frame >= NEMESIS_REVEAL_DONE_FRAME }

    pub fn update(&mut self) {
        if !self.active { return; }
        self.frame += 1;

        // Typewriter advances in assembly phase
        if self.frame >= 15 {
            self.text_timer += 1;
            if self.text_timer >= 2 {
                self.text_timer = 0;
                self.chars_shown += 1;
            }
        }

        if self.frame >= NEMESIS_REVEAL_DONE_FRAME {
            self.active = false;
        }
    }

    pub fn draw(&self, ctx: &mut BTerm, bg: (u8, u8, u8), frame: u64) {
        if !self.active { return; }
        let f = self.frame;
        let bg_rgb = RGB::from_u8(bg.0, bg.1, bg.2);

        // ── Phase: Darkening (0-15) ────────────────────────────────────────────
        let dark_alpha = (f as f32 / 15.0).min(1.0);
        // Overlay dark vignette over full screen
        let dark_v = (dark_alpha * 35.0) as u8;
        if dark_v > 0 {
            for y in [0i32, 1, 78, 79] {
                for x in 0..160i32 {
                    ctx.print_color(x, y, RGB::from_u8(dark_v, dark_v/4, dark_v/4), bg_rgb, " ");
                }
            }
        }

        if f < 15 {
            // "Something remembers you..." teaser text
            let t = f as f32 / 15.0;
            let tv = (t * 80.0) as u8;
            let teaser = "Something remembers you...";
            let tx = (80 - teaser.len() as i32 / 2).max(0);
            ctx.print_color(tx, 70, RGB::from_u8(tv, tv/4, tv/4), bg_rgb, teaser);
            return;
        }

        // ── Phase: Assembly + Hold (15+) ──────────────────────────────────────
        let bw = 44i32;
        let bh = 12i32;
        let bx = (160 - bw) / 2;
        let by = 30i32;

        // Panel fade in
        let panel_alpha = ((f as f32 - 15.0) / 15.0).min(1.0);
        let pv = (panel_alpha * 200.0) as u8;
        let border_col = RGB::from_u8(pv, pv / 5, pv / 5);

        // Draw panel border
        ctx.draw_box(bx, by, bw, bh, border_col, bg_rgb);

        // ── Content reveals char-by-char ─────────────────────────────────────
        let mut pos = 0usize;   // total chars consumed from content budget

        // All content lines in order
        let content: Vec<(&str, String, (u8, u8, u8))> = vec![
            ("title",   "N E M E S I S".to_string(),                    (pv, pv/5, pv/5)),
            ("blank",   String::new(),                                    (0,0,0)),
            ("slayer",  format!("\"Slayer of {}\"", self.player_name),   (pv/2, pv/8, pv/8)),
            ("blank",   String::new(),                                    (0,0,0)),
            ("name",    self.enemy_name.clone(),                          (pv, pv/3, pv/3)),
            ("ability", format!("Ability: {}", self.unique_ability),      (pv/2, pv/4, pv/8)),
            ("stats",   format!("HP: +{}%   DMG: +{}%", self.hp_bonus_pct, self.dmg_bonus_pct), (pv/2, pv/8, pv/8)),
            ("blank",   String::new(),                                    (0,0,0)),
            ("quote",   "\"It has not forgotten.\"".to_string(),          (pv/3, pv/8, pv/8)),
        ];

        let mut line_y = by + 1;
        for (_, text, col) in &content {
            line_y += 1;
            if line_y >= by + bh { break; }
            if text.is_empty() { continue; }

            let show_len = self.chars_shown.saturating_sub(pos).min(text.len());
            pos += text.len() + 1;

            if show_len == 0 { continue; }
            let shown: String = text.chars().take(show_len).collect();
            let center_x = bx + (bw - shown.len() as i32) / 2;
            if center_x > 0 {
                ctx.print_color(center_x, line_y, RGB::from_u8(col.0, col.1, col.2), bg_rgb, &shown);
            }
        }

        // ── Hold phase: particle field behind the box ─────────────────────────
        if f >= 75 {
            // Dark red particles drifting in background
            let hold_t = (f - 75) as f32 / 45.0;
            let pfield_v = (hold_t * 50.0) as u8;
            // Draw a few static "particles" that pulse
            let particle_positions = [
                (bx - 3, by + 2), (bx - 2, by + 5), (bx - 4, by + 8),
                (bx + bw + 2, by + 3), (bx + bw + 3, by + 6), (bx + bw + 2, by + 9),
            ];
            for (px, py) in &particle_positions {
                let pulse = (frame / 6 + (*px as u64)) % 2 == 0;
                let v = if pulse { pfield_v } else { pfield_v / 2 };
                if *px > 0 && *px < 159 && *py > 0 && *py < 79 {
                    ctx.print_color(*px, *py, RGB::from_u8(v, v/5, v/5), bg_rgb, "·");
                }
            }
        }

        // ── Continue hint ─────────────────────────────────────────────────────
        if f >= 100 {
            let hint_alpha = ((f - 100) as f32 / 20.0).min(1.0);
            let hv = (hint_alpha * 70.0) as u8;
            ctx.print_color(bx + 6, by + bh + 1, RGB::from_u8(hv, hv, hv), bg_rgb, "[ Enter ] Begin encounter");
        }
    }
}
