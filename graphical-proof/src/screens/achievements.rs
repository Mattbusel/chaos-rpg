//! Achievements browser — scrollable list with filter (All/Unlocked/Locked).

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let pgup = engine.input.just_pressed(Key::PageUp);
    let pgdn = engine.input.just_pressed(Key::PageDown);
    let tab = engine.input.just_pressed(Key::Tab) || engine.input.just_pressed(Key::F);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);

    let total = state.achievements.achievements.len();
    if up && state.achievement_scroll > 0 { state.achievement_scroll -= 1; }
    if down && state.achievement_scroll < total.saturating_sub(1) { state.achievement_scroll += 1; }
    if pgup { state.achievement_scroll = state.achievement_scroll.saturating_sub(15); }
    if pgdn { state.achievement_scroll = (state.achievement_scroll + 15).min(total.saturating_sub(1)); }
    if tab { state.achievement_filter = (state.achievement_filter + 1) % 3; state.achievement_scroll = 0; }
    if esc || enter { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    let filter_name = match state.achievement_filter {
        0 => "All",
        1 => "Unlocked",
        _ => "Locked",
    };

    let unlocked_count = state.achievements.unlocked_count();
    let total = state.achievements.total_count();

    // Header
    ui_render::heading_centered(engine, "ACHIEVEMENTS", 5.0, theme.heading);
    ui_render::small(engine, &format!("{}/{} unlocked  Filter: [{}]", unlocked_count, total, filter_name), -5.0, 4.2, theme.dim);

    // Progress bar
    let progress = if total > 0 { unlocked_count as f32 / total as f32 } else { 0.0 };
    ui_render::bar(engine, -4.0, 3.7, 8.0, progress, theme.success, theme.muted, 0.25);

    // Filter achievements
    let filtered: Vec<&chaos_rpg_core::achievements::Achievement> = state.achievements.achievements.iter()
        .filter(|a| match state.achievement_filter {
            1 => a.unlocked,
            2 => !a.unlocked,
            _ => true,
        })
        .collect();

    if filtered.is_empty() {
        ui_render::body(engine, "No achievements match filter.", -5.5, 1.0, theme.dim);
    } else {
        let start = state.achievement_scroll.saturating_sub(5);
        let end = (start + 14).min(filtered.len());
        for (di, idx) in (start..end).enumerate() {
            let ach = filtered[idx];
            let is_selected = idx == state.achievement_scroll;

            let (icon, color, emission) = if is_selected {
                ("> ", theme.selected, 0.8)
            } else if ach.unlocked {
                ("* ", theme.success, 0.4)
            } else {
                ("  ", theme.dim, 0.2)
            };

            let rarity_tag: String = format!("{:?}", ach.rarity).chars().take(6).collect();
            let line = format!("{}{} [{}]", icon, ach.name, rarity_tag);
            let truncated: String = line.chars().take(40).collect();
            ui_render::text(engine, &truncated, -8.2, 3.0 - di as f32 * 0.48, color, 0.28, emission);
        }

        // Detail panel for selected achievement
        if let Some(ach) = filtered.get(state.achievement_scroll) {
            let px = 2.0;
            ui_render::body(engine, &ach.name, px, 3.0, theme.heading);

            // Word-wrap description
            let max_w = 28;
            let mut y = 2.3;
            let mut line = String::new();
            for word in ach.description.split_whitespace() {
                if line.len() + word.len() + 1 > max_w {
                    ui_render::small(engine, &line, px, y, theme.primary);
                    y -= 0.35;
                    line = word.to_string();
                } else {
                    if !line.is_empty() { line.push(' '); }
                    line.push_str(word);
                }
            }
            if !line.is_empty() {
                ui_render::small(engine, &line, px, y, theme.primary);
                y -= 0.35;
            }

            ui_render::small(engine, &format!("Rarity: {:?}", ach.rarity), px, y - 0.2, theme.accent);
            let status = if ach.unlocked { "UNLOCKED" } else { "LOCKED" };
            let sc = if ach.unlocked { theme.success } else { theme.dim };
            ui_render::small(engine, status, px, y - 0.55, sc);
        }
    }

    ui_render::small(engine, "[Up/Dn] Scroll [PgUp/Dn] Page [Tab/F] Filter [Esc/Enter] Back", -8.5, -5.2, theme.muted);
}
