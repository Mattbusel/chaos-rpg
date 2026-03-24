//! Screen transition effects between AppScreen changes.
//!
//! All transitions are rendered as immediate-mode glyphs on the Overlay layer.
//! Camera at (0,0,-10) looking at origin. Visible area at z=0: +/-8.7 x, +/-5.4 y.

use proof_engine::prelude::*;

// ---------------------------------------------------------------------------
// Hash helper (same as environmental.rs — self-contained)
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
// Transition kinds
// ---------------------------------------------------------------------------

/// Identifies which visual transition to play.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransitionKind {
    FadeToBlack,
    SlideLeft,
    SlideRight,
    Dissolve,
    ChaosWarp,
    BossIntro,
    DeathTransition,
    VictoryTransition,
    FloorTransition,
}

// ---------------------------------------------------------------------------
// Active transition state
// ---------------------------------------------------------------------------

/// An in-progress transition.
struct ActiveTransition {
    kind: TransitionKind,
    elapsed: f32,
    duration: f32,
    /// Optional text payload (boss name, etc.).
    text: String,
    /// Cached random seeds for dissolve / chaos patterns.
    seed_base: u32,
}

impl ActiveTransition {
    fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    fn is_done(&self) -> bool {
        self.elapsed >= self.duration
    }
}

// ---------------------------------------------------------------------------
// Render helpers
// ---------------------------------------------------------------------------

fn render_text_line(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, scale: f32, emission: f32) {
    let sp = scale * 0.85;
    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' { continue; }
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * sp, y, 0.98),
            scale: Vec2::splat(scale),
            color,
            emission,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
}

fn render_text_centered(engine: &mut ProofEngine, text: &str, y: f32, color: Vec4, scale: f32, emission: f32) {
    let w = text.len() as f32 * scale * 0.85;
    render_text_line(engine, text, -w * 0.5, y, color, scale, emission);
}

/// Fill the screen with a solid-color glyph block at given alpha.
fn fill_screen(engine: &mut ProofEngine, color: Vec3, alpha: f32) {
    for xi in 0..20 {
        for yi in 0..12 {
            let x = -9.5 + xi as f32;
            let y = -5.5 + yi as f32;
            engine.spawn_glyph(Glyph {
                character: '\u{2588}',
                position: Vec3::new(x, y, 0.95),
                color: Vec4::new(color.x, color.y, color.z, alpha),
                scale: Vec2::splat(1.0),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Individual transition renderers
// ---------------------------------------------------------------------------

fn render_fade_to_black(engine: &mut ProofEngine, t: f32) {
    // First half: fade out to black. Second half: hold black then fade in.
    let alpha = if t < 0.5 {
        t * 2.0
    } else {
        (1.0 - t) * 2.0
    };
    fill_screen(engine, Vec3::ZERO, alpha.clamp(0.0, 1.0));
}

fn render_slide(engine: &mut ProofEngine, t: f32, going_left: bool) {
    // A wall of block glyphs sliding across the screen.
    let dir = if going_left { -1.0 } else { 1.0 };
    let edge = if t < 0.5 {
        // Wipe in
        let wipe_t = t * 2.0;
        -10.0 * dir + wipe_t * 20.0 * dir
    } else {
        // Wipe out (opposite direction)
        let wipe_t = (t - 0.5) * 2.0;
        10.0 * dir - (1.0 - wipe_t) * 20.0 * dir
    };

    for yi in 0..12 {
        let y = -5.5 + yi as f32;
        for xi in 0..22 {
            let x_base = -10.0 + xi as f32;
            let x = x_base;
            let visible = if going_left {
                x > edge
            } else {
                x < edge
            };
            if visible && t < 0.5 || !visible && t >= 0.5 {
                continue;
            }
            engine.spawn_glyph(Glyph {
                character: '\u{2588}',
                position: Vec3::new(x, y, 0.95),
                color: Vec4::new(0.05, 0.05, 0.08, 0.95),
                scale: Vec2::splat(1.0),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }
}

fn render_dissolve(engine: &mut ProofEngine, t: f32, seed_base: u32) {
    // Random glyphs appear/disappear based on threshold.
    let threshold = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
    let total = 20 * 12;
    for idx in 0..total {
        let xi = idx % 20;
        let yi = idx / 20;
        let x = -9.5 + xi as f32;
        let y = -5.5 + yi as f32;
        let r = hash_f32(seed_base.wrapping_add(idx as u32));
        if r < threshold {
            let ch_pool = ['\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'];
            let ci = (r * 4.0) as usize % ch_pool.len();
            engine.spawn_glyph(Glyph {
                character: ch_pool[ci],
                position: Vec3::new(x, y, 0.95),
                color: Vec4::new(0.02, 0.02, 0.05, 0.9),
                scale: Vec2::splat(1.0),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }
}

fn render_chaos_warp(engine: &mut ProofEngine, t: f32, frame_seed: u32) {
    // Distort screen with sinusoidal warping that intensifies then resolves.
    let intensity = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
    let warp_chars = ['\u{2591}', '\u{2592}', '~', '\u{00b7}', '%', '#'];

    for idx in 0..120 {
        let seed = frame_seed.wrapping_add(idx);
        let base_x = hash_range(seed, -9.0, 9.0);
        let base_y = hash_range(seed.wrapping_add(1), -5.0, 5.0);

        let warp_x = (base_y * 2.0 + t * 20.0).sin() * intensity * 3.0;
        let warp_y = (base_x * 1.5 + t * 15.0).cos() * intensity * 2.0;

        let x = base_x + warp_x;
        let y = base_y + warp_y;
        let alpha = intensity * hash_range(seed.wrapping_add(2), 0.3, 0.8);
        let ci = (seed as usize) % warp_chars.len();

        engine.spawn_glyph(Glyph {
            character: warp_chars[ci],
            position: Vec3::new(x, y, 0.94),
            color: Vec4::new(0.4, 0.1, 0.7, alpha),
            scale: Vec2::splat(hash_range(seed.wrapping_add(3), 0.3, 0.8)),
            emission: alpha * 0.5,
            glow_color: Vec3::new(0.5, 0.2, 0.9),
            glow_radius: 0.5,
            layer: RenderLayer::Overlay,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Overlay darkness at peak
    if intensity > 0.8 {
        let dark_alpha = (intensity - 0.8) * 5.0;
        fill_screen(engine, Vec3::ZERO, dark_alpha.min(0.9));
    }
}

fn render_boss_intro(engine: &mut ProofEngine, t: f32, name: &str) {
    // Phase 1 (0..0.3): screen darkens
    // Phase 2 (0.3..0.7): name assembles letter by letter
    // Phase 3 (0.7..1.0): flash and fade

    if t < 0.3 {
        let dark = t / 0.3;
        fill_screen(engine, Vec3::ZERO, dark * 0.85);
    } else if t < 0.7 {
        fill_screen(engine, Vec3::ZERO, 0.85);

        // Letter-by-letter reveal
        let reveal_t = (t - 0.3) / 0.4;
        let total_chars = name.len();
        let chars_shown = ((reveal_t * total_chars as f32 * 1.5).floor() as usize).min(total_chars);

        let sp = 0.7 * 0.85;
        let total_w = total_chars as f32 * sp;
        let start_x = -total_w * 0.5;

        for (i, ch) in name.chars().enumerate() {
            if i >= chars_shown || ch == ' ' { continue; }

            let char_age = reveal_t - (i as f32 / (total_chars as f32 * 1.5));
            let scale_factor = if char_age < 0.05 { char_age / 0.05 } else { 1.0 };
            let glow = if char_age < 0.1 { 2.0 } else { 0.8 };

            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(start_x + i as f32 * sp, 1.0, 0.98),
                scale: Vec2::splat(0.7 * scale_factor),
                color: Vec4::new(1.0, 0.3, 0.2, 1.0),
                emission: glow,
                glow_color: Vec3::new(1.0, 0.2, 0.1),
                glow_radius: 2.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }

        // Subtitle
        if chars_shown >= total_chars {
            render_text_centered(
                engine,
                "A CHALLENGER APPROACHES",
                -1.0,
                Vec4::new(0.7, 0.7, 0.7, 0.6),
                0.35,
                0.3,
            );
        }
    } else {
        // Flash out
        let fade = (t - 0.7) / 0.3;
        let flash = if fade < 0.2 { fade * 5.0 } else { (1.0 - fade) * 1.25 };
        fill_screen(engine, Vec3::new(1.0, 0.4, 0.2), flash.clamp(0.0, 1.0) * 0.6);
        fill_screen(engine, Vec3::ZERO, (1.0 - fade).max(0.0) * 0.5);
    }
}

fn render_death_transition(engine: &mut ProofEngine, t: f32, seed_base: u32) {
    // Phase 1 (0..0.4): crack lines appear across screen
    // Phase 2 (0.4..0.7): screen shatters into debris
    // Phase 3 (0.7..1.0): debris falls, fade to black

    if t < 0.4 {
        let crack_t = t / 0.4;
        let crack_count = (crack_t * 30.0) as usize;
        let crack_chars = ['/', '\\', '|', '-', '+', 'X'];

        for i in 0..crack_count {
            let seed = seed_base.wrapping_add(i as u32 * 53);
            let x = hash_range(seed, -8.0, 8.0);
            let y = hash_range(seed.wrapping_add(1), -4.5, 4.5);
            let ci = i % crack_chars.len();
            let alpha = crack_t * 0.8;

            engine.spawn_glyph(Glyph {
                character: crack_chars[ci],
                position: Vec3::new(x, y, 0.96),
                color: Vec4::new(0.9, 0.9, 0.95, alpha),
                scale: Vec2::splat(0.4),
                emission: alpha * 0.5,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    } else if t < 0.7 {
        let shatter_t = (t - 0.4) / 0.3;
        let debris_count = 60;
        let debris_chars = ['\u{2588}', '\u{2593}', '\u{2592}', '\u{2591}', '#'];

        for i in 0..debris_count {
            let seed = seed_base.wrapping_add(i as u32 * 37 + 1000);
            let origin_x = hash_range(seed, -8.0, 8.0);
            let origin_y = hash_range(seed.wrapping_add(1), -4.5, 4.5);
            let vel_x = hash_range(seed.wrapping_add(2), -3.0, 3.0);
            let vel_y = hash_range(seed.wrapping_add(3), -1.0, -5.0); // falling
            let x = origin_x + vel_x * shatter_t;
            let y = origin_y + vel_y * shatter_t;
            let rot = shatter_t * hash_range(seed.wrapping_add(4), -5.0, 5.0);
            let alpha = 1.0 - shatter_t * 0.5;
            let ci = i % debris_chars.len();

            engine.spawn_glyph(Glyph {
                character: debris_chars[ci],
                position: Vec3::new(x, y, 0.96),
                color: Vec4::new(0.4, 0.1, 0.1, alpha),
                scale: Vec2::splat(hash_range(seed.wrapping_add(5), 0.3, 0.8)),
                rotation: rot,
                emission: 0.1,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    } else {
        let fade = (t - 0.7) / 0.3;
        // Remaining debris still falling
        let remaining = (20.0 * (1.0 - fade)) as usize;
        for i in 0..remaining {
            let seed = seed_base.wrapping_add(i as u32 * 37 + 2000);
            let x = hash_range(seed, -8.0, 8.0);
            let fall_dist = 5.0 + fade * 8.0;
            let y = hash_range(seed.wrapping_add(1), 2.0, 5.0) - fall_dist;
            let alpha = (1.0 - fade) * 0.6;

            engine.spawn_glyph(Glyph {
                character: '\u{2593}',
                position: Vec3::new(x, y, 0.96),
                color: Vec4::new(0.3, 0.05, 0.05, alpha),
                scale: Vec2::splat(0.5),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
        fill_screen(engine, Vec3::ZERO, fade);
    }
}

fn render_victory_transition(engine: &mut ProofEngine, t: f32) {
    // Golden light expanding from center, then glorious fade.
    let expand = if t < 0.6 { t / 0.6 } else { 1.0 };
    let max_radius = expand * 12.0;

    // Golden rays
    let ray_count = 24;
    for i in 0..ray_count {
        let angle = (i as f32 / ray_count as f32) * std::f32::consts::TAU;
        let ray_len = max_radius;
        let segments = (ray_len * 2.0) as usize;
        for s in 0..segments {
            let r = (s as f32 / segments as f32) * ray_len;
            let x = angle.cos() * r;
            let y = angle.sin() * r;
            if x.abs() > 9.5 || y.abs() > 5.5 { continue; }

            let fade_along = 1.0 - (r / ray_len);
            let alpha = fade_along * 0.4 * expand;

            engine.spawn_glyph(Glyph {
                character: '\u{00b7}',
                position: Vec3::new(x, y, 0.94),
                color: Vec4::new(1.0, 0.88, 0.3, alpha),
                scale: Vec2::splat(0.25),
                emission: alpha * 2.0,
                glow_color: Vec3::new(1.0, 0.85, 0.2),
                glow_radius: 0.8,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }

    // Center star
    if t > 0.1 {
        let star_alpha = (expand * 1.5).min(1.0);
        engine.spawn_glyph(Glyph {
            character: '*',
            position: Vec3::new(0.0, 0.0, 0.97),
            color: Vec4::new(1.0, 1.0, 0.8, star_alpha),
            scale: Vec2::splat(1.5),
            emission: 3.0,
            glow_color: Vec3::new(1.0, 0.9, 0.5),
            glow_radius: 5.0,
            layer: RenderLayer::Overlay,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Fade out at end
    if t > 0.75 {
        let fade = (t - 0.75) / 0.25;
        fill_screen(engine, Vec3::new(1.0, 0.95, 0.7), fade * 0.8);
    }
}

fn render_floor_transition(engine: &mut ProofEngine, t: f32) {
    // Staircase descending animation.
    let step_count = 8;
    let step_width = 3.0;
    let step_height = 0.8;

    // Background darkens
    fill_screen(engine, Vec3::ZERO, 0.7);

    let descent = t * step_count as f32;

    for s in 0..step_count {
        let sf = s as f32;
        let visible = descent >= sf;
        if !visible { continue; }

        let appear_t = ((descent - sf) / 1.0).min(1.0);
        let x_center = -step_width * 0.5 + sf * 0.3;
        let y_base = 3.0 - sf * step_height;

        // Step surface
        let stair_n = (step_width / 0.4) as usize;
        for i in 0..stair_n {
            let x = x_center + i as f32 * 0.4;
            let alpha = appear_t * 0.8;
            engine.spawn_glyph(Glyph {
                character: '\u{2588}',
                position: Vec3::new(x, y_base, 0.93),
                color: Vec4::new(0.3, 0.25, 0.2, alpha),
                scale: Vec2::new(0.4, 0.3),
                emission: 0.1,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }

        // Step edge highlight
        engine.spawn_glyph(Glyph {
            character: '_',
            position: Vec3::new(x_center + step_width * 0.5, y_base + step_height * 0.5, 0.94),
            color: Vec4::new(0.6, 0.5, 0.3, appear_t * 0.5),
            scale: Vec2::new(step_width, 0.1),
            emission: 0.2,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }

    // Player figure descending
    let player_step = descent.floor().min((step_count - 1) as f32);
    let step_frac = descent.fract();
    let px = -step_width * 0.5 + player_step * 0.3 + step_frac * 0.3 + step_width * 0.5;
    let py = 3.0 - player_step * step_height - step_frac * step_height + step_height * 0.8;

    engine.spawn_glyph(Glyph {
        character: '@',
        position: Vec3::new(px, py, 0.96),
        color: Vec4::new(0.9, 0.85, 0.7, 0.9),
        scale: Vec2::splat(0.5),
        emission: 0.6,
        glow_color: Vec3::new(0.8, 0.7, 0.4),
        glow_radius: 1.0,
        layer: RenderLayer::Overlay,
        ..Default::default()
    });

    // "DESCENDING..." text
    let text_alpha = if t > 0.3 { ((t - 0.3) / 0.2).min(1.0) * 0.7 } else { 0.0 };
    if text_alpha > 0.01 {
        render_text_centered(
            engine,
            "DESCENDING...",
            -4.0,
            Vec4::new(0.6, 0.55, 0.4, text_alpha),
            0.4,
            0.3,
        );
    }
}

// ---------------------------------------------------------------------------
// TransitionManager
// ---------------------------------------------------------------------------

/// Manages queued screen transitions. Blocks input while active.
pub struct TransitionManager {
    active: Option<ActiveTransition>,
    queue: Vec<(TransitionKind, f32, String)>, // (kind, duration, text)
}

impl TransitionManager {
    pub fn new() -> Self {
        Self {
            active: None,
            queue: Vec::new(),
        }
    }

    /// Queue a transition. Duration in seconds.
    pub fn queue(&mut self, kind: TransitionKind, duration: f32) {
        self.queue.push((kind, duration, String::new()));
    }

    /// Queue a transition with associated text (e.g. boss name).
    pub fn queue_with_text(&mut self, kind: TransitionKind, duration: f32, text: &str) {
        self.queue.push((kind, duration, text.to_string()));
    }

    /// Start a transition immediately, replacing any current one.
    pub fn start(&mut self, kind: TransitionKind, duration: f32) {
        self.active = Some(ActiveTransition {
            kind,
            elapsed: 0.0,
            duration,
            text: String::new(),
            seed_base: kind as u32 * 9973 + 42,
        });
    }

    /// Start with text payload.
    pub fn start_with_text(&mut self, kind: TransitionKind, duration: f32, text: &str) {
        self.active = Some(ActiveTransition {
            kind,
            elapsed: 0.0,
            duration,
            text: text.to_string(),
            seed_base: kind as u32 * 9973 + 42,
        });
    }

    /// Returns true while a transition is playing (input should be blocked).
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Returns the progress (0..1) of current transition, or 0 if none.
    pub fn progress(&self) -> f32 {
        self.active.as_ref().map(|a| a.progress()).unwrap_or(0.0)
    }

    /// Returns true when transition just crossed the midpoint (screen fully covered).
    /// Useful for swapping the underlying screen content.
    pub fn at_midpoint(&self) -> bool {
        if let Some(ref a) = self.active {
            let prev = (a.elapsed - 0.016) / a.duration;
            let curr = a.progress();
            prev < 0.5 && curr >= 0.5
        } else {
            false
        }
    }

    /// Update and render the active transition.
    pub fn update_and_render(&mut self, engine: &mut ProofEngine, dt: f32) {
        // If no active transition, pop from queue.
        if self.active.is_none() {
            if let Some((kind, dur, text)) = self.queue.pop() {
                self.active = Some(ActiveTransition {
                    kind,
                    elapsed: 0.0,
                    duration: dur,
                    text,
                    seed_base: kind as u32 * 9973 + 42,
                });
            }
        }

        let finished = if let Some(ref mut tr) = self.active {
            tr.elapsed += dt;

            let t = tr.progress();
            let seed = tr.seed_base;

            match tr.kind {
                TransitionKind::FadeToBlack => render_fade_to_black(engine, t),
                TransitionKind::SlideLeft => render_slide(engine, t, true),
                TransitionKind::SlideRight => render_slide(engine, t, false),
                TransitionKind::Dissolve => render_dissolve(engine, t, seed),
                TransitionKind::ChaosWarp => render_chaos_warp(engine, t, seed),
                TransitionKind::BossIntro => render_boss_intro(engine, t, &tr.text.clone()),
                TransitionKind::DeathTransition => render_death_transition(engine, t, seed),
                TransitionKind::VictoryTransition => render_victory_transition(engine, t),
                TransitionKind::FloorTransition => render_floor_transition(engine, t),
            }

            tr.is_done()
        } else {
            false
        };

        if finished {
            self.active = None;
        }
    }
}
