//! Chaos Field — animated mathematical background present on all screens.
//!
//! The Proof is always computing. This makes it visible.
//! ~800 character draws per frame. Trivial for OpenGL.

use bracket_lib::prelude::*;

// ── Character sets ────────────────────────────────────────────────────────────

const DIGITS: &[&str] = &[
    "0","1","2","3","4","5","6","7","8","9","+","-","=",
];

const MATH: &[&str] = &[
    "0","1","2","3","4","5","6","7","8","9",
    "+","-","=","*","/","~","#","%",":",
    "π","φ","∞","Δ","λ","ε","ζ","∑","∂","μ","σ","ω","α","β",
    "∇","∫","∏","Ω","±","≠","≈","∧","∨","√",
];

const CHAOS: &[&str] = &[
    "0","1","2","3","4","5","6","7","8","9",
    "π","φ","∞","Δ","λ","ε","ζ","∑","∂","μ","σ","ω","α","β",
    "∇","∫","∏","Ω","±","≠","≈","∧","∨","√",
    "?","!","@","#","$","%","^","&","*","|","\\",
    "░","▒","▓","█","▄","▀","▌","▐",
];

// ── Column ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct FieldColumn {
    /// Fractional Y position of the head character (0.0 = top, 79.0 = bottom)
    head_y: f32,
    /// Scroll speed in tiles per frame (positive = downward)
    speed: f32,
    /// Trail characters (char set index for each trail position)
    trail: [usize; 8],
    /// Trail length (3-8)
    trail_len: usize,
    /// Frames until next character mutation
    mutate_timer: u32,
    /// Phase offset for column-level sine wobble
    phase: f32,
}

// ── Chaos Field ───────────────────────────────────────────────────────────────

pub struct ChaosField {
    columns: Vec<FieldColumn>,
    /// Pulse wave X position (-1 = inactive)
    pulse_x: f32,
    /// Frames until pulse starts (or re-starts)
    pulse_timer: u32,
    /// Whether a pulse is currently sweeping
    pulse_active: bool,
}

impl ChaosField {
    pub fn new() -> Self {
        let mut columns = Vec::with_capacity(160);
        for col in 0..160usize {
            // Deterministic per-column seed
            let seed = (col as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let speed = 0.03 + ((seed >> 20) % 100) as f32 * 0.0008; // 0.03-0.11 tiles/frame
            let head_y = ((seed >> 10) % 80) as f32;
            let trail_len = 3 + ((seed >> 5) % 6) as usize; // 3-8
            let phase = col as f32 * 0.13;
            let mut trail = [0usize; 8];
            for (i, t) in trail.iter_mut().enumerate() {
                *t = ((seed >> i) % MATH.len() as u64) as usize;
            }
            columns.push(FieldColumn {
                head_y, speed, trail, trail_len,
                mutate_timer: (seed % 40) as u32,
                phase,
            });
        }
        ChaosField {
            columns,
            pulse_x: -1.0,
            pulse_timer: 300, // first pulse at 5s
            pulse_active: false,
        }
    }

    /// Update field state. Called once per frame in `tick()`.
    pub fn update(&mut self, frame: u64, floor: u32, corruption: u32) {
        // Global speed multiplier from floor depth
        let floor_mult = match floor {
            0..=10  => 1.0f32,
            11..=25 => 1.3,
            26..=50 => 1.7,
            51..=75 => 2.2,
            76..=99 => 2.8,
            _       => 3.5,
        };

        let corruption_f = (corruption as f32 / 400.0).clamp(0.0, 1.0);

        // ── Pulse wave ────────────────────────────────────────────────────────
        if self.pulse_active {
            self.pulse_x += 2.5;
            if self.pulse_x > 164.0 {
                self.pulse_active = false;
                self.pulse_x = -1.0;
                // Next pulse: 20-40 seconds
                self.pulse_timer = 60 * (20 + (frame % 20) as u32);
            }
        } else if self.pulse_timer > 0 {
            self.pulse_timer -= 1;
        } else {
            self.pulse_active = true;
            self.pulse_x = 0.0;
        }

        // ── Columns ───────────────────────────────────────────────────────────
        for (col_i, col) in self.columns.iter_mut().enumerate() {
            let col_seed = col_i as u64 * 7 + frame;

            // Corruption jitter: erratic speed/direction at high corruption
            let mut speed = col.speed * floor_mult;
            if corruption_f > 0.5 {
                let jitter_roll = col_seed % 200;
                if jitter_roll < (corruption_f * 30.0) as u64 {
                    speed *= -1.0; // briefly reverse
                }
            }
            // Sine wave modulation so the field "breathes"
            let wobble = (frame as f32 * 0.02 + col.phase).sin() * 0.008;
            col.head_y = (col.head_y + speed + wobble).rem_euclid(80.0);

            // Character mutation
            if col.mutate_timer > 0 {
                col.mutate_timer -= 1;
            } else {
                let char_set = if corruption_f > 0.5 || floor > 75 {
                    CHAOS
                } else if floor > 25 || corruption_f > 0.2 {
                    MATH
                } else {
                    DIGITS
                };
                let trail_slot = (col_seed % col.trail_len as u64) as usize;
                col.trail[trail_slot] = ((col_seed ^ frame) % char_set.len() as u64) as usize;
                col.mutate_timer = 6 + (col_seed % 30) as u32;
            }
        }
    }

    /// Draw the chaos field. Call AFTER `ctx.cls_bg` and BEFORE UI panels.
    pub fn draw(
        &self,
        ctx: &mut BTerm,
        bg: (u8, u8, u8),
        muted: (u8, u8, u8),
        accent: (u8, u8, u8),
        floor: u32,
        corruption: u32,
        frame: u64,
    ) {
        let bg_rgb = RGB::from_u8(bg.0, bg.1, bg.2);
        let corruption_f = (corruption as f32 / 400.0).clamp(0.0, 1.0);

        // Base brightness: very low so it stays behind UI
        let base_bright: f32 = match floor {
            0..=10  => 0.055,
            11..=25 => 0.070,
            26..=50 => 0.090,
            51..=75 => 0.110,
            76..=99 => 0.135,
            _       => 0.160,
        };

        // Character set selection (for rendering; index stored in trail)
        let char_set: &[&str] = if corruption_f > 0.5 || floor > 75 {
            CHAOS
        } else if floor > 25 || corruption_f > 0.2 {
            MATH
        } else {
            DIGITS
        };

        for (col_i, col) in self.columns.iter().enumerate() {
            let x = col_i as i32;

            // Pulse boost: column near the sweep front gets a brightness spike
            let pulse_boost = if self.pulse_active {
                let dist = (x as f32 - self.pulse_x).abs();
                if dist < 4.0 { (1.0 - dist / 4.0) * 0.30 } else { 0.0 }
            } else {
                0.0
            };

            let head = col.head_y as i32;

            for trail_i in 0..col.trail_len {
                // Trail position: head is brightest, tail fades
                let y = (head - trail_i as i32).rem_euclid(80);
                if y <= 0 || y >= 79 { continue; } // keep off the outer border

                let trail_frac = 1.0 - (trail_i as f32 / col.trail_len as f32);
                let brightness = (base_bright * trail_frac + pulse_boost) * trail_frac;

                // Very slight corruption accent tint at high corruption
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

                // At very high corruption: occasional inversion (bright flash)
                let (r, g, b) = if corruption_f > 0.85 {
                    let glitch = (col_i as u64 * 31 + frame / 4) % 120;
                    if glitch == 0 {
                        (r.max(60).saturating_add(80), g.max(20), b.max(60).saturating_add(60))
                    } else {
                        (r, g, b)
                    }
                } else {
                    (r, g, b)
                };

                let fg = RGB::from_u8(r.max(2), g.max(2), b.max(2));
                let ch_idx = col.trail[trail_i] % char_set.len();
                let ch = char_set[ch_idx];

                ctx.print_color(x, y, fg, bg_rgb, ch);
            }
        }
    }
}
