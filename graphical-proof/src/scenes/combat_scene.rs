//! Combat scene builder — constructs the 3D arena for combat encounters.
//!
//! Spawns a perspective floor grid (pattern varies by floor epoch), arena walls,
//! entity pedestals, ambient force fields, and lighting glyphs.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;
use crate::lighting::{SceneLighting, RoomLighting};

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

const GRID_COLS: i32 = 10;
const GRID_ROWS: i32 = 6;
const GRID_SPACING: f32 = 0.8;
const WALL_X: f32 = 7.0;
const WALL_HEIGHT: i32 = 8;
const PLAYER_X: f32 = -4.0;
const ENEMY_X: f32 = 4.0;
const PEDESTAL_COUNT: usize = 12;
const PEDESTAL_RADIUS: f32 = 1.2;

const FLOOR_MATH: &[char] = &['\u{03C0}', '\u{03C6}', '\u{2207}', '\u{2202}', '\u{221E}', '\u{2211}', '\u{222B}', '\u{0394}'];
const FLOOR_CRACKED: &[char] = &['/', '\\', '|', '_', '#', '%', '\u{2591}', '\u{2592}'];
const FLOOR_BROKEN: &[char] = &['\u{2588}', '\u{2593}', '\u{2592}', '\u{2591}', '#', '\u{25A0}', '!', '?'];
const PEDESTAL_CHARS: &[char] = &['\u{25CB}', '\u{25CF}', '\u{25C6}', '\u{25C7}', '\u{2022}', '\u{2218}'];

#[inline] fn h(s: u64) -> u64 { let mut v = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); v ^= v >> 33; v.wrapping_mul(0xff51afd7ed558ccd) }
#[inline] fn hf(s: u64) -> f32 { (h(s) & 0x00FF_FFFF) as f32 / 16_777_216.0 }

// ═══════════════════════════════════════════════════════════════════════════════
// FLOOR GRID PATTERNS
// ═══════════════════════════════════════════════════════════════════════════════

fn floor_char(floor: u32, gx: i32, gz: i32, seed: u64) -> char {
    let idx = ((gx.unsigned_abs() + gz.unsigned_abs()) as u64).wrapping_add(seed);
    match floor {
        0..=10 => { // Orderly dots
            if (gx + gz) % 2 == 0 { '\u{00B7}' } else { ' ' }
        }
        11..=25 => { // Dots with operators
            if (gx + gz) % 2 == 0 {
                ['+', '\u{00B7}', '-', '\u{00B7}', '=', '\u{00B7}'][(idx as usize) % 6]
            } else { ' ' }
        }
        26..=50 => { // Mathematical symbols
            if (gx + gz) % 3 == 0 { FLOOR_MATH[(idx as usize) % FLOOR_MATH.len()] }
            else if (gx + gz) % 2 == 0 { '\u{00B7}' }
            else { ' ' }
        }
        51..=75 => { // Cracked — some missing tiles
            if hf(idx) < 0.15 { ' ' } else { FLOOR_CRACKED[(idx as usize) % FLOOR_CRACKED.len()] }
        }
        _ => { // Broken — heavy gaps
            if hf(idx) < 0.3 { ' ' } else { FLOOR_BROKEN[(idx as usize) % FLOOR_BROKEN.len()] }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FORCE FIELDS
// ═══════════════════════════════════════════════════════════════════════════════

enum ForceField { Wind(f32, f32), GravityWell(f32), Vortex(f32) }

fn room_field(state: &GameState) -> ForceField {
    if state.is_boss_fight { return ForceField::GravityWell(0.3); }
    if state.floor.as_ref().map_or(false, |f| {
        use chaos_rpg_core::world::RoomType;
        f.current().room_type == RoomType::ChaosRift
    }) { return ForceField::Vortex(0.5); }
    ForceField::Wind(0.15, 0.05)
}

fn apply_field(field: &ForceField, x: f32, y: f32, t: f32) -> (f32, f32) {
    match *field {
        ForceField::Wind(dx, dy) => ((t * 0.3).sin() * dx, (t * 0.2).cos() * dy),
        ForceField::GravityWell(s) => {
            let d = (x * x + y * y).sqrt().max(0.5);
            let p = s / d * (1.0 + (t * 0.5).sin() * 0.3) * 0.02;
            (-x * p, -y * p)
        }
        ForceField::Vortex(s) => {
            let d = (x * x + y * y).sqrt().max(0.5);
            let a = s / d * (1.0 + (t * 1.5).sin() * 0.4) * 0.03;
            (-y * a, x * a)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Build the combat arena and return the scene lighting state.
pub fn build_arena(state: &GameState, engine: &mut ProofEngine) -> SceneLighting {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 / 60.0;
    let floor = state.floor_num;
    let corruption = state.corruption_frac();
    let field = room_field(state);

    let room_type = if state.is_boss_fight { RoomLighting::Boss }
        else if state.floor.as_ref().map_or(false, |f| {
            use chaos_rpg_core::world::RoomType;
            f.current().room_type == RoomType::ChaosRift
        }) { RoomLighting::ChaosRift }
        else { RoomLighting::Combat };

    let mut lighting = SceneLighting::new();
    lighting.setup_room(room_type, floor, Vec3::ZERO);

    // ── 1. FLOOR GRID — 20x12 perspective grid at y=-3 ──────────────────

    let floor_y = -3.0;
    let max_dist_sq = (GRID_COLS * GRID_COLS + GRID_ROWS * GRID_ROWS) as f32;
    for gx in -GRID_COLS..=GRID_COLS {
        for gz in -GRID_ROWS..=GRID_ROWS {
            let ch = floor_char(floor, gx, gz, state.seed);
            if ch == ' ' { continue; }

            let x = gx as f32 * GRID_SPACING;
            let z = gz as f32 * GRID_SPACING;
            let fade = (1.0 - (gx * gx + gz * gz) as f32 / max_dist_sq).max(0.0);
            let alpha = fade * 0.18;
            if alpha < 0.005 { continue; }

            let (fx, fy) = apply_field(&field, x, z, t);
            let jx = if corruption > 0.4 {
                (hf((gx as u64).wrapping_mul(31).wrapping_add(gz as u64).wrapping_add(state.frame)) - 0.5) * corruption * 0.15
            } else { 0.0 };

            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x + fx + jx, floor_y + fy, z),
                scale: Vec2::splat(0.2 + fade * 0.15),
                color: Vec4::new(theme.muted.x * alpha, theme.muted.y * alpha, theme.muted.z * alpha, alpha),
                emission: alpha * 0.15,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }
    }

    // ── 2. ARENA WALLS — faint vertical lines at x=±7 ───────────────────

    for side in [-1.0_f32, 1.0] {
        let wx = WALL_X * side;
        for wy in 0..WALL_HEIGHT {
            let y = -3.0 + wy as f32 * 0.9;
            let hfade = 1.0 - wy as f32 / WALL_HEIGHT as f32;
            let a = 0.06 * hfade * (1.0 + (t * 0.3 + wy as f32 * 0.4).sin() * 0.15);
            engine.spawn_glyph(Glyph {
                character: '\u{2502}', // │
                position: Vec3::new(wx, y, 0.0),
                scale: Vec2::new(0.15, 0.35),
                color: Vec4::new(theme.border.x * a, theme.border.y * a, theme.border.z * a, a),
                emission: a * 0.1,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }
    }

    // ── 3. ENTITY PEDESTALS — bright glyph rings ────────────────────────

    let enemy_tint = state.enemy.as_ref()
        .and_then(|e| crate::lighting::enemy_element_tint(&e.name))
        .map(|v| Vec4::new(v.x, v.y, v.z, 1.0))
        .unwrap_or(theme.danger);

    spawn_pedestal(engine, t, PLAYER_X, floor_y + 0.05, theme.accent, state.frame);
    spawn_pedestal(engine, t, ENEMY_X, floor_y + 0.05, enemy_tint, state.frame);

    // ── 4. FORCE FIELD PARTICLES ─────────────────────────────────────────

    let ff_chars: &[char] = match field {
        ForceField::Wind(..)        => &['\u{2192}', '\u{00B7}', '-'],
        ForceField::GravityWell(..) => &['\u{25CB}', '\u{00B7}', '\u{25CF}'],
        ForceField::Vortex(..)      => &['\u{21BB}', '\u{00B7}', '\u{21BA}'],
    };
    for i in 0..16_usize {
        let seed = h(i as u64 + 30000);
        let phase = hf(seed) * std::f32::consts::TAU;
        let r = 2.0 + hf(seed + 1) * 5.0;
        let ba = phase + t * 0.1;
        let bx = ba.cos() * r;
        let by = -2.0 + hf(seed + 2) * 4.0;
        let (fx, fy) = apply_field(&field, bx, by, t + phase);
        let a = (0.03 + corruption * 0.02) * (0.5 + 0.5 * (t * 0.5 + phase).sin());
        engine.spawn_glyph(Glyph {
            character: ff_chars[i % ff_chars.len()],
            position: Vec3::new(bx + fx * 10.0, by + fy * 10.0, ba.sin() * r * 0.5),
            scale: Vec2::splat(0.12),
            color: Vec4::new(theme.dim.x * a, theme.dim.y * a, theme.dim.z * a, a),
            emission: a * 0.2,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::World,
            ..Default::default()
        });
    }

    // ── 5. LIGHTING GLYPHS — bright points at key positions ──────────────

    let lights: &[(f32, f32, f32, f32)] = if state.is_boss_fight {
        &[(0.0, 4.0, 0.0, 1.0), (-4.0, 3.0, 0.0, 0.6), (4.0, 3.0, 0.0, 0.8),
          (-6.5, 1.0, 0.0, 0.3), (6.5, 1.0, 0.0, 0.3)]
    } else {
        &[(0.0, 3.5, 0.0, 0.7), (-4.0, 2.5, 0.0, 0.5), (4.0, 2.5, 0.0, 0.5)]
    };
    for (i, &(lx, ly, lz, inten)) in lights.iter().enumerate() {
        let flicker = 0.85 + 0.15 * (t * 3.0 + i as f32 * 1.7).sin() * (t * 7.3 + i as f32 * 0.9).sin();
        let a = inten * flicker * 0.3;
        let tint = if state.is_boss_fight && corruption > 0.3 {
            let c = (corruption - 0.3) * 1.4;
            Vec3::new(theme.heading.x * (1.0 - c) + theme.danger.x * c,
                      theme.heading.y * (1.0 - c * 0.5),
                      theme.heading.z * (1.0 - c) + theme.danger.z * c * 0.3)
        } else {
            Vec3::new(theme.heading.x, theme.heading.y, theme.heading.z)
        };
        engine.spawn_glyph(Glyph {
            character: '\u{2736}', // ✶
            position: Vec3::new(lx, ly, lz),
            scale: Vec2::splat(0.15 + inten * 0.1),
            color: Vec4::new(tint.x * a, tint.y * a, tint.z * a, a),
            emission: inten * flicker * 2.0,
            glow_color: tint, glow_radius: 2.0 + inten * 2.0,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::World,
            ..Default::default()
        });
    }

    // ── 6. ENGINE LIGHTING ───────────────────────────────────────────────

    lighting.add_player_light(Vec3::new(PLAYER_X, 0.0, 0.0));
    lighting.add_enemy_light(
        Vec3::new(ENEMY_X, 0.0, 0.0),
        state.enemy.as_ref().and_then(|e| crate::lighting::enemy_element_tint(&e.name)),
    );
    if let Some(boss_id) = state.boss_id {
        crate::lighting::apply_boss_lighting(&mut lighting, boss_id, state.boss_turn, Vec3::ZERO);
    }

    lighting
}

// ═══════════════════════════════════════════════════════════════════════════════
// PEDESTAL HELPER
// ═══════════════════════════════════════════════════════════════════════════════

fn spawn_pedestal(engine: &mut ProofEngine, t: f32, cx: f32, y: f32, tint: Vec4, frame: u64) {
    for i in 0..PEDESTAL_COUNT {
        let angle = (i as f32 / PEDESTAL_COUNT as f32) * std::f32::consts::TAU;
        let r = PEDESTAL_RADIUS * (1.0 + (t * 0.8 + i as f32 * 0.3).sin() * 0.1);
        let px = cx + angle.cos() * r;
        let pz = angle.sin() * r * 0.5;
        let pulse = 0.5 + 0.5 * (t * 2.0 - i as f32 * 0.5).sin();
        let a = 0.25 * (0.6 + pulse * 0.4);
        engine.spawn_glyph(Glyph {
            character: PEDESTAL_CHARS[((i + frame as usize / 30) % PEDESTAL_CHARS.len())],
            position: Vec3::new(px, y, pz),
            scale: Vec2::splat(0.2),
            color: Vec4::new(tint.x * a, tint.y * a, tint.z * a, a),
            emission: a * 0.8,
            glow_color: Vec3::new(tint.x, tint.y, tint.z),
            glow_radius: 0.5,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::World,
            ..Default::default()
        });
    }
    // Center glow
    let ca = 0.15 * (0.5 + 0.5 * (t * 1.5).sin());
    engine.spawn_glyph(Glyph {
        character: '\u{25CE}', // ◎
        position: Vec3::new(cx, y, 0.0),
        scale: Vec2::splat(0.4),
        color: Vec4::new(tint.x * ca, tint.y * ca, tint.z * ca, ca),
        emission: ca * 1.5,
        glow_color: Vec3::new(tint.x, tint.y, tint.z),
        glow_radius: 1.0,
        blend_mode: BlendMode::Additive,
        layer: RenderLayer::World,
        ..Default::default()
    });
}
