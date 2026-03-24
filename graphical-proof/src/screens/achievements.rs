//! Achievements browser — scrollable list with filter (All/Unlocked/Locked).

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let pgup = engine.input.just_pressed(Key::PageUp);
    let pgdn = engine.input.just_pressed(Key::PageDown);
    let tab = engine.input.just_pressed(Key::Tab) || engine.input.just_pressed(Key::F);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);

    let total = state.achievements.achievements.len();
    if up && state.achievement_scroll > 0 { state.achievement_scroll -= 1; }
    if down && state.achievement_scroll < total.saturating_sub(1) { state.achievement_scroll += 1; }
    if pgup { state.achievement_scroll = state.achievement_scroll.saturating_sub(15); }
    if pgdn { state.achievement_scroll = (state.achievement_scroll + 15).min(total.saturating_sub(1)); }
    if tab { state.achievement_filter = (state.achievement_filter + 1) % 3; state.achievement_scroll = 0; }
    if esc { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    let filter_name = match state.achievement_filter {
        0 => "All",
        1 => "Unlocked",
        _ => "Locked",
    };

    let unlocked_count = state.achievements.unlocked_count();
    let total = state.achievements.total_count();

    let header = format!("ACHIEVEMENTS — {}/{} unlocked — Filter: [{}]", unlocked_count, total, filter_name);
    render_text(engine, &header, -18.0, 9.0, theme.heading, 0.8);

    // Filter achievements
    let filtered: Vec<&chaos_rpg_core::achievements::Achievement> = state.achievements.achievements.iter()
        .filter(|a| match state.achievement_filter {
            1 => a.unlocked,
            2 => !a.unlocked,
            _ => true,
        })
        .collect();

    if filtered.is_empty() {
        render_text(engine, "No achievements match the current filter.", -12.0, 4.0, theme.dim, 0.3);
    } else {
        let start = state.achievement_scroll.saturating_sub(8);
        let end = (start + 18).min(filtered.len());
        for (di, idx) in (start..end).enumerate() {
            let ach = filtered[idx];
            let is_selected = idx == state.achievement_scroll;

            let (icon, color, emission) = if is_selected {
                ("> ", theme.selected, 0.8)
            } else if ach.unlocked {
                ("★ ", theme.success, 0.4)
            } else {
                ("  ", theme.dim, 0.2)
            };

            let line = format!("{}{} — {}", icon, ach.name, ach.description);
            let truncated: String = line.chars().take(65).collect();
            render_text(engine, &truncated, -18.0, 7.0 - di as f32 * 0.95, color, emission);
        }
    }

    render_text(engine, "[Up/Down] Scroll  [PgUp/PgDn] Page  [Tab/F] Filter  [Esc] Back",
        -18.0, -12.0, theme.muted, 0.2);
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
