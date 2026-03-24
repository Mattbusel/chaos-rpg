//! Bestiary — enemy catalog with rotating enemy preview, stats, drop tables.

use proof_engine::prelude::*;
use chaos_rpg_core::enemy::generate_enemy;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

/// Generate a static preview of enemies at various floor levels.
fn generate_preview() -> Vec<(String, String, u32)> {
    let mut entries = Vec::new();
    let seed_base = 42u64;
    for floor in [1, 5, 10, 15, 20, 25, 30, 40, 50, 60, 75, 90, 100] {
        for i in 0..3u64 {
            let enemy = generate_enemy(floor, seed_base.wrapping_add(floor as u64 * 10 + i));
            let tier_str = format!("{:?}", enemy.tier);
            if !entries.iter().any(|(n, _, _): &(String, String, u32)| n == &enemy.name) {
                entries.push((enemy.name, tier_str, floor));
            }
        }
    }
    entries
}

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Space);

    let entries = generate_preview();
    let total = entries.len();
    if up && state.bestiary_scroll > 0 { state.bestiary_scroll -= 1; }
    if down && state.bestiary_scroll < total.saturating_sub(1) { state.bestiary_scroll += 1; }
    if esc { state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let entries = generate_preview();
    let frame = state.frame;

    // ── Header ──
    ui_render::heading_centered(engine, "BESTIARY", 4.8, theme.heading);
    ui_render::text_centered(engine, &format!("{} enemies catalogued", entries.len()), 4.0, theme.dim, 0.25, 0.3);

    ui_render::text_centered(engine, "================================", 3.3, theme.border, 0.22, 0.12);

    if entries.is_empty() {
        ui_render::text_centered(engine, "No enemies in the bestiary.", 1.0, theme.dim, 0.3, 0.3);
    } else {
        // ── Left panel: enemy list ──
        let start = state.bestiary_scroll.saturating_sub(6);
        let end = (start + 14).min(entries.len());
        for (di, idx) in (start..end).enumerate() {
            let (ref name, ref tier, floor) = entries[idx];
            let selected = idx == state.bestiary_scroll;
            let color = if selected { theme.selected } else { theme.primary };
            let em = if selected { 0.7 } else { 0.3 };
            let prefix = if selected { "> " } else { "  " };
            let tier_short: String = tier.chars().take(6).collect();
            let line = format!("{}{} ({} F{})", prefix, name, tier_short, floor);
            let truncated: String = line.chars().take(30).collect();
            ui_render::text(engine, &truncated, -8.2, 2.8 - di as f32 * 0.48, color, 0.25, em);

            // Tier color indicator
            let tier_color = match tier.as_str() {
                "Elite" | "Boss" => theme.danger,
                "Miniboss" => theme.warn,
                _ => theme.dim,
            };
            if selected {
                engine.spawn_glyph(Glyph {
                    character: '#',
                    position: Vec3::new(-8.6, 2.8 - di as f32 * 0.48, 0.0),
                    color: tier_color,
                    emission: 0.5,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        // ── Right panel: selected enemy detail ──
        if let Some((ref name, ref tier, floor)) = entries.get(state.bestiary_scroll) {
            let px = 1.2;
            let mut py = 3.0;

            // Enemy name with tier coloring
            let name_trunc: String = name.chars().take(20).collect();
            ui_render::text(engine, &name_trunc, px, py, theme.heading, 0.35, 0.7);
            py -= 0.5;

            ui_render::small(engine, &format!("Tier: {}", tier), px, py, theme.accent);
            py -= 0.4;
            ui_render::small(engine, &format!("First appears: Floor {}", floor), px, py, theme.primary);
            py -= 0.55;

            // Generate a fresh enemy to show stats
            let enemy = generate_enemy(*floor, 42u64.wrapping_add(*floor as u64 * 10));

            // HP bar
            ui_render::small(engine, &format!("HP: {}", enemy.max_hp), px, py, theme.hp_high);
            let hp_norm = (enemy.max_hp as f32 / 500.0).clamp(0.0, 1.0);
            ui_render::bar(engine, px + 3.0, py, 3.5, hp_norm, theme.hp_high, theme.muted, 0.22);
            py -= 0.4;

            // Damage bar
            ui_render::small(engine, &format!("Dmg: {}", enemy.base_damage), px, py, theme.danger);
            let dmg_norm = (enemy.base_damage as f32 / 100.0).clamp(0.0, 1.0);
            ui_render::bar(engine, px + 3.0, py, 3.5, dmg_norm, theme.danger, theme.muted, 0.22);
            py -= 0.4;

            // Rewards
            ui_render::small(engine, &format!("XP: {}", enemy.xp_reward), px, py, theme.xp);
            py -= 0.35;
            ui_render::small(engine, &format!("Gold: {}", enemy.gold_reward), px, py, theme.gold);
            py -= 0.45;

            // Special ability
            if let Some(ability) = enemy.special_ability {
                ui_render::small(engine, &format!("Ability: {}", ability), px, py, theme.warn);
                py -= 0.4;
            }

            // Floor ability
            let floor_ab = format!("{:?}", enemy.floor_ability);
            let floor_ab_trunc: String = floor_ab.chars().take(28).collect();
            ui_render::small(engine, &format!("Floor: {}", floor_ab_trunc), px, py, theme.dim);

            // Rotating enemy preview (ASCII art animation)
            let anim_offset = ((frame as f32 * 0.04).sin() * 0.2).abs();
            let preview_chars = ['/', '|', '\\', '|'];
            let anim_idx = ((frame / 15) % 4) as usize;
            engine.spawn_glyph(Glyph {
                character: preview_chars[anim_idx],
                position: Vec3::new(px + 5.5, 1.5 + anim_offset, 0.0),
                color: theme.danger,
                emission: 0.6,
                layer: RenderLayer::UI,
                ..Default::default()
            });
            engine.spawn_glyph(Glyph {
                character: 'O',
                position: Vec3::new(px + 5.5, 2.0 + anim_offset, 0.0),
                color: theme.danger,
                emission: 0.5,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    // ── Footer ──
    ui_render::small(engine, "[Up/Down] Scroll  [Esc/Space] Back", -5.5, -5.2, theme.muted);
}
