//! Scoreboard — Hall of Chaos + Hall of Misery.

use proof_engine::prelude::*;
use chaos_rpg_core::scoreboard::{load_scores, load_misery_scores};
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    if esc || enter { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    ui_render::screen_backing(engine, 0.6);

    let scores = load_scores();
    let misery_scores = load_misery_scores();

    // Hall of Chaos (left panel)
    ui_render::body(engine, "HALL OF CHAOS", -8.0, 5.0, theme.heading);
    ui_render::small(engine, "Rank Name            Score    Floor", -8.0, 4.3, theme.muted);

    for (i, entry) in scores.iter().enumerate().take(10) {
        let color = match i {
            0 => theme.gold,
            1..=2 => theme.accent,
            _ => theme.primary,
        };
        let name_trunc: String = entry.name.chars().take(14).collect();
        let line = format!("{:>4} {:14} {:>8} {:>5}", i + 1, name_trunc, entry.score, entry.floor_reached);
        ui_render::text(engine, &line, -8.0, 3.6 - i as f32 * 0.42, color, 0.25, if i < 3 { 0.6 } else { 0.35 });
    }

    if scores.is_empty() {
        ui_render::small(engine, "No scores yet.", -7.0, 3.0, theme.dim);
    }

    // Hall of Misery (right panel)
    ui_render::body(engine, "HALL OF MISERY", 1.5, 5.0, theme.danger);
    ui_render::small(engine, "Rank Name            Misery   Floor", 1.5, 4.3, theme.muted);

    for (i, entry) in misery_scores.iter().enumerate().take(10) {
        let color = match i {
            0 => theme.danger,
            1..=2 => theme.warn,
            _ => theme.dim,
        };
        let name_trunc: String = entry.name.chars().take(14).collect();
        let line = format!("{:>4} {:14} {:>8.0} {:>5}", i + 1, name_trunc, entry.misery_index, entry.floor_reached);
        ui_render::text(engine, &line, 1.5, 3.6 - i as f32 * 0.42, color, 0.25, if i < 3 { 0.5 } else { 0.25 });
    }

    if misery_scores.is_empty() {
        ui_render::small(engine, "No misery yet.", 2.5, 3.0, theme.dim);
    }

    ui_render::small(engine, "[Enter/Space/Esc] Back", -3.5, -5.2, theme.muted);
}
