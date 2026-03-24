//! Game over screen — death recap with stats and narrative.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape);
    if enter {
        crate::state::delete_save();
        state.player = None;
        state.floor = None;
        state.enemy = None;
        state.combat_state = None;
        state.screen = AppScreen::Title;
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Flashing "YOU DIED" title
    let flash = ((state.frame as f32 * 0.08).sin() * 0.3 + 0.7).max(0.0);
    let death_color = Vec4::new(theme.danger.x * flash + 0.3, theme.danger.y * flash, theme.danger.z * flash, 1.0);
    render_text(engine, "YOU DIED", -3.0, 8.0, death_color, 1.2);
    render_text(engine, "The mathematics have consumed you.", -10.0, 6.0, theme.dim, 0.4);

    if let Some(ref player) = state.player {
        let stats = [
            format!("{} — {} Lv.{}", player.name, player.class.name(), player.level),
            format!("Floor: {} | Kills: {} | Gold: {}", state.floor_num, player.kills, player.gold),
            format!("Damage Dealt: {} | Damage Taken: {}", player.total_damage_dealt, player.total_damage_taken),
            format!("Spells Cast: {} | Items Used: {}", player.spells_cast, player.items_used),
            format!("Corruption: {} | Power Tier: {:?}", player.corruption, player.power_tier()),
        ];
        for (i, line) in stats.iter().enumerate() {
            render_text(engine, line, -16.0, 2.0 - i as f32 * 1.3, theme.primary, 0.4);
        }
    }

    // Last combat log entries
    let log_start = state.combat_log.len().saturating_sub(8);
    for (i, msg) in state.combat_log[log_start..].iter().enumerate() {
        let truncated: String = msg.chars().take(70).collect();
        render_text(engine, &truncated, -16.0, -5.0 - i as f32 * 0.8, theme.dim, 0.25);
    }

    render_text(engine, "[Enter] Return to Title", -7.0, -12.0, theme.muted, 0.3);
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
