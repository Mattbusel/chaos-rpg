//! Scoreboard — Hall of Chaos + Hall of Misery.

use proof_engine::prelude::*;
use chaos_rpg_core::scoreboard::{load_scores, load_misery_scores};
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q) || engine.input.just_pressed(Key::Enter);
    if esc { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Load scores
    let scores = load_scores();
    let misery_scores = load_misery_scores();

    // ── Hall of Chaos (left panel) ──
    render_text(engine, "HALL OF CHAOS", -18.0, 9.0, theme.heading, 0.9);
    render_text(engine, "Rank  Name              Score     Floor", -18.0, 7.5, theme.muted, 0.3);
    render_text(engine, "─".repeat(40).as_str(), -18.0, 7.0, theme.muted, 0.15);

    for (i, entry) in scores.iter().enumerate().take(15) {
        let color = match i {
            0 => theme.gold,
            1..=2 => theme.accent,
            _ => theme.primary,
        };
        let line = format!("{:>4}  {:16}  {:>8}  {:>5}", i + 1, entry.name, entry.score, entry.floor_reached);
        render_text(engine, &line, -18.0, 6.0 - i as f32 * 0.9, color, if i < 3 { 0.6 } else { 0.35 });
    }

    if scores.is_empty() {
        render_text(engine, "No scores yet.", -16.0, 5.0, theme.dim, 0.3);
    }

    // ── Hall of Misery (right panel) ──
    render_text(engine, "HALL OF MISERY", 2.0, 9.0, theme.danger, 0.9);
    render_text(engine, "Rank  Name              Misery", 2.0, 7.5, theme.muted, 0.3);
    render_text(engine, "─".repeat(35).as_str(), 2.0, 7.0, theme.muted, 0.15);

    for (i, entry) in misery_scores.iter().enumerate().take(15) {
        let color = match i {
            0 => theme.danger,
            1..=2 => theme.warn,
            _ => theme.dim,
        };
        let line = format!("{:>4}  {:16}  {:>10.0}", i + 1, entry.name, entry.misery_index);
        render_text(engine, &line, 2.0, 6.0 - i as f32 * 0.9, color, if i < 3 { 0.5 } else { 0.25 });
    }

    if misery_scores.is_empty() {
        render_text(engine, "No misery yet.", 4.0, 5.0, theme.dim, 0.3);
    }

    render_text(engine, "[Enter/Esc] Back", -5.0, -12.0, theme.muted, 0.2);
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
