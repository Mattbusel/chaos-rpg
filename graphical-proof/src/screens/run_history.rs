//! Run history — scrollable list of past runs with detail panel,
//! floor reached, cause of death, date, stats breakdown.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Space);

    let total = state.run_history.runs.len();
    if up && state.history_scroll > 0 { state.history_scroll -= 1; }
    if down && state.history_scroll < total.saturating_sub(1) { state.history_scroll += 1; }
    if esc { state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    // ── Header ──
    ui_render::heading_centered(engine, "RUN HISTORY", 4.8, theme.heading);
    ui_render::text_centered(engine, &format!("{} runs recorded", state.run_history.runs.len()), 4.0, theme.dim, 0.25, 0.3);

    if state.run_history.runs.is_empty() {
        ui_render::text_centered(engine, "No runs recorded yet.", 1.5, theme.dim, 0.35, 0.3);
        ui_render::text_centered(engine, "Go die a few times.", 0.8, theme.muted, 0.25, 0.2);
    } else {
        // ── Left panel: run list ──
        let start = state.history_scroll.saturating_sub(6);
        let end = (start + 14).min(state.run_history.runs.len());

        ui_render::text_centered(engine, "--------------------------------", 3.3, theme.border, 0.22, 0.12);

        for (di, idx) in (start..end).enumerate() {
            let run = &state.run_history.runs[idx];
            let selected = idx == state.history_scroll;
            let color = if selected { theme.selected } else { theme.primary };
            let em = if selected { 0.7 } else { 0.3 };
            let prefix = if selected { "> " } else { "  " };
            let won_marker = if run.won { "#" } else { "x" };
            let line = format!("{}{} {} {} Lv.{} F{}", prefix, won_marker, run.name, run.class, run.level, run.floor);
            let truncated: String = line.chars().take(32).collect();
            ui_render::text(engine, &truncated, -8.2, 2.8 - di as f32 * 0.5, color, 0.25, em);

            // Won/died indicator dot
            let dot_color = if run.won { theme.success } else { theme.danger };
            let dot_pulse = if selected { ((frame as f32 * 0.08).sin() * 0.3 + 0.7).max(0.0) } else { 0.5 };
            engine.spawn_glyph(Glyph {
                character: if run.won { '*' } else { '.' },
                position: Vec3::new(-8.6, 2.8 - di as f32 * 0.5, 0.0),
                color: Vec4::new(dot_color.x * dot_pulse, dot_color.y * dot_pulse, dot_color.z * dot_pulse, dot_pulse),
                emission: dot_pulse * 0.5,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // ── Right panel: selected run detail ──
        if let Some(run) = state.run_history.runs.get(state.history_scroll) {
            let px = 1.2;
            let mut py = 3.0;

            // Character header
            let header = format!("{} - {}", run.name, run.class);
            let truncated: String = header.chars().take(28).collect();
            ui_render::text(engine, &truncated, px, py, theme.heading, 0.32, 0.6);
            py -= 0.5;

            // Mode and difficulty
            ui_render::small(engine, &format!("{} | {}", run.difficulty, run.game_mode), px, py, theme.dim);
            py -= 0.4;

            // Core stats
            ui_render::small(engine, &format!("Floor: {}  Lv: {}  Kills: {}", run.floor, run.level, run.kills), px, py, theme.primary);
            py -= 0.4;
            ui_render::small(engine, &format!("Score: {}  Gold: {}", run.score, run.gold), px, py, theme.gold);
            py -= 0.4;
            ui_render::small(engine, &format!("Dmg: {} dealt / {} taken", run.damage_dealt, run.damage_taken), px, py, theme.dim);
            py -= 0.4;
            ui_render::small(engine, &format!("Tier: {}  Corr: {}", run.power_tier, run.corruption), px, py, theme.accent);
            py -= 0.55;

            // Cause of death
            if !run.cause_of_death.is_empty() {
                let cause: String = run.cause_of_death.chars().take(30).collect();
                ui_render::text(engine, &format!("Cause: {}", cause), px, py, theme.danger, 0.27, 0.5);
                py -= 0.45;
            }

            // Date
            ui_render::small(engine, &run.date, px, py, theme.muted);
            py -= 0.55;

            // Auto-narrative excerpt
            if !run.auto_narrative.is_empty() {
                let excerpt: String = run.auto_narrative.chars().take(35).collect();
                ui_render::small(engine, &excerpt, px, py, theme.dim);
            }
        }
    }

    // ── Footer ──
    ui_render::small(engine, "[Up/Down] Scroll  [Esc/Space] Back", -5.5, -5.2, theme.muted);
}
