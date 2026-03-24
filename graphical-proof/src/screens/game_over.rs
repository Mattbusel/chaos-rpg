//! Game over screen — full 5-phase death cinematic sequence.
//!
//! Phase timing (elapsed = 2.7 - title_logo_timer):
//!   Phase 1 (0.0-0.7s):  KILLING BLOW — red ring burst, camera shake, damage
//!   Phase 2 (0.7-1.1s):  CRACK — 8 fracture lines radiating from center
//!   Phase 3 (1.1-1.5s):  COLLAPSE — fractures fade, ember debris with gravity
//!   Phase 4 (1.5-1.8s):  VOID — near-black, faint pulsing embers at edges
//!   Phase 5 (1.8s+):     EPITAPH — typewriter "YOU DIED", stats, log, hint

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

const CINEMATIC_DUR: f32 = 2.7;
const P1: f32 = 0.7;
const P2: f32 = 1.1;
const P3: f32 = 1.5;
const P4: f32 = 1.8;
const RING_N: usize = 36;
const CRACK_DIRS: usize = 8;
const CRACK_STEPS: usize = 16;
const EMBER_N: usize = 35;
const VOID_EMBER_N: usize = 7;
const TYPEWRITER_CPS: f32 = 25.0;
const LOG_LINES: usize = 8;
const WRAP_W: usize = 55;

// ── Update ───────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, dt: f32) {
    let elapsed = CINEMATIC_DUR - state.title_logo_timer.max(0.0);
    if !state.death_cinematic_done {
        if elapsed < P1 { engine.add_trauma(0.3 * dt); }
        if elapsed >= P1 && elapsed < P2 { engine.add_trauma(0.1 * dt); }
        if state.title_logo_timer <= 0.0 { state.death_cinematic_done = true; }
        return;
    }
    if engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape) {
        crate::state::delete_save();
        state.player = None;
        state.floor = None;
        state.enemy = None;
        state.combat_state = None;
        state.screen = AppScreen::Title;
        state.death_cinematic_done = false;
        state.title_logo_timer = CINEMATIC_DUR;
    }
}

// ── Render ───────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let elapsed = CINEMATIC_DUR - state.title_logo_timer.max(0.0);
    let f = state.frame;

    if      elapsed < P1 { phase1_killing_blow(state, engine, elapsed, f); }
    else if elapsed < P2 { phase2_crack(engine, elapsed, f); }
    else if elapsed < P3 { phase3_collapse(engine, elapsed, f); }
    else if elapsed < P4 { phase4_void(engine, f); }
    else                  { phase5_epitaph(state, engine, theme, elapsed, f); }
}

// ── Phase 1: KILLING BLOW (0.0–0.7s) ────────────────────────────────────────

fn phase1_killing_blow(state: &GameState, engine: &mut ProofEngine, elapsed: f32, _f: u64) {
    let t = elapsed / P1;
    let fade = 1.0 - t;
    // Expanding red ring burst — 36 glyphs in a growing circle
    let radius = t * 22.0;
    let chars = ['░', '▒', '▓', '█', '▓', '▒'];
    for i in 0..RING_N {
        let a = (i as f32 / RING_N as f32) * std::f32::consts::TAU;
        let ci = (i + (t * 6.0) as usize) % chars.len();
        let b = fade * 0.85;
        engine.spawn_glyph(Glyph {
            character: chars[ci],
            position: Vec3::new(a.cos() * radius, a.sin() * radius * 0.55, 0.0),
            color: Vec4::new(b, b * 0.12, 0.0, fade),
            emission: b * 1.8,
            glow_color: Vec3::new(b, 0.04, 0.0),
            glow_radius: 2.0 * fade,
            blend_mode: BlendMode::Additive,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
    // Inner ring — 12 dimmer points
    let ir = t * 10.0;
    for i in 0..12 {
        let a = (i as f32 / 12.0) * std::f32::consts::TAU + 0.3;
        let b = fade * 0.5;
        engine.spawn_glyph(Glyph {
            character: '·',
            position: Vec3::new(a.cos() * ir, a.sin() * ir * 0.55, 0.0),
            color: Vec4::new(b, b * 0.15, 0.0, b),
            emission: b, blend_mode: BlendMode::Additive,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }
    // "KILLING BLOW" label — large orange, fading
    let la = fade.powf(0.5);
    ui_render::text_centered(engine, "KILLING BLOW", 4.2,
        Vec4::new(0.95 * la, 0.45 * la, 0.05, la), 0.7, la * 1.4);
    // Giant damage number centered
    let dmg = extract_last_damage(&state.combat_log);
    let da = fade * 0.95;
    ui_render::text_centered(engine, &dmg, 0.5,
        Vec4::new(da, 0.2 * da, 0.0, da), 1.3, da * 1.6);
    // Killer name below
    let kn = killer_name(state);
    let na = fade * 0.8;
    ui_render::text_centered(engine, &kn, -1.5,
        Vec4::new(0.7 * na, 0.15 * na, 0.0, na), 0.4, na * 0.8);
}

// ── Phase 2: CRACK (0.7–1.1s) ───────────────────────────────────────────────

fn phase2_crack(engine: &mut ProofEngine, elapsed: f32, _f: u64) {
    let t = (elapsed - P1) / (P2 - P1);
    let extent = t * CRACK_STEPS as f32;
    let overall = 1.0 - t * 0.3;
    // 8 fracture lines from center using crack characters
    let dirs: [(f32, f32, char); CRACK_DIRS] = [
        ( 1.0, 0.0,'─'), (-1.0, 0.0,'─'), ( 0.0, 1.0,'│'), ( 0.0,-1.0,'│'),
        ( 0.71, 0.71,'╱'), (-0.71, 0.71,'╲'), ( 0.71,-0.71,'╲'), (-0.71,-0.71,'╱'),
    ];
    for (di, &(dx, dy, ch)) in dirs.iter().enumerate() {
        for s in 0..(extent as usize).min(CRACK_STEPS) {
            let sf = s as f32;
            let df = 1.0 - sf / CRACK_STEPS as f32;
            let b = df * overall * 0.9;
            let om = ((di as f32 * 1.7 + sf * 0.4).sin() * 0.5 + 0.5) * 0.3;
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(dx * sf * 1.1, dy * sf * 0.65, 0.0),
                color: Vec4::new(b * 0.95, b * (0.12 + om), 0.0, b),
                emission: b * 1.2,
                glow_color: Vec3::new(b * 0.8, b * 0.1, 0.0),
                glow_radius: 0.6 * df,
                layer: RenderLayer::Overlay, ..Default::default()
            });
        }
    }
    // Bright center node
    let cp = (1.0 - t) * 0.8;
    engine.spawn_glyph(Glyph {
        character: '╳', position: Vec3::ZERO,
        color: Vec4::new(cp, cp * 0.15, 0.0, cp),
        emission: cp * 2.0, glow_color: Vec3::new(1.0, 0.15, 0.0),
        glow_radius: 3.0 * cp, blend_mode: BlendMode::Additive,
        layer: RenderLayer::Overlay, ..Default::default()
    });
}

// ── Phase 3: COLLAPSE (1.1–1.5s) ────────────────────────────────────────────

fn phase3_collapse(engine: &mut ProofEngine, elapsed: f32, frame: u64) {
    let t = (elapsed - P2) / (P3 - P2);
    let fade = (1.0 - t).max(0.0);
    let local_t = elapsed - P2;
    // Fading fracture remnants
    let ra = fade * 0.3;
    if ra > 0.02 {
        for &(dx, dy, ch) in &[(1.0f32,0.0,'─'),(-1.0,0.0,'─'),(0.0,1.0,'│'),(0.0,-1.0,'│')] {
            for s in 0..6 {
                let sf = s as f32 * 1.1;
                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: Vec3::new(dx * sf, dy * sf * 0.6, 0.0),
                    color: Vec4::new(ra, ra * 0.1, 0.0, ra),
                    emission: ra * 0.5, layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }
    }
    // 35 ember particles with velocity and gravity
    let echar = ['·', '·', '.', '·', ','];
    for i in 0..EMBER_N {
        let seed = i as f32 * 83.7;
        let vx = (seed * 1.13).sin() * 5.5;
        let vy0 = (seed * 0.71).cos() * 3.0 + 2.0;
        let grav = 8.0;
        let x = vx * local_t * 0.6;
        let y = vy0 * local_t - 0.5 * grav * local_t * local_t;
        let b = (fade * (0.4 + (seed * 0.37).cos().abs() * 0.5)).max(0.0);
        if b < 0.02 { continue; }
        let r = b * (0.85 + (seed * 2.3).sin() * 0.15);
        let g = b * (0.15 + (seed * 1.7).cos() * 0.1);
        engine.spawn_glyph(Glyph {
            character: echar[i % echar.len()],
            position: Vec3::new(x, y, 0.0),
            velocity: Vec3::new(vx * 0.1, -grav * local_t * 0.1, 0.0),
            color: Vec4::new(r, g, 0.0, b),
            emission: b * 0.7, glow_color: Vec3::new(r, g * 0.5, 0.0),
            glow_radius: 0.4 * b, blend_mode: BlendMode::Additive,
            layer: RenderLayer::Particle, ..Default::default()
        });
    }
    // Screen darkening overlay
    engine.spawn_glyph(Glyph {
        character: '█', position: Vec3::ZERO, scale: Vec2::splat(20.0),
        color: Vec4::new(0.0, 0.0, 0.0, t * 0.6),
        layer: RenderLayer::Overlay, ..Default::default()
    });
}

// ── Phase 4: VOID (1.5–1.8s) ────────────────────────────────────────────────

fn phase4_void(engine: &mut ProofEngine, frame: u64) {
    // Near-black backdrop
    engine.spawn_glyph(Glyph {
        character: '█', position: Vec3::ZERO, scale: Vec2::splat(25.0),
        color: Vec4::new(0.02, 0.0, 0.0, 0.95),
        layer: RenderLayer::Background, ..Default::default()
    });
    // 7 faint red embers pulsing at screen edges
    let edges: [(f32,f32); VOID_EMBER_N] = [
        (-7.5,-4.0),(7.0,3.5),(-6.0,4.5),(8.0,-3.0),(-8.0,0.5),(6.5,-4.8),(0.5,5.0),
    ];
    for (i, &(ex, ey)) in edges.iter().enumerate() {
        let p = ((frame as f32 * 0.12 + i as f32 * 1.3).sin() * 0.5 + 0.5) * 0.12 + 0.03;
        engine.spawn_glyph(Glyph {
            character: '·',
            position: Vec3::new(ex, ey, 0.0),
            color: Vec4::new(p, p * 0.1, 0.0, p * 1.5),
            emission: p * 0.8, glow_color: Vec3::new(p, 0.02, 0.0),
            glow_radius: 1.0, blend_mode: BlendMode::Additive,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }
    // "..." center dim pulsing
    let dp = ((frame as f32 * 0.15).sin() * 0.5 + 0.5) * 0.12 + 0.03;
    ui_render::text_centered(engine, "...", 0.0,
        Vec4::new(dp, dp * 0.1, 0.0, dp * 2.5), 0.4, dp * 0.6);
}

// ── Phase 5: EPITAPH (1.8s+) ────────────────────────────────────────────────

fn phase5_epitaph(
    state: &GameState, engine: &mut ProofEngine,
    theme: &crate::theme::Theme, elapsed: f32, frame: u64,
) {
    let et = elapsed - P4;
    // Dim background
    engine.spawn_glyph(Glyph {
        character: '█', position: Vec3::ZERO, scale: Vec2::splat(25.0),
        color: Vec4::new(0.015, 0.0, 0.005, 0.92),
        layer: RenderLayer::Background, ..Default::default()
    });

    // ── "Y O U   D I E D" typewriter reveal (1 char per 0.04s) ──────────────
    let title = "Y  O  U     D  I  E  D";
    let reveal = ((et * TYPEWRITER_CPS) as usize).min(title.len());
    let sp = 0.85 * 0.7;
    let tw = title.len() as f32 * sp;
    let tx = -tw * 0.5;
    let ty = 4.8;
    for (i, ch) in title.chars().take(reveal).enumerate() {
        if ch == ' ' { continue; }
        let p = ((frame as f32 * 0.12 + i as f32 * 0.3).sin() * 0.2 + 0.8).max(0.0);
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(tx + i as f32 * sp, ty, 0.0),
            scale: Vec2::splat(0.7),
            color: Vec4::new(p * 0.95, p * 0.08, 0.02, 1.0),
            emission: p * 1.5, glow_color: Vec3::new(0.8, 0.05, 0.0),
            glow_radius: 1.2, layer: RenderLayer::UI, ..Default::default()
        });
    }

    // ── "Killed by: [name]" fades in at +0.5s ───────────────────────────────
    if et > 0.5 {
        let f = ((et - 0.5) * 2.5).min(1.0);
        ui_render::text_centered(engine, &format!("Killed by: {}", killer_name(state)), 3.3,
            Vec4::new(0.7*f, 0.12*f, 0.05*f, f), 0.4, 0.5*f);
    }

    // ── "Final blow: [damage]" fades in at +1.0s ─────────────────────────────
    if et > 1.0 {
        let f = ((et - 1.0) * 2.5).min(1.0);
        ui_render::text_centered(engine,
            &format!("Final blow: {}", extract_last_damage(&state.combat_log)), 2.5,
            Vec4::new(0.6*f, 0.1*f, 0.03*f, f), 0.35, 0.4*f);
    }

    // ── Separator line of ─ characters ───────────────────────────────────────
    if et > 1.2 {
        let f = ((et - 1.2) * 3.0).min(1.0);
        ui_render::text_centered(engine, &"─".repeat(40), 1.8,
            Vec4::new(0.3*f, 0.05*f, 0.02*f, f*0.6), 0.25, 0.2*f);
    }

    // ── Word-wrapped epitaph/summary with typewriter reveal ──────────────────
    if et > 1.4 {
        let rt = et - 1.4;
        let recap = if state.last_recap_text.is_empty() {
            "The mathematics have consumed you.".to_string()
        } else { state.last_recap_text.clone() };
        let wrapped = word_wrap(&recap, WRAP_W);
        for (li, line) in wrapped.iter().enumerate() {
            let ld = li as f32 * 0.15;
            let loc = rt - ld;
            if loc <= 0.0 { break; }
            let vis = ((loc * 30.0) as usize).min(line.len());
            let shown: String = line.chars().take(vis).collect();
            let lf = (loc * 2.0).min(1.0);
            ui_render::text(engine, &shown, -7.0, 1.2 - li as f32 * 0.45,
                Vec4::new(theme.dim.x*lf, theme.dim.y*lf, theme.dim.z*lf, lf*0.85),
                0.3, 0.25*lf);
        }
    }

    // ── Full stat block (floor, kills, gold, damage, spells, corruption, tier)
    if et > 2.0 {
        let sf = ((et - 2.0) * 2.0).min(1.0);
        if let Some(ref p) = state.player {
            let lines = [
                format!("{} -- {} Lv.{}", p.name, p.class.name(), p.level),
                format!("Floor: {} | Kills: {} | Gold: {}", state.floor_num, p.kills, p.gold),
                format!("Damage Dealt: {} | Taken: {}", p.total_damage_dealt, p.total_damage_taken),
                format!("Spells: {} | Items: {} | Corruption: {}", p.spells_cast, p.items_used, p.corruption),
                format!("Power Tier: {:?}", p.power_tier()),
            ];
            // Header separator
            ui_render::text(engine, &"─".repeat(36), -7.0, -0.05,
                Vec4::new(0.25*sf, 0.04*sf, 0.01, sf*0.4), 0.25, 0.15*sf);
            for (i, l) in lines.iter().enumerate() {
                let ld = i as f32 * 0.08;
                let lf = ((et - 2.0 - ld) * 3.0).clamp(0.0, 1.0);
                if lf <= 0.0 { continue; }
                let a = sf * lf;
                ui_render::text(engine, l, -7.0, -0.5 - i as f32 * 0.55,
                    Vec4::new(theme.primary.x*a, theme.primary.y*a, theme.primary.z*a, a),
                    0.3, 0.35*a);
            }
        }
    }

    // ── Last 8 combat log entries ────────────────────────────────────────────
    if et > 2.8 {
        let lf = ((et - 2.8) * 2.0).min(1.0);
        let start = state.combat_log.len().saturating_sub(LOG_LINES);
        ui_render::text(engine, "Combat Log:", -7.0, -3.1,
            Vec4::new(theme.dim.x*lf*0.8, theme.dim.y*lf*0.8, theme.dim.z*lf*0.8, lf*0.7),
            0.3, 0.2*lf);
        for (i, msg) in state.combat_log[start..].iter().enumerate() {
            let ld = i as f32 * 0.05;
            let ilf = ((et - 2.8 - ld) * 3.0).clamp(0.0, 1.0);
            if ilf <= 0.0 { continue; }
            let trunc: String = msg.chars().take(60).collect();
            let a = lf * ilf * 0.75;
            ui_render::text(engine, &trunc, -7.0, -3.6 - i as f32 * 0.4,
                Vec4::new(theme.dim.x*a, theme.dim.y*a, theme.dim.z*a, a), 0.25, 0.15*a);
        }
    }

    // ── "[Enter] Return to Title" hint fades in at the end ───────────────────
    if state.death_cinematic_done && et > 3.5 {
        let hf = ((et - 3.5) * 1.5).min(1.0);
        let hp = ((frame as f32 * 0.06).sin() * 0.3 + 0.7).max(0.0);
        let a = hf * hp;
        ui_render::text_centered(engine, "[Enter] Return to Title", -5.0,
            Vec4::new(a*0.5, a*0.5, a*0.5, a), 0.3, 0.25*a);
    }

    // ── Ambient embers (slow drift) ──────────────────────────────────────────
    for i in 0..5u32 {
        let s = i as f32 * 47.3 + frame as f32 * 0.003;
        let x = (s * 1.7).sin() * 8.0;
        let y = (s * 0.9).cos() * 5.0 - (frame as f32 * 0.005 + i as f32 * 0.3).rem_euclid(10.0) + 5.0;
        let b = ((s * 2.1).sin() * 0.5 + 0.5) * 0.06;
        engine.spawn_glyph(Glyph {
            character: '·', position: Vec3::new(x, y, 0.0),
            color: Vec4::new(b, b * 0.1, 0.0, b), emission: b * 0.4,
            blend_mode: BlendMode::Additive, layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn killer_name(state: &GameState) -> String {
    state.enemy.as_ref().map(|e| e.name.clone()).unwrap_or_else(|| "the unknown".into())
}

fn extract_last_damage(log: &[String]) -> String {
    for entry in log.iter().rev() {
        for word in entry.split_whitespace() {
            if let Ok(n) = word.trim_matches(|c: char| !c.is_ascii_digit()).parse::<i64>() {
                if n > 0 { return format!("{}", n); }
            }
        }
    }
    "???".into()
}

fn word_wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() { cur = word.into(); }
        else if cur.len() + 1 + word.len() <= width { cur.push(' '); cur.push_str(word); }
        else { lines.push(cur); cur = word.into(); }
    }
    if !cur.is_empty() { lines.push(cur); }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}
