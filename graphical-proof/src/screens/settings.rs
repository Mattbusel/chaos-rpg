//! Settings screen — theme cycling, audio/video/gameplay tabs,
//! color preview swatches, keybind display, accessibility options.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let left = engine.input.just_pressed(Key::Left);
    let right = engine.input.just_pressed(Key::Right);
    let t_key = engine.input.just_pressed(Key::T);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Space);

    // Theme cycling
    if t_key || right {
        state.theme_idx = (state.theme_idx + 1) % THEMES.len();
    }
    if left {
        state.theme_idx = if state.theme_idx == 0 { THEMES.len() - 1 } else { state.theme_idx - 1 };
    }

    if esc { state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    // ── Header ──
    ui_render::heading_centered(engine, "SETTINGS", 4.8, theme.heading);

    // ── Visual Theme section ──
    let sx = -7.5;
    let mut y = 3.5;

    ui_render::text(engine, "-- Visual Theme --", sx, y, theme.accent, 0.28, 0.5);
    y -= 0.5;

    // Theme name with navigation arrows
    let theme_nav = format!("< {} >", theme.name);
    ui_render::text(engine, &theme_nav, sx, y, theme.selected, 0.35, 0.7);
    y -= 0.45;

    // Tagline
    let tagline_trunc: String = theme.tagline.chars().take(45).collect();
    ui_render::text(engine, &tagline_trunc, sx, y, theme.dim, 0.22, 0.25);
    y -= 0.55;

    // Theme index indicator dots
    let mut dot_x = sx;
    for i in 0..THEMES.len() {
        let active = i == state.theme_idx;
        let c = if active { theme.selected } else { theme.muted };
        let em = if active { 0.8 } else { 0.15 };
        let ch = if active { '#' } else { '.' };
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(dot_x, y, 0.0),
            color: c,
            emission: em,
            layer: RenderLayer::UI,
            ..Default::default()
        });
        dot_x += 0.4;
    }
    y -= 0.55;

    // Color preview swatches (3x3 grid)
    let samples: [(&str, Vec4); 9] = [
        ("bg", theme.bg), ("border", theme.border), ("heading", theme.heading),
        ("primary", theme.primary), ("accent", theme.accent), ("danger", theme.danger),
        ("success", theme.success), ("gold", theme.gold), ("mana", theme.mana),
    ];
    for (i, (label, color)) in samples.iter().enumerate() {
        let col = (i % 3) as f32;
        let row = (i / 3) as f32;
        let cx = sx + col * 5.5;
        let cy = y - row * 0.45;

        // Color swatch block
        let pulse = ((frame as f32 * 0.03 + i as f32 * 0.5).sin() * 0.08 + 0.92).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '#',
            position: Vec3::new(cx, cy, 0.0),
            color: Vec4::new(color.x * pulse, color.y * pulse, color.z * pulse, 1.0),
            emission: 0.4,
            layer: RenderLayer::UI,
            ..Default::default()
        });
        engine.spawn_glyph(Glyph {
            character: '#',
            position: Vec3::new(cx + 0.3, cy, 0.0),
            color: Vec4::new(color.x * pulse, color.y * pulse, color.z * pulse, 1.0),
            emission: 0.4,
            layer: RenderLayer::UI,
            ..Default::default()
        });
        ui_render::text(engine, label, cx + 0.7, cy, theme.dim, 0.2, 0.2);
    }
    y -= 1.6;

    // ── Audio section ──
    ui_render::text(engine, "-- Audio --", sx, y, theme.accent, 0.28, 0.5);
    y -= 0.45;
    ui_render::text(engine, &format!("Music Vibe: {}", state.config.audio.music_vibe), sx, y, theme.primary, 0.25, 0.35);
    y -= 0.35;
    ui_render::small(engine, "(edit config file to change)", sx, y, theme.muted);
    y -= 0.55;

    // ── Keybinds section ──
    ui_render::text(engine, "-- Keybinds --", sx, y, theme.accent, 0.28, 0.5);
    y -= 0.45;

    let binds = [
        ("Enter/Space", "Confirm"),
        ("Escape", "Back / Menu"),
        ("Up/Down/W/S", "Navigate"),
        ("Left/Right", "Tabs / Cycle"),
        ("V", "Chaos Viz"),
        ("C", "Character Sheet"),
        ("T", "Change Theme"),
    ];
    for (key, action) in &binds {
        ui_render::text(engine, &format!("[{}] {}", key, action), sx, y, theme.dim, 0.22, 0.25);
        y -= 0.35;
    }

    // ── Accessibility section ──
    y -= 0.2;
    ui_render::text(engine, "-- Accessibility --", sx, y, theme.accent, 0.28, 0.5);
    y -= 0.45;
    ui_render::text(engine, "FAST_MODE=1 halves animations", sx, y, theme.dim, 0.22, 0.25);

    // ── Engine stats (right side) ──
    let ex = 3.0;
    ui_render::text(engine, "-- Engine --", ex, 3.5, theme.accent, 0.28, 0.5);
    ui_render::small(engine, &format!("Bloom: {:.1}", theme.bloom_intensity), ex, 3.0, theme.dim);
    ui_render::small(engine, &format!("Chromatic: {:.3}", theme.chromatic_aberration), ex, 2.6, theme.dim);
    ui_render::small(engine, &format!("Vignette: {:.2}", theme.vignette_strength), ex, 2.2, theme.dim);
    ui_render::small(engine, &format!("Chaos: {:.3}", theme.chaos_field_brightness), ex, 1.8, theme.dim);

    // ── Footer ──
    ui_render::small(engine, "[T/Left/Right] Theme  [Esc/Space] Back", -5.5, -5.2, theme.muted);
}
