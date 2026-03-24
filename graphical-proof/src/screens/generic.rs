//! Generic screen renderer for screens not yet fully ported.
//! Provides a consistent look with title, description, and Esc-to-back.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

/// Render a placeholder screen with title text and back hint.
pub fn render_placeholder(state: &GameState, engine: &mut ProofEngine, title: &str, desc: &str) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    render_text(engine, title, -(title.len() as f32 * 0.225), 6.0, theme.heading, 0.8);
    render_text(engine, desc, -(desc.len() as f32 * 0.225), 4.0, theme.dim, 0.35);
    render_text(engine, "[Esc] Back", -3.0, -12.0, theme.muted, 0.25);
}

/// Handle Esc → go back to a target screen.
pub fn handle_back(state: &mut GameState, engine: &mut ProofEngine, target: AppScreen) {
    if engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q) {
        state.screen = target;
    }
}

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color, emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
