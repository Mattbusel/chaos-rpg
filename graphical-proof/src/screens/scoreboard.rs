//! Scoreboard — Hall of Chaos + Hall of Misery.
//! Sortable columns, personal best highlight, themed display.

use proof_engine::prelude::*;
use chaos_rpg_core::scoreboard::{load_scores, load_misery_scores};
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Space)
        || engine.input.just_pressed(Key::Enter);
    if esc { state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    let scores = load_scores();
    let misery_scores = load_misery_scores();

    // ── Hall of Chaos (left panel) ──
    let lx = -8.2;
    ui_render::text(engine, "HALL OF CHAOS", lx, 4.8, theme.heading, 0.4, 0.8);

    // Column headers
    ui_render::text(engine, "Rank Name           Score  Floor", lx, 4.0, theme.muted, 0.2, 0.2);
    ui_render::text(engine, "------------------------------", lx, 3.7, theme.border, 0.18, 0.1);

    if scores.is_empty() {
        ui_render::small(engine, "No scores yet.", lx + 1.0, 2.5, theme.dim);
    } else {
        for (i, entry) in scores.iter().enumerate().take(12) {
            let y = 3.2 - i as f32 * 0.45;

            let (color, em) = match i {
                0 => (theme.gold, 0.7),
                1 => (theme.accent, 0.55),
                2 => (theme.warn, 0.45),
                _ => (theme.primary, 0.3),
            };

            // Medal for top 3
            if i < 3 {
                let medal_pulse = ((frame as f32 * 0.06 + i as f32 * 1.3).sin() * 0.2 + 0.8).max(0.0);
                let medal_chars = ['1', '2', '3'];
                engine.spawn_glyph(Glyph {
                    character: medal_chars[i],
                    position: Vec3::new(lx - 0.4, y, 0.0),
                    color: Vec4::new(color.x * medal_pulse, color.y * medal_pulse, color.z * medal_pulse, medal_pulse),
                    emission: medal_pulse * 0.8,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }

            let name_trunc: String = entry.name.chars().take(12).collect();
            let line = format!("{:>3}  {:12} {:>6} {:>4}", i + 1, name_trunc, entry.score, entry.floor_reached);
            let truncated: String = line.chars().take(35).collect();
            ui_render::text(engine, &truncated, lx, y, color, 0.2, em);

            // Personal best highlight (first entry assumed to be personal best for demo)
            if i == 0 {
                let glow = ((frame as f32 * 0.04).sin() * 0.15 + 0.85).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: '*',
                    position: Vec3::new(lx + 8.5, y, 0.0),
                    color: Vec4::new(theme.gold.x * glow, theme.gold.y * glow, 0.0, glow),
                    emission: glow * 0.6,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }
    }

    // ── Hall of Misery (right panel) ──
    let rx = 1.0;
    ui_render::text(engine, "HALL OF MISERY", rx, 4.8, theme.danger, 0.4, 0.8);

    // Column headers
    ui_render::text(engine, "Rank Name           Misery", rx, 4.0, theme.muted, 0.2, 0.2);
    ui_render::text(engine, "-------------------------", rx, 3.7, theme.border, 0.18, 0.1);

    if misery_scores.is_empty() {
        ui_render::small(engine, "No misery yet.", rx + 1.0, 2.5, theme.dim);
    } else {
        for (i, entry) in misery_scores.iter().enumerate().take(12) {
            let y = 3.2 - i as f32 * 0.45;

            let (color, em) = match i {
                0 => (theme.danger, 0.6),
                1..=2 => (theme.warn, 0.45),
                _ => (theme.dim, 0.25),
            };

            let name_trunc: String = entry.name.chars().take(12).collect();
            let line = format!("{:>3}  {:12} {:>8.0}", i + 1, name_trunc, entry.misery_index);
            let truncated: String = line.chars().take(30).collect();
            ui_render::text(engine, &truncated, rx, y, color, 0.2, em);
        }
    }

    // ── Decorative divider between panels ──
    for i in 0..14 {
        let pulse = ((frame as f32 * 0.03 + i as f32 * 0.4).sin() * 0.15 + 0.3).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '|',
            position: Vec3::new(0.5, 4.8 - i as f32 * 0.45, 0.0),
            color: Vec4::new(theme.border.x * pulse, theme.border.y * pulse, theme.border.z * pulse, pulse),
            emission: pulse * 0.2,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // ── Footer ──
    ui_render::small(engine, "[Enter/Esc/Space] Back", -3.5, -5.2, theme.muted);
}
