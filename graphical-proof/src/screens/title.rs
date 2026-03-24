//! Title screen — CHAOS RPG logo, menu, chaos field background.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter);
    let theme_key = engine.input.just_pressed(Key::T);
    let quit = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);

    if up && state.selected_menu > 0 { state.selected_menu -= 1; }
    if down && state.selected_menu < 7 { state.selected_menu += 1; }
    if theme_key { state.theme_idx = (state.theme_idx + 1) % THEMES.len(); }

    if enter {
        match state.selected_menu {
            0 => {
                if state.save_exists {
                    if let Some(save) = crate::state::read_save() {
                        state.player = Some(save.player);
                        state.floor = save.floor;
                        state.floor_num = save.floor_num;
                        state.floor_seed = save.floor_seed;
                        state.seed = save.seed;
                        state.current_mana = save.current_mana;
                        state.is_boss_fight = save.is_boss_fight;
                        state.nemesis_spawned = save.nemesis_spawned;
                        state.combat_log = save.combat_log;
                        state.screen = AppScreen::FloorNav;
                    }
                }
            }
            1 => state.screen = AppScreen::ModeSelect,
            2 => state.screen = AppScreen::Tutorial,
            3 => state.screen = AppScreen::Achievements,
            4 => state.screen = AppScreen::RunHistory,
            5 => state.screen = AppScreen::DailyLeaderboard,
            6 => state.screen = AppScreen::Settings,
            7 => engine.request_quit(),
            _ => {}
        }
    }
    if quit { engine.request_quit(); }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // ── Logo (large, centered, pulsing) ──
    ui_render::title(engine, "CHAOS RPG", 3.5, theme.heading);

    // ── Tagline (small, centered below logo) ──
    ui_render::text_centered(engine, theme.tagline, 1.8, theme.dim, 0.3, 0.3);

    // ── Menu items ──
    let items = [
        if state.save_exists { "Continue" } else { "Continue (no save)" },
        "New Run", "Tutorial", "Achievements",
        "Run History", "Daily Leaderboard", "Settings", "Quit",
    ];

    for (idx, label) in items.iter().enumerate() {
        let selected = idx == state.selected_menu;
        let color = if selected { theme.selected } else { theme.primary };
        let emission = if selected { 0.8 } else { 0.3 };
        let prefix = if selected { "> " } else { "  " };
        let line = format!("{}{}", prefix, label);
        let y = 0.0 - idx as f32 * 0.65;
        ui_render::text(engine, &line, -3.5, y, color, 0.45, emission);
    }

    // ── Theme indicator (bottom) ──
    let theme_line = format!("[T] Theme: {}", theme.name);
    ui_render::small(engine, &theme_line, -4.0, -5.0, theme.muted);
}
