//! Title screen — CHAOS RPG logo, menu, chaos field background.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    // Read all input into locals
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let theme_key = engine.input.just_pressed(Key::T);
    let quit = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    // Number keys for direct menu selection
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

    // Direct number key selection
    if num1 { state.selected_menu = 0; }
    if num2 { state.selected_menu = 1; }
    if num3 { state.selected_menu = 2; }
    if num4 { state.selected_menu = 3; }
    if num5 { state.selected_menu = 4; }
    if num6 { state.selected_menu = 5; }
    if num7 { state.selected_menu = 6; }
    if num8 { state.selected_menu = 7; }

    // Activate menu item on Enter, Space, or number key
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

    // Logo
    ui_render::title(engine, "CHAOS RPG", 3.5, theme.heading);

    // Tagline
    ui_render::text_centered(engine, theme.tagline, 1.8, theme.dim, 0.25, 0.25);

    // Menu items with number keys
    let items = [
        "[1] Continue", "[2] New Run", "[3] Tutorial", "[4] Achievements",
        "[5] Run History", "[6] Daily Board", "[7] Settings", "[8] Quit",
    ];

    for (idx, label) in items.iter().enumerate() {
        let selected = idx == state.selected_menu;
        let color = if selected { theme.selected } else { theme.primary };
        let emission = if selected { 0.8 } else { 0.3 };
        let prefix = if selected { "> " } else { "  " };
        let line = format!("{}{}", prefix, label);
        let y = 0.0 - idx as f32 * 0.55;
        ui_render::text(engine, &line, -3.0, y, color, 0.4, emission);
    }

    // Theme indicator
    ui_render::small(engine, &format!("[T] Theme: {}", theme.name), -3.5, -5.0, theme.muted);

    // Input hint
    ui_render::small(engine, "Arrow keys + Enter/Space to select", -4.0, -5.4, theme.muted);

    // Debug: show last pressed keys
    let pressed_keys: Vec<&str> = [
        (engine.input.is_pressed(Key::Enter), "ENTER"),
        (engine.input.is_pressed(Key::Space), "SPACE"),
        (engine.input.is_pressed(Key::Up), "UP"),
        (engine.input.is_pressed(Key::Down), "DOWN"),
    ].iter().filter(|(p, _)| *p).map(|(_, n)| *n).collect();
    if !pressed_keys.is_empty() {
        let debug_text = format!("Keys: {}", pressed_keys.join(" "));
        ui_render::small(engine, &debug_text, -3.0, 5.0, theme.accent);
    }
}
