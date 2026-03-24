//! Shop screen — buy items and healing from the Archivist NPC.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Enter);
    if esc { state.screen = AppScreen::FloorNav; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    render_text(engine, "THE ARCHIVIST'S SHOP", -6.0, 8.0, theme.heading, 0.8);

    if let Some(ref player) = state.player {
        let gold_text = format!("Your Gold: {}", player.gold);
        render_text(engine, &gold_text, -8.0, 6.0, theme.gold, 0.6);
    }

    for (i, (item, price)) in state.shop_items.iter().enumerate() {
        let y = 3.0 - i as f32 * 1.5;
        let selected = i == state.shop_cursor;
        let color = if selected { theme.selected } else { theme.primary };
        let text = format!("[{}] {} — {} gold ({:?})", i + 1, item.name, price, item.rarity);
        render_text(engine, &text, -14.0, y, color, if selected { 0.7 } else { 0.4 });
    }

    render_text(engine, "[H] Heal  [1-4] Buy  [Enter/Esc] Leave", -12.0, -10.0, theme.muted, 0.25);
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
