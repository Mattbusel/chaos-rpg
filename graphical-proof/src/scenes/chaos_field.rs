//! The Chaos Field — the living mathematical background of CHAOS RPG.
//!
//! Runs behind EVERY screen. 150 glyphs across three parallax layers, each
//! driven by one of 10 mathematical chaos engines. Reactive to corruption,
//! floor depth, screen type, and boss state. Includes periodic pulse waves
//! and the terrifying Null boss death sequence.
//!
//! Glyph budget: ~150 background glyphs per frame (30 far + 50 mid + 70 near).

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// CHARACTER SETS
// ═══════════════════════════════════════════════════════════════════════════════

const FAR_CHARS: &[char] = &[
    '\u{222B}', // ∫
    '\u{2211}', // ∑
    '\u{220F}', // ∏
    '\u{03A9}', // Ω
    '\u{221E}', // ∞
    '\u{2207}', // ∇
    '\u{2202}', // ∂
    '\u{03C6}', // φ
    '\u{03C0}', // π
    '\u{03BB}', // λ
    '\u{03B6}', // ζ
    '\u{0394}', // Δ
];

const MID_CHARS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    '+', '-', '=', '*', '/', '%', '^', '~', '<', '>',
];

const NEAR_CHARS: &[char] = &[
    '\u{00B7}', // ·
    '\u{00B7}', // ·
    ',', '.', '`', '\'',
    '\u{00B7}', // ·
    '\u{2219}', // ∙
    '\u{2027}', // ‧
];

// ═══════════════════════════════════════════════════════════════════════════════
// CHAOS ENGINE ASSIGNMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// Which mathematical engine drives a glyph's motion.
#[derive(Clone, Copy, Debug)]
enum ChaosEngine {
    Linear,       // 0: straight vertical fall
    Lorenz,       // 1: chaotic 2D orbits via irrational frequency ratios
    Mandelbrot,   // 2: escape-boundary radial motion
    Fibonacci,    // 3: golden spiral paths
    Collatz,      // 4: triangle-wave bounce (3n+1 / n/2 pattern)
    Lissajous,    // 5: parametric figure-8 paths
    Pendulum,     // 6: damped swing
    Sine,         // 7: sinusoidal weave
    Perlin,       // 8: noise-driven drift
    Orbit,        // 9: elliptical orbit
}

impl ChaosEngine {
    fn from_index(i: usize) -> Self {
        match i % 10 {
            0 => ChaosEngine::Linear,
            1 => ChaosEngine::Lorenz,
            2 => ChaosEngine::Mandelbrot,
            3 => ChaosEngine::Fibonacci,
            4 => ChaosEngine::Collatz,
            5 => ChaosEngine::Lissajous,
            6 => ChaosEngine::Pendulum,
            7 => ChaosEngine::Sine,
            8 => ChaosEngine::Perlin,
            _ => ChaosEngine::Orbit,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

const FAR_COUNT: usize = 30;
const MID_COUNT: usize = 50;
const NEAR_COUNT: usize = 70;

/// Golden angle in radians (used by Fibonacci engine).
const GOLDEN_ANGLE: f32 = 2.399_963_2; // 2π / φ²

/// Visible area bounds (camera at z=-10, FOV gives ±8.7x, ±5.4y at z=0).
const VIEW_X: f32 = 9.5;  // slight overscan so glyphs don't pop in at edges
const VIEW_Y: f32 = 6.5;

/// Pulse wave parameters.
const PULSE_MIN_INTERVAL: f32 = 10.0;
const PULSE_MAX_INTERVAL: f32 = 30.0;
const PULSE_WIDTH: f32 = 3.0;       // world units of the bright band
const PULSE_BRIGHTNESS: f32 = 0.15; // additive alpha boost

// ═══════════════════════════════════════════════════════════════════════════════
// DETERMINISTIC HASH
// ═══════════════════════════════════════════════════════════════════════════════

/// Fast deterministic hash for seeding per-glyph parameters.
#[inline]
fn hash(seed: u64) -> u64 {
    let mut h = seed;
    h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h
}

/// Map a hash to [0, 1).
#[inline]
fn hash_f32(seed: u64) -> f32 {
    (hash(seed) & 0x00FF_FFFF) as f32 / 16_777_216.0
}

/// Map a hash to [-1, 1).
#[inline]
fn hash_signed(seed: u64) -> f32 {
    hash_f32(seed) * 2.0 - 1.0
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCREEN TYPE MODIFIERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Screen-reactive modifiers applied to the chaos field.
struct ScreenModifiers {
    speed_mult: f32,
    alpha_mult: f32,
    color_shift: Vec3, // additive RGB shift
    ripple: f32,       // 0-1, combat ripple intensity
}

fn screen_modifiers(state: &GameState) -> ScreenModifiers {
    match state.screen {
        AppScreen::Combat => {
            // Ripple on recent hits (player_flash or enemy_flash > 0)
            let ripple = (state.player_flash + state.enemy_flash).clamp(0.0, 1.0);
            ScreenModifiers {
                speed_mult: 1.2,
                alpha_mult: 1.0,
                color_shift: Vec3::new(ripple * 0.05, 0.0, 0.0),
                ripple,
            }
        }
        AppScreen::Shop | AppScreen::Crafting => {
            // Shrine-like calm
            ScreenModifiers {
                speed_mult: 0.4,
                alpha_mult: 0.7,
                color_shift: Vec3::ZERO,
                ripple: 0.0,
            }
        }
        AppScreen::GameOver => {
            // Field dies down
            ScreenModifiers {
                speed_mult: 0.1,
                alpha_mult: 0.3,
                color_shift: Vec3::new(-0.02, -0.02, -0.02),
                ripple: 0.0,
            }
        }
        AppScreen::Victory => {
            // Triumphant glow
            ScreenModifiers {
                speed_mult: 0.6,
                alpha_mult: 1.2,
                color_shift: Vec3::new(0.02, 0.04, 0.02),
                ripple: 0.0,
            }
        }
        AppScreen::Title => {
            // Dramatic, slightly faster
            ScreenModifiers {
                speed_mult: 0.8,
                alpha_mult: 1.1,
                color_shift: Vec3::ZERO,
                ripple: 0.0,
            }
        }
        _ => ScreenModifiers {
            speed_mult: 1.0,
            alpha_mult: 1.0,
            color_shift: Vec3::ZERO,
            ripple: 0.0,
        },
    }
}

/// Detect if this is a ChaosRift room.
fn is_chaos_rift(state: &GameState) -> bool {
    state.floor.as_ref().map_or(false, |f| {
        use chaos_rpg_core::world::RoomType;
        f.current().room_type == RoomType::ChaosRift
    })
}

/// Detect if this is a Shrine room.
fn is_shrine(state: &GameState) -> bool {
    state.floor.as_ref().map_or(false, |f| {
        use chaos_rpg_core::world::RoomType;
        f.current().room_type == RoomType::Shrine
    })
}

/// Detect the Null boss fight (boss_id == 10 is the Null entity).
fn is_null_fight(state: &GameState) -> bool {
    state.boss_id == Some(10)
}

// ═══════════════════════════════════════════════════════════════════════════════
// MOTION FUNCTIONS — one per ChaosEngine
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute the (x, y) offset for a glyph driven by the given chaos engine.
/// `t` is time in seconds, `seed` is the per-glyph deterministic seed,
/// `base_x`/`base_y` are the glyph's home position.
fn engine_position(
    engine: ChaosEngine,
    t: f32,
    seed: u64,
    base_x: f32,
    base_y: f32,
    intensity: f32,
) -> (f32, f32) {
    match engine {
        ChaosEngine::Linear => {
            // Straight vertical fall, wrap around
            let speed = 0.5 + hash_f32(seed) * 1.5;
            let y = ((base_y + t * speed * intensity) % (VIEW_Y * 2.0 + 2.0)) - VIEW_Y - 1.0;
            let wobble = (t * 0.3 + hash_f32(seed + 1) * 6.28).sin() * 0.15;
            (base_x + wobble, y)
        }
        ChaosEngine::Lorenz => {
            // Chaotic 2D orbits using irrational frequency ratios
            let fx = hash_f32(seed + 10) * 0.3 + 0.1;
            let fy = fx * std::f32::consts::SQRT_2; // irrational ratio
            let ax = 2.0 + hash_f32(seed + 11) * 3.0;
            let ay = 1.5 + hash_f32(seed + 12) * 2.5;
            let phase_x = hash_f32(seed + 13) * 6.28;
            let phase_y = hash_f32(seed + 14) * 6.28;
            let x = base_x + (t * fx * intensity + phase_x).sin() * ax;
            let y = base_y + (t * fy * intensity + phase_y).cos() * ay;
            (x, y)
        }
        ChaosEngine::Mandelbrot => {
            // Radial motion: approach/recede from a center based on escape boundary
            let cx = hash_signed(seed + 20) * 3.0;
            let cy = hash_signed(seed + 21) * 2.0;
            let escape_t = (t * 0.2 * intensity + hash_f32(seed + 22) * 6.28).sin();
            let r = 2.0 + escape_t * 1.5;
            let angle = t * 0.15 * intensity + hash_f32(seed + 23) * 6.28;
            (cx + angle.cos() * r, cy + angle.sin() * r)
        }
        ChaosEngine::Fibonacci => {
            // Golden spiral paths
            let idx = (seed & 0xFF) as f32;
            let angle = GOLDEN_ANGLE * idx + t * 0.2 * intensity;
            let r = (idx * 0.08).sqrt() * 3.0;
            let breathe = 1.0 + (t * 0.1 + idx * 0.01).sin() * 0.2;
            (angle.cos() * r * breathe, angle.sin() * r * breathe)
        }
        ChaosEngine::Collatz => {
            // Triangle wave bounce approximating the 3n+1 pattern
            let freq = 0.4 + hash_f32(seed + 40) * 0.6;
            let phase = hash_f32(seed + 41) * 6.28;
            let tri = ((t * freq * intensity + phase) % 2.0 - 1.0).abs() * 2.0 - 1.0;
            // Occasional "3n+1 spike": sharp upward jump
            let spike_phase = (t * freq * 0.33 * intensity + phase).sin();
            let spike = if spike_phase > 0.85 { (spike_phase - 0.85) * 10.0 } else { 0.0 };
            let y = base_y + tri * 3.0 + spike * 2.0;
            let drift = (t * 0.05 + hash_f32(seed + 42) * 6.28).sin() * 0.5;
            (base_x + drift, ((y + VIEW_Y + 1.0) % (VIEW_Y * 2.0 + 2.0)) - VIEW_Y - 1.0)
        }
        ChaosEngine::Lissajous => {
            // Parametric Lissajous figure
            let a = 2.0 + (seed % 3) as f32;
            let b = 3.0 + (seed % 4) as f32;
            let delta = hash_f32(seed + 50) * std::f32::consts::PI;
            let scale = 2.5 + hash_f32(seed + 51) * 2.0;
            let x = (a * t * 0.2 * intensity + delta).sin() * scale;
            let y = (b * t * 0.2 * intensity).sin() * scale * 0.7;
            (x, y)
        }
        ChaosEngine::Pendulum => {
            // Damped pendulum swing
            let length = 3.0 + hash_f32(seed + 60) * 4.0;
            let freq = (9.81 / length).sqrt() * 0.5;
            let damp = 0.005 + hash_f32(seed + 61) * 0.01;
            let amp = 4.0 * (-damp * t).exp().max(0.3); // don't fully damp out
            let angle = (t * freq * intensity + hash_f32(seed + 62) * 6.28).sin() * amp;
            (base_x + angle, base_y + (1.0 - angle.abs() / 4.0) * 0.5)
        }
        ChaosEngine::Sine => {
            // Sinusoidal weave: vertical fall with horizontal sine
            let speed_y = 0.3 + hash_f32(seed + 70) * 0.8;
            let freq_x = 0.3 + hash_f32(seed + 71) * 0.5;
            let amp_x = 1.0 + hash_f32(seed + 72) * 2.0;
            let phase = hash_f32(seed + 73) * 6.28;
            let y = ((base_y + t * speed_y * intensity) % (VIEW_Y * 2.0 + 2.0)) - VIEW_Y - 1.0;
            let x = base_x + (t * freq_x * intensity + phase).sin() * amp_x;
            (x, y)
        }
        ChaosEngine::Perlin => {
            // Noise-driven drift using layered sine approximation
            let nx = (t * 0.13 * intensity + hash_f32(seed + 80) * 100.0).sin() * 0.7
                   + (t * 0.31 * intensity + hash_f32(seed + 81) * 100.0).sin() * 0.3
                   + (t * 0.71 * intensity + hash_f32(seed + 82) * 100.0).sin() * 0.15;
            let ny = (t * 0.17 * intensity + hash_f32(seed + 83) * 100.0).sin() * 0.7
                   + (t * 0.37 * intensity + hash_f32(seed + 84) * 100.0).sin() * 0.3
                   + (t * 0.79 * intensity + hash_f32(seed + 85) * 100.0).sin() * 0.15;
            (base_x + nx * 3.0, base_y + ny * 2.5)
        }
        ChaosEngine::Orbit => {
            // Elliptical orbit
            let cx = hash_signed(seed + 90) * 2.0;
            let cy = hash_signed(seed + 91) * 1.5;
            let rx = 1.5 + hash_f32(seed + 92) * 3.0;
            let ry = 1.0 + hash_f32(seed + 93) * 2.0;
            let speed = 0.2 + hash_f32(seed + 94) * 0.4;
            let angle = t * speed * intensity + hash_f32(seed + 95) * 6.28;
            (cx + angle.cos() * rx, cy + angle.sin() * ry)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PULSE WAVE
// ═══════════════════════════════════════════════════════════════════════════════

/// Calculate pulse wave brightness addition at a given x position.
/// Returns 0.0 when no pulse is active, up to PULSE_BRIGHTNESS at peak.
fn pulse_brightness(t: f32, x: f32) -> f32 {
    // Determine current pulse cycle
    let cycle_len = PULSE_MIN_INTERVAL + (PULSE_MAX_INTERVAL - PULSE_MIN_INTERVAL) * 0.5;
    let cycle_t = t % cycle_len;

    // Pulse sweeps from left to right over ~2 seconds
    let sweep_duration = 2.5;
    if cycle_t > sweep_duration {
        return 0.0;
    }

    let pulse_x = -VIEW_X + (cycle_t / sweep_duration) * (VIEW_X * 2.0 + PULSE_WIDTH);
    let dist = (x - pulse_x).abs();
    if dist < PULSE_WIDTH {
        PULSE_BRIGHTNESS * (1.0 - dist / PULSE_WIDTH)
    } else {
        0.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NULL BOSS DEATH SEQUENCE
// ═══════════════════════════════════════════════════════════════════════════════

/// During the Null boss fight, glyphs die one by one.
/// Returns (alive_fraction, freeze_factor).
/// alive_fraction: 1.0 = all alive, 0.0 = all dead.
/// freeze_factor: how much to slow surviving glyphs (0 = normal, 1 = frozen).
fn null_death_state(state: &GameState) -> (f32, f32) {
    if !is_null_fight(state) {
        return (1.0, 0.0);
    }
    // Boss turn drives the death sequence: each turn kills more of the field
    let turn = state.boss_turn as f32;
    let max_turns = 20.0; // field fully dead by turn 20
    let alive = (1.0 - turn / max_turns).clamp(0.0, 1.0);
    let freeze = (turn / max_turns * 1.5).clamp(0.0, 1.0);
    (alive, freeze)
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize the chaos field. Called once at engine start.
/// The field is fully immediate-mode (re-spawned every frame), so this is a no-op.
pub fn init(_state: &GameState, _engine: &mut ProofEngine) {
    // Intentionally empty — all rendering happens in update().
}

/// Update and render the chaos field background.
/// Called every frame. Spawns ~150 glyphs across three parallax layers.
pub fn update(state: &GameState, engine: &mut ProofEngine, dt: f32) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 / 60.0; // approximate seconds from frame count
    let floor = state.floor_num;
    let corruption = state.corruption_frac();
    let brightness = theme.chaos_field_brightness;

    // ── Floor-based multipliers ──
    let floor_speed = match floor {
        0..=10  => 1.0_f32,
        11..=25 => 1.3,
        26..=50 => 1.7,
        51..=75 => 2.2,
        76..=99 => 2.8,
        _       => 3.5,
    };

    let floor_density = match floor {
        0..=10  => 1.0_f32,
        11..=25 => 1.1,
        26..=50 => 1.2,
        51..=75 => 1.3,
        _       => 1.4,
    };

    // ── Screen modifiers ──
    let mods = screen_modifiers(state);
    let speed = floor_speed * mods.speed_mult;

    // ── Shrine slowdown override ──
    let speed = if is_shrine(state) { speed * 0.3 } else { speed };

    // ── ChaosRift override: maximum chaos ──
    let speed = if is_chaos_rift(state) { speed * 2.5 } else { speed };
    let rift_boost = if is_chaos_rift(state) { 0.08 } else { 0.0 };

    // ── Null boss death ──
    let (null_alive, null_freeze) = null_death_state(state);
    let speed = speed * (1.0 - null_freeze);

    // ── Corruption color shift: tint toward purple ──
    let corrupt_r = corruption * 0.15;
    let corrupt_b = corruption * 0.25;
    let corrupt_g = corruption * -0.05;

    // ── Compute base colors ──
    let base_color = Vec3::new(
        (theme.muted.x + corrupt_r + mods.color_shift.x).clamp(0.0, 1.0),
        (theme.muted.y + corrupt_g + mods.color_shift.y).clamp(0.0, 1.0),
        (theme.muted.z + corrupt_b + mods.color_shift.z).clamp(0.0, 1.0),
    );

    let accent_color = Vec3::new(
        theme.accent.x,
        theme.accent.y,
        theme.accent.z,
    );

    // ── Combat ripple: radial distortion from center ──
    let ripple_t = mods.ripple;

    // ════════════════════════════════════════════════════════════════════════════
    // FAR LAYER — 30 large math symbols
    // ════════════════════════════════════════════════════════════════════════════

    let far_actual = ((FAR_COUNT as f32 * floor_density * null_alive) as usize).max(1);
    for i in 0..far_actual {
        let seed = hash(i as u64 + 1000);
        let engine_type = ChaosEngine::from_index(i);

        // Home position distributed across the view
        let home_x = hash_signed(seed + 1) * VIEW_X;
        let home_y = hash_signed(seed + 2) * VIEW_Y;

        let (mut x, mut y) = engine_position(engine_type, t, seed, home_x, home_y, speed * 0.5);

        // Combat ripple: push outward from center
        if ripple_t > 0.01 {
            let dist = (x * x + y * y).sqrt().max(0.1);
            let push = ripple_t * 0.8 / dist;
            x += x * push;
            y += y * push;
        }

        // Wrap to visible area
        x = wrap(x, VIEW_X);
        y = wrap(y, VIEW_Y);

        let ch = FAR_CHARS[i % FAR_CHARS.len()];
        let alpha_base = brightness * 0.08;
        let pulse_add = pulse_brightness(t, x);
        let alpha = (alpha_base + pulse_add + rift_boost) * mods.alpha_mult;

        // Corruption glitch: occasionally replace character
        let ch = if corruption > 0.5 && hash_f32(seed + state.frame) < (corruption - 0.5) * 0.1 {
            glitch_char(seed + state.frame)
        } else {
            ch
        };

        // Mix base and accent color
        let accent_blend = 0.15 + corruption * 0.3;
        let color = Vec3::new(
            base_color.x * (1.0 - accent_blend) + accent_color.x * accent_blend,
            base_color.y * (1.0 - accent_blend) + accent_color.y * accent_blend,
            base_color.z * (1.0 - accent_blend) + accent_color.z * accent_blend,
        );

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 2.0),
            scale: Vec2::splat(0.5),
            color: Vec4::new(color.x * alpha, color.y * alpha, color.z * alpha, alpha),
            emission: alpha * 0.15,
            glow_color: Vec3::new(color.x, color.y, color.z),
            glow_radius: 0.3,
            rotation: (t * 0.05 + i as f32 * 0.5).sin() * 0.15,
            entropy: corruption * 0.5,
            temperature: corruption * 0.3,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ════════════════════════════════════════════════════════════════════════════
    // MID LAYER — 50 digits and operators
    // ════════════════════════════════════════════════════════════════════════════

    let mid_actual = ((MID_COUNT as f32 * floor_density * null_alive) as usize).max(1);
    for i in 0..mid_actual {
        let seed = hash(i as u64 + 5000);
        let engine_type = ChaosEngine::from_index(i + 3); // offset to vary engine distribution

        let home_x = hash_signed(seed + 1) * VIEW_X * 1.1;
        let home_y = hash_signed(seed + 2) * VIEW_Y * 1.1;

        let (mut x, mut y) = engine_position(engine_type, t, seed, home_x, home_y, speed * 0.8);

        // Combat ripple
        if ripple_t > 0.01 {
            let dist = (x * x + y * y).sqrt().max(0.1);
            let push = ripple_t * 0.5 / dist;
            x += x * push;
            y += y * push;
        }

        x = wrap(x, VIEW_X * 1.1);
        y = wrap(y, VIEW_Y * 1.1);

        let ch = MID_CHARS[i % MID_CHARS.len()];
        let alpha_base = brightness * 0.05;
        let pulse_add = pulse_brightness(t, x) * 0.7;
        let alpha = (alpha_base + pulse_add + rift_boost * 0.5) * mods.alpha_mult;

        // Corruption glitch
        let ch = if corruption > 0.3 && hash_f32(seed + state.frame) < (corruption - 0.3) * 0.08 {
            glitch_char(seed + state.frame)
        } else {
            ch
        };

        let color = Vec3::new(
            base_color.x * 0.9 + accent_color.x * 0.1,
            base_color.y * 0.9 + accent_color.y * 0.1,
            base_color.z * 0.9 + accent_color.z * 0.1,
        );

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 3.0),
            scale: Vec2::splat(0.3),
            color: Vec4::new(color.x * alpha, color.y * alpha, color.z * alpha, alpha),
            emission: alpha * 0.08,
            rotation: (t * 0.08 + i as f32 * 0.3).cos() * 0.1,
            entropy: corruption * 0.3,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ════════════════════════════════════════════════════════════════════════════
    // NEAR LAYER — 70 tiny debris particles
    // ════════════════════════════════════════════════════════════════════════════

    let near_actual = ((NEAR_COUNT as f32 * floor_density * null_alive) as usize).max(1);
    for i in 0..near_actual {
        let seed = hash(i as u64 + 10000);
        let engine_type = ChaosEngine::from_index(i + 7); // offset again

        let home_x = hash_signed(seed + 1) * VIEW_X * 1.2;
        let home_y = hash_signed(seed + 2) * VIEW_Y * 1.2;

        let (mut x, mut y) = engine_position(engine_type, t, seed, home_x, home_y, speed * 1.4);

        // Combat ripple (subtle on near layer)
        if ripple_t > 0.01 {
            let dist = (x * x + y * y).sqrt().max(0.1);
            let push = ripple_t * 0.3 / dist;
            x += x * push;
            y += y * push;
        }

        x = wrap(x, VIEW_X * 1.2);
        y = wrap(y, VIEW_Y * 1.2);

        let ch = NEAR_CHARS[i % NEAR_CHARS.len()];
        let alpha_base = brightness * 0.03;
        let pulse_add = pulse_brightness(t, x) * 0.4;
        let alpha = (alpha_base + pulse_add + rift_boost * 0.3) * mods.alpha_mult;

        // Deep corruption makes near debris flicker
        let flicker = if corruption > 0.7 {
            let f = hash_f32(seed + state.frame * 3) < (corruption - 0.7) * 0.3;
            if f { 0.0 } else { 1.0 }
        } else {
            1.0
        };

        let final_alpha = alpha * flicker;

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 4.0),
            scale: Vec2::splat(0.15),
            color: Vec4::new(
                base_color.x * final_alpha,
                base_color.y * final_alpha,
                base_color.z * final_alpha,
                final_alpha,
            ),
            emission: final_alpha * 0.03,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ════════════════════════════════════════════════════════════════════════════
    // NULL BOSS: FROZEN GLYPH GHOSTS
    // When the field is partially dead, render "corpse" glyphs — frozen, dim,
    // slowly fading to nothing. These are the glyphs that have "died".
    // ════════════════════════════════════════════════════════════════════════════

    if is_null_fight(state) && null_alive < 0.95 {
        let dead_count = ((1.0 - null_alive) * (FAR_COUNT + MID_COUNT + NEAR_COUNT) as f32) as usize;
        let ghost_alpha = 0.02 * null_alive; // ghosts fade as more die

        for i in 0..dead_count.min(50) {
            let seed = hash(i as u64 + 50000);
            let home_x = hash_signed(seed + 1) * VIEW_X;
            let home_y = hash_signed(seed + 2) * VIEW_Y;

            // Frozen in place — no motion
            let ch = if i < FAR_COUNT {
                FAR_CHARS[i % FAR_CHARS.len()]
            } else if i < FAR_COUNT + MID_COUNT {
                MID_CHARS[i % MID_CHARS.len()]
            } else {
                NEAR_CHARS[i % NEAR_CHARS.len()]
            };

            let gray = ghost_alpha;
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(home_x, home_y, 3.0),
                scale: Vec2::splat(0.2),
                color: Vec4::new(gray, gray, gray, gray),
                emission: 0.0,
                layer: RenderLayer::Background,
                ..Default::default()
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITY
// ═══════════════════════════════════════════════════════════════════════════════

/// Wrap a coordinate to stay within [-limit, limit] with smooth cycling.
#[inline]
fn wrap(val: f32, limit: f32) -> f32 {
    let range = limit * 2.0;
    let shifted = val + limit;
    let wrapped = ((shifted % range) + range) % range;
    wrapped - limit
}

/// Return a random "glitch" character for corruption effects.
#[inline]
fn glitch_char(seed: u64) -> char {
    const GLITCH: &[char] = &[
        '\u{2588}', // █
        '\u{2591}', // ░
        '\u{2592}', // ▒
        '\u{2593}', // ▓
        '\u{25A0}', // ■
        '\u{00BF}', // ¿
        '\u{00D8}', // Ø
        '\u{2260}', // ≠
        '\u{2248}', // ≈
        '\u{221A}', // √
    ];
    GLITCH[(hash(seed) as usize) % GLITCH.len()]
}
