//! Settings screen — music vibe, theme, accessibility options.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let left = engine.input.just_pressed(Key::Left);
    let right = engine.input.just_pressed(Key::Right);
    let t_key = engine.input.just_pressed(Key::T);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);

    // Theme cycling
    if t_key || right {
        state.theme_idx = (state.theme_idx + 1) % THEMES.len();
    }
    if left {
        state.theme_idx = if state.theme_idx == 0 { THEMES.len() - 1 } else { state.theme_idx - 1 };
    }

    if esc { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    render_text(engine, "SETTINGS", -3.0, 9.0, theme.heading, 0.9);

    // Theme
    render_text(engine, "Visual Theme:", -12.0, 6.0, theme.primary, 0.5);
    render_text(engine, &format!("[T/Left/Right]  {}", theme.name), -12.0, 4.5, theme.selected, 0.7);
    render_text(engine, theme.tagline, -12.0, 3.5, theme.dim, 0.3);

    // Theme preview colors
    let preview_y = 1.5;
    let samples = [
        ("bg", theme.bg), ("border", theme.border), ("heading", theme.heading),
        ("primary", theme.primary), ("accent", theme.accent), ("danger", theme.danger),
        ("success", theme.success), ("gold", theme.gold), ("mana", theme.mana),
    ];
    for (i, (label, color)) in samples.iter().enumerate() {
        let x = -12.0 + (i % 3) as f32 * 8.0;
        let y = preview_y - (i / 3) as f32 * 1.2;
        render_text(engine, &format!("██ {}", label), x, y, *color, 0.6);
    }

    // Audio (informational — actual vibe set in config)
    render_text(engine, "Audio:", -12.0, -3.0, theme.primary, 0.5);
    render_text(engine, &format!("Music Vibe: {}", state.config.audio.music_vibe), -12.0, -4.2, theme.dim, 0.35);

    // Accessibility
    render_text(engine, "Accessibility:", -12.0, -6.0, theme.primary, 0.5);
    render_text(engine, "Set FAST_MODE=1 env var to halve all animation durations", -12.0, -7.2, theme.dim, 0.3);

    render_text(engine, "[T/Left/Right] Cycle Theme  [Esc] Back", -12.0, -12.0, theme.muted, 0.2);
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
