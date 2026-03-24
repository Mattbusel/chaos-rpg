//! Crafting bench — select item, then select operation.
//!
//! 9 operations: Reforge, Augment, Annul, Corrupt, Fuse, EngineLock, Shatter, Imbue, Repair

use proof_engine::prelude::*;
use crate::state::{AppScreen, CraftPhase, GameState};
use crate::theme::THEMES;

const OPS: &[(&str, &str)] = &[
    ("Reforge",    "Chaos-roll all modifiers anew"),
    ("Augment",    "Add one new random modifier"),
    ("Annul",      "Remove one random modifier"),
    ("Corrupt",    "Risk-tiered chaos outcome"),
    ("Fuse",       "Double value, upgrade rarity"),
    ("EngineLock", "Embed chaos engine signature"),
    ("Shatter",    "Destroy item, scatter mods"),
    ("Imbue",      "Add 3 charges (max 9)"),
    ("Repair",     "Restore durability"),
];

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let enter = engine.input.just_pressed(Key::Enter);
    let esc = engine.input.just_pressed(Key::Escape);

    match state.craft_phase {
        CraftPhase::SelectItem => {
            let item_count = state.player.as_ref().map(|p| p.inventory.len()).unwrap_or(0);
            if item_count == 0 {
                if esc || enter { state.screen = AppScreen::FloorNav; }
                return;
            }
            if up && state.craft_item_cursor > 0 { state.craft_item_cursor -= 1; }
            if down && state.craft_item_cursor < item_count.saturating_sub(1) { state.craft_item_cursor += 1; }
            if enter { state.craft_phase = CraftPhase::SelectOp; state.craft_op_cursor = 0; }
            if esc { state.screen = AppScreen::FloorNav; }
        }
        CraftPhase::SelectOp => {
            if up && state.craft_op_cursor > 0 { state.craft_op_cursor -= 1; }
            if down && state.craft_op_cursor < OPS.len() - 1 { state.craft_op_cursor += 1; }
            if enter {
                // Execute craft operation
                state.craft_message = format!("Applied {} to item.", OPS[state.craft_op_cursor].0);
                state.craft_anim_timer = 1.0;
                state.craft_anim_type = (state.craft_op_cursor + 1) as u8;
                state.craft_phase = CraftPhase::SelectItem;
            }
            if esc { state.craft_phase = CraftPhase::SelectItem; }
        }
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    render_text(engine, "CRAFTING BENCH", -4.5, 9.0, theme.heading, 0.9);

    match state.craft_phase {
        CraftPhase::SelectItem => {
            render_text(engine, "Select an item to modify:", -10.0, 7.0, theme.dim, 0.4);

            if let Some(ref player) = state.player {
                if player.inventory.is_empty() {
                    render_text(engine, "Your inventory is empty.", -8.0, 4.0, theme.muted, 0.3);
                } else {
                    let start = state.craft_item_cursor.saturating_sub(8);
                    let end = (start + 16).min(player.inventory.len());
                    for (display_i, idx) in (start..end).enumerate() {
                        let item = &player.inventory[idx];
                        let selected = idx == state.craft_item_cursor;
                        let color = if selected { theme.selected } else { theme.primary };
                        let prefix = if selected { "> " } else { "  " };
                        let line = format!("{}{} ({:?})", prefix, item.name, item.rarity);
                        let truncated: String = line.chars().take(50).collect();
                        render_text(engine, &truncated, -16.0, 5.0 - display_i as f32 * 1.0, color,
                            if selected { 0.7 } else { 0.35 });
                    }
                }

                // Show selected item detail on right side
                if let Some(item) = player.inventory.get(state.craft_item_cursor) {
                    render_text(engine, &item.name, 4.0, 7.0, theme.heading, 0.7);
                    render_text(engine, &format!("Rarity: {:?}", item.rarity), 4.0, 5.5, theme.accent, 0.5);
                    render_text(engine, &format!("Value: {}", item.value), 4.0, 4.5, theme.gold, 0.4);
                    render_text(engine, &format!("Durability: {}/{}", item.durability, item.max_durability),
                        4.0, 3.5, theme.primary, 0.4);
                    for (mi, m) in item.stat_modifiers.iter().enumerate().take(6) {
                        let mod_text = format!("{}: {:+}", m.stat, m.value);
                        render_text(engine, &mod_text, 4.0, 2.0 - mi as f32 * 0.9, theme.dim, 0.35);
                    }
                }
            }

            render_text(engine, "[Up/Down] Select  [Enter] Choose  [Esc] Leave", -14.0, -12.0, theme.muted, 0.25);
        }

        CraftPhase::SelectOp => {
            if let Some(ref player) = state.player {
                if let Some(item) = player.inventory.get(state.craft_item_cursor) {
                    render_text(engine, &format!("Crafting: {}", item.name), -10.0, 7.0, theme.heading, 0.7);
                }
            }

            render_text(engine, "Select operation:", -10.0, 5.5, theme.dim, 0.4);

            for (i, (name, desc)) in OPS.iter().enumerate() {
                let selected = i == state.craft_op_cursor;
                let color = if selected { theme.selected } else { theme.primary };
                let prefix = if selected { "> " } else { "  " };
                let line = format!("{}[{}] {} — {}", prefix, i + 1, name, desc);
                let truncated: String = line.chars().take(60).collect();
                render_text(engine, &truncated, -16.0, 3.5 - i as f32 * 1.1, color,
                    if selected { 0.6 } else { 0.3 });
            }

            render_text(engine, "[Up/Down] Select  [Enter] Apply  [Esc] Back", -14.0, -12.0, theme.muted, 0.25);
        }
    }

    // Craft result message
    if !state.craft_message.is_empty() && state.craft_anim_timer > 0.0 {
        let fade = state.craft_anim_timer.min(1.0);
        render_text(engine, &state.craft_message, -12.0, -9.0,
            Vec4::new(theme.success.x * fade, theme.success.y * fade, theme.success.z * fade, fade), 0.5 * fade);
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
