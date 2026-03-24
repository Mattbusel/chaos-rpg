//! Room view — event resolution (shrine, trap, treasure, portal, etc).

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape);
    let p_key = engine.input.just_pressed(Key::P);
    let l_key = engine.input.just_pressed(Key::L);

    // Pick up item
    if p_key {
        if let Some(item) = state.room_event.pending_item.take() {
            if let Some(ref mut player) = state.player {
                player.inventory.push(item);
            }
        }
        if state.room_event.portal_available {
            // Skip floors via portal
            state.floor_num += 3;
            state.room_event.resolved = true;
        }
    }
    // Learn spell
    if l_key {
        if let Some(spell) = state.room_event.pending_spell.take() {
            if let Some(ref mut player) = state.player {
                player.known_spells.push(spell);
            }
        }
    }
    // Continue
    if enter && state.room_event.resolved {
        state.screen = AppScreen::FloorNav;
    } else if enter {
        state.room_event.resolved = true;
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    if !state.room_event.title.is_empty() {
        render_text(engine, &state.room_event.title, -8.0, 7.0, theme.heading, 0.8);
    }

    for (i, line) in state.room_event.lines.iter().enumerate() {
        let truncated: String = line.chars().take(70).collect();
        render_text(engine, &truncated, -16.0, 4.0 - i as f32 * 1.0, theme.primary, 0.4);
    }

    let mut hints = Vec::new();
    if state.room_event.pending_item.is_some() { hints.push("[P] Pick up"); }
    if state.room_event.pending_spell.is_some() { hints.push("[L] Learn spell"); }
    if state.room_event.portal_available { hints.push("[P] Enter portal"); }
    hints.push("[Enter] Continue");

    render_text(engine, &hints.join("  "), -12.0, -10.0, theme.muted, 0.25);
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
