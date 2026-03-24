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

    // ── Dungeon minimap overlay (from proof-engine DungeonBridge) ──
    render_dungeon_minimap(state, engine, theme);
}

/// Render the proof-engine dungeon minimap in the top-right corner of the room view.
fn render_dungeon_minimap(
    state: &GameState,
    engine: &mut ProofEngine,
    theme: &crate::theme::Theme,
) {
    use crate::dungeon_bridge::TileVisibility;

    let bridge = match &state.dungeon_bridge {
        Some(b) => b,
        None => return,
    };

    if let Some(minimap) = bridge.get_minimap_data() {
        let offset_x = 12.0_f32;
        let offset_y = 8.0_f32;
        let scale = 0.35_f32;

        for entry in &minimap.entries {
            let px = offset_x + entry.x as f32 * scale;
            let py = offset_y - entry.y as f32 * scale;
            let (r, g, b) = entry.color;
            let color = Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 0.8);
            engine.spawn_glyph(Glyph {
                character: entry.ch,
                position: Vec3::new(px, py, 0.0),
                color,
                emission: if entry.ch == '@' { 0.8 } else { 0.2 },
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // Biome label
        let biome = bridge.biome();
        let floor_label = format!("F{} - {}", bridge.floor_number(), biome);
        render_text(engine, &floor_label, offset_x - 1.0, offset_y + 1.0, theme.muted, 0.3);
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
