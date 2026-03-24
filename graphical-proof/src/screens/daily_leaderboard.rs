//! Daily leaderboard — today's seeded challenge rankings.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let refresh = engine.input.just_pressed(Key::R);

    if refresh {
        state.daily_status = "Fetching...".to_string();
        // Attempt to fetch from configured endpoint
        let endpoint = &state.config.leaderboard.url;
        let date = chrono_date_string();
        if endpoint.is_empty() {
            state.daily_status = "No leaderboard endpoint configured.".to_string();
        } else {
            match chaos_rpg_core::daily_leaderboard::fetch_scores(endpoint, &date) {
                Ok(rows) => {
                    state.daily_status = format!("Updated — {} entries", rows.len());
                    state.daily_rows = rows;
                }
                Err(e) => {
                    state.daily_status = format!("Error: {}", e);
                }
            }
        }
    }
    if esc { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    render_text(engine, "DAILY LEADERBOARD", -5.5, 9.0, theme.heading, 0.9);
    render_text(engine, &state.daily_status, -8.0, 7.5, theme.dim, 0.3);

    // Column headers
    render_text(engine, "Rank  Name              Class       Floor  Score", -18.0, 6.5, theme.muted, 0.3);

    if state.daily_rows.is_empty() {
        render_text(engine, "No scores yet. Press [R] to refresh.", -10.0, 4.0, theme.dim, 0.3);
    } else {
        for (i, row) in state.daily_rows.iter().enumerate().take(20) {
            let line = format!("{:>4}  {:16}  {:10}  {:>5}  {:>8}",
                row.rank, row.name, row.class, row.floor, row.score);
            let truncated: String = line.chars().take(60).collect();
            let color = if i < 3 { theme.gold } else { theme.primary };
            render_text(engine, &truncated, -18.0, 5.0 - i as f32 * 0.9, color, if i < 3 { 0.6 } else { 0.35 });
        }
    }

    render_text(engine, "[R] Refresh  [Esc] Back", -18.0, -12.0, theme.muted, 0.2);
}

/// Simple date string (YYYY-MM-DD) without external chrono dependency.
fn chrono_date_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    // Approximate — good enough for daily seed matching
    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
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
