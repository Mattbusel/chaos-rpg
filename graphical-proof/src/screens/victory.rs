//! Victory screen — golden celebration with stats.

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

    // Golden shimmer title
    let shimmer = ((state.frame as f32 * 0.05).sin() * 0.15 + 0.85).max(0.0);
    let gold = Vec4::new(1.0, 0.88 * shimmer, 0.1, 1.0);
    render_text(engine, "VICTORY", -2.5, 8.0, gold, 1.5);
    render_text(engine, "The Proof has been evaluated. You are the solution.", -15.0, 6.0, theme.success, 0.5);

    if let Some(ref player) = state.player {
        let stats = [
            format!("{} — {} Lv.{}", player.name, player.class.name(), player.level),
            format!("Floors: {} | Kills: {} | Gold: {}", state.floor_num, player.kills, player.gold),
            format!("Damage Dealt: {} | Damage Taken: {}", player.total_damage_dealt, player.total_damage_taken),
            format!("Power Tier: {:?}", player.power_tier()),
        ];
        for (i, line) in stats.iter().enumerate() {
            render_text(engine, line, -14.0, 2.0 - i as f32 * 1.5, theme.heading, 0.5);
        }
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
