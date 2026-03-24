//! Player entity rendering — 12 unique class formations.
//!
//! Each `CharacterClass` gets a visually distinct glyph formation with
//! class-specific symbols, color palettes, idle animations, and HP-linked
//! cohesion dynamics.
//!
//! ## Rendering modes
//!
//! * **Immediate-mode** — `render_player()` / `render_player_full()` spawn glyphs
//!   every frame via `engine.spawn_glyph()`. This is the primary render path.
//! * **Formation-backed** — `build_player_entity()` returns an `AmorphousEntity`
//!   whose formation / chars / colors describe the shape declaratively.
//!
//! ## Visual features
//!
//! * 6 archetype profiles (Warrior, Mage, Rogue, Cleric, Necromancer, Berserker)
//!   mapped from all 12 `CharacterClass` variants.
//! * Unique idle animation per archetype (breathing + class-specific).
//! * Combat stance, cast pose, hurt recoil, death dissolution.
//! * Equipment visualization: weapon glyph changes, armor density scales.
//! * Status effect overlays: burning, frozen, poisoned, blessed.
//! * HP-based cohesion: tight at full HP, drifting/wobbling near death.
//! * Level-up flash: formation tightens, new glyph added.
//! * Movement lean: trailing glyphs follow with spring physics.

use proof_engine::prelude::*;
use chaos_rpg_core::character::CharacterClass;
use std::f32::consts::{PI, TAU};

use super::formations::{
    self, ClassArchetype, FormationShape, PlayerAnimState,
};

// ── Public entry point ───────────────────────────────────────────────────────

/// Render the player entity for a single frame.
///
/// * `engine`   — proof engine handle for spawning glyphs.
/// * `class`    — which of the 12 classes to render.
/// * `position` — world-space center (typically `(-4, 0, 0)` in combat).
/// * `hp_frac`  — health fraction `[0.0, 1.0]` — controls formation cohesion.
/// * `frame`    — monotonic frame counter driving idle animations.
pub fn render_player(
    engine: &mut ProofEngine,
    class: CharacterClass,
    position: Vec3,
    hp_frac: f32,
    frame: u64,
) {
    let hp = hp_frac.clamp(0.0, 1.0);
    let time = frame as f32 / 60.0;

    match class {
        CharacterClass::Mage        => render_mage(engine, position, hp, time, frame),
        CharacterClass::Berserker   => render_berserker(engine, position, hp, time, frame),
        CharacterClass::Ranger      => render_ranger(engine, position, hp, time, frame),
        CharacterClass::Thief       => render_thief(engine, position, hp, time, frame),
        CharacterClass::Necromancer => render_necromancer(engine, position, hp, time, frame),
        CharacterClass::Alchemist   => render_alchemist(engine, position, hp, time, frame),
        CharacterClass::Paladin     => render_paladin(engine, position, hp, time, frame),
        CharacterClass::VoidWalker  => render_voidwalker(engine, position, hp, time, frame),
        CharacterClass::Warlord     => render_warlord(engine, position, hp, time, frame),
        CharacterClass::Trickster   => render_trickster(engine, position, hp, time, frame),
        CharacterClass::Runesmith   => render_runesmith(engine, position, hp, time, frame),
        CharacterClass::Chronomancer => render_chronomancer(engine, position, hp, time, frame),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Scatter offset applied when HP drops. At hp=1 offset is 0; at hp=0 offset is ~1.2.
fn scatter(hp: f32, idx: usize, time: f32) -> Vec3 {
    let chaos = (1.0 - hp) * 1.2;
    let seed = idx as f32 * 1.618;
    Vec3::new(
        (seed * 3.7 + time * 1.1).sin() * chaos,
        (seed * 2.3 + time * 0.9).cos() * chaos,
        0.0,
    )
}

/// Idle breathing scale oscillation.
fn breathe(time: f32, rate: f32, depth: f32) -> f32 {
    1.0 + (time * rate * TAU).sin() * depth
}

/// Spawn a single entity-layer glyph.
fn spawn(
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

/// Spawn an entity-layer glyph with glow properties.
fn spawn_glow(
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

// ── Mage: Loose diamond of arcane symbols (25 glyphs) ────────────────────────
// Blue-purple palette. Orbiting symbol glyphs at edges. High emission.

fn render_mage(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, frame: u64) {
    let glyphs: &[char] = &['*', '\u{25C6}', '\u{221E}', '\u{2202}', '\u{2211}'];
    let size: i32 = 3;
    let mut idx = 0usize;
    let scale = breathe(time, 0.8, 0.04);

    // Core diamond — ~21 positions from a diamond lattice
    for dy in -size..=size {
        let width = size - dy.abs();
        for dx in -width..=width {
            let base = Vec3::new(dx as f32 * 0.7, dy as f32 * 0.6, 0.0) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let pulse = ((time * 3.0 + idx as f32 * 0.5).sin() * 0.15 + 0.85).max(0.0);
            let color = Vec4::new(0.35 * pulse, 0.25 * pulse, 0.95 * pulse, 1.0);
            spawn_glow(
                engine, glyphs[idx % glyphs.len()], p, color,
                1.2, 0.9, Vec3::new(0.4, 0.2, 1.0), 0.6,
            );
            idx += 1;
        }
    }

    // 4 orbiting arcane symbols at the diamond edges
    let orbit_chars: &[char] = &['\u{2206}', '\u{03A9}', '\u{03C0}', '\u{222B}'];
    for i in 0..4 {
        let angle = (i as f32 / 4.0) * TAU + time * 1.5;
        let r = 2.2 + (time * 2.0 + i as f32).sin() * 0.3;
        let p = pos + Vec3::new(angle.cos() * r, angle.sin() * r, 0.0)
            + scatter(hp, idx + i, time);
        let flicker = ((frame as f32 * 0.2 + i as f32 * 1.2).sin() * 0.3 + 0.7).max(0.0);
        let color = Vec4::new(0.6 * flicker, 0.3 * flicker, 1.0 * flicker, 0.9);
        spawn_glow(
            engine, orbit_chars[i], p, color,
            1.8, 1.1, Vec3::new(0.5, 0.2, 1.0), 0.9,
        );
    }
}

// ── Berserker: Tight aggressive cluster (30 glyphs) ──────────────────────────
// Red palette. Below 30% HP: glow intensifies, emission doubles.

fn render_berserker(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let glyphs: &[char] = &['>', '<', '!', '#', '\u{2588}'];
    let rage_boost = if hp < 0.3 { 2.0 } else { 1.0 };
    let emission = 0.8 * rage_boost;
    let scale = breathe(time, 1.2, 0.03);
    let mut idx = 0usize;

    // Tight rectangular cluster: 5 rows x 6 cols = 30 glyphs
    for row in -2..=2 {
        for col in -2..=3 {
            let jitter_x = (idx as f32 * 2.71 + time * 4.0).sin() * 0.06 * rage_boost;
            let jitter_y = (idx as f32 * 3.14 + time * 3.5).cos() * 0.06 * rage_boost;
            let base = Vec3::new(
                col as f32 * 0.45 + jitter_x,
                row as f32 * 0.45 + jitter_y,
                0.0,
            ) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let r_val = (0.85 * rage_boost).min(1.0);
            let flicker = if hp < 0.3 {
                ((time * 8.0 + idx as f32).sin() * 0.2 + 0.8).max(0.4)
            } else {
                1.0
            };
            let color = Vec4::new(r_val * flicker, 0.15 * flicker, 0.1 * flicker, 1.0);
            let glow = Vec3::new(1.0, 0.15, 0.05) * rage_boost;
            spawn_glow(
                engine, glyphs[idx % glyphs.len()], p, color,
                emission, 0.85, glow, 0.5 * rage_boost,
            );
            idx += 1;
        }
    }
}

// ── Ranger: Arrow formation pointing right (20 glyphs) ──────────────────────
// Green palette. Precise geometric chevron.

fn render_ranger(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let glyphs: &[char] = &['/', '\\', '|', '\u{2192}'];
    let scale = breathe(time, 0.6, 0.03);
    let mut idx = 0usize;

    // Arrow shape: chevron tip + shaft + tail feathers
    let offsets: [(f32, f32); 20] = [
        // Tip
        (2.0, 0.0),
        // Upper wing
        (1.4, 0.4), (0.8, 0.8), (0.2, 1.2), (-0.4, 1.6),
        // Lower wing
        (1.4, -0.4), (0.8, -0.8), (0.2, -1.2), (-0.4, -1.6),
        // Shaft
        (1.0, 0.0), (0.4, 0.0), (-0.2, 0.0), (-0.8, 0.0), (-1.4, 0.0),
        // Inner chevron fill
        (1.0, 0.25), (1.0, -0.25), (0.4, 0.5), (0.4, -0.5),
        // Tail feathers
        (-1.4, 0.3), (-1.4, -0.3),
    ];

    for &(ox, oy) in &offsets {
        let base = Vec3::new(ox, oy, 0.0) * scale * 0.7;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(0.25, 0.82, 0.2, 1.0);
        spawn(engine, glyphs[idx % glyphs.len()], p, color, 0.5, 0.8);
        idx += 1;
    }
}

// ── Thief: Small compact cluster (15 glyphs) ────────────────────────────────
// Gray palette. Dim emission for stealth.

fn render_thief(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let glyphs: &[char] = &['.', '\u{00B7}', '~', '-'];
    let scale = breathe(time, 0.5, 0.02);
    let mut idx = 0usize;

    // 3 concentric micro-rings: 3 + 6 + 6 = 15 glyphs
    for ring in 0..3 {
        let r = (ring as f32 + 1.0) * 0.35;
        let count = if ring == 0 { 3 } else { 6 };
        for i in 0..count {
            if idx >= 15 { break; }
            let angle = (i as f32 / count as f32) * TAU + ring as f32 * 0.3;
            let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let dim = ((time * 1.5 + idx as f32 * 0.9).sin() * 0.1 + 0.4).max(0.2);
            let color = Vec4::new(0.5 * dim, 0.5 * dim, 0.5 * dim, 0.7);
            spawn(engine, glyphs[idx % glyphs.len()], p, color, 0.15, 0.65);
            idx += 1;
        }
    }
}

// ── Necromancer: Ring with dark center (25 glyphs) ──────────────────────────
// Dark green/purple palette. Occasional soul wisp particles.

fn render_necromancer(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, frame: u64) {
    let outer: &[char] = &['\u{2620}', '\u{2020}', '\u{00B7}', '\u{25CB}'];
    let scale = breathe(time, 0.7, 0.03);
    let mut idx = 0usize;

    // Dark center void — 5 dim glyphs
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * TAU;
        let r = 0.25;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(0.1, 0.05, 0.15, 0.6);
        spawn(engine, '\u{00B7}', p, color, 0.1, 0.7);
        idx += 1;
    }

    // Outer ring — 16 glyphs
    for i in 0..16 {
        let angle = (i as f32 / 16.0) * TAU + time * 0.4;
        let r = 1.4;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let pulse = ((time * 2.0 + i as f32 * 0.7).sin() * 0.2 + 0.8).max(0.0);
        let color = Vec4::new(0.2 * pulse, 0.6 * pulse, 0.25 * pulse, 1.0);
        spawn_glow(
            engine, outer[idx % outer.len()], p, color,
            0.7, 0.85, Vec3::new(0.3, 0.7, 0.4), 0.5,
        );
        idx += 1;
    }

    // Inner ring — 4 skulls
    for i in 0..4 {
        let angle = (i as f32 / 4.0) * TAU - time * 0.3;
        let r = 0.7;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(0.5, 0.2, 0.6, 0.9);
        spawn_glow(
            engine, '\u{2620}', p, color,
            0.9, 0.95, Vec3::new(0.6, 0.1, 0.8), 0.6,
        );
        idx += 1;
    }

    // Soul wisp particle every ~8 frames
    if frame % 8 == 0 {
        let wisp_angle = time * 2.5;
        let wisp_r = 1.8 + (time * 1.3).sin() * 0.5;
        let p = pos + Vec3::new(
            wisp_angle.cos() * wisp_r,
            wisp_angle.sin() * wisp_r + 0.5,
            0.0,
        );
        engine.spawn_glyph(Glyph {
            character: '\u{2022}',
            position: p,
            color: Vec4::new(0.4, 0.9, 0.5, 0.5),
            emission: 1.5,
            glow_color: Vec3::new(0.3, 1.0, 0.4),
            glow_radius: 1.0,
            lifetime: 0.6,
            life_function: Some(MathFunction::Breathing { rate: 4.0, depth: 0.6 }),
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ── Alchemist: Bubbling formation (20 glyphs) ───────────────────────────────
// Purple/gold palette. Bubbling idle animation.

fn render_alchemist(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let glyphs: &[char] = &['~', '\u{2248}', '\u{25CB}', '\u{25CF}'];
    let scale = breathe(time, 0.9, 0.04);
    let mut idx = 0usize;

    // Bottom row — 6 wide glyphs with upward bubble wobble
    for i in 0..6 {
        let x = (i as f32 - 2.5) * 0.5;
        let bubble_y = ((time * 3.0 + i as f32 * 1.1).sin() * 0.12).abs();
        let base = Vec3::new(x, -1.0 + bubble_y, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(0.65, 0.45, 0.9, 1.0);
        spawn(engine, glyphs[idx % glyphs.len()], p, color, 0.6, 0.85);
        idx += 1;
    }

    // Middle rows — 2 tapering rows (5 + 3 = 8 glyphs)
    for row in 0..2 {
        let half_w = 2 - row;
        for col in -half_w..=half_w {
            let bubble_y = ((time * 2.5 + idx as f32 * 0.8).sin() * 0.1).abs();
            let base = Vec3::new(
                col as f32 * 0.5,
                row as f32 * 0.5 + bubble_y,
                0.0,
            ) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let gold_mix = (idx as f32 * 0.3 + time).sin() * 0.5 + 0.5;
            let color = Vec4::new(
                0.65 + 0.25 * gold_mix,
                0.4 + 0.4 * gold_mix,
                0.9 - 0.5 * gold_mix,
                1.0,
            );
            spawn(engine, glyphs[idx % glyphs.len()], p, color, 0.7, 0.85);
            idx += 1;
        }
    }

    // Top bubbles — fill remaining to reach 20
    while idx < 20 {
        let i = idx - 14;
        let angle = (i as f32 / 6.0) * TAU + time * 1.8;
        let r = 0.4;
        let base = Vec3::new(angle.cos() * r, 1.2 + angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(0.85, 0.75, 0.3, 0.8);
        spawn(engine, '\u{25CB}', p, color, 0.9, 0.7);
        idx += 1;
    }
}

// ── Paladin: Cross / shield formation (25 glyphs) ───────────────────────────
// Golden palette. Steady warm glow.

fn render_paladin(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let glyphs: &[char] = &['+', '\u{2020}', '\u{25A0}', '\u{2588}'];
    let scale = breathe(time, 0.5, 0.02);
    let gold = Vec3::new(0.95, 0.85, 0.35);
    let mut idx = 0usize;

    // Vertical bar of the cross — 9 glyphs
    for row in -4..=4 {
        let base = Vec3::new(0.0, row as f32 * 0.45, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let bright = 0.85 + (time * 1.5 + idx as f32).sin() * 0.1;
        let color = Vec4::new(gold.x * bright, gold.y * bright, gold.z * bright, 1.0);
        spawn_glow(engine, glyphs[idx % glyphs.len()], p, color, 0.9, 0.9, gold, 0.5);
        idx += 1;
    }

    // Horizontal bar — 8 glyphs (skip center to avoid overlap)
    for col in -4..=4 {
        if col == 0 { continue; }
        let base = Vec3::new(col as f32 * 0.45, 0.45 * 0.5, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let bright = 0.85 + (time * 1.5 + idx as f32).sin() * 0.1;
        let color = Vec4::new(gold.x * bright, gold.y * bright, gold.z * bright, 1.0);
        spawn_glow(engine, glyphs[idx % glyphs.len()], p, color, 0.9, 0.9, gold, 0.5);
        idx += 1;
    }

    // Shield corners — 8 fill glyphs to reach 25
    let corners: [(f32, f32); 8] = [
        (-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0),
        (-1.0, 0.0), (1.0, 0.0), (0.0, 1.5), (0.0, -1.5),
    ];
    for &(cx, cy) in &corners {
        let base = Vec3::new(cx * 0.7, cy * 0.7, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(gold.x * 0.7, gold.y * 0.7, gold.z * 0.5, 0.85);
        spawn_glow(
            engine, '\u{25A0}', p, color,
            0.6, 0.8, gold * 0.7, 0.3,
        );
        idx += 1;
    }
}

// ── VoidWalker: Formation with gaps, phase-in/out (20 glyphs) ────────────────
// Purple palette. Some glyphs toggle visible based on frame.

fn render_voidwalker(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, frame: u64) {
    let glyphs: &[char] = &['\u{2591}', '\u{2592}', '\u{2593}', '\u{00B7}', '~'];
    let scale = breathe(time, 0.6, 0.05);
    let mut idx = 0usize;

    for i in 0..20 {
        // Each glyph has its own visibility period (prime-ish for visual variety)
        let phase_period = 17 + (i % 7) * 3;
        let visible = ((frame + i as u64 * 5) % phase_period as u64) > (phase_period as u64 / 3);
        if !visible {
            idx += 1;
            continue;
        }

        let angle = (i as f32 / 20.0) * TAU + time * 0.5;
        let gap = ((i as f32 * 1.618 + time * 2.0).sin() * 0.4).abs();
        let r = 1.2 + gap;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let flicker = ((time * 5.0 + i as f32 * 2.1).sin() * 0.3 + 0.6).max(0.1);
        let color = Vec4::new(0.55 * flicker, 0.15 * flicker, 0.85 * flicker, 0.7);
        spawn_glow(
            engine, glyphs[idx % glyphs.len()], p, color,
            1.0, 0.85, Vec3::new(0.6, 0.1, 0.9), 0.7,
        );
        idx += 1;
    }
}

// ── Warlord: Military grid (30 glyphs) ──────────────────────────────────────
// Steel gray palette. Rigid disciplined layout.

fn render_warlord(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let glyphs: &[char] = &['|', '\u{2500}', '\u{25A0}', '\u{25A1}'];
    let scale = breathe(time, 0.4, 0.015);
    let steel = Vec4::new(0.6, 0.62, 0.65, 1.0);
    let mut idx = 0usize;

    // 5 rows x 6 cols = 30 glyphs in strict military grid
    for row in -2i32..=2 {
        for col in -2..=3 {
            let base = Vec3::new(
                col as f32 * 0.5 - 0.25,
                row as f32 * 0.5,
                0.0,
            ) * scale;
            let p = pos + base + scatter(hp, idx, time);
            let row_i: i32 = row;
            let rank_bright = 1.0 - (row_i.abs() as f32 * 0.08);
            let color = Vec4::new(
                steel.x * rank_bright,
                steel.y * rank_bright,
                steel.z * rank_bright,
                1.0,
            );
            spawn(engine, glyphs[idx % glyphs.len()], p, color, 0.35, 0.85);
            idx += 1;
        }
    }
}

// ── Trickster: Shifting formation (15 glyphs) ───────────────────────────────
// Multi-colored palette. Glyphs swap positions each frame.

fn render_trickster(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, frame: u64) {
    let glyphs: &[char] = &['?', '!', '@', '&', '%'];
    let scale = breathe(time, 1.0, 0.04);

    // 15 base positions in a loose cluster
    let base_positions: [(f32, f32); 15] = [
        (0.0, 0.0), (0.6, 0.3), (-0.6, 0.3), (0.6, -0.3), (-0.6, -0.3),
        (0.0, 0.7), (0.0, -0.7), (1.0, 0.0), (-1.0, 0.0),
        (0.3, 1.0), (-0.3, 1.0), (0.3, -1.0), (-0.3, -1.0),
        (1.0, 0.6), (-1.0, -0.6),
    ];

    // Permutation: each glyph occupies a different slot each frame
    let shift = (frame % 15) as usize;

    for i in 0..15 {
        let slot = (i + shift) % 15;
        let (ox, oy) = base_positions[slot];
        let base = Vec3::new(ox, oy, 0.0) * scale;
        let p = pos + base + scatter(hp, i, time);
        let hue = ((i as f32 + frame as f32 * 0.1) % 5.0) / 5.0;
        let color = hue_to_rgba(hue);
        spawn(engine, glyphs[i % glyphs.len()], p, color, 0.8, 0.85);
    }
}

/// Convert a [0,1] hue value to a saturated RGBA color.
fn hue_to_rgba(h: f32) -> Vec4 {
    let h6 = h * 6.0;
    let frac = h6 - h6.floor();
    let (r, g, b) = match h6 as u32 % 6 {
        0 => (1.0, frac, 0.0),
        1 => (1.0 - frac, 1.0, 0.0),
        2 => (0.0, 1.0, frac),
        3 => (0.0, 1.0 - frac, 1.0),
        4 => (frac, 0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - frac),
    };
    Vec4::new(r, g, b, 1.0)
}

// ── Runesmith: Runic circle (25 glyphs) ─────────────────────────────────────
// Orange/amber palette. Glowing rune inscriptions.

fn render_runesmith(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let runes: &[char] = &['\u{16B1}', '\u{16A2}', '\u{16BE}', '#'];
    let scale = breathe(time, 0.6, 0.03);
    let amber = Vec3::new(0.95, 0.65, 0.2);
    let mut idx = 0usize;

    // Outer runic circle — 16 runes rotating slowly clockwise
    for i in 0..16 {
        let angle = (i as f32 / 16.0) * TAU + time * 0.25;
        let r = 1.5;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let heat = ((time * 2.0 + i as f32 * 0.5).sin() * 0.15 + 0.85).max(0.0);
        let color = Vec4::new(amber.x * heat, amber.y * heat, amber.z * heat * 0.7, 1.0);
        spawn_glow(
            engine, runes[idx % runes.len()], p, color,
            0.9, 0.9, Vec3::new(1.0, 0.5, 0.1), 0.5,
        );
        idx += 1;
    }

    // Inner circle — 6 runes counter-rotating
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * TAU - time * 0.4;
        let r = 0.7;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(1.0, 0.75, 0.3, 1.0);
        spawn_glow(
            engine, runes[idx % runes.len()], p, color,
            1.1, 1.0, Vec3::new(1.0, 0.6, 0.15), 0.6,
        );
        idx += 1;
    }

    // Center rune cluster — 3 stacked for depth
    for i in 0..3 {
        let wobble = (time * 1.5 + i as f32 * TAU / 3.0).sin() * 0.15;
        let base = Vec3::new(wobble, wobble * 0.5, 0.0) * scale;
        let p = pos + base + scatter(hp, idx, time);
        let color = Vec4::new(1.0, 0.85, 0.4, 1.0);
        spawn_glow(
            engine, '#', p, color,
            1.4, 1.1, Vec3::new(1.0, 0.7, 0.2), 0.8,
        );
        idx += 1;
    }
}

// ── Chronomancer: Time-offset breathing (20 glyphs) ─────────────────────────
// Blue/white palette. Each glyph breathes at a different phase.

fn render_chronomancer(engine: &mut ProofEngine, pos: Vec3, hp: f32, time: f32, _frame: u64) {
    let glyphs: &[char] = &['\u{29D6}', '\u{25CB}', '\u{00B7}', '\u{2234}'];
    let mut idx = 0usize;

    // 20 glyphs in a clock-like ring, phase-staggered breathing
    for i in 0..20 {
        let angle = (i as f32 / 20.0) * TAU;
        let phase_offset = (i as f32 / 20.0) * TAU;
        let local_scale = 1.0 + (time * 1.2 + phase_offset).sin() * 0.12;
        let r = 1.3 * local_scale;
        let base = Vec3::new(angle.cos() * r, angle.sin() * r, 0.0);
        let p = pos + base + scatter(hp, idx, time);

        // Blue-white gradient: early indices whiter, later indices bluer
        let white_mix = (20 - i) as f32 / 20.0;
        let alpha = 0.85 + (time * 1.2 + phase_offset).sin() * 0.15;
        let color = Vec4::new(
            0.4 + 0.6 * white_mix,
            0.5 + 0.5 * white_mix,
            0.95,
            alpha.clamp(0.0, 1.0),
        );
        let em = (0.7 + (time * 1.2 + phase_offset).sin() * 0.4).max(0.2);
        spawn_glow(
            engine, glyphs[idx % glyphs.len()], p, color,
            em, 0.85, Vec3::new(0.5, 0.7, 1.0), 0.5,
        );
        idx += 1;
    }
}

// ════════════════════════════════════════════════════════════════════════════
//  PART 2 — Full-featured rendering with status effects, poses, equipment
// ════════════════════════════════════════════════════════════════════════════

// ── Status effect visuals ────────────────────────────────────────────────────

/// Active status effects that modify glyph colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffect {
    None,
    Burning,
    Frozen,
    Poisoned,
    Blessed,
}

/// Equipment tier affecting glyph density and armor chars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentTier {
    Bare,
    Light,
    Medium,
    Heavy,
}

// ── Class-to-archetype mapping ───────────────────────────────────────────────

/// Map any CharacterClass to one of six visual archetypes.
pub fn class_archetype(class: CharacterClass) -> ClassArchetype {
    match class {
        CharacterClass::Berserker   => ClassArchetype::Berserker,
        CharacterClass::Mage        => ClassArchetype::Mage,
        CharacterClass::Thief       => ClassArchetype::Rogue,
        CharacterClass::Paladin     => ClassArchetype::Cleric,
        CharacterClass::Necromancer => ClassArchetype::Necromancer,
        CharacterClass::Ranger      => ClassArchetype::Rogue,
        CharacterClass::Alchemist   => ClassArchetype::Mage,
        CharacterClass::VoidWalker  => ClassArchetype::Necromancer,
        CharacterClass::Warlord     => ClassArchetype::Warrior,
        CharacterClass::Trickster   => ClassArchetype::Rogue,
        CharacterClass::Runesmith   => ClassArchetype::Warrior,
        CharacterClass::Chronomancer => ClassArchetype::Mage,
    }
}

// ── Archetype color palettes ─────────────────────────────────────────────────

fn archetype_base_color(arch: ClassArchetype) -> Vec4 {
    match arch {
        ClassArchetype::Warrior     => Vec4::new(0.7, 0.65, 0.55, 1.0),
        ClassArchetype::Mage        => Vec4::new(0.35, 0.25, 0.9, 1.0),
        ClassArchetype::Rogue       => Vec4::new(0.45, 0.45, 0.5, 1.0),
        ClassArchetype::Cleric      => Vec4::new(0.95, 0.9, 0.5, 1.0),
        ClassArchetype::Necromancer => Vec4::new(0.3, 0.65, 0.3, 1.0),
        ClassArchetype::Berserker   => Vec4::new(0.9, 0.25, 0.15, 1.0),
    }
}

fn archetype_accent_color(arch: ClassArchetype) -> Vec4 {
    match arch {
        ClassArchetype::Warrior     => Vec4::new(0.9, 0.85, 0.6, 1.0),
        ClassArchetype::Mage        => Vec4::new(0.6, 0.4, 1.0, 1.0),
        ClassArchetype::Rogue       => Vec4::new(0.2, 0.2, 0.25, 1.0),
        ClassArchetype::Cleric      => Vec4::new(1.0, 1.0, 0.85, 1.0),
        ClassArchetype::Necromancer => Vec4::new(0.5, 0.1, 0.6, 1.0),
        ClassArchetype::Berserker   => Vec4::new(1.0, 0.6, 0.1, 1.0),
    }
}

fn archetype_glow_color(arch: ClassArchetype) -> Vec3 {
    match arch {
        ClassArchetype::Warrior     => Vec3::new(0.8, 0.7, 0.4),
        ClassArchetype::Mage        => Vec3::new(0.4, 0.2, 1.0),
        ClassArchetype::Rogue       => Vec3::new(0.2, 0.2, 0.3),
        ClassArchetype::Cleric      => Vec3::new(1.0, 0.95, 0.6),
        ClassArchetype::Necromancer => Vec3::new(0.3, 0.7, 0.4),
        ClassArchetype::Berserker   => Vec3::new(1.0, 0.3, 0.1),
    }
}

fn archetype_emission(arch: ClassArchetype) -> f32 {
    match arch {
        ClassArchetype::Warrior     => 0.4,
        ClassArchetype::Mage        => 1.2,
        ClassArchetype::Rogue       => 0.15,
        ClassArchetype::Cleric      => 0.9,
        ClassArchetype::Necromancer => 0.7,
        ClassArchetype::Berserker   => 0.8,
    }
}

// ── Glyph palettes per archetype ─────────────────────────────────────────────

fn archetype_body_glyphs(arch: ClassArchetype) -> &'static [char] {
    match arch {
        ClassArchetype::Warrior     => &['|', '-', '+', '=', '#', '/', '\\'],
        ClassArchetype::Mage        => &['*', '~', '`', '\'', '.', ':', ';'],
        ClassArchetype::Rogue       => &['.', '`', '\'', ',', ':', ';', '-'],
        ClassArchetype::Cleric      => &['+', '|', '-', '*', '.', ':', '\''],
        ClassArchetype::Necromancer => &['#', 'X', 'x', '+', '-', '|', '%'],
        ClassArchetype::Berserker   => &['>', '<', '!', '#', '|', '/', '\\'],
    }
}

fn archetype_weapon_glyphs(arch: ClassArchetype) -> &'static [char] {
    match arch {
        ClassArchetype::Warrior     => &['|', '/', '\\', '>', '<', '#'],
        ClassArchetype::Mage        => &['|', '*', '+', '=', '^'],
        ClassArchetype::Rogue       => &['/', '\\', '|', '<', '>'],
        ClassArchetype::Cleric      => &['+', '*', '#', '!', '^'],
        ClassArchetype::Necromancer => &['#', 'X', '+', '|', '-'],
        ClassArchetype::Berserker   => &['/', '\\', '|', '>', '<', 'X'],
    }
}

fn archetype_armor_glyphs(arch: ClassArchetype, tier: EquipmentTier) -> &'static [char] {
    match tier {
        EquipmentTier::Bare => &['.', ' ', '`'],
        EquipmentTier::Light => match arch {
            ClassArchetype::Warrior     => &['.', '-', '~'],
            ClassArchetype::Mage        => &['~', '`', '\''],
            ClassArchetype::Rogue       => &['.', '`', '\''],
            ClassArchetype::Cleric      => &['.', '\'', ':'],
            ClassArchetype::Necromancer => &['.', ':', '~'],
            ClassArchetype::Berserker   => &['.', '-', '|'],
        },
        EquipmentTier::Medium => match arch {
            ClassArchetype::Warrior     => &['=', '-', '#'],
            ClassArchetype::Mage        => &['~', '=', '+'],
            ClassArchetype::Rogue       => &['-', '=', '~'],
            ClassArchetype::Cleric      => &['+', '=', '#'],
            ClassArchetype::Necromancer => &['=', '#', '-'],
            ClassArchetype::Berserker   => &['#', '=', '+'],
        },
        EquipmentTier::Heavy => &['#', '=', '+', '|', '-'],
    }
}

// ── PlayerVisualState ────────────────────────────────────────────────────────

/// Full player visual state tracked across frames.
#[derive(Clone)]
pub struct PlayerVisualState {
    pub class: CharacterClass,
    pub archetype: ClassArchetype,
    pub anim_state: PlayerAnimState,
    pub prev_anim_state: PlayerAnimState,
    pub transition_t: f32,
    pub status_effect: StatusEffect,
    pub equipment_tier: EquipmentTier,
    pub weapon_glyph_override: Option<char>,
    pub hp_frac: f32,
    pub level: u32,
    pub hit_reaction_t: f32,
    pub death_t: f32,
    pub level_up_t: f32,
    pub velocity: Vec3,
    pub time: f32,
}

impl PlayerVisualState {
    pub fn new(class: CharacterClass) -> Self {
        let archetype = class_archetype(class);
        Self {
            class,
            archetype,
            anim_state: PlayerAnimState::Idle,
            prev_anim_state: PlayerAnimState::Idle,
            transition_t: 1.0,
            status_effect: StatusEffect::None,
            equipment_tier: EquipmentTier::Light,
            weapon_glyph_override: None,
            hp_frac: 1.0,
            level: 1,
            hit_reaction_t: 1.0,
            death_t: 0.0,
            level_up_t: 1.0,
            velocity: Vec3::ZERO,
            time: 0.0,
        }
    }

    /// Advance by dt seconds.
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;
        if self.transition_t < 1.0 {
            self.transition_t = (self.transition_t + dt * 3.0).min(1.0);
        }
        if self.hit_reaction_t < 1.0 {
            self.hit_reaction_t = (self.hit_reaction_t + dt * 4.0).min(1.0);
        }
        if self.anim_state == PlayerAnimState::Death && self.death_t < 1.0 {
            self.death_t = (self.death_t + dt * 0.5).min(1.0);
        }
        if self.level_up_t < 1.0 {
            self.level_up_t = (self.level_up_t + dt * 1.5).min(1.0);
        }
    }

    pub fn set_anim_state(&mut self, state: PlayerAnimState) {
        if state != self.anim_state {
            self.prev_anim_state = self.anim_state;
            self.anim_state = state;
            self.transition_t = 0.0;
        }
    }

    pub fn trigger_hit(&mut self) {
        self.hit_reaction_t = 0.0;
        self.set_anim_state(PlayerAnimState::Hurt);
    }

    pub fn trigger_death(&mut self) {
        self.death_t = 0.0;
        self.set_anim_state(PlayerAnimState::Death);
    }

    pub fn trigger_level_up(&mut self) {
        self.level_up_t = 0.0;
        self.level += 1;
    }
}

// ── Full-featured render entry point ─────────────────────────────────────────

/// Render the player with full visual state: status effects, poses, equipment,
/// level-up, movement animation, and death dissolution.
///
/// This is the advanced counterpart to `render_player()`. If you only need a
/// quick render (e.g. character select preview), use `render_player()` instead.
pub fn render_player_full(
    engine: &mut ProofEngine,
    state: &PlayerVisualState,
    position: Vec3,
    frame: u64,
) {
    let arch = state.archetype;
    let base_count = arch.base_glyph_count() + (state.level as usize / 3);
    let scale = arch.formation_scale();
    let time = state.time;

    // Choose formation based on animation state
    let current_shape = arch.formation_for_state(state.anim_state);
    let prev_shape = arch.formation_for_state(state.prev_anim_state);

    let cur_positions = current_shape.generate_positions(base_count, scale);
    let prev_positions = prev_shape.generate_positions(base_count, scale);

    // Interpolate formation transition
    let st = smooth_step(state.transition_t);
    let mut positions = formations::interpolate_formations(&prev_positions, &cur_positions, st);

    // Breathing (class-specific)
    positions = apply_class_idle(&positions, arch, time);

    // HP-based drift
    positions = formations::apply_hp_drift(&positions, state.hp_frac, time);

    // Hit reaction
    if state.hit_reaction_t < 1.0 {
        positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.5);
    }

    // Movement lean
    positions = formations::apply_movement_lean(&positions, state.velocity, 0.3);

    // Death dissolution
    if state.anim_state == PlayerAnimState::Death {
        let (dp, _) = formations::death_dissolution(&positions, state.death_t, 42);
        positions = dp;
    }

    // Level-up flash: tighten + new glyph
    if state.level_up_t < 1.0 {
        let new_pos = Vec3::new(0.0, scale * 1.5, 0.0);
        let (lp, _) = formations::level_up_formation(&positions, state.level_up_t, new_pos);
        positions = lp;
    }

    // Generate glyphs and colors
    let body = archetype_body_glyphs(arch);
    let weapon = archetype_weapon_glyphs(arch);
    let armor = archetype_armor_glyphs(arch, state.equipment_tier);
    let base_color = archetype_base_color(arch);
    let accent = archetype_accent_color(arch);
    let glow = archetype_glow_color(arch);
    let emission = archetype_emission(arch);

    let death_alpha = if state.anim_state == PlayerAnimState::Death {
        (1.0 - state.death_t).max(0.0)
    } else {
        1.0
    };

    let level_glow = if state.level_up_t < 1.0 {
        if state.level_up_t < 0.5 { state.level_up_t / 0.5 } else { 1.0 - (state.level_up_t - 0.5) / 0.5 }
    } else {
        0.0
    };

    for (i, p) in positions.iter().enumerate() {
        let world_pos = position + *p;

        // Pick character
        let ch = if i == 0 {
            state.weapon_glyph_override.unwrap_or(weapon[0])
        } else if i < 3 {
            weapon[i % weapon.len()]
        } else if p.length() > scale * 0.8 {
            armor[i % armor.len()]
        } else {
            body[i % body.len()]
        };

        // Pick color: inner = accent, outer = base
        let dist = p.length();
        let t = (dist / (scale * 1.2)).clamp(0.0, 1.0);
        let mut color = lerp_color(accent, base_color, t);

        // Status effect overlay
        color = apply_status_color(color, state.status_effect, time, i);

        // Level-up glow
        if level_glow > 0.0 {
            color.x = (color.x + level_glow * 0.4).min(1.0);
            color.y = (color.y + level_glow * 0.4).min(1.0);
            color.z = (color.z + level_glow * 0.2).min(1.0);
        }

        // Death fade
        color.w *= death_alpha;

        let em = emission + level_glow * 0.8;
        let gr = if level_glow > 0.0 { 0.8 } else { 0.4 };

        spawn_glow(engine, ch, world_pos, color, em, 0.85, glow, gr);
    }
}

// ── Status effect color modifier ─────────────────────────────────────────────

fn apply_status_color(base: Vec4, effect: StatusEffect, time: f32, idx: usize) -> Vec4 {
    match effect {
        StatusEffect::None => base,
        StatusEffect::Burning => formations::color_burning(base, time, idx),
        StatusEffect::Frozen => formations::color_frozen(base, time, idx),
        StatusEffect::Poisoned => formations::color_poisoned(base, time, idx),
        StatusEffect::Blessed => formations::color_blessed(base, time, idx),
    }
}

// ── Class-specific idle (formation version) ──────────────────────────────────

fn apply_class_idle(positions: &[Vec3], arch: ClassArchetype, time: f32) -> Vec<Vec3> {
    let rate = arch.pulse_rate();
    let depth = arch.pulse_depth();

    match arch {
        ClassArchetype::Warrior => {
            // Uniform shield breathing
            formations::apply_breathing(positions, time, rate, depth)
        }
        ClassArchetype::Mage => {
            // Outer ring slowly rotates
            let base = formations::apply_breathing(positions, time, rate, depth);
            base.iter()
                .map(|p| {
                    let dist = p.length();
                    if dist > 0.8 {
                        let angle = time * 0.2 * (dist - 0.8);
                        let (s, c) = angle.sin_cos();
                        Vec3::new(p.x * c - p.y * s, p.x * s + p.y * c, p.z)
                    } else {
                        *p
                    }
                })
                .collect()
        }
        ClassArchetype::Rogue => {
            // Subtle asymmetric
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let phase = time * rate * TAU + i as f32 * 0.5;
                    Vec3::new(
                        p.x * (1.0 + phase.sin() * depth * 0.7),
                        p.y * (1.0 + (phase + PI * 0.5).sin() * depth),
                        p.z,
                    )
                })
                .collect()
        }
        ClassArchetype::Cleric => {
            // Radiant pulse from center
            positions
                .iter()
                .map(|p| {
                    let wave = (time * rate * TAU - p.length() * 2.0).sin() * depth;
                    *p * (1.0 + wave)
                })
                .collect()
        }
        ClassArchetype::Necromancer => {
            // Dark irregular heartbeat
            let beat = (time * rate * TAU).sin();
            let d = if beat > 0.7 { depth * 1.5 } else if beat > 0.3 { depth * 0.3 } else { -depth * 0.5 };
            positions.iter().map(|p| *p * (1.0 + d)).collect()
        }
        ClassArchetype::Berserker => {
            // Aggressive rapid jitter
            let rage = (time * rate * TAU).sin().abs() * depth * 1.5;
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let j = ((time * 12.0 + i as f32 * 3.7).sin() * rage * 0.3).abs();
                    *p * (1.0 + rage + j)
                })
                .collect()
        }
    }
}

// ── AmorphousEntity builder (formation-backed) ──────────────────────────────

/// Build an AmorphousEntity for the player (formation-backed approach).
pub fn build_player_entity(class: CharacterClass, position: Vec3) -> AmorphousEntity {
    let arch = class_archetype(class);
    let count = arch.base_glyph_count();
    let scale = arch.formation_scale();
    let shape = arch.idle_formation();

    let positions = shape.generate_positions(count, scale);
    let body = archetype_body_glyphs(arch);
    let weapon = archetype_weapon_glyphs(arch);
    let base_color = archetype_base_color(arch);
    let accent = archetype_accent_color(arch);

    let mut chars = Vec::with_capacity(count);
    let mut colors = Vec::with_capacity(count);

    for (i, p) in positions.iter().enumerate() {
        if i < 3 {
            chars.push(weapon[i % weapon.len()]);
            colors.push(accent);
        } else {
            chars.push(body[i % body.len()]);
            let dist = p.length();
            let t = (dist / (scale * 1.2)).clamp(0.0, 1.0);
            colors.push(lerp_color(accent, base_color, t));
        }
    }

    let mut entity = AmorphousEntity::new(format!("player_{:?}", class), position);
    entity.entity_mass = 50.0;
    entity.pulse_rate = arch.pulse_rate();
    entity.pulse_depth = arch.pulse_depth();
    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity
}

/// Update an existing AmorphousEntity from a PlayerVisualState.
pub fn update_player_entity(entity: &mut AmorphousEntity, state: &PlayerVisualState) {
    let arch = state.archetype;
    let count = arch.base_glyph_count() + (state.level as usize / 3);
    let scale = arch.formation_scale();

    let current_shape = arch.formation_for_state(state.anim_state);
    let prev_shape = arch.formation_for_state(state.prev_anim_state);

    let cur = current_shape.generate_positions(count, scale);
    let prev = prev_shape.generate_positions(count, scale);

    let mut positions = formations::interpolate_formations(
        &prev, &cur, smooth_step(state.transition_t),
    );

    positions = apply_class_idle(&positions, arch, state.time);
    positions = formations::apply_hp_drift(&positions, state.hp_frac, state.time);

    if state.hit_reaction_t < 1.0 {
        positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.5);
    }
    positions = formations::apply_movement_lean(&positions, state.velocity, 0.3);

    if state.anim_state == PlayerAnimState::Death {
        let (dp, _) = formations::death_dissolution(&positions, state.death_t, 42);
        positions = dp;
    }
    if state.level_up_t < 1.0 {
        let new_pos = Vec3::new(0.0, scale * 1.5, 0.0);
        let (lp, _) = formations::level_up_formation(&positions, state.level_up_t, new_pos);
        positions = lp;
    }

    let body = archetype_body_glyphs(arch);
    let weapon = archetype_weapon_glyphs(arch);
    let armor = archetype_armor_glyphs(arch, state.equipment_tier);
    let base_color = archetype_base_color(arch);
    let accent = archetype_accent_color(arch);

    let death_alpha = if state.anim_state == PlayerAnimState::Death {
        (1.0 - state.death_t).max(0.0)
    } else {
        1.0
    };
    let level_glow = if state.level_up_t < 1.0 {
        if state.level_up_t < 0.5 { state.level_up_t / 0.5 } else { 1.0 - (state.level_up_t - 0.5) / 0.5 }
    } else {
        0.0
    };

    let len = positions.len();
    let mut chars = Vec::with_capacity(len);
    let mut colors = Vec::with_capacity(len);

    for (i, p) in positions.iter().enumerate() {
        let ch = if i == 0 {
            state.weapon_glyph_override.unwrap_or(weapon[0])
        } else if i < 3 {
            weapon[i % weapon.len()]
        } else if p.length() > scale * 0.8 {
            armor[i % armor.len()]
        } else {
            body[i % body.len()]
        };
        chars.push(ch);

        let dist = p.length();
        let t = (dist / (scale * 1.2)).clamp(0.0, 1.0);
        let mut color = lerp_color(accent, base_color, t);
        color = apply_status_color(color, state.status_effect, state.time, i);
        if level_glow > 0.0 {
            color.x = (color.x + level_glow * 0.4).min(1.0);
            color.y = (color.y + level_glow * 0.4).min(1.0);
            color.z = (color.z + level_glow * 0.2).min(1.0);
        }
        color.w *= death_alpha;
        colors.push(color);
    }

    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity.hp = state.hp_frac * entity.max_hp;
    entity.update_cohesion();
}

// ── Weapon glyph mapping ─────────────────────────────────────────────────────

/// Map a weapon name to a representative glyph.
pub fn weapon_name_to_glyph(name: &str) -> char {
    let lower = name.to_lowercase();
    if lower.contains("sword") || lower.contains("blade") { '/' }
    else if lower.contains("axe") { 'X' }
    else if lower.contains("mace") || lower.contains("hammer") { '#' }
    else if lower.contains("staff") || lower.contains("wand") { '|' }
    else if lower.contains("dagger") || lower.contains("knife") { '\\' }
    else if lower.contains("bow") { ')' }
    else if lower.contains("spear") || lower.contains("lance") { '!' }
    else if lower.contains("shield") { '=' }
    else { '+' }
}

/// Map armor value to tier.
pub fn armor_value_to_tier(armor: u32) -> EquipmentTier {
    match armor {
        0 => EquipmentTier::Bare,
        1..=10 => EquipmentTier::Light,
        11..=25 => EquipmentTier::Medium,
        _ => EquipmentTier::Heavy,
    }
}

// ── Utility ──────────────────────────────────────────────────────────────────

fn smooth_step(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Lerp two colors.
pub fn lerp_color(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Return glyph count for a player at the given level and class.
pub fn player_glyph_count(class: CharacterClass, level: u32) -> usize {
    class_archetype(class).base_glyph_count() + (level as usize / 3)
}

/// Return base + accent colors for a class.
pub fn get_class_colors(class: CharacterClass) -> (Vec4, Vec4) {
    let arch = class_archetype(class);
    (archetype_base_color(arch), archetype_accent_color(arch))
}

/// Return emission intensity for a class.
pub fn get_class_emission(class: CharacterClass) -> f32 {
    archetype_emission(class_archetype(class))
}
