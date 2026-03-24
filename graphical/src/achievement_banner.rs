// Achievement banner — CHAOS RPG "Every Action Tells a Story"
//
// 7-tier rarity system with escalating visual drama.
// Replaces the simple text box with rarity-aware display.

use bracket_lib::prelude::*;
use crate::text_effects::draw_rainbow;

// ── Rarity tier ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BannerRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
    Omega,
}

impl BannerRarity {
    /// Color for this tier's primary display.
    pub fn primary_color(&self) -> (u8, u8, u8) {
        match self {
            BannerRarity::Common    => (160, 160, 160),
            BannerRarity::Uncommon  => ( 60, 200,  60),
            BannerRarity::Rare      => ( 60, 120, 255),
            BannerRarity::Epic      => (160,  40, 220),
            BannerRarity::Legendary => (255, 200,   0),
            BannerRarity::Mythic    => (255,  50,  50),
            BannerRarity::Omega     => (255, 140,   0), // base for rainbow
        }
    }

    /// Number of burst particles on unlock.
    pub fn particle_count(&self) -> usize {
        match self {
            BannerRarity::Common    =>  6,
            BannerRarity::Uncommon  => 12,
            BannerRarity::Rare      => 18,
            BannerRarity::Epic      => 24,
            BannerRarity::Legendary => 32,
            BannerRarity::Mythic    => 44,
            BannerRarity::Omega     => 64,
        }
    }

    /// Base display duration in frames (at 30fps).
    pub fn base_frames(&self) -> u32 {
        match self {
            BannerRarity::Common    =>  60,  // 2.0s
            BannerRarity::Uncommon  =>  60,
            BannerRarity::Rare      =>  75,  // 2.5s
            BannerRarity::Epic      =>  75,
            BannerRarity::Legendary =>  90,  // 3.0s
            BannerRarity::Mythic    =>  90,
            BannerRarity::Omega     => 120,  // 4.0s
        }
    }

    /// Whether the achievement name renders typewriter-style.
    pub fn use_typewriter(&self) -> bool {
        matches!(self, BannerRarity::Rare | BannerRarity::Epic
            | BannerRarity::Legendary | BannerRarity::Mythic | BannerRarity::Omega)
    }

    /// Whether to pulse the screen border.
    pub fn pulse_border(&self) -> bool {
        matches!(self, BannerRarity::Epic | BannerRarity::Legendary
            | BannerRarity::Mythic | BannerRarity::Omega)
    }

    /// Whether to briefly dim the screen (spotlight the banner).
    pub fn spotlight(&self) -> bool {
        matches!(self, BannerRarity::Mythic | BannerRarity::Omega)
    }

    pub fn name(&self) -> &'static str {
        match self {
            BannerRarity::Common    => "COMMON",
            BannerRarity::Uncommon  => "UNCOMMON",
            BannerRarity::Rare      => "RARE",
            BannerRarity::Epic      => "EPIC",
            BannerRarity::Legendary => "LEGENDARY",
            BannerRarity::Mythic    => "MYTHIC",
            BannerRarity::Omega     => "OMEGA",
        }
    }
}

// ── AchievementBanner ─────────────────────────────────────────────────────────

pub struct AchievementBanner {
    pub active: bool,
    pub rarity: BannerRarity,
    text: String,
    frames_remaining: u32,
    total_frames: u32,
    typewriter_len: usize,
    typewriter_tick: u32,
    // Particles queued for emission this frame
    pub pending_particles: Vec<BannerParticle>,
    particles_emitted: bool,
}

pub struct BannerParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ch: &'static str,
    pub col: (u8, u8, u8),
    pub lifetime: u32,
}

impl AchievementBanner {
    pub fn new() -> Self {
        Self {
            active: false,
            rarity: BannerRarity::Common,
            text: String::new(),
            frames_remaining: 0,
            total_frames: 0,
            typewriter_len: 0,
            typewriter_tick: 0,
            pending_particles: Vec::new(),
            particles_emitted: false,
        }
    }

    pub fn start(&mut self, text: &str, rarity: BannerRarity, speed: f32) {
        self.active = true;
        self.rarity = rarity;
        self.text = text.chars().take(50).collect();
        let frames = (rarity.base_frames() as f32 / speed.max(0.1)) as u32;
        self.frames_remaining = frames;
        self.total_frames = frames;
        self.typewriter_len = if rarity.use_typewriter() { 0 } else { self.text.len() };
        self.typewriter_tick = 0;
        self.particles_emitted = false;
        self.pending_particles.clear();
    }

    pub fn update(&mut self, frame: u64) {
        if !self.active { return; }
        self.pending_particles.clear();

        // Typewriter advance (2 chars/frame for Rare, 1 char/frame for others)
        if self.typewriter_len < self.text.len() {
            self.typewriter_tick += 1;
            let rate = if matches!(self.rarity, BannerRarity::Omega) { 3 } else { 2 };
            if self.typewriter_tick >= rate {
                self.typewriter_tick = 0;
                self.typewriter_len += 1;
            }
        }

        // Emit particles on first frame
        if !self.particles_emitted {
            self.particles_emitted = true;
            self.emit_unlock_particles(frame);
        }

        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
        } else {
            self.active = false;
        }
    }

    fn emit_unlock_particles(&mut self, frame: u64) {
        use std::f32::consts::TAU;
        let count = self.rarity.particle_count();
        let col = self.rarity.primary_color();
        // Banner is at the top center, y≈1-4
        let cx = 80.0f32;
        let cy = 3.0f32;
        let chars: &[&'static str] = match self.rarity {
            BannerRarity::Common    => &["·", "*"],
            BannerRarity::Uncommon  => &["·", "+", "*"],
            BannerRarity::Rare      => &["✦", "·", "+"],
            BannerRarity::Epic      => &["✦", "★", "·"],
            BannerRarity::Legendary => &["★", "✦", "◉", "·"],
            BannerRarity::Mythic    => &["★", "✦", "◉", "·", "~"],
            BannerRarity::Omega     => &["★", "✦", "◉", "♦", "·", "~", "✧"],
        };
        for i in 0..count {
            let angle = i as f32 * TAU / count as f32;
            let speed = 0.2 + (i % 5) as f32 * 0.08;
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed * 0.5 - 0.1;
            let ch = chars[i % chars.len()];
            let lt = 20 + (i % 15) as u32;
            // Omega: random colors
            let pcol = if self.rarity == BannerRarity::Omega {
                let hue = i as f32 * TAU / count as f32;
                (
                    ((hue.cos() + 1.0) * 0.5 * 200.0 + 55.0) as u8,
                    (((hue + 2.09).cos() + 1.0) * 0.5 * 200.0 + 55.0) as u8,
                    (((hue + 4.19).cos() + 1.0) * 0.5 * 200.0 + 55.0) as u8,
                )
            } else {
                col
            };
            self.pending_particles.push(BannerParticle { x: cx, y: cy, vx, vy, ch, col: pcol, lifetime: lt });
        }
    }

    pub fn draw(&self, ctx: &mut BTerm, bg: (u8, u8, u8), frame: u64) {
        if !self.active { return; }
        let bg_rgb = RGB::from_u8(bg.0, bg.1, bg.2);
        let col = self.rarity.primary_color();

        // Fade in/out
        let alpha = if self.total_frames > 0 {
            let elapsed = self.total_frames - self.frames_remaining;
            let fade_in  = (elapsed as f32 / 20.0).min(1.0);
            let fade_out = (self.frames_remaining as f32 / 30.0).min(1.0);
            fade_in * fade_out
        } else { 1.0 };

        let vr = (col.0 as f32 * alpha) as u8;
        let vg = (col.1 as f32 * alpha) as u8;
        let vb = (col.2 as f32 * alpha) as u8;
        let fade_col = RGB::from_u8(vr.max(4), vg.max(4), vb.max(4));

        // Banner box size
        let shown: String = self.text.chars().take(self.typewriter_len).collect();
        let box_w = (shown.len() as i32 + 6).max(26).min(80);
        let bx = ((160 - box_w) / 2).max(0);
        let by = 1i32;
        let box_h = 4i32;

        // Spotlight: dim background slightly for Mythic+
        if self.rarity.spotlight() && alpha > 0.5 {
            let dim = ((alpha - 0.5) / 0.5 * 30.0) as u8;
            for x in 0..bx {
                for y in 0..8i32 {
                    ctx.print_color(x, y, RGB::from_u8(dim, dim, dim), bg_rgb, " ");
                }
            }
            for x in (bx + box_w + 1)..160i32 {
                for y in 0..8i32 {
                    ctx.print_color(x, y, RGB::from_u8(dim, dim, dim), bg_rgb, " ");
                }
            }
        }

        // Draw box border
        ctx.draw_box(bx, by, box_w, box_h, fade_col, bg_rgb);

        // Tier label on box top edge
        let tier_label = format!(" ACHIEVEMENT: {} ", self.rarity.name());
        let tx = bx + (box_w - tier_label.len() as i32) / 2;
        if tx > 0 {
            ctx.print_color(tx, by, fade_col, bg_rgb, &tier_label);
        }

        // Achievement name — Omega gets rainbow
        let text_x = bx + 3;
        let text_y = by + 2;
        if self.rarity == BannerRarity::Omega {
            draw_rainbow(ctx, text_x, text_y, &shown, bg_rgb, frame);
        } else {
            // Legendary+: slightly larger-feeling via bright heading color
            let name_col = if matches!(self.rarity, BannerRarity::Legendary | BannerRarity::Mythic) {
                RGB::from_u8(
                    (col.0 as f32 * alpha * 1.1).min(255.0) as u8,
                    (col.1 as f32 * alpha * 1.1).min(255.0) as u8,
                    (col.2 as f32 * alpha * 1.1).min(255.0) as u8,
                )
            } else {
                fade_col
            };
            ctx.print_color(text_x, text_y, name_col, bg_rgb, &shown);
        }

        // Border pulse for Epic+: flash the box corners
        if self.rarity.pulse_border() && (frame / 4) % 2 == 0 {
            let pv = (alpha * 255.0) as u8;
            let pulse_col = RGB::from_u8(pv, (pv as f32 * col.1 as f32 / 255.0) as u8, 0);
            ctx.set(bx, by, pulse_col, bg_rgb, 218u16);           // ╔
            ctx.set(bx + box_w, by, pulse_col, bg_rgb, 191u16);   // ╗
            ctx.set(bx, by + box_h, pulse_col, bg_rgb, 192u16);   // ╚
            ctx.set(bx + box_w, by + box_h, pulse_col, bg_rgb, 217u16); // ╝
        }

        // Omega: extra glow line below banner
        if self.rarity == BannerRarity::Omega && alpha > 0.6 {
            let glow_y = by + box_h + 1;
            if glow_y < 79 {
                draw_rainbow(ctx, bx, glow_y,
                    &"─".repeat(box_w as usize).chars().take(box_w as usize).collect::<String>(),
                    bg_rgb, frame);
            }
        }
    }
}

/// Map a string rarity name (from achievement core) to BannerRarity.
pub fn rarity_from_name(name: &str) -> BannerRarity {
    match name.to_lowercase().as_str() {
        "uncommon" => BannerRarity::Uncommon,
        "rare"     => BannerRarity::Rare,
        "epic"     => BannerRarity::Epic,
        "legendary"=> BannerRarity::Legendary,
        "mythic"   => BannerRarity::Mythic,
        "omega"    => BannerRarity::Omega,
        _          => BannerRarity::Common,
    }
}
