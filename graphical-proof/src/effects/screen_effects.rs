//! Post-processing-style effects rendered as glyph overlays.
//!
//! These simulate common screen-space effects (CRT scanlines, chromatic aberration,
//! film grain, etc.) using immediate-mode glyph rendering on the Overlay layer.
//! Camera at (0,0,-10) looking at origin. Visible area at z=0: +/-8.7 x, +/-5.4 y.

use proof_engine::prelude::*;

// ---------------------------------------------------------------------------
// Hash helper
// ---------------------------------------------------------------------------

fn hash_f32(seed: u32) -> f32 {
    let mut s = seed;
    s ^= s >> 16;
    s = s.wrapping_mul(0x45d9f3b);
    s ^= s >> 16;
    (s & 0x00FF_FFFF) as f32 / 16_777_215.0
}

fn hash_range(seed: u32, lo: f32, hi: f32) -> f32 {
    lo + hash_f32(seed) * (hi - lo)
}

// ---------------------------------------------------------------------------
// CRT Scanline effect
// ---------------------------------------------------------------------------

pub struct CrtScanlines {
    pub active: bool,
    pub line_alpha: f32,
    pub color_shift: f32,
}

impl CrtScanlines {
    pub fn new() -> Self {
        Self {
            active: false,
            line_alpha: 0.15,
            color_shift: 0.02,
        }
    }

    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active {
            return;
        }

        // Horizontal scanlines at regular intervals
        let line_spacing = 0.5;
        let scroll = (frame as f32 * 0.005) % line_spacing;
        let mut y = -5.5 + scroll;
        while y < 5.5 {
            for xi in 0..20 {
                let x = -9.5 + xi as f32;
                engine.spawn_glyph(Glyph {
                    character: '\u{2500}',
                    position: Vec3::new(x, y, 0.88),
                    color: Vec4::new(0.0, 0.0, 0.0, self.line_alpha),
                    scale: Vec2::new(1.0, 0.08),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Multiply,
                    ..Default::default()
                });
            }
            y += line_spacing;
        }

        // Slight color fringing at screen edges
        if self.color_shift > 0.001 {
            let shift = self.color_shift;
            // Red channel shifted left, blue shifted right (simplified)
            for yi in 0..6 {
                let y_pos = -5.0 + yi as f32 * 2.0;
                // Left edge: red tint
                engine.spawn_glyph(Glyph {
                    character: '\u{2588}',
                    position: Vec3::new(-9.0 - shift * 10.0, y_pos, 0.87),
                    color: Vec4::new(1.0, 0.0, 0.0, shift * 3.0),
                    scale: Vec2::new(0.3, 2.0),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
                // Right edge: blue tint
                engine.spawn_glyph(Glyph {
                    character: '\u{2588}',
                    position: Vec3::new(9.0 + shift * 10.0, y_pos, 0.87),
                    color: Vec4::new(0.0, 0.0, 1.0, shift * 3.0),
                    scale: Vec2::new(0.3, 2.0),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Chromatic aberration
// ---------------------------------------------------------------------------

pub struct ChromaticAberration {
    pub active: bool,
    pub intensity: f32, // world-unit offset
}

impl ChromaticAberration {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.08,
        }
    }

    /// Render RGB-split ghost copies at slight offsets.
    /// This works best when called with specific text/glyph positions to duplicate.
    /// For a global effect we scatter colored dots at offset positions.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active || self.intensity < 0.001 {
            return;
        }

        let offset = self.intensity;
        let dot_count = 40;

        for i in 0..dot_count {
            let seed = (frame as u32).wrapping_mul(6173).wrapping_add(i);
            let x = hash_range(seed, -8.0, 8.0);
            let y = hash_range(seed.wrapping_add(1), -4.5, 4.5);

            // Red copy offset left
            engine.spawn_glyph(Glyph {
                character: '\u{2591}',
                position: Vec3::new(x - offset, y, 0.86),
                color: Vec4::new(1.0, 0.0, 0.0, 0.12),
                scale: Vec2::splat(0.3),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
            // Blue copy offset right
            engine.spawn_glyph(Glyph {
                character: '\u{2591}',
                position: Vec3::new(x + offset, y, 0.86),
                color: Vec4::new(0.0, 0.0, 1.0, 0.12),
                scale: Vec2::splat(0.3),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Film grain
// ---------------------------------------------------------------------------

pub struct FilmGrain {
    pub active: bool,
    pub density: f32, // 0..1
    pub alpha: f32,
}

impl FilmGrain {
    pub fn new() -> Self {
        Self {
            active: false,
            density: 0.3,
            alpha: 0.08,
        }
    }

    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active {
            return;
        }

        let total = (240.0 * self.density) as usize;
        for i in 0..total {
            // Different seed each frame for noisy refresh
            let seed = (frame as u32).wrapping_mul(9029).wrapping_add(i as u32);
            let x = hash_range(seed, -9.0, 9.0);
            let y = hash_range(seed.wrapping_add(1), -5.5, 5.5);
            let bright = hash_f32(seed.wrapping_add(2));

            engine.spawn_glyph(Glyph {
                character: '.',
                position: Vec3::new(x, y, 0.82),
                color: Vec4::new(bright, bright, bright, self.alpha),
                scale: Vec2::splat(0.08),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Heat shimmer
// ---------------------------------------------------------------------------

pub struct HeatShimmer {
    pub active: bool,
    pub intensity: f32,
    pub y_floor: f32, // vertical threshold below which shimmer appears
}

impl HeatShimmer {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.5,
            y_floor: -3.0,
        }
    }

    /// Render wavy distortion glyphs near the floor (fire/lava areas).
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active || self.intensity < 0.01 {
            return;
        }

        let rows = 4;
        let cols = 20;
        for yi in 0..rows {
            let base_y = self.y_floor + yi as f32 * 0.6;
            for xi in 0..cols {
                let x = -9.5 + xi as f32;
                let wave = (frame as f32 * 0.04 + x * 0.8 + yi as f32 * 1.2).sin();
                let offset_y = wave * self.intensity * 0.2;
                let alpha = (1.0 - (yi as f32 / rows as f32)) * self.intensity * 0.15;

                engine.spawn_glyph(Glyph {
                    character: '\u{2591}',
                    position: Vec3::new(x, base_y + offset_y, 0.3),
                    color: Vec4::new(1.0, 0.8, 0.5, alpha),
                    scale: Vec2::new(1.0, 0.5),
                    emission: alpha * 0.5,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Underwater effect
// ---------------------------------------------------------------------------

pub struct UnderwaterEffect {
    pub active: bool,
    pub depth: f32, // 0..1 (deeper = more blue, more distortion)
}

impl UnderwaterEffect {
    pub fn new() -> Self {
        Self {
            active: false,
            depth: 0.4,
        }
    }

    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active {
            return;
        }

        // Blue tint overlay
        let tint_alpha = 0.1 + self.depth * 0.15;
        for xi in 0..10 {
            for yi in 0..6 {
                let x = -9.0 + xi as f32 * 2.0;
                let y = -5.0 + yi as f32 * 2.0;
                let wave = (frame as f32 * 0.02 + x * 0.3 + y * 0.5).sin() * 0.03;
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y + wave, 0.8),
                    color: Vec4::new(0.1, 0.2, 0.6, tint_alpha),
                    scale: Vec2::splat(2.0),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        // Rising bubbles
        let bubble_count = (15.0 * self.depth + 5.0) as usize;
        for i in 0..bubble_count {
            let seed = i as u32 * 2311;
            let base_x = hash_range(seed, -8.0, 8.0);
            let speed = hash_range(seed.wrapping_add(1), 1.0, 3.0);
            let phase = hash_f32(seed.wrapping_add(2));
            let raw_y = -5.0 + ((frame as f32 * 0.016 * speed + phase * 15.0) % 11.0);
            let wobble = (frame as f32 * 0.03 + phase * 6.28).sin() * 0.3;
            let alpha = 0.3 + hash_f32(seed.wrapping_add(3)) * 0.3;

            engine.spawn_glyph(Glyph {
                character: 'o',
                position: Vec3::new(base_x + wobble, raw_y, 0.15),
                color: Vec4::new(0.5, 0.7, 1.0, alpha),
                scale: Vec2::splat(hash_range(seed.wrapping_add(4), 0.08, 0.18)),
                emission: 0.3,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }

        // Wavy distortion lines
        for yi in 0..8 {
            let y = -4.0 + yi as f32 * 1.2;
            let wave_phase = frame as f32 * 0.015 + yi as f32 * 0.8;
            for xi in 0..20 {
                let x = -9.5 + xi as f32;
                let wave_offset = (wave_phase + x * 0.4).sin() * self.depth * 0.15;
                let alpha = self.depth * 0.06;

                engine.spawn_glyph(Glyph {
                    character: '~',
                    position: Vec3::new(x, y + wave_offset, 0.75),
                    color: Vec4::new(0.3, 0.5, 0.8, alpha),
                    scale: Vec2::new(0.5, 0.15),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Time slow (bullet time)
// ---------------------------------------------------------------------------

pub struct TimeSlowEffect {
    pub active: bool,
    pub intensity: f32, // 0..1
}

impl TimeSlowEffect {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.6,
        }
    }

    /// Desaturation overlay + motion trail ghost copies.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active || self.intensity < 0.01 {
            return;
        }

        // Desaturation: grey overlay with multiply blend
        let grey_alpha = self.intensity * 0.2;
        for xi in 0..10 {
            for yi in 0..6 {
                let x = -9.0 + xi as f32 * 2.0;
                let y = -5.0 + yi as f32 * 2.0;
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y, 0.78),
                    color: Vec4::new(0.5, 0.5, 0.5, grey_alpha),
                    scale: Vec2::splat(2.0),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Multiply,
                    ..Default::default()
                });
            }
        }

        // Motion trail: ghost copies of a central area, trailing behind
        let trail_count = (self.intensity * 5.0) as usize;
        for trail in 1..=trail_count {
            let trail_alpha = (1.0 - trail as f32 / (trail_count + 1) as f32) * 0.15 * self.intensity;
            let offset_x = trail as f32 * 0.08;

            for i in 0..8 {
                let seed = (frame as u32).wrapping_mul(1117).wrapping_add(i);
                let x = hash_range(seed, -6.0, 6.0) - offset_x;
                let y = hash_range(seed.wrapping_add(1), -3.0, 3.0);

                engine.spawn_glyph(Glyph {
                    character: '\u{2591}',
                    position: Vec3::new(x, y, 0.76),
                    color: Vec4::new(0.6, 0.6, 0.7, trail_alpha),
                    scale: Vec2::splat(0.4),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rage mode
// ---------------------------------------------------------------------------

pub struct RageModeEffect {
    pub active: bool,
    pub intensity: f32,
}

impl RageModeEffect {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.7,
        }
    }

    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active || self.intensity < 0.01 {
            return;
        }

        // Red tint overlay
        let pulse = ((frame as f32 * 0.1).sin() * 0.15 + 0.85).max(0.0);
        let tint_alpha = self.intensity * 0.12 * pulse;
        for xi in 0..10 {
            for yi in 0..6 {
                let x = -9.0 + xi as f32 * 2.0;
                let y = -5.0 + yi as f32 * 2.0;
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y, 0.83),
                    color: Vec4::new(0.8, 0.05, 0.0, tint_alpha),
                    scale: Vec2::splat(2.0),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        // Screen edge pulsing
        let edge_alpha = self.intensity * 0.3 * pulse;
        for yi in 0..12 {
            let y = -5.5 + yi as f32;
            // Left edge
            engine.spawn_glyph(Glyph {
                character: '\u{2588}',
                position: Vec3::new(-9.0, y, 0.84),
                color: Vec4::new(0.9, 0.1, 0.0, edge_alpha),
                scale: Vec2::new(0.5, 1.0),
                emission: edge_alpha * 1.5,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
            // Right edge
            engine.spawn_glyph(Glyph {
                character: '\u{2588}',
                position: Vec3::new(9.0, y, 0.84),
                color: Vec4::new(0.9, 0.1, 0.0, edge_alpha),
                scale: Vec2::new(0.5, 1.0),
                emission: edge_alpha * 1.5,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }

        // Speed lines converging toward center
        let line_count = (self.intensity * 16.0) as usize;
        for i in 0..line_count {
            let seed = (frame as u32).wrapping_mul(4517).wrapping_add(i as u32);
            let angle = hash_range(seed, 0.0, std::f32::consts::TAU);
            let r = hash_range(seed.wrapping_add(1), 5.0, 9.0);
            let len = hash_range(seed.wrapping_add(2), 0.5, 2.0);

            let x = angle.cos() * r;
            let y = angle.sin() * r;
            let dx = -angle.cos() * len;
            let dy = -angle.sin() * len;

            // Draw 3-segment line toward center
            for s in 0..3 {
                let t = s as f32 / 3.0;
                let lx = x + dx * t;
                let ly = y + dy * t;
                if lx.abs() > 9.5 || ly.abs() > 5.5 { continue; }
                let alpha = (1.0 - t) * self.intensity * 0.3;

                engine.spawn_glyph(Glyph {
                    character: '-',
                    position: Vec3::new(lx, ly, 0.83),
                    color: Vec4::new(1.0, 0.3, 0.1, alpha),
                    scale: Vec2::new(0.3, 0.1),
                    rotation: angle,
                    emission: alpha,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Low HP warning
// ---------------------------------------------------------------------------

pub struct LowHpWarning {
    pub active: bool,
    pub hp_fraction: f32, // 0..1 (lower = more intense)
}

impl LowHpWarning {
    pub fn new() -> Self {
        Self {
            active: false,
            hp_fraction: 1.0,
        }
    }

    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active || self.hp_fraction > 0.35 {
            return;
        }

        let danger = 1.0 - (self.hp_fraction / 0.35); // 0 at 35% hp, 1 at 0%

        // Heartbeat-paced red vignette pulse
        let heartbeat_bpm = 80.0 + danger * 60.0; // faster as hp drops
        let heartbeat_period = 60.0 / heartbeat_bpm;
        let beat_phase = (frame as f32 * 0.016 / heartbeat_period).fract();
        // Sharp pulse at start of beat, quick decay
        let beat_strength = if beat_phase < 0.15 {
            (beat_phase / 0.15).powi(2)
        } else if beat_phase < 0.3 {
            1.0 - ((beat_phase - 0.15) / 0.15)
        } else {
            0.0
        };

        let vignette_alpha = danger * 0.3 * (0.3 + beat_strength * 0.7);

        // Red vignette
        for xi in 0..20 {
            for yi in 0..12 {
                let x = -9.5 + xi as f32;
                let y = -5.5 + yi as f32;
                let dx = x / 9.5;
                let dy = y / 5.5;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < 0.5 { continue; }

                let falloff = ((dist - 0.5) / 0.5).min(1.0);
                let alpha = falloff * vignette_alpha;

                engine.spawn_glyph(Glyph {
                    character: '\u{2588}',
                    position: Vec3::new(x, y, 0.89),
                    color: Vec4::new(0.7, 0.0, 0.0, alpha),
                    scale: Vec2::splat(1.0),
                    emission: alpha * beat_strength,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        // Blood drip glyphs at top edge
        if danger > 0.3 {
            let drip_count = (danger * 12.0) as usize;
            for i in 0..drip_count {
                let seed = i as u32 * 2999;
                let x = hash_range(seed, -8.0, 8.0);
                let drip_len = hash_range(seed.wrapping_add(1), 0.3, 1.5) * danger;
                let drip_y = 5.3 - (frame as f32 * 0.01 + hash_f32(seed.wrapping_add(2)) * 5.0) % (drip_len + 0.5);
                let alpha = danger * 0.4;

                engine.spawn_glyph(Glyph {
                    character: '|',
                    position: Vec3::new(x, drip_y, 0.88),
                    color: Vec4::new(0.6, 0.0, 0.0, alpha),
                    scale: Vec2::new(0.1, 0.3),
                    emission: 0.1,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Corruption overlay
// ---------------------------------------------------------------------------

pub struct CorruptionOverlay {
    pub active: bool,
    pub corruption_level: f32, // 0..1
}

impl CorruptionOverlay {
    pub fn new() -> Self {
        Self {
            active: false,
            corruption_level: 0.0,
        }
    }

    /// Purple noise creeping from screen edges, intensity scales with corruption.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active || self.corruption_level < 0.05 {
            return;
        }

        let intensity = self.corruption_level;
        let reach = intensity * 5.0; // How far inward from edges
        let noise_count = (intensity * 60.0) as usize;
        let noise_chars = ['\u{2591}', '\u{2592}', '\u{2593}', '.', ',', '\''];

        for i in 0..noise_count {
            // Refresh each frame for noisy appearance
            let seed = (frame as u32).wrapping_mul(7331).wrapping_add(i as u32);
            let edge = (hash_f32(seed) * 4.0) as u32;
            let t = hash_f32(seed.wrapping_add(1));
            let depth = hash_f32(seed.wrapping_add(2)) * reach;

            let (x, y) = match edge {
                0 => (-8.7 + depth, -5.4 + t * 10.8),
                1 => (8.7 - depth, -5.4 + t * 10.8),
                2 => (-8.7 + t * 17.4, 5.4 - depth),
                _ => (-8.7 + t * 17.4, -5.4 + depth),
            };

            let fade = 1.0 - (depth / reach.max(0.01)).min(1.0);
            let alpha = fade * fade * intensity * 0.5;
            let ci = ((frame as usize + i) * 3) % noise_chars.len();

            let pulse = ((frame as f32 * 0.06 + i as f32 * 0.9).sin() * 0.3 + 0.7).max(0.0);

            engine.spawn_glyph(Glyph {
                character: noise_chars[ci],
                position: Vec3::new(x, y, 0.85),
                color: Vec4::new(0.45, 0.05, 0.65, alpha * pulse),
                scale: Vec2::splat(hash_range(seed.wrapping_add(3), 0.15, 0.5)),
                emission: alpha * 0.6,
                glow_color: Vec3::new(0.4, 0.0, 0.6),
                glow_radius: 0.3,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Boss aura
// ---------------------------------------------------------------------------

pub struct BossAura {
    pub active: bool,
    pub color: Vec4,
    pub radius: f32,
    pub position: Vec3,
}

impl BossAura {
    pub fn new() -> Self {
        Self {
            active: false,
            color: Vec4::new(1.0, 0.3, 0.2, 0.6),
            radius: 2.5,
            position: Vec3::new(5.0, 0.0, 0.0),
        }
    }

    /// Set aura color by element name.
    pub fn set_element(&mut self, element: &str) {
        self.color = match element.to_lowercase().as_str() {
            "fire" | "burn" => Vec4::new(1.0, 0.4, 0.1, 0.6),
            "ice" | "frost" => Vec4::new(0.3, 0.7, 1.0, 0.6),
            "lightning" | "shock" => Vec4::new(1.0, 1.0, 0.4, 0.7),
            "poison" | "necrotic" => Vec4::new(0.2, 0.8, 0.2, 0.5),
            "arcane" | "chaos" => Vec4::new(0.6, 0.2, 1.0, 0.6),
            "dark" | "void" => Vec4::new(0.2, 0.0, 0.3, 0.5),
            "holy" | "divine" => Vec4::new(1.0, 0.9, 0.4, 0.7),
            _ => Vec4::new(0.8, 0.3, 0.3, 0.5),
        };
    }

    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active {
            return;
        }

        let ring_points = 24;
        let pulse = ((frame as f32 * 0.05).sin() * 0.15 + 0.85).max(0.0);

        for i in 0..ring_points {
            let angle = (i as f32 / ring_points as f32) * std::f32::consts::TAU
                + frame as f32 * 0.01;
            let r = self.radius + (frame as f32 * 0.03 + i as f32 * 0.5).sin() * 0.2;
            let x = self.position.x + angle.cos() * r;
            let y = self.position.y + angle.sin() * r * 0.5; // squash for perspective
            let alpha = self.color.w * pulse;

            engine.spawn_glyph(Glyph {
                character: '\u{00b7}',
                position: Vec3::new(x, y, self.position.z - 0.1),
                color: Vec4::new(self.color.x, self.color.y, self.color.z, alpha),
                scale: Vec2::splat(0.2),
                emission: alpha * 1.5,
                glow_color: Vec3::new(self.color.x, self.color.y, self.color.z),
                glow_radius: 0.8,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }

        // Inner glow ring (brighter, smaller)
        for i in 0..12 {
            let angle = (i as f32 / 12.0) * std::f32::consts::TAU
                - frame as f32 * 0.02;
            let r = self.radius * 0.6;
            let x = self.position.x + angle.cos() * r;
            let y = self.position.y + angle.sin() * r * 0.5;
            let alpha = self.color.w * 0.4 * pulse;

            engine.spawn_glyph(Glyph {
                character: '\u{2726}',
                position: Vec3::new(x, y, self.position.z - 0.05),
                color: Vec4::new(
                    (self.color.x + 0.3).min(1.0),
                    (self.color.y + 0.3).min(1.0),
                    (self.color.z + 0.3).min(1.0),
                    alpha,
                ),
                scale: Vec2::splat(0.15),
                emission: alpha * 2.0,
                glow_color: Vec3::new(self.color.x, self.color.y, self.color.z),
                glow_radius: 1.2,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Master screen effects controller
// ---------------------------------------------------------------------------

/// Bundles all screen-level post-processing effects.
pub struct ScreenEffects {
    pub crt_scanlines: CrtScanlines,
    pub chromatic_aberration: ChromaticAberration,
    pub film_grain: FilmGrain,
    pub heat_shimmer: HeatShimmer,
    pub underwater: UnderwaterEffect,
    pub time_slow: TimeSlowEffect,
    pub rage_mode: RageModeEffect,
    pub low_hp_warning: LowHpWarning,
    pub corruption_overlay: CorruptionOverlay,
    pub boss_aura: BossAura,
}

impl ScreenEffects {
    pub fn new() -> Self {
        Self {
            crt_scanlines: CrtScanlines::new(),
            chromatic_aberration: ChromaticAberration::new(),
            film_grain: FilmGrain::new(),
            heat_shimmer: HeatShimmer::new(),
            underwater: UnderwaterEffect::new(),
            time_slow: TimeSlowEffect::new(),
            rage_mode: RageModeEffect::new(),
            low_hp_warning: LowHpWarning::new(),
            corruption_overlay: CorruptionOverlay::new(),
            boss_aura: BossAura::new(),
        }
    }

    /// Render all active screen effects.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        self.crt_scanlines.render(engine, frame);
        self.chromatic_aberration.render(engine, frame);
        self.film_grain.render(engine, frame);
        self.heat_shimmer.render(engine, frame);
        self.underwater.render(engine, frame);
        self.time_slow.render(engine, frame);
        self.rage_mode.render(engine, frame);
        self.low_hp_warning.render(engine, frame);
        self.corruption_overlay.render(engine, frame);
        self.boss_aura.render(engine, frame);
    }

    /// Disable all effects.
    pub fn disable_all(&mut self) {
        self.crt_scanlines.active = false;
        self.chromatic_aberration.active = false;
        self.film_grain.active = false;
        self.heat_shimmer.active = false;
        self.underwater.active = false;
        self.time_slow.active = false;
        self.rage_mode.active = false;
        self.low_hp_warning.active = false;
        self.corruption_overlay.active = false;
        self.boss_aura.active = false;
    }
}
