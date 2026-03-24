//! Settings screen — theme, audio, accessibility options.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let left = engine.input.just_pressed(Key::Left);
    let right = engine.input.just_pressed(Key::Right);
    let t_key = engine.input.just_pressed(Key::T);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);

    if t_key || right {
        state.theme_idx = (state.theme_idx + 1) % THEMES.len();
    }
    if left {
        state.theme_idx = if state.theme_idx == 0 { THEMES.len() - 1 } else { state.theme_idx - 1 };
    }

    if esc || enter { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    ui_render::heading_centered(engine, "SETTINGS", 5.0, theme.heading);

    // Theme
    ui_render::body(engine, "Visual Theme:", -7.0, 3.5, theme.primary);
    ui_render::body(engine, &format!("[T/Left/Right]  {}", theme.name), -7.0, 2.8, theme.selected);
    ui_render::small(engine, theme.tagline, -7.0, 2.2, theme.dim);

    // Theme preview colors
    let samples = [
        ("bg", theme.bg), ("border", theme.border), ("heading", theme.heading),
        ("primary", theme.primary), ("accent", theme.accent), ("danger", theme.danger),
        ("success", theme.success), ("gold", theme.gold), ("mana", theme.mana),
    ];
    for (i, (label, color)) in samples.iter().enumerate() {
        let x = -7.0 + (i % 3) as f32 * 5.5;
        let y = 1.2 - (i / 3) as f32 * 0.45;
        ui_render::small(engine, &format!("## {}", label), x, y, *color);
    }

    // Audio
    ui_render::body(engine, "Audio:", -7.0, -0.5, theme.primary);
    ui_render::small(engine, &format!("Music Vibe: {}", state.config.audio.music_vibe), -7.0, -1.1, theme.dim);

    // Accessibility
    ui_render::body(engine, "Accessibility:", -7.0, -2.0, theme.primary);
    ui_render::small(engine, "Set FAST_MODE=1 to halve animation times", -7.0, -2.6, theme.dim);

    ui_render::small(engine, "[T/Left/Right] Cycle Theme  [Enter/Space/Esc] Back", -7.5, -5.2, theme.muted);
}
