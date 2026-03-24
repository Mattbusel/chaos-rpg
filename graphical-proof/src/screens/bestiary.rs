//! Bestiary — enemy encyclopedia.
//! Shows enemies the player has encountered during runs.
//! Since the core bestiary is data-driven and starts empty,
//! we generate a preview using the enemy generation system.

use proof_engine::prelude::*;
use chaos_rpg_core::enemy::generate_enemy;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

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

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);

    let entries = generate_preview();
    let total = entries.len();
    if up && state.bestiary_scroll > 0 { state.bestiary_scroll -= 1; }
    if down && state.bestiary_scroll < total.saturating_sub(1) { state.bestiary_scroll += 1; }
    if esc { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let entries = generate_preview();

    render_text(engine, &format!("BESTIARY — {} enemies catalogued", entries.len()),
        -18.0, 9.0, theme.heading, 0.8);

    if entries.is_empty() {
        render_text(engine, "No enemies in the bestiary.", -8.0, 4.0, theme.dim, 0.3);
    } else {
        // Left panel: enemy list
        let start = state.bestiary_scroll.saturating_sub(7);
        let end = (start + 16).min(entries.len());
        for (di, idx) in (start..end).enumerate() {
            let (ref name, ref tier, floor) = entries[idx];
            let selected = idx == state.bestiary_scroll;
            let color = if selected { theme.selected } else { theme.primary };
            let prefix = if selected { "> " } else { "  " };
            let line = format!("{}{} ({}, F{})", prefix, name, tier, floor);
            let truncated: String = line.chars().take(40).collect();
            render_text(engine, &truncated, -18.0, 7.0 - di as f32 * 0.95, color,
                if selected { 0.7 } else { 0.35 });
        }

        // Right panel: selected enemy detail
        if let Some((ref name, ref tier, floor)) = entries.get(state.bestiary_scroll) {
            render_text(engine, name, 2.0, 7.0, theme.heading, 0.7);
            render_text(engine, &format!("Tier: {}", tier), 2.0, 5.5, theme.accent, 0.5);
            render_text(engine, &format!("First appears: Floor {}", floor), 2.0, 4.5, theme.primary, 0.4);

            // Generate a fresh enemy to show stats
            let enemy = generate_enemy(*floor, 42u64.wrapping_add(*floor as u64 * 10));
            render_text(engine, &format!("HP: {}", enemy.max_hp), 2.0, 3.0, theme.hp_high, 0.4);
            render_text(engine, &format!("Base Damage: {}", enemy.base_damage), 2.0, 2.0, theme.danger, 0.4);
            render_text(engine, &format!("XP Reward: {}", enemy.xp_reward), 2.0, 1.0, theme.xp, 0.4);
            render_text(engine, &format!("Gold Reward: {}", enemy.gold_reward), 2.0, 0.0, theme.gold, 0.4);
            if let Some(ability) = enemy.special_ability {
                render_text(engine, &format!("Ability: {}", ability), 2.0, -1.5, theme.warn, 0.4);
            }
            render_text(engine, &format!("Floor Ability: {:?}", enemy.floor_ability), 2.0, -2.5, theme.dim, 0.3);
        }
    }

    render_text(engine, "[Up/Down] Scroll  [Esc] Back", -18.0, -12.0, theme.muted, 0.2);
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
