//! Victory screen — full 3-phase golden celebration cinematic.
//!
//! Phase timing (elapsed = 2.7 - title_logo_timer):
//!   Phase 1 (0.0-0.5s):  GOLD FLASH — expanding golden ring, camera shake, glow
//!   Phase 2 (0.5-1.5s):  STILLNESS — "THE PROOF HAS BEEN EVALUATED" shimmer
//!   Phase 3 (1.5s+):     CELEBRATION — title, subtitle, fountain, stats, score

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

const CINEMATIC_DUR: f32 = 2.7;
const P1: f32 = 0.5;
const P2: f32 = 1.5;
const RING_N: usize = 44;
const FOUNTAIN_MAX: usize = 24;

// ── Update ───────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let elapsed = CINEMATIC_DUR - state.title_logo_timer.max(0.0);
    if elapsed < P1 { engine.add_trauma(0.12); }
    if elapsed > 2.2 {
        if engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape) {
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
    let elapsed = CINEMATIC_DUR - state.title_logo_timer.max(0.0);
    let f = state.frame;

    if      elapsed < P1 { phase1_gold_flash(engine, elapsed, f); }
    else if elapsed < P2 { phase2_stillness(engine, elapsed, f); }
    else                  { phase3_celebration(state, engine, theme, elapsed, f); }
}

// ── Phase 1: GOLD FLASH (0.0–0.5s) ──────────────────────────────────────────

fn phase1_gold_flash(engine: &mut ProofEngine, elapsed: f32, _f: u64) {
    let t = elapsed / P1;
    let fade = (1.0 - t).max(0.0);

    // Expanding golden ring — 44 ✦ glyphs in a circle
    let radius = t * 26.0;
    for i in 0..RING_N {
        let a = (i as f32 / RING_N as f32) * std::f32::consts::TAU;
        let sparkle = ((i as f32 * 3.7 + t * 12.0).sin() * 0.15 + 0.85).max(0.0);
        let al = fade * sparkle;
        engine.spawn_glyph(Glyph {
            character: '\u{2726}', // ✦
            position: Vec3::new(a.cos() * radius, a.sin() * radius * 0.55, 0.0),
            color: Vec4::new(al, 0.85 * al, 0.1 * al, al),
            emission: al * 2.5,
            glow_color: Vec3::new(1.0, 0.8, 0.15),
            glow_radius: 2.5 * fade,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }
    // Inner ring — 16 + glyphs
    let ir = t * 12.0;
    for i in 0..16 {
        let a = (i as f32 / 16.0) * std::f32::consts::TAU + 0.5;
        let al = fade * 0.6;
        engine.spawn_glyph(Glyph {
            character: '+',
            position: Vec3::new(a.cos() * ir, a.sin() * ir * 0.55, 0.0),
            color: Vec4::new(al, 0.9 * al, 0.2 * al, al),
            emission: al * 1.5, blend_mode: BlendMode::Additive,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }
    // Screen-wide golden glow — large glyph with high emission and glow_radius
    let gi = fade * 1.8;
    engine.spawn_glyph(Glyph {
        character: '\u{2726}', position: Vec3::ZERO, scale: Vec2::splat(8.0),
        color: Vec4::new(1.0, 0.85, 0.1, gi * 0.3),
        emission: gi * 3.0, glow_color: Vec3::new(1.0, 0.85, 0.15),
        glow_radius: 12.0 * fade, blend_mode: BlendMode::Additive,
        layer: RenderLayer::Overlay, ..Default::default()
    });
}

// ── Phase 2: STILLNESS (0.5–1.5s) ───────────────────────────────────────────

fn phase2_stillness(engine: &mut ProofEngine, elapsed: f32, frame: u64) {
    let t = (elapsed - P1) / (P2 - P1);
    let fi = t.min(1.0);

    // Dim warm background
    engine.spawn_glyph(Glyph {
        character: '█', position: Vec3::ZERO, scale: Vec2::splat(25.0),
        color: Vec4::new(0.02, 0.015, 0.0, 0.85),
        layer: RenderLayer::Background, ..Default::default()
    });

    // "THE PROOF HAS BEEN EVALUATED" — golden shimmer, per-char phase
    let text = "THE PROOF HAS BEEN EVALUATED";
    let sp = 0.85 * 0.45;
    let tw = text.len() as f32 * sp;
    let tx = -tw * 0.5;
    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' { continue; }
        let cs = ((frame as f32 * 0.04 + i as f32 * 0.2).sin() * 0.12 + 0.88).max(0.0) * fi;
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(tx + i as f32 * sp, 1.0, 0.0),
            scale: Vec2::splat(0.45),
            color: Vec4::new(cs, 0.85 * cs, 0.12 * cs, fi),
            emission: cs * 0.9, glow_color: Vec3::new(1.0, 0.8, 0.1),
            glow_radius: 0.5 * fi, layer: RenderLayer::UI, ..Default::default()
        });
    }
    // Background stars
    for (i, &(sx, sy)) in [(-5.0f32,3.0),(6.0,-2.5),(-3.0,-3.8)].iter().enumerate() {
        let p = ((frame as f32 * 0.02 + i as f32 * 2.0).sin() * 0.5 + 0.5) * 0.06 * fi;
        engine.spawn_glyph(Glyph {
            character: '\u{2726}', position: Vec3::new(sx, sy, 0.0),
            color: Vec4::new(p, p * 0.85, p * 0.1, p * 2.0),
            emission: p, blend_mode: BlendMode::Additive,
            layer: RenderLayer::Particle, ..Default::default()
        });
    }
}

// ── Phase 3: CELEBRATION (1.5s+) ─────────────────────────────────────────────

fn phase3_celebration(
    state: &GameState, engine: &mut ProofEngine,
    theme: &crate::theme::Theme, elapsed: f32, frame: u64,
) {
    let ct = elapsed - P2;

    // Warm background
    engine.spawn_glyph(Glyph {
        character: '█', position: Vec3::ZERO, scale: Vec2::splat(25.0),
        color: Vec4::new(0.025, 0.02, 0.005, 0.9),
        layer: RenderLayer::Background, ..Default::default()
    });

    // ── "VICTORY" — large gold with sin-wave shimmer on emission ─────────────
    let title = "V I C T O R Y";
    let sp = 0.85;
    let tw = title.len() as f32 * sp;
    let tx = -tw * 0.5;
    let tf = (ct * 3.0).min(1.0);
    for (i, ch) in title.chars().enumerate() {
        if ch == ' ' { continue; }
        let w = ((frame as f32 * 0.05 + i as f32 * 0.5).sin() * 0.15 + 0.85).max(0.0);
        let b = w * tf;
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(tx + i as f32 * sp, 4.5, 0.0),
            scale: Vec2::splat(1.0),
            color: Vec4::new(b, 0.85 * b, 0.1 * b, tf),
            emission: b * 1.8, glow_color: Vec3::new(1.0, 0.8, 0.1),
            glow_radius: 1.5 * tf, layer: RenderLayer::UI, ..Default::default()
        });
    }

    // ── "You are the solution." subtitle ─────────────────────────────────────
    if ct > 0.3 {
        let sf = ((ct - 0.3) * 2.5).min(1.0);
        ui_render::text_centered(engine, "You are the solution.", 3.0,
            Vec4::new(theme.success.x*sf, theme.success.y*sf, theme.success.z*sf, sf),
            0.4, 0.6*sf);
    }

    // ── Gold fountain — 24 particles spawning upward with velocity & gravity ─
    if ct > 0.4 {
        let pt = ct - 0.4;
        let count = ((pt * 12.0) as usize).min(FOUNTAIN_MAX);
        let chars = ['\u{2726}', '\u{2605}', '\u{00B7}', '+', '\u{25C6}']; // ✦ ★ · + ◆
        for i in 0..count {
            let seed = i as f32 * 73.1;
            let delay = i as f32 * 0.08;
            let lt = (pt - delay).max(0.0);
            if lt <= 0.0 { continue; }
            let cycle = 2.0;
            let t = lt.rem_euclid(cycle);
            let vx = (seed * 1.13).sin() * 2.5;
            let vy0 = 5.0 + (seed * 0.71).cos().abs() * 3.0;
            let grav = 6.0;
            let x = vx * t;
            let y = -2.0 + vy0 * t - 0.5 * grav * t * t;
            let lf = (1.0 - t / cycle).max(0.0);
            let al = lf * 0.85;
            if al < 0.03 { continue; }
            let gr = 0.82 + (seed * 0.7).sin() * 0.12;
            let gb = 0.05 + (seed * 1.3).cos().abs() * 0.1;
            engine.spawn_glyph(Glyph {
                character: chars[i % chars.len()],
                position: Vec3::new(x, y, 0.0),
                velocity: Vec3::new(vx * 0.05, (vy0 - grav * t) * 0.05, 0.0),
                color: Vec4::new(al, gr * al, gb * al, al),
                emission: al, glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: 0.8 * lf, blend_mode: BlendMode::Additive,
                layer: RenderLayer::Particle, ..Default::default()
            });
        }
    }

    // ── Full stat block (name, class, floors, kills, gold, damage, tier) ─────
    if ct > 0.8 {
        let sf = ((ct - 0.8) * 2.0).min(1.0);
        // Separator
        ui_render::text_centered(engine, &"─".repeat(36), 2.0,
            Vec4::new(0.5*sf, 0.4*sf, 0.05*sf, sf*0.5), 0.25, 0.2*sf);

        if let Some(ref p) = state.player {
            let lines = [
                format!("{} -- {} Lv.{}", p.name, p.class.name(), p.level),
                format!("Floors: {} | Kills: {} | Gold: {}", state.floor_num, p.kills, p.gold),
                format!("Damage Dealt: {} | Taken: {}", p.total_damage_dealt, p.total_damage_taken),
                format!("Spells: {} | Items: {}", p.spells_cast, p.items_used),
                format!("Power Tier: {:?}", p.power_tier()),
            ];
            for (i, l) in lines.iter().enumerate() {
                let ld = i as f32 * 0.1;
                let lf = ((ct - 0.8 - ld) * 3.0).clamp(0.0, 1.0);
                if lf <= 0.0 { continue; }
                let a = sf * lf;
                ui_render::text(engine, l, -7.0, 1.2 - i as f32 * 0.55,
                    Vec4::new(theme.heading.x*a, theme.heading.y*a, theme.heading.z*a, a),
                    0.3, 0.45*a);
            }
        }
    }

    // ── Score count-up animation (0 to final over 2s with ease-out) ──────────
    if ct > 1.5 {
        let st = ct - 1.5;
        let final_score = compute_score(state);
        let dur = 2.0;
        let prog = (st / dur).min(1.0);
        let eased = 1.0 - (1.0 - prog).powi(3);
        let shown = (final_score as f64 * eased as f64) as u64;
        let stxt = format!("SCORE: {}", shown);
        let sf = (st * 2.0).min(1.0);
        let done = prog >= 1.0;
        let ssp = 0.85 * 0.5;
        let sw = stxt.len() as f32 * ssp;
        let sx = -sw * 0.5;
        let sy = -2.2;
        for (i, ch) in stxt.chars().enumerate() {
            if ch == ' ' { continue; }
            let g = if done {
                ((frame as f32 * 0.06 + i as f32 * 0.3).sin() * 0.15 + 0.85).max(0.0)
            } else { 0.7 + prog * 0.3 };
            let b = g * sf;
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(sx + i as f32 * ssp, sy, 0.0),
                scale: Vec2::splat(0.5),
                color: Vec4::new(b, 0.88 * b, 0.12 * b, sf),
                emission: b * 1.2, glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: if done { 0.8 } else { 0.3 },
                layer: RenderLayer::UI, ..Default::default()
            });
        }
        // Flash when score finishes counting
        if done && st < dur + 0.3 {
            let fl = (1.0 - (st - dur) / 0.3).max(0.0) * 0.4;
            engine.spawn_glyph(Glyph {
                character: '\u{2726}', position: Vec3::new(0.0, sy, 0.0),
                scale: Vec2::splat(3.0),
                color: Vec4::new(1.0, 0.85, 0.1, fl),
                emission: fl * 2.0, glow_color: Vec3::new(1.0, 0.8, 0.1),
                glow_radius: 4.0 * fl, blend_mode: BlendMode::Additive,
                layer: RenderLayer::Overlay, ..Default::default()
            });
        }
    }

    // ── "[Enter] Return to Title" hint ───────────────────────────────────────
    if ct > 2.5 {
        let hf = ((ct - 2.5) * 1.5).min(1.0);
        let hp = ((frame as f32 * 0.06).sin() * 0.3 + 0.7).max(0.0);
        let a = hf * hp;
        ui_render::text_centered(engine, "[Enter] Return to Title", -4.5,
            Vec4::new(a*0.5, a*0.5, a*0.5, a), 0.3, 0.25*a);
    }

    // ── Ambient golden motes ─────────────────────────────────────────────────
    if ct > 0.5 {
        for i in 0..6u32 {
            let s = i as f32 * 31.7 + frame as f32 * 0.004;
            let x = (s * 1.3).sin() * 7.5;
            let y = (s * 0.8).cos() * 4.5 + (frame as f32 * 0.008 + i as f32 * 0.7).sin() * 0.5;
            let b = ((s * 2.7).sin() * 0.5 + 0.5) * 0.08;
            engine.spawn_glyph(Glyph {
                character: '\u{2726}', position: Vec3::new(x, y, 0.0),
                color: Vec4::new(b, b * 0.85, b * 0.1, b * 1.5),
                emission: b * 0.6, blend_mode: BlendMode::Additive,
                layer: RenderLayer::Particle, ..Default::default()
            });
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn compute_score(state: &GameState) -> u64 {
    state.player.as_ref().map(|p| {
        state.floor_num as u64 * 100
            + p.kills as u64 * 25
            + p.gold.max(0) as u64
            + p.total_damage_dealt.max(0) as u64 / 10
            + p.spells_cast as u64 * 15
    }).unwrap_or(0)
}
