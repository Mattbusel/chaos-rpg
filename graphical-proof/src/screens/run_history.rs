//! Run history — scrollable list of past runs with detail panel.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);

    let total = state.run_history.runs.len();
    if up && state.history_scroll > 0 { state.history_scroll -= 1; }
    if down && state.history_scroll < total.saturating_sub(1) { state.history_scroll += 1; }
    if esc || enter { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    ui_render::screen_backing(engine, 0.6);

    ui_render::heading_centered(engine, "RUN HISTORY", 5.0, theme.heading);
    ui_render::small(engine, &format!("{} runs recorded", state.run_history.runs.len()), -3.0, 4.2, theme.dim);

    if state.run_history.runs.is_empty() {
        ui_render::body(engine, "No runs recorded yet.", -4.5, 1.0, theme.dim);
        ui_render::small(engine, "Go die a few times.", -3.5, 0.3, theme.muted);
    } else {
        // Left panel: run list
        let start = state.history_scroll.saturating_sub(5);
        let end = (start + 14).min(state.run_history.runs.len());
        for (di, idx) in (start..end).enumerate() {
            let run = &state.run_history.runs[idx];
            let selected = idx == state.history_scroll;
            let color = if selected { theme.selected } else { theme.primary };
            let prefix = if selected { "> " } else { "  " };
            let won_marker = if run.won { "*" } else { "x" };
            let line = format!("{}{} {} {} Lv{} F{}", prefix, won_marker, run.name, run.class, run.level, run.floor);
            let truncated: String = line.chars().take(32).collect();
            ui_render::text(engine, &truncated, -8.2, 3.2 - di as f32 * 0.48, color, 0.28,
                if selected { 0.7 } else { 0.35 });
        }

        // Right panel: selected run detail
        if let Some(run) = state.run_history.runs.get(state.history_scroll) {
            let px = 1.5;
            let mut y = 3.2;

            ui_render::body(engine, &format!("{} - {}", run.name, run.class), px, y, theme.heading);
            y -= 0.55;
            ui_render::small(engine, &format!("Diff: {} | Mode: {}", run.difficulty, run.game_mode), px, y, theme.dim);
            y -= 0.42;
            ui_render::small(engine, &format!("Floor: {} | Lv: {} | Kills: {}", run.floor, run.level, run.kills), px, y, theme.primary);
            y -= 0.42;
            ui_render::small(engine, &format!("Score: {} | Gold: {}", run.score, run.gold), px, y, theme.gold);
            y -= 0.42;
            ui_render::small(engine, &format!("Dmg: {} dealt / {} taken", run.damage_dealt, run.damage_taken), px, y, theme.dim);
            y -= 0.42;
            ui_render::small(engine, &format!("Tier: {} | Corruption: {}", run.power_tier, run.corruption), px, y, theme.accent);
            y -= 0.5;

            if !run.cause_of_death.is_empty() {
                let cod: String = run.cause_of_death.chars().take(30).collect();
                ui_render::small(engine, &format!("Cause: {}", cod), px, y, theme.danger);
                y -= 0.42;
            }
            ui_render::small(engine, &run.date, px, y, theme.muted);
            y -= 0.5;

            if !run.auto_narrative.is_empty() {
                let excerpt: String = run.auto_narrative.chars().take(35).collect();
                ui_render::small(engine, &excerpt, px, y, theme.dim);
            }
        }
    }

    ui_render::small(engine, "[Up/Down] Scroll  [Enter/Space/Esc] Back", -6.5, -5.2, theme.muted);
}
