//! Boon selection screen — choose 1 of 3 starting boons.

use proof_engine::prelude::*;
use chaos_rpg_core::character::Boon;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter);
    let key1 = engine.input.just_pressed(Key::Num1);
    let key2 = engine.input.just_pressed(Key::Num2);
    let key3 = engine.input.just_pressed(Key::Num3);
    let esc = engine.input.just_pressed(Key::Escape);

    if up && state.boon_cursor > 0 { state.boon_cursor -= 1; }
    if down && state.boon_cursor < 2 { state.boon_cursor += 1; }
    if key1 { state.boon_cursor = 0; }
    if key2 { state.boon_cursor = 1; }
    if key3 { state.boon_cursor = 2; }

    if enter {
        if let Some(ref mut player) = state.player {
            let boon = state.boon_options[state.boon_cursor];
            player.apply_boon(boon);
            // Initialize mana
            state.current_mana = state.max_mana();
            // Generate first floor
            let floor = chaos_rpg_core::world::generate_floor(state.floor_num, state.floor_seed);
            state.floor = Some(floor);
            state.screen = AppScreen::FloorNav;
        }
    }
    if esc { state.screen = AppScreen::CharacterCreation; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    render_text(engine, "CHOOSE YOUR BOON", -5.5, 8.0, theme.heading, 0.9);
    render_text(engine, "One gift to carry into The Proof.", -9.0, 6.5, theme.dim, 0.35);

    for (i, boon) in state.boon_options.iter().enumerate() {
        let selected = i == state.boon_cursor;
        let y = 3.5 - i as f32 * 3.5;
        let color = if selected { theme.selected } else { theme.primary };
        let prefix = if selected { "> " } else { "  " };

        render_text(engine, &format!("{}[{}] {:?}", prefix, i + 1, boon),
                    -10.0, y, color, if selected { 0.9 } else { 0.4 });
    }

    render_text(engine, "[Enter] Choose  [1-3] Select  [Esc] Back", -12.0, -10.0, theme.muted, 0.25);
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
