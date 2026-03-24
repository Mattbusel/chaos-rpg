//! Enemy entity rendering — tier-scaled formations.
//!
//! Five enemy tiers with escalating glyph counts, formation complexity,
//! and visual intensity. Enemy name characters used as glyph symbols.
//! All rendering via `engine.spawn_glyph()`.

use proof_engine::prelude::*;
use std::f32::consts::{PI, TAU};

use super::formations::{self, FormationShape, ElementalDeathStyle};

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

// ════════════════════════════════════════════════════════════════════════════════
// PART 2: Element system, boss profiles, spawn/death, AmorphousEntity builder
// ════════════════════════════════════════════════════════════════════════════════

/// Element type driving enemy visual theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyElement { Fire, Ice, Lightning, Poison, Shadow, Holy, Neutral }

impl EnemyElement {
    pub fn primary_color(&self) -> Vec4 {
        match self {
            Self::Fire=>Vec4::new(1.0,0.45,0.1,1.0), Self::Ice=>Vec4::new(0.5,0.75,1.0,1.0),
            Self::Lightning=>Vec4::new(1.0,1.0,0.3,1.0), Self::Poison=>Vec4::new(0.3,0.8,0.2,1.0),
            Self::Shadow=>Vec4::new(0.3,0.15,0.45,1.0), Self::Holy=>Vec4::new(1.0,0.95,0.7,1.0),
            Self::Neutral=>Vec4::new(0.7,0.25,0.2,1.0),
        }
    }
    pub fn accent_color(&self) -> Vec4 {
        match self {
            Self::Fire=>Vec4::new(1.0,0.7,0.0,1.0), Self::Ice=>Vec4::new(0.8,0.9,1.0,1.0),
            Self::Lightning=>Vec4::new(1.0,1.0,0.8,1.0), Self::Poison=>Vec4::new(0.5,0.2,0.7,1.0),
            Self::Shadow=>Vec4::new(0.15,0.05,0.25,1.0), Self::Holy=>Vec4::new(1.0,1.0,1.0,1.0),
            Self::Neutral=>Vec4::new(0.9,0.4,0.3,1.0),
        }
    }
    pub fn glyph_palette(&self) -> &'static [char] {
        match self {
            Self::Fire=>&['^','*','~','#','!','v','>','<'],
            Self::Ice=>&['*','+','.',':','#','=','-','o'],
            Self::Lightning=>&['!','/','\\','X','+','#','|','-'],
            Self::Poison=>&['~','.',':',';','?','%','&','S'],
            Self::Shadow=>&['.',' ',':','`','\'',',','-','~'],
            Self::Holy=>&['*','+','.','\'',':','!','#','^'],
            Self::Neutral=>&['#','X','+','-','|','/','\\','.'],
        }
    }
    pub fn death_style(&self) -> ElementalDeathStyle {
        match self {
            Self::Fire=>ElementalDeathStyle::Fire, Self::Ice=>ElementalDeathStyle::Ice,
            Self::Lightning=>ElementalDeathStyle::Lightning, Self::Poison=>ElementalDeathStyle::Poison,
            Self::Shadow=>ElementalDeathStyle::Shadow, Self::Holy=>ElementalDeathStyle::Holy,
            Self::Neutral=>ElementalDeathStyle::Default,
        }
    }
    pub fn glow_color(&self) -> Vec3 {
        match self {
            Self::Fire=>Vec3::new(1.0,0.4,0.05), Self::Ice=>Vec3::new(0.4,0.7,1.0),
            Self::Lightning=>Vec3::new(1.0,1.0,0.5), Self::Poison=>Vec3::new(0.3,0.8,0.2),
            Self::Shadow=>Vec3::new(0.2,0.05,0.3), Self::Holy=>Vec3::new(1.0,0.95,0.7),
            Self::Neutral=>Vec3::new(0.8,0.2,0.1),
        }
    }
    pub fn preferred_formation(&self) -> FormationShape {
        match self {
            Self::Fire=>FormationShape::Triangle, Self::Ice=>FormationShape::Diamond,
            Self::Lightning=>FormationShape::Star, Self::Poison=>FormationShape::Spiral,
            Self::Shadow=>FormationShape::Crescent, Self::Holy=>FormationShape::Ring,
            Self::Neutral=>FormationShape::Cluster,
        }
    }
    pub fn emission(&self) -> f32 {
        match self { Self::Fire=>0.5, Self::Ice=>0.3, Self::Lightning=>0.6, Self::Poison=>0.2,
                     Self::Shadow=>0.1, Self::Holy=>0.5, Self::Neutral=>0.15 }
    }
}

/// Guess an element from an enemy name.
pub fn element_from_name(name: &str) -> EnemyElement {
    let l = name.to_lowercase();
    if l.contains("fire")||l.contains("flame")||l.contains("ember")||l.contains("pyro") { EnemyElement::Fire }
    else if l.contains("ice")||l.contains("frost")||l.contains("crystal")||l.contains("cryo") { EnemyElement::Ice }
    else if l.contains("lightning")||l.contains("thunder")||l.contains("volt")||l.contains("shock") { EnemyElement::Lightning }
    else if l.contains("poison")||l.contains("toxic")||l.contains("venom")||l.contains("acid") { EnemyElement::Poison }
    else if l.contains("shadow")||l.contains("dark")||l.contains("void")||l.contains("abyss") { EnemyElement::Shadow }
    else if l.contains("holy")||l.contains("light")||l.contains("radiant")||l.contains("divine") { EnemyElement::Holy }
    else { EnemyElement::Neutral }
}

/// Unique boss visual identifiers matching proof-engine BossType.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossVisualProfile {
    Mirror, Null, Committee, FibonacciHydra, Eigenstate,
    Ouroboros, AlgorithmReborn, ChaosWeaver, VoidSerpent, PrimeFactorial,
}

impl BossVisualProfile {
    pub fn from_name(name: &str) -> Option<Self> {
        let l = name.to_lowercase();
        if l.contains("mirror") { Some(Self::Mirror) }
        else if l.contains("null") { Some(Self::Null) }
        else if l.contains("committee")||l.contains("judge") { Some(Self::Committee) }
        else if l.contains("fibonacci")||l.contains("hydra") { Some(Self::FibonacciHydra) }
        else if l.contains("eigen") { Some(Self::Eigenstate) }
        else if l.contains("ouroboros") { Some(Self::Ouroboros) }
        else if l.contains("algorithm")||l.contains("reborn") { Some(Self::AlgorithmReborn) }
        else if l.contains("chaos")||l.contains("weaver") { Some(Self::ChaosWeaver) }
        else if l.contains("void")||l.contains("serpent") { Some(Self::VoidSerpent) }
        else if l.contains("prime")||l.contains("factorial") { Some(Self::PrimeFactorial) }
        else { None }
    }
    pub fn glyph_count(&self) -> usize {
        match self { Self::Mirror=>18, Self::Null=>15, Self::Committee=>25,
            Self::FibonacciHydra=>21, Self::Eigenstate=>20, Self::Ouroboros=>24,
            Self::AlgorithmReborn=>30, Self::ChaosWeaver=>22, Self::VoidSerpent=>28,
            Self::PrimeFactorial=>20 }
    }
    pub fn formation(&self, phase: u32) -> FormationShape {
        match self {
            Self::Mirror=>FormationShape::Diamond, Self::Null=>FormationShape::Ring,
            Self::Committee=>FormationShape::Semicircle,
            Self::FibonacciHydra=> if phase==0 { FormationShape::Cluster } else { FormationShape::Swarm },
            Self::Eigenstate=> if phase%2==0 { FormationShape::Star } else { FormationShape::Diamond },
            Self::Ouroboros=>FormationShape::Ring,
            Self::AlgorithmReborn=> match phase { 0=>FormationShape::Grid, 1=>FormationShape::Diamond, 2=>FormationShape::Star, _=>FormationShape::Pentagram },
            Self::ChaosWeaver=> [FormationShape::Star,FormationShape::Spiral,FormationShape::Cross,FormationShape::Triangle,FormationShape::Pentagon][(phase as usize)%5],
            Self::VoidSerpent=>FormationShape::Snake, Self::PrimeFactorial=>FormationShape::Grid,
        }
    }
    pub fn formation_scale(&self) -> f32 {
        match self { Self::Mirror=>1.5, Self::Null=>1.3, Self::Committee=>2.5,
            Self::FibonacciHydra=>2.0, Self::Eigenstate=>1.6, Self::Ouroboros=>2.0,
            Self::AlgorithmReborn=>2.5, Self::ChaosWeaver=>1.8, Self::VoidSerpent=>3.0,
            Self::PrimeFactorial=>1.5 }
    }
}

/// Full enemy visual state tracked across frames.
#[derive(Clone)]
pub struct EnemyVisualState {
    pub name: String, pub tier: EnemyTier, pub element: EnemyElement,
    pub boss_profile: Option<BossVisualProfile>,
    pub hp_frac: f32, pub phase: u32, pub spawn_t: f32, pub death_t: f32,
    pub hit_reaction_t: f32, pub time: f32, pub is_alive: bool,
}

impl EnemyVisualState {
    pub fn new(name: &str, tier: u32, element: EnemyElement) -> Self {
        Self { name: name.to_string(), tier: EnemyTier::from_tier(tier), element,
            boss_profile: BossVisualProfile::from_name(name),
            hp_frac: 1.0, phase: 0, spawn_t: 0.0, death_t: 0.0,
            hit_reaction_t: 1.0, time: 0.0, is_alive: true }
    }
    pub fn from_name(name: &str, tier: u32) -> Self { Self::new(name, tier, element_from_name(name)) }
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;
        if self.spawn_t < 1.0 { self.spawn_t = (self.spawn_t + dt*2.0).min(1.0); }
        if !self.is_alive && self.death_t < 1.0 { self.death_t = (self.death_t + dt*0.8).min(1.0); }
        if self.hit_reaction_t < 1.0 { self.hit_reaction_t = (self.hit_reaction_t + dt*5.0).min(1.0); }
    }
    pub fn trigger_hit(&mut self) { self.hit_reaction_t = 0.0; }
    pub fn trigger_death(&mut self) { self.is_alive = false; self.death_t = 0.0; }
    pub fn set_phase(&mut self, phase: u32) { self.phase = phase; }
    pub fn is_death_complete(&self) -> bool { !self.is_alive && self.death_t >= 1.0 }
}

/// Render an enemy with full visual state.
pub fn render_enemy_full(engine: &mut ProofEngine, state: &EnemyVisualState, position: Vec3, _frame: u64) {
    let element = state.element;
    let primary = element.primary_color();
    let accent = element.accent_color();
    let glow = element.glow_color();
    let em = element.emission() + state.tier.em();

    let shape = if let Some(bp) = state.boss_profile { bp.formation(state.phase) }
                else { element.preferred_formation() };
    let count = if let Some(bp) = state.boss_profile { bp.glyph_count() } else { state.tier.count() };
    let scale = if let Some(bp) = state.boss_profile { bp.formation_scale() }
                else { match state.tier { EnemyTier::Minion=>0.6, EnemyTier::Elite=>1.0,
                    EnemyTier::Champion=>1.3, EnemyTier::Boss=>1.8, EnemyTier::Abomination=>2.2 } };

    let target = shape.generate_positions(count, scale);
    let mut positions = if state.spawn_t < 1.0 { formations::spawn_animation(&target, state.spawn_t) }
                        else { target };

    if state.is_alive { positions = elem_idle(&positions, element, state.time); }
    positions = formations::apply_hp_drift(&positions, state.hp_frac, state.time);
    if state.hit_reaction_t < 1.0 { positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.4); }
    if let Some(bp) = state.boss_profile { positions = boss_anim(&positions, bp, state.phase, state.time); }
    if !state.is_alive {
        let ds = element.death_style();
        positions = positions.iter().enumerate().map(|(i, p)| ds.modify_death_pos(*p, state.death_t, i)).collect();
    }

    let palette = element.glyph_palette();
    let name_chars: Vec<char> = state.name.chars().filter(|c| !c.is_whitespace()).take(4).collect();
    let ds = element.death_style();

    for (i, p) in positions.iter().enumerate() {
        let wp = position + *p;
        let ch = if i < name_chars.len() { name_chars[i] } else { palette[i % palette.len()] };
        let dist = p.length();
        let t = (dist / (scale*1.5)).clamp(0.0, 1.0);
        let mut color = lc(primary, accent, t);
        if state.spawn_t < 1.0 {
            let fl = (1.0 - state.spawn_t)*0.5;
            color.x = (color.x + fl).min(1.0);
            color.y = (color.y + fl).min(1.0);
            color.z = (color.z + fl).min(1.0);
        }
        if !state.is_alive { color = ds.death_color(color, state.death_t); }
        let gr = match state.tier { EnemyTier::Minion=>0.0, EnemyTier::Elite=>0.3,
            EnemyTier::Champion=>0.5, EnemyTier::Boss=>0.8, EnemyTier::Abomination=>1.0 };
        if gr > 0.01 { seg(engine, ch, wp, color, em, state.tier.gs(), glow, gr); }
        else { se(engine, ch, wp, color, em, state.tier.gs()); }
    }
}

fn elem_idle(positions: &[Vec3], element: EnemyElement, time: f32) -> Vec<Vec3> {
    match element {
        EnemyElement::Fire => positions.iter().enumerate().map(|(i, p)| {
            *p + Vec3::new((time*6.0+i as f32*2.1).sin()*0.05, ((time*TAU+i as f32).sin()*0.04).abs(), 0.0)
        }).collect(),
        EnemyElement::Ice => formations::apply_breathing(positions, time, 0.4, 0.03),
        EnemyElement::Lightning => positions.iter().enumerate().map(|(i, p)| {
            *p + Vec3::new((time*15.0+i as f32*3.7).sin()*0.03, (time*12.0+i as f32*5.1).cos()*0.03, 0.0)
        }).collect(),
        EnemyElement::Poison => positions.iter().enumerate().map(|(i, p)| {
            *p + Vec3::new(0.0, ((time*3.0+i as f32*1.3).sin()*0.04).max(0.0), 0.0)
        }).collect(),
        EnemyElement::Shadow => positions.iter().enumerate().map(|(i, p)| {
            *p + Vec3::new((time*1.5+p.y*2.0+i as f32*0.5).sin()*0.06, 0.0, 0.0)
        }).collect(),
        EnemyElement::Holy => positions.iter().map(|p| {
            let w = (time*TAU - p.length()*3.0).sin()*0.04;
            *p * (1.0 + w)
        }).collect(),
        EnemyElement::Neutral => formations::apply_breathing(positions, time, 0.6, 0.03),
    }
}

fn boss_anim(positions: &[Vec3], profile: BossVisualProfile, phase: u32, time: f32) -> Vec<Vec3> {
    match profile {
        BossVisualProfile::Mirror => positions.iter().enumerate().map(|(i, p)| {
            Vec3::new(p.x + (time*3.0+i as f32*0.8).sin()*0.02, p.y, p.z)
        }).collect(),
        BossVisualProfile::Null => {
            let pulse = (time*0.8).sin()*0.15;
            positions.iter().map(|p| *p * (1.0+pulse)).collect()
        }
        BossVisualProfile::Committee => {
            let cnt = positions.len();
            positions.iter().enumerate().map(|(i, p)| {
                let j = (i*5)/cnt.max(1);
                *p + Vec3::new(0.0, (time*1.5+j as f32*1.2).sin()*0.08, 0.0)
            }).collect()
        }
        BossVisualProfile::FibonacciHydra => {
            let heads = (phase+1).min(5) as usize;
            let hc = (positions.len()/heads).max(1);
            positions.iter().enumerate().map(|(i, p)| {
                let h = i/hc;
                *p + Vec3::new((time*2.0+h as f32*PI*0.4).sin()*0.1, 0.0, 0.0)
            }).collect()
        }
        BossVisualProfile::Eigenstate => positions.iter().enumerate().map(|(i, p)| {
            let off = if i%2==0 { 1.0 } else { -1.0 };
            *p + Vec3::new((time*1.0).sin()*0.15*off, 0.0, 0.0)
        }).collect(),
        BossVisualProfile::Ouroboros => formations::apply_rotation(positions, time, 0.3),
        BossVisualProfile::AlgorithmReborn => {
            let int = 0.03 + phase as f32 * 0.02;
            let pulse = (time*(1.0+phase as f32*0.3)).sin()*int;
            positions.iter().map(|p| *p * (1.0+pulse)).collect()
        }
        BossVisualProfile::ChaosWeaver => positions.iter().enumerate().map(|(i, p)| {
            let sd = (i as u32).wrapping_mul(2654435761);
            *p + Vec3::new((time*8.0+sd as f32*0.001).sin()*0.08, (time*7.0+sd as f32*0.0013).cos()*0.08, 0.0)
        }).collect(),
        BossVisualProfile::VoidSerpent => positions.iter().enumerate().map(|(i, p)| {
            let t = i as f32 / positions.len() as f32;
            *p + Vec3::new(0.0, (time*2.0+t*TAU*1.5).sin()*0.15, 0.0)
        }).collect(),
        BossVisualProfile::PrimeFactorial => positions.iter().enumerate().map(|(i, p)| {
            *p * (1.0 + (time*3.0+i as f32*0.5).sin()*0.03)
        }).collect(),
    }
}

/// Build an AmorphousEntity for an enemy (backwards-compatible).
pub fn build_enemy_entity(name: &str, tier: u32, position: Vec3) -> AmorphousEntity {
    let et = EnemyTier::from_tier(tier);
    let element = element_from_name(name);
    let bp = BossVisualProfile::from_name(name);
    let (count, scale) = if let Some(b) = bp { (b.glyph_count(), b.formation_scale()) }
        else { (et.count(), match et { EnemyTier::Minion=>0.6, EnemyTier::Elite=>1.0,
            EnemyTier::Champion=>1.3, EnemyTier::Boss=>1.8, EnemyTier::Abomination=>2.2 }) };
    let shape = if let Some(b) = bp { b.formation(0) } else { element.preferred_formation() };
    let positions = shape.generate_positions(count, scale);
    let palette = element.glyph_palette();
    let primary = element.primary_color();
    let accent = element.accent_color();
    let nc: Vec<char> = name.chars().filter(|c| !c.is_whitespace()).take(4).collect();

    let mut chars = Vec::with_capacity(count);
    let mut colors = Vec::with_capacity(count);
    for (i, p) in positions.iter().enumerate() {
        chars.push(if i < nc.len() { nc[i] } else { palette[i % palette.len()] });
        let t = (p.length() / (scale*1.5)).clamp(0.0, 1.0);
        colors.push(lc(primary, accent, t));
    }

    let mut entity = AmorphousEntity::new(format!("enemy_{}", name), position);
    entity.entity_mass = 30.0 + tier as f32 * 10.0;
    entity.pulse_rate = 0.8;
    entity.pulse_depth = 0.04;
    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity
}

/// Update an existing AmorphousEntity from an EnemyVisualState.
pub fn update_enemy_entity(entity: &mut AmorphousEntity, state: &EnemyVisualState) {
    let shape = if let Some(bp) = state.boss_profile { bp.formation(state.phase) }
                else { state.element.preferred_formation() };
    let count = if let Some(bp) = state.boss_profile { bp.glyph_count() } else { state.tier.count() };
    let scale = if let Some(bp) = state.boss_profile { bp.formation_scale() }
                else { match state.tier { EnemyTier::Minion=>0.6, EnemyTier::Elite=>1.0,
                    EnemyTier::Champion=>1.3, EnemyTier::Boss=>1.8, EnemyTier::Abomination=>2.2 } };

    let target = shape.generate_positions(count, scale);
    let mut positions = if state.spawn_t < 1.0 { formations::spawn_animation(&target, state.spawn_t) }
                        else { target };
    if state.is_alive { positions = elem_idle(&positions, state.element, state.time); }
    positions = formations::apply_hp_drift(&positions, state.hp_frac, state.time);
    if state.hit_reaction_t < 1.0 { positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.4); }
    if let Some(bp) = state.boss_profile { positions = boss_anim(&positions, bp, state.phase, state.time); }
    if !state.is_alive {
        let ds = state.element.death_style();
        positions = positions.iter().enumerate().map(|(i, p)| ds.modify_death_pos(*p, state.death_t, i)).collect();
    }

    let palette = state.element.glyph_palette();
    let primary = state.element.primary_color();
    let accent = state.element.accent_color();
    let nc: Vec<char> = state.name.chars().filter(|c| !c.is_whitespace()).take(4).collect();
    let ds = state.element.death_style();

    let len = positions.len();
    let mut chars = Vec::with_capacity(len);
    let mut colors = Vec::with_capacity(len);
    for (i, p) in positions.iter().enumerate() {
        chars.push(if i < nc.len() { nc[i] } else { palette[i % palette.len()] });
        let t = (p.length() / (scale*1.5)).clamp(0.0, 1.0);
        let mut color = lc(primary, accent, t);
        if state.spawn_t < 1.0 {
            let fl = (1.0-state.spawn_t)*0.5;
            color.x = (color.x+fl).min(1.0); color.y = (color.y+fl).min(1.0); color.z = (color.z+fl).min(1.0);
        }
        if !state.is_alive { color = ds.death_color(color, state.death_t); }
        colors.push(color);
    }
    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity.hp = state.hp_frac * entity.max_hp;
    entity.update_cohesion();
}

/// Classify an enemy by tier, element, and optional boss profile.
pub fn classify_enemy(name: &str, tier: u32) -> (EnemyTier, EnemyElement, Option<BossVisualProfile>) {
    (EnemyTier::from_tier(tier), element_from_name(name), BossVisualProfile::from_name(name))
}

fn lc(a: Vec4, b: Vec4, t: f32) -> Vec4 { a + (b - a) * t.clamp(0.0, 1.0) }
