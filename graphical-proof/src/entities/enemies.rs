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

// ════════════════════════════════════════════════════════════════════════════
//  PART 2 — Element system, boss profiles, spawn/death, AmorphousEntity
// ════════════════════════════════════════════════════════════════════════════

// ── Enemy element ────────────────────────────────────────────────────────────

/// Element type driving enemy visual theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyElement {
    Fire,
    Ice,
    Lightning,
    Poison,
    Shadow,
    Holy,
    Neutral,
}

impl EnemyElement {
    /// Primary color for this element.
    pub fn primary_color(&self) -> Vec4 {
        match self {
            EnemyElement::Fire      => Vec4::new(1.0, 0.45, 0.1, 1.0),
            EnemyElement::Ice       => Vec4::new(0.5, 0.75, 1.0, 1.0),
            EnemyElement::Lightning => Vec4::new(1.0, 1.0, 0.3, 1.0),
            EnemyElement::Poison    => Vec4::new(0.3, 0.8, 0.2, 1.0),
            EnemyElement::Shadow    => Vec4::new(0.3, 0.15, 0.45, 1.0),
            EnemyElement::Holy      => Vec4::new(1.0, 0.95, 0.7, 1.0),
            EnemyElement::Neutral   => Vec4::new(0.7, 0.25, 0.2, 1.0),
        }
    }

    /// Accent / secondary color.
    pub fn accent_color(&self) -> Vec4 {
        match self {
            EnemyElement::Fire      => Vec4::new(1.0, 0.7, 0.0, 1.0),
            EnemyElement::Ice       => Vec4::new(0.8, 0.9, 1.0, 1.0),
            EnemyElement::Lightning => Vec4::new(1.0, 1.0, 0.8, 1.0),
            EnemyElement::Poison    => Vec4::new(0.5, 0.2, 0.7, 1.0),
            EnemyElement::Shadow    => Vec4::new(0.15, 0.05, 0.25, 1.0),
            EnemyElement::Holy      => Vec4::new(1.0, 1.0, 1.0, 1.0),
            EnemyElement::Neutral   => Vec4::new(0.9, 0.4, 0.3, 1.0),
        }
    }

    /// Glyph palette for this element.
    pub fn glyph_palette(&self) -> &'static [char] {
        match self {
            EnemyElement::Fire      => &['^', '*', '~', '#', '!', 'v', '>', '<'],
            EnemyElement::Ice       => &['*', '+', '.', ':', '#', '=', '-', 'o'],
            EnemyElement::Lightning => &['!', '/', '\\', 'X', '+', '#', '|', '-'],
            EnemyElement::Poison    => &['~', '.', ':', ';', '?', '%', '&', 'S'],
            EnemyElement::Shadow    => &['.', ' ', ':', '`', '\'', ',', '-', '~'],
            EnemyElement::Holy      => &['*', '+', '.', '\'', ':', '!', '#', '^'],
            EnemyElement::Neutral   => &['#', 'X', '+', '-', '|', '/', '\\', '.'],
        }
    }

    /// Death style for this element.
    pub fn death_style(&self) -> ElementalDeathStyle {
        match self {
            EnemyElement::Fire      => ElementalDeathStyle::Fire,
            EnemyElement::Ice       => ElementalDeathStyle::Ice,
            EnemyElement::Lightning => ElementalDeathStyle::Lightning,
            EnemyElement::Poison    => ElementalDeathStyle::Poison,
            EnemyElement::Shadow    => ElementalDeathStyle::Shadow,
            EnemyElement::Holy      => ElementalDeathStyle::Holy,
            EnemyElement::Neutral   => ElementalDeathStyle::Default,
        }
    }

    /// Glow color for this element.
    pub fn glow_color(&self) -> Vec3 {
        match self {
            EnemyElement::Fire      => Vec3::new(1.0, 0.4, 0.05),
            EnemyElement::Ice       => Vec3::new(0.4, 0.7, 1.0),
            EnemyElement::Lightning => Vec3::new(1.0, 1.0, 0.5),
            EnemyElement::Poison    => Vec3::new(0.3, 0.8, 0.2),
            EnemyElement::Shadow    => Vec3::new(0.2, 0.05, 0.3),
            EnemyElement::Holy      => Vec3::new(1.0, 0.95, 0.7),
            EnemyElement::Neutral   => Vec3::new(0.8, 0.2, 0.1),
        }
    }

    /// Preferred formation shape for this element.
    pub fn preferred_formation(&self) -> FormationShape {
        match self {
            EnemyElement::Fire      => FormationShape::Triangle,
            EnemyElement::Ice       => FormationShape::Diamond,
            EnemyElement::Lightning => FormationShape::Star,
            EnemyElement::Poison    => FormationShape::Spiral,
            EnemyElement::Shadow    => FormationShape::Crescent,
            EnemyElement::Holy      => FormationShape::Ring,
            EnemyElement::Neutral   => FormationShape::Cluster,
        }
    }

    /// Emission intensity for this element.
    pub fn emission(&self) -> f32 {
        match self {
            EnemyElement::Fire      => 0.5,
            EnemyElement::Ice       => 0.3,
            EnemyElement::Lightning => 0.6,
            EnemyElement::Poison    => 0.2,
            EnemyElement::Shadow    => 0.1,
            EnemyElement::Holy      => 0.5,
            EnemyElement::Neutral   => 0.15,
        }
    }
}

/// Guess an element from an enemy name.
pub fn element_from_name(name: &str) -> EnemyElement {
    let lower = name.to_lowercase();
    if lower.contains("fire") || lower.contains("flame") || lower.contains("ember")
        || lower.contains("inferno") || lower.contains("pyro")
    { EnemyElement::Fire }
    else if lower.contains("ice") || lower.contains("frost") || lower.contains("crystal")
        || lower.contains("cryo") || lower.contains("frozen")
    { EnemyElement::Ice }
    else if lower.contains("lightning") || lower.contains("thunder") || lower.contains("volt")
        || lower.contains("shock") || lower.contains("spark")
    { EnemyElement::Lightning }
    else if lower.contains("poison") || lower.contains("toxic") || lower.contains("venom")
        || lower.contains("acid") || lower.contains("plague")
    { EnemyElement::Poison }
    else if lower.contains("shadow") || lower.contains("dark") || lower.contains("void")
        || lower.contains("night") || lower.contains("abyss")
    { EnemyElement::Shadow }
    else if lower.contains("holy") || lower.contains("light") || lower.contains("radiant")
        || lower.contains("divine") || lower.contains("sacred")
    { EnemyElement::Holy }
    else { EnemyElement::Neutral }
}

// ── Boss visual profiles ─────────────────────────────────────────────────────

/// Unique boss visual identifiers (matching proof-engine BossType).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossVisualProfile {
    Mirror,
    Null,
    Committee,
    FibonacciHydra,
    Eigenstate,
    Ouroboros,
    AlgorithmReborn,
    ChaosWeaver,
    VoidSerpent,
    PrimeFactorial,
}

impl BossVisualProfile {
    /// Map a boss name string to a profile.
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        if lower.contains("mirror") { Some(BossVisualProfile::Mirror) }
        else if lower.contains("null") { Some(BossVisualProfile::Null) }
        else if lower.contains("committee") || lower.contains("judge") { Some(BossVisualProfile::Committee) }
        else if lower.contains("fibonacci") || lower.contains("hydra") { Some(BossVisualProfile::FibonacciHydra) }
        else if lower.contains("eigen") { Some(BossVisualProfile::Eigenstate) }
        else if lower.contains("ouroboros") { Some(BossVisualProfile::Ouroboros) }
        else if lower.contains("algorithm") || lower.contains("reborn") { Some(BossVisualProfile::AlgorithmReborn) }
        else if lower.contains("chaos") || lower.contains("weaver") { Some(BossVisualProfile::ChaosWeaver) }
        else if lower.contains("void") || lower.contains("serpent") { Some(BossVisualProfile::VoidSerpent) }
        else if lower.contains("prime") || lower.contains("factorial") { Some(BossVisualProfile::PrimeFactorial) }
        else { None }
    }

    /// Glyph count for this boss.
    pub fn glyph_count(&self) -> usize {
        match self {
            BossVisualProfile::Mirror          => 18,
            BossVisualProfile::Null            => 15,
            BossVisualProfile::Committee       => 25,
            BossVisualProfile::FibonacciHydra  => 21,
            BossVisualProfile::Eigenstate      => 20,
            BossVisualProfile::Ouroboros       => 24,
            BossVisualProfile::AlgorithmReborn => 30,
            BossVisualProfile::ChaosWeaver     => 22,
            BossVisualProfile::VoidSerpent     => 28,
            BossVisualProfile::PrimeFactorial  => 20,
        }
    }

    /// Formation shape for this boss at the given phase.
    pub fn formation(&self, phase: u32) -> FormationShape {
        match self {
            BossVisualProfile::Mirror          => FormationShape::Diamond,
            BossVisualProfile::Null            => FormationShape::Ring,
            BossVisualProfile::Committee       => FormationShape::Semicircle,
            BossVisualProfile::FibonacciHydra  => if phase == 0 { FormationShape::Cluster } else { FormationShape::Swarm },
            BossVisualProfile::Eigenstate      => if phase % 2 == 0 { FormationShape::Star } else { FormationShape::Diamond },
            BossVisualProfile::Ouroboros       => FormationShape::Ring,
            BossVisualProfile::AlgorithmReborn => match phase { 0 => FormationShape::Grid, 1 => FormationShape::Diamond, 2 => FormationShape::Star, _ => FormationShape::Pentagram },
            BossVisualProfile::ChaosWeaver     => [FormationShape::Star, FormationShape::Spiral, FormationShape::Cross, FormationShape::Triangle, FormationShape::Pentagon][(phase as usize) % 5],
            BossVisualProfile::VoidSerpent     => FormationShape::Snake,
            BossVisualProfile::PrimeFactorial  => FormationShape::Grid,
        }
    }

    /// Formation scale for this boss.
    pub fn formation_scale(&self) -> f32 {
        match self {
            BossVisualProfile::Mirror          => 1.5,
            BossVisualProfile::Null            => 1.3,
            BossVisualProfile::Committee       => 2.5,
            BossVisualProfile::FibonacciHydra  => 2.0,
            BossVisualProfile::Eigenstate      => 1.6,
            BossVisualProfile::Ouroboros       => 2.0,
            BossVisualProfile::AlgorithmReborn => 2.5,
            BossVisualProfile::ChaosWeaver     => 1.8,
            BossVisualProfile::VoidSerpent     => 3.0,
            BossVisualProfile::PrimeFactorial  => 1.5,
        }
    }
}

// ── Enemy visual state (full-featured) ───────────────────────────────────────

/// Full enemy visual state tracked across frames.
#[derive(Clone)]
pub struct EnemyVisualState {
    pub name: String,
    pub tier: EnemyTier,
    pub element: EnemyElement,
    pub boss_profile: Option<BossVisualProfile>,
    pub hp_frac: f32,
    pub phase: u32,
    pub spawn_t: f32,
    pub death_t: f32,
    pub hit_reaction_t: f32,
    pub time: f32,
    pub is_alive: bool,
}

impl EnemyVisualState {
    pub fn new(name: &str, tier: u32, element: EnemyElement) -> Self {
        let et = EnemyTier::from_tier(tier);
        let boss_profile = BossVisualProfile::from_name(name);
        Self {
            name: name.to_string(),
            tier: et,
            element,
            boss_profile,
            hp_frac: 1.0,
            phase: 0,
            spawn_t: 0.0,
            death_t: 0.0,
            hit_reaction_t: 1.0,
            time: 0.0,
            is_alive: true,
        }
    }

    pub fn from_name(name: &str, tier: u32) -> Self {
        Self::new(name, tier, element_from_name(name))
    }

    /// Advance by dt seconds.
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;
        if self.spawn_t < 1.0 { self.spawn_t = (self.spawn_t + dt * 2.0).min(1.0); }
        if !self.is_alive && self.death_t < 1.0 { self.death_t = (self.death_t + dt * 0.8).min(1.0); }
        if self.hit_reaction_t < 1.0 { self.hit_reaction_t = (self.hit_reaction_t + dt * 5.0).min(1.0); }
    }

    pub fn trigger_hit(&mut self) { self.hit_reaction_t = 0.0; }

    pub fn trigger_death(&mut self) {
        self.is_alive = false;
        self.death_t = 0.0;
    }

    pub fn set_phase(&mut self, phase: u32) { self.phase = phase; }

    pub fn is_death_complete(&self) -> bool { !self.is_alive && self.death_t >= 1.0 }
}

// ── Full-featured render entry point ─────────────────────────────────────────

/// Render an enemy with full visual state: element theme, spawn/death
/// animations, boss-specific profiles, and hit reactions.
pub fn render_enemy_full(
    engine: &mut ProofEngine,
    state: &EnemyVisualState,
    position: Vec3,
    frame: u64,
) {
    let time = state.time;
    let element = state.element;
    let primary = element.primary_color();
    let accent = element.accent_color();
    let glow = element.glow_color();
    let em = element.emission() + state.tier.emission();

    // Determine formation positions
    let formation_shape = if let Some(bp) = state.boss_profile {
        bp.formation(state.phase)
    } else {
        element.preferred_formation()
    };
    let glyph_count = if let Some(bp) = state.boss_profile {
        bp.glyph_count()
    } else {
        state.tier.glyph_count()
    };
    let scale = if let Some(bp) = state.boss_profile {
        bp.formation_scale()
    } else {
        match state.tier {
            EnemyTier::Minion      => 0.6,
            EnemyTier::Elite       => 1.0,
            EnemyTier::Champion    => 1.3,
            EnemyTier::Boss        => 1.8,
            EnemyTier::Abomination => 2.2,
        }
    };

    let target_positions = formation_shape.generate_positions(glyph_count, scale);

    // Spawn animation: glyphs expand from center
    let mut positions = if state.spawn_t < 1.0 {
        formations::spawn_animation(&target_positions, state.spawn_t)
    } else {
        target_positions.clone()
    };

    // Idle animation (element-specific)
    if state.is_alive {
        positions = apply_element_idle(&positions, element, time);
    }

    // HP-based drift
    positions = formations::apply_hp_drift(&positions, state.hp_frac, time);

    // Hit reaction
    if state.hit_reaction_t < 1.0 {
        positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.4);
    }

    // Boss-specific animation
    if let Some(bp) = state.boss_profile {
        positions = apply_boss_anim(&positions, bp, state.phase, time);
    }

    // Death dissolution
    if !state.is_alive {
        let style = element.death_style();
        positions = positions
            .iter()
            .enumerate()
            .map(|(i, p)| style.modify_death_pos(*p, state.death_t, i))
            .collect();
    }

    // Build char palette
    let palette = element.glyph_palette();
    let name_chars: Vec<char> = state.name.chars().filter(|c| !c.is_whitespace()).take(4).collect();

    // Death color
    let death_style = element.death_style();

    for (i, p) in positions.iter().enumerate() {
        let world_pos = position + *p;

        // Pick character from name chars first, then element palette
        let ch = if i < name_chars.len() {
            name_chars[i]
        } else {
            palette[i % palette.len()]
        };

        // Color: blend primary/accent based on distance from center
        let dist = p.length();
        let t = (dist / (scale * 1.5)).clamp(0.0, 1.0);
        let mut color = lerp_color(primary, accent, t);

        // Spawn flash
        if state.spawn_t < 1.0 {
            let flash = (1.0 - state.spawn_t) * 0.5;
            color.x = (color.x + flash).min(1.0);
            color.y = (color.y + flash).min(1.0);
            color.z = (color.z + flash).min(1.0);
        }

        // Death color
        if !state.is_alive {
            color = death_style.death_color(color, state.death_t);
        }

        // Glow pulse for elite+ tiers
        let glow_r = match state.tier {
            EnemyTier::Minion => 0.0,
            EnemyTier::Elite => 0.3,
            EnemyTier::Champion => 0.5,
            EnemyTier::Boss => 0.8,
            EnemyTier::Abomination => 1.0,
        };

        if glow_r > 0.01 {
            spawn_enemy_glow(engine, ch, world_pos, color, em, state.tier.glyph_scale(), glow, glow_r);
        } else {
            spawn_enemy(engine, ch, world_pos, color, em, state.tier.glyph_scale());
        }
    }
}

// ── Element-specific idle animation ──────────────────────────────────────────

fn apply_element_idle(positions: &[Vec3], element: EnemyElement, time: f32) -> Vec<Vec3> {
    match element {
        EnemyElement::Fire => {
            // Flickering upward drift
            positions.iter().enumerate().map(|(i, p)| {
                let flicker = (time * 6.0 + i as f32 * 2.1).sin() * 0.05;
                let rise = ((time * TAU + i as f32).sin() * 0.04).abs();
                *p + Vec3::new(flicker, rise, 0.0)
            }).collect()
        }
        EnemyElement::Ice => {
            // Slow crystalline pulse
            formations::apply_breathing(positions, time, 0.4, 0.03)
        }
        EnemyElement::Lightning => {
            // Jittery flicker
            positions.iter().enumerate().map(|(i, p)| {
                let jx = (time * 15.0 + i as f32 * 3.7).sin() * 0.03;
                let jy = (time * 12.0 + i as f32 * 5.1).cos() * 0.03;
                *p + Vec3::new(jx, jy, 0.0)
            }).collect()
        }
        EnemyElement::Poison => {
            // Bubbling
            positions.iter().enumerate().map(|(i, p)| {
                let bubble = ((time * 3.0 + i as f32 * 1.3).sin() * 0.04).max(0.0);
                *p + Vec3::new(0.0, bubble, 0.0)
            }).collect()
        }
        EnemyElement::Shadow => {
            // Undulating tendrils
            positions.iter().enumerate().map(|(i, p)| {
                let wave = (time * 1.5 + p.y * 2.0 + i as f32 * 0.5).sin() * 0.06;
                *p + Vec3::new(wave, 0.0, 0.0)
            }).collect()
        }
        EnemyElement::Holy => {
            // Radiant center pulse
            positions.iter().map(|p| {
                let dist = p.length();
                let wave = (time * TAU - dist * 3.0).sin() * 0.04;
                *p * (1.0 + wave)
            }).collect()
        }
        EnemyElement::Neutral => {
            formations::apply_breathing(positions, time, 0.6, 0.03)
        }
    }
}

// ── Boss-specific animation ──────────────────────────────────────────────────

fn apply_boss_anim(
    positions: &[Vec3],
    profile: BossVisualProfile,
    phase: u32,
    time: f32,
) -> Vec<Vec3> {
    match profile {
        BossVisualProfile::Mirror => {
            // Subtle mirror shimmer
            positions.iter().enumerate().map(|(i, p)| {
                let shimmer = (time * 3.0 + i as f32 * 0.8).sin() * 0.02;
                Vec3::new(p.x + shimmer, p.y, p.z)
            }).collect()
        }
        BossVisualProfile::Null => {
            // Void contract/expand
            let pulse = (time * 0.8).sin() * 0.15;
            positions.iter().map(|p| *p * (1.0 + pulse)).collect()
        }
        BossVisualProfile::Committee => {
            // Each judge bobs independently
            let count = positions.len();
            positions.iter().enumerate().map(|(i, p)| {
                let judge = (i * 5) / count.max(1);
                let bob = (time * 1.5 + judge as f32 * 1.2).sin() * 0.08;
                *p + Vec3::new(0.0, bob, 0.0)
            }).collect()
        }
        BossVisualProfile::FibonacciHydra => {
            // Heads sway
            let heads = (phase + 1).min(5) as usize;
            let hc = (positions.len() / heads).max(1);
            positions.iter().enumerate().map(|(i, p)| {
                let head = i / hc;
                let sway = (time * 2.0 + head as f32 * PI * 0.4).sin() * 0.1;
                *p + Vec3::new(sway, 0.0, 0.0)
            }).collect()
        }
        BossVisualProfile::Eigenstate => {
            // Two states shimmer
            positions.iter().enumerate().map(|(i, p)| {
                let offset = if i % 2 == 0 { 1.0 } else { -1.0 };
                let shift = (time * 1.0).sin() * 0.15 * offset;
                *p + Vec3::new(shift, 0.0, 0.0)
            }).collect()
        }
        BossVisualProfile::Ouroboros => {
            formations::apply_rotation(positions, time, 0.3)
        }
        BossVisualProfile::AlgorithmReborn => {
            let intensity = 0.03 + phase as f32 * 0.02;
            let pulse = (time * (1.0 + phase as f32 * 0.3)).sin() * intensity;
            positions.iter().map(|p| *p * (1.0 + pulse)).collect()
        }
        BossVisualProfile::ChaosWeaver => {
            positions.iter().enumerate().map(|(i, p)| {
                let seed = (i as u32).wrapping_mul(2654435761);
                let jx = (time * 8.0 + seed as f32 * 0.001).sin() * 0.08;
                let jy = (time * 7.0 + seed as f32 * 0.0013).cos() * 0.08;
                *p + Vec3::new(jx, jy, 0.0)
            }).collect()
        }
        BossVisualProfile::VoidSerpent => {
            // Sinusoidal body
            positions.iter().enumerate().map(|(i, p)| {
                let t = i as f32 / positions.len() as f32;
                let wave = (time * 2.0 + t * TAU * 1.5).sin() * 0.15;
                *p + Vec3::new(0.0, wave, 0.0)
            }).collect()
        }
        BossVisualProfile::PrimeFactorial => {
            // Grid pulses
            positions.iter().enumerate().map(|(i, p)| {
                let wave = (time * 3.0 + i as f32 * 0.5).sin() * 0.03;
                *p * (1.0 + wave)
            }).collect()
        }
    }
}

// ── AmorphousEntity builder (formation-backed) ──────────────────────────────

/// Build an AmorphousEntity for an enemy (backwards-compatible signature).
pub fn build_enemy_entity(name: &str, tier: u32, position: Vec3) -> AmorphousEntity {
    let enemy_tier = EnemyTier::from_tier(tier);
    let element = element_from_name(name);
    let boss_profile = BossVisualProfile::from_name(name);

    let (glyph_count, scale) = if let Some(bp) = boss_profile {
        (bp.glyph_count(), bp.formation_scale())
    } else {
        (enemy_tier.glyph_count(), match enemy_tier {
            EnemyTier::Minion      => 0.6,
            EnemyTier::Elite       => 1.0,
            EnemyTier::Champion    => 1.3,
            EnemyTier::Boss        => 1.8,
            EnemyTier::Abomination => 2.2,
        })
    };

    let formation_shape = if let Some(bp) = boss_profile {
        bp.formation(0)
    } else {
        element.preferred_formation()
    };

    let positions = formation_shape.generate_positions(glyph_count, scale);
    let palette = element.glyph_palette();
    let primary = element.primary_color();
    let accent = element.accent_color();
    let name_chars: Vec<char> = name.chars().filter(|c| !c.is_whitespace()).take(4).collect();

    let mut chars = Vec::with_capacity(glyph_count);
    let mut colors = Vec::with_capacity(glyph_count);

    for (i, p) in positions.iter().enumerate() {
        let ch = if i < name_chars.len() { name_chars[i] } else { palette[i % palette.len()] };
        chars.push(ch);

        let dist = p.length();
        let t = (dist / (scale * 1.5)).clamp(0.0, 1.0);
        colors.push(lerp_color(primary, accent, t));
    }

    let mut entity = AmorphousEntity::new(format!("enemy_{}", name), position);
    entity.entity_mass = 30.0 + tier as f32 * 10.0;
    entity.pulse_rate = 0.8;
    entity.pulse_depth = 0.04;
    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity
}

/// Update an existing AmorphousEntity from an EnemyVisualState.
pub fn update_enemy_entity(entity: &mut AmorphousEntity, state: &EnemyVisualState) {
    let formation_shape = if let Some(bp) = state.boss_profile {
        bp.formation(state.phase)
    } else {
        state.element.preferred_formation()
    };
    let glyph_count = if let Some(bp) = state.boss_profile {
        bp.glyph_count()
    } else {
        state.tier.glyph_count()
    };
    let scale = if let Some(bp) = state.boss_profile {
        bp.formation_scale()
    } else {
        match state.tier {
            EnemyTier::Minion      => 0.6,
            EnemyTier::Elite       => 1.0,
            EnemyTier::Champion    => 1.3,
            EnemyTier::Boss        => 1.8,
            EnemyTier::Abomination => 2.2,
        }
    };

    let target = formation_shape.generate_positions(glyph_count, scale);
    let mut positions = if state.spawn_t < 1.0 {
        formations::spawn_animation(&target, state.spawn_t)
    } else {
        target
    };

    if state.is_alive { positions = apply_element_idle(&positions, state.element, state.time); }
    positions = formations::apply_hp_drift(&positions, state.hp_frac, state.time);
    if state.hit_reaction_t < 1.0 { positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.4); }
    if let Some(bp) = state.boss_profile { positions = apply_boss_anim(&positions, bp, state.phase, state.time); }

    if !state.is_alive {
        let style = state.element.death_style();
        positions = positions.iter().enumerate().map(|(i, p)| style.modify_death_pos(*p, state.death_t, i)).collect();
    }

    let palette = state.element.glyph_palette();
    let primary = state.element.primary_color();
    let accent = state.element.accent_color();
    let name_chars: Vec<char> = state.name.chars().filter(|c| !c.is_whitespace()).take(4).collect();

    let len = positions.len();
    let mut chars = Vec::with_capacity(len);
    let mut colors = Vec::with_capacity(len);
    let death_style = state.element.death_style();

    for (i, p) in positions.iter().enumerate() {
        let ch = if i < name_chars.len() { name_chars[i] } else { palette[i % palette.len()] };
        chars.push(ch);

        let dist = p.length();
        let t = (dist / (scale * 1.5)).clamp(0.0, 1.0);
        let mut color = lerp_color(primary, accent, t);

        if state.spawn_t < 1.0 {
            let flash = (1.0 - state.spawn_t) * 0.5;
            color.x = (color.x + flash).min(1.0);
            color.y = (color.y + flash).min(1.0);
            color.z = (color.z + flash).min(1.0);
        }
        if !state.is_alive {
            color = death_style.death_color(color, state.death_t);
        }
        colors.push(color);
    }

    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity.hp = state.hp_frac * entity.max_hp;
    entity.update_cohesion();
}

// ── Classification utility ───────────────────────────────────────────────────

/// Classify an enemy by tier, element, and optional boss profile.
pub fn classify_enemy(name: &str, tier: u32) -> (EnemyTier, EnemyElement, Option<BossVisualProfile>) {
    let et = EnemyTier::from_tier(tier);
    let element = element_from_name(name);
    let boss = BossVisualProfile::from_name(name);
    (et, element, boss)
}

// ── Utility ──────────────────────────────────────────────────────────────────

fn lerp_color(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Convert hue [0,1] to a saturated RGBA color.
fn hue_to_color(hue: f32, saturation: f32) -> Vec4 {
    let h = hue * 6.0;
    let c = saturation;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Vec4::new(r, g, b, 1.0)
}
