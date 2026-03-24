//! Player entity rendering — 12 unique class formations.
//!
//! Each `CharacterClass` gets a visually distinct glyph formation with
//! class-specific symbols, color palettes, idle animations, and HP-linked
//! cohesion dynamics. All rendering via `engine.spawn_glyph()`.

use proof_engine::prelude::*;
use chaos_rpg_core::character::CharacterClass;
use std::f32::consts::{PI, TAU};

use super::formations::{self, ClassArchetype, PlayerAnimState};

/// Render the player entity for a single frame.
pub fn render_player(
    engine: &mut ProofEngine, class: CharacterClass,
    position: Vec3, hp_frac: f32, frame: u64,
) {
    let hp = hp_frac.clamp(0.0, 1.0);
    let t = frame as f32 / 60.0;
    match class {
        CharacterClass::Mage        => render_mage(engine, position, hp, t, frame),
        CharacterClass::Berserker   => render_berserker(engine, position, hp, t),
        CharacterClass::Ranger      => render_ranger(engine, position, hp, t),
        CharacterClass::Thief       => render_thief(engine, position, hp, t),
        CharacterClass::Necromancer => render_necromancer(engine, position, hp, t, frame),
        CharacterClass::Alchemist   => render_alchemist(engine, position, hp, t),
        CharacterClass::Paladin     => render_paladin(engine, position, hp, t),
        CharacterClass::VoidWalker  => render_voidwalker(engine, position, hp, t, frame),
        CharacterClass::Warlord     => render_warlord(engine, position, hp, t),
        CharacterClass::Trickster   => render_trickster(engine, position, hp, t, frame),
        CharacterClass::Runesmith   => render_runesmith(engine, position, hp, t),
        CharacterClass::Chronomancer => render_chronomancer(engine, position, hp, t),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn scat(hp: f32, i: usize, t: f32) -> Vec3 {
    let c = (1.0 - hp) * 1.2;
    let s = i as f32 * 1.618;
    Vec3::new((s*3.7+t*1.1).sin()*c, (s*2.3+t*0.9).cos()*c, 0.0)
}

fn br(t: f32, rate: f32, depth: f32) -> f32 {
    1.0 + (t * rate * TAU).sin() * depth
}

fn sp(e: &mut ProofEngine, ch: char, p: Vec3, c: Vec4, em: f32, sc: f32) {
    e.spawn_glyph(Glyph {
        character: ch, position: p, color: c, emission: em,
        scale: Vec2::new(sc, sc), layer: RenderLayer::Entity, ..Default::default()
    });
}

fn spg(e: &mut ProofEngine, ch: char, p: Vec3, c: Vec4, em: f32, sc: f32, gc: Vec3, gr: f32) {
    e.spawn_glyph(Glyph {
        character: ch, position: p, color: c, emission: em,
        scale: Vec2::new(sc, sc), glow_color: gc, glow_radius: gr,
        layer: RenderLayer::Entity, ..Default::default()
    });
}

// ── Mage: Loose diamond (25 glyphs) — blue-purple, orbiting edge symbols ────

fn render_mage(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32, fr: u64) {
    let gl: &[char] = &['*', '\u{25C6}', '\u{221E}', '\u{2202}', '\u{2211}'];
    let sc = br(t, 0.8, 0.04);
    let mut i = 0usize;
    for dy in -3i32..=3 {
        let w = 3 - dy.abs();
        for dx in -w..=w {
            let b = Vec3::new(dx as f32 * 0.7, dy as f32 * 0.6, 0.0) * sc;
            let pulse = ((t*3.0 + i as f32*0.5).sin()*0.15 + 0.85).max(0.0);
            let c = Vec4::new(0.35*pulse, 0.25*pulse, 0.95*pulse, 1.0);
            spg(e, gl[i%5], pos+b+scat(hp,i,t), c, 1.2, 0.9, Vec3::new(0.4,0.2,1.0), 0.6);
            i += 1;
        }
    }
    let orb: &[char] = &['\u{2206}', '\u{03A9}', '\u{03C0}', '\u{222B}'];
    for j in 0..4 {
        let a = (j as f32/4.0)*TAU + t*1.5;
        let r = 2.2 + (t*2.0+j as f32).sin()*0.3;
        let p = pos + Vec3::new(a.cos()*r, a.sin()*r, 0.0) + scat(hp, i+j, t);
        let f = ((fr as f32*0.2 + j as f32*1.2).sin()*0.3 + 0.7).max(0.0);
        spg(e, orb[j], p, Vec4::new(0.6*f,0.3*f,f,0.9), 1.8, 1.1, Vec3::new(0.5,0.2,1.0), 0.9);
    }
}

// ── Berserker: Tight aggressive cluster (30 glyphs) — red, rage at <30% HP ──

fn render_berserker(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let gl: &[char] = &['>', '<', '!', '#', '\u{2588}'];
    let rage = if hp < 0.3 { 2.0 } else { 1.0 };
    let em = 0.8 * rage;
    let sc = br(t, 1.2, 0.03);
    let mut i = 0usize;
    for row in -2..=2 {
        for col in -2..=3 {
            let jx = (i as f32*2.71 + t*4.0).sin() * 0.06 * rage;
            let jy = (i as f32*3.14 + t*3.5).cos() * 0.06 * rage;
            let b = Vec3::new(col as f32*0.45+jx, row as f32*0.45+jy, 0.0) * sc;
            let rv = (0.85*rage).min(1.0);
            let fl = if hp < 0.3 { ((t*8.0+i as f32).sin()*0.2+0.8).max(0.4) } else { 1.0 };
            let c = Vec4::new(rv*fl, 0.15*fl, 0.1*fl, 1.0);
            spg(e, gl[i%5], pos+b+scat(hp,i,t), c, em, 0.85,
                Vec3::new(1.0,0.15,0.05)*rage, 0.5*rage);
            i += 1;
        }
    }
}

// ── Ranger: Arrow formation (20 glyphs) — green, precise chevron ────────────

fn render_ranger(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let gl: &[char] = &['/', '\\', '|', '\u{2192}'];
    let sc = br(t, 0.6, 0.03);
    let offs: [(f32,f32); 20] = [
        (2.0,0.0),(1.4,0.4),(0.8,0.8),(0.2,1.2),(-0.4,1.6),
        (1.4,-0.4),(0.8,-0.8),(0.2,-1.2),(-0.4,-1.6),
        (1.0,0.0),(0.4,0.0),(-0.2,0.0),(-0.8,0.0),(-1.4,0.0),
        (1.0,0.25),(1.0,-0.25),(0.4,0.5),(0.4,-0.5),(-1.4,0.3),(-1.4,-0.3),
    ];
    for (i, &(ox,oy)) in offs.iter().enumerate() {
        let b = Vec3::new(ox, oy, 0.0) * sc * 0.7;
        sp(e, gl[i%4], pos+b+scat(hp,i,t), Vec4::new(0.25,0.82,0.2,1.0), 0.5, 0.8);
    }
}

// ── Thief: Small compact cluster (15 glyphs) — gray, dim stealth ────────────

fn render_thief(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let gl: &[char] = &['.', '\u{00B7}', '~', '-'];
    let sc = br(t, 0.5, 0.02);
    let mut i = 0usize;
    for ring in 0..3 {
        let r = (ring as f32 + 1.0) * 0.35;
        let cnt = if ring == 0 { 3 } else { 6 };
        for j in 0..cnt {
            if i >= 15 { break; }
            let a = (j as f32 / cnt as f32) * TAU + ring as f32 * 0.3;
            let b = Vec3::new(a.cos()*r, a.sin()*r, 0.0) * sc;
            let d = ((t*1.5+i as f32*0.9).sin()*0.1+0.4).max(0.2);
            sp(e, gl[i%4], pos+b+scat(hp,i,t), Vec4::new(0.5*d,0.5*d,0.5*d,0.7), 0.15, 0.65);
            i += 1;
        }
    }
}

// ── Necromancer: Ring with dark center (25 glyphs) — green/purple, wisps ────

fn render_necromancer(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32, fr: u64) {
    let out: &[char] = &['\u{2620}', '\u{2020}', '\u{00B7}', '\u{25CB}'];
    let sc = br(t, 0.7, 0.03);
    let mut i = 0usize;
    for j in 0..5 {
        let a = (j as f32/5.0)*TAU;
        let b = Vec3::new(a.cos()*0.25, a.sin()*0.25, 0.0) * sc;
        sp(e, '\u{00B7}', pos+b+scat(hp,i,t), Vec4::new(0.1,0.05,0.15,0.6), 0.1, 0.7);
        i += 1;
    }
    for j in 0..16 {
        let a = (j as f32/16.0)*TAU + t*0.4;
        let b = Vec3::new(a.cos()*1.4, a.sin()*1.4, 0.0) * sc;
        let p = ((t*2.0+j as f32*0.7).sin()*0.2+0.8).max(0.0);
        spg(e, out[i%4], pos+b+scat(hp,i,t), Vec4::new(0.2*p,0.6*p,0.25*p,1.0),
            0.7, 0.85, Vec3::new(0.3,0.7,0.4), 0.5);
        i += 1;
    }
    for j in 0..4 {
        let a = (j as f32/4.0)*TAU - t*0.3;
        let b = Vec3::new(a.cos()*0.7, a.sin()*0.7, 0.0) * sc;
        spg(e, '\u{2620}', pos+b+scat(hp,i,t), Vec4::new(0.5,0.2,0.6,0.9),
            0.9, 0.95, Vec3::new(0.6,0.1,0.8), 0.6);
        i += 1;
    }
    if fr % 8 == 0 {
        let wa = t*2.5;
        let wr = 1.8 + (t*1.3).sin()*0.5;
        e.spawn_glyph(Glyph {
            character: '\u{2022}',
            position: pos + Vec3::new(wa.cos()*wr, wa.sin()*wr+0.5, 0.0),
            color: Vec4::new(0.4,0.9,0.5,0.5), emission: 1.5,
            glow_color: Vec3::new(0.3,1.0,0.4), glow_radius: 1.0,
            lifetime: 0.6, life_function: Some(MathFunction::Breathing { rate: 4.0, depth: 0.6 }),
            layer: RenderLayer::Particle, blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ── Alchemist: Bubbling formation (20 glyphs) — purple/gold ─────────────────

fn render_alchemist(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let gl: &[char] = &['~', '\u{2248}', '\u{25CB}', '\u{25CF}'];
    let sc = br(t, 0.9, 0.04);
    let mut i = 0usize;
    for j in 0..6 {
        let x = (j as f32 - 2.5) * 0.5;
        let by = ((t*3.0+j as f32*1.1).sin()*0.12).abs();
        sp(e, gl[i%4], pos+Vec3::new(x,-1.0+by,0.0)*sc+scat(hp,i,t),
            Vec4::new(0.65,0.45,0.9,1.0), 0.6, 0.85);
        i += 1;
    }
    for row in 0..2 {
        let hw = 2 - row;
        for col in -hw..=hw {
            let by = ((t*2.5+i as f32*0.8).sin()*0.1).abs();
            let gm = (i as f32*0.3+t).sin()*0.5+0.5;
            sp(e, gl[i%4], pos+Vec3::new(col as f32*0.5, row as f32*0.5+by, 0.0)*sc+scat(hp,i,t),
                Vec4::new(0.65+0.25*gm, 0.4+0.4*gm, 0.9-0.5*gm, 1.0), 0.7, 0.85);
            i += 1;
        }
    }
    while i < 20 {
        let j = i - 14;
        let a = (j as f32/6.0)*TAU + t*1.8;
        sp(e, '\u{25CB}', pos+Vec3::new(a.cos()*0.4, 1.2+a.sin()*0.4, 0.0)*sc+scat(hp,i,t),
            Vec4::new(0.85,0.75,0.3,0.8), 0.9, 0.7);
        i += 1;
    }
}

// ── Paladin: Cross/shield (25 glyphs) — golden, steady glow ─────────────────

fn render_paladin(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let gl: &[char] = &['+', '\u{2020}', '\u{25A0}', '\u{2588}'];
    let sc = br(t, 0.5, 0.02);
    let g = Vec3::new(0.95, 0.85, 0.35);
    let mut i = 0usize;
    for row in -4..=4 {
        let b = Vec3::new(0.0, row as f32*0.45, 0.0) * sc;
        let bv = 0.85 + (t*1.5+i as f32).sin()*0.1;
        spg(e, gl[i%4], pos+b+scat(hp,i,t), Vec4::new(g.x*bv,g.y*bv,g.z*bv,1.0),
            0.9, 0.9, g, 0.5);
        i += 1;
    }
    for col in -4..=4 {
        if col == 0 { continue; }
        let b = Vec3::new(col as f32*0.45, 0.225, 0.0) * sc;
        let bv = 0.85 + (t*1.5+i as f32).sin()*0.1;
        spg(e, gl[i%4], pos+b+scat(hp,i,t), Vec4::new(g.x*bv,g.y*bv,g.z*bv,1.0),
            0.9, 0.9, g, 0.5);
        i += 1;
    }
    for &(cx,cy) in &[(-1.0f32,1.0),(1.0,1.0),(-1.0,-1.0),(1.0,-1.0),
                       (-1.0,0.0),(1.0,0.0),(0.0,1.5),(0.0,-1.5)] {
        let b = Vec3::new(cx*0.7, cy*0.7, 0.0) * sc;
        spg(e, '\u{25A0}', pos+b+scat(hp,i,t), Vec4::new(g.x*0.7,g.y*0.7,g.z*0.5,0.85),
            0.6, 0.8, g*0.7, 0.3);
        i += 1;
    }
}

// ── VoidWalker: Phase-in/out (20 glyphs) — purple, gaps ─────────────────────

fn render_voidwalker(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32, fr: u64) {
    let gl: &[char] = &['\u{2591}', '\u{2592}', '\u{2593}', '\u{00B7}', '~'];
    let sc = br(t, 0.6, 0.05);
    let mut idx = 0usize;
    for i in 0u64..20 {
        let period = 17 + (i % 7) * 3;
        if ((fr + i*5) % period) <= period / 3 { idx += 1; continue; }
        let a = (i as f32/20.0)*TAU + t*0.5;
        let gap = ((i as f32*1.618 + t*2.0).sin()*0.4).abs();
        let r = 1.2 + gap;
        let b = Vec3::new(a.cos()*r, a.sin()*r, 0.0) * sc;
        let fl = ((t*5.0+i as f32*2.1).sin()*0.3+0.6).max(0.1);
        spg(e, gl[idx%5], pos+b+scat(hp,idx,t), Vec4::new(0.55*fl,0.15*fl,0.85*fl,0.7),
            1.0, 0.85, Vec3::new(0.6,0.1,0.9), 0.7);
        idx += 1;
    }
}

// ── Warlord: Military grid (30 glyphs) — steel gray ─────────────────────────

fn render_warlord(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let gl: &[char] = &['|', '\u{2500}', '\u{25A0}', '\u{25A1}'];
    let sc = br(t, 0.4, 0.015);
    let st = Vec4::new(0.6, 0.62, 0.65, 1.0);
    let mut i = 0usize;
    for row in -2i32..=2 {
        for col in -2i32..=3 {
            let b = Vec3::new(col as f32*0.5-0.25, row as f32*0.5, 0.0) * sc;
            let rb = 1.0 - row.abs() as f32 * 0.08;
            sp(e, gl[i%4], pos+b+scat(hp,i,t),
                Vec4::new(st.x*rb, st.y*rb, st.z*rb, 1.0), 0.35, 0.85);
            i += 1;
        }
    }
}

// ── Trickster: Shifting positions (15 glyphs) — multi-colored ───────────────

fn render_trickster(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32, fr: u64) {
    let gl: &[char] = &['?', '!', '@', '&', '%'];
    let sc = br(t, 1.0, 0.04);
    let bp: [(f32,f32); 15] = [
        (0.0,0.0),(0.6,0.3),(-0.6,0.3),(0.6,-0.3),(-0.6,-0.3),
        (0.0,0.7),(0.0,-0.7),(1.0,0.0),(-1.0,0.0),
        (0.3,1.0),(-0.3,1.0),(0.3,-1.0),(-0.3,-1.0),(1.0,0.6),(-1.0,-0.6),
    ];
    let shift = (fr % 15) as usize;
    for i in 0..15 {
        let (ox,oy) = bp[(i+shift)%15];
        sp(e, gl[i%5], pos+Vec3::new(ox,oy,0.0)*sc+scat(hp,i,t), hue_col(i, fr), 0.8, 0.85);
    }
}

fn hue_col(i: usize, fr: u64) -> Vec4 {
    let h = ((i as f32 + fr as f32 * 0.1) % 5.0) / 5.0;
    let h6 = h*6.0; let f = h6 - h6.floor();
    let (r,g,b) = match h6 as u32 % 6 {
        0=>(1.0,f,0.0), 1=>(1.0-f,1.0,0.0), 2=>(0.0,1.0,f),
        3=>(0.0,1.0-f,1.0), 4=>(f,0.0,1.0), _=>(1.0,0.0,1.0-f),
    };
    Vec4::new(r, g, b, 1.0)
}

// ── Runesmith: Runic circle (25 glyphs) — orange/amber ──────────────────────

fn render_runesmith(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let ru: &[char] = &['\u{16B1}', '\u{16A2}', '\u{16BE}', '#'];
    let sc = br(t, 0.6, 0.03);
    let am = Vec3::new(0.95, 0.65, 0.2);
    let mut i = 0usize;
    for j in 0..16 {
        let a = (j as f32/16.0)*TAU + t*0.25;
        let b = Vec3::new(a.cos()*1.5, a.sin()*1.5, 0.0) * sc;
        let h = ((t*2.0+j as f32*0.5).sin()*0.15+0.85).max(0.0);
        spg(e, ru[i%4], pos+b+scat(hp,i,t), Vec4::new(am.x*h,am.y*h,am.z*h*0.7,1.0),
            0.9, 0.9, Vec3::new(1.0,0.5,0.1), 0.5);
        i += 1;
    }
    for j in 0..6 {
        let a = (j as f32/6.0)*TAU - t*0.4;
        let b = Vec3::new(a.cos()*0.7, a.sin()*0.7, 0.0) * sc;
        spg(e, ru[i%4], pos+b+scat(hp,i,t), Vec4::new(1.0,0.75,0.3,1.0),
            1.1, 1.0, Vec3::new(1.0,0.6,0.15), 0.6);
        i += 1;
    }
    for j in 0..3 {
        let w = (t*1.5+j as f32*TAU/3.0).sin()*0.15;
        spg(e, '#', pos+Vec3::new(w,w*0.5,0.0)*sc+scat(hp,i,t),
            Vec4::new(1.0,0.85,0.4,1.0), 1.4, 1.1, Vec3::new(1.0,0.7,0.2), 0.8);
        i += 1;
    }
}

// ── Chronomancer: Time-offset breathing (20 glyphs) — blue/white ────────────

fn render_chronomancer(e: &mut ProofEngine, pos: Vec3, hp: f32, t: f32) {
    let gl: &[char] = &['\u{29D6}', '\u{25CB}', '\u{00B7}', '\u{2234}'];
    for i in 0..20 {
        let a = (i as f32/20.0) * TAU;
        let ph = (i as f32/20.0) * TAU;
        let ls = 1.0 + (t*1.2+ph).sin()*0.12;
        let r = 1.3 * ls;
        let b = Vec3::new(a.cos()*r, a.sin()*r, 0.0);
        let wm = (20-i) as f32 / 20.0;
        let al = (0.85 + (t*1.2+ph).sin()*0.15).clamp(0.0, 1.0);
        let em = (0.7 + (t*1.2+ph).sin()*0.4).max(0.2);
        spg(e, gl[i%4], pos+b+scat(hp,i,t),
            Vec4::new(0.4+0.6*wm, 0.5+0.5*wm, 0.95, al), em, 0.85,
            Vec3::new(0.5,0.7,1.0), 0.5);
    }
}
