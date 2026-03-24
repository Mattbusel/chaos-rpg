//! Character sheet screen — 5 tabs: Stats, Inventory, Effects, Lore, Log.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

const TAB_NAMES: [&str; 5] = ["Stats", "Inventory", "Effects", "Lore", "Log"];

// ── Helpers ─────────────────────────────────────────────────────────────────

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color,
            emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

fn render_bar(engine: &mut ProofEngine, x: f32, y: f32, width: usize, ratio: f32, fill: Vec4, bg: Vec4) {
    let filled = ((ratio.clamp(0.0, 1.0) * width as f32) as usize).min(width);
    for i in 0..width {
        let ch = if i < filled { '=' } else { '-' };
        let color = if i < filled { fill } else { bg };
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color,
            emission: if i < filled { 0.6 } else { 0.1 },
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    // Read all input before mutating state
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

    if pressed_esc || pressed_c {
        state.screen = AppScreen::FloorNav;
    }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let tab = state.char_tab as usize;

    // Tab bar at top
    let mut tx = -12.0;
    for (i, name) in TAB_NAMES.iter().enumerate() {
        let label = format!("[{}] {}", i + 1, name);
        let color = if i == tab { theme.selected } else { theme.dim };
        let emission = if i == tab { 0.8 } else { 0.3 };
        render_text(engine, &label, tx, 8.5, color, emission);
        tx += label.len() as f32 * 0.45 + 1.2;
    }

    // Separator
    render_text(engine, "----------------------------------------", -12.0, 7.8, theme.border, 0.2);

    // Title
    render_text(engine, &format!("Character Sheet - {}", TAB_NAMES[tab]),
                -8.0, 7.0, theme.heading, 0.7);

    let player = match state.player.as_ref() {
        Some(p) => p,
        None => {
            render_text(engine, "No character data.", -6.0, 4.0, theme.danger, 0.5);
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
    render_text(engine, "[Esc/C] Back  [1-5] Tabs  [Left/Right] Cycle",
                -12.0, -8.5, theme.muted, 0.3);
}

// ── Tab 1: Stats ────────────────────────────────────────────────────────────

fn render_stats(
    state: &GameState,
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -11.0;
    let mut y = 6.0;

    // Core info row
    let info = format!(
        "Lv {} | XP {} | Gold {} | Kills {} | Power {:?}",
        player.level, player.xp, player.gold, player.kills, player.power_tier()
    );
    render_text(engine, &info, x, y, theme.primary, 0.5);
    y -= 1.0;

    let floor_info = format!(
        "Floor {} | Rooms Cleared {} | Corruption {}",
        player.floor, player.rooms_cleared, player.corruption
    );
    render_text(engine, &floor_info, x, y, theme.primary, 0.5);
    y -= 1.5;

    // Stats with bars
    let stats: [(&str, i64); 7] = [
        ("Vitality ", player.stats.vitality),
        ("Force    ", player.stats.force),
        ("Mana     ", player.stats.mana),
        ("Cunning  ", player.stats.cunning),
        ("Precision", player.stats.precision),
        ("Entropy  ", player.stats.entropy),
        ("Luck     ", player.stats.luck),
    ];

    render_text(engine, "-- Primary Stats --", x, y, theme.accent, 0.6);
    y -= 0.8;

    for (name, val) in &stats {
        let label = format!("{}: {:>4}", name, val);
        render_text(engine, &label, x, y, theme.primary, 0.5);
        // Bar: scale so 50 is full, clamp
        let ratio = (*val as f32 / 50.0).clamp(0.0, 1.0);
        render_bar(engine, x + 7.5, y, 16, ratio, theme.accent, theme.dim);
        y -= 0.7;
    }

    y -= 0.5;

    // HP / Max HP
    let hp_label = format!("HP: {} / {}", player.current_hp, player.max_hp);
    render_text(engine, &hp_label, x, y, theme.success, 0.6);
    let hp_ratio = if player.max_hp > 0 { player.current_hp as f32 / player.max_hp as f32 } else { 0.0 };
    render_bar(engine, x + 7.5, y, 16, hp_ratio, theme.success, theme.dim);
    y -= 0.9;

    // Current floor rooms info
    if let Some(ref floor) = state.floor {
        let rooms_info = format!("Floor rooms: {}/{}", floor.current_room + 1, floor.rooms.len());
        render_text(engine, &rooms_info, x, y, theme.muted, 0.4);
    }
}

// ── Tab 2: Inventory ────────────────────────────────────────────────────────

fn render_inventory(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -11.0;
    let mut y = 6.0;

    // Equipped items
    render_text(engine, "-- Equipped --", x, y, theme.accent, 0.6);
    y -= 0.8;

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
        render_text(engine, &text, x, y, theme.primary, 0.5);
        y -= 0.7;
    }

    y -= 0.5;
    render_text(engine, "-- Inventory --", x, y, theme.accent, 0.6);
    y -= 0.8;

    if player.inventory.is_empty() {
        render_text(engine, "(empty)", x, y, theme.muted, 0.3);
    } else {
        for item in player.inventory.iter().take(12) {
            let mod_str = item.stat_modifiers.first()
                .map(|m| format!("{:?} {:+}", m.stat, m.value))
                .unwrap_or_default();
            let line = format!("{} [{:?}] {}", item.name, item.rarity, mod_str);
            render_text(engine, &line, x, y, theme.primary, 0.45);
            y -= 0.7;
            if y < -7.5 { break; }
        }
        if player.inventory.len() > 12 {
            let more = format!("... and {} more items", player.inventory.len() - 12);
            render_text(engine, &more, x, y, theme.muted, 0.3);
        }
    }
}

// ── Tab 3: Effects ──────────────────────────────────────────────────────────

fn render_effects(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -11.0;
    let mut y = 6.0;

    // Corruption
    let corr_text = format!("Corruption: {}", player.corruption);
    render_text(engine, &corr_text, x, y, theme.warn, 0.6);
    let corr_ratio = (player.corruption as f32 / 100.0).clamp(0.0, 1.0);
    render_bar(engine, x + 8.0, y, 14, corr_ratio, theme.danger, theme.dim);
    y -= 1.0;

    // Misery index
    let misery_text = format!("Misery Index: {:.2}", player.misery.misery_index);
    render_text(engine, &misery_text, x, y, theme.warn, 0.5);
    y -= 1.2;

    render_text(engine, "-- Active Status Effects --", x, y, theme.accent, 0.6);
    y -= 0.8;

    if player.status_effects.is_empty() {
        render_text(engine, "(none)", x, y, theme.muted, 0.3);
    } else {
        for effect in player.status_effects.iter() {
            let line = format!("{:?}", effect);
            // Truncate long debug strings
            let display: String = line.chars().take(70).collect();
            render_text(engine, &display, x, y, theme.primary, 0.45);
            y -= 0.7;
            if y < -7.5 { break; }
        }
    }
}

// ── Tab 4: Lore ─────────────────────────────────────────────────────────────

fn render_lore(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
) {
    let x = -11.0;
    let mut y = 6.0;

    // Class
    render_text(engine, "-- Class --", x, y, theme.accent, 0.6);
    y -= 0.8;
    render_text(engine, &format!("{}", player.class.name()), x, y, theme.heading, 0.7);
    y -= 0.8;

    // Wrap class description
    let desc = player.class.description();
    for chunk in desc.as_bytes().chunks(60) {
        let line = std::str::from_utf8(chunk).unwrap_or("");
        render_text(engine, line, x, y, theme.primary, 0.45);
        y -= 0.7;
    }
    y -= 0.5;

    // Background
    render_text(engine, "-- Background --", x, y, theme.accent, 0.6);
    y -= 0.8;
    render_text(engine, &format!("{}", player.background.name()), x, y, theme.heading, 0.7);
    y -= 1.0;

    // Boon
    render_text(engine, "-- Boon --", x, y, theme.accent, 0.6);
    y -= 0.8;
    match &player.boon {
        Some(boon) => {
            let boon_text = format!("{:?}", boon);
            let display: String = boon_text.chars().take(70).collect();
            render_text(engine, &display, x, y, theme.primary, 0.5);
        }
        None => {
            render_text(engine, "(no boon)", x, y, theme.muted, 0.3);
        }
    }
}

// ── Tab 5: Log ──────────────────────────────────────────────────────────────

fn render_log(
    state: &GameState,
    engine: &mut ProofEngine,
    theme: &crate::theme::Theme,
) {
    let x = -11.0;
    let mut y = 6.0;

    render_text(engine, "-- Combat Log (last 20) --", x, y, theme.accent, 0.6);
    y -= 0.9;

    if state.combat_log.is_empty() {
        render_text(engine, "(no log entries)", x, y, theme.muted, 0.3);
        return;
    }

    let start = if state.combat_log.len() > 20 {
        state.combat_log.len() - 20
    } else {
        0
    };

    for entry in &state.combat_log[start..] {
        let display: String = entry.chars().take(65).collect();
        render_text(engine, &display, x, y, theme.primary, 0.4);
        y -= 0.65;
        if y < -7.5 { break; }
    }
}
