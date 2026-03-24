//! Achievements browser — grid view with progress bars, unlock animations,
//! category filter (All/Unlocked/Locked).

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let pgup = engine.input.just_pressed(Key::PageUp);
    let pgdn = engine.input.just_pressed(Key::PageDown);
    let tab = engine.input.just_pressed(Key::Tab) || engine.input.just_pressed(Key::F);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Space);

    let total = state.achievements.achievements.len();
    if up && state.achievement_scroll > 0 { state.achievement_scroll -= 1; }
    if down && state.achievement_scroll < total.saturating_sub(1) { state.achievement_scroll += 1; }
    if pgup { state.achievement_scroll = state.achievement_scroll.saturating_sub(15); }
    if pgdn { state.achievement_scroll = (state.achievement_scroll + 15).min(total.saturating_sub(1)); }
    if tab { state.achievement_filter = (state.achievement_filter + 1) % 3; state.achievement_scroll = 0; }
    if esc { state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    let filter_name = match state.achievement_filter {
        0 => "All",
        1 => "Unlocked",
        _ => "Locked",
    };

    let unlocked_count = state.achievements.unlocked_count();
    let total = state.achievements.total_count();

    // ── Header ──
    ui_render::heading_centered(engine, "ACHIEVEMENTS", 4.8, theme.heading);

    // Progress summary
    let summary = format!("{}/{} unlocked | Filter: {}", unlocked_count, total, filter_name);
    ui_render::text_centered(engine, &summary, 4.0, theme.dim, 0.25, 0.3);

    // Overall progress bar
    let overall_pct = if total > 0 { unlocked_count as f32 / total as f32 } else { 0.0 };
    ui_render::bar(engine, -3.5, 3.5, 7.0, overall_pct, theme.gold, theme.muted, 0.25);
    ui_render::small(engine, &format!("{}%", (overall_pct * 100.0) as u32), 4.0, 3.5, theme.gold);

    // Filter tabs
    let filters = ["[All]", "[Unlocked]", "[Locked]"];
    let mut fx = -5.0;
    for (i, f) in filters.iter().enumerate() {
        let active = i == state.achievement_filter as usize;
        let c = if active { theme.selected } else { theme.dim };
        let em = if active { 0.7 } else { 0.2 };
        ui_render::text(engine, f, fx, 3.0, c, 0.25, em);
        fx += f.len() as f32 * 0.25 * 0.85 + 0.8;
    }

    ui_render::text_centered(engine, "--------------------------------", 2.6, theme.border, 0.22, 0.12);

    // ── Filter achievements ──
    let filtered: Vec<(usize, &chaos_rpg_core::achievements::Achievement)> = state.achievements.achievements.iter()
        .enumerate()
        .filter(|(_, a)| match state.achievement_filter {
            1 => a.unlocked,
            2 => !a.unlocked,
            _ => true,
        })
        .collect();

    if filtered.is_empty() {
        ui_render::text_centered(engine, "No achievements match this filter.", 0.5, theme.dim, 0.3, 0.3);
    } else {
        let start = state.achievement_scroll.saturating_sub(6);
        let end = (start + 14).min(filtered.len());
        for (di, &(_, ach)) in filtered[start..end].iter().enumerate() {
            let real_idx = start + di;
            let is_selected = real_idx == state.achievement_scroll;

            let y = 2.0 - di as f32 * 0.55;

            // Icon
            let (icon, icon_color) = if ach.unlocked {
                ('#', theme.success)
            } else {
                ('.', theme.muted)
            };

            // Unlock animation for selected unlocked achievement
            let icon_em = if is_selected && ach.unlocked {
                ((frame as f32 * 0.1).sin() * 0.3 + 0.7).max(0.0)
            } else if ach.unlocked {
                0.4
            } else {
                0.1
            };

            engine.spawn_glyph(Glyph {
                character: icon,
                position: Vec3::new(-8.2, y, 0.0),
                color: Vec4::new(icon_color.x * icon_em, icon_color.y * icon_em, icon_color.z * icon_em, 1.0),
                emission: icon_em * 0.8,
                layer: RenderLayer::UI,
                ..Default::default()
            });

            // Achievement name and description
            let color = if is_selected { theme.selected } else if ach.unlocked { theme.primary } else { theme.dim };
            let em = if is_selected { 0.7 } else { 0.3 };
            let prefix = if is_selected { "> " } else { "  " };
            let rarity_tag: String = format!("{:?}", ach.rarity).chars().take(4).collect();
            let line = format!("{}{} [{}]", prefix, ach.name, rarity_tag);
            let truncated: String = line.chars().take(30).collect();
            ui_render::text(engine, &truncated, -7.8, y, color, 0.27, em);

            // Description on the right for selected
            if is_selected {
                let desc: String = ach.description.chars().take(30).collect();
                ui_render::small(engine, &desc, 1.5, y, theme.dim);
            }
        }
    }

    // ── Selected achievement detail panel ──
    if !filtered.is_empty() {
        let sel_idx = state.achievement_scroll.min(filtered.len().saturating_sub(1));
        if let Some(&(_, ach)) = filtered.get(sel_idx) {
            let px = 1.5;
            let py = -2.5;

            ui_render::text(engine, "+----Detail----+", px - 0.2, py + 0.4, theme.border, 0.22, 0.2);
            ui_render::text(engine, &ach.name, px, py, theme.heading, 0.3, 0.6);
            ui_render::small(engine, &ach.description, px, py - 0.4, theme.dim);
            ui_render::small(engine, &format!("Rarity: {:?}", ach.rarity), px, py - 0.75, theme.accent);
            let status = if ach.unlocked { "UNLOCKED" } else { "Locked" };
            let status_c = if ach.unlocked { theme.success } else { theme.muted };
            ui_render::small(engine, status, px, py - 1.05, status_c);
        }
    }

    // ── Decorative sparkles for unlocked achievements ──
    if unlocked_count > 0 {
        for i in 0..4u32 {
            let seed_f = i as f32 * 37.1 + frame as f32 * 0.03;
            let px = -6.0 + seed_f.sin() * 7.0;
            let py = -4.0 + (seed_f * 0.7).cos() * 0.8;
            let sparkle = ((frame as f32 * 0.07 + i as f32 * 1.5).sin() * 0.4 + 0.5).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '+',
                position: Vec3::new(px, py, 0.0),
                color: Vec4::new(theme.gold.x * sparkle, theme.gold.y * sparkle, 0.0, sparkle * 0.5),
                emission: sparkle * 0.3,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    // ── Footer ──
    ui_render::small(engine, "[Up/Down] Scroll  [PgUp/Dn] Page  [Tab/F] Filter  [Esc/Space] Back", -8.0, -5.2, theme.muted);
}
