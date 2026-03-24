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

// ════════════════════════════════════════════════════════════════════════════════
// PART 2: Status effects, equipment, poses, AmorphousEntity builder, visual state
// ════════════════════════════════════════════════════════════════════════════════

/// Active status effects that modify glyph colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffect { None, Burning, Frozen, Poisoned, Blessed }

/// Equipment tier affecting glyph density and armor chars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentTier { Bare, Light, Medium, Heavy }

/// Map any CharacterClass to one of six visual archetypes.
pub fn class_archetype(class: CharacterClass) -> ClassArchetype {
    match class {
        CharacterClass::Berserker    => ClassArchetype::Berserker,
        CharacterClass::Mage         => ClassArchetype::Mage,
        CharacterClass::Thief        => ClassArchetype::Rogue,
        CharacterClass::Paladin      => ClassArchetype::Cleric,
        CharacterClass::Necromancer  => ClassArchetype::Necromancer,
        CharacterClass::Ranger       => ClassArchetype::Rogue,
        CharacterClass::Alchemist    => ClassArchetype::Mage,
        CharacterClass::VoidWalker   => ClassArchetype::Necromancer,
        CharacterClass::Warlord      => ClassArchetype::Warrior,
        CharacterClass::Trickster    => ClassArchetype::Rogue,
        CharacterClass::Runesmith    => ClassArchetype::Warrior,
        CharacterClass::Chronomancer => ClassArchetype::Mage,
    }
}

fn arch_base_color(a: ClassArchetype) -> Vec4 {
    match a {
        ClassArchetype::Warrior     => Vec4::new(0.7,0.65,0.55,1.0),
        ClassArchetype::Mage        => Vec4::new(0.35,0.25,0.9,1.0),
        ClassArchetype::Rogue       => Vec4::new(0.45,0.45,0.5,1.0),
        ClassArchetype::Cleric      => Vec4::new(0.95,0.9,0.5,1.0),
        ClassArchetype::Necromancer => Vec4::new(0.3,0.65,0.3,1.0),
        ClassArchetype::Berserker   => Vec4::new(0.9,0.25,0.15,1.0),
    }
}

fn arch_accent_color(a: ClassArchetype) -> Vec4 {
    match a {
        ClassArchetype::Warrior     => Vec4::new(0.9,0.85,0.6,1.0),
        ClassArchetype::Mage        => Vec4::new(0.6,0.4,1.0,1.0),
        ClassArchetype::Rogue       => Vec4::new(0.2,0.2,0.25,1.0),
        ClassArchetype::Cleric      => Vec4::new(1.0,1.0,0.85,1.0),
        ClassArchetype::Necromancer => Vec4::new(0.5,0.1,0.6,1.0),
        ClassArchetype::Berserker   => Vec4::new(1.0,0.6,0.1,1.0),
    }
}

fn arch_glow(a: ClassArchetype) -> Vec3 {
    match a {
        ClassArchetype::Warrior     => Vec3::new(0.8,0.7,0.4),
        ClassArchetype::Mage        => Vec3::new(0.4,0.2,1.0),
        ClassArchetype::Rogue       => Vec3::new(0.2,0.2,0.3),
        ClassArchetype::Cleric      => Vec3::new(1.0,0.95,0.6),
        ClassArchetype::Necromancer => Vec3::new(0.3,0.7,0.4),
        ClassArchetype::Berserker   => Vec3::new(1.0,0.3,0.1),
    }
}

fn arch_emission(a: ClassArchetype) -> f32 {
    match a {
        ClassArchetype::Warrior => 0.4, ClassArchetype::Mage => 1.2,
        ClassArchetype::Rogue => 0.15, ClassArchetype::Cleric => 0.9,
        ClassArchetype::Necromancer => 0.7, ClassArchetype::Berserker => 0.8,
    }
}

fn arch_body(a: ClassArchetype) -> &'static [char] {
    match a {
        ClassArchetype::Warrior     => &['|','-','+','=','#','/','\\'],
        ClassArchetype::Mage        => &['*','~','`','\'','.',':',';'],
        ClassArchetype::Rogue       => &['.','`','\'',',',':',';','-'],
        ClassArchetype::Cleric      => &['+','|','-','*','.',':','\''],
        ClassArchetype::Necromancer => &['#','X','x','+','-','|','%'],
        ClassArchetype::Berserker   => &['>','<','!','#','|','/','\\'],
    }
}

fn arch_weapon(a: ClassArchetype) -> &'static [char] {
    match a {
        ClassArchetype::Warrior     => &['|','/','\\','>','<','#'],
        ClassArchetype::Mage        => &['|','*','+','=','^'],
        ClassArchetype::Rogue       => &['/','\\','|','<','>'],
        ClassArchetype::Cleric      => &['+','*','#','!','^'],
        ClassArchetype::Necromancer => &['#','X','+','|','-'],
        ClassArchetype::Berserker   => &['/','\\','|','>','<','X'],
    }
}

fn arch_armor(a: ClassArchetype, tier: EquipmentTier) -> &'static [char] {
    match tier {
        EquipmentTier::Bare  => &['.',' ','`'],
        EquipmentTier::Light => match a {
            ClassArchetype::Warrior|ClassArchetype::Berserker => &['.','-','~'],
            ClassArchetype::Mage|ClassArchetype::Cleric       => &['~','`','\''],
            _ => &['.','`','\''],
        },
        EquipmentTier::Medium => &['=','-','#'],
        EquipmentTier::Heavy  => &['#','=','+','|','-'],
    }
}

/// Full player visual state tracked across frames.
#[derive(Clone)]
pub struct PlayerVisualState {
    pub class: CharacterClass,
    pub archetype: ClassArchetype,
    pub anim_state: PlayerAnimState,
    pub prev_anim_state: PlayerAnimState,
    pub transition_t: f32,
    pub status_effect: StatusEffect,
    pub equipment_tier: EquipmentTier,
    pub weapon_glyph_override: Option<char>,
    pub hp_frac: f32,
    pub level: u32,
    pub hit_reaction_t: f32,
    pub death_t: f32,
    pub level_up_t: f32,
    pub velocity: Vec3,
    pub time: f32,
}

impl PlayerVisualState {
    pub fn new(class: CharacterClass) -> Self {
        Self {
            archetype: class_archetype(class), class,
            anim_state: PlayerAnimState::Idle, prev_anim_state: PlayerAnimState::Idle,
            transition_t: 1.0, status_effect: StatusEffect::None,
            equipment_tier: EquipmentTier::Light, weapon_glyph_override: None,
            hp_frac: 1.0, level: 1, hit_reaction_t: 1.0, death_t: 0.0,
            level_up_t: 1.0, velocity: Vec3::ZERO, time: 0.0,
        }
    }
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;
        if self.transition_t < 1.0 { self.transition_t = (self.transition_t + dt*3.0).min(1.0); }
        if self.hit_reaction_t < 1.0 { self.hit_reaction_t = (self.hit_reaction_t + dt*4.0).min(1.0); }
        if self.anim_state == PlayerAnimState::Death && self.death_t < 1.0 {
            self.death_t = (self.death_t + dt*0.5).min(1.0);
        }
        if self.level_up_t < 1.0 { self.level_up_t = (self.level_up_t + dt*1.5).min(1.0); }
    }
    pub fn set_anim_state(&mut self, state: PlayerAnimState) {
        if state != self.anim_state {
            self.prev_anim_state = self.anim_state;
            self.anim_state = state;
            self.transition_t = 0.0;
        }
    }
    pub fn trigger_hit(&mut self) { self.hit_reaction_t = 0.0; self.set_anim_state(PlayerAnimState::Hurt); }
    pub fn trigger_death(&mut self) { self.death_t = 0.0; self.set_anim_state(PlayerAnimState::Death); }
    pub fn trigger_level_up(&mut self) { self.level_up_t = 0.0; self.level += 1; }
}

/// Render the player with full visual state.
pub fn render_player_full(engine: &mut ProofEngine, state: &PlayerVisualState, position: Vec3, _frame: u64) {
    let arch = state.archetype;
    let count = arch.base_glyph_count() + (state.level as usize / 3);
    let scale = arch.formation_scale();
    let time = state.time;

    let cur_shape = arch.formation_for_state(state.anim_state);
    let prev_shape = arch.formation_for_state(state.prev_anim_state);
    let cur_pos = cur_shape.generate_positions(count, scale);
    let prev_pos = prev_shape.generate_positions(count, scale);

    let st = { let t = state.transition_t.clamp(0.0,1.0); t*t*(3.0 - 2.0*t) };
    let mut positions = formations::interpolate_formations(&prev_pos, &cur_pos, st);
    positions = class_idle(&positions, arch, time);
    positions = formations::apply_hp_drift(&positions, state.hp_frac, time);
    if state.hit_reaction_t < 1.0 {
        positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.5);
    }
    positions = formations::apply_movement_lean(&positions, state.velocity, 0.3);
    if state.anim_state == PlayerAnimState::Death {
        let (dp, _) = formations::death_dissolution(&positions, state.death_t, 42);
        positions = dp;
    }
    if state.level_up_t < 1.0 {
        let (lp, _) = formations::level_up_formation(&positions, state.level_up_t,
            Vec3::new(0.0, scale*1.5, 0.0));
        positions = lp;
    }

    let body = arch_body(arch);
    let weapon = arch_weapon(arch);
    let armor = arch_armor(arch, state.equipment_tier);
    let base_c = arch_base_color(arch);
    let accent_c = arch_accent_color(arch);
    let glow = arch_glow(arch);
    let em = arch_emission(arch);

    let death_alpha = if state.anim_state == PlayerAnimState::Death {
        (1.0 - state.death_t).max(0.0)
    } else { 1.0 };
    let lu_glow = if state.level_up_t < 1.0 {
        if state.level_up_t < 0.5 { state.level_up_t / 0.5 } else { 1.0 - (state.level_up_t - 0.5) / 0.5 }
    } else { 0.0 };

    for (i, p) in positions.iter().enumerate() {
        let wp = position + *p;
        let ch = if i == 0 { state.weapon_glyph_override.unwrap_or(weapon[0]) }
                 else if i < 3 { weapon[i % weapon.len()] }
                 else if p.length() > scale * 0.8 { armor[i % armor.len()] }
                 else { body[i % body.len()] };
        let dist = p.length();
        let t = (dist / (scale * 1.2)).clamp(0.0, 1.0);
        let mut color = lc(accent_c, base_c, t);
        color = status_color(color, state.status_effect, time, i);
        if lu_glow > 0.0 {
            color.x = (color.x + lu_glow*0.4).min(1.0);
            color.y = (color.y + lu_glow*0.4).min(1.0);
            color.z = (color.z + lu_glow*0.2).min(1.0);
        }
        color.w *= death_alpha;
        spg(engine, ch, wp, color, em + lu_glow*0.8, 0.85, glow, if lu_glow > 0.0 { 0.8 } else { 0.4 });
    }
}

fn status_color(base: Vec4, fx: StatusEffect, time: f32, i: usize) -> Vec4 {
    match fx {
        StatusEffect::None => base,
        StatusEffect::Burning  => formations::color_burning(base, time, i),
        StatusEffect::Frozen   => formations::color_frozen(base, time, i),
        StatusEffect::Poisoned => formations::color_poisoned(base, time, i),
        StatusEffect::Blessed  => formations::color_blessed(base, time, i),
    }
}

fn class_idle(positions: &[Vec3], arch: ClassArchetype, time: f32) -> Vec<Vec3> {
    let rate = arch.pulse_rate();
    let depth = arch.pulse_depth();
    match arch {
        ClassArchetype::Warrior => formations::apply_breathing(positions, time, rate, depth),
        ClassArchetype::Mage => {
            let base = formations::apply_breathing(positions, time, rate, depth);
            base.iter().map(|p| {
                let d = p.length();
                if d > 0.8 {
                    let a = time * 0.2 * (d - 0.8);
                    let (s, c) = a.sin_cos();
                    Vec3::new(p.x*c - p.y*s, p.x*s + p.y*c, p.z)
                } else { *p }
            }).collect()
        }
        ClassArchetype::Rogue => positions.iter().enumerate().map(|(i, p)| {
            let ph = time * rate * TAU + i as f32 * 0.5;
            Vec3::new(p.x * (1.0 + ph.sin()*depth*0.7), p.y * (1.0 + (ph+PI*0.5).sin()*depth), p.z)
        }).collect(),
        ClassArchetype::Cleric => positions.iter().map(|p| {
            let w = (time * rate * TAU - p.length() * 2.0).sin() * depth;
            *p * (1.0 + w)
        }).collect(),
        ClassArchetype::Necromancer => {
            let beat = (time * rate * TAU).sin();
            let d = if beat > 0.7 { depth*1.5 } else if beat > 0.3 { depth*0.3 } else { -depth*0.5 };
            positions.iter().map(|p| *p * (1.0 + d)).collect()
        }
        ClassArchetype::Berserker => {
            let rage = (time * rate * TAU).sin().abs() * depth * 1.5;
            positions.iter().enumerate().map(|(i, p)| {
                let j = ((time*12.0 + i as f32*3.7).sin() * rage * 0.3).abs();
                *p * (1.0 + rage + j)
            }).collect()
        }
    }
}

/// Build an AmorphousEntity for the player (formation-backed).
pub fn build_player_entity(class: CharacterClass, position: Vec3) -> AmorphousEntity {
    let arch = class_archetype(class);
    let count = arch.base_glyph_count();
    let scale = arch.formation_scale();
    let shape = arch.idle_formation();
    let positions = shape.generate_positions(count, scale);
    let body = arch_body(arch);
    let weapon = arch_weapon(arch);
    let base_c = arch_base_color(arch);
    let accent_c = arch_accent_color(arch);

    let mut chars = Vec::with_capacity(count);
    let mut colors = Vec::with_capacity(count);
    for (i, p) in positions.iter().enumerate() {
        if i < 3 { chars.push(weapon[i % weapon.len()]); colors.push(accent_c); }
        else {
            chars.push(body[i % body.len()]);
            let t = (p.length() / (scale*1.2)).clamp(0.0, 1.0);
            colors.push(lc(accent_c, base_c, t));
        }
    }
    let mut entity = AmorphousEntity::new(format!("player_{:?}", class), position);
    entity.entity_mass = 50.0;
    entity.pulse_rate = arch.pulse_rate();
    entity.pulse_depth = arch.pulse_depth();
    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity
}

/// Update an existing AmorphousEntity from a PlayerVisualState.
pub fn update_player_entity(entity: &mut AmorphousEntity, state: &PlayerVisualState) {
    let arch = state.archetype;
    let count = arch.base_glyph_count() + (state.level as usize / 3);
    let scale = arch.formation_scale();

    let cur = arch.formation_for_state(state.anim_state).generate_positions(count, scale);
    let prev = arch.formation_for_state(state.prev_anim_state).generate_positions(count, scale);
    let st = { let t = state.transition_t.clamp(0.0,1.0); t*t*(3.0 - 2.0*t) };
    let mut positions = formations::interpolate_formations(&prev, &cur, st);
    positions = class_idle(&positions, arch, state.time);
    positions = formations::apply_hp_drift(&positions, state.hp_frac, state.time);
    if state.hit_reaction_t < 1.0 { positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.5); }
    positions = formations::apply_movement_lean(&positions, state.velocity, 0.3);
    if state.anim_state == PlayerAnimState::Death {
        let (dp, _) = formations::death_dissolution(&positions, state.death_t, 42); positions = dp;
    }
    if state.level_up_t < 1.0 {
        let (lp, _) = formations::level_up_formation(&positions, state.level_up_t, Vec3::new(0.0, scale*1.5, 0.0));
        positions = lp;
    }

    let body = arch_body(arch);
    let weapon = arch_weapon(arch);
    let armor = arch_armor(arch, state.equipment_tier);
    let base_c = arch_base_color(arch);
    let accent_c = arch_accent_color(arch);
    let death_alpha = if state.anim_state == PlayerAnimState::Death { (1.0 - state.death_t).max(0.0) } else { 1.0 };
    let lu_glow = if state.level_up_t < 1.0 {
        if state.level_up_t < 0.5 { state.level_up_t / 0.5 } else { 1.0 - (state.level_up_t - 0.5) / 0.5 }
    } else { 0.0 };

    let len = positions.len();
    let mut chars = Vec::with_capacity(len);
    let mut colors = Vec::with_capacity(len);
    for (i, p) in positions.iter().enumerate() {
        let ch = if i == 0 { state.weapon_glyph_override.unwrap_or(weapon[0]) }
                 else if i < 3 { weapon[i % weapon.len()] }
                 else if p.length() > scale * 0.8 { armor[i % armor.len()] }
                 else { body[i % body.len()] };
        chars.push(ch);
        let t = (p.length() / (scale*1.2)).clamp(0.0, 1.0);
        let mut color = lc(accent_c, base_c, t);
        color = status_color(color, state.status_effect, state.time, i);
        if lu_glow > 0.0 {
            color.x = (color.x + lu_glow*0.4).min(1.0);
            color.y = (color.y + lu_glow*0.4).min(1.0);
            color.z = (color.z + lu_glow*0.2).min(1.0);
        }
        color.w *= death_alpha;
        colors.push(color);
    }
    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity.hp = state.hp_frac * entity.max_hp;
    entity.update_cohesion();
}

/// Map a weapon name to a representative glyph.
pub fn weapon_name_to_glyph(name: &str) -> char {
    let l = name.to_lowercase();
    if l.contains("sword") || l.contains("blade") { '/' }
    else if l.contains("axe") { 'X' }
    else if l.contains("mace") || l.contains("hammer") { '#' }
    else if l.contains("staff") || l.contains("wand") { '|' }
    else if l.contains("dagger") { '\\' }
    else if l.contains("bow") { ')' }
    else if l.contains("spear") { '!' }
    else { '+' }
}

/// Map armor value to tier.
pub fn armor_value_to_tier(armor: u32) -> EquipmentTier {
    match armor { 0 => EquipmentTier::Bare, 1..=10 => EquipmentTier::Light,
                  11..=25 => EquipmentTier::Medium, _ => EquipmentTier::Heavy }
}

fn lc(a: Vec4, b: Vec4, t: f32) -> Vec4 { a + (b - a) * t.clamp(0.0, 1.0) }

/// Return base + accent colors for a class.
pub fn get_class_colors(class: CharacterClass) -> (Vec4, Vec4) {
    let a = class_archetype(class); (arch_base_color(a), arch_accent_color(a))
}

/// Return glyph count for a player at the given level.
pub fn player_glyph_count(class: CharacterClass, level: u32) -> usize {
    class_archetype(class).base_glyph_count() + (level as usize / 3)
}
