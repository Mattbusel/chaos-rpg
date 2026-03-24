//! Chaos Field — animated mathematical background with 3-layer parallax.
//!
//! Layer 1 (FAR):  Large symbols at slow speed — the deep constants of The Proof
//! Layer 2 (MID):  Mixed digits/operators at medium speed — active computations
//! Layer 3 (NEAR): Small debris at faster speed — computational noise
//!
//! The parallax creates a sense of depth: as you descend into The Proof,
//! the layers accelerate and the character sets grow wilder.

use bracket_lib::prelude::*;

// ── Character sets ────────────────────────────────────────────────────────────

const DIGITS: &[&str] = &["0","1","2","3","4","5","6","7","8","9","+","-","="];

const MATH: &[&str] = &[
    "+","-","=","*","/","~",":",
    "π","φ","∞","Δ","λ","ε","ζ","∑","∂","μ","σ","ω","α","β",
    "∇","∫","∏","Ω","±","≠","≈","∧","∨","√",
];

const CHAOS: &[&str] = &[
    "π","φ","∞","Δ","λ","ε","ζ","∑","∂","μ","σ","ω","α","β",
    "∇","∫","∏","Ω","±","≠","≈","∧","∨","√",
    "?","!","@","#","$","%","^","&","*","|","\\",
    "░","▒","▓","█","▄","▀","▌","▐",
];

// Large math symbols for FAR layer
const FAR_CHARS: &[&str] = &[
    "∫","∑","∏","Ω","∞","∇","∂","φ","π","λ","ζ","Δ",
];

// Small debris for NEAR layer
const NEAR_CHARS: &[&str] = &["·","·","·","·","·","·","·","·","·",",",".","`","'","´","¨"];

// ── Column ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct FieldColumn {
    head_y: f32,
    speed: f32,
    trail: [usize; 8],
    trail_len: usize,
    mutate_timer: u32,
    phase: f32,
}

// ── Layer ─────────────────────────────────────────────────────────────────────

struct ParallaxLayer {
    columns: Vec<FieldColumn>,
    speed_mult: f32,
    opacity: f32,   // base brightness multiplier
    char_set: LayerCharSet,
}

#[derive(Clone, Copy)]
enum LayerCharSet { Far, Mid, Near }

impl ParallaxLayer {
    fn new(cols: usize, speed_mult: f32, opacity: f32, char_set: LayerCharSet, seed_offset: u64) -> Self {
        let chars_len = match char_set {
            LayerCharSet::Far  => FAR_CHARS.len(),
            LayerCharSet::Mid  => MATH.len(),
            LayerCharSet::Near => NEAR_CHARS.len(),
        };
        let mut columns = Vec::with_capacity(cols);
        for col in 0..cols {
            let seed = ((col as u64).wrapping_add(seed_offset))
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let speed = 0.02 + ((seed >> 20) % 100) as f32 * 0.0006;
            let head_y = ((seed >> 10) % 80) as f32;
            let trail_len = 2 + ((seed >> 5) % 5) as usize;
            let phase = col as f32 * 0.17 + seed_offset as f32 * 0.03;
            let mut trail = [0usize; 8];
            for (i, t) in trail.iter_mut().enumerate() {
                *t = ((seed >> i) % chars_len as u64) as usize;
            }
            columns.push(FieldColumn {
                head_y, speed, trail, trail_len,
                mutate_timer: (seed % 50) as u32,
                phase,
            });
        }
        ParallaxLayer { columns, speed_mult, opacity, char_set }
    }

    fn update(&mut self, frame: u64, floor_mult: f32, corruption_f: f32) {
        let char_set_len = match self.char_set {
            LayerCharSet::Far  => FAR_CHARS.len(),
            LayerCharSet::Mid  => MATH.len(),
            LayerCharSet::Near => NEAR_CHARS.len(),
        };
        for (col_i, col) in self.columns.iter_mut().enumerate() {
            let col_seed = col_i as u64 * 11 + frame;
            let mut speed = col.speed * floor_mult * self.speed_mult;
            // Corruption reversal
            if corruption_f > 0.5 {
                let jitter = col_seed % 200;
                if jitter < (corruption_f * 25.0) as u64 { speed *= -1.0; }
            }
            let wobble = (frame as f32 * 0.02 + col.phase).sin() * 0.006;
            col.head_y = (col.head_y + speed + wobble).rem_euclid(80.0);

            if col.mutate_timer > 0 {
                col.mutate_timer -= 1;
            } else {
                let slot = (col_seed % col.trail_len as u64) as usize;
                col.trail[slot] = ((col_seed ^ frame) % char_set_len as u64) as usize;
                col.mutate_timer = 8 + (col_seed % 35) as u32;
            }
        }
    }

    fn draw(&self, ctx: &mut BTerm, bg_rgb: RGB, muted: (u8,u8,u8), accent: (u8,u8,u8),
            base_bright: f32, corruption_f: f32, frame: u64, pulse_x: f32, pulse_active: bool) {
        let char_set: &[&str] = match self.char_set {
            LayerCharSet::Far  => FAR_CHARS,
            LayerCharSet::Mid  => MATH,
            LayerCharSet::Near => NEAR_CHARS,
        };

        for (col_i, col) in self.columns.iter().enumerate() {
            let x = col_i as i32;
            let pulse_boost = if pulse_active {
                let dist = (x as f32 - pulse_x).abs();
                if dist < 5.0 { (1.0 - dist / 5.0) * 0.25 } else { 0.0 }
            } else { 0.0 };

            let head = col.head_y as i32;
            for trail_i in 0..col.trail_len {
                let y = (head - trail_i as i32).rem_euclid(80);
                if y <= 0 || y >= 79 { continue; }

                let trail_frac = 1.0 - (trail_i as f32 / col.trail_len as f32);
                let brightness = (base_bright * self.opacity * trail_frac + pulse_boost) * trail_frac;

                let (r, g, b) = if corruption_f > 0.25 {
                    let tint = ((corruption_f - 0.25) * 1.33).clamp(0.0, 1.0);
                    (
                        (muted.0 as f32 * brightness + accent.0 as f32 * brightness * tint * 0.4) as u8,
                        (muted.1 as f32 * brightness * (1.0 - tint * 0.2)) as u8,
                        (muted.2 as f32 * brightness + accent.2 as f32 * brightness * tint * 0.4) as u8,
                    )
                } else {
                    (
                        (muted.0 as f32 * brightness) as u8,
                        (muted.1 as f32 * brightness) as u8,
                        (muted.2 as f32 * brightness) as u8,
                    )
                };

                // High-corruption glitch flash
                let (r, g, b) = if corruption_f > 0.85 {
                    let glitch = (col_i as u64 * 31 + frame / 4) % 120;
                    if glitch == 0 {
                        (r.max(60).saturating_add(80), g.max(20), b.max(60).saturating_add(60))
                    } else { (r, g, b) }
                } else { (r, g, b) };

                let fg = RGB::from_u8(r.max(2), g.max(2), b.max(2));
                let ch_idx = col.trail[trail_i] % char_set.len();
                ctx.print_color(x, y, fg, bg_rgb, char_set[ch_idx]);
            }
        }
    }
}

// ── ChaosField ────────────────────────────────────────────────────────────────

pub struct ChaosField {
    far:    ParallaxLayer,  // large symbols, slow
    mid:    ParallaxLayer,  // digits/math, medium (original field, halved column count)
    near:   ParallaxLayer,  // tiny debris, faster

    pulse_x:      f32,
    pulse_timer:  u32,
    pulse_active: bool,
}

impl ChaosField {
    pub fn new() -> Self {
        Self {
            // Far layer: 40 evenly-spaced columns of large symbols
            far:  ParallaxLayer::new(40,  0.3,  0.65, LayerCharSet::Far,  0),
            // Mid layer: disabled — was causing distracting character rain
            mid:  ParallaxLayer::new(0,   1.0,  0.0,  LayerCharSet::Mid,  1000),
            // Near layer: 80 columns of debris at 1.5× speed
            near: ParallaxLayer::new(80,  1.55, 0.5,  LayerCharSet::Near, 5000),
            pulse_x:      -1.0,
            pulse_timer:  300,
            pulse_active: false,
        }
    }

    pub fn update(&mut self, frame: u64, floor: u32, corruption: u32) {
        let floor_mult = match floor {
            0..=10  => 1.0f32,
            11..=25 => 1.3,
            26..=50 => 1.7,
            51..=75 => 2.2,
            76..=99 => 2.8,
            _       => 3.5,
        };
        let corruption_f = (corruption as f32 / 400.0).clamp(0.0, 1.0);

        // Corruption: occasionally swap parallax speeds of far/near layers
        let swap_speeds = corruption_f > 0.8 && ((frame / 80) % 5 == 0);

        // Pulse wave
        if self.pulse_active {
            self.pulse_x += 2.5;
            if self.pulse_x > 164.0 {
                self.pulse_active = false;
                self.pulse_x = -1.0;
                self.pulse_timer = 60 * (20 + (frame % 20) as u32);
            }
        } else if self.pulse_timer > 0 {
            self.pulse_timer -= 1;
        } else {
            self.pulse_active = true;
            self.pulse_x = 0.0;
        }

        if swap_speeds {
            // Briefly invert far/near speeds for disorientation
            self.far.speed_mult  = 1.55;
            self.near.speed_mult = 0.3;
        } else {
            self.far.speed_mult  = 0.3;
            self.near.speed_mult = 1.55;
        }

        self.far.update(frame, floor_mult, corruption_f);
        self.mid.update(frame, floor_mult, corruption_f);
        self.near.update(frame, floor_mult, corruption_f);
    }

    pub fn draw(
        &self, ctx: &mut BTerm,
        bg: (u8, u8, u8), muted: (u8, u8, u8), accent: (u8, u8, u8),
        floor: u32, corruption: u32, frame: u64,
    ) {
        let bg_rgb = RGB::from_u8(bg.0, bg.1, bg.2);
        let corruption_f = (corruption as f32 / 400.0).clamp(0.0, 1.0);

        let base_bright: f32 = match floor {
            0..=10  => 0.055,
            11..=25 => 0.070,
            26..=50 => 0.090,
            51..=75 => 0.110,
            76..=99 => 0.135,
            _       => 0.160,
        };

        // Far layer: large symbols at 40% of base brightness
        self.far.draw(ctx, bg_rgb, muted, accent, base_bright * 0.7, corruption_f, frame, self.pulse_x, self.pulse_active);
        // Mid layer: original behavior
        self.mid.draw(ctx, bg_rgb, muted, accent, base_bright, corruption_f, frame, self.pulse_x, self.pulse_active);
        // Near layer: tiny debris at 55% brightness
        self.near.draw(ctx, bg_rgb, muted, accent, base_bright * 0.55, corruption_f, frame, self.pulse_x, self.pulse_active);
    }
}
