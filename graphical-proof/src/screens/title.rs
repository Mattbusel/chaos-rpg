//! Title screen — CHAOS RPG logo, menu, chaos field background.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    // Read input state into locals to avoid borrow conflicts
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter);
    let theme_key = engine.input.just_pressed(Key::T);
    let quit = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);

    if up && state.selected_menu > 0 {
        state.selected_menu -= 1;
    }
    if down && state.selected_menu < 7 {
        state.selected_menu += 1;
    }
    if theme_key {
        state.theme_idx = (state.theme_idx + 1) % THEMES.len();
    }

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

    if quit {
        engine.request_quit();
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // ── Logo ──
    let logo = "CHAOS RPG";
    let logo_y = 4.0;
    for (i, ch) in logo.chars().enumerate() {
        let x = -4.0 + i as f32 * 1.2;
        let pulse = ((state.frame as f32 * 0.03 + i as f32 * 0.3).sin() * 0.3 + 0.7).max(0.0);
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, logo_y, 0.0),
            color: theme.heading,
            emission: 1.0 + pulse,
            glow_color: theme.glow_from(theme.accent),
            glow_radius: 2.0 + pulse,
            scale: Vec2::splat(2.0),
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // ── Tagline ──
    let tagline = theme.tagline;
    for (i, ch) in tagline.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(-15.0 + i as f32 * 0.38, logo_y - 2.0, 0.0),
            color: theme.dim,
            emission: 0.3,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // ── Menu items ──
    let items = [
        if state.save_exists { "Continue" } else { "Continue (no save)" },
        "New Run",
        "Tutorial",
        "Achievements",
        "Run History",
        "Daily Leaderboard",
        "Settings",
        "Quit",
    ];

    for (idx, label) in items.iter().enumerate() {
        let is_selected = idx == state.selected_menu;
        let color = if is_selected { theme.selected } else { theme.primary };
        let emission = if is_selected { 1.0 } else { 0.4 };
        let prefix = if is_selected { "> " } else { "  " };
        let text = format!("{}{}", prefix, label);

        let y = logo_y - 5.0 - idx as f32 * 1.2;
        for (ci, ch) in text.chars().enumerate() {
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(-6.0 + ci as f32 * 0.5, y, 0.0),
                color,
                emission,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    // ── Theme indicator ──
    let theme_text = format!("[T] Theme: {}", theme.name);
    for (i, ch) in theme_text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(-10.0 + i as f32 * 0.38, -14.0, 0.0),
            color: theme.muted,
            emission: 0.2,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
