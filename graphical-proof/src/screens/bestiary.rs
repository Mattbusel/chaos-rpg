//! Bestiary — enemy encyclopedia.
//! Shows enemies the player has encountered during runs.
//! Generates a preview using the enemy generation system.

use proof_engine::prelude::*;
use chaos_rpg_core::enemy::generate_enemy;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

/// Generate a static preview of enemies at various floor levels.
fn generate_preview() -> Vec<(String, String, u32)> {
    let mut entries = Vec::new();
    let seed_base = 42u64;
    for floor in [1, 5, 10, 15, 20, 25, 30, 40, 50, 60, 75, 90, 100] {
        for i in 0..3u64 {
            let enemy = generate_enemy(floor, seed_base.wrapping_add(floor as u64 * 10 + i));
            let tier_str = format!("{:?}", enemy.tier);
            if !entries.iter().any(|(n, _, _): &(String, String, u32)| n == &enemy.name) {
                entries.push((enemy.name, tier_str, floor));
            }
        }
    }
    entries
}

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);

    let entries = generate_preview();
    let total = entries.len();
    if up && state.bestiary_scroll > 0 { state.bestiary_scroll -= 1; }
    if down && state.bestiary_scroll < total.saturating_sub(1) { state.bestiary_scroll += 1; }
    if esc || enter { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let entries = generate_preview();

    ui_render::heading_centered(engine, "BESTIARY", 5.0, theme.heading);
    ui_render::small(engine, &format!("{} enemies catalogued", entries.len()), -3.5, 4.2, theme.dim);

    if entries.is_empty() {
        ui_render::body(engine, "No enemies in the bestiary.", -5.0, 1.0, theme.dim);
    } else {
        // Left panel: enemy list
        let start = state.bestiary_scroll.saturating_sub(5);
        let end = (start + 14).min(entries.len());
        for (di, idx) in (start..end).enumerate() {
            let (ref name, ref tier, floor) = entries[idx];
            let selected = idx == state.bestiary_scroll;
            let color = if selected { theme.selected } else { theme.primary };
            let prefix = if selected { "> " } else { "  " };
            let line = format!("{}{} ({}, F{})", prefix, name, tier, floor);
            let truncated: String = line.chars().take(32).collect();
            ui_render::text(engine, &truncated, -8.2, 3.2 - di as f32 * 0.48, color, 0.28,
                if selected { 0.7 } else { 0.35 });
        }

        // Right panel: selected enemy detail
        if let Some((ref name, ref tier, floor)) = entries.get(state.bestiary_scroll) {
            let px = 1.5;
            let mut y = 3.2;

            ui_render::body(engine, name, px, y, theme.heading);
            y -= 0.55;
            ui_render::small(engine, &format!("Tier: {}", tier), px, y, theme.accent);
            y -= 0.42;
            ui_render::small(engine, &format!("First appears: Floor {}", floor), px, y, theme.primary);
            y -= 0.55;

            // Generate fresh enemy for stats
            let enemy = generate_enemy(*floor, 42u64.wrapping_add(*floor as u64 * 10));
            ui_render::small(engine, &format!("HP: {}", enemy.max_hp), px, y, theme.hp_high);
            y -= 0.42;
            ui_render::small(engine, &format!("Base Damage: {}", enemy.base_damage), px, y, theme.danger);
            y -= 0.42;
            ui_render::small(engine, &format!("XP Reward: {}", enemy.xp_reward), px, y, theme.xp);
            y -= 0.42;
            ui_render::small(engine, &format!("Gold Reward: {}", enemy.gold_reward), px, y, theme.gold);
            y -= 0.5;

            if let Some(ability) = enemy.special_ability {
                ui_render::small(engine, &format!("Ability: {}", ability), px, y, theme.warn);
                y -= 0.42;
            }
            ui_render::small(engine, &format!("Floor: {:?}", enemy.floor_ability), px, y, theme.dim);
        }
    }

    ui_render::small(engine, "[Up/Down] Scroll  [Enter/Space/Esc] Back", -6.5, -5.2, theme.muted);
}
