//! Mode selection screen — Story / Infinite / Daily.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameMode, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let esc = engine.input.just_pressed(Key::Escape);
    let num1 = engine.input.just_pressed(Key::Num1);
    let num2 = engine.input.just_pressed(Key::Num2);
    let num3 = engine.input.just_pressed(Key::Num3);

    if up && state.mode_cursor > 0 { state.mode_cursor -= 1; }
    if down && state.mode_cursor < 2 { state.mode_cursor += 1; }
    if num1 { state.mode_cursor = 0; }
    if num2 { state.mode_cursor = 1; }
    if num3 { state.mode_cursor = 2; }

    if enter || num1 || num2 || num3 {
        state.game_mode = match state.mode_cursor {
            0 => { state.max_floor = 10; GameMode::Story }
            1 => { state.max_floor = u32::MAX; GameMode::Infinite }
            _ => { state.max_floor = u32::MAX; GameMode::Daily }
        };
        state.screen = AppScreen::CharacterCreation;
    }
    if esc { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    ui_render::heading_centered(engine, "SELECT GAME MODE", 4.5, theme.heading);

    let modes = [
        ("[1] Story Mode", "10 floors. Beat the final boss."),
        ("[2] Infinite Mode", "No end. How deep can you go?"),
        ("[3] Daily Challenge", "Same seed for everyone today."),
    ];

    for (i, (name, desc)) in modes.iter().enumerate() {
        let selected = i == state.mode_cursor;
        let y = 2.5 - i as f32 * 2.0;
        let color = if selected { theme.selected } else { theme.primary };
        let prefix = if selected { "> " } else { "  " };
        ui_render::text(engine, &format!("{}{}", prefix, name), -4.5, y, color, 0.45, if selected { 0.8 } else { 0.4 });
        ui_render::small(engine, desc, -3.5, y - 0.5, theme.dim);
    }

    ui_render::small(engine, "Enter/Space to select | Esc to go back", -5.0, -4.5, theme.muted);
}
