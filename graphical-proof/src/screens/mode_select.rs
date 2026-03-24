//! Mode selection screen — Story / Infinite / Daily.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameMode, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter);
    let esc = engine.input.just_pressed(Key::Escape);

    if up && state.mode_cursor > 0 { state.mode_cursor -= 1; }
    if down && state.mode_cursor < 2 { state.mode_cursor += 1; }

    if enter {
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

    render_text(engine, "SELECT GAME MODE", -5.0, 7.0, theme.heading, 0.9);

    let modes = [
        ("Story Mode", "10 floors. Reach the end. Beat the final boss."),
        ("Infinite Mode", "No end. Escalating chaos. How deep can you go?"),
        ("Daily Challenge", "Seeded run. Same seed for everyone today."),
    ];

    for (i, (name, desc)) in modes.iter().enumerate() {
        let selected = i == state.mode_cursor;
        let y = 3.0 - i as f32 * 3.0;
        let color = if selected { theme.selected } else { theme.primary };
        let prefix = if selected { "> " } else { "  " };

        render_text(engine, &format!("{}{}", prefix, name), -8.0, y, color, if selected { 1.0 } else { 0.5 });
        render_text(engine, desc, -6.0, y - 1.0, theme.dim, 0.3);
    }

    render_text(engine, "[Enter] Select  [Esc] Back", -8.0, -10.0, theme.muted, 0.25);
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
