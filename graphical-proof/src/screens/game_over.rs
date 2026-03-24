//! Game over screen — full death cinematic sequence.
//!
//! 5-phase cinematic ported from graphical/src/death_seq.rs:
//!   Phase 1 (0.0-0.7s):  KILLING BLOW — red flash, damage number, camera shake
//!   Phase 2 (0.7-1.1s):  CRACK — fracture lines radiate from center
//!   Phase 3 (1.1-1.5s):  COLLAPSE — debris scatter, color drains
//!   Phase 4 (1.5-1.8s):  VOID — near-black, red embers
//!   Phase 5 (1.8s+):     EPITAPH — "YOU DIED" typewriter, stats, continue

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, dt: f32) {
    // Death cinematic timer (seconds)
    if !state.death_cinematic_done {
        // Camera shake during phase 1
        let elapsed = 2.7 - state.title_logo_timer.max(0.0); // reuse timer for cinematic
        if elapsed < 0.7 {
            engine.add_trauma(0.15 * dt);
        }
        // Mark done after full sequence plays
        if state.title_logo_timer <= 0.0 {
            state.death_cinematic_done = true;
        }
        return; // No input during cinematic
    }

    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape);
    if enter {
        crate::state::delete_save();
        state.player = None;
        state.floor = None;
        state.enemy = None;
        state.combat_state = None;
        state.screen = AppScreen::Title;
        state.death_cinematic_done = false;
        state.title_logo_timer = 2.7; // reset for next use
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let elapsed = 2.7 - state.title_logo_timer.max(0.0);
    let frame = state.frame;

    // ── Phase 1: KILLING BLOW (0.0 - 0.7s) ──
    if elapsed < 0.7 {
        let progress = elapsed / 0.7;

        // Red expanding ring burst
        let ring_radius = progress * 20.0;
        let ring_chars = ['░', '▒', '▓', '█'];
        let num_points = 32;
        for i in 0..num_points {
            let angle = (i as f32 / num_points as f32) * std::f32::consts::TAU;
            let x = angle.cos() * ring_radius;
            let y = angle.sin() * ring_radius * 0.6; // squash vertically
            let fade = 1.0 - progress;
            let rv = fade * 0.8;
            engine.spawn_glyph(Glyph {
                character: ring_chars[i % ring_chars.len()],
                position: Vec3::new(x, y, 0.0),
                color: Vec4::new(rv, rv * 0.15, 0.0, fade),
                emission: rv * 1.5,
                glow_color: Vec3::new(rv, 0.05, 0.0),
                glow_radius: 1.5,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }

        // "KILLING BLOW" label
        let label = "KILLING BLOW";
        let label_fade = (1.0 - progress).max(0.0);
        render_text_centered(engine, label, 2.0,
            Vec4::new(label_fade * 0.8, label_fade * 0.15, 0.0, label_fade), label_fade);

        // Giant damage number
        if let Some(ref _enemy) = state.enemy {
            let last_log = state.combat_log.last().cloned().unwrap_or_default();
            render_text_centered(engine, &last_log, 0.0,
                Vec4::new(label_fade, label_fade * 0.15, 0.0, label_fade), label_fade * 0.8);
        }
        return;
    }

    // ── Phase 2: CRACK (0.7 - 1.1s) ──
    if elapsed < 1.1 {
        let progress = (elapsed - 0.7) / 0.4;
        let crack_len = progress * 18.0;

        // 8 fracture lines from center
        let dirs: [(f32, f32); 8] = [
            (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0),
            (0.7, 0.7), (-0.7, 0.7), (0.7, -0.7), (-0.7, -0.7),
        ];
        let crack_chars = ['─', '─', '│', '│', '╱', '╲', '╱', '╲'];

        for (di, &(dx, dy)) in dirs.iter().enumerate() {
            for step in 0..(crack_len as i32) {
                let x = dx * step as f32;
                let y = dy * step as f32 * 0.6;
                let fade = 1.0 - (step as f32 / crack_len);
                let brightness = fade * (1.0 - progress * 0.5);
                engine.spawn_glyph(Glyph {
                    character: crack_chars[di],
                    position: Vec3::new(x, y, 0.0),
                    color: Vec4::new(brightness * 0.9, brightness * 0.15, 0.0, brightness),
                    emission: brightness,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }
        return;
    }

    // ── Phase 3: COLLAPSE (1.1 - 1.5s) ──
    if elapsed < 1.5 {
        let progress = (elapsed - 1.1) / 0.4;
        let fade = 1.0 - progress;

        // Scattered ember debris
        for i in 0..20u32 {
            let seed_f = i as f32 * 83.7 + frame as f32 * 0.15;
            let x = seed_f.sin() * 16.0;
            let y = seed_f.cos() * 8.0 + progress * 3.0;
            let brightness = fade * 0.4;
            engine.spawn_glyph(Glyph {
                character: '·',
                position: Vec3::new(x, y, 0.0),
                color: Vec4::new(brightness, brightness * 0.2, 0.0, brightness),
                emission: brightness * 0.5,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
        return;
    }

    // ── Phase 4: VOID (1.5 - 1.8s) ──
    if elapsed < 1.8 {
        // Near-black with faint pulsing embers
        let pulse = ((frame as f32 * 0.15).sin() * 0.5 + 0.5) * 0.15;
        render_text_centered(engine, "...", 0.0,
            Vec4::new(pulse, pulse * 0.15, 0.0, pulse * 2.0), pulse);
        return;
    }

    // ── Phase 5: EPITAPH (1.8s+) ──
    let epi_time = elapsed - 1.8;

    // "YOU DIED" — typewriter reveal
    let title = "Y  O  U    D  I  E  D";
    let reveal_chars = ((epi_time * 25.0) as usize).min(title.len());
    let title_shown: String = title.chars().take(reveal_chars).collect();
    let pulse = ((frame as f32 * 0.12).sin() * 0.2 + 0.8).max(0.0);
    let death_color = Vec4::new(pulse, pulse * 0.12, 0.0, 1.0);
    render_text_centered(engine, &title_shown, 6.0, death_color, 1.2);

    // Subtitle
    if epi_time > 0.5 {
        let sub_fade = ((epi_time - 0.5) * 2.0).min(1.0);
        render_text_centered(engine, "The mathematics have consumed you.", 4.0,
            Vec4::new(theme.dim.x * sub_fade, theme.dim.y * sub_fade, theme.dim.z * sub_fade, sub_fade), 0.4);
    }

    // Stats
    if epi_time > 1.0 {
        if let Some(ref player) = state.player {
            let stats_fade = ((epi_time - 1.0) * 1.5).min(1.0);
            let stats = [
                format!("{} — {} Lv.{}", player.name, player.class.name(), player.level),
                format!("Floor: {} | Kills: {} | Gold: {}", state.floor_num, player.kills, player.gold),
                format!("Damage Dealt: {} | Taken: {}", player.total_damage_dealt, player.total_damage_taken),
                format!("Spells: {} | Items: {} | Corruption: {}", player.spells_cast, player.items_used, player.corruption),
                format!("Power Tier: {:?}", player.power_tier()),
            ];
            for (i, line) in stats.iter().enumerate() {
                let color = Vec4::new(
                    theme.primary.x * stats_fade,
                    theme.primary.y * stats_fade,
                    theme.primary.z * stats_fade,
                    stats_fade,
                );
                render_text(engine, line, -16.0, 1.0 - i as f32 * 1.3, color, 0.4 * stats_fade);
            }
        }
    }

    // Combat log
    if epi_time > 1.5 {
        let log_fade = ((epi_time - 1.5) * 2.0).min(1.0);
        let log_start = state.combat_log.len().saturating_sub(6);
        for (i, msg) in state.combat_log[log_start..].iter().enumerate() {
            let truncated: String = msg.chars().take(65).collect();
            let color = Vec4::new(
                theme.dim.x * log_fade,
                theme.dim.y * log_fade,
                theme.dim.z * log_fade,
                log_fade * 0.8,
            );
            render_text(engine, &truncated, -16.0, -5.0 - i as f32 * 0.9, color, 0.2 * log_fade);
        }
    }

    // Continue hint
    if state.death_cinematic_done {
        let hint_pulse = ((frame as f32 * 0.06).sin() * 0.3 + 0.7).max(0.0);
        render_text_centered(engine, "[Enter] View run summary", -12.0,
            Vec4::new(hint_pulse * 0.4, hint_pulse * 0.4, hint_pulse * 0.4, hint_pulse), 0.3);
    }
}

// ── Helpers ──

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
