//! The Chaos Field — the living mathematical background.
//!
//! Runs behind every screen. 2000+ glyphs driven by MathFunction variants,
//! reactive to game state (corruption, floor depth, combat).

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;

/// Character sets for the chaos field layers.
const FAR_CHARS: &[char] = &['∫', '∑', '∏', 'Ω', '∞', '∇', '∂', 'φ', 'π', 'λ', 'ζ', 'Δ'];
const NEAR_CHARS: &[char] = &['·', '·', '·', ',', '.', '`', '\'', '·', '·'];

/// Initialize the chaos field (called once on engine start).
pub fn init(_state: &GameState, _engine: &mut ProofEngine) {
    // The chaos field is rendered dynamically each frame via spawn_glyph,
    // since proof-engine clears transient glyphs. For a persistent field,
    // we would use scene graph nodes. For now, we render in update().
}

/// Update and render the chaos field background.
pub fn update(state: &GameState, engine: &mut ProofEngine, _dt: f32) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;
    let floor = state.floor_num;
    let corruption = state.corruption_frac();
    let brightness = theme.chaos_field_brightness;

    // Floor-based speed multiplier
    let floor_mult = match floor {
        0..=10 => 1.0_f32,
        11..=25 => 1.3,
        26..=50 => 1.7,
        51..=75 => 2.2,
        76..=99 => 2.8,
        _ => 3.5,
    };

    // ── Far layer: large math symbols, slow ──
    let far_count = 40;
    for i in 0..far_count {
        let seed = i as u64 * 6364136223846793005u64 + 1442695040888963407;
        let base_x = (i as f32 / far_count as f32) * 40.0 - 20.0;
        let speed = 0.02 + (seed % 100) as f32 * 0.0006;
        let y_phase = (seed >> 10) as f32 * 0.001;

        let y = ((frame as f32 * speed * 0.3 * floor_mult + y_phase) % 30.0) - 15.0;
        let wobble = (frame as f32 * 0.02 + i as f32 * 0.17).sin() * 0.3;

        let ch = FAR_CHARS[i % FAR_CHARS.len()];
        let alpha = brightness * 0.7;

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(base_x + wobble, y, -5.0),
            color: Vec4::new(
                theme.muted.x * alpha,
                theme.muted.y * alpha,
                theme.muted.z * alpha,
                0.4,
            ),
            emission: alpha * 0.5,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }

    // ── Near layer: tiny debris, faster ──
    let near_count = 80;
    for i in 0..near_count {
        let seed = (i as u64 + 5000) * 6364136223846793005u64 + 1442695040888963407;
        let base_x = (i as f32 / near_count as f32) * 44.0 - 22.0;
        let speed = 0.02 + (seed % 100) as f32 * 0.0006;
        let y_phase = (seed >> 10) as f32 * 0.001;

        let y = ((frame as f32 * speed * 1.55 * floor_mult + y_phase) % 34.0) - 17.0;

        let ch = NEAR_CHARS[i % NEAR_CHARS.len()];
        let alpha = brightness * 0.55;

        // Corruption tint
        let tint = if corruption > 0.25 {
            let t = ((corruption - 0.25) * 1.33).clamp(0.0, 1.0);
            Vec4::new(
                theme.muted.x * alpha + theme.accent.x * alpha * t * 0.4,
                theme.muted.y * alpha * (1.0 - t * 0.2),
                theme.muted.z * alpha + theme.accent.z * alpha * t * 0.4,
                0.3,
            )
        } else {
            Vec4::new(
                theme.muted.x * alpha,
                theme.muted.y * alpha,
                theme.muted.z * alpha,
                0.3,
            )
        };

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(base_x, y, -3.0),
            color: tint,
            emission: alpha * 0.3,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }
}
