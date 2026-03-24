//! Victory screen — full 3-phase golden celebration cinematic.
//!
//! Uses engine capabilities: expanding golden glyph rings, camera shake,
//! golden glow halos, additive-blended fountain particles with velocity/gravity,
//! sin-wave shimmer on emission, score count-up animation, and layered rendering.
//!
//! Phase timing (elapsed = 2.7 - title_logo_timer):
//!   Phase 1 (0.0–0.5s):  GOLD FLASH — expanding golden ring, camera shake, glow
//!   Phase 2 (0.5–1.5s):  STILLNESS — "THE PROOF HAS BEEN EVALUATED" shimmer
//!   Phase 3 (1.5s+):     CELEBRATION — title, subtitle, fountain, stats, score

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Constants ────────────────────────────────────────────────────────────────

const CINEMATIC_DURATION: f32 = 2.7;

const PHASE1_END: f32 = 0.5;
const PHASE2_END: f32 = 1.5;

const RING_GLYPH_COUNT: usize = 44;
const FOUNTAIN_PARTICLE_MAX: usize = 24;

// ── Update ───────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let elapsed = CINEMATIC_DURATION - state.title_logo_timer.max(0.0);

    // Camera shake during gold flash phase
    if elapsed < PHASE1_END {
        engine.add_trauma(0.12);
    }

    // Accept input after the celebration phase has had time to display
    if elapsed > 2.2 {
        let enter = engine.input.just_pressed(Key::Enter)
            || engine.input.just_pressed(Key::Escape);
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

// ── Render ───────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let elapsed = CINEMATIC_DURATION - state.title_logo_timer.max(0.0);
    let frame = state.frame;

    if elapsed < PHASE1_END {
        render_phase1_gold_flash(engine, elapsed, frame);
    } else if elapsed < PHASE2_END {
        render_phase2_stillness(engine, elapsed, frame);
    } else {
        render_phase3_celebration(state, engine, theme, elapsed, frame);
    }
}

// ── Phase 1: GOLD FLASH (0.0 – 0.5s) ────────────────────────────────────────
//
// Expanding golden ring (44 glyph in a circle).
// Camera shake (handled in update).
// Screen-wide golden glow (large glyph with high emission and glow_radius).

fn render_phase1_gold_flash(
    engine: &mut ProofEngine,
    elapsed: f32,
    _frame: u64,
) {
    let progress = elapsed / PHASE1_END;
    let fade = (1.0 - progress).max(0.0);

    // ── Expanding golden ring of characters ─────────────────────────────────
    let ring_radius = progress * 26.0;
    for i in 0..RING_GLYPH_COUNT {
        let angle = (i as f32 / RING_GLYPH_COUNT as f32) * std::f32::consts::TAU;
        let x = angle.cos() * ring_radius;
        let y = angle.sin() * ring_radius * 0.55;

        // Slight sparkle variation per glyph
        let sparkle = ((i as f32 * 3.7 + progress * 12.0).sin() * 0.15 + 0.85).max(0.0);
        let alpha = fade * sparkle;

        engine.spawn_glyph(Glyph {
            character: '\u{2726}', // ✦
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(1.0 * alpha, 0.85 * alpha, 0.1 * alpha, alpha),
            emission: alpha * 2.5,
            glow_color: Vec3::new(1.0, 0.8, 0.15),
            glow_radius: 2.5 * fade,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }

    // ── Inner ring (smaller, denser) ─────────────────────────────────────────
    let inner_radius = progress * 12.0;
    for i in 0..16 {
        let angle = (i as f32 / 16.0) * std::f32::consts::TAU + 0.5;
        let x = angle.cos() * inner_radius;
        let y = angle.sin() * inner_radius * 0.55;
        let alpha = fade * 0.6;
        engine.spawn_glyph(Glyph {
            character: '+',
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(1.0 * alpha, 0.9 * alpha, 0.2 * alpha, alpha),
            emission: alpha * 1.5,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }

    // ── Screen-wide golden glow (center flash) ───────────────────────────────
    let glow_intensity = fade * 1.8;
    engine.spawn_glyph(Glyph {
        character: '\u{2726}', // ✦
        position: Vec3::ZERO,
        scale: Vec2::splat(8.0),
        color: Vec4::new(1.0, 0.85, 0.1, glow_intensity * 0.3),
        emission: glow_intensity * 3.0,
        glow_color: Vec3::new(1.0, 0.85, 0.15),
        glow_radius: 12.0 * fade,
        blend_mode: BlendMode::Additive,
        layer: RenderLayer::Overlay,
        ..Default::default()
    });
}

// ── Phase 2: STILLNESS (0.5 – 1.5s) ─────────────────────────────────────────
//
// "THE PROOF HAS BEEN EVALUATED" fades in with golden shimmer.
// Brief silence — just the text and dim background.

fn render_phase2_stillness(
    engine: &mut ProofEngine,
    elapsed: f32,
    frame: u64,
) {
    let progress = (elapsed - PHASE1_END) / (PHASE2_END - PHASE1_END);
    let fade_in = progress.min(1.0);

    // ── Dim warm background ──────────────────────────────────────────────────
    engine.spawn_glyph(Glyph {
        character: '█',
        position: Vec3::ZERO,
        scale: Vec2::splat(25.0),
        color: Vec4::new(0.02, 0.015, 0.0, 0.85),
        layer: RenderLayer::Background,
        ..Default::default()
    });

    // ── Golden shimmer text ──────────────────────────────────────────────────
    let text = "THE PROOF HAS BEEN EVALUATED";
    let shimmer = ((frame as f32 * 0.04).sin() * 0.12 + 0.88).max(0.0) * fade_in;

    // Render each character with individual shimmer phase
    let sp = 0.85 * 0.45; // spacing for scale 0.45
    let text_w = text.len() as f32 * sp;
    let text_x = -text_w * 0.5;
    let text_y = 1.0;

    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' { continue; }
        let char_shimmer = ((frame as f32 * 0.04 + i as f32 * 0.2).sin() * 0.12 + 0.88).max(0.0) * fade_in;
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(text_x + i as f32 * sp, text_y, 0.0),
            scale: Vec2::splat(0.45),
            color: Vec4::new(1.0 * char_shimmer, 0.85 * char_shimmer, 0.12 * char_shimmer, fade_in),
            emission: char_shimmer * 0.9,
            glow_color: Vec3::new(1.0, 0.8, 0.1),
            glow_radius: 0.5 * fade_in,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // ── Faint background stars (2-3 dim glyphs) ─────────────────────────────
    let star_positions = [(-5.0, 3.0), (6.0, -2.5), (-3.0, -3.8)];
    for (i, &(sx, sy)) in star_positions.iter().enumerate() {
        let pulse = ((frame as f32 * 0.02 + i as f32 * 2.0).sin() * 0.5 + 0.5) * 0.06 * fade_in;
        engine.spawn_glyph(Glyph {
            character: '\u{2726}', // ✦
            position: Vec3::new(sx, sy, 0.0),
            color: Vec4::new(pulse, pulse * 0.85, pulse * 0.1, pulse * 2.0),
            emission: pulse,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── Phase 3: CELEBRATION (1.5s+) ─────────────────────────────────────────────
//
// "VICTORY" in large gold text with sin-wave shimmer on emission.
// "You are the solution." subtitle.
// Gold fountain: 24 particles spawning upward from center with velocity/gravity.
// Full stat block, score count-up, continue hint.

fn render_phase3_celebration(
    state: &GameState,
    engine: &mut ProofEngine,
    theme: &crate::theme::Theme,
    elapsed: f32,
    frame: u64,
) {
    let cele_time = elapsed - PHASE2_END;

    // ── Warm background ──────────────────────────────────────────────────────
    engine.spawn_glyph(Glyph {
        character: '█',
        position: Vec3::ZERO,
        scale: Vec2::splat(25.0),
        color: Vec4::new(0.025, 0.02, 0.005, 0.9),
        layer: RenderLayer::Background,
        ..Default::default()
    });

    // ── "VICTORY" — large gold with per-char sin-wave shimmer ────────────────
    let title_text = "V I C T O R Y";
    let sp = 0.85 * 1.0; // spacing for scale 1.0
    let title_w = title_text.len() as f32 * sp;
    let title_x = -title_w * 0.5;
    let title_y = 4.5;
    let title_fade = (cele_time * 3.0).min(1.0);

    for (i, ch) in title_text.chars().enumerate() {
        if ch == ' ' { continue; }
        let wave = ((frame as f32 * 0.05 + i as f32 * 0.5).sin() * 0.15 + 0.85).max(0.0);
        let brightness = wave * title_fade;
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(title_x + i as f32 * sp, title_y, 0.0),
            scale: Vec2::splat(1.0),
            color: Vec4::new(1.0 * brightness, 0.85 * brightness, 0.1 * brightness, title_fade),
            emission: brightness * 1.8,
            glow_color: Vec3::new(1.0, 0.8, 0.1),
            glow_radius: 1.5 * title_fade,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // ── "You are the solution." subtitle ─────────────────────────────────────
    if cele_time > 0.3 {
        let sub_fade = ((cele_time - 0.3) * 2.5).min(1.0);
        ui_render::text_centered(
            engine,
            "You are the solution.",
            3.0,
            Vec4::new(
                theme.success.x * sub_fade,
                theme.success.y * sub_fade,
                theme.success.z * sub_fade,
                sub_fade,
            ),
            0.4,
            0.6 * sub_fade,
        );
    }

    // ── Gold fountain particles ──────────────────────────────────────────────
    if cele_time > 0.4 {
        let particle_time = cele_time - 0.4;
        let particle_count = ((particle_time * 12.0) as usize).min(FOUNTAIN_PARTICLE_MAX);
        let fountain_chars = ['\u{2726}', '\u{2605}', '\u{00B7}', '+', '\u{25C6}']; // ✦ ★ · + ◆

        for i in 0..particle_count {
            let seed = i as f32 * 73.1;

            // Stagger spawn times — each particle has its own "birth" time
            let spawn_delay = i as f32 * 0.08;
            let local_t = (particle_time - spawn_delay).max(0.0);
            if local_t <= 0.0 { continue; }

            // Cyclical: particles respawn every ~2 seconds
            let cycle = 2.0;
            let t = local_t.rem_euclid(cycle);

            // Initial velocity: mostly upward, slight horizontal spread
            let vx = (seed * 1.13).sin() * 2.5;
            let vy_initial = 5.0 + (seed * 0.71).cos().abs() * 3.0;

            // Apply gravity
            let gravity = 6.0;
            let x = vx * t;
            let y = -2.0 + vy_initial * t - 0.5 * gravity * t * t;

            // Fade out as particle falls
            let life_fade = (1.0 - t / cycle).max(0.0);
            let alpha = life_fade * 0.85;

            if alpha < 0.03 { continue; }

            // Color variation
            let gold_r = 1.0;
            let gold_g = 0.82 + (seed * 0.7).sin() * 0.12;
            let gold_b = 0.05 + (seed * 1.3).cos().abs() * 0.1;

            engine.spawn_glyph(Glyph {
                character: fountain_chars[i % fountain_chars.len()],
                position: Vec3::new(x, y, 0.0),
                velocity: Vec3::new(vx * 0.05, (vy_initial - gravity * t) * 0.05, 0.0),
                color: Vec4::new(gold_r * alpha, gold_g * alpha, gold_b * alpha, alpha),
                emission: alpha * 1.0,
                glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: 0.8 * life_fade,
                blend_mode: BlendMode::Additive,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }

    // ── Separator ────────────────────────────────────────────────────────────
    if cele_time > 0.7 {
        let sep_fade = ((cele_time - 0.7) * 3.0).min(1.0);
        let sep: String = "─".repeat(36);
        ui_render::text_centered(
            engine,
            &sep,
            2.0,
            Vec4::new(0.5 * sep_fade, 0.4 * sep_fade, 0.05 * sep_fade, sep_fade * 0.5),
            0.25,
            0.2 * sep_fade,
        );
    }

    // ── Full stat block ──────────────────────────────────────────────────────
    if cele_time > 0.8 {
        let stats_fade = ((cele_time - 0.8) * 2.0).min(1.0);

        if let Some(ref player) = state.player {
            let stat_lines = [
                format!("{} -- {} Lv.{}", player.name, player.class.name(), player.level),
                format!("Floors Cleared: {} | Kills: {}", state.floor_num, player.kills),
                format!("Gold Collected: {}", player.gold),
                format!("Damage Dealt: {} | Taken: {}", player.total_damage_dealt, player.total_damage_taken),
                format!("Spells Cast: {} | Items Used: {}", player.spells_cast, player.items_used),
                format!("Power Tier: {:?}", player.power_tier()),
            ];

            let stats_y_start = 1.2;

            for (i, line) in stat_lines.iter().enumerate() {
                let line_delay = i as f32 * 0.1;
                let line_fade = ((cele_time - 0.8 - line_delay) * 3.0).clamp(0.0, 1.0);
                if line_fade <= 0.0 { continue; }

                let alpha = stats_fade * line_fade;
                let c = Vec4::new(
                    theme.heading.x * alpha,
                    theme.heading.y * alpha,
                    theme.heading.z * alpha,
                    alpha,
                );
                ui_render::text(
                    engine,
                    line,
                    -7.0,
                    stats_y_start - i as f32 * 0.55,
                    c,
                    0.3,
                    0.45 * alpha,
                );
            }
        }
    }

    // ── Score count-up animation ─────────────────────────────────────────────
    if cele_time > 1.5 {
        let score_time = cele_time - 1.5;
        let final_score = compute_score(state);
        let count_duration = 2.0; // seconds to count from 0 to final
        let progress = (score_time / count_duration).min(1.0);

        // Ease-out curve for satisfying count-up feel
        let eased = 1.0 - (1.0 - progress).powi(3);
        let displayed_score = (final_score as f64 * eased as f64) as u64;

        let score_text = format!("SCORE: {}", displayed_score);
        let score_fade = (score_time * 2.0).min(1.0);

        // Score glows brighter as it reaches final value
        let glow = if progress >= 1.0 {
            ((frame as f32 * 0.06).sin() * 0.15 + 0.85).max(0.0)
        } else {
            0.7 + progress * 0.3
        };

        let sp_scale = 0.5;
        let sp = 0.85 * sp_scale;
        let score_w = score_text.len() as f32 * sp;
        let score_x = -score_w * 0.5;
        let score_y = -2.2;

        for (i, ch) in score_text.chars().enumerate() {
            if ch == ' ' { continue; }
            let char_glow = if progress >= 1.0 {
                ((frame as f32 * 0.06 + i as f32 * 0.3).sin() * 0.15 + 0.85).max(0.0)
            } else {
                glow
            };
            let brightness = char_glow * score_fade;
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(score_x + i as f32 * sp, score_y, 0.0),
                scale: Vec2::splat(sp_scale),
                color: Vec4::new(1.0 * brightness, 0.88 * brightness, 0.12 * brightness, score_fade),
                emission: brightness * 1.2,
                glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: if progress >= 1.0 { 0.8 } else { 0.3 },
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // ── Score complete flash ─────────────────────────────────────────────
        if progress >= 1.0 && score_time < count_duration + 0.3 {
            let flash = (1.0 - (score_time - count_duration) / 0.3).max(0.0) * 0.4;
            engine.spawn_glyph(Glyph {
                character: '\u{2726}', // ✦
                position: Vec3::new(0.0, score_y, 0.0),
                scale: Vec2::splat(3.0),
                color: Vec4::new(1.0, 0.85, 0.1, flash),
                emission: flash * 2.0,
                glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: 4.0 * flash,
                blend_mode: BlendMode::Additive,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }

    // ── "[Enter] Return to Title" hint ───────────────────────────────────────
    if cele_time > 2.5 {
        let hint_time = cele_time - 2.5;
        let hint_fade = (hint_time * 1.5).min(1.0);
        let hint_pulse = ((frame as f32 * 0.06).sin() * 0.3 + 0.7).max(0.0);
        let alpha = hint_fade * hint_pulse;
        ui_render::text_centered(
            engine,
            "[Enter] Return to Title",
            -4.5,
            Vec4::new(alpha * 0.5, alpha * 0.5, alpha * 0.5, alpha),
            0.3,
            0.25 * alpha,
        );
    }

    // ── Ambient golden motes (continuous) ────────────────────────────────────
    if cele_time > 0.5 {
        for i in 0..6u32 {
            let seed = i as f32 * 31.7 + frame as f32 * 0.004;
            let x = (seed * 1.3).sin() * 7.5;
            let y_base = ((seed * 0.8).cos() * 4.5);
            let drift = (frame as f32 * 0.008 + i as f32 * 0.7).sin() * 0.5;
            let brightness = ((seed * 2.7).sin() * 0.5 + 0.5) * 0.08;
            engine.spawn_glyph(Glyph {
                character: '\u{2726}', // ✦
                position: Vec3::new(x, y_base + drift, 0.0),
                color: Vec4::new(brightness, brightness * 0.85, brightness * 0.1, brightness * 1.5),
                emission: brightness * 0.6,
                blend_mode: BlendMode::Additive,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Compute a simple score from the player's run statistics.
fn compute_score(state: &GameState) -> u64 {
    if let Some(ref player) = state.player {
        let floor_score = state.floor_num as u64 * 100;
        let kill_score = player.kills as u64 * 25;
        let gold_score = player.gold.max(0) as u64;
        let damage_score = player.total_damage_dealt.max(0) as u64 / 10;
        let spell_score = player.spells_cast as u64 * 15;
        floor_score + kill_score + gold_score + damage_score + spell_score
    } else {
        0
    }
}
