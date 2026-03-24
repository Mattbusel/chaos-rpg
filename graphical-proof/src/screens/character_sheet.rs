//! Character sheet screen — 5 tabs: Stats, Inventory, Effects, Lore, Log.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

const TAB_NAMES: [&str; 5] = ["Stats", "Inventory", "Effects", "Lore", "Log"];

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let pressed_1 = engine.input.just_pressed(Key::Num1);
    let pressed_2 = engine.input.just_pressed(Key::Num2);
    let pressed_3 = engine.input.just_pressed(Key::Num3);
    let pressed_4 = engine.input.just_pressed(Key::Num4);
    let pressed_5 = engine.input.just_pressed(Key::Num5);
    let pressed_left = engine.input.just_pressed(Key::Left);
    let pressed_right = engine.input.just_pressed(Key::Right);
    let pressed_tab = engine.input.just_pressed(Key::Tab);
    let pressed_esc = engine.input.just_pressed(Key::Escape);
    let pressed_c = engine.input.just_pressed(Key::C);
    let pressed_enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);

    if pressed_1 { state.char_tab = 0; }
    if pressed_2 { state.char_tab = 1; }
    if pressed_3 { state.char_tab = 2; }
    if pressed_4 { state.char_tab = 3; }
    if pressed_5 { state.char_tab = 4; }

    if pressed_right || pressed_tab {
        state.char_tab = (state.char_tab + 1) % 5;
    }
    if pressed_left {
        state.char_tab = (state.char_tab + 4) % 5;
    }

    if pressed_esc || pressed_c || pressed_enter {
        state.screen = AppScreen::FloorNav;
    }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let tab = state.char_tab as usize;

    // Tab bar
    let tab_labels: Vec<String> = TAB_NAMES.iter().enumerate()
        .map(|(i, n)| format!("[{}] {}", i + 1, n))
        .collect();
    let total_w: f32 = tab_labels.iter().map(|l| l.len() as f32 * 0.255 + 0.6).sum();
    let mut tx = -(total_w * 0.5);
    for (i, label) in tab_labels.iter().enumerate() {
        let color = if i == tab { theme.selected } else { theme.dim };
        ui_render::text(engine, label, tx, 5.0, color, 0.3, if i == tab { 0.8 } else { 0.3 });
        tx += label.len() as f32 * 0.255 + 0.6;
    }

    // Title
    ui_render::heading_centered(engine, &format!("Character - {}", TAB_NAMES[tab]), 4.2, theme.heading);

    let player = match state.player.as_ref() {
        Some(p) => p,
        None => {
            ui_render::body(engine, "No character data.", -4.0, 1.0, theme.danger);
            return;
        }
    };

    match tab {
        0 => render_stats(state, engine, player, theme),
        1 => render_inventory(engine, player, theme),
        2 => render_effects(engine, player, theme),
        3 => render_lore(engine, player, theme),
        4 => render_log(state, engine, theme),
        _ => {}
    }

    // Footer
    ui_render::small(engine, "[Esc/C/Enter] Back  [1-5] Tabs  [Left/Right] Cycle", -7.5, -5.2, theme.muted);
}

// ── Tab 1: Stats ────────────────────────────────────────────────────────────

fn render_stats(
    state: &GameState,
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -8.0;
    let mut y = 3.2;

    // Core info
    let info = format!(
        "Lv {} | XP {} | Gold {} | Kills {}",
        player.level, player.xp, player.gold, player.kills
    );
    ui_render::body(engine, &info, x, y, theme.primary);
    y -= 0.55;

    let floor_info = format!(
        "Floor {} | Rooms {} | Corruption {}",
        player.floor, player.rooms_cleared, player.corruption
    );
    ui_render::small(engine, &floor_info, x, y, theme.dim);
    y -= 0.7;

    // Stats with bars
    let stats: [(&str, i64); 7] = [
        ("VIT", player.stats.vitality),
        ("FOR", player.stats.force),
        ("MAN", player.stats.mana),
        ("CUN", player.stats.cunning),
        ("PRE", player.stats.precision),
        ("ENT", player.stats.entropy),
        ("LCK", player.stats.luck),
    ];

    ui_render::small(engine, "-- Primary Stats --", x, y, theme.accent);
    y -= 0.45;

    for (name, val) in &stats {
        let label = format!("{}: {:>4}", name, val);
        ui_render::small(engine, &label, x, y, theme.primary);
        let ratio = (*val as f32 / 50.0).clamp(0.0, 1.0);
        ui_render::bar(engine, x + 3.5, y, 4.0, ratio, theme.accent, theme.dim, 0.25);
        y -= 0.42;
    }

    y -= 0.3;

    // HP / Max HP
    let hp_ratio = if player.max_hp > 0 { player.current_hp as f32 / player.max_hp as f32 } else { 0.0 };
    ui_render::small(engine, &format!("HP: {}/{}", player.current_hp, player.max_hp), x, y, theme.hp_color(hp_ratio));
    ui_render::bar(engine, x + 4.0, y, 4.5, hp_ratio, theme.hp_color(hp_ratio), theme.dim, 0.25);
    y -= 0.5;

    // Mana
    let max_mp = (player.stats.mana + 50).max(50);
    ui_render::small(engine, &format!("MP: {}/{}", state.current_mana, max_mp), x, y, theme.mana);
    let mp_ratio = if max_mp > 0 { state.current_mana as f32 / max_mp as f32 } else { 0.0 };
    ui_render::bar(engine, x + 4.0, y, 4.5, mp_ratio, theme.mana, theme.dim, 0.25);
    y -= 0.5;

    // Power tier
    ui_render::small(engine, &format!("Power: {:?}", player.power_tier()), x, y, theme.gold);

    // Floor rooms
    if let Some(ref floor) = state.floor {
        ui_render::small(engine, &format!("Floor rooms: {}/{}", floor.current_room + 1, floor.rooms.len()), 2.0, y, theme.muted);
    }
}

// ── Tab 2: Inventory ────────────────────────────────────────────────────────

fn render_inventory(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -8.0;
    let mut y = 3.2;

    ui_render::small(engine, "-- Equipped --", x, y, theme.accent);
    y -= 0.42;

    let slots: [(&str, &Option<chaos_rpg_core::items::Item>); 5] = [
        ("Weapon", &player.equipped.weapon),
        ("Body  ", &player.equipped.body),
        ("Ring 1", &player.equipped.ring1),
        ("Ring 2", &player.equipped.ring2),
        ("Amulet", &player.equipped.amulet),
    ];

    for (slot_name, item_opt) in &slots {
        let text = match item_opt {
            Some(item) => {
                let mod_str = item.stat_modifiers.first()
                    .map(|m| format!("{:?} {:+}", m.stat, m.value))
                    .unwrap_or_default();
                format!("{}: {} [{:?}] {}", slot_name, item.name, item.rarity, mod_str)
            }
            None => format!("{}: (empty)", slot_name),
        };
        let truncated: String = text.chars().take(48).collect();
        ui_render::small(engine, &truncated, x, y, theme.primary);
        y -= 0.38;
    }

    y -= 0.3;
    ui_render::small(engine, "-- Inventory --", x, y, theme.accent);
    y -= 0.42;

    if player.inventory.is_empty() {
        ui_render::small(engine, "(empty)", x, y, theme.muted);
    } else {
        for item in player.inventory.iter().take(14) {
            let mod_str = item.stat_modifiers.first()
                .map(|m| format!("{:?} {:+}", m.stat, m.value))
                .unwrap_or_default();
            let line = format!("{} [{:?}] {}", item.name, item.rarity, mod_str);
            let truncated: String = line.chars().take(50).collect();
            ui_render::small(engine, &truncated, x, y, theme.primary);
            y -= 0.38;
            if y < -4.8 { break; }
        }
        if player.inventory.len() > 14 {
            ui_render::small(engine, &format!("... and {} more", player.inventory.len() - 14), x, y, theme.muted);
        }
    }
}

// ── Tab 3: Effects ──────────────────────────────────────────────────────────

fn render_effects(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -8.0;
    let mut y = 3.2;

    // Corruption
    let corr_ratio = (player.corruption as f32 / 100.0).clamp(0.0, 1.0);
    ui_render::small(engine, &format!("Corruption: {}", player.corruption), x, y, theme.warn);
    ui_render::bar(engine, x + 5.0, y, 3.5, corr_ratio, theme.danger, theme.dim, 0.25);
    y -= 0.55;

    // Misery
    ui_render::small(engine, &format!("Misery Index: {:.2}", player.misery.misery_index), x, y, theme.warn);
    y -= 0.7;

    ui_render::small(engine, "-- Active Status Effects --", x, y, theme.accent);
    y -= 0.42;

    if player.status_effects.is_empty() {
        ui_render::small(engine, "(none)", x, y, theme.muted);
    } else {
        for effect in player.status_effects.iter() {
            let line = format!("{:?}", effect);
            let display: String = line.chars().take(50).collect();
            ui_render::small(engine, &display, x, y, theme.primary);
            y -= 0.38;
            if y < -4.8 { break; }
        }
    }
}

// ── Tab 4: Lore ─────────────────────────────────────────────────────────────

fn render_lore(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -8.0;
    let mut y = 3.2;

    ui_render::small(engine, "-- Class --", x, y, theme.accent);
    y -= 0.42;
    ui_render::body(engine, player.class.name(), x, y, theme.heading);
    y -= 0.55;

    // Word-wrap class description
    let desc = player.class.description();
    for chunk in desc.as_bytes().chunks(48) {
        let line = std::str::from_utf8(chunk).unwrap_or("");
        ui_render::small(engine, line, x, y, theme.primary);
        y -= 0.38;
    }
    y -= 0.3;

    ui_render::small(engine, "-- Background --", x, y, theme.accent);
    y -= 0.42;
    ui_render::body(engine, player.background.name(), x, y, theme.heading);
    y -= 0.6;

    ui_render::small(engine, "-- Boon --", x, y, theme.accent);
    y -= 0.42;
    match &player.boon {
        Some(boon) => {
            let boon_text = format!("{:?}", boon);
            let display: String = boon_text.chars().take(50).collect();
            ui_render::small(engine, &display, x, y, theme.primary);
        }
        None => {
            ui_render::small(engine, "(no boon)", x, y, theme.muted);
        }
    }
}

// ── Tab 5: Log ──────────────────────────────────────────────────────────────

fn render_log(
    state: &GameState,
    engine: &mut ProofEngine,
    theme: &crate::theme::Theme,
) {
    let x = -8.0;
    let mut y = 3.2;

    ui_render::small(engine, "-- Combat Log (last 20) --", x, y, theme.accent);
    y -= 0.42;

    if state.combat_log.is_empty() {
        ui_render::small(engine, "(no log entries)", x, y, theme.muted);
        return;
    }

    let start = state.combat_log.len().saturating_sub(20);
    for entry in &state.combat_log[start..] {
        let display: String = entry.chars().take(52).collect();
        ui_render::small(engine, &display, x, y, theme.primary);
        y -= 0.38;
        if y < -4.8 { break; }
    }
}
