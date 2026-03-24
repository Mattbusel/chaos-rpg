//! Victory screen — golden celebration cinematic.
//!
//! Phases:
//!   Phase 1 (0.0-0.5s):  Gold flash — boss death explosion
//!   Phase 2 (0.5-1.5s):  Stillness — silence, golden tint
//!   Phase 3 (1.5s+):     Celebration — title, stats, gold fountain particles

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    // Reuse title_logo_timer for cinematic (starts at 2.7 from game_over init)
    let elapsed = 2.7 - state.title_logo_timer.max(0.0);

    // Camera shake during gold flash
    if elapsed < 0.5 {
        engine.add_trauma(0.08);
    }

    // Accept input after cinematic
    if elapsed > 2.0 {
        let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape);
        if enter {
            crate::state::delete_save();
            state.player = None;
            state.floor = None;
            state.enemy = None;
            state.combat_state = None;
            state.screen = AppScreen::Title;
            state.title_logo_timer = 1.5;
        }
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let elapsed = 2.7 - state.title_logo_timer.max(0.0);
    let frame = state.frame;

    // ── Phase 1: Gold flash (0.0 - 0.5s) ──
    if elapsed < 0.5 {
        let progress = elapsed / 0.5;
        // Expanding golden ring
        let ring_r = progress * 25.0;
        let num_points = 40;
        for i in 0..num_points {
            let angle = (i as f32 / num_points as f32) * std::f32::consts::TAU;
            let x = angle.cos() * ring_r;
            let y = angle.sin() * ring_r * 0.6;
            let fade = (1.0 - progress).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '✦',
                position: Vec3::new(x, y, 0.0),
                color: Vec4::new(1.0, 0.88 * fade, 0.1, fade),
                emission: fade * 2.0,
                glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: 2.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
        return;
    }

    // ── Phase 2: Stillness (0.5 - 1.5s) ──
    if elapsed < 1.5 {
        let fade_in = ((elapsed - 0.5) / 1.0).min(1.0);
        // Gentle golden glow building
        let shimmer = ((frame as f32 * 0.03).sin() * 0.1 + 0.9) * fade_in;
        let gold = Vec4::new(1.0 * shimmer, 0.88 * shimmer, 0.1 * shimmer, fade_in);

        render_text_centered(engine, "THE PROOF HAS BEEN EVALUATED", 4.0, gold, fade_in * 0.8);
        return;
    }

    // ── Phase 3: Celebration (1.5s+) ──
    let cele_time = elapsed - 1.5;
    let shimmer = ((frame as f32 * 0.05).sin() * 0.15 + 0.85).max(0.0);
    let gold = Vec4::new(1.0, 0.88 * shimmer, 0.1, 1.0);

    // Main title
    render_text_centered(engine, "VICTORY", 8.0, gold, 1.5);

    // Subtitle
    let sub_fade = (cele_time * 2.0).min(1.0);
    render_text_centered(engine, "You are the solution.", 6.0,
        Vec4::new(theme.success.x * sub_fade, theme.success.y * sub_fade, theme.success.z * sub_fade, sub_fade), 0.5);

    // Gold fountain particles
    if cele_time > 0.3 {
        let particle_count = ((cele_time - 0.3) * 10.0).min(20.0) as usize;
        for i in 0..particle_count {
            let seed_f = i as f32 * 73.1 + frame as f32 * 0.08;
            let x = seed_f.sin() * 6.0;
            let y = -2.0 + (seed_f.cos() * 3.0).abs() + (frame as f32 * 0.02 + i as f32 * 0.5).sin() * 2.0;
            let gold_var = Vec4::new(
                1.0,
                0.85 + (seed_f * 0.7).sin() * 0.1,
                0.0 + (seed_f * 1.3).cos() * 0.1,
                0.7,
            );
            let chars = ['✦', '★', '·', '+', '◆'];
            engine.spawn_glyph(Glyph {
                character: chars[i % chars.len()],
                position: Vec3::new(x, y, 0.0),
                color: gold_var,
                emission: 0.8,
                glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: 1.0,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }

    // Stats
    if cele_time > 0.5 {
        let stats_fade = ((cele_time - 0.5) * 2.0).min(1.0);
        if let Some(ref player) = state.player {
            let stats = [
                format!("{} — {} Lv.{}", player.name, player.class.name(), player.level),
                format!("Floors: {} | Kills: {} | Gold: {}", state.floor_num, player.kills, player.gold),
                format!("Damage Dealt: {} | Taken: {}", player.total_damage_dealt, player.total_damage_taken),
                format!("Power Tier: {:?}", player.power_tier()),
            ];
            for (i, line) in stats.iter().enumerate() {
                let color = Vec4::new(
                    theme.heading.x * stats_fade,
                    theme.heading.y * stats_fade,
                    theme.heading.z * stats_fade,
                    stats_fade,
                );
                render_text(engine, line, -14.0, 1.0 - i as f32 * 1.5, color, 0.5 * stats_fade);
            }
        }
    }

    // Continue hint
    if cele_time > 1.5 {
        let hint_pulse = ((frame as f32 * 0.06).sin() * 0.3 + 0.7).max(0.0);
        render_text_centered(engine, "[Enter] Return to Title", -12.0,
            Vec4::new(hint_pulse * 0.4, hint_pulse * 0.4, hint_pulse * 0.4, hint_pulse), 0.3);
    }
}

fn render_text_centered(engine: &mut ProofEngine, text: &str, y: f32, color: Vec4, emission: f32) {
    let x = -(text.len() as f32 * 0.225);
    render_text(engine, text, x, y, color, emission);
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
