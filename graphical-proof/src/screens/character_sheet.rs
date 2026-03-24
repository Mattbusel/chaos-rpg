//! Character sheet — 5 tabs: Stats, Inventory, Effects, Lore, Log.
//! Full stat bars, equipment slots, stat comparison, scrollable content.

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
    let frame = state.frame;

    // ── Tab bar ──
    let mut tx = -7.5;
    for (i, name) in TAB_NAMES.iter().enumerate() {
        let label = format!("[{}] {}", i + 1, name);
        let selected = i == tab;
        let color = if selected { theme.selected } else { theme.dim };
        let em = if selected { 0.8 } else { 0.25 };
        ui_render::text(engine, &label, tx, 5.0, color, 0.3, em);
        tx += label.len() as f32 * 0.3 * 0.85 + 0.8;
    }

    // Active tab underline
    ui_render::text_centered(engine, "================================", 4.5, theme.border, 0.22, 0.15);

    // Title
    let title = format!("Character Sheet - {}", TAB_NAMES[tab]);
    ui_render::text_centered(engine, &title, 4.0, theme.heading, 0.4, 0.6);

    let player = match state.player.as_ref() {
        Some(p) => p,
        None => {
            ui_render::text_centered(engine, "No character data.", 1.0, theme.danger, 0.4, 0.5);
            return;
        }
    };

    match tab {
        0 => render_stats(state, engine, player, theme, frame),
        1 => render_inventory(engine, player, theme, frame),
        2 => render_effects(engine, player, theme, frame),
        3 => render_lore(engine, player, theme),
        4 => render_log(state, engine, theme),
        _ => {}
    }

    // Footer
    ui_render::small(engine, "[Esc/C] Back  [1-5] Tabs  [Left/Right] Cycle", -6.5, -5.2, theme.muted);
}

// ── Tab 1: Stats ────────────────────────────────────────────────────────────

fn render_stats(
    state: &GameState,
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
    frame: u64,
) {
    let x = -8.0;
    let mut y = 3.2;

    // Core info
    let info = format!(
        "Lv {} | XP {} | Gold {} | Kills {} | {:?}",
        player.level, player.xp, player.gold, player.kills, player.power_tier()
    );
    let truncated: String = info.chars().take(50).collect();
    ui_render::text(engine, &truncated, x, y, theme.primary, 0.28, 0.4);
    y -= 0.5;

    let floor_info = format!(
        "Floor {} | Rooms {} | Corruption {}",
        player.floor, player.rooms_cleared, player.corruption
    );
    ui_render::text(engine, &floor_info, x, y, theme.dim, 0.25, 0.3);
    y -= 0.7;

    // HP bar
    let hp_pct = if player.max_hp > 0 { player.current_hp as f32 / player.max_hp as f32 } else { 0.0 };
    ui_render::text(engine, "HP", x, y, theme.muted, 0.25, 0.3);
    ui_render::bar(engine, x + 1.2, y, 4.5, hp_pct, theme.hp_color(hp_pct), theme.muted, 0.25);
    ui_render::small(engine, &format!("{}/{}", player.current_hp, player.max_hp), x + 6.0, y, theme.hp_color(hp_pct));
    y -= 0.5;

    // Mana bar
    let max_mp = state.max_mana();
    if max_mp > 0 {
        let mp_pct = state.current_mana as f32 / max_mp as f32;
        ui_render::text(engine, "MP", x, y, theme.muted, 0.25, 0.3);
        ui_render::bar(engine, x + 1.2, y, 4.5, mp_pct, theme.mana, theme.muted, 0.25);
        ui_render::small(engine, &format!("{}/{}", state.current_mana, max_mp), x + 6.0, y, theme.mana);
        y -= 0.55;
    }

    // Section header
    ui_render::text(engine, "-- Primary Stats --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;

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

    for (name, val) in &stats {
        let ratio = (*val as f32 / 50.0).clamp(0.0, 1.0);
        let stat_color = if *val > 30 { theme.success } else if *val > 15 { theme.primary } else { theme.warn };
        ui_render::text(engine, &format!("{}: {:>3}", name, val), x, y, stat_color, 0.25, 0.4);
        ui_render::bar(engine, x + 2.8, y, 4.0, ratio, theme.accent, theme.muted, 0.22);

        // Animated sparkle for high stats
        if *val > 40 {
            let sparkle = ((frame as f32 * 0.08 + *val as f32).sin() * 0.4 + 0.6).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '*',
                position: Vec3::new(x + 7.2, y, 0.0),
                color: Vec4::new(theme.gold.x * sparkle, theme.gold.y * sparkle, 0.0, sparkle),
                emission: sparkle,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
        y -= 0.4;
    }

    // Floor info
    y -= 0.3;
    if let Some(ref floor) = state.floor {
        let rooms_info = format!("Floor rooms: {}/{}", floor.current_room + 1, floor.rooms.len());
        ui_render::small(engine, &rooms_info, x, y, theme.muted);
    }
}

// ── Tab 2: Inventory ────────────────────────────────────────────────────────

fn render_inventory(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
    frame: u64,
) {
    let x = -8.0;
    let mut y = 3.2;

    // Equipped items section
    ui_render::text(engine, "-- Equipped --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;

    let slots: [(&str, &Option<chaos_rpg_core::items::Item>); 5] = [
        ("Weapon", &player.equipped.weapon),
        ("Body  ", &player.equipped.body),
        ("Ring 1", &player.equipped.ring1),
        ("Ring 2", &player.equipped.ring2),
        ("Amulet", &player.equipped.amulet),
    ];

    for (slot_name, item_opt) in &slots {
        let (text, color) = match item_opt {
            Some(item) => {
                let mod_str = item.stat_modifiers.first()
                    .map(|m| format!("{:+} {}", m.value, m.stat))
                    .unwrap_or_default();
                let rarity_tag: String = format!("{:?}", item.rarity).chars().take(4).collect();
                (format!("{}: {} [{}] {}", slot_name, item.name, rarity_tag, mod_str), theme.primary)
            }
            None => (format!("{}: (empty)", slot_name), theme.muted),
        };
        let truncated: String = text.chars().take(50).collect();
        ui_render::text(engine, &truncated, x, y, color, 0.25, 0.35);

        // Slot icon glow for equipped items
        if item_opt.is_some() {
            let glow = ((frame as f32 * 0.04 + y * 2.0).sin() * 0.15 + 0.5).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '#',
                position: Vec3::new(x - 0.4, y, 0.0),
                color: Vec4::new(theme.accent.x * glow, theme.accent.y * glow, theme.accent.z * glow, glow),
                emission: glow * 0.5,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
        y -= 0.4;
    }

    y -= 0.3;
    ui_render::text(engine, "-- Inventory --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;

    if player.inventory.is_empty() {
        ui_render::small(engine, "(empty)", x, y, theme.muted);
    } else {
        for item in player.inventory.iter().take(14) {
            let mod_str = item.stat_modifiers.first()
                .map(|m| format!("{:+} {}", m.value, m.stat))
                .unwrap_or_default();
            let rarity_tag: String = format!("{:?}", item.rarity).chars().take(4).collect();
            let line = format!("{} [{}] {}", item.name, rarity_tag, mod_str);
            let truncated: String = line.chars().take(48).collect();
            ui_render::text(engine, &truncated, x, y, theme.primary, 0.23, 0.3);
            y -= 0.38;
            if y < -4.8 { break; }
        }
        if player.inventory.len() > 14 {
            let more = format!("... +{} more", player.inventory.len() - 14);
            ui_render::small(engine, &more, x, y, theme.muted);
        }
    }
}

// ── Tab 3: Effects ──────────────────────────────────────────────────────────

fn render_effects(
    engine: &mut ProofEngine,
    player: &chaos_rpg_core::character::Character,
    theme: &crate::theme::Theme,
    frame: u64,
) {
    let x = -8.0;
    let mut y = 3.2;

    // Corruption
    let corr_ratio = (player.corruption as f32 / 100.0).clamp(0.0, 1.0);
    ui_render::text(engine, &format!("Corruption: {}", player.corruption), x, y, theme.warn, 0.3, 0.5);
    ui_render::bar(engine, x + 5.0, y, 3.5, corr_ratio, theme.danger, theme.muted, 0.25);
    y -= 0.55;

    // Misery index
    ui_render::text(engine, &format!("Misery: {:.1}", player.misery.misery_index), x, y, theme.warn, 0.3, 0.45);
    y -= 0.55;

    // Corruption visual warning
    if player.corruption > 50 {
        let pulse = ((frame as f32 * 0.12).sin() * 0.3 + 0.7).max(0.0);
        let warn_c = Vec4::new(theme.danger.x * pulse, 0.0, 0.0, pulse);
        ui_render::small(engine, "!! Engines mutating !!", x, y, warn_c);
        y -= 0.4;
    }

    y -= 0.2;
    ui_render::text(engine, "-- Active Status Effects --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;

    if player.status_effects.is_empty() {
        ui_render::small(engine, "(none)", x, y, theme.muted);
    } else {
        for effect in player.status_effects.iter() {
            let line = format!("{:?}", effect);
            let display: String = line.chars().take(55).collect();
            ui_render::text(engine, &display, x, y, theme.primary, 0.23, 0.35);
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

    // Class section
    ui_render::text(engine, "-- Class --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;
    ui_render::text(engine, &player.class.name().to_string(), x, y, theme.heading, 0.4, 0.7);
    y -= 0.55;

    // Wrap class description
    let desc = player.class.description();
    let max_width = 48;
    let mut line_buf = String::new();
    for word in desc.split_whitespace() {
        if line_buf.len() + word.len() + 1 > max_width {
            ui_render::text(engine, &line_buf, x, y, theme.dim, 0.23, 0.3);
            y -= 0.35;
            line_buf = word.to_string();
        } else {
            if !line_buf.is_empty() { line_buf.push(' '); }
            line_buf.push_str(word);
        }
    }
    if !line_buf.is_empty() {
        ui_render::text(engine, &line_buf, x, y, theme.dim, 0.23, 0.3);
        y -= 0.35;
    }

    y -= 0.3;
    ui_render::text(engine, "-- Background --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;
    ui_render::text(engine, &player.background.name().to_string(), x, y, theme.heading, 0.35, 0.6);
    y -= 0.6;

    // Boon
    ui_render::text(engine, "-- Boon --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;
    match &player.boon {
        Some(boon) => {
            let boon_text = format!("{:?}", boon);
            let display: String = boon_text.chars().take(55).collect();
            ui_render::text(engine, &display, x, y, theme.primary, 0.25, 0.4);
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

    ui_render::text(engine, "-- Combat Log (last 20) --", x, y, theme.accent, 0.28, 0.5);
    y -= 0.45;

    if state.combat_log.is_empty() {
        ui_render::small(engine, "(no log entries)", x, y, theme.muted);
        return;
    }

    let start = state.combat_log.len().saturating_sub(20);
    for entry in &state.combat_log[start..] {
        let display: String = entry.chars().take(55).collect();
        ui_render::text(engine, &display, x, y, theme.primary, 0.22, 0.3);
        y -= 0.35;
        if y < -4.8 { break; }
    }
}
