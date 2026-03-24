//! Comprehensive VFX system bridging proof-engine particles, force fields, and
//! effects to the chaos-rpg game.
//!
//! Five major subsystems:
//!   1. Combat VFX Manager — attack trails, projectiles, explosions, dissolves
//!   2. Status Effect VFX  — burn, freeze, poison, bleed, stun, regen
//!   3. Environment VFX    — room transitions, loot sparkles, shrines, rifts
//!   4. Boss VFX           — mirror, null void, glitch, hydra, committee, ouroboros
//!   5. Force Field Integration — gravity wells, shockwaves, vortices, wind
//!
//! Camera at (0,0,-10), +X = right, +Y = up.
//! Visible area at z=0: roughly ±8.7 horizontal, ±5.4 vertical.
//! All systems operate in immediate mode — glyphs are cleared each frame.

use proof_engine::prelude::*;
use crate::state::GameState;

// ═══════════════════════════════════════════════════════════════════════════════
//  Constants
// ═══════════════════════════════════════════════════════════════════════════════

const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::TAU;
const PHI: f32 = 1.618_033_9;
const GOLDEN_ANGLE: f32 = 2.399_963_2; // radians

/// Visible half-widths at z=0.
const VIEW_HX: f32 = 8.7;
const VIEW_HY: f32 = 5.4;

// ═══════════════════════════════════════════════════════════════════════════════
//  Utility helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Deterministic pseudo-random from an integer seed — returns [0, 1).
fn hash_f32(seed: u64) -> f32 {
    let h = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((h >> 33) as f32) / (u32::MAX as f32)
}

/// Spawn a line of text as individual glyphs.
fn spawn_text(
    engine: &mut ProofEngine,
    text: &str,
    origin: Vec3,
    color: Vec4,
    emission: f32,
    layer: RenderLayer,
) {
    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(origin.x + i as f32 * 0.6, origin.y, origin.z),
            color,
            emission,
            layer,
            ..Default::default()
        });
    }
}

/// Linear interpolation.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smoothstep for nice fade curves.
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  1. COMBAT VFX MANAGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Element colors for spell projectiles.
fn element_color(element: &str) -> (Vec4, Vec3) {
    match element {
        "fire" => (
            Vec4::new(1.0, 0.4, 0.1, 1.0),
            Vec3::new(1.0, 0.5, 0.1),
        ),
        "ice" | "frost" => (
            Vec4::new(0.3, 0.7, 1.0, 1.0),
            Vec3::new(0.4, 0.7, 1.0),
        ),
        "lightning" => (
            Vec4::new(1.0, 1.0, 0.3, 1.0),
            Vec3::new(1.0, 1.0, 0.5),
        ),
        "poison" | "acid" => (
            Vec4::new(0.2, 0.9, 0.2, 1.0),
            Vec3::new(0.3, 0.8, 0.2),
        ),
        "shadow" | "dark" => (
            Vec4::new(0.5, 0.1, 0.7, 1.0),
            Vec3::new(0.4, 0.1, 0.6),
        ),
        "holy" | "light" => (
            Vec4::new(1.0, 0.95, 0.7, 1.0),
            Vec3::new(1.0, 0.95, 0.8),
        ),
        _ => (
            Vec4::new(0.8, 0.8, 0.8, 1.0),
            Vec3::new(0.7, 0.7, 0.7),
        ),
    }
}

/// Trail glyph characters per element.
fn element_char(element: &str, index: usize) -> char {
    match element {
        "fire" => ['*', '◆', '▲', '●'][index % 4],
        "ice" | "frost" => ['❄', '◇', '△', '○'][index % 4],
        "lightning" => ['⚡', '╋', '┃', '━'][index % 4],
        "poison" | "acid" => ['◉', '○', '●', '·'][index % 4],
        "shadow" | "dark" => ['▓', '▒', '░', '·'][index % 4],
        "holy" | "light" => ['✦', '✧', '◇', '·'][index % 4],
        _ => ['*', '·', '+', '×'][index % 4],
    }
}

/// Render an attack trail: 8-12 glyphs along a line from attacker to defender.
///
/// `progress` is 0.0..1.0 — the trail advances along the path over time.
pub fn render_attack_trail(
    engine: &mut ProofEngine,
    from: Vec3,
    to: Vec3,
    progress: f32,
    frame: u64,
) {
    let count = 10; // 8-12 range, we pick the middle
    let dir = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);

    for i in 0..count {
        let t = (i as f32 / count as f32) * progress;
        if t > progress {
            break;
        }
        // Fade older segments
        let age = progress - t;
        let alpha = (1.0 - age * 2.0).clamp(0.0, 1.0);
        if alpha <= 0.0 {
            continue;
        }

        let wobble = (frame as f32 * 0.3 + i as f32 * 1.7).sin() * 0.15;
        let pos = Vec3::new(
            from.x + dir.x * t + wobble,
            from.y + dir.y * t + wobble * 0.5,
            0.0,
        );

        let trail_chars = ['━', '─', '╌', '·', '⟩', '▸'];
        engine.spawn_glyph(Glyph {
            character: trail_chars[i % trail_chars.len()],
            position: pos,
            velocity: Vec3::new(dir.x * 0.3, dir.y * 0.3, 0.0),
            color: Vec4::new(1.0, 0.9, 0.7, alpha),
            emission: alpha * 0.6,
            glow_color: Vec3::new(1.0, 0.8, 0.4),
            glow_radius: alpha * 0.3,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render a spell projectile following a parametric curve.
///
/// `t` is 0.0..1.0 progression along the path.
pub fn render_spell_projectile(
    engine: &mut ProofEngine,
    from: Vec3,
    to: Vec3,
    t: f32,
    element: &str,
    frame: u64,
) {
    let (color, glow) = element_color(element);
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    // Parametric arc: quadratic bezier with a midpoint above the line
    let mid_y = (from.y + to.y) * 0.5 + 2.5;
    let one_minus_t = 1.0 - t;

    let px = one_minus_t * one_minus_t * from.x
        + 2.0 * one_minus_t * t * (from.x + dx * 0.5)
        + t * t * to.x;
    let py = one_minus_t * one_minus_t * from.y
        + 2.0 * one_minus_t * t * mid_y
        + t * t * to.y;

    // Leading projectile glyph
    engine.spawn_glyph(Glyph {
        character: element_char(element, 0),
        position: Vec3::new(px, py, 0.0),
        scale: Vec2::new(1.5, 1.5),
        color,
        emission: 1.0,
        glow_color: glow,
        glow_radius: 1.2,
        layer: RenderLayer::Particle,
        ..Default::default()
    });

    // Trailing particles (6-8 behind the head)
    for i in 1..8 {
        let trail_t = (t - i as f32 * 0.04).max(0.0);
        let trail_one = 1.0 - trail_t;
        let tx = trail_one * trail_one * from.x
            + 2.0 * trail_one * trail_t * (from.x + dx * 0.5)
            + trail_t * trail_t * to.x;
        let ty = trail_one * trail_one * from.y
            + 2.0 * trail_one * trail_t * mid_y
            + trail_t * trail_t * to.y;

        let fade = 1.0 - (i as f32 / 8.0);
        let jitter_x = (frame as f32 * 0.5 + i as f32 * 2.3).sin() * 0.1;
        let jitter_y = (frame as f32 * 0.7 + i as f32 * 3.1).cos() * 0.1;

        engine.spawn_glyph(Glyph {
            character: element_char(element, i),
            position: Vec3::new(tx + jitter_x, ty + jitter_y, 0.0),
            color: Vec4::new(color.x, color.y, color.z, fade * 0.8),
            emission: fade * 0.6,
            glow_color: glow,
            glow_radius: fade * 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render a critical hit explosion: 20+ particles bursting radially.
///
/// `t` is 0.0..1.0 — explosion lifetime.
pub fn render_crit_explosion(
    engine: &mut ProofEngine,
    center: Vec3,
    t: f32,
    frame: u64,
) {
    let count = 24;
    let explosion_chars = ['✦', '✧', '◆', '◇', '*', '×', '⊕', '+'];
    let expand = t * 5.0; // radius expands over time
    let fade = (1.0 - t).max(0.0);

    for i in 0..count {
        let angle = (i as f32 / count as f32) * TAU;
        let speed_variation = 0.7 + hash_f32(frame + i as u64) * 0.6;
        let r = expand * speed_variation;

        let x = center.x + angle.cos() * r;
        let y = center.y + angle.sin() * r;

        let flicker = ((frame as f32 * 0.4 + i as f32 * 1.5).sin() * 0.3 + 0.7).max(0.0);
        let alpha = fade * flicker;

        // Colors shift from white-hot center to orange edges
        let heat = (1.0 - t * 0.7).max(0.0);
        engine.spawn_glyph(Glyph {
            character: explosion_chars[i % explosion_chars.len()],
            position: Vec3::new(x, y, 0.0),
            velocity: Vec3::new(angle.cos() * 2.0, angle.sin() * 2.0, 0.0),
            color: Vec4::new(1.0, heat * 0.7 + 0.3, heat * 0.3, alpha),
            emission: alpha * 0.9,
            glow_color: Vec3::new(1.0, 0.6, 0.2),
            glow_radius: alpha * 0.5,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Central flash (big, bright, fades fast)
    if t < 0.3 {
        let flash_alpha = (1.0 - t / 0.3).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '█',
            position: center,
            scale: Vec2::new(3.0 - t * 6.0, 3.0 - t * 6.0),
            color: Vec4::new(1.0, 1.0, 0.9, flash_alpha * 0.8),
            emission: flash_alpha,
            glow_color: Vec3::new(1.0, 0.95, 0.8),
            glow_radius: flash_alpha * 2.0,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
}

/// Render death dissolve: entity glyphs scatter outward with gravity.
///
/// `glyphs` are the character representations of the dying entity.
/// `t` is 0.0..1.0 — dissolve lifetime.
pub fn render_death_dissolve(
    engine: &mut ProofEngine,
    center: Vec3,
    glyphs: &[char],
    t: f32,
    frame: u64,
) {
    let gravity = -3.0;
    let fade = (1.0 - t).max(0.0);

    for (i, &ch) in glyphs.iter().enumerate() {
        let seed = (i as u64).wrapping_mul(7919) + 13;
        let angle = hash_f32(seed) * TAU;
        let speed = 1.0 + hash_f32(seed + 1) * 3.0;
        let spin = hash_f32(seed + 2) * 2.0 - 1.0;

        // Physics: outward velocity + gravity
        let elapsed = t * 2.0; // scale time
        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;
        let x = center.x + vx * elapsed;
        let y = center.y + vy * elapsed + 0.5 * gravity * elapsed * elapsed;

        // Flicker near end of life
        let flicker = if t > 0.7 {
            ((frame as f32 * 0.8 + i as f32).sin() * 0.5 + 0.5).max(0.0)
        } else {
            1.0
        };

        let alpha = fade * flicker;
        if alpha < 0.01 {
            continue;
        }

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 0.0),
            velocity: Vec3::new(vx * 0.5, vy * 0.5 + gravity * elapsed, 0.0),
            color: Vec4::new(0.6, 0.15, 0.1, alpha),
            emission: alpha * 0.3,
            layer: RenderLayer::Entity,
            ..Default::default()
        });
    }
}

/// Render a floating damage number that rises and fades.
///
/// `t` is 0.0..1.0 — the number's lifetime.
pub fn render_damage_number(
    engine: &mut ProofEngine,
    amount: i64,
    position: Vec3,
    t: f32,
    is_crit: bool,
) {
    let rise = t * 2.5; // float upward
    let alpha = if t < 0.1 {
        t / 0.1 // fade in
    } else {
        (1.0 - (t - 0.1) / 0.9).max(0.0) // fade out
    };

    let text = if is_crit {
        format!("!!{}!!", amount)
    } else {
        format!("{}", amount)
    };

    let (color, emission, scale) = if is_crit {
        (
            Vec4::new(1.0, 0.9, 0.1, alpha),
            0.9,
            Vec2::new(1.4, 1.4),
        )
    } else {
        (
            Vec4::new(1.0, 0.3, 0.2, alpha),
            0.4,
            Vec2::new(1.0, 1.0),
        )
    };

    let wobble = (t * PI * 3.0).sin() * 0.3 * (1.0 - t);
    let y = position.y + rise;
    let x = position.x + wobble - (text.len() as f32 * 0.3);

    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.6, y, 0.0),
            scale,
            color,
            emission,
            glow_color: Vec3::new(color.x, color.y, color.z),
            glow_radius: if is_crit { 0.6 } else { 0.2 },
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

/// Render a shield flash: brief bright rectangle at the defender position.
///
/// `t` is 0.0..1.0 — flash lifetime (typically short, 0.2s).
pub fn render_shield_flash(
    engine: &mut ProofEngine,
    position: Vec3,
    t: f32,
) {
    let alpha = (1.0 - t * 3.0).clamp(0.0, 1.0); // fast fade
    if alpha <= 0.0 {
        return;
    }

    let shield_chars = ['╔', '═', '╗', '║', ' ', '║', '╚', '═', '╝'];
    let offsets: [(f32, f32); 9] = [
        (-1.0, 1.0),  (0.0, 1.0),  (1.0, 1.0),
        (-1.0, 0.0),  (0.0, 0.0),  (1.0, 0.0),
        (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0),
    ];

    for (i, &ch) in shield_chars.iter().enumerate() {
        if ch == ' ' {
            continue;
        }
        let (ox, oy) = offsets[i];
        let scale_expand = 1.0 + t * 0.5; // slightly grows as it fades
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(
                position.x + ox * scale_expand,
                position.y + oy * scale_expand,
                0.0,
            ),
            color: Vec4::new(0.4, 0.7, 1.0, alpha * 0.9),
            emission: alpha * 0.8,
            glow_color: Vec3::new(0.5, 0.8, 1.0),
            glow_radius: alpha * 1.0,
            layer: RenderLayer::Entity,
            ..Default::default()
        });
    }
}

/// Render healing spiral: golden particles spiraling upward.
///
/// `t` is 0.0..1.0 — spiral lifetime.
pub fn render_healing_spiral(
    engine: &mut ProofEngine,
    center: Vec3,
    t: f32,
    frame: u64,
) {
    let count = 16;
    let fade = (1.0 - t).max(0.0);

    for i in 0..count {
        let phase = i as f32 / count as f32;
        let particle_t = (t * 3.0 + phase) % 1.0;
        let rise = particle_t * 4.0;
        let angle = particle_t * TAU * 2.0 + phase * TAU;
        let radius = 0.8 + (1.0 - particle_t) * 0.5;

        let x = center.x + angle.cos() * radius;
        let y = center.y + rise - 1.0;
        let alpha = fade * (1.0 - particle_t) * 0.9;

        if alpha < 0.01 || y > center.y + 4.0 {
            continue;
        }

        let sparkle = ((frame as f32 * 0.6 + i as f32 * 2.0).sin() * 0.2 + 0.8).max(0.0);
        let heal_chars = ['+', '✦', '◇', '·'];

        engine.spawn_glyph(Glyph {
            character: heal_chars[i % heal_chars.len()],
            position: Vec3::new(x, y, 0.0),
            velocity: Vec3::new(-angle.sin() * 0.5, 1.5, 0.0),
            color: Vec4::new(0.2, 1.0, 0.4, alpha * sparkle),
            emission: alpha * 0.7,
            glow_color: Vec3::new(0.3, 1.0, 0.5),
            glow_radius: alpha * 0.4,
            life_function: Some(MathFunction::Breathing {
                rate: 2.0 + i as f32 * 0.3,
                depth: 0.2,
            }),
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  2. STATUS EFFECT VFX
// ═══════════════════════════════════════════════════════════════════════════════

/// Render burning embers: orange particles drifting upward, flickering.
pub fn render_burn_embers(
    engine: &mut ProofEngine,
    entity_pos: Vec3,
    frame: u64,
) {
    let count = 10;
    for i in 0..count {
        let seed = (frame / 4 + i as u64).wrapping_mul(2654435761);
        let rx = hash_f32(seed) * 2.0 - 1.0;
        let phase = hash_f32(seed + 1);
        let rise = (frame as f32 * 0.02 + phase * 6.0) % 3.0;

        let flicker = ((frame as f32 * 0.5 + i as f32 * 3.7).sin() * 0.4 + 0.6).max(0.0);
        let alpha = flicker * (1.0 - rise / 3.0).max(0.0);

        let ember_chars = ['▪', '·', '◦', '◘'];
        engine.spawn_glyph(Glyph {
            character: ember_chars[i as usize % ember_chars.len()],
            position: Vec3::new(entity_pos.x + rx, entity_pos.y + rise, 0.0),
            velocity: Vec3::new(rx * 0.3, 1.0, 0.0),
            color: Vec4::new(1.0, 0.5 * flicker, 0.1, alpha),
            emission: alpha * 0.8,
            glow_color: Vec3::new(1.0, 0.4, 0.05),
            glow_radius: alpha * 0.3,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render freeze shards: blue crystalline particles drifting slowly down.
pub fn render_freeze_shards(
    engine: &mut ProofEngine,
    entity_pos: Vec3,
    frame: u64,
) {
    let count = 8;
    for i in 0..count {
        let seed = (frame / 6 + i as u64).wrapping_mul(1103515245);
        let rx = hash_f32(seed) * 3.0 - 1.5;
        let phase = hash_f32(seed + 1);
        let fall = (frame as f32 * 0.015 + phase * 5.0) % 4.0;

        let shimmer = ((frame as f32 * 0.3 + i as f32 * 2.1).sin() * 0.3 + 0.7).max(0.0);
        let alpha = shimmer * (1.0 - fall / 4.0).max(0.0);

        let shard_chars = ['◇', '◆', '❄', '△', '▽', '✧'];
        engine.spawn_glyph(Glyph {
            character: shard_chars[i as usize % shard_chars.len()],
            position: Vec3::new(
                entity_pos.x + rx,
                entity_pos.y + 2.0 - fall,
                0.0,
            ),
            velocity: Vec3::new(0.0, -0.3, 0.0),
            color: Vec4::new(0.4 * shimmer, 0.7, 1.0, alpha),
            emission: alpha * 0.5,
            glow_color: Vec3::new(0.5, 0.8, 1.0),
            glow_radius: alpha * 0.25,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render poison bubbles: green circles rising from entity.
pub fn render_poison_bubbles(
    engine: &mut ProofEngine,
    entity_pos: Vec3,
    frame: u64,
) {
    let count = 6;
    for i in 0..count {
        let seed = (frame / 8 + i as u64).wrapping_mul(48271);
        let rx = hash_f32(seed) * 2.0 - 1.0;
        let phase = hash_f32(seed + 1);
        let rise = (frame as f32 * 0.018 + phase * 4.0) % 3.5;
        let wobble = (rise * PI + i as f32).sin() * 0.2;

        let size_pulse = ((rise * PI * 0.5).sin() * 0.3 + 0.7).max(0.0);
        let alpha = (1.0 - rise / 3.5).max(0.0) * size_pulse;

        // Bubbles pop at the top
        let ch = if rise > 3.0 { '·' } else if rise > 2.0 { '○' } else { '◎' };

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(entity_pos.x + rx + wobble, entity_pos.y + rise - 0.5, 0.0),
            velocity: Vec3::new(wobble * 0.5, 0.8, 0.0),
            color: Vec4::new(0.2, 0.85, 0.15, alpha * 0.8),
            emission: alpha * 0.4,
            glow_color: Vec3::new(0.3, 0.9, 0.2),
            glow_radius: alpha * 0.2,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render bleed drops: red dots falling from entity to floor with gravity.
pub fn render_bleed_drops(
    engine: &mut ProofEngine,
    entity_pos: Vec3,
    frame: u64,
) {
    let count = 7;
    let gravity = -4.0;
    let floor_y = entity_pos.y - 2.5;

    for i in 0..count {
        let seed = (frame / 5 + i as u64).wrapping_mul(16807);
        let rx = hash_f32(seed) * 1.6 - 0.8;
        let phase = hash_f32(seed + 1);
        let t = (frame as f32 * 0.025 + phase * 3.0) % 2.0;

        // Parabolic fall
        let vy_init = 0.5;
        let y = entity_pos.y + vy_init * t + 0.5 * gravity * t * t;
        let clamped_y = y.max(floor_y);

        let alpha = if clamped_y <= floor_y {
            // Splat on floor — fade out
            let time_on_floor = t - ((vy_init + (vy_init * vy_init - 2.0 * gravity * (entity_pos.y - floor_y)).max(0.0).sqrt()) / (-gravity));
            (1.0 - time_on_floor * 2.0).clamp(0.0, 0.8)
        } else {
            0.9
        };

        let ch = if clamped_y <= floor_y { '·' } else { '●' };

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(entity_pos.x + rx, clamped_y, 0.0),
            color: Vec4::new(0.8, 0.05, 0.05, alpha),
            emission: 0.2,
            glow_color: Vec3::new(0.6, 0.0, 0.0),
            glow_radius: 0.1,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render stun stars: yellow stars orbiting around entity head.
pub fn render_stun_stars(
    engine: &mut ProofEngine,
    entity_pos: Vec3,
    frame: u64,
) {
    let count = 5;
    let orbit_radius = 1.2;
    let orbit_y = entity_pos.y + 2.0; // above head
    let speed = frame as f32 * 0.06;

    for i in 0..count {
        let angle = speed + (i as f32 / count as f32) * TAU;
        let x = entity_pos.x + angle.cos() * orbit_radius;
        let y = orbit_y + angle.sin() * 0.3; // slight vertical bob

        let twinkle = ((frame as f32 * 0.4 + i as f32 * 1.9).sin() * 0.3 + 0.7).max(0.0);
        let star_chars = ['★', '☆', '✦', '✧', '⊛'];

        engine.spawn_glyph(Glyph {
            character: star_chars[i % star_chars.len()],
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(1.0, 0.95, 0.2, twinkle),
            emission: twinkle * 0.7,
            glow_color: Vec3::new(1.0, 0.9, 0.3),
            glow_radius: twinkle * 0.3,
            life_function: Some(MathFunction::Orbit {
                radius: orbit_radius,
                speed: 1.5,
            }),
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render regen glow: soft green particles pulsing outward.
pub fn render_regen_glow(
    engine: &mut ProofEngine,
    entity_pos: Vec3,
    frame: u64,
) {
    let pulse_phase = (frame as f32 * 0.04).sin() * 0.5 + 0.5; // slow heartbeat
    let count = 8;

    for i in 0..count {
        let angle = (i as f32 / count as f32) * TAU;
        let expand = 0.5 + pulse_phase * 1.5;
        let x = entity_pos.x + angle.cos() * expand;
        let y = entity_pos.y + angle.sin() * expand;
        let alpha = (1.0 - expand / 2.5).clamp(0.0, 0.7) * pulse_phase;

        engine.spawn_glyph(Glyph {
            character: '+',
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(0.2, 0.9, 0.3, alpha),
            emission: alpha * 0.5,
            glow_color: Vec3::new(0.3, 1.0, 0.4),
            glow_radius: alpha * 0.6,
            life_function: Some(MathFunction::Breathing {
                rate: 1.5,
                depth: 0.3,
            }),
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Central cross that breathes
    let cross_alpha = pulse_phase * 0.5;
    engine.spawn_glyph(Glyph {
        character: '✚',
        position: entity_pos,
        scale: Vec2::new(1.0 + pulse_phase * 0.3, 1.0 + pulse_phase * 0.3),
        color: Vec4::new(0.3, 1.0, 0.5, cross_alpha),
        emission: cross_alpha * 0.6,
        glow_color: Vec3::new(0.3, 1.0, 0.4),
        glow_radius: cross_alpha * 0.8,
        layer: RenderLayer::Particle,
        ..Default::default()
    });
}

/// Dispatch the correct status effect VFX by name.
pub fn render_status_effect(
    engine: &mut ProofEngine,
    effect_name: &str,
    entity_pos: Vec3,
    frame: u64,
) {
    match effect_name {
        "burn" | "fire" => render_burn_embers(engine, entity_pos, frame),
        "freeze" | "frozen" | "ice" => render_freeze_shards(engine, entity_pos, frame),
        "poison" | "toxic" => render_poison_bubbles(engine, entity_pos, frame),
        "bleed" | "bleeding" => render_bleed_drops(engine, entity_pos, frame),
        "stun" | "stunned" => render_stun_stars(engine, entity_pos, frame),
        "regen" | "regeneration" => render_regen_glow(engine, entity_pos, frame),
        _ => {} // unknown status — no VFX
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  3. ENVIRONMENT VFX
// ═══════════════════════════════════════════════════════════════════════════════

/// Render room entry flash: bright white flash that fades on room entry.
///
/// `t` is 0.0..1.0 — flash lifetime.
pub fn render_room_entry_flash(
    engine: &mut ProofEngine,
    t: f32,
) {
    let alpha = (1.0 - t * 2.5).clamp(0.0, 0.8);
    if alpha <= 0.0 {
        return;
    }

    // Full-screen overlay of bright glyphs
    let step = 1.8;
    let mut y = -VIEW_HY;
    while y <= VIEW_HY {
        let mut x = -VIEW_HX;
        while x <= VIEW_HX {
            engine.spawn_glyph(Glyph {
                character: '░',
                position: Vec3::new(x, y, 0.5),
                color: Vec4::new(1.0, 1.0, 0.95, alpha * 0.6),
                emission: alpha,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
            x += step;
        }
        y += step;
    }
}

/// Render floor transition wipe: horizontal sweep of particles.
///
/// `t` is 0.0..1.0 — wipe progression left to right.
pub fn render_floor_transition_wipe(
    engine: &mut ProofEngine,
    t: f32,
    frame: u64,
) {
    let wipe_x = -VIEW_HX + t * VIEW_HX * 2.0; // sweeps left to right
    let band_width = 2.0;

    for i in 0..30 {
        let seed = (frame + i as u64).wrapping_mul(6364136223846793005);
        let ry = -VIEW_HY + hash_f32(seed) * VIEW_HY * 2.0;
        let rx_offset = hash_f32(seed + 1) * band_width - band_width * 0.5;
        let x = wipe_x + rx_offset;

        if x < -VIEW_HX || x > VIEW_HX {
            continue;
        }

        let brightness = (1.0 - (rx_offset / band_width).abs()).max(0.0);
        let wipe_chars = ['▓', '▒', '░', '·'];
        let ch_idx = ((rx_offset + band_width * 0.5) / band_width * 3.0) as usize;

        engine.spawn_glyph(Glyph {
            character: wipe_chars[ch_idx.min(3)],
            position: Vec3::new(x, ry, 0.2),
            velocity: Vec3::new(4.0, 0.0, 0.0),
            color: Vec4::new(0.8, 0.85, 1.0, brightness * 0.7),
            emission: brightness * 0.5,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
}

/// Render loot sparkle: orbiting golden particles around a dropped item.
pub fn render_loot_sparkle(
    engine: &mut ProofEngine,
    item_pos: Vec3,
    frame: u64,
) {
    let count = 6;
    let orbit_speed = frame as f32 * 0.05;

    for i in 0..count {
        let angle = orbit_speed + (i as f32 / count as f32) * TAU;
        let r = 0.6 + (frame as f32 * 0.08 + i as f32).sin() * 0.15;
        let x = item_pos.x + angle.cos() * r;
        let y = item_pos.y + angle.sin() * r * 0.5; // elliptical

        let sparkle = ((frame as f32 * 0.5 + i as f32 * 2.5).sin() * 0.4 + 0.6).max(0.0);

        engine.spawn_glyph(Glyph {
            character: if i % 2 == 0 { '✦' } else { '·' },
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(1.0, 0.88, 0.2, sparkle),
            emission: sparkle * 0.8,
            glow_color: Vec3::new(1.0, 0.85, 0.3),
            glow_radius: sparkle * 0.3,
            life_function: Some(MathFunction::Sine {
                freq: 2.0,
                amp: 0.1,
                phase: i as f32,
            }),
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render shrine glow: peaceful blue-white particle aura.
pub fn render_shrine_glow(
    engine: &mut ProofEngine,
    shrine_pos: Vec3,
    frame: u64,
) {
    let count = 12;
    let breath = (frame as f32 * 0.025).sin() * 0.3 + 0.7;

    for i in 0..count {
        let angle = (i as f32 / count as f32) * TAU;
        let r = 1.0 + breath * 0.5;
        let bob = (frame as f32 * 0.03 + i as f32 * 0.8).sin() * 0.3;

        let x = shrine_pos.x + angle.cos() * r;
        let y = shrine_pos.y + angle.sin() * r * 0.6 + bob;
        let alpha = breath * 0.5;

        engine.spawn_glyph(Glyph {
            character: if i % 3 == 0 { '◇' } else { '·' },
            position: Vec3::new(x, y, -0.5),
            color: Vec4::new(0.6, 0.8, 1.0, alpha),
            emission: alpha * 0.6,
            glow_color: Vec3::new(0.7, 0.85, 1.0),
            glow_radius: alpha * 0.5,
            life_function: Some(MathFunction::Breathing {
                rate: 0.8 + i as f32 * 0.1,
                depth: 0.15,
            }),
            layer: RenderLayer::World,
            ..Default::default()
        });
    }

    // Central shrine symbol
    engine.spawn_glyph(Glyph {
        character: '✦',
        position: Vec3::new(shrine_pos.x, shrine_pos.y, -0.5),
        scale: Vec2::new(1.5, 1.5),
        color: Vec4::new(0.8, 0.9, 1.0, breath * 0.7),
        emission: breath * 0.8,
        glow_color: Vec3::new(0.8, 0.9, 1.0),
        glow_radius: breath * 1.0,
        layer: RenderLayer::World,
        ..Default::default()
    });
}

/// Render rift crackle: chaotic multi-color particle bursts.
pub fn render_rift_crackle(
    engine: &mut ProofEngine,
    rift_pos: Vec3,
    frame: u64,
) {
    let count = 14;
    let rift_chars = ['╳', '╋', '┼', '⊗', '◈', '▣', '※'];

    for i in 0..count {
        let seed = (frame.wrapping_mul(3) + i as u64).wrapping_mul(2147483647);
        let angle = hash_f32(seed) * TAU;
        let r = hash_f32(seed + 1) * 2.0;
        let x = rift_pos.x + angle.cos() * r;
        let y = rift_pos.y + angle.sin() * r;

        // Chaotic color cycling
        let hue = hash_f32(seed + 2);
        let (cr, cg, cb) = if hue < 0.33 {
            (1.0, 0.2, 0.8)
        } else if hue < 0.66 {
            (0.2, 0.8, 1.0)
        } else {
            (1.0, 0.6, 0.1)
        };

        let flicker = hash_f32(seed + 3);
        if flicker < 0.3 {
            continue; // some particles randomly absent — feels chaotic
        }

        engine.spawn_glyph(Glyph {
            character: rift_chars[i % rift_chars.len()],
            position: Vec3::new(x, y, 0.0),
            velocity: Vec3::new(
                (angle + PI * 0.5).cos() * 1.5,
                (angle + PI * 0.5).sin() * 1.5,
                0.0,
            ),
            color: Vec4::new(cr, cg, cb, flicker),
            emission: flicker * 0.7,
            glow_color: Vec3::new(cr * 0.8, cg * 0.8, cb * 0.8),
            glow_radius: flicker * 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

/// Render trap warning: red pulsing particles at trap location.
pub fn render_trap_warning(
    engine: &mut ProofEngine,
    trap_pos: Vec3,
    frame: u64,
) {
    let pulse = (frame as f32 * 0.1).sin() * 0.4 + 0.6;
    let fast_pulse = (frame as f32 * 0.3).sin() * 0.3 + 0.7;

    // Warning symbol
    engine.spawn_glyph(Glyph {
        character: '⚠',
        position: Vec3::new(trap_pos.x, trap_pos.y + 0.8, 0.0),
        scale: Vec2::new(1.2, 1.2),
        color: Vec4::new(1.0, 0.2, 0.1, pulse),
        emission: pulse * 0.7,
        glow_color: Vec3::new(1.0, 0.2, 0.0),
        glow_radius: pulse * 0.6,
        layer: RenderLayer::Entity,
        ..Default::default()
    });

    // Ring of danger particles
    let count = 8;
    for i in 0..count {
        let angle = (i as f32 / count as f32) * TAU + frame as f32 * 0.04;
        let r = 0.8 + fast_pulse * 0.3;
        let x = trap_pos.x + angle.cos() * r;
        let y = trap_pos.y + angle.sin() * r * 0.4;

        engine.spawn_glyph(Glyph {
            character: '●',
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(1.0, 0.1, 0.05, fast_pulse * 0.6),
            emission: fast_pulse * 0.5,
            glow_color: Vec3::new(0.8, 0.0, 0.0),
            glow_radius: 0.15,
            layer: RenderLayer::World,
            ..Default::default()
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  4. BOSS VFX
// ═══════════════════════════════════════════════════════════════════════════════

/// Mirror Reflection: duplicate all player VFX at the mirrored position.
///
/// Call this with the original position and it spawns a ghost copy at -x.
pub fn render_mirror_reflection(
    engine: &mut ProofEngine,
    original_pos: Vec3,
    glyphs: &[char],
    frame: u64,
) {
    let mirrored_x = -original_pos.x;
    let flicker = ((frame as f32 * 0.15).sin() * 0.2 + 0.6).max(0.0);

    for (i, &ch) in glyphs.iter().enumerate() {
        let row = i / 4;
        let col = i % 4;
        // Mirror horizontally: negate the x-offset from center
        let x = mirrored_x - col as f32 * 0.6;
        let y = original_pos.y + row as f32 * 0.8;

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(0.5, 0.7, 1.0, flicker * 0.5),
            emission: flicker * 0.3,
            glow_color: Vec3::new(0.4, 0.6, 1.0),
            glow_radius: flicker * 0.2,
            layer: RenderLayer::Entity,
            ..Default::default()
        });
    }

    // Mirror-line particles
    for i in 0..20 {
        let y = -VIEW_HY + (i as f32 / 20.0) * VIEW_HY * 2.0;
        let shimmer = ((frame as f32 * 0.1 + i as f32 * 0.3).sin() * 0.3 + 0.5).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '│',
            position: Vec3::new(0.0, y, 0.0),
            color: Vec4::new(0.6, 0.8, 1.0, shimmer * 0.4),
            emission: shimmer * 0.3,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
}

/// Null Void: expanding dark circle that absorbs nearby particles.
///
/// `t` is 0.0..1.0 — expansion progress.
pub fn render_null_void(
    engine: &mut ProofEngine,
    center: Vec3,
    t: f32,
    frame: u64,
) {
    let max_radius = 4.0;
    let radius = t * max_radius;
    let ring_count = 24;

    // Dark interior filled with dim glyphs
    let fill_step = 0.8;
    let mut fy = center.y - radius;
    while fy <= center.y + radius {
        let mut fx = center.x - radius;
        while fx <= center.x + radius {
            let dx = fx - center.x;
            let dy = fy - center.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < radius * 0.9 {
                let darkness = (1.0 - dist / radius).clamp(0.0, 0.8);
                engine.spawn_glyph(Glyph {
                    character: '░',
                    position: Vec3::new(fx, fy, 0.1),
                    color: Vec4::new(0.05, 0.0, 0.1, darkness),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
            fx += fill_step;
        }
        fy += fill_step;
    }

    // Bright edge ring
    for i in 0..ring_count {
        let angle = (i as f32 / ring_count as f32) * TAU;
        let wobble = (frame as f32 * 0.2 + i as f32 * 1.3).sin() * 0.15;
        let r = radius + wobble;
        let x = center.x + angle.cos() * r;
        let y = center.y + angle.sin() * r;

        let edge_pulse = ((frame as f32 * 0.3 + i as f32).sin() * 0.3 + 0.7).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '◈',
            position: Vec3::new(x, y, 0.1),
            color: Vec4::new(0.3, 0.0, 0.5, edge_pulse * 0.8),
            emission: edge_pulse * 0.6,
            glow_color: Vec3::new(0.4, 0.0, 0.6),
            glow_radius: edge_pulse * 0.3,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
}

/// Algorithm Glitch: random position-offset glitches on visible glyphs.
///
/// Returns offset vectors that should be applied to all visible glyphs.
/// Call each frame during the glitch effect.
pub fn render_algorithm_glitch(
    engine: &mut ProofEngine,
    frame: u64,
    intensity: f32,
) {
    // Spawn glitch artifact glyphs at random positions
    let count = (intensity * 20.0) as usize;

    for i in 0..count {
        let seed = frame.wrapping_mul(31) + i as u64;
        let x = (hash_f32(seed) * 2.0 - 1.0) * VIEW_HX;
        let y = (hash_f32(seed + 1) * 2.0 - 1.0) * VIEW_HY;

        // Glitch characters: corrupted-looking
        let glitch_chars = ['█', '▓', '▒', '░', '╳', '┼', '▪', '■'];
        let ch_idx = (hash_f32(seed + 2) * glitch_chars.len() as f32) as usize;

        // Color corruption
        let cr = hash_f32(seed + 3);
        let cg = hash_f32(seed + 4) * 0.3;
        let cb = hash_f32(seed + 5);

        let alpha = intensity * hash_f32(seed + 6) * 0.7;
        if alpha < 0.05 {
            continue;
        }

        // Horizontal displacement bands (scan-line glitch)
        let band_offset = if ((y * 3.0) as i32 + frame as i32) % 7 < 2 {
            (hash_f32(seed + 7) - 0.5) * 3.0 * intensity
        } else {
            0.0
        };

        engine.spawn_glyph(Glyph {
            character: glitch_chars[ch_idx % glitch_chars.len()],
            position: Vec3::new(x + band_offset, y, 0.3),
            color: Vec4::new(cr, cg, cb, alpha),
            emission: alpha * 0.4,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
}

/// Hydra Spiral: golden ratio spiral particle paths.
pub fn render_hydra_spiral(
    engine: &mut ProofEngine,
    center: Vec3,
    frame: u64,
    head_count: usize,
) {
    let particles_per_arm = 20;
    let rotation = frame as f32 * 0.02;

    for arm in 0..head_count.min(8) {
        let arm_offset = (arm as f32 / head_count as f32) * TAU;

        for i in 0..particles_per_arm {
            let angle = GOLDEN_ANGLE * i as f32 + arm_offset + rotation;
            let r = (i as f32).sqrt() * 0.8;
            let x = center.x + angle.cos() * r;
            let y = center.y + angle.sin() * r;

            let depth = i as f32 / particles_per_arm as f32;
            let alpha = (1.0 - depth) * 0.8;

            // Golden color with slight arm-dependent hue shift
            let hue_shift = arm as f32 * 0.05;
            let sparkle = ((frame as f32 * 0.2 + i as f32 * 0.7).sin() * 0.2 + 0.8).max(0.0);

            engine.spawn_glyph(Glyph {
                character: if i % 5 == 0 { 'φ' } else { '·' },
                position: Vec3::new(x, y, 0.0),
                color: Vec4::new(
                    (1.0 - hue_shift).max(0.5),
                    (0.85 - hue_shift * 0.5).max(0.4),
                    0.2,
                    alpha * sparkle,
                ),
                emission: alpha * sparkle * 0.5,
                glow_color: Vec3::new(1.0, 0.85, 0.3),
                glow_radius: alpha * 0.2,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }
}

/// Committee Votes: floating checkmark / cross particles above judges.
///
/// `votes` is a slice of (position, approved) pairs.
pub fn render_committee_votes(
    engine: &mut ProofEngine,
    votes: &[(Vec3, bool)],
    t: f32,
    frame: u64,
) {
    for (idx, &(pos, approved)) in votes.iter().enumerate() {
        let (ch, color) = if approved {
            ('✓', Vec4::new(0.2, 1.0, 0.3, 1.0))
        } else {
            ('✗', Vec4::new(1.0, 0.2, 0.2, 1.0))
        };

        // Rise and scale up then fade
        let rise = t * 2.0;
        let scale_anim = if t < 0.3 {
            smoothstep(0.0, 0.3, t) * 2.0
        } else {
            2.0 - (t - 0.3) * 0.5
        };
        let alpha = if t < 0.2 {
            t / 0.2
        } else {
            (1.0 - (t - 0.2) / 0.8).max(0.0)
        };

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(pos.x, pos.y + 1.5 + rise, 0.0),
            scale: Vec2::new(scale_anim, scale_anim),
            color: Vec4::new(color.x, color.y, color.z, alpha),
            emission: alpha * 0.7,
            glow_color: Vec3::new(color.x, color.y, color.z),
            glow_radius: alpha * 0.5,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Sparkle ring around vote
        let spark_count = 4;
        for s in 0..spark_count {
            let angle = (s as f32 / spark_count as f32) * TAU + frame as f32 * 0.1;
            let sr = 0.5 * scale_anim * 0.3;
            engine.spawn_glyph(Glyph {
                character: '·',
                position: Vec3::new(
                    pos.x + angle.cos() * sr,
                    pos.y + 1.5 + rise + angle.sin() * sr,
                    0.0,
                ),
                color: Vec4::new(color.x, color.y, color.z, alpha * 0.5),
                emission: alpha * 0.3,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }
}

/// Ouroboros Ring: circular particle trail following the ouroboros cycle.
///
/// `cycle_phase` is 0.0..1.0 — position of the head along the ring.
pub fn render_ouroboros_ring(
    engine: &mut ProofEngine,
    center: Vec3,
    ring_radius: f32,
    cycle_phase: f32,
    frame: u64,
) {
    let segment_count = 36;
    let head_angle = cycle_phase * TAU;

    for i in 0..segment_count {
        let seg_phase = i as f32 / segment_count as f32;
        let angle = seg_phase * TAU;

        // Distance from head along the ring (wrapping)
        let dist_from_head = ((seg_phase - cycle_phase + 1.0) % 1.0).min(
            (cycle_phase - seg_phase + 1.0) % 1.0,
        );
        let is_tail_end = ((seg_phase - cycle_phase + 1.0) % 1.0) > 0.5;

        let x = center.x + angle.cos() * ring_radius;
        let y = center.y + angle.sin() * ring_radius * 0.5; // elliptical perspective

        // Head is bright, tail fades
        let brightness = if dist_from_head < 0.05 {
            1.0 // head
        } else if is_tail_end {
            (0.5 - dist_from_head).max(0.1) * 1.5
        } else {
            (0.5 - dist_from_head).max(0.1) * 1.2
        };

        // Segment character: head is a distinct symbol
        let ch = if dist_from_head < 0.03 {
            '◉'
        } else if dist_from_head < 0.1 {
            '●'
        } else if dist_from_head < 0.3 {
            '○'
        } else {
            '·'
        };

        let pulse = ((frame as f32 * 0.08 + i as f32 * 0.5).sin() * 0.15 + 0.85).max(0.0);
        let alpha = brightness * pulse;

        // Color gradient: head is bright gold, body green-gold, tail red
        let (cr, cg, cb) = if dist_from_head < 0.1 {
            (1.0, 0.9, 0.3) // gold head
        } else if is_tail_end {
            (0.8, 0.3, 0.1) // reddish tail
        } else {
            (0.4, 0.7, 0.2) // green body
        };

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(cr, cg, cb, alpha),
            emission: alpha * 0.5,
            glow_color: Vec3::new(cr * 0.8, cg * 0.8, cb * 0.8),
            glow_radius: alpha * 0.2,
            life_function: Some(MathFunction::Sine {
                freq: 1.0,
                amp: 0.05,
                phase: seg_phase * TAU,
            }),
            layer: RenderLayer::Entity,
            ..Default::default()
        });
    }

    // Infinity/cycle symbol at center
    let center_pulse = (frame as f32 * 0.05).sin() * 0.3 + 0.7;
    engine.spawn_glyph(Glyph {
        character: '∞',
        position: center,
        scale: Vec2::new(1.8, 1.8),
        color: Vec4::new(0.8, 0.7, 0.2, center_pulse * 0.5),
        emission: center_pulse * 0.4,
        glow_color: Vec3::new(0.9, 0.8, 0.3),
        glow_radius: center_pulse * 0.6,
        layer: RenderLayer::Entity,
        ..Default::default()
    });
}

// ═══════════════════════════════════════════════════════════════════════════════
//  5. FORCE FIELD INTEGRATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Apply a combat gravity well at the boss position, pulling particles inward.
pub fn apply_combat_gravity_well(
    engine: &mut ProofEngine,
    boss_pos: Vec3,
    strength: f32,
) {
    engine.add_field(ForceField::Gravity {
        center: boss_pos,
        strength,
        falloff: 2.0, // quadratic falloff for subtle pull
    });
}

/// Apply an impact shockwave: expanding ring of force pushing particles outward.
///
/// `radius` is the current expansion radius of the shockwave.
/// Implemented as a negative gravity (repulsion) at the impact center.
pub fn apply_impact_shockwave(
    engine: &mut ProofEngine,
    center: Vec3,
    radius: f32,
    strength: f32,
) {
    // Negative strength = repulsion; field pushes outward
    engine.add_field(ForceField::Gravity {
        center,
        strength: -strength,
        falloff: 1.5,
    });

    // Visual ring at the shockwave front
    let ring_segments = 20;
    for i in 0..ring_segments {
        let angle = (i as f32 / ring_segments as f32) * TAU;
        let x = center.x + angle.cos() * radius;
        let y = center.y + angle.sin() * radius;

        let alpha = (1.0 - radius / 8.0).clamp(0.0, 0.6);
        if alpha <= 0.0 {
            continue;
        }

        engine.spawn_glyph(Glyph {
            character: '·',
            position: Vec3::new(x, y, 0.0),
            velocity: Vec3::new(angle.cos() * 3.0, angle.sin() * 3.0, 0.0),
            color: Vec4::new(1.0, 0.9, 0.7, alpha),
            emission: alpha * 0.6,
            glow_color: Vec3::new(1.0, 0.85, 0.6),
            glow_radius: alpha * 0.3,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Screen trauma proportional to strength
    engine.add_trauma(strength * 0.15);
}

/// Apply a vortex field: swirling particles around a chaos rift.
pub fn apply_vortex_field(
    engine: &mut ProofEngine,
    center: Vec3,
    strength: f32,
    radius: f32,
) {
    engine.add_field(ForceField::Vortex {
        center,
        axis: Vec3::new(0.0, 0.0, 1.0), // spin in the screen plane
        strength,
        radius,
    });
}

/// Apply a wind effect: particles blown by weather wind direction.
pub fn apply_wind_effect(
    engine: &mut ProofEngine,
    direction: Vec3,
    strength: f32,
    turbulence: f32,
) {
    engine.add_field(ForceField::Wind {
        direction,
        strength,
        turbulence,
    });
}

/// Composite force field setup for a boss arena with gravity well + vortex.
pub fn apply_boss_arena_fields(
    engine: &mut ProofEngine,
    boss_pos: Vec3,
    boss_id: u8,
    frame: u64,
) {
    match boss_id {
        // The Null — strong gravity pulling everything in
        5 => {
            let pulse = (frame as f32 * 0.03).sin() * 0.3 + 0.7;
            apply_combat_gravity_well(engine, boss_pos, 3.0 * pulse);
        }
        // Chaos Weaver — vortex around the rift
        7 => {
            apply_vortex_field(engine, boss_pos, 2.0, 5.0);
        }
        // Void Serpent — wind pushing toward arena edges
        9 => {
            let wind_angle = frame as f32 * 0.01;
            apply_wind_effect(
                engine,
                Vec3::new(wind_angle.cos(), wind_angle.sin(), 0.0),
                1.5,
                0.4,
            );
        }
        // All other bosses — subtle gravity
        _ => {
            apply_combat_gravity_well(engine, boss_pos, 0.5);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Master render dispatcher
// ═══════════════════════════════════════════════════════════════════════════════

/// VFX event that the game logic can queue for rendering.
#[derive(Debug, Clone)]
pub enum VfxEvent {
    AttackTrail { from: Vec3, to: Vec3, progress: f32 },
    SpellProjectile { from: Vec3, to: Vec3, t: f32, element: String },
    CritExplosion { center: Vec3, t: f32 },
    DeathDissolve { center: Vec3, glyphs: Vec<char>, t: f32 },
    DamageNumber { amount: i64, position: Vec3, t: f32, is_crit: bool },
    ShieldFlash { position: Vec3, t: f32 },
    HealingSpiral { center: Vec3, t: f32 },
    StatusEffect { name: String, entity_pos: Vec3 },
    RoomEntryFlash { t: f32 },
    FloorTransitionWipe { t: f32 },
    LootSparkle { position: Vec3 },
    ShrineGlow { position: Vec3 },
    RiftCrackle { position: Vec3 },
    TrapWarning { position: Vec3 },
    MirrorReflection { position: Vec3, glyphs: Vec<char> },
    NullVoid { center: Vec3, t: f32 },
    AlgorithmGlitch { intensity: f32 },
    HydraSpiral { center: Vec3, head_count: usize },
    CommitteeVotes { votes: Vec<(Vec3, bool)>, t: f32 },
    OuroborosRing { center: Vec3, radius: f32, phase: f32 },
    ImpactShockwave { center: Vec3, radius: f32, strength: f32 },
}

/// Process a list of VFX events, dispatching to the appropriate render function.
pub fn render_vfx_events(
    engine: &mut ProofEngine,
    events: &[VfxEvent],
    frame: u64,
) {
    for event in events {
        match event {
            VfxEvent::AttackTrail { from, to, progress } => {
                render_attack_trail(engine, *from, *to, *progress, frame);
            }
            VfxEvent::SpellProjectile { from, to, t, element } => {
                render_spell_projectile(engine, *from, *to, *t, element, frame);
            }
            VfxEvent::CritExplosion { center, t } => {
                render_crit_explosion(engine, *center, *t, frame);
            }
            VfxEvent::DeathDissolve { center, glyphs, t } => {
                render_death_dissolve(engine, *center, glyphs, *t, frame);
            }
            VfxEvent::DamageNumber { amount, position, t, is_crit } => {
                render_damage_number(engine, *amount, *position, *t, *is_crit);
            }
            VfxEvent::ShieldFlash { position, t } => {
                render_shield_flash(engine, *position, *t);
            }
            VfxEvent::HealingSpiral { center, t } => {
                render_healing_spiral(engine, *center, *t, frame);
            }
            VfxEvent::StatusEffect { name, entity_pos } => {
                render_status_effect(engine, name, *entity_pos, frame);
            }
            VfxEvent::RoomEntryFlash { t } => {
                render_room_entry_flash(engine, *t);
            }
            VfxEvent::FloorTransitionWipe { t } => {
                render_floor_transition_wipe(engine, *t, frame);
            }
            VfxEvent::LootSparkle { position } => {
                render_loot_sparkle(engine, *position, frame);
            }
            VfxEvent::ShrineGlow { position } => {
                render_shrine_glow(engine, *position, frame);
            }
            VfxEvent::RiftCrackle { position } => {
                render_rift_crackle(engine, *position, frame);
            }
            VfxEvent::TrapWarning { position } => {
                render_trap_warning(engine, *position, frame);
            }
            VfxEvent::MirrorReflection { position, glyphs } => {
                render_mirror_reflection(engine, *position, glyphs, frame);
            }
            VfxEvent::NullVoid { center, t } => {
                render_null_void(engine, *center, *t, frame);
            }
            VfxEvent::AlgorithmGlitch { intensity } => {
                render_algorithm_glitch(engine, frame, *intensity);
            }
            VfxEvent::HydraSpiral { center, head_count } => {
                render_hydra_spiral(engine, *center, frame, *head_count);
            }
            VfxEvent::CommitteeVotes { votes, t } => {
                render_committee_votes(engine, votes, *t, frame);
            }
            VfxEvent::OuroborosRing { center, radius, phase } => {
                render_ouroboros_ring(engine, *center, *radius, *phase, frame);
            }
            VfxEvent::ImpactShockwave { center, radius, strength } => {
                apply_impact_shockwave(engine, *center, *radius, *strength);
            }
        }
    }
}
