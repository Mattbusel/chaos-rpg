//! Room view — event resolution (shrine, trap, treasure, portal, etc).

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let esc = engine.input.just_pressed(Key::Escape);
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
    if (enter || esc) && state.room_event.resolved {
        state.screen = AppScreen::FloorNav;
    } else if enter || esc {
        state.room_event.resolved = true;
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Event title
    if !state.room_event.title.is_empty() {
        ui_render::heading_centered(engine, &state.room_event.title, 4.5, theme.heading);
    }

    // Event lines
    for (i, line) in state.room_event.lines.iter().enumerate() {
        let truncated: String = line.chars().take(48).collect();
        ui_render::small(engine, &truncated, -7.5, 3.0 - i as f32 * 0.42, theme.primary);
    }

    // Stat deltas
    let mut info_y = 3.0 - state.room_event.lines.len() as f32 * 0.42 - 0.5;
    if state.room_event.gold_delta != 0 {
        let sign = if state.room_event.gold_delta > 0 { "+" } else { "" };
        let c = if state.room_event.gold_delta > 0 { theme.gold } else { theme.danger };
        ui_render::small(engine, &format!("{}{}  gold", sign, state.room_event.gold_delta), -7.5, info_y, c);
        info_y -= 0.38;
    }
    if state.room_event.hp_delta != 0 {
        let sign = if state.room_event.hp_delta > 0 { "+" } else { "" };
        let c = if state.room_event.hp_delta > 0 { theme.success } else { theme.danger };
        ui_render::small(engine, &format!("{}{}  HP", sign, state.room_event.hp_delta), -7.5, info_y, c);
        info_y -= 0.38;
    }
    if state.room_event.damage_taken != 0 {
        ui_render::small(engine, &format!("-{} damage taken", state.room_event.damage_taken), -7.5, info_y, theme.danger);
        info_y -= 0.38;
    }
    for (stat, val) in &state.room_event.stat_bonuses {
        let sign = if *val > 0 { "+" } else { "" };
        ui_render::small(engine, &format!("{}{} {}", sign, val, stat), -7.5, info_y, theme.accent);
        info_y -= 0.38;
    }

    // Pending pickups
    if let Some(ref item) = state.room_event.pending_item {
        ui_render::body(engine, &format!("Item: {}", item.name), -7.5, info_y - 0.2, theme.gold);
    }
    if let Some(ref spell) = state.room_event.pending_spell {
        ui_render::body(engine, &format!("Spell: {}", spell.name), -7.5, info_y - 0.6, theme.mana);
    }

    // Control hints
    let mut hints = Vec::new();
    if state.room_event.pending_item.is_some() { hints.push("[P] Pick up"); }
    if state.room_event.pending_spell.is_some() { hints.push("[L] Learn spell"); }
    if state.room_event.portal_available { hints.push("[P] Enter portal"); }
    hints.push("[Enter/Space] Continue");

    ui_render::small(engine, &hints.join("  "), -7.5, -5.0, theme.muted);

    // Dungeon minimap overlay
    render_dungeon_minimap(state, engine, theme);
}

/// Render the proof-engine dungeon minimap in the top-right corner.
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
        let offset_x = 5.5_f32;
        let offset_y = 4.5_f32;
        let scale = 0.3_f32;

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

        let biome = bridge.biome();
        let floor_label = format!("F{} - {}", bridge.floor_number(), biome);
        ui_render::small(engine, &floor_label, offset_x - 0.5, offset_y + 0.8, theme.muted);
    }
}
