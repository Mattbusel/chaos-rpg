//! Environmental visual effects — weather, atmosphere, screen-level overlays.
//!
//! All effects are rendered as immediate-mode glyphs (cleared each frame).
//! Camera at (0,0,-10) looking at origin. Visible area at z=0: +/-8.7 x, +/-5.4 y.

use proof_engine::prelude::*;

// ---------------------------------------------------------------------------
// Helper: pseudo-random from seed (deterministic, no crate dependency)
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
// Rain system
// ---------------------------------------------------------------------------

/// Persistent rain state.
pub struct RainSystem {
    pub active: bool,
    pub intensity: f32, // 0..1  (1 = downpour)
    /// Accumulated time for lightning cadence.
    lightning_timer: f32,
    /// Remaining flash brightness (decays each frame).
    flash_brightness: f32,
}

impl RainSystem {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.6,
            lightning_timer: 0.0,
            flash_brightness: 0.0,
        }
    }

    /// Render rain drops, splashes, and occasional lightning flash.
    pub fn render(&mut self, engine: &mut ProofEngine, dt: f32, frame: u64) {
        if !self.active {
            return;
        }

        let drop_count = (200.0 * self.intensity) as usize;

        // ── Falling raindrops ──
        for i in 0..drop_count {
            let seed = (frame as u32).wrapping_mul(7919).wrapping_add(i as u32);
            let x = hash_range(seed, -9.0, 9.0);
            // Vertical position cycles fast so rain looks like it's falling
            let speed = hash_range(seed.wrapping_add(1), 8.0, 14.0);
            let phase = hash_f32(seed.wrapping_add(2));
            let raw_y = 6.0 - ((frame as f32 * 0.016 * speed + phase * 20.0) % 12.0);
            let y = raw_y;
            let alpha = hash_range(seed.wrapping_add(3), 0.15, 0.45) * self.intensity;

            engine.spawn_glyph(Glyph {
                character: '|',
                position: Vec3::new(x, y, 0.2),
                color: Vec4::new(0.55, 0.6, 0.75, alpha),
                scale: Vec2::new(0.15, 0.35),
                emission: 0.05,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });

            // ── Splash on floor ──
            if raw_y <= -4.8 {
                let splash_alpha = alpha * 0.6;
                engine.spawn_glyph(Glyph {
                    character: '.',
                    position: Vec3::new(x, -5.0, 0.2),
                    color: Vec4::new(0.55, 0.6, 0.8, splash_alpha),
                    scale: Vec2::splat(0.2),
                    emission: 0.02,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }

        // ── Lightning ──
        self.lightning_timer += dt;
        if self.lightning_timer > 4.0 + hash_range(frame as u32, 0.0, 6.0) * (1.0 - self.intensity * 0.5) {
            self.lightning_timer = 0.0;
            self.flash_brightness = 1.0;
        }
        if self.flash_brightness > 0.01 {
            self.render_lightning_flash(engine);
            self.flash_brightness *= 0.7; // fast decay
        }
    }

    fn render_lightning_flash(&self, engine: &mut ProofEngine) {
        let a = self.flash_brightness * 0.6;
        // Full-screen white overlay
        for xi in 0..20 {
            for yi in 0..12 {
                let x = -9.5 + xi as f32;
                let y = -5.5 + yi as f32;
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y, 0.9),
                    color: Vec4::new(0.9, 0.9, 1.0, a),
                    scale: Vec2::splat(1.0),
                    emission: a * 2.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
        // Lightning bolt line
        let bolt_chars = ['/', '\\', '|', '/', '\\', '|', '/'];
        for (i, &ch) in bolt_chars.iter().enumerate() {
            let x = hash_range(i as u32 * 31 + 7, -3.0, 3.0);
            let y = 5.0 - i as f32 * 1.5;
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x, y, 0.95),
                color: Vec4::new(1.0, 1.0, 0.95, self.flash_brightness),
                emission: 3.0 * self.flash_brightness,
                glow_color: Vec3::new(0.8, 0.85, 1.0),
                glow_radius: 3.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Snow system
// ---------------------------------------------------------------------------

pub struct SnowSystem {
    pub active: bool,
    pub intensity: f32,
    /// Accumulated snow on the floor (0..1).
    pub accumulation: f32,
}

impl SnowSystem {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.5,
            accumulation: 0.0,
        }
    }

    pub fn render(&mut self, engine: &mut ProofEngine, dt: f32, frame: u64) {
        if !self.active {
            return;
        }

        let flake_count = (150.0 * self.intensity) as usize;

        for i in 0..flake_count {
            let seed = (frame as u32).wrapping_mul(4201).wrapping_add(i as u32);
            let base_x = hash_range(seed, -9.5, 9.5);
            let speed = hash_range(seed.wrapping_add(1), 1.5, 3.5);
            let phase = hash_f32(seed.wrapping_add(2));
            let drift_amp = hash_range(seed.wrapping_add(3), 0.3, 1.2);

            let raw_y = 6.0 - ((frame as f32 * 0.016 * speed + phase * 20.0) % 12.0);
            let drift = (frame as f32 * 0.016 * 1.5 + phase * 6.28).sin() * drift_amp;
            let x = base_x + drift;
            let y = raw_y;

            let ch = if i % 5 == 0 { '*' } else { '.' };
            let alpha = hash_range(seed.wrapping_add(4), 0.3, 0.7) * self.intensity;

            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x, y, 0.15),
                color: Vec4::new(0.9, 0.92, 1.0, alpha),
                scale: Vec2::splat(0.2),
                emission: 0.15,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }

        // ── Snow accumulation on floor ──
        self.accumulation = (self.accumulation + dt * 0.01 * self.intensity).min(1.0);
        if self.accumulation > 0.05 {
            let n = (self.accumulation * 40.0) as usize;
            for i in 0..n {
                let x = -9.0 + (i as f32 / n as f32) * 18.0;
                let alpha = self.accumulation * 0.6;
                engine.spawn_glyph(Glyph {
                    character: '_',
                    position: Vec3::new(x, -5.1, 0.1),
                    color: Vec4::new(0.85, 0.88, 0.95, alpha),
                    scale: Vec2::new(0.5, 0.2),
                    emission: 0.05,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fog system
// ---------------------------------------------------------------------------

pub struct FogSystem {
    pub active: bool,
    pub density: f32, // 0..1
}

impl FogSystem {
    pub fn new() -> Self {
        Self {
            active: false,
            density: 0.4,
        }
    }

    /// Render fog as layered semi-transparent glyph sheets at different Z depths.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active {
            return;
        }

        let layers: [(f32, f32, char); 4] = [
            (-0.5, 0.10, '\u{2591}'), // light shade, near
            (-1.5, 0.15, '\u{2592}'), // medium shade, mid
            (-2.5, 0.12, '\u{2591}'), // light shade, far
            (-3.5, 0.08, '\u{2593}'), // dark shade, very far
        ];

        for (layer_idx, &(z, base_alpha, ch)) in layers.iter().enumerate() {
            let drift = (frame as f32 * 0.003 + layer_idx as f32 * 1.5).sin() * 1.5;
            let cols = 20;
            let rows = 6;
            for xi in 0..cols {
                for yi in 0..rows {
                    let x = -10.0 + xi as f32 + drift;
                    let y = -3.0 + yi as f32 * 1.8;
                    let alpha = base_alpha * self.density;
                    let pulse = ((frame as f32 * 0.005 + xi as f32 * 0.3 + yi as f32 * 0.7).sin() * 0.3 + 0.7).max(0.0);

                    engine.spawn_glyph(Glyph {
                        character: ch,
                        position: Vec3::new(x, y, z),
                        color: Vec4::new(0.6, 0.6, 0.65, alpha * pulse),
                        scale: Vec2::splat(1.0),
                        emission: 0.0,
                        layer: RenderLayer::World,
                        blend_mode: BlendMode::Normal,
                        ..Default::default()
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fire ambient — floating embers
// ---------------------------------------------------------------------------

pub struct FireAmbient {
    pub active: bool,
    pub intensity: f32,
}

impl FireAmbient {
    pub fn new() -> Self {
        Self {
            active: false,
            intensity: 0.6,
        }
    }

    /// Floating ember particles rising from screen edges.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if !self.active {
            return;
        }

        let ember_count = (60.0 * self.intensity) as usize;

        for i in 0..ember_count {
            let seed = i as u32 * 1597;
            let edge = hash_f32(seed) > 0.5; // left or right edge
            let base_x = if edge { -8.5 } else { 8.5 };
            let drift_x = hash_range(seed.wrapping_add(1), -1.5, 1.5);
            let speed = hash_range(seed.wrapping_add(2), 1.0, 3.0);
            let phase = hash_f32(seed.wrapping_add(3));
            let raw_y = -5.0 + ((frame as f32 * 0.016 * speed + phase * 15.0) % 11.0);
            let wobble = (frame as f32 * 0.03 + phase * 6.28).sin() * 0.4;

            let life_frac = (raw_y + 5.0) / 11.0; // 0 at bottom, 1 at top
            let alpha = (1.0 - life_frac) * 0.8 * self.intensity;

            // Color: bright yellow-orange at bottom, dim red at top
            let r = 1.0;
            let g = 0.6 * (1.0 - life_frac * 0.7);
            let b = 0.1 * (1.0 - life_frac);

            engine.spawn_glyph(Glyph {
                character: if i % 3 == 0 { '\u{00b7}' } else { '.' },
                position: Vec3::new(base_x + drift_x + wobble, raw_y, 0.1),
                color: Vec4::new(r, g, b, alpha),
                scale: Vec2::splat(0.2),
                emission: alpha * 1.5,
                glow_color: Vec3::new(1.0, 0.5, 0.1),
                glow_radius: 0.4,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Corruption visual — screen-edge distortion
// ---------------------------------------------------------------------------

pub struct CorruptionVisual {
    pub corruption: f32, // 0..1 (scales how far inward the effect creeps)
}

impl CorruptionVisual {
    pub fn new() -> Self {
        Self { corruption: 0.0 }
    }

    /// Purple/void distortion glyphs creeping inward from screen edges.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if self.corruption < 0.05 {
            return;
        }

        // How far in from edge the effect reaches (in world units)
        let reach = self.corruption * 6.0; // at max corruption, reaches 6 units inward
        let glyph_count = (self.corruption * 80.0) as usize;
        let glitch_chars = ['\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}', '#', '%', '&'];

        for i in 0..glyph_count {
            let seed = (frame as u32).wrapping_mul(3571).wrapping_add(i as u32);
            // Pick an edge: 0=left, 1=right, 2=top, 3=bottom
            let edge = (hash_f32(seed) * 4.0) as u32;
            let t = hash_f32(seed.wrapping_add(1));
            let depth = hash_f32(seed.wrapping_add(2)) * reach;

            let (x, y) = match edge {
                0 => (-8.7 + depth, -5.4 + t * 10.8),
                1 => (8.7 - depth, -5.4 + t * 10.8),
                2 => (-8.7 + t * 17.4, 5.4 - depth),
                _ => (-8.7 + t * 17.4, -5.4 + depth),
            };

            let fade = 1.0 - (depth / reach).min(1.0);
            let pulse = ((frame as f32 * 0.08 + i as f32 * 0.5).sin() * 0.3 + 0.7).max(0.0);
            let alpha = fade * pulse * self.corruption * 0.7;

            let ch_idx = ((frame as usize + i) * 7) % glitch_chars.len();

            engine.spawn_glyph(Glyph {
                character: glitch_chars[ch_idx],
                position: Vec3::new(x, y, 0.6),
                color: Vec4::new(0.5, 0.1, 0.8, alpha),
                scale: Vec2::splat(hash_range(seed.wrapping_add(3), 0.2, 0.6)),
                emission: alpha * 0.8,
                glow_color: Vec3::new(0.4, 0.0, 0.7),
                glow_radius: 0.6,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Floor-depth atmosphere — color grading overlay
// ---------------------------------------------------------------------------

pub struct FloorAtmosphere {
    pub floor_depth: u32, // current dungeon floor (1-based)
}

impl FloorAtmosphere {
    pub fn new() -> Self {
        Self { floor_depth: 1 }
    }

    /// Subtle color overlay that shifts with dungeon depth.
    /// Shallow = warm amber, deep = cold blue-purple, very deep = crimson-black.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        if self.floor_depth <= 1 {
            return;
        }

        let depth_norm = ((self.floor_depth as f32 - 1.0) / 20.0).min(1.0);

        // Interpolate color grading
        let (r, g, b) = if depth_norm < 0.5 {
            let t = depth_norm * 2.0;
            (
                0.4 * (1.0 - t) + 0.15 * t,
                0.35 * (1.0 - t) + 0.15 * t,
                0.2 * (1.0 - t) + 0.5 * t,
            )
        } else {
            let t = (depth_norm - 0.5) * 2.0;
            (
                0.15 * (1.0 - t) + 0.5 * t,
                0.15 * (1.0 - t) + 0.05 * t,
                0.5 * (1.0 - t) + 0.15 * t,
            )
        };

        let alpha = 0.06 + depth_norm * 0.08;
        let pulse = ((frame as f32 * 0.002).sin() * 0.02 + 1.0).max(0.0);

        // Sparse overlay grid
        for xi in 0..10 {
            for yi in 0..6 {
                let x = -9.0 + xi as f32 * 2.0;
                let y = -5.0 + yi as f32 * 2.0;
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y, 0.7),
                    color: Vec4::new(r, g, b, alpha * pulse),
                    scale: Vec2::splat(2.0),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Multiply,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Screen shake
// ---------------------------------------------------------------------------

/// CPU-side screen shake with decay. Produces a camera offset each frame.
pub struct ScreenShake {
    trauma: f32,   // current trauma 0..1
    decay: f32,    // per-second decay rate
    strength: f32, // max pixel offset in world units
}

impl ScreenShake {
    pub fn new() -> Self {
        Self {
            trauma: 0.0,
            decay: 3.0,
            strength: 0.6,
        }
    }

    /// Add trauma (clamped to 1.0).
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    /// Tick and return (offset_x, offset_y). Apply to camera or glyph positions.
    pub fn update(&mut self, dt: f32, frame: u64) -> (f32, f32) {
        if self.trauma < 0.001 {
            self.trauma = 0.0;
            return (0.0, 0.0);
        }
        let shake = self.trauma * self.trauma; // quadratic for feel
        let ox = ((frame as f32 * 17.3).sin()) * shake * self.strength;
        let oy = ((frame as f32 * 23.7).cos()) * shake * self.strength;
        self.trauma = (self.trauma - self.decay * dt).max(0.0);
        (ox, oy)
    }

    pub fn is_active(&self) -> bool {
        self.trauma > 0.001
    }
}

// ---------------------------------------------------------------------------
// Vignette effect
// ---------------------------------------------------------------------------

pub struct Vignette {
    pub active: bool,
    pub strength: f32, // 0..1
    pub color: Vec4,
}

impl Vignette {
    pub fn new() -> Self {
        Self {
            active: true,
            strength: 0.3,
            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    /// Render darkened corner / edge glyphs for vignette.
    pub fn render(&self, engine: &mut ProofEngine) {
        if !self.active || self.strength < 0.01 {
            return;
        }

        // Place dark glyphs around the screen perimeter with falloff toward center
        let steps_x = 20;
        let steps_y = 12;
        for xi in 0..steps_x {
            for yi in 0..steps_y {
                let x = -9.5 + xi as f32;
                let y = -5.5 + yi as f32;

                // Distance from center, normalized
                let dx = x / 9.5;
                let dy = y / 5.5;
                let dist = (dx * dx + dy * dy).sqrt();

                // Only draw in outer region
                if dist < 0.6 {
                    continue;
                }

                let falloff = ((dist - 0.6) / 0.4).min(1.0);
                let alpha = falloff * falloff * self.strength;

                engine.spawn_glyph(Glyph {
                    character: '\u{2588}',
                    position: Vec3::new(x, y, 0.85),
                    color: Vec4::new(self.color.x, self.color.y, self.color.z, alpha),
                    scale: Vec2::splat(1.0),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Normal,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Screen flash
// ---------------------------------------------------------------------------

pub struct ScreenFlash {
    brightness: f32,
    color: Vec3,
    decay_rate: f32,
}

impl ScreenFlash {
    pub fn new() -> Self {
        Self {
            brightness: 0.0,
            color: Vec3::new(1.0, 1.0, 1.0),
            decay_rate: 5.0,
        }
    }

    /// Trigger a flash with given color and intensity.
    pub fn trigger(&mut self, color: Vec3, intensity: f32) {
        self.brightness = intensity.min(2.0);
        self.color = color;
    }

    pub fn trigger_white(&mut self, intensity: f32) {
        self.trigger(Vec3::new(1.0, 1.0, 1.0), intensity);
    }

    pub fn trigger_red(&mut self, intensity: f32) {
        self.trigger(Vec3::new(1.0, 0.15, 0.1), intensity);
    }

    pub fn trigger_gold(&mut self, intensity: f32) {
        self.trigger(Vec3::new(1.0, 0.85, 0.2), intensity);
    }

    pub fn update(&mut self, dt: f32) {
        if self.brightness > 0.001 {
            self.brightness = (self.brightness - self.decay_rate * dt).max(0.0);
        }
    }

    pub fn render(&self, engine: &mut ProofEngine) {
        if self.brightness < 0.005 {
            return;
        }
        let a = self.brightness.min(1.0);
        for xi in 0..20 {
            for yi in 0..12 {
                let x = -9.5 + xi as f32;
                let y = -5.5 + yi as f32;
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y, 0.92),
                    color: Vec4::new(self.color.x, self.color.y, self.color.z, a),
                    scale: Vec2::splat(1.0),
                    emission: self.brightness * 2.0,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Blood splatter — persistent fading glyphs on arena
// ---------------------------------------------------------------------------

/// One blood splat instance.
struct BloodDrop {
    x: f32,
    y: f32,
    character: char,
    alpha: f32,
    scale: f32,
}

pub struct BloodSplatter {
    drops: Vec<BloodDrop>,
    max_drops: usize,
    fade_rate: f32,
}

impl BloodSplatter {
    pub fn new() -> Self {
        Self {
            drops: Vec::with_capacity(128),
            max_drops: 128,
            fade_rate: 0.03,
        }
    }

    /// Spawn a cluster of blood drops at a position.
    pub fn spawn(&mut self, x: f32, y: f32, count: usize, frame: u64) {
        let blood_chars = ['.', ',', '\'', '`', '*', ';'];
        for i in 0..count {
            if self.drops.len() >= self.max_drops {
                // Remove oldest
                self.drops.remove(0);
            }
            let seed = (frame as u32).wrapping_mul(997).wrapping_add(i as u32);
            self.drops.push(BloodDrop {
                x: x + hash_range(seed, -1.5, 1.5),
                y: y + hash_range(seed.wrapping_add(1), -0.8, 0.8),
                character: blood_chars[(i + frame as usize) % blood_chars.len()],
                alpha: hash_range(seed.wrapping_add(2), 0.5, 0.9),
                scale: hash_range(seed.wrapping_add(3), 0.15, 0.35),
            });
        }
    }

    /// Fade and render all active blood drops.
    pub fn update_and_render(&mut self, engine: &mut ProofEngine, dt: f32) {
        self.drops.retain_mut(|d| {
            d.alpha -= self.fade_rate * dt;
            d.alpha > 0.01
        });

        for d in &self.drops {
            engine.spawn_glyph(Glyph {
                character: d.character,
                position: Vec3::new(d.x, d.y, -0.1),
                color: Vec4::new(0.65, 0.05, 0.05, d.alpha),
                scale: Vec2::splat(d.scale),
                emission: 0.1,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }
    }

    /// Clear all blood (e.g. on screen transition).
    pub fn clear(&mut self) {
        self.drops.clear();
    }
}

// ---------------------------------------------------------------------------
// Footstep dust
// ---------------------------------------------------------------------------

struct DustPuff {
    x: f32,
    y: f32,
    age: f32,
    lifetime: f32,
}

pub struct FootstepDust {
    puffs: Vec<DustPuff>,
    max_puffs: usize,
}

impl FootstepDust {
    pub fn new() -> Self {
        Self {
            puffs: Vec::with_capacity(32),
            max_puffs: 32,
        }
    }

    /// Spawn a small dust burst at entity feet.
    pub fn spawn(&mut self, x: f32, y: f32, frame: u64) {
        let count = 3;
        for i in 0..count {
            if self.puffs.len() >= self.max_puffs {
                self.puffs.remove(0);
            }
            let seed = (frame as u32).wrapping_add(i as u32 * 113);
            self.puffs.push(DustPuff {
                x: x + hash_range(seed, -0.3, 0.3),
                y: y + hash_range(seed.wrapping_add(1), -0.1, 0.2),
                age: 0.0,
                lifetime: hash_range(seed.wrapping_add(2), 0.3, 0.6),
            });
        }
    }

    pub fn update_and_render(&mut self, engine: &mut ProofEngine, dt: f32) {
        self.puffs.retain_mut(|p| {
            p.age += dt;
            p.age < p.lifetime
        });

        for p in &self.puffs {
            let t = p.age / p.lifetime;
            let alpha = (1.0 - t) * 0.4;
            let rise = t * 0.5;
            engine.spawn_glyph(Glyph {
                character: '.',
                position: Vec3::new(p.x, p.y + rise, 0.05),
                color: Vec4::new(0.5, 0.45, 0.35, alpha),
                scale: Vec2::splat(0.15 + t * 0.1),
                emission: 0.0,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }

    pub fn clear(&mut self) {
        self.puffs.clear();
    }
}

// ---------------------------------------------------------------------------
// Master environmental effects controller
// ---------------------------------------------------------------------------

/// Bundles all environmental systems for convenient single-call update/render.
pub struct EnvironmentalFx {
    pub rain: RainSystem,
    pub snow: SnowSystem,
    pub fog: FogSystem,
    pub fire_ambient: FireAmbient,
    pub corruption_visual: CorruptionVisual,
    pub floor_atmosphere: FloorAtmosphere,
    pub screen_shake: ScreenShake,
    pub vignette: Vignette,
    pub screen_flash: ScreenFlash,
    pub blood_splatter: BloodSplatter,
    pub footstep_dust: FootstepDust,
}

impl EnvironmentalFx {
    pub fn new() -> Self {
        Self {
            rain: RainSystem::new(),
            snow: SnowSystem::new(),
            fog: FogSystem::new(),
            fire_ambient: FireAmbient::new(),
            corruption_visual: CorruptionVisual::new(),
            floor_atmosphere: FloorAtmosphere::new(),
            screen_shake: ScreenShake::new(),
            vignette: Vignette::new(),
            screen_flash: ScreenFlash::new(),
            blood_splatter: BloodSplatter::new(),
            footstep_dust: FootstepDust::new(),
        }
    }

    /// Update all systems and render all active effects.
    pub fn update_and_render(&mut self, engine: &mut ProofEngine, dt: f32, frame: u64) {
        // Weather
        self.rain.render(engine, dt, frame);
        self.snow.render(engine, dt, frame);
        self.fog.render(engine, frame);
        self.fire_ambient.render(engine, frame);

        // Atmosphere
        self.corruption_visual.render(engine, frame);
        self.floor_atmosphere.render(engine, frame);

        // Screen-level
        self.screen_flash.update(dt);
        self.screen_flash.render(engine);
        self.vignette.render(engine);

        // Persistent world effects
        self.blood_splatter.update_and_render(engine, dt);
        self.footstep_dust.update_and_render(engine, dt);
    }

    /// Get screen shake offset for this frame. Apply to camera or entity positions.
    pub fn shake_offset(&mut self, dt: f32, frame: u64) -> (f32, f32) {
        self.screen_shake.update(dt, frame)
    }

    /// Convenience: set weather by name.
    pub fn set_weather(&mut self, weather: &str) {
        self.rain.active = false;
        self.snow.active = false;
        self.fog.active = false;
        self.fire_ambient.active = false;

        match weather {
            "rain" | "storm" => {
                self.rain.active = true;
                self.rain.intensity = if weather == "storm" { 0.9 } else { 0.5 };
            }
            "snow" | "blizzard" => {
                self.snow.active = true;
                self.snow.intensity = if weather == "blizzard" { 0.9 } else { 0.4 };
            }
            "fog" | "mist" => {
                self.fog.active = true;
                self.fog.density = if weather == "fog" { 0.5 } else { 0.25 };
            }
            "fire" | "inferno" => {
                self.fire_ambient.active = true;
                self.fire_ambient.intensity = if weather == "inferno" { 1.0 } else { 0.5 };
            }
            "clear" | "none" => {}
            _ => {}
        }
    }
}
