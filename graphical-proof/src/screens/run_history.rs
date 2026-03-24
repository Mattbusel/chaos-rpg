//! Run history — scrollable list of past runs with detail panel.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);

    let total = state.run_history.runs.len();
    if up && state.history_scroll > 0 { state.history_scroll -= 1; }
    if down && state.history_scroll < total.saturating_sub(1) { state.history_scroll += 1; }
    if esc { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    render_text(engine, &format!("RUN HISTORY — {} runs", state.run_history.runs.len()),
        -18.0, 9.0, theme.heading, 0.8);

    if state.run_history.runs.is_empty() {
        render_text(engine, "No runs recorded yet. Go die a few times.", -12.0, 4.0, theme.dim, 0.3);
    } else {
        // Left panel: run list
        let start = state.history_scroll.saturating_sub(6);
        let end = (start + 14).min(state.run_history.runs.len());
        for (di, idx) in (start..end).enumerate() {
            let run = &state.run_history.runs[idx];
            let selected = idx == state.history_scroll;
            let color = if selected { theme.selected } else { theme.primary };
            let prefix = if selected { "> " } else { "  " };
            let won_marker = if run.won { "★" } else { "☠" };
            let line = format!("{}{} {} {} Lv.{} F{}", prefix, won_marker, run.name, run.class, run.level, run.floor);
            let truncated: String = line.chars().take(40).collect();
            render_text(engine, &truncated, -18.0, 7.0 - di as f32 * 1.0, color,
                if selected { 0.7 } else { 0.35 });
        }

        // Right panel: selected run detail
        if let Some(run) = state.run_history.runs.get(state.history_scroll) {
            let x = 2.0;
            render_text(engine, &format!("{} — {}", run.name, run.class), x, 7.0, theme.heading, 0.6);
            render_text(engine, &format!("Difficulty: {} | Mode: {}", run.difficulty, run.game_mode), x, 5.5, theme.dim, 0.35);
            render_text(engine, &format!("Floor: {} | Level: {} | Kills: {}", run.floor, run.level, run.kills), x, 4.5, theme.primary, 0.4);
            render_text(engine, &format!("Score: {} | Gold: {}", run.score, run.gold), x, 3.5, theme.gold, 0.4);
            render_text(engine, &format!("Damage: {} dealt / {} taken", run.damage_dealt, run.damage_taken), x, 2.5, theme.dim, 0.35);
            render_text(engine, &format!("Tier: {} | Corruption: {}", run.power_tier, run.corruption), x, 1.5, theme.accent, 0.4);
            if !run.cause_of_death.is_empty() {
                render_text(engine, &format!("Cause: {}", run.cause_of_death), x, 0.0, theme.danger, 0.4);
            }
            render_text(engine, &run.date, x, -1.0, theme.muted, 0.25);

            // Auto-narrative excerpt
            if !run.auto_narrative.is_empty() {
                let excerpt: String = run.auto_narrative.chars().take(70).collect();
                render_text(engine, &excerpt, x, -3.0, theme.dim, 0.25);
            }
        }
    }

    render_text(engine, "[Up/Down] Scroll  [Esc] Back", -18.0, -12.0, theme.muted, 0.2);
}

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color, emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
