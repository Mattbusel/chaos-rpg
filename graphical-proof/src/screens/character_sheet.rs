//! Character sheet — 5 tabs: Stats, Inventory, Body, Effects, Lore.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;

pub fn update(state: &mut GameState, _engine: &mut ProofEngine, _dt: f32) {
    // TODO: implement tab switching and character sheet interaction
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let tab_names = ["Stats", "Inventory", "Body", "Effects", "Lore"];
    let tab = state.char_tab as usize;

    // Tab bar
    let mut x = -12.0;
    for (i, name) in tab_names.iter().enumerate() {
        let color = if i == tab { theme.selected } else { theme.dim };
        let emission = if i == tab { 0.8 } else { 0.3 };
        render_text(engine, name, x, 8.0, color, emission);
        x += name.len() as f32 * 0.45 + 1.5;
    }

    // Current tab header
    render_text(engine, &format!("Character Sheet — {}", tab_names[tab]),
                -8.0, 6.0, theme.heading, 0.7);
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
