//! Enemy entity rendering using proof-engine AmorphousEntity.
//!
//! Five enemy tiers with increasing visual complexity, six elemental themes,
//! ten unique boss visual profiles, spawn/death animations, and element-specific
//! dissolution effects.

use proof_engine::prelude::*;
use glam::{Vec3, Vec4};
use std::f32::consts::{PI, TAU};

use super::formations::{
    self, FormationShape, ElementalDeathStyle,
};

// ── Enemy element ────────────────────────────────────────────────────────────

/// Element type that drives enemy visual theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyElement {
    Fire,
    Ice,
    Lightning,
    Poison,
    Shadow,
    Holy,
    Neutral,
}

impl EnemyElement {
    /// Primary color for this element.
    pub fn primary_color(&self) -> Vec4 {
        match self {
            EnemyElement::Fire => Vec4::new(1.0, 0.45, 0.1, 1.0),
            EnemyElement::Ice => Vec4::new(0.5, 0.75, 1.0, 1.0),
            EnemyElement::Lightning => Vec4::new(1.0, 1.0, 0.3, 1.0),
            EnemyElement::Poison => Vec4::new(0.3, 0.8, 0.2, 1.0),
            EnemyElement::Shadow => Vec4::new(0.3, 0.15, 0.45, 1.0),
            EnemyElement::Holy => Vec4::new(1.0, 0.95, 0.7, 1.0),
            EnemyElement::Neutral => Vec4::new(0.7, 0.25, 0.2, 1.0),
        }
    }

    /// Accent color for this element.
    pub fn accent_color(&self) -> Vec4 {
        match self {
            EnemyElement::Fire => Vec4::new(1.0, 0.7, 0.0, 1.0),
            EnemyElement::Ice => Vec4::new(0.8, 0.9, 1.0, 1.0),
            EnemyElement::Lightning => Vec4::new(1.0, 1.0, 0.8, 1.0),
            EnemyElement::Poison => Vec4::new(0.5, 0.2, 0.7, 1.0),
            EnemyElement::Shadow => Vec4::new(0.15, 0.05, 0.25, 1.0),
            EnemyElement::Holy => Vec4::new(1.0, 1.0, 1.0, 1.0),
            EnemyElement::Neutral => Vec4::new(0.9, 0.4, 0.3, 1.0),
        }
    }

    /// Character palette for this element.
    pub fn glyph_palette(&self) -> &'static [char] {
        match self {
            EnemyElement::Fire => &['^', '*', '~', '#', '!', 'v', '>', '<'],
            EnemyElement::Ice => &['*', '+', '.', ':', '#', '=', '-', 'o'],
            EnemyElement::Lightning => &['!', '/', '\\', 'X', '+', '#', '|', '-'],
            EnemyElement::Poison => &['~', '.', ':', ';', '?', '%', '&', 'S'],
            EnemyElement::Shadow => &['.', ' ', ':', '`', '\'', ',', '-', '~'],
            EnemyElement::Holy => &['*', '+', '.', '\'', ':', '!', '#', '^'],
            EnemyElement::Neutral => &['#', 'X', '+', '-', '|', '/', '\\', '.'],
        }
    }

    /// Death style for this element.
    pub fn death_style(&self) -> ElementalDeathStyle {
        match self {
            EnemyElement::Fire => ElementalDeathStyle::Fire,
            EnemyElement::Ice => ElementalDeathStyle::Ice,
            EnemyElement::Lightning => ElementalDeathStyle::Lightning,
            EnemyElement::Poison => ElementalDeathStyle::Poison,
            EnemyElement::Shadow => ElementalDeathStyle::Shadow,
            EnemyElement::Holy => ElementalDeathStyle::Holy,
            EnemyElement::Neutral => ElementalDeathStyle::Default,
        }
    }

    /// Formation shape preference for this element.
    pub fn preferred_formation(&self) -> FormationShape {
        match self {
            EnemyElement::Fire => FormationShape::Triangle,
            EnemyElement::Ice => FormationShape::Diamond,
            EnemyElement::Lightning => FormationShape::Star,
            EnemyElement::Poison => FormationShape::Spiral,
            EnemyElement::Shadow => FormationShape::Crescent,
            EnemyElement::Holy => FormationShape::Ring,
            EnemyElement::Neutral => FormationShape::Cluster,
        }
    }

    /// Emission intensity for this element.
    pub fn emission(&self) -> f32 {
        match self {
            EnemyElement::Fire => 0.5,
            EnemyElement::Ice => 0.3,
            EnemyElement::Lightning => 0.6,
            EnemyElement::Poison => 0.2,
            EnemyElement::Shadow => 0.1,
            EnemyElement::Holy => 0.5,
            EnemyElement::Neutral => 0.15,
        }
    }
}

// ── Enemy tier ───────────────────────────────────────────────────────────────

/// Visual tier determining enemy complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnemyVisualTier {
    /// T1: Common — 3-5 glyphs, simple shapes, single color
    Common,
    /// T2: Uncommon — 5-8 glyphs, element-colored accents, idle animation
    Uncommon,
    /// T3: Rare — 8-12 glyphs, full formation, particles, glow
    Rare,
    /// T4: Elite — 12-18 glyphs, complex formation, aura, status particles
    Elite,
    /// T5: Boss — 15-30 glyphs, unique formation per boss, phase-dependent visuals
    Boss,
}

impl EnemyVisualTier {
    /// Glyph count range for this tier.
    pub fn glyph_range(&self) -> (usize, usize) {
        match self {
            EnemyVisualTier::Common => (3, 5),
            EnemyVisualTier::Uncommon => (5, 8),
            EnemyVisualTier::Rare => (8, 12),
            EnemyVisualTier::Elite => (12, 18),
            EnemyVisualTier::Boss => (15, 30),
        }
    }

    /// Default glyph count (midpoint of range).
    pub fn default_glyph_count(&self) -> usize {
        let (lo, hi) = self.glyph_range();
        (lo + hi) / 2
    }

    /// Formation scale for this tier.
    pub fn formation_scale(&self) -> f32 {
        match self {
            EnemyVisualTier::Common => 0.6,
            EnemyVisualTier::Uncommon => 0.8,
            EnemyVisualTier::Rare => 1.0,
            EnemyVisualTier::Elite => 1.3,
            EnemyVisualTier::Boss => 1.8,
        }
    }

    /// Entity mass for this tier.
    pub fn entity_mass(&self) -> f32 {
        match self {
            EnemyVisualTier::Common => 20.0,
            EnemyVisualTier::Uncommon => 30.0,
            EnemyVisualTier::Rare => 50.0,
            EnemyVisualTier::Elite => 80.0,
            EnemyVisualTier::Boss => 150.0,
        }
    }

    /// Whether this tier has idle animation.
    pub fn has_idle_anim(&self) -> bool {
        !matches!(self, EnemyVisualTier::Common)
    }

    /// Whether this tier has particle effects.
    pub fn has_particles(&self) -> bool {
        matches!(self, EnemyVisualTier::Rare | EnemyVisualTier::Elite | EnemyVisualTier::Boss)
    }

    /// Whether this tier has a glow aura.
    pub fn has_glow(&self) -> bool {
        matches!(self, EnemyVisualTier::Rare | EnemyVisualTier::Elite | EnemyVisualTier::Boss)
    }

    /// Whether this tier has a status aura.
    pub fn has_aura(&self) -> bool {
        matches!(self, EnemyVisualTier::Elite | EnemyVisualTier::Boss)
    }

    /// Pulse rate for idle animation.
    pub fn pulse_rate(&self) -> f32 {
        match self {
            EnemyVisualTier::Common => 0.5,
            EnemyVisualTier::Uncommon => 0.8,
            EnemyVisualTier::Rare => 1.0,
            EnemyVisualTier::Elite => 1.2,
            EnemyVisualTier::Boss => 0.6,
        }
    }

    /// Map a numeric tier (0-based) to a visual tier.
    pub fn from_numeric(tier: u32) -> Self {
        match tier {
            0 => EnemyVisualTier::Common,
            1 => EnemyVisualTier::Uncommon,
            2 => EnemyVisualTier::Rare,
            3 => EnemyVisualTier::Elite,
            _ => EnemyVisualTier::Boss,
        }
    }
}

// ── Boss visual profiles ─────────────────────────────────────────────────────

/// Unique boss visual profile identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossVisualProfile {
    Mirror,
    Null,
    Committee,
    FibonacciHydra,
    Eigenstate,
    Ouroboros,
    AlgorithmReborn,
    ChaosWeaver,
    VoidSerpent,
    PrimeFactorial,
}

impl BossVisualProfile {
    /// Map a boss name string to a profile.
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        if lower.contains("mirror") {
            Some(BossVisualProfile::Mirror)
        } else if lower.contains("null") {
            Some(BossVisualProfile::Null)
        } else if lower.contains("committee") || lower.contains("judge") {
            Some(BossVisualProfile::Committee)
        } else if lower.contains("fibonacci") || lower.contains("hydra") {
            Some(BossVisualProfile::FibonacciHydra)
        } else if lower.contains("eigen") {
            Some(BossVisualProfile::Eigenstate)
        } else if lower.contains("ouroboros") {
            Some(BossVisualProfile::Ouroboros)
        } else if lower.contains("algorithm") || lower.contains("reborn") {
            Some(BossVisualProfile::AlgorithmReborn)
        } else if lower.contains("chaos") || lower.contains("weaver") {
            Some(BossVisualProfile::ChaosWeaver)
        } else if lower.contains("void") || lower.contains("serpent") {
            Some(BossVisualProfile::VoidSerpent)
        } else if lower.contains("prime") || lower.contains("factorial") {
            Some(BossVisualProfile::PrimeFactorial)
        } else {
            None
        }
    }

    /// Glyph count for this boss.
    pub fn glyph_count(&self) -> usize {
        match self {
            BossVisualProfile::Mirror => 18,
            BossVisualProfile::Null => 15,
            BossVisualProfile::Committee => 25,
            BossVisualProfile::FibonacciHydra => 21,
            BossVisualProfile::Eigenstate => 20,
            BossVisualProfile::Ouroboros => 24,
            BossVisualProfile::AlgorithmReborn => 30,
            BossVisualProfile::ChaosWeaver => 22,
            BossVisualProfile::VoidSerpent => 28,
            BossVisualProfile::PrimeFactorial => 20,
        }
    }

    /// Formation scale for this boss.
    pub fn formation_scale(&self) -> f32 {
        match self {
            BossVisualProfile::Mirror => 1.5,
            BossVisualProfile::Null => 1.3,
            BossVisualProfile::Committee => 2.5,
            BossVisualProfile::FibonacciHydra => 2.0,
            BossVisualProfile::Eigenstate => 1.6,
            BossVisualProfile::Ouroboros => 2.0,
            BossVisualProfile::AlgorithmReborn => 2.5,
            BossVisualProfile::ChaosWeaver => 1.8,
            BossVisualProfile::VoidSerpent => 3.0,
            BossVisualProfile::PrimeFactorial => 1.5,
        }
    }
}

// ── Enemy visual state ───────────────────────────────────────────────────────

/// Full enemy visual state tracked across frames.
#[derive(Clone)]
pub struct EnemyVisualState {
    pub name: String,
    pub tier: EnemyVisualTier,
    pub element: EnemyElement,
    pub boss_profile: Option<BossVisualProfile>,
    pub hp_frac: f32,
    pub phase: u32,
    pub spawn_t: f32,
    pub death_t: f32,
    pub hit_reaction_t: f32,
    pub time: f32,
    pub is_alive: bool,
}

impl EnemyVisualState {
    pub fn new(name: &str, tier: EnemyVisualTier, element: EnemyElement) -> Self {
        let boss_profile = if tier == EnemyVisualTier::Boss {
            BossVisualProfile::from_name(name)
        } else {
            None
        };
        Self {
            name: name.to_string(),
            tier,
            element,
            boss_profile,
            hp_frac: 1.0,
            phase: 0,
            spawn_t: 0.0,
            death_t: 0.0,
            hit_reaction_t: 1.0,
            time: 0.0,
            is_alive: true,
        }
    }

    /// Advance visual state by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;
        if self.spawn_t < 1.0 {
            self.spawn_t = (self.spawn_t + dt * 2.0).min(1.0);
        }
        if !self.is_alive && self.death_t < 1.0 {
            self.death_t = (self.death_t + dt * 0.8).min(1.0);
        }
        if self.hit_reaction_t < 1.0 {
            self.hit_reaction_t = (self.hit_reaction_t + dt * 5.0).min(1.0);
        }
    }

    /// Trigger hit reaction.
    pub fn trigger_hit(&mut self) {
        self.hit_reaction_t = 0.0;
    }

    /// Trigger death.
    pub fn trigger_death(&mut self) {
        self.is_alive = false;
        self.death_t = 0.0;
    }

    /// Set boss phase.
    pub fn set_phase(&mut self, phase: u32) {
        self.phase = phase;
    }

    /// Is the death animation complete?
    pub fn is_death_complete(&self) -> bool {
        !self.is_alive && self.death_t >= 1.0
    }
}

// ── Core entity builder ──────────────────────────────────────────────────────

/// Build an AmorphousEntity for an enemy with glyph count scaled by tier.
pub fn build_enemy_entity(name: &str, tier: u32, position: Vec3) -> AmorphousEntity {
    let visual_tier = EnemyVisualTier::from_numeric(tier);
    let element = element_from_name(name);
    build_enemy_entity_full(name, visual_tier, element, position)
}

/// Build an enemy entity with full visual configuration.
pub fn build_enemy_entity_full(
    name: &str,
    tier: EnemyVisualTier,
    element: EnemyElement,
    position: Vec3,
) -> AmorphousEntity {
    let boss_profile = if tier == EnemyVisualTier::Boss {
        BossVisualProfile::from_name(name)
    } else {
        None
    };

    let (glyph_count, scale) = if let Some(bp) = boss_profile {
        (bp.glyph_count(), bp.formation_scale())
    } else {
        (tier.default_glyph_count(), tier.formation_scale())
    };

    let formation_shape = if let Some(bp) = boss_profile {
        boss_formation_shape(bp, 0)
    } else {
        element_formation_shape(element, tier)
    };

    let positions = formation_shape.generate_positions(glyph_count, scale);

    let (chars, colors) = if let Some(bp) = boss_profile {
        generate_boss_glyphs(bp, &positions, element, 0)
    } else {
        generate_tier_glyphs(tier, element, &positions)
    };

    let mut entity = AmorphousEntity::new(format!("enemy_{}", name), position);
    entity.entity_mass = tier.entity_mass();
    entity.pulse_rate = tier.pulse_rate();
    entity.pulse_depth = if tier.has_idle_anim() { 0.04 } else { 0.01 };
    entity.formation = positions;
    entity.formation_chars = chars;
    entity.formation_colors = colors;
    entity
}

/// Build an enemy entity from a full EnemyVisualState.
pub fn build_enemy_entity_from_state(
    state: &EnemyVisualState,
    position: Vec3,
) -> AmorphousEntity {
    let (glyph_count, scale) = if let Some(bp) = state.boss_profile {
        (bp.glyph_count(), bp.formation_scale())
    } else {
        (state.tier.default_glyph_count(), state.tier.formation_scale())
    };

    let formation_shape = if let Some(bp) = state.boss_profile {
        boss_formation_shape(bp, state.phase)
    } else {
        element_formation_shape(state.element, state.tier)
    };

    let mut positions = formation_shape.generate_positions(glyph_count, scale);

    // Spawn animation
    if state.spawn_t < 1.0 {
        positions = formations::spawn_animation(&positions, state.spawn_t);
    }

    // Idle animation for tiers that support it
    if state.tier.has_idle_anim() && state.is_alive {
        positions = apply_enemy_idle(&positions, state.tier, state.element, state.time);
    }

    // HP-based drift
    positions = formations::apply_hp_drift(&positions, state.hp_frac, state.time);

    // Hit reaction
    if state.hit_reaction_t < 1.0 {
        positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.4);
    }

    // Boss-specific animations
    if let Some(bp) = state.boss_profile {
        positions = apply_boss_animation(&positions, bp, state.phase, state.time);
    }

    // Death animation
    if !state.is_alive {
        let death_style = state.element.death_style();
        positions = positions
            .iter()
            .enumerate()
            .map(|(i, p)| death_style.modify_death_pos(*p, state.death_t, i))
            .collect();
    }

    // Generate glyphs
    let (chars, mut colors) = if let Some(bp) = state.boss_profile {
        generate_boss_glyphs(bp, &positions, state.element, state.phase)
    } else {
        generate_tier_glyphs(state.tier, state.element, &positions)
    };

    // Spawn color flash
    if state.spawn_t < 1.0 {
        let flash = (1.0 - state.spawn_t) * 0.5;
        let element_color = state.element.primary_color();
        for c in &mut colors {
            c.x = (c.x + element_color.x * flash).min(1.0);
            c.y = (c.y + element_color.y * flash).min(1.0);
            c.z = (c.z + element_color.z * flash).min(1.0);
        }
    }

    // Death color
    if !state.is_alive {
        let death_style = state.element.death_style();
        colors = colors
            .iter()
            .map(|c| death_style.death_color(*c, state.death_t))
            .collect();
    }

    // Aura glow for elite/boss
    if state.tier.has_glow() && state.is_alive {
        let glow = (state.time * 2.0).sin() * 0.1 + 0.1;
        for c in &mut colors {
            c.x = (c.x + glow).min(1.0);
            c.y = (c.y + glow * 0.5).min(1.0);
        }
    }

    // Ensure equal lengths
    let len = positions.len();
    let mut final_chars = chars;
    while final_chars.len() < len {
        final_chars.push('.');
    }
    while colors.len() < len {
        colors.push(state.element.primary_color());
    }
    final_chars.truncate(len);
    colors.truncate(len);

    let mut entity = AmorphousEntity::new(format!("enemy_{}", state.name), position);
    entity.entity_mass = state.tier.entity_mass();
    entity.pulse_rate = state.tier.pulse_rate();
    entity.pulse_depth = if state.tier.has_idle_anim() { 0.04 } else { 0.01 };
    entity.hp = state.hp_frac * 100.0;
    entity.max_hp = 100.0;
    entity.formation = positions;
    entity.formation_chars = final_chars;
    entity.formation_colors = colors;
    entity.update_cohesion();
    entity
}

/// Update an existing enemy entity from visual state.
pub fn update_enemy_entity(entity: &mut AmorphousEntity, state: &EnemyVisualState) {
    let (glyph_count, scale) = if let Some(bp) = state.boss_profile {
        (bp.glyph_count(), bp.formation_scale())
    } else {
        (state.tier.default_glyph_count(), state.tier.formation_scale())
    };

    let formation_shape = if let Some(bp) = state.boss_profile {
        boss_formation_shape(bp, state.phase)
    } else {
        element_formation_shape(state.element, state.tier)
    };

    let mut positions = formation_shape.generate_positions(glyph_count, scale);

    if state.spawn_t < 1.0 {
        positions = formations::spawn_animation(&positions, state.spawn_t);
    }

    if state.tier.has_idle_anim() && state.is_alive {
        positions = apply_enemy_idle(&positions, state.tier, state.element, state.time);
    }

    positions = formations::apply_hp_drift(&positions, state.hp_frac, state.time);

    if state.hit_reaction_t < 1.0 {
        positions = formations::apply_hit_reaction(&positions, state.hit_reaction_t, 0.4);
    }

    if let Some(bp) = state.boss_profile {
        positions = apply_boss_animation(&positions, bp, state.phase, state.time);
    }

    if !state.is_alive {
        let death_style = state.element.death_style();
        positions = positions
            .iter()
            .enumerate()
            .map(|(i, p)| death_style.modify_death_pos(*p, state.death_t, i))
            .collect();
    }

    let (chars, mut colors) = if let Some(bp) = state.boss_profile {
        generate_boss_glyphs(bp, &positions, state.element, state.phase)
    } else {
        generate_tier_glyphs(state.tier, state.element, &positions)
    };

    if state.spawn_t < 1.0 {
        let flash = (1.0 - state.spawn_t) * 0.5;
        let ec = state.element.primary_color();
        for c in &mut colors {
            c.x = (c.x + ec.x * flash).min(1.0);
            c.y = (c.y + ec.y * flash).min(1.0);
            c.z = (c.z + ec.z * flash).min(1.0);
        }
    }

    if !state.is_alive {
        let death_style = state.element.death_style();
        colors = colors
            .iter()
            .map(|c| death_style.death_color(*c, state.death_t))
            .collect();
    }

    if state.tier.has_glow() && state.is_alive {
        let glow = (state.time * 2.0).sin() * 0.1 + 0.1;
        for c in &mut colors {
            c.x = (c.x + glow).min(1.0);
            c.y = (c.y + glow * 0.5).min(1.0);
        }
    }

    let len = positions.len();
    let mut final_chars = chars;
    while final_chars.len() < len {
        final_chars.push('.');
    }
    while colors.len() < len {
        colors.push(state.element.primary_color());
    }
    final_chars.truncate(len);
    colors.truncate(len);

    entity.formation = positions;
    entity.formation_chars = final_chars;
    entity.formation_colors = colors;
    entity.hp = state.hp_frac * entity.max_hp;
    entity.update_cohesion();
}

// ── Tier-based glyph generation ──────────────────────────────────────────────

fn generate_tier_glyphs(
    tier: EnemyVisualTier,
    element: EnemyElement,
    positions: &[Vec3],
) -> (Vec<char>, Vec<Vec4>) {
    let count = positions.len();
    let palette = element.glyph_palette();
    let primary = element.primary_color();
    let accent = element.accent_color();

    let mut chars = Vec::with_capacity(count);
    let mut colors = Vec::with_capacity(count);

    match tier {
        EnemyVisualTier::Common => {
            // Simple: single color, basic chars
            for i in 0..count {
                chars.push(palette[i % palette.len()]);
                colors.push(primary);
            }
        }
        EnemyVisualTier::Uncommon => {
            // Element-colored accents on outer glyphs
            for i in 0..count {
                let dist = positions.get(i).map(|p| p.length()).unwrap_or(0.0);
                chars.push(palette[i % palette.len()]);
                if dist > 0.5 {
                    colors.push(accent);
                } else {
                    colors.push(primary);
                }
            }
        }
        EnemyVisualTier::Rare => {
            // Full formation with glow on edge glyphs
            for i in 0..count {
                let dist = positions.get(i).map(|p| p.length()).unwrap_or(0.0);
                chars.push(palette[i % palette.len()]);
                let t = (dist / 1.5).clamp(0.0, 1.0);
                colors.push(lerp_color(primary, accent, t));
            }
        }
        EnemyVisualTier::Elite => {
            // Complex: layered coloring with aura fringe
            for i in 0..count {
                let dist = positions.get(i).map(|p| p.length()).unwrap_or(0.0);
                if i < 3 {
                    // Core glyphs: special chars
                    chars.push(core_glyph_for_element(element));
                    colors.push(accent);
                } else if dist > 1.0 {
                    // Aura fringe
                    chars.push(aura_glyph_for_element(element));
                    let alpha = (1.5 - dist).clamp(0.3, 0.8);
                    colors.push(Vec4::new(accent.x, accent.y, accent.z, alpha));
                } else {
                    chars.push(palette[i % palette.len()]);
                    let t = (dist / 1.2).clamp(0.0, 1.0);
                    colors.push(lerp_color(primary, accent, t * 0.5));
                }
            }
        }
        EnemyVisualTier::Boss => {
            // Boss tier without specific profile: elaborate generic boss
            for i in 0..count {
                let dist = positions.get(i).map(|p| p.length()).unwrap_or(0.0);
                if i == 0 {
                    chars.push('@');
                    colors.push(Vec4::new(1.0, 1.0, 1.0, 1.0));
                } else if i < 5 {
                    chars.push(core_glyph_for_element(element));
                    colors.push(accent);
                } else if dist > 1.5 {
                    chars.push(aura_glyph_for_element(element));
                    let alpha = (2.0 - dist).clamp(0.2, 0.7);
                    colors.push(Vec4::new(accent.x * 0.8, accent.y * 0.8, accent.z * 0.8, alpha));
                } else {
                    chars.push(palette[i % palette.len()]);
                    let t = (dist / 1.8).clamp(0.0, 1.0);
                    colors.push(lerp_color(primary, accent, t));
                }
            }
        }
    }

    (chars, colors)
}

fn core_glyph_for_element(element: EnemyElement) -> char {
    match element {
        EnemyElement::Fire => '#',
        EnemyElement::Ice => '*',
        EnemyElement::Lightning => '!',
        EnemyElement::Poison => '%',
        EnemyElement::Shadow => '@',
        EnemyElement::Holy => '*',
        EnemyElement::Neutral => '#',
    }
}

fn aura_glyph_for_element(element: EnemyElement) -> char {
    match element {
        EnemyElement::Fire => '~',
        EnemyElement::Ice => '.',
        EnemyElement::Lightning => '-',
        EnemyElement::Poison => '~',
        EnemyElement::Shadow => ' ',
        EnemyElement::Holy => '\'',
        EnemyElement::Neutral => '.',
    }
}

// ── Boss-specific glyph generation ───────────────────────────────────────────

fn generate_boss_glyphs(
    profile: BossVisualProfile,
    positions: &[Vec3],
    element: EnemyElement,
    phase: u32,
) -> (Vec<char>, Vec<Vec4>) {
    let count = positions.len();
    let mut chars = Vec::with_capacity(count);
    let mut colors = Vec::with_capacity(count);

    match profile {
        BossVisualProfile::Mirror => {
            // Copies player formation in inverted colors — uses mirror-like palette
            let mirror_chars = ['/', '\\', '|', '-', '+', 'X', '=', '#', '*', '.'];
            let base = Vec4::new(0.6, 0.6, 0.7, 1.0);
            let highlight = Vec4::new(0.9, 0.9, 1.0, 1.0);
            for i in 0..count {
                chars.push(mirror_chars[i % mirror_chars.len()]);
                let t = (i as f32 / count as f32 * PI).sin();
                colors.push(lerp_color(base, highlight, t));
            }
        }
        BossVisualProfile::Null => {
            // Void entity — erases/blanks around it
            let null_chars = [' ', '.', ' ', ':', ' ', '`', ' ', ','];
            let void_color = Vec4::new(0.1, 0.05, 0.15, 0.7);
            let edge_color = Vec4::new(0.3, 0.1, 0.4, 0.4);
            for i in 0..count {
                let dist = positions.get(i).map(|p| p.length()).unwrap_or(0.0);
                chars.push(null_chars[i % null_chars.len()]);
                if dist > 1.0 {
                    colors.push(edge_color);
                } else {
                    colors.push(void_color);
                }
            }
        }
        BossVisualProfile::Committee => {
            // 5 judge sub-entities in semicircle — distinct cluster per judge
            let judge_chars = ['J', 'U', 'D', 'G', 'E'];
            let judge_colors = [
                Vec4::new(0.8, 0.2, 0.2, 1.0),
                Vec4::new(0.2, 0.7, 0.2, 1.0),
                Vec4::new(0.2, 0.2, 0.8, 1.0),
                Vec4::new(0.8, 0.8, 0.2, 1.0),
                Vec4::new(0.7, 0.3, 0.7, 1.0),
            ];
            for i in 0..count {
                let judge_idx = (i * 5) / count.max(1);
                let judge_idx = judge_idx.min(4);
                if i % 5 == 0 {
                    chars.push(judge_chars[judge_idx]);
                } else {
                    chars.push(['#', '|', '-', '+'][i % 4]);
                }
                colors.push(judge_colors[judge_idx]);
            }
        }
        BossVisualProfile::FibonacciHydra => {
            // Fibonacci pattern — splits into smaller copies when hit
            let head_count = (count / (phase + 1) as usize).max(3);
            let heads = (phase + 1).min(5) as usize;
            let fib_chars = ['H', '<', '>', 'v', '^', '~', '.', '#'];
            let head_color = Vec4::new(0.4, 0.8, 0.3, 1.0);
            let body_color = Vec4::new(0.3, 0.6, 0.2, 1.0);
            for i in 0..count {
                let head_idx = i / head_count.max(1);
                if head_idx < heads && i % head_count == 0 {
                    chars.push('H');
                    colors.push(head_color);
                } else {
                    chars.push(fib_chars[i % fib_chars.len()]);
                    colors.push(body_color);
                }
            }
        }
        BossVisualProfile::Eigenstate => {
            // Two overlapping translucent formations that swap
            let eigen_a = Vec4::new(0.3, 0.5, 0.9, 0.6);
            let eigen_b = Vec4::new(0.9, 0.5, 0.3, 0.6);
            let chars_a = ['|', '+', '-', '.', ':'];
            let chars_b = ['/', '\\', 'X', '*', '#'];
            for i in 0..count {
                if phase % 2 == 0 {
                    if i % 2 == 0 {
                        chars.push(chars_a[i % chars_a.len()]);
                        colors.push(eigen_a);
                    } else {
                        chars.push(chars_b[i % chars_b.len()]);
                        colors.push(eigen_b);
                    }
                } else {
                    if i % 2 == 0 {
                        chars.push(chars_b[i % chars_b.len()]);
                        colors.push(eigen_b);
                    } else {
                        chars.push(chars_a[i % chars_a.len()]);
                        colors.push(eigen_a);
                    }
                }
            }
        }
        BossVisualProfile::Ouroboros => {
            // Circular snake eating its tail — glyphs rotate
            let ouro_chars = ['O', '=', '~', '-', '=', '~', '-', 'O'];
            let head_color = Vec4::new(0.9, 0.8, 0.2, 1.0);
            let body_color = Vec4::new(0.5, 0.6, 0.3, 1.0);
            let tail_color = Vec4::new(0.3, 0.4, 0.2, 0.7);
            for i in 0..count {
                let t = i as f32 / count as f32;
                chars.push(ouro_chars[i % ouro_chars.len()]);
                if t < 0.1 || t > 0.9 {
                    colors.push(head_color); // Head/mouth region
                } else if t < 0.3 {
                    colors.push(body_color);
                } else {
                    colors.push(lerp_color(body_color, tail_color, (t - 0.3) / 0.6));
                }
            }
        }
        BossVisualProfile::AlgorithmReborn => {
            // Massive complex formation — phase-dependent reshaping
            let algo_chars = match phase {
                0 => &['0', '1', '+', '-', '=', '>', '<'][..],
                1 => &['#', '@', '!', '?', '*', '&', '%'][..],
                2 => &['A', 'L', 'G', 'O', 'R', 'I', 'T'][..],
                _ => &['X', 'X', '#', '#', '!', '!', '*'][..],
            };
            let phase_color = match phase {
                0 => Vec4::new(0.3, 0.8, 0.3, 1.0),
                1 => Vec4::new(0.8, 0.5, 0.2, 1.0),
                2 => Vec4::new(0.9, 0.2, 0.2, 1.0),
                _ => Vec4::new(1.0, 0.1, 0.1, 1.0),
            };
            for i in 0..count {
                chars.push(algo_chars[i % algo_chars.len()]);
                let dist = positions.get(i).map(|p| p.length()).unwrap_or(0.0);
                let bright = (1.0 - dist * 0.1).clamp(0.4, 1.0);
                colors.push(Vec4::new(
                    phase_color.x * bright,
                    phase_color.y * bright,
                    phase_color.z * bright,
                    phase_color.w,
                ));
            }
        }
        BossVisualProfile::ChaosWeaver => {
            // Formation constantly shifts between random patterns
            let chaos_chars = ['?', '!', '#', '*', '~', '@', '%', '&', '$', '^'];
            for i in 0..count {
                // Use time-seeded deterministic "random" index
                let seed = (i as u32).wrapping_mul(2654435761);
                let char_idx = (seed as usize + phase as usize) % chaos_chars.len();
                chars.push(chaos_chars[char_idx]);
                // Color: chaotic rainbow
                let hue = (i as f32 * 0.3 + phase as f32 * 0.5) % 1.0;
                colors.push(hue_to_color(hue, 0.8));
            }
        }
        BossVisualProfile::VoidSerpent => {
            // Long sinusoidal body of glyphs
            let serpent_chars = ['<', '=', '~', '-', '=', '~', '-', '>'];
            let head_color = Vec4::new(0.6, 0.1, 0.8, 1.0);
            let body_color = Vec4::new(0.3, 0.05, 0.5, 0.9);
            for i in 0..count {
                let t = i as f32 / count as f32;
                if i == 0 {
                    chars.push('<');
                    colors.push(head_color);
                } else if i == count - 1 {
                    chars.push('>');
                    colors.push(body_color);
                } else {
                    chars.push(serpent_chars[i % serpent_chars.len()]);
                    colors.push(lerp_color(head_color, body_color, t));
                }
            }
        }
        BossVisualProfile::PrimeFactorial => {
            // Number-glyph entity that factors on damage
            let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            let operators = ['+', '*', '/', '-', '=', '!'];
            let num_color = Vec4::new(0.2, 0.8, 0.9, 1.0);
            let op_color = Vec4::new(0.9, 0.9, 0.4, 1.0);
            for i in 0..count {
                if i % 3 == 0 {
                    chars.push(operators[i / 3 % operators.len()]);
                    colors.push(op_color);
                } else {
                    let digit_seed = (i as u32 + phase * 7).wrapping_mul(2654435761);
                    chars.push(digits[(digit_seed % 10) as usize]);
                    colors.push(num_color);
                }
            }
        }
    }

    (chars, colors)
}

// ── Boss formation shapes ────────────────────────────────────────────────────

fn boss_formation_shape(profile: BossVisualProfile, phase: u32) -> FormationShape {
    match profile {
        BossVisualProfile::Mirror => FormationShape::Diamond,
        BossVisualProfile::Null => FormationShape::Ring,
        BossVisualProfile::Committee => FormationShape::Semicircle,
        BossVisualProfile::FibonacciHydra => {
            if phase == 0 {
                FormationShape::Cluster
            } else {
                FormationShape::Swarm
            }
        }
        BossVisualProfile::Eigenstate => {
            if phase % 2 == 0 {
                FormationShape::Star
            } else {
                FormationShape::Diamond
            }
        }
        BossVisualProfile::Ouroboros => FormationShape::Ring,
        BossVisualProfile::AlgorithmReborn => match phase {
            0 => FormationShape::Grid,
            1 => FormationShape::Diamond,
            2 => FormationShape::Star,
            _ => FormationShape::Pentagram,
        },
        BossVisualProfile::ChaosWeaver => {
            // Cycle through formations based on phase
            let shapes = [
                FormationShape::Star,
                FormationShape::Spiral,
                FormationShape::Cross,
                FormationShape::Triangle,
                FormationShape::Pentagon,
            ];
            shapes[(phase as usize) % shapes.len()]
        }
        BossVisualProfile::VoidSerpent => FormationShape::Snake,
        BossVisualProfile::PrimeFactorial => FormationShape::Grid,
    }
}

// ── Element-based formation selection ────────────────────────────────────────

fn element_formation_shape(element: EnemyElement, tier: EnemyVisualTier) -> FormationShape {
    match tier {
        EnemyVisualTier::Common | EnemyVisualTier::Uncommon => {
            // Simple formations for low tiers
            match element {
                EnemyElement::Fire => FormationShape::Triangle,
                EnemyElement::Ice => FormationShape::Line,
                EnemyElement::Lightning => FormationShape::Vee,
                EnemyElement::Poison => FormationShape::Cluster,
                EnemyElement::Shadow => FormationShape::Crescent,
                EnemyElement::Holy => FormationShape::Cross,
                EnemyElement::Neutral => FormationShape::Cluster,
            }
        }
        _ => element.preferred_formation(),
    }
}

// ── Idle animation per tier/element ──────────────────────────────────────────

fn apply_enemy_idle(
    positions: &[Vec3],
    tier: EnemyVisualTier,
    element: EnemyElement,
    time: f32,
) -> Vec<Vec3> {
    let rate = tier.pulse_rate();
    let depth = 0.04;

    match element {
        EnemyElement::Fire => {
            // Flickering upward drift
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let flicker = (time * 6.0 + i as f32 * 2.1).sin() * 0.05;
                    let rise = (time * rate * TAU + i as f32).sin() * depth;
                    *p + Vec3::new(flicker, rise.abs() * 0.3, 0.0)
                })
                .collect()
        }
        EnemyElement::Ice => {
            // Slow crystalline pulse
            formations::apply_breathing(positions, time, rate * 0.5, depth * 0.7)
        }
        EnemyElement::Lightning => {
            // Jittery, electric arc flicker
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let jitter_x = (time * 15.0 + i as f32 * 3.7).sin() * 0.03;
                    let jitter_y = (time * 12.0 + i as f32 * 5.1).cos() * 0.03;
                    *p + Vec3::new(jitter_x, jitter_y, 0.0)
                })
                .collect()
        }
        EnemyElement::Poison => {
            // Bubbling, irregular pulse
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let bubble = ((time * 3.0 + i as f32 * 1.3).sin() * depth).max(0.0);
                    *p + Vec3::new(0.0, bubble, 0.0)
                })
                .collect()
        }
        EnemyElement::Shadow => {
            // Shadow tendrils: slow undulation
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let wave = (time * 1.5 + p.y * 2.0 + i as f32 * 0.5).sin() * 0.06;
                    *p + Vec3::new(wave, 0.0, 0.0)
                })
                .collect()
        }
        EnemyElement::Holy => {
            // Radiant pulse from center
            positions
                .iter()
                .map(|p| {
                    let dist = p.length();
                    let wave = (time * rate * TAU - dist * 3.0).sin() * depth;
                    *p * (1.0 + wave)
                })
                .collect()
        }
        EnemyElement::Neutral => {
            formations::apply_breathing(positions, time, rate, depth)
        }
    }
}

// ── Boss-specific animation ──────────────────────────────────────────────────

fn apply_boss_animation(
    positions: &[Vec3],
    profile: BossVisualProfile,
    phase: u32,
    time: f32,
) -> Vec<Vec3> {
    match profile {
        BossVisualProfile::Mirror => {
            // Subtle mirror shimmer
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let shimmer = (time * 3.0 + i as f32 * 0.8).sin() * 0.02;
                    Vec3::new(p.x + shimmer, p.y, p.z)
                })
                .collect()
        }
        BossVisualProfile::Null => {
            // Pulsing void: contract/expand
            let pulse = (time * 0.8).sin() * 0.15;
            positions.iter().map(|p| *p * (1.0 + pulse)).collect()
        }
        BossVisualProfile::Committee => {
            // Each judge sub-group bobs independently
            let count = positions.len();
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let judge = (i * 5) / count.max(1);
                    let bob = (time * 1.5 + judge as f32 * 1.2).sin() * 0.08;
                    *p + Vec3::new(0.0, bob, 0.0)
                })
                .collect()
        }
        BossVisualProfile::FibonacciHydra => {
            // Heads sway independently
            let heads = (phase + 1).min(5) as usize;
            let head_count = (positions.len() / heads).max(1);
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let head = i / head_count;
                    let sway = (time * 2.0 + head as f32 * PI * 0.4).sin() * 0.1;
                    *p + Vec3::new(sway, 0.0, 0.0)
                })
                .collect()
        }
        BossVisualProfile::Eigenstate => {
            // Two overlapping states shimmer in/out
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let offset = if i % 2 == 0 { 1.0 } else { -1.0 };
                    let shift = (time * 1.0).sin() * 0.15 * offset;
                    *p + Vec3::new(shift, 0.0, 0.0)
                })
                .collect()
        }
        BossVisualProfile::Ouroboros => {
            // Rotate the ring
            formations::apply_rotation(positions, time, 0.3)
        }
        BossVisualProfile::AlgorithmReborn => {
            // Phase-dependent pulsing intensity
            let intensity = 0.03 + phase as f32 * 0.02;
            let pulse = (time * (1.0 + phase as f32 * 0.3)).sin() * intensity;
            positions.iter().map(|p| *p * (1.0 + pulse)).collect()
        }
        BossVisualProfile::ChaosWeaver => {
            // Random jitter on all glyphs
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let seed = (i as u32).wrapping_mul(2654435761);
                    let jx = (time * 8.0 + seed as f32 * 0.001).sin() * 0.08;
                    let jy = (time * 7.0 + seed as f32 * 0.0013).cos() * 0.08;
                    *p + Vec3::new(jx, jy, 0.0)
                })
                .collect()
        }
        BossVisualProfile::VoidSerpent => {
            // Sinusoidal body motion
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let t = i as f32 / positions.len() as f32;
                    let wave = (time * 2.0 + t * TAU * 1.5).sin() * 0.15;
                    *p + Vec3::new(0.0, wave, 0.0)
                })
                .collect()
        }
        BossVisualProfile::PrimeFactorial => {
            // Grid pulses in waves
            positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let wave = (time * 3.0 + i as f32 * 0.5).sin() * 0.03;
                    *p * (1.0 + wave)
                })
                .collect()
        }
    }
}

// ── Element inference from name ──────────────────────────────────────────────

/// Guess an element from an enemy name.
pub fn element_from_name(name: &str) -> EnemyElement {
    let lower = name.to_lowercase();
    if lower.contains("fire") || lower.contains("flame") || lower.contains("ember")
        || lower.contains("inferno") || lower.contains("pyro") || lower.contains("blaze")
    {
        EnemyElement::Fire
    } else if lower.contains("ice") || lower.contains("frost") || lower.contains("crystal")
        || lower.contains("cryo") || lower.contains("frozen") || lower.contains("glacier")
    {
        EnemyElement::Ice
    } else if lower.contains("lightning") || lower.contains("thunder") || lower.contains("volt")
        || lower.contains("shock") || lower.contains("electric") || lower.contains("spark")
    {
        EnemyElement::Lightning
    } else if lower.contains("poison") || lower.contains("toxic") || lower.contains("venom")
        || lower.contains("acid") || lower.contains("plague") || lower.contains("bile")
    {
        EnemyElement::Poison
    } else if lower.contains("shadow") || lower.contains("dark") || lower.contains("void")
        || lower.contains("night") || lower.contains("abyss") || lower.contains("umbra")
    {
        EnemyElement::Shadow
    } else if lower.contains("holy") || lower.contains("light") || lower.contains("radiant")
        || lower.contains("divine") || lower.contains("sacred") || lower.contains("celestial")
    {
        EnemyElement::Holy
    } else {
        EnemyElement::Neutral
    }
}

// ── Utility ──────────────────────────────────────────────────────────────────

/// Lerp between two Vec4 colors.
fn lerp_color(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Convert a hue [0, 1] to an RGB Vec4 with given saturation.
fn hue_to_color(hue: f32, saturation: f32) -> Vec4 {
    let h = hue * 6.0;
    let c = saturation;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Vec4::new(r, g, b, 1.0)
}

/// Get the visual tier for a numeric tier value (backwards compatible).
pub fn tier_to_visual(tier: u32) -> EnemyVisualTier {
    EnemyVisualTier::from_numeric(tier)
}

/// Get element and profile from enemy name for external callers.
pub fn classify_enemy(name: &str, tier: u32) -> (EnemyVisualTier, EnemyElement, Option<BossVisualProfile>) {
    let visual_tier = EnemyVisualTier::from_numeric(tier);
    let element = element_from_name(name);
    let boss_profile = if visual_tier == EnemyVisualTier::Boss {
        BossVisualProfile::from_name(name)
    } else {
        None
    };
    (visual_tier, element, boss_profile)
}
