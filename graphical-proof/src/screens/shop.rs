//! Shop screen — item cards, gold display, buy/sell, item preview, merchant NPC.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let esc = engine.input.just_pressed(Key::Escape);
    let h_key = engine.input.just_pressed(Key::H);
    let num1 = engine.input.just_pressed(Key::Num1);
    let num2 = engine.input.just_pressed(Key::Num2);
    let num3 = engine.input.just_pressed(Key::Num3);
    let num4 = engine.input.just_pressed(Key::Num4);

    let item_count = state.shop_items.len();
    let total_slots = item_count + 1;

    if up && state.shop_cursor > 0 { state.shop_cursor -= 1; }
    if down && state.shop_cursor < total_slots.saturating_sub(1) { state.shop_cursor += 1; }

    if num1 && item_count > 0 { state.shop_cursor = 0; }
    if num2 && item_count > 1 { state.shop_cursor = 1; }
    if num3 && item_count > 2 { state.shop_cursor = 2; }
    if num4 && item_count > 3 { state.shop_cursor = 3; }

    if enter || h_key {
        if state.shop_cursor < item_count {
            if let Some(ref mut player) = state.player {
                let (_item, price) = &state.shop_items[state.shop_cursor];
                if player.gold >= *price {
                    let (bought, cost) = state.shop_items.remove(state.shop_cursor);
                    player.gold -= cost;
                    player.inventory.push(bought);
                    if state.shop_cursor >= state.shop_items.len() && state.shop_cursor > 0 {
                        state.shop_cursor -= 1;
                    }
                }
            }
        } else if h_key || state.shop_cursor == item_count {
            if let Some(ref mut player) = state.player {
                if player.gold >= state.shop_heal_cost && player.current_hp < player.max_hp {
                    player.gold -= state.shop_heal_cost;
                    let heal = player.max_hp / 3;
                    player.current_hp = (player.current_hp + heal).min(player.max_hp);
                }
            }
        }
    }

    if esc { state.screen = AppScreen::FloorNav; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    // Header
    ui_render::heading_centered(engine, "THE ARCHIVIST'S SHOP", 4.8, theme.heading);

    // Animated merchant
    let bob = ((frame as f32 * 0.06).sin() * 0.15).abs();
    let mx = 4.5;
    let my = 3.2 + bob;
    let merchant = ["  /\\  ", " /  \\ ", " |{}| ", " |  | ", " /  \\ "];
    for (i, line) in merchant.iter().enumerate() {
        let pulse = ((frame as f32 * 0.04 + i as f32 * 0.3).sin() * 0.15 + 0.85).max(0.0);
        let c = Vec4::new(theme.accent.x * pulse, theme.accent.y * pulse, theme.accent.z * pulse, 1.0);
        ui_render::small(engine, line, mx, my - i as f32 * 0.35, c);
    }

    // Gold + HP
    if let Some(ref player) = state.player {
        ui_render::body(engine, &format!("Gold: {}", player.gold), -8.0, 3.8, theme.gold);
        let hp_pct = player.current_hp as f32 / player.max_hp.max(1) as f32;
        ui_render::small(engine, "HP", -8.0, 3.2, theme.muted);
        ui_render::bar(engine, -7.0, 3.2, 3.5, hp_pct, theme.hp_color(hp_pct), theme.muted, 0.25);
        ui_render::small(engine, &format!("{}/{}", player.current_hp, player.max_hp), -3.0, 3.2, theme.hp_color(hp_pct));
    }

    // Separator
    ui_render::text_centered(engine, "--------------------------------", 2.5, theme.border, 0.25, 0.15);

    // Item list
    let list_x = -8.2;
    let mut y = 1.8;

    if state.shop_items.is_empty() {
        ui_render::body(engine, "Sold out!", list_x, y, theme.dim);
    } else {
        for (i, (item, price)) in state.shop_items.iter().enumerate() {
            let selected = i == state.shop_cursor;
            let color = if selected { theme.selected } else { theme.primary };
            let em = if selected { 0.8 } else { 0.4 };
            let prefix = if selected { "> " } else { "  " };
            let rarity_tag: String = format!("{:?}", item.rarity).chars().take(6).collect();
            let line = format!("{}[{}] {} ({}) {}g", prefix, i + 1, item.name, rarity_tag, price);
            let truncated: String = line.chars().take(38).collect();
            ui_render::text(engine, &truncated, list_x, y, color, 0.3, em);
            y -= 0.55;
        }
    }

    // Heal option
    let heal_idx = state.shop_items.len();
    let heal_selected = state.shop_cursor == heal_idx;
    let heal_color = if heal_selected { theme.selected } else { theme.success };
    let heal_em = if heal_selected { 0.8 } else { 0.4 };
    let heal_prefix = if heal_selected { "> " } else { "  " };
    ui_render::text(engine, &format!("{}[H] Heal - {}g", heal_prefix, state.shop_heal_cost), list_x, y, heal_color, 0.3, heal_em);

    // Item preview (right panel)
    if state.shop_cursor < state.shop_items.len() {
        let (ref item, price) = state.shop_items[state.shop_cursor];
        let px = 2.0;
        let mut py = 1.8;

        let rarity_color = match format!("{:?}", item.rarity).as_str() {
            "Legendary" => theme.gold,
            "Rare" => theme.accent,
            "Uncommon" => theme.success,
            _ => theme.primary,
        };
        ui_render::text(engine, &item.name, px, py, rarity_color, 0.35, 0.7);
        py -= 0.5;
        ui_render::small(engine, &format!("Rarity: {:?}", item.rarity), px, py, theme.accent);
        py -= 0.4;
        ui_render::small(engine, &format!("Price: {} gold", price), px, py, theme.gold);
        py -= 0.4;
        ui_render::small(engine, &format!("Dur: {}/{}", item.durability, item.max_durability), px, py, theme.primary);
        py -= 0.55;

        if !item.stat_modifiers.is_empty() {
            ui_render::small(engine, "-- Modifiers --", px, py, theme.accent);
            py -= 0.4;
            for m in item.stat_modifiers.iter().take(5) {
                let sign = if m.value >= 0 { "+" } else { "" };
                let c = if m.value >= 0 { theme.success } else { theme.danger };
                ui_render::small(engine, &format!("{}{} {}", sign, m.value, m.stat), px, py, c);
                py -= 0.35;
            }
        }

        if let Some(ref player) = state.player {
            if player.gold < price {
                let pulse = ((frame as f32 * 0.1).sin() * 0.3 + 0.7).max(0.0);
                ui_render::small(engine, "Not enough gold!",  px, py - 0.3,
                    Vec4::new(theme.danger.x * pulse, theme.danger.y * pulse, theme.danger.z * pulse, 1.0));
            }
        }
    }

    // Footer
    ui_render::small(engine, "[Up/Down] Select  [Enter/Space] Buy  [H] Heal  [Esc] Leave", -8.0, -5.0, theme.muted);
}
