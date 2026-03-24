//! Floor navigation screen — room map with fog of war.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, _engine: &mut ProofEngine, _dt: f32) {
    // TODO: implement room navigation input
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    render_text(engine, &format!("Floor {} — Navigate rooms", state.floor_num),
                -8.0, 6.0, theme.heading, 0.8);
    render_text(engine, "[Arrow keys] Move  [Enter] Enter room  [C] Character  [Esc] Back",
                -16.0, -12.0, theme.dim, 0.3);
}

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color,
            emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
