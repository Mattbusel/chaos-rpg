//! Daily leaderboard — today's seeded challenge rankings.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let refresh = engine.input.just_pressed(Key::R);

    if refresh {
        state.daily_status = "Fetching...".to_string();
        let endpoint = &state.config.leaderboard.url;
        let date = chrono_date_string();
        if endpoint.is_empty() {
            state.daily_status = "No leaderboard endpoint configured.".to_string();
        } else {
            match chaos_rpg_core::daily_leaderboard::fetch_scores(endpoint, &date) {
                Ok(rows) => {
                    state.daily_status = format!("Updated - {} entries", rows.len());
                    state.daily_rows = rows;
                }
                Err(e) => {
                    state.daily_status = format!("Error: {}", e);
                }
            }
        }
    }
    if esc || enter { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    ui_render::screen_backing(engine, 0.6);

    ui_render::heading_centered(engine, "DAILY LEADERBOARD", 5.0, theme.heading);
    let status_trunc: String = state.daily_status.chars().take(45).collect();
    ui_render::small(engine, &status_trunc, -6.0, 4.2, theme.dim);

    // Column headers
    ui_render::small(engine, "Rank  Name            Class       Floor  Score", -8.0, 3.5, theme.muted);

    if state.daily_rows.is_empty() {
        ui_render::body(engine, "No scores yet. Press [R] to refresh.", -6.5, 1.0, theme.dim);
    } else {
        for (i, row) in state.daily_rows.iter().enumerate().take(16) {
            let name_trunc: String = row.name.chars().take(14).collect();
            let class_trunc: String = row.class.chars().take(10).collect();
            let line = format!("{:>4}  {:14}  {:10}  {:>5}  {:>8}",
                row.rank, name_trunc, class_trunc, row.floor, row.score);
            let color = if i < 3 { theme.gold } else { theme.primary };
            let em = if i < 3 { 0.6 } else { 0.35 };
            ui_render::text(engine, &line, -8.0, 2.8 - i as f32 * 0.45, color, 0.25, em);
        }
    }

    ui_render::small(engine, "[R] Refresh  [Enter/Space/Esc] Back", -5.5, -5.2, theme.muted);
}

/// Simple date string (YYYY-MM-DD) without external chrono dependency.
fn chrono_date_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
}
