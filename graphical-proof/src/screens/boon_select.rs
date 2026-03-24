//! Boon selection screen — choose 1 of 3 starting boons.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let num1 = engine.input.just_pressed(Key::Num1);
    let num2 = engine.input.just_pressed(Key::Num2);
    let num3 = engine.input.just_pressed(Key::Num3);
    let esc = engine.input.just_pressed(Key::Escape);

    if up && state.boon_cursor > 0 { state.boon_cursor -= 1; }
    if down && state.boon_cursor < 2 { state.boon_cursor += 1; }
    if num1 { state.boon_cursor = 0; }
    if num2 { state.boon_cursor = 1; }
    if num3 { state.boon_cursor = 2; }

    if enter || num1 || num2 || num3 {
        if let Some(ref mut player) = state.player {
            let boon = state.boon_options[state.boon_cursor];
            player.apply_boon(boon);
            state.current_mana = state.max_mana();
            let floor = chaos_rpg_core::world::generate_floor(state.floor_num, state.floor_seed);
            state.floor = Some(floor);
            state.screen = AppScreen::FloorNav;
        }
    }
    if esc { state.screen = AppScreen::CharacterCreation; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    ui_render::heading_centered(engine, "CHOOSE YOUR BOON", 4.5, theme.heading);
    ui_render::text_centered(engine, "One gift to carry into The Proof.", 3.5, theme.dim, 0.3, 0.3);

    for (i, boon) in state.boon_options.iter().enumerate() {
        let sel = i == state.boon_cursor;
        let y = 1.5 - i as f32 * 1.8;
        let color = if sel { theme.selected } else { theme.primary };
        let prefix = if sel { "> " } else { "  " };
        ui_render::text(engine, &format!("{}[{}] {:?}", prefix, i + 1, boon), -5.0, y, color, 0.4, if sel { 0.8 } else { 0.35 });
    }

    ui_render::small(engine, "Enter/Space to choose | 1-3 direct | Esc back", -6.0, -4.5, theme.muted);
}
