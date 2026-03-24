//! Enemy entity rendering — tier-scaled formations.
//!
//! Five enemy tiers with escalating glyph counts, formation complexity, and
//! visual intensity. Enemy name characters are used as glyph symbols.
//!
//! ## Rendering modes
//!
//! * **Immediate-mode** — `render_enemy()` / `render_enemy_full()` spawn glyphs
//!   every frame via `engine.spawn_glyph()`.
//! * **Formation-backed** — `build_enemy_entity()` returns an `AmorphousEntity`.
//!
//! ## Visual features
//!
//! * 5 enemy tiers (Minion / Elite / Champion / Boss / Abomination).
//! * 7 element themes (Fire / Ice / Lightning / Poison / Shadow / Holy / Neutral).
//! * 10 unique boss visual profiles matching proof-engine BossType.
//! * Spawn animation (glyphs expand from center outward).
//! * Death animation with element-specific dissolution.

use proof_engine::prelude::*;
use std::f32::consts::{PI, TAU};

use super::formations::{
    self, FormationShape, ElementalDeathStyle,
};

// ── Tier constants ───────────────────────────────────────────────────────────

/// Enemy tier enum for determining visual complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyTier {
    Minion,      // 10 glyphs, simple cluster
    Elite,       // 20 glyphs, ring formation
    Champion,    // 30 glyphs, double ring
    Boss,        // 50+ glyphs, complex formation
    Abomination, // 80+ glyphs, massive chaotic mass
}

impl EnemyTier {
    /// Derive tier from a numeric tier value.
    pub fn from_tier(tier: u32) -> Self {
        match tier {
            0..=1 => EnemyTier::Minion,
            2..=3 => EnemyTier::Elite,
            4..=5 => EnemyTier::Champion,
            6..=7 => EnemyTier::Boss,
            _     => EnemyTier::Abomination,
        }
    }

    /// How many glyphs this tier spawns.
    fn glyph_count(self) -> usize {
        match self {
            EnemyTier::Minion      => 10,
            EnemyTier::Elite       => 20,
            EnemyTier::Champion    => 30,
            EnemyTier::Boss        => 55,
            EnemyTier::Abomination => 85,
        }
    }

    /// Base emission intensity.
    fn emission(self) -> f32 {
        match self {
            EnemyTier::Minion      => 0.3,
            EnemyTier::Elite       => 0.5,
            EnemyTier::Champion    => 0.7,
            EnemyTier::Boss        => 1.0,
            EnemyTier::Abomination => 1.4,
        }
    }

    /// Base glyph scale.
    fn glyph_scale(self) -> f32 {
        match self {
            EnemyTier::Minion      => 0.7,
            EnemyTier::Elite       => 0.8,
            EnemyTier::Champion    => 0.85,
            EnemyTier::Boss        => 0.95,
            EnemyTier::Abomination => 1.05,
        }
    }

    /// Color tint (base hostile red, escalating saturation).
    fn base_color(self) -> Vec4 {
        match self {
            EnemyTier::Minion      => Vec4::new(0.75, 0.30, 0.25, 1.0),
            EnemyTier::Elite       => Vec4::new(0.85, 0.25, 0.20, 1.0),
            EnemyTier::Champion    => Vec4::new(0.90, 0.20, 0.15, 1.0),
            EnemyTier::Boss        => Vec4::new(0.95, 0.15, 0.10, 1.0),
            EnemyTier::Abomination => Vec4::new(1.00, 0.08, 0.08, 1.0),
        }
    }
}

// ── Public entry point ───────────────────────────────────────────────────────

/// Render an enemy entity for a single frame.
///
/// * `engine`   — proof engine handle for spawning glyphs.
/// * `name`     — enemy name; first characters used as glyph symbols.
/// * `tier`     — numeric tier (0-1 Minion, 2-3 Elite, 4-5 Champion, 6-7 Boss, 8+ Abomination).
/// * `position` — world-space center (typically `(4, 0, 0)` in combat).
/// * `hp_frac`  — health fraction `[0.0, 1.0]` — controls formation cohesion.
/// * `frame`    — monotonic frame counter for idle animations.
pub fn render_enemy(
    engine: &mut ProofEngine,
    name: &str,
    tier: u32,
    position: Vec3,
    hp_frac: f32,
    frame: u64,
) {
    let hp = hp_frac.clamp(0.0, 1.0);
    let time = frame as f32 / 60.0;
    let enemy_tier = EnemyTier::from_tier(tier);

    // Build the character palette from the enemy's name + fallback symbols
    let chars = build_char_palette(name);

    match enemy_tier {
        EnemyTier::Minion      => render_minion(engine, &chars, position, hp, time, enemy_tier),
        EnemyTier::Elite       => render_elite(engine, &chars, position, hp, time, enemy_tier),
        EnemyTier::Champion    => render_champion(engine, &chars, position, hp, time, enemy_tier),
        EnemyTier::Boss        => render_boss(engine, &chars, position, hp, time, frame, enemy_tier),
        EnemyTier::Abomination => render_abomination(engine, &chars, position, hp, time, frame, enemy_tier),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a glyph character palette from the enemy name's first few characters
/// plus hostile fallback symbols.
fn build_char_palette(name: &str) -> Vec<char> {
    let mut chars: Vec<char> = name.chars()
        .filter(|c| !c.is_whitespace())
        .take(6)
        .collect();
    // Pad with hostile-looking fallback symbols
    let fallbacks = ['\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}', '\u{25CF}', '\u{25C6}'];
    for &fb in &fallbacks {
        if chars.len() >= 8 { break; }
        chars.push(fb);
    }
    chars
}

/// HP-linked scatter — formation breaks apart as health drops.
fn scatter(hp: f32, idx: usize, time: f32) -> Vec3 {
    let chaos = (1.0 - hp) * 1.4;
    let seed = idx as f32 * 1.618;
    Vec3::new(
        (seed * 3.7 + time * 1.1).sin() * chaos,
        (seed * 2.3 + time * 0.9).cos() * chaos,
        0.0,
    )
}

/// Breathing scale oscillation.
fn breathe(time: f32, rate: f32, depth: f32) -> f32 {
    1.0 + (time * rate * TAU).sin() * depth
}

/// Spawn a single enemy glyph on the Entity layer.
fn spawn_enemy(
    engine: &mut ProofEngine,
    ch: char,
    pos: Vec3,
    color: Vec4,
    emission: f32,
    scale: f32,
) {
    engine.spawn_glyph(Glyph {
        character: ch,
        position: pos,
        color,
        emission,
        scale: Vec2::new(scale, scale),
        layer: RenderLayer::Entity,
        ..Default::default()
    });
}

/// Spawn an enemy glyph with glow on the Entity layer.
fn spawn_enemy_glow(
    engine: &mut ProofEngine,
    ch: char,
    pos: Vec3,
    color: Vec4,
    emission: f32,
    scale: f32,
    glow_color: Vec3,
    glow_radius: f32,
) {
    engine.spawn_glyph(Glyph {
        character: ch,
        position: pos,
        color,
        emission,
        scale: Vec2::new(scale, scale),
        glow_color,
        glow_radius,
        layer: RenderLayer::Entity,
        ..Default::default()
    });
}

// ── Minion: Simple cluster (10 glyphs) ──────────────────────────────────────
// Weak scattered blob. Slow subtle breathing.

fn render_minion(
    engine: &mut ProofEngine,
    chars: &[char],
    pos: Vec3,
    hp: f32,
    time: f32,
    tier: EnemyTier,
) {
    let scale = breathe(time, 0.5, 0.02);
    let count = tier.glyph_count();
    let base_color = tier.base_color();
    let em = tier.emission();
    let gs = tier.glyph_scale();

    // Polar cluster with slight radius variation
    for i in 0..count {
        let angle = (i as f32 / count as f32) * TAU + 0.3;
        let r = 0.4 + (i as f32 * 1.618).fract() * 0.6;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, i, time);
        let dim = 0.8 + (time * 1.5 + i as f32 * 0.7).sin() * 0.15;
        let color = Vec4::new(
            base_color.x * dim,
            base_color.y * dim,
            base_color.z * dim,
            base_color.w,
        );
        spawn_enemy(engine, chars[i % chars.len()], p, color, em, gs);
    }
}

// ── Elite: Ring formation (20 glyphs) ───────────────────────────────────────
// Single rotating ring with pulsing emission.

fn render_elite(
    engine: &mut ProofEngine,
    chars: &[char],
    pos: Vec3,
    hp: f32,
    time: f32,
    tier: EnemyTier,
) {
    let scale = breathe(time, 0.7, 0.03);
    let count = tier.glyph_count();
    let base_color = tier.base_color();
    let em = tier.emission();
    let gs = tier.glyph_scale();

    for i in 0..count {
        let angle = (i as f32 / count as f32) * TAU + time * 0.6;
        let r = 1.2;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, i, time);
        let pulse = ((time * 2.5 + i as f32 * 0.4).sin() * 0.2 + 0.8).max(0.3);
        let color = Vec4::new(
            base_color.x * pulse,
            base_color.y * pulse,
            base_color.z * pulse,
            1.0,
        );
        let glow = Vec3::new(base_color.x, base_color.y * 0.5, base_color.z * 0.3);
        spawn_enemy_glow(engine, chars[i % chars.len()], p, color, em * pulse, gs, glow, 0.4);
    }
}

// ── Champion: Double ring (30 glyphs) ───────────────────────────────────────
// Outer ring (18) + inner ring (12), counter-rotating.

fn render_champion(
    engine: &mut ProofEngine,
    chars: &[char],
    pos: Vec3,
    hp: f32,
    time: f32,
    tier: EnemyTier,
) {
    let scale = breathe(time, 0.8, 0.035);
    let base_color = tier.base_color();
    let em = tier.emission();
    let gs = tier.glyph_scale();
    let mut idx = 0usize;

    // Outer ring — 18 glyphs rotating clockwise
    for i in 0..18 {
        let angle = (i as f32 / 18.0) * TAU + time * 0.5;
        let r = 1.6;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let pulse = ((time * 2.0 + i as f32 * 0.5).sin() * 0.15 + 0.85).max(0.3);
        let color = Vec4::new(
            base_color.x * pulse,
            base_color.y * pulse,
            base_color.z * pulse,
            1.0,
        );
        spawn_enemy_glow(
            engine, chars[idx % chars.len()], p, color,
            em * pulse, gs, Vec3::new(0.9, 0.2, 0.1), 0.5,
        );
        idx += 1;
    }

    // Inner ring — 12 glyphs counter-rotating
    for i in 0..12 {
        let angle = (i as f32 / 12.0) * TAU - time * 0.7;
        let r = 0.8;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let bright = ((time * 3.0 + i as f32 * 0.8).sin() * 0.2 + 0.9).max(0.4);
        let color = Vec4::new(
            (base_color.x * bright).min(1.0),
            base_color.y * bright * 1.2,
            base_color.z * bright,
            1.0,
        );
        spawn_enemy_glow(
            engine, chars[idx % chars.len()], p, color,
            em * 1.2, gs * 1.05, Vec3::new(1.0, 0.3, 0.15), 0.6,
        );
        idx += 1;
    }
}

// ── Boss: Complex star/helix formation (55 glyphs) ──────────────────────────
// Star core + double helix arms + floating crown. Dramatic glow.

fn render_boss(
    engine: &mut ProofEngine,
    chars: &[char],
    pos: Vec3,
    hp: f32,
    time: f32,
    frame: u64,
    tier: EnemyTier,
) {
    let scale = breathe(time, 0.6, 0.04);
    let base_color = tier.base_color();
    let em = tier.emission();
    let gs = tier.glyph_scale();
    let mut idx = 0usize;

    // Star core — 5-pointed star, 3 glyphs per arm = 15
    for arm in 0..5 {
        let arm_angle = (arm as f32 / 5.0) * TAU + time * 0.3;
        for depth in 0..3 {
            let r = (depth as f32 + 1.0) * 0.5;
            let base = Vec3::new(arm_angle.cos() * r, arm_angle.sin() * r, 0.0) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let intensity = 1.0 - depth as f32 * 0.15;
            let color = Vec4::new(
                base_color.x * intensity,
                base_color.y * intensity + 0.1,
                base_color.z * intensity,
                1.0,
            );
            spawn_enemy_glow(
                engine, chars[idx % chars.len()], p, color,
                em * intensity, gs, Vec3::new(1.0, 0.2, 0.1), 0.8,
            );
            idx += 1;
        }
    }

    // Double helix arms — 2 interleaved spirals, 16 glyphs each = 32
    for strand in 0..2 {
        let phase = strand as f32 * std::f32::consts::PI;
        for i in 0..16 {
            let t_param = i as f32 / 16.0;
            let angle = t_param * TAU * 2.0 + time * 0.8 + phase;
            let helix_r = 0.5 + t_param * 0.3;
            let y_offset = (t_param - 0.5) * 3.5;
            let base = Vec3::new(
                angle.cos() * helix_r,
                y_offset + angle.sin() * 0.3,
                0.0,
            ) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let wave = ((time * 2.5 + i as f32 * 0.6).sin() * 0.2 + 0.8).max(0.3);
            let strand_tint = if strand == 0 { 0.0 } else { 0.15 };
            let color = Vec4::new(
                base_color.x * wave,
                base_color.y * wave + strand_tint,
                base_color.z * wave + strand_tint * 0.5,
                0.9,
            );
            spawn_enemy_glow(
                engine, chars[idx % chars.len()], p, color,
                em * wave, gs * 0.9, Vec3::new(1.0, 0.15, 0.05), 0.6,
            );
            idx += 1;
        }
    }

    // Floating crown — 8 glyphs orbiting above center
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * TAU + time * 1.2;
        let r = 0.6;
        let base = Vec3::new(
            angle.cos() * r,
            2.2 + (time * 1.5).sin() * 0.2,
            0.0,
        ) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let flash = ((frame as f32 * 0.1 + i as f32).sin() * 0.3 + 0.7).max(0.3);
        let color = Vec4::new(1.0 * flash, 0.8 * flash, 0.2 * flash, 1.0);
        spawn_enemy_glow(
            engine, chars[idx % chars.len()], p, color,
            em * 1.5, gs * 1.1, Vec3::new(1.0, 0.7, 0.1), 1.0,
        );
        idx += 1;
    }
}

// ── Abomination: Massive chaotic mass (85 glyphs) ──────────────────────────
// Triple-layer rings + chaotic tendrils + pulsing core.

fn render_abomination(
    engine: &mut ProofEngine,
    chars: &[char],
    pos: Vec3,
    hp: f32,
    time: f32,
    frame: u64,
    tier: EnemyTier,
) {
    let scale = breathe(time, 0.5, 0.05);
    let base_color = tier.base_color();
    let em = tier.emission();
    let gs = tier.glyph_scale();
    let mut idx = 0usize;

    // Pulsing core — 10 glyphs in a tight throbbing cluster
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * TAU;
        let r = 0.35 + (time * 3.0 + i as f32 * 0.9).sin().abs() * 0.2;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let throb = ((time * 4.0 + i as f32).sin() * 0.3 + 0.7).max(0.2);
        let color = Vec4::new(1.0 * throb, 0.05, 0.05, 1.0);
        spawn_enemy_glow(
            engine, chars[idx % chars.len()], p, color,
            em * 1.8 * throb, gs * 1.1, Vec3::new(1.0, 0.1, 0.05), 1.2,
        );
        idx += 1;
    }

    // Three concentric rings: 15 + 20 + 25 = 60 glyphs
    let ring_configs: [(usize, f32, f32); 3] = [
        (15, 1.0, 0.4),   // inner ring
        (20, 1.8, -0.3),  // middle ring (counter-rotating)
        (25, 2.6, 0.2),   // outer ring
    ];
    for &(count, radius, rot_speed) in &ring_configs {
        for i in 0..count {
            let angle = (i as f32 / count as f32) * TAU + time * rot_speed;
            let wobble = ((time * 2.0 + idx as f32 * 1.3).sin() * 0.15).abs();
            let r = radius + wobble;
            let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let wave = ((time * 1.8 + idx as f32 * 0.4).sin() * 0.2 + 0.8).max(0.3);
            let color = Vec4::new(
                base_color.x * wave,
                base_color.y * wave,
                base_color.z * wave,
                0.9,
            );
            spawn_enemy_glow(
                engine, chars[idx % chars.len()], p, color,
                em * wave, gs, Vec3::new(0.9, 0.1, 0.08), 0.6,
            );
            idx += 1;
        }
    }

    // Chaotic tendrils — 15 glyphs reaching outward at irregular angles
    for i in 0..15 {
        let seed = i as f32 * 2.618;
        let tendril_angle = seed * TAU * 0.618 + time * 0.15;
        let reach = 3.0 + (time * 1.2 + seed).sin() * 0.8;
        let lateral = (time * 2.5 + seed * 3.0).sin() * 0.4;
        let base = Vec3::new(
            tendril_angle.cos() * reach + lateral,
            tendril_angle.sin() * reach,
            0.0,
        ) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let fade = ((time * 3.0 + seed).sin() * 0.3 + 0.6).max(0.15);
        let color = Vec4::new(
            base_color.x * fade,
            base_color.y * fade + 0.05,
            base_color.z * fade + 0.08,
            0.7,
        );
        engine.spawn_glyph(Glyph {
            character: chars[idx % chars.len()],
            position: p,
            color,
            emission: em * fade * 1.3,
            scale: Vec2::new(gs * 0.8, gs * 0.8),
            glow_color: Vec3::new(1.0, 0.15, 0.1),
            glow_radius: 0.4,
            temperature: 0.8,
            entropy: 0.6,
            visible: (frame + i as u64) % 3 != 0,
            layer: RenderLayer::Entity,
            ..Default::default()
        });
        idx += 1;
    }
}
