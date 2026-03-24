//! Enemy entity rendering — tier-scaled formations.
//!
//! Five enemy tiers with escalating glyph counts, formation complexity,
//! and visual intensity. Enemy name characters used as glyph symbols.
//! All rendering via `engine.spawn_glyph()`.

use proof_engine::prelude::*;
use std::f32::consts::TAU;

// ── Tier enum ────────────────────────────────────────────────────────────────

/// Enemy tier determining visual complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyTier { Minion, Elite, Champion, Boss, Abomination }

impl EnemyTier {
    pub fn from_tier(tier: u32) -> Self {
        match tier {
            0..=1 => Self::Minion, 2..=3 => Self::Elite, 4..=5 => Self::Champion,
            6..=7 => Self::Boss, _ => Self::Abomination,
        }
    }
    fn count(self) -> usize {
        match self { Self::Minion=>10, Self::Elite=>20, Self::Champion=>30,
                     Self::Boss=>55, Self::Abomination=>85 }
    }
    fn em(self) -> f32 {
        match self { Self::Minion=>0.3, Self::Elite=>0.5, Self::Champion=>0.7,
                     Self::Boss=>1.0, Self::Abomination=>1.4 }
    }
    fn gs(self) -> f32 {
        match self { Self::Minion=>0.7, Self::Elite=>0.8, Self::Champion=>0.85,
                     Self::Boss=>0.95, Self::Abomination=>1.05 }
    }
    fn color(self) -> Vec4 {
        match self {
            Self::Minion      => Vec4::new(0.75,0.30,0.25,1.0),
            Self::Elite       => Vec4::new(0.85,0.25,0.20,1.0),
            Self::Champion    => Vec4::new(0.90,0.20,0.15,1.0),
            Self::Boss        => Vec4::new(0.95,0.15,0.10,1.0),
            Self::Abomination => Vec4::new(1.00,0.08,0.08,1.0),
        }
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Render an enemy entity for a single frame.
pub fn render_enemy(
    engine: &mut ProofEngine, name: &str, tier: u32,
    position: Vec3, hp_frac: f32, frame: u64,
) {
    let hp = hp_frac.clamp(0.0, 1.0);
    let t = frame as f32 / 60.0;
    let et = EnemyTier::from_tier(tier);
    let chars = build_palette(name);
    match et {
        EnemyTier::Minion      => render_minion(engine, &chars, position, hp, t, et),
        EnemyTier::Elite       => render_elite(engine, &chars, position, hp, t, et),
        EnemyTier::Champion    => render_champion(engine, &chars, position, hp, t, et),
        EnemyTier::Boss        => render_boss(engine, &chars, position, hp, t, frame, et),
        EnemyTier::Abomination => render_abom(engine, &chars, position, hp, t, frame, et),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn build_palette(name: &str) -> Vec<char> {
    let mut c: Vec<char> = name.chars().filter(|c| !c.is_whitespace()).take(6).collect();
    for &fb in &['\u{2591}','\u{2592}','\u{2593}','\u{2588}','\u{25CF}','\u{25C6}'] {
        if c.len() >= 8 { break; }
        c.push(fb);
    }
    c
}

fn scat(hp: f32, i: usize, t: f32) -> Vec3 {
    let c = (1.0 - hp) * 1.4;
    let s = i as f32 * 1.618;
    Vec3::new((s*3.7+t*1.1).sin()*c, (s*2.3+t*0.9).cos()*c, 0.0)
}

fn br(t: f32, rate: f32, depth: f32) -> f32 {
    1.0 + (t*rate*TAU).sin() * depth
}

fn se(e: &mut ProofEngine, ch: char, p: Vec3, c: Vec4, em: f32, sc: f32) {
    e.spawn_glyph(Glyph {
        character: ch, position: p, color: c, emission: em,
        scale: Vec2::new(sc,sc), layer: RenderLayer::Entity, ..Default::default()
    });
}

fn seg(e: &mut ProofEngine, ch: char, p: Vec3, c: Vec4, em: f32, sc: f32, gc: Vec3, gr: f32) {
    e.spawn_glyph(Glyph {
        character: ch, position: p, color: c, emission: em,
        scale: Vec2::new(sc,sc), glow_color: gc, glow_radius: gr,
        layer: RenderLayer::Entity, ..Default::default()
    });
}

// ── Minion: Simple cluster (10 glyphs) — slow breathing ─────────────────────

fn render_minion(e: &mut ProofEngine, ch: &[char], pos: Vec3, hp: f32, t: f32, tier: EnemyTier) {
    let sc = br(t, 0.5, 0.02);
    let bc = tier.color();
    for i in 0..tier.count() {
        let a = (i as f32 / tier.count() as f32) * TAU + 0.3;
        let r = 0.4 + (i as f32 * 1.618).fract() * 0.6;
        let b = Vec3::new(a.cos()*r, a.sin()*r, 0.0) * sc;
        let d = 0.8 + (t*1.5+i as f32*0.7).sin()*0.15;
        se(e, ch[i%ch.len()], pos+b+scat(hp,i,t),
            Vec4::new(bc.x*d, bc.y*d, bc.z*d, 1.0), tier.em(), tier.gs());
    }
}

// ── Elite: Ring formation (20 glyphs) — rotating, pulsing ───────────────────

fn render_elite(e: &mut ProofEngine, ch: &[char], pos: Vec3, hp: f32, t: f32, tier: EnemyTier) {
    let sc = br(t, 0.7, 0.03);
    let bc = tier.color();
    for i in 0..tier.count() {
        let a = (i as f32/tier.count() as f32)*TAU + t*0.6;
        let b = Vec3::new(a.cos()*1.2, a.sin()*1.2, 0.0) * sc;
        let p = ((t*2.5+i as f32*0.4).sin()*0.2+0.8).max(0.3);
        seg(e, ch[i%ch.len()], pos+b+scat(hp,i,t),
            Vec4::new(bc.x*p, bc.y*p, bc.z*p, 1.0), tier.em()*p, tier.gs(),
            Vec3::new(bc.x, bc.y*0.5, bc.z*0.3), 0.4);
    }
}

// ── Champion: Double ring (30 glyphs) — counter-rotating ────────────────────

fn render_champion(e: &mut ProofEngine, ch: &[char], pos: Vec3, hp: f32, t: f32, tier: EnemyTier) {
    let sc = br(t, 0.8, 0.035);
    let bc = tier.color();
    let mut idx = 0usize;
    // Outer ring — 18
    for i in 0..18 {
        let a = (i as f32/18.0)*TAU + t*0.5;
        let b = Vec3::new(a.cos()*1.6, a.sin()*1.6, 0.0) * sc;
        let p = ((t*2.0+i as f32*0.5).sin()*0.15+0.85).max(0.3);
        seg(e, ch[idx%ch.len()], pos+b+scat(hp,idx,t),
            Vec4::new(bc.x*p,bc.y*p,bc.z*p,1.0), tier.em()*p, tier.gs(),
            Vec3::new(0.9,0.2,0.1), 0.5);
        idx += 1;
    }
    // Inner ring — 12, counter-rotating
    for i in 0..12 {
        let a = (i as f32/12.0)*TAU - t*0.7;
        let b = Vec3::new(a.cos()*0.8, a.sin()*0.8, 0.0) * sc;
        let bv = ((t*3.0+i as f32*0.8).sin()*0.2+0.9).max(0.4);
        seg(e, ch[idx%ch.len()], pos+b+scat(hp,idx,t),
            Vec4::new((bc.x*bv).min(1.0), bc.y*bv*1.2, bc.z*bv, 1.0),
            tier.em()*1.2, tier.gs()*1.05, Vec3::new(1.0,0.3,0.15), 0.6);
        idx += 1;
    }
}

// ── Boss: Star core + helix + crown (55 glyphs) ─────────────────────────────

fn render_boss(
    e: &mut ProofEngine, ch: &[char], pos: Vec3, hp: f32, t: f32, fr: u64, tier: EnemyTier,
) {
    let sc = br(t, 0.6, 0.04);
    let bc = tier.color();
    let mut idx = 0usize;
    // Star core — 5 arms x 3 = 15
    for arm in 0..5 {
        let aa = (arm as f32/5.0)*TAU + t*0.3;
        for d in 0..3 {
            let r = (d as f32+1.0)*0.5;
            let b = Vec3::new(aa.cos()*r, aa.sin()*r, 0.0)*sc;
            let int = 1.0 - d as f32*0.15;
            seg(e, ch[idx%ch.len()], pos+b+scat(hp,idx,t),
                Vec4::new(bc.x*int, bc.y*int+0.1, bc.z*int, 1.0),
                tier.em()*int, tier.gs(), Vec3::new(1.0,0.2,0.1), 0.8);
            idx += 1;
        }
    }
    // Double helix — 2 strands x 16 = 32
    for strand in 0..2 {
        let phase = strand as f32 * std::f32::consts::PI;
        for i in 0..16 {
            let tp = i as f32/16.0;
            let a = tp*TAU*2.0 + t*0.8 + phase;
            let hr = 0.5 + tp*0.3;
            let yo = (tp-0.5)*3.5;
            let b = Vec3::new(a.cos()*hr, yo+a.sin()*0.3, 0.0)*sc;
            let w = ((t*2.5+i as f32*0.6).sin()*0.2+0.8).max(0.3);
            let st = if strand==0 { 0.0 } else { 0.15 };
            seg(e, ch[idx%ch.len()], pos+b+scat(hp,idx,t),
                Vec4::new(bc.x*w, bc.y*w+st, bc.z*w+st*0.5, 0.9),
                tier.em()*w, tier.gs()*0.9, Vec3::new(1.0,0.15,0.05), 0.6);
            idx += 1;
        }
    }
    // Crown — 8 orbiting above
    for i in 0..8 {
        let a = (i as f32/8.0)*TAU + t*1.2;
        let b = Vec3::new(a.cos()*0.6, 2.2+(t*1.5).sin()*0.2, 0.0)*sc;
        let fl = ((fr as f32*0.1+i as f32).sin()*0.3+0.7).max(0.3);
        seg(e, ch[idx%ch.len()], pos+b+scat(hp,idx,t),
            Vec4::new(fl, 0.8*fl, 0.2*fl, 1.0),
            tier.em()*1.5, tier.gs()*1.1, Vec3::new(1.0,0.7,0.1), 1.0);
        idx += 1;
    }
}

// ── Abomination: Massive chaotic mass (85 glyphs) ──────────────────────────

fn render_abom(
    e: &mut ProofEngine, ch: &[char], pos: Vec3, hp: f32, t: f32, fr: u64, tier: EnemyTier,
) {
    let sc = br(t, 0.5, 0.05);
    let bc = tier.color();
    let mut idx = 0usize;
    // Pulsing core — 10
    for i in 0..10 {
        let a = (i as f32/10.0)*TAU;
        let r = 0.35 + (t*3.0+i as f32*0.9).sin().abs()*0.2;
        let b = Vec3::new(a.cos()*r, a.sin()*r, 0.0)*sc;
        let th = ((t*4.0+i as f32).sin()*0.3+0.7).max(0.2);
        seg(e, ch[idx%ch.len()], pos+b+scat(hp,idx,t),
            Vec4::new(th, 0.05, 0.05, 1.0),
            tier.em()*1.8*th, tier.gs()*1.1, Vec3::new(1.0,0.1,0.05), 1.2);
        idx += 1;
    }
    // Three rings: 15 + 20 + 25 = 60
    for &(cnt, rad, rs) in &[(15usize,1.0f32,0.4f32),(20,1.8,-0.3),(25,2.6,0.2)] {
        for i in 0..cnt {
            let a = (i as f32/cnt as f32)*TAU + t*rs;
            let wb = ((t*2.0+idx as f32*1.3).sin()*0.15).abs();
            let r = rad + wb;
            let b = Vec3::new(a.cos()*r, a.sin()*r, 0.0)*sc;
            let w = ((t*1.8+idx as f32*0.4).sin()*0.2+0.8).max(0.3);
            seg(e, ch[idx%ch.len()], pos+b+scat(hp,idx,t),
                Vec4::new(bc.x*w, bc.y*w, bc.z*w, 0.9),
                tier.em()*w, tier.gs(), Vec3::new(0.9,0.1,0.08), 0.6);
            idx += 1;
        }
    }
    // Chaotic tendrils — 15
    for i in 0..15 {
        let sd = i as f32 * 2.618;
        let ta = sd*TAU*0.618 + t*0.15;
        let reach = 3.0 + (t*1.2+sd).sin()*0.8;
        let lat = (t*2.5+sd*3.0).sin()*0.4;
        let b = Vec3::new(ta.cos()*reach+lat, ta.sin()*reach, 0.0)*sc;
        let fd = ((t*3.0+sd).sin()*0.3+0.6).max(0.15);
        e.spawn_glyph(Glyph {
            character: ch[idx%ch.len()], position: pos+b+scat(hp,idx,t),
            color: Vec4::new(bc.x*fd, bc.y*fd+0.05, bc.z*fd+0.08, 0.7),
            emission: tier.em()*fd*1.3, scale: Vec2::new(tier.gs()*0.8, tier.gs()*0.8),
            glow_color: Vec3::new(1.0,0.15,0.1), glow_radius: 0.4,
            temperature: 0.8, entropy: 0.6,
            visible: (fr+i as u64) % 3 != 0,
            layer: RenderLayer::Entity, ..Default::default()
        });
        idx += 1;
    }
}
