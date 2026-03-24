//! The Chaos Field — the living mathematical background of CHAOS RPG.
//!
//! Runs behind EVERY screen. 150 glyphs across three parallax layers, each
//! driven by one of 10 mathematical chaos engines. Reactive to corruption,
//! floor depth, screen type, and boss state. Periodic pulse waves sweep
//! the field; the Null boss fight kills glyphs one by one.
//!
//! # Architecture
//!
//! The field is fully immediate-mode: every frame, `update()` spawns all
//! background glyphs from scratch using `engine.spawn_glyph()`. The engine
//! clears transient glyphs between frames, so no cleanup is needed.
//!
//! ## Three Parallax Layers
//!
//! | Layer | Count | Characters           | Scale | Alpha | Speed   |
//! |-------|-------|----------------------|-------|-------|---------|
//! | FAR   | 30    | Math symbols (integral, sigma, ...) | 0.5 | 0.08 | slow    |
//! | MID   | 50    | Digits & operators   | 0.3   | 0.05  | medium  |
//! | NEAR  | 70    | Tiny debris (dots)   | 0.15  | 0.03  | fast    |
//!
//! ## Ten Chaos Engines
//!
//! Each glyph is assigned to one engine by index modulo 10:
//! 0=Linear, 1=Lorenz, 2=Mandelbrot, 3=Fibonacci, 4=Collatz,
//! 5=Lissajous, 6=Pendulum, 7=Sine, 8=Perlin, 9=Orbit.
//!
//! ## Reactivity
//!
//! - **Corruption** (0-1): speed increase, purple color shift, glitch chars
//! - **Floor depth**: density and speed multipliers
//! - **Screen type**: Combat ripples, Shrine calm, Rift maximum chaos
//! - **Null boss**: field freezes and dies glyph-by-glyph

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// CHARACTER SETS
// ═══════════════════════════════════════════════════════════════════════════════

const FAR_CHARS: &[char] = &[
    '\u{222B}', '\u{2211}', '\u{220F}', '\u{03A9}', '\u{221E}', '\u{2207}',
    '\u{2202}', '\u{03C6}', '\u{03C0}', '\u{03BB}', '\u{03B6}', '\u{0394}',
];
const MID_CHARS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    '+', '-', '=', '*', '/', '%', '^', '~', '<', '>',
];
const NEAR_CHARS: &[char] = &['\u{00B7}', '\u{00B7}', ',', '.', '`', '\'', '\u{2219}', '\u{2027}'];

const GLITCH_CHARS: &[char] = &[
    '\u{2588}', '\u{2591}', '\u{2592}', '\u{2593}', '\u{25A0}',
    '\u{00BF}', '\u{00D8}', '\u{2260}', '\u{2248}', '\u{221A}',
];

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

const FAR_COUNT: usize = 8;
const MID_COUNT: usize = 12;
const NEAR_COUNT: usize = 15;
const GOLDEN_ANGLE: f32 = 2.399_963_2;
const VIEW_X: f32 = 9.5;
const VIEW_Y: f32 = 6.5;

// ═══════════════════════════════════════════════════════════════════════════════
// CHAOS ENGINE
// ═══════════════════════════════════════════════════════════════════════════════

/// Which mathematical engine drives a glyph's motion.
fn engine_position(engine_idx: usize, t: f32, seed: u64, bx: f32, by: f32, intensity: f32) -> (f32, f32) {
    match engine_idx % 10 {
        0 => { // Linear: straight vertical fall
            let speed = 0.5 + hf(seed) * 1.5;
            let y = ((by + t * speed * intensity) % (VIEW_Y * 2.0 + 2.0)) - VIEW_Y - 1.0;
            (bx + (t * 0.3 + hf(seed + 1) * 6.28).sin() * 0.15, y)
        }
        1 => { // Lorenz: chaotic orbits via irrational frequency ratios
            let fx = hf(seed + 10) * 0.3 + 0.1;
            let fy = fx * std::f32::consts::SQRT_2;
            let ax = 2.0 + hf(seed + 11) * 3.0;
            let ay = 1.5 + hf(seed + 12) * 2.5;
            (bx + (t * fx * intensity + hf(seed + 13) * 6.28).sin() * ax,
             by + (t * fy * intensity + hf(seed + 14) * 6.28).cos() * ay)
        }
        2 => { // Mandelbrot: escape-boundary radial motion
            let cx = hs(seed + 20) * 3.0;
            let cy = hs(seed + 21) * 2.0;
            let r = 2.0 + (t * 0.2 * intensity + hf(seed + 22) * 6.28).sin() * 1.5;
            let a = t * 0.15 * intensity + hf(seed + 23) * 6.28;
            (cx + a.cos() * r, cy + a.sin() * r)
        }
        3 => { // Fibonacci: golden spiral paths
            let idx = (seed & 0xFF) as f32;
            let a = GOLDEN_ANGLE * idx + t * 0.2 * intensity;
            let r = (idx * 0.08).sqrt() * 3.0 * (1.0 + (t * 0.1 + idx * 0.01).sin() * 0.2);
            (a.cos() * r, a.sin() * r)
        }
        4 => { // Collatz: triangle wave bounce with 3n+1 spikes
            let freq = 0.4 + hf(seed + 40) * 0.6;
            let phase = hf(seed + 41) * 6.28;
            let tri = ((t * freq * intensity + phase) % 2.0 - 1.0).abs() * 2.0 - 1.0;
            let spike_p = (t * freq * 0.33 * intensity + phase).sin();
            let spike = if spike_p > 0.85 { (spike_p - 0.85) * 10.0 } else { 0.0 };
            let y = ((by + tri * 3.0 + spike * 2.0 + VIEW_Y + 1.0) % (VIEW_Y * 2.0 + 2.0)) - VIEW_Y - 1.0;
            (bx + (t * 0.05 + hf(seed + 42) * 6.28).sin() * 0.5, y)
        }
        5 => { // Lissajous: parametric figure paths
            let a = 2.0 + (seed % 3) as f32;
            let b = 3.0 + (seed % 4) as f32;
            let s = 2.5 + hf(seed + 51) * 2.0;
            ((a * t * 0.2 * intensity + hf(seed + 50) * 3.14).sin() * s,
             (b * t * 0.2 * intensity).sin() * s * 0.7)
        }
        6 => { // Pendulum: damped swing
            let len = 3.0 + hf(seed + 60) * 4.0;
            let freq = (9.81 / len).sqrt() * 0.5;
            let amp = 4.0 * (-0.008 * t).exp().max(0.3);
            let angle = (t * freq * intensity + hf(seed + 62) * 6.28).sin() * amp;
            (bx + angle, by + (1.0 - angle.abs() / 4.0) * 0.5)
        }
        7 => { // Sine: vertical fall with horizontal weave
            let sy = 0.3 + hf(seed + 70) * 0.8;
            let fx = 0.3 + hf(seed + 71) * 0.5;
            let ax = 1.0 + hf(seed + 72) * 2.0;
            let y = ((by + t * sy * intensity) % (VIEW_Y * 2.0 + 2.0)) - VIEW_Y - 1.0;
            (bx + (t * fx * intensity + hf(seed + 73) * 6.28).sin() * ax, y)
        }
        8 => { // Perlin: layered-sine noise drift
            let nx = (t * 0.13 * intensity + hf(seed + 80) * 100.0).sin() * 0.7
                   + (t * 0.31 * intensity + hf(seed + 81) * 100.0).sin() * 0.3
                   + (t * 0.71 * intensity + hf(seed + 82) * 100.0).sin() * 0.15;
            let ny = (t * 0.17 * intensity + hf(seed + 83) * 100.0).sin() * 0.7
                   + (t * 0.37 * intensity + hf(seed + 84) * 100.0).sin() * 0.3;
            (bx + nx * 3.0, by + ny * 2.5)
        }
        _ => { // Orbit: elliptical
            let cx = hs(seed + 90) * 2.0;
            let cy = hs(seed + 91) * 1.5;
            let rx = 1.5 + hf(seed + 92) * 3.0;
            let ry = 1.0 + hf(seed + 93) * 2.0;
            let a = t * (0.2 + hf(seed + 94) * 0.4) * intensity + hf(seed + 95) * 6.28;
            (cx + a.cos() * rx, cy + a.sin() * ry)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

#[inline] fn h(s: u64) -> u64 { let mut h = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); h ^= h >> 33; h.wrapping_mul(0xff51afd7ed558ccd) }
#[inline] fn hf(s: u64) -> f32 { (h(s) & 0x00FF_FFFF) as f32 / 16_777_216.0 }
#[inline] fn hs(s: u64) -> f32 { hf(s) * 2.0 - 1.0 }
#[inline] fn wrap(v: f32, lim: f32) -> f32 { let r = lim * 2.0; ((v + lim) % r + r) % r - lim }

fn glitch_char(seed: u64) -> char { GLITCH_CHARS[(h(seed) as usize) % GLITCH_CHARS.len()] }

/// Pulse wave: horizontal brightness sweep every ~15 seconds.
fn pulse_brightness(t: f32, x: f32) -> f32 {
    let cycle_t = t % 15.0;
    if cycle_t > 2.5 { return 0.0; }
    let pulse_x = -VIEW_X + (cycle_t / 2.5) * (VIEW_X * 2.0 + 3.0);
    let dist = (x - pulse_x).abs();
    if dist < 3.0 { 0.15 * (1.0 - dist / 3.0) } else { 0.0 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCREEN MODIFIERS
// ═══════════════════════════════════════════════════════════════════════════════

struct Mods { speed: f32, alpha: f32, color: Vec3, ripple: f32 }

fn screen_mods(state: &GameState) -> Mods {
    match state.screen {
        AppScreen::Combat => {
            let r = (state.player_flash + state.enemy_flash).clamp(0.0, 1.0);
            Mods { speed: 1.2, alpha: 1.0, color: Vec3::new(r * 0.05, 0.0, 0.0), ripple: r }
        }
        AppScreen::Shop | AppScreen::Crafting =>
            Mods { speed: 0.4, alpha: 0.7, color: Vec3::ZERO, ripple: 0.0 },
        AppScreen::GameOver =>
            Mods { speed: 0.1, alpha: 0.3, color: Vec3::new(-0.02, -0.02, -0.02), ripple: 0.0 },
        AppScreen::Victory =>
            Mods { speed: 0.6, alpha: 1.2, color: Vec3::new(0.02, 0.04, 0.02), ripple: 0.0 },
        AppScreen::Title =>
            Mods { speed: 0.8, alpha: 1.1, color: Vec3::ZERO, ripple: 0.0 },
        _ => Mods { speed: 1.0, alpha: 1.0, color: Vec3::ZERO, ripple: 0.0 },
    }
}

fn is_chaos_rift(state: &GameState) -> bool {
    state.floor.as_ref().map_or(false, |f| {
        use chaos_rpg_core::world::RoomType;
        f.current().room_type == RoomType::ChaosRift
    })
}
fn is_shrine(state: &GameState) -> bool {
    state.floor.as_ref().map_or(false, |f| {
        use chaos_rpg_core::world::RoomType;
        f.current().room_type == RoomType::Shrine
    })
}
fn is_null_fight(state: &GameState) -> bool { state.boss_id == Some(10) }

/// Null boss death: returns (alive_fraction, freeze_factor).
fn null_death(state: &GameState) -> (f32, f32) {
    if !is_null_fight(state) { return (1.0, 0.0); }
    let turn = state.boss_turn as f32;
    let frac = (turn / 20.0).clamp(0.0, 1.0);
    (1.0 - frac, (frac * 1.5).clamp(0.0, 1.0))
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize (no-op; field is fully immediate-mode).
pub fn init(_state: &GameState, _engine: &mut ProofEngine) {}

/// Render the chaos field. Called every frame.
pub fn update(state: &GameState, engine: &mut ProofEngine, _dt: f32) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 / 60.0;
    let corruption = state.corruption_frac();
    let brightness = theme.chaos_field_brightness;

    // Floor-based speed & density
    let (floor_speed, floor_density) = match state.floor_num {
        0..=10  => (1.0_f32, 1.0_f32),
        11..=25 => (1.3, 1.1),
        26..=50 => (1.7, 1.2),
        51..=75 => (2.2, 1.3),
        76..=99 => (2.8, 1.35),
        _       => (3.5, 1.4),
    };

    let mods = screen_mods(state);
    let mut speed = floor_speed * mods.speed;
    if is_shrine(state) { speed *= 0.3; }
    let rift = is_chaos_rift(state);
    if rift { speed *= 2.5; }
    let rift_boost = if rift { 0.08 } else { 0.0 };

    let (null_alive, null_freeze) = null_death(state);
    speed *= 1.0 - null_freeze;

    // Corruption purple shift
    let base_color = Vec3::new(
        (theme.muted.x + corruption * 0.15 + mods.color.x).clamp(0.0, 1.0),
        (theme.muted.y - corruption * 0.05 + mods.color.y).clamp(0.0, 1.0),
        (theme.muted.z + corruption * 0.25 + mods.color.z).clamp(0.0, 1.0),
    );
    let accent = Vec3::new(theme.accent.x, theme.accent.y, theme.accent.z);
    let ripple = mods.ripple;

    // ── Helper closure: apply combat ripple ──
    let ripple_push = |x: &mut f32, y: &mut f32, strength: f32| {
        if ripple > 0.01 {
            let d = (*x * *x + *y * *y).sqrt().max(0.1);
            let p = ripple * strength / d;
            *x += *x * p;
            *y += *y * p;
        }
    };

    // ════════════════════════════════════════════════════════════════════════
    // FAR LAYER — 30 large math symbols, scale 0.5, alpha 0.08, slow
    // ════════════════════════════════════════════════════════════════════════

    let far_n = ((FAR_COUNT as f32 * floor_density * null_alive) as usize).max(1);
    for i in 0..far_n {
        let seed = h(i as u64 + 1000);
        let (mut x, mut y) = engine_position(
            i, t, seed, hs(seed + 1) * VIEW_X, hs(seed + 2) * VIEW_Y, speed * 0.5,
        );
        ripple_push(&mut x, &mut y, 0.8);
        x = wrap(x, VIEW_X);
        y = wrap(y, VIEW_Y);

        let ch = if corruption > 0.5 && hf(seed + state.frame) < (corruption - 0.5) * 0.1 {
            glitch_char(seed + state.frame)
        } else {
            FAR_CHARS[i % FAR_CHARS.len()]
        };

        let ab = 0.15 + corruption * 0.3; // accent blend
        let col = base_color * (1.0 - ab) + accent * ab;
        let a = (brightness * 0.03 + pulse_brightness(t, x) * 0.3 + rift_boost * 0.5) * mods.alpha;

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 2.0),
            scale: Vec2::splat(0.25),
            color: Vec4::new(col.x * a, col.y * a, col.z * a, a),
            emission: a * 0.15,
            glow_color: col, glow_radius: 0.3,
            rotation: (t * 0.05 + i as f32 * 0.5).sin() * 0.15,
            entropy: corruption * 0.5,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ════════════════════════════════════════════════════════════════════════
    // MID LAYER — 50 digits/operators, scale 0.3, alpha 0.05, medium
    // ════════════════════════════════════════════════════════════════════════

    let mid_n = ((MID_COUNT as f32 * floor_density * null_alive) as usize).max(1);
    for i in 0..mid_n {
        let seed = h(i as u64 + 5000);
        let (mut x, mut y) = engine_position(
            i + 3, t, seed, hs(seed + 1) * VIEW_X * 1.1, hs(seed + 2) * VIEW_Y * 1.1, speed * 0.8,
        );
        ripple_push(&mut x, &mut y, 0.5);
        x = wrap(x, VIEW_X * 1.1);
        y = wrap(y, VIEW_Y * 1.1);

        let ch = if corruption > 0.3 && hf(seed + state.frame) < (corruption - 0.3) * 0.08 {
            glitch_char(seed + state.frame)
        } else {
            MID_CHARS[i % MID_CHARS.len()]
        };

        let col = base_color * 0.9 + accent * 0.1;
        let a = (brightness * 0.02 + pulse_brightness(t, x) * 0.2 + rift_boost * 0.3) * mods.alpha;

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 3.0),
            scale: Vec2::splat(0.08),
            color: Vec4::new(col.x * a, col.y * a, col.z * a, a),
            emission: a * 0.08,
            rotation: (t * 0.08 + i as f32 * 0.3).cos() * 0.1,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ════════════════════════════════════════════════════════════════════════
    // NEAR LAYER — 70 tiny debris, scale 0.15, alpha 0.03, fast
    // ════════════════════════════════════════════════════════════════════════

    let near_n = ((NEAR_COUNT as f32 * floor_density * null_alive) as usize).max(1);
    for i in 0..near_n {
        let seed = h(i as u64 + 10000);
        let (mut x, mut y) = engine_position(
            i + 7, t, seed, hs(seed + 1) * VIEW_X * 1.2, hs(seed + 2) * VIEW_Y * 1.2, speed * 1.4,
        );
        ripple_push(&mut x, &mut y, 0.3);
        x = wrap(x, VIEW_X * 1.2);
        y = wrap(y, VIEW_Y * 1.2);

        let ch = NEAR_CHARS[i % NEAR_CHARS.len()];
        let a = (brightness * 0.01 + pulse_brightness(t, x) * 0.15 + rift_boost * 0.2) * mods.alpha;

        // Deep corruption flicker
        let flicker = if corruption > 0.7 && hf(seed + state.frame * 3) < (corruption - 0.7) * 0.3 {
            0.0
        } else { 1.0 };
        let a = a * flicker;

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 4.0),
            scale: Vec2::splat(0.08),
            color: Vec4::new(base_color.x * a, base_color.y * a, base_color.z * a, a),
            emission: a * 0.03,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ════════════════════════════════════════════════════════════════════════
    // CORRUPTION GLITCH OVERLAY
    // At high corruption, spawn a handful of bright glitch symbols that
    // flash in random positions for single frames, creating visual static.
    // ════════════════════════════════════════════════════════════════════════

    if corruption > 0.6 {
        let glitch_count = ((corruption - 0.6) * 25.0) as usize; // 0-10 glitches
        for i in 0..glitch_count {
            let seed = h(state.frame.wrapping_mul(7).wrapping_add(i as u64));
            // Only show ~40% of the time for a flickering effect
            if hf(seed + 99) > 0.4 { continue; }
            let gx = hs(seed + 1) * VIEW_X * 0.8;
            let gy = hs(seed + 2) * VIEW_Y * 0.8;
            let ga = 0.06 + hf(seed + 3) * 0.08;
            engine.spawn_glyph(Glyph {
                character: glitch_char(seed),
                position: Vec3::new(gx, gy, 2.5),
                scale: Vec2::splat(0.2 + hf(seed + 4) * 0.3),
                color: Vec4::new(
                    accent.x * ga * 1.5,
                    accent.y * ga * 0.3,
                    accent.z * ga * 1.8,
                    ga,
                ),
                emission: ga * 2.0,
                glow_color: Vec3::new(accent.x, 0.1, accent.z),
                glow_radius: 0.5,
                blend_mode: BlendMode::Additive,
                layer: RenderLayer::Background,
                ..Default::default()
            });
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // CHAOS RIFT: VORTEX CENTER GLYPH
    // When in a ChaosRift room, spawn a bright swirling symbol at center
    // that all nearby glyphs appear to orbit around.
    // ════════════════════════════════════════════════════════════════════════

    if rift {
        let vortex_spin = t * 2.0;
        let vortex_pulse = 0.5 + 0.5 * (t * 1.2).sin();
        let va = 0.12 * vortex_pulse;
        engine.spawn_glyph(Glyph {
            character: '\u{2609}', // Sun symbol as vortex center
            position: Vec3::new(0.0, 0.0, 2.0),
            scale: Vec2::splat(0.8 + vortex_pulse * 0.3),
            color: Vec4::new(accent.x * va, accent.y * va, accent.z * va, va),
            emission: va * 3.0,
            glow_color: Vec3::new(accent.x, accent.y, accent.z),
            glow_radius: 3.0,
            rotation: vortex_spin,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ════════════════════════════════════════════════════════════════════════
    // NULL BOSS: FROZEN GLYPH CORPSES
    // Glyphs that have "died" — frozen, grey, slowly fading to nothing.
    // The field DIES. Every glyph stops. One by one. Terrifying.
    // ════════════════════════════════════════════════════════════════════════

    if is_null_fight(state) && null_alive < 0.95 {
        let dead = ((1.0 - null_alive) * (FAR_COUNT + MID_COUNT + NEAR_COUNT) as f32) as usize;
        let ga = 0.02 * null_alive;
        for i in 0..dead.min(50) {
            let seed = h(i as u64 + 50000);
            // Each dead glyph has a unique "time of death" based on its index
            // relative to the boss turn progression — they don't all die at once
            let death_turn = (i as f32 / 50.0 * 20.0) as u32;
            let time_dead = state.boss_turn.saturating_sub(death_turn) as f32;
            let fade = (1.0 - time_dead * 0.1).clamp(0.0, 1.0);
            let final_a = ga * fade;
            if final_a < 0.001 { continue; }

            let ch = if i < FAR_COUNT { FAR_CHARS[i % FAR_CHARS.len()] }
                     else if i < FAR_COUNT + MID_COUNT { MID_CHARS[i % MID_CHARS.len()] }
                     else { NEAR_CHARS[i % NEAR_CHARS.len()] };
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(hs(seed + 1) * VIEW_X, hs(seed + 2) * VIEW_Y, 3.0),
                scale: Vec2::splat(0.2),
                color: Vec4::new(final_a, final_a, final_a, final_a),
                layer: RenderLayer::Background,
                ..Default::default()
            });
        }
    }
}
