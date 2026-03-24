//! Title screen — CHAOS RPG logo, menu, chaos field background.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let theme_key = engine.input.just_pressed(Key::T);
    let quit = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let num1 = engine.input.just_pressed(Key::Num1);
    let num2 = engine.input.just_pressed(Key::Num2);
    let num3 = engine.input.just_pressed(Key::Num3);
    let num4 = engine.input.just_pressed(Key::Num4);
    let num5 = engine.input.just_pressed(Key::Num5);
    let num6 = engine.input.just_pressed(Key::Num6);
    let num7 = engine.input.just_pressed(Key::Num7);
    let num8 = engine.input.just_pressed(Key::Num8);

    if up && state.selected_menu > 0 { state.selected_menu -= 1; }
    if down && state.selected_menu < 7 { state.selected_menu += 1; }
    if theme_key { state.theme_idx = (state.theme_idx + 1) % THEMES.len(); }

    if num1 { state.selected_menu = 0; }
    if num2 { state.selected_menu = 1; }
    if num3 { state.selected_menu = 2; }
    if num4 { state.selected_menu = 3; }
    if num5 { state.selected_menu = 4; }
    if num6 { state.selected_menu = 5; }
    if num7 { state.selected_menu = 6; }
    if num8 { state.selected_menu = 7; }

    let activate = enter || num1 || num2 || num3 || num4 || num5 || num6 || num7 || num8;

    if activate {
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

    // Dark backing for text readability over chaos field
    ui_render::screen_backing(engine, 0.4);

    // Logo (top) — Tier 1 brightness
    ui_render::title(engine, "CHAOS RPG", 4.5, theme.heading);

    // Tagline — Tier 4 brightness, higher emission for readability over chaos field
    ui_render::text_centered(engine, theme.tagline, 3.0, theme.dim, 0.28, 0.4);

    // Semi-transparent dark panel behind menu for contrast
    ui_render::panel_bg(engine, -5.0, 2.0, 10.0, 6.5, theme.bg, 0.4);
    ui_render::box_single(engine, -5.0, 2.0, 10.0, 6.5, theme.border, 0.3, 0.3);

    // Menu items — Tier 2/3 brightness
    let items = [
        "[1] Continue", "[2] New Run", "[3] Tutorial", "[4] Achievements",
        "[5] Run History", "[6] Daily Board", "[7] Settings", "[8] Quit",
    ];

    let menu_top = 1.5;
    let line_height = 0.6;

    for (idx, label) in items.iter().enumerate() {
        let selected = idx == state.selected_menu;
        // Tier 1 (selected) vs Tier 3 (unselected)
        let color = if selected { theme.selected } else { theme.primary };
        let emission = if selected { 1.0 } else { 0.4 };
        let y = menu_top - idx as f32 * line_height;

        if selected {
            ui_render::cursor_arrow(engine, -4.2, y, theme.accent, 0.4, state.frame);
        }
        ui_render::text(engine, label, -3.2, y, color, 0.4, emission);
    }

    // Footer — Tier 5 brightness
    ui_render::small(engine, &format!("[T] Theme: {}", theme.name), -4.0, -4.5, theme.muted);
    ui_render::small(engine, "Arrows + Enter/Space | Number keys", -4.5, -5.0, theme.muted);
}
