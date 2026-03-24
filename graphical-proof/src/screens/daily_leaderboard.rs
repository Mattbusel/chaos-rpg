//! Daily leaderboard — today's seeded challenge rankings,
//! ranked list with player highlight, score breakdown.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Space);
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
    if esc { state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    // ── Header ──
    ui_render::heading_centered(engine, "DAILY LEADERBOARD", 4.8, theme.heading);

    // Status line
    let status_color = if state.daily_status.contains("Error") { theme.danger } else { theme.dim };
    ui_render::text_centered(engine, &state.daily_status, 3.8, status_color, 0.25, 0.3);

    // Date display
    let date_str = chrono_date_string();
    ui_render::small(engine, &format!("Date: {}", date_str), -8.0, 3.3, theme.muted);

    ui_render::text_centered(engine, "================================", 2.8, theme.border, 0.22, 0.12);

    // ── Column headers ──
    ui_render::text(engine, "Rank Name            Class     Floor Score", -8.2, 2.4, theme.muted, 0.22, 0.2);

    if state.daily_rows.is_empty() {
        ui_render::text_centered(engine, "No scores yet. Press [R] to refresh.", 0.5, theme.dim, 0.3, 0.3);
    } else {
        for (i, row) in state.daily_rows.iter().enumerate().take(16) {
            let y = 1.8 - i as f32 * 0.48;

            // Rank medal colors for top 3
            let (color, em) = match i {
                0 => (theme.gold, 0.7),
                1 => (theme.accent, 0.55),
                2 => (theme.warn, 0.5),
                _ => (theme.primary, 0.3),
            };

            // Medal icon for top 3
            if i < 3 {
                let medal_chars = ['1', '2', '3'];
                let medal_pulse = ((frame as f32 * 0.06 + i as f32 * 1.5).sin() * 0.2 + 0.8).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: medal_chars[i],
                    position: Vec3::new(-8.6, y, 0.0),
                    color: Vec4::new(color.x * medal_pulse, color.y * medal_pulse, color.z * medal_pulse, medal_pulse),
                    emission: medal_pulse * 0.8,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }

            let name_trunc: String = row.name.chars().take(14).collect();
            let class_trunc: String = row.class.chars().take(8).collect();
            let line = format!("{:>3}  {:14} {:8} {:>4} {:>7}",
                row.rank, name_trunc, class_trunc, row.floor, row.score);
            let truncated: String = line.chars().take(48).collect();
            ui_render::text(engine, &truncated, -8.2, y, color, 0.22, em);

            // Won indicator
            if row.won {
                let sparkle = ((frame as f32 * 0.08 + i as f32).sin() * 0.3 + 0.7).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: '*',
                    position: Vec3::new(8.0, y, 0.0),
                    color: Vec4::new(theme.success.x * sparkle, theme.success.y * sparkle, theme.success.z * sparkle, sparkle),
                    emission: sparkle * 0.5,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }
    }

    // ── Score breakdown for selected ──
    if let Some(first) = state.daily_rows.first() {
        ui_render::text(engine, "Top Score Breakdown:", 4.5, -2.5, theme.accent, 0.25, 0.4);
        ui_render::small(engine, &format!("Name: {}", first.name), 4.5, -3.0, theme.primary);
        ui_render::small(engine, &format!("Kills: {}", first.kills), 4.5, -3.4, theme.dim);
        ui_render::small(engine, &format!("Floor: {}", first.floor), 4.5, -3.7, theme.dim);
        ui_render::small(engine, &format!("Score: {}", first.score), 4.5, -4.0, theme.gold);
    }

    // ── Footer ──
    ui_render::small(engine, "[R] Refresh  [Esc/Space] Back", -5.0, -5.2, theme.muted);
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
