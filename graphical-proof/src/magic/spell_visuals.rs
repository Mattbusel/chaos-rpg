//! Full spell visual system for every element and spell type.
//!
//! 8 elements, 5 visual stages per spell, 40+ individual spell visuals.
//! All rendering via immediate-mode glyph spawning each frame.

use proof_engine::prelude::*;
use glam::{Vec3, Vec4};

// ── Element definitions ─────────────────────────────────────────────────────

/// The 8 magical elements in CHAOS RPG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Element {
    Fire,
    Ice,
    Lightning,
    Poison,
    Shadow,
    Holy,
    Arcane,
    Chaos,
}

impl Element {
    /// Base color palette: (primary, secondary, accent).
    pub fn colors(&self) -> (Vec4, Vec4, Vec4) {
        match self {
            Element::Fire      => (Vec4::new(1.0, 0.3, 0.05, 1.0), Vec4::new(1.0, 0.6, 0.1, 1.0), Vec4::new(1.0, 0.9, 0.2, 1.0)),
            Element::Ice       => (Vec4::new(0.2, 0.6, 1.0, 1.0),  Vec4::new(0.5, 0.85, 1.0, 1.0), Vec4::new(0.9, 0.95, 1.0, 1.0)),
            Element::Lightning => (Vec4::new(1.0, 1.0, 0.3, 1.0),  Vec4::new(0.8, 0.9, 1.0, 1.0),  Vec4::new(1.0, 1.0, 1.0, 1.0)),
            Element::Poison    => (Vec4::new(0.1, 0.8, 0.2, 1.0),  Vec4::new(0.4, 0.9, 0.1, 1.0),  Vec4::new(0.7, 1.0, 0.3, 1.0)),
            Element::Shadow    => (Vec4::new(0.3, 0.05, 0.5, 1.0), Vec4::new(0.15, 0.0, 0.3, 1.0), Vec4::new(0.6, 0.2, 0.8, 1.0)),
            Element::Holy      => (Vec4::new(1.0, 0.95, 0.5, 1.0), Vec4::new(1.0, 0.85, 0.3, 1.0), Vec4::new(1.0, 1.0, 0.8, 1.0)),
            Element::Arcane    => (Vec4::new(0.4, 0.2, 1.0, 1.0),  Vec4::new(0.6, 0.4, 1.0, 1.0),  Vec4::new(0.8, 0.6, 1.0, 1.0)),
            Element::Chaos     => (Vec4::new(1.0, 0.0, 0.5, 1.0),  Vec4::new(0.0, 1.0, 0.5, 1.0),  Vec4::new(0.5, 0.0, 1.0, 1.0)),
        }
    }

    /// Glyph set for this element (5+ chars).
    pub fn glyphs(&self) -> &[char] {
        match self {
            Element::Fire      => &['🜂', '∆', '▲', '♨', '⚡', '∿'],
            Element::Ice       => &['❄', '✱', '◇', '❆', '✧', '◆'],
            Element::Lightning => &['⚡', '↯', '϶', '⌁', '∿', '↝'],
            Element::Poison    => &['☠', '⊛', '◉', '⊙', '⊕', '∞'],
            Element::Shadow    => &['◐', '◑', '●', '◍', '◎', '⊘'],
            Element::Holy      => &['✝', '☆', '✦', '◈', '✧', '⊕'],
            Element::Arcane    => &['✦', '⊛', '◈', '⟐', '⊗', '⊞'],
            Element::Chaos     => &['✶', '⊕', '⊘', '⊗', '⊞', '⊠'],
        }
    }

    /// Particle count multiplier for this element.
    pub fn particle_density(&self) -> usize {
        match self {
            Element::Fire      => 12,
            Element::Ice       => 8,
            Element::Lightning => 15,
            Element::Poison    => 10,
            Element::Shadow    => 6,
            Element::Holy      => 10,
            Element::Arcane    => 9,
            Element::Chaos     => 14,
        }
    }

    /// Trail style: how many trail segments to render behind a projectile.
    pub fn trail_length(&self) -> usize {
        match self {
            Element::Fire      => 8,
            Element::Ice       => 5,
            Element::Lightning => 12,
            Element::Poison    => 7,
            Element::Shadow    => 10,
            Element::Holy      => 6,
            Element::Arcane    => 7,
            Element::Chaos     => 9,
        }
    }
}

// ── Spell visual stages ─────────────────────────────────────────────────────

/// The 5 stages of a spell's visual lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellVisualStage {
    Charge,     // Growing rune circle at caster position
    Cast,       // Flash at caster, rune release, projectile spawn
    Travel,     // Projectile moves toward target with trail
    Impact,     // Explosion at target
    Aftermath,  // Lingering effect
}

// ── Spell catalog ───────────────────────────────────────────────────────────

/// Every named spell visual in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellVisual {
    // Fire
    Fireball, Inferno, Meteor, FlameWall, Immolate, FireShield, PhoenixStrike,
    // Ice
    IceLance, Blizzard, FrostNova, GlacialSpike, IceWall, FrozenTomb,
    // Lightning
    ChainLightning, ThunderStrike, BallLightning, Storm, StaticField,
    // Poison
    ToxicSpray, Plague, VenomStrike, Miasma, Decay,
    // Shadow
    ShadowBolt, VoidZone, DarkPact, SoulDrain, Eclipse,
    // Holy
    Smite, DivineLight, Sanctuary, Resurrect, Judgment,
    // Arcane
    MagicMissile, ArcaneBlast, ManaBurn, Dispel, Counterspell,
    // Chaos
    ChaosBolt, RealityTear, Entropy,
}

impl SpellVisual {
    pub fn element(&self) -> Element {
        match self {
            Self::Fireball | Self::Inferno | Self::Meteor | Self::FlameWall
            | Self::Immolate | Self::FireShield | Self::PhoenixStrike => Element::Fire,

            Self::IceLance | Self::Blizzard | Self::FrostNova | Self::GlacialSpike
            | Self::IceWall | Self::FrozenTomb => Element::Ice,

            Self::ChainLightning | Self::ThunderStrike | Self::BallLightning
            | Self::Storm | Self::StaticField => Element::Lightning,

            Self::ToxicSpray | Self::Plague | Self::VenomStrike
            | Self::Miasma | Self::Decay => Element::Poison,

            Self::ShadowBolt | Self::VoidZone | Self::DarkPact
            | Self::SoulDrain | Self::Eclipse => Element::Shadow,

            Self::Smite | Self::DivineLight | Self::Sanctuary
            | Self::Resurrect | Self::Judgment => Element::Holy,

            Self::MagicMissile | Self::ArcaneBlast | Self::ManaBurn
            | Self::Dispel | Self::Counterspell => Element::Arcane,

            Self::ChaosBolt | Self::RealityTear | Self::Entropy => Element::Chaos,
        }
    }

    /// Try to match a spell name string to a known visual.
    pub fn from_name(name: &str) -> Option<Self> {
        let n = name.to_lowercase();
        // Fire
        if n.contains("fireball") { return Some(Self::Fireball); }
        if n.contains("inferno") { return Some(Self::Inferno); }
        if n.contains("meteor") { return Some(Self::Meteor); }
        if n.contains("flame wall") || n.contains("flamewall") { return Some(Self::FlameWall); }
        if n.contains("immolate") { return Some(Self::Immolate); }
        if n.contains("fire shield") || n.contains("fireshield") { return Some(Self::FireShield); }
        if n.contains("phoenix") { return Some(Self::PhoenixStrike); }
        // Ice
        if n.contains("ice lance") || n.contains("icelance") { return Some(Self::IceLance); }
        if n.contains("blizzard") { return Some(Self::Blizzard); }
        if n.contains("frost nova") || n.contains("frostnova") { return Some(Self::FrostNova); }
        if n.contains("glacial") { return Some(Self::GlacialSpike); }
        if n.contains("ice wall") || n.contains("icewall") { return Some(Self::IceWall); }
        if n.contains("frozen tomb") { return Some(Self::FrozenTomb); }
        // Lightning
        if n.contains("chain lightning") || n.contains("chainlightning") { return Some(Self::ChainLightning); }
        if n.contains("thunder") { return Some(Self::ThunderStrike); }
        if n.contains("ball lightning") || n.contains("balllightning") { return Some(Self::BallLightning); }
        if n.contains("storm") { return Some(Self::Storm); }
        if n.contains("static") { return Some(Self::StaticField); }
        // Poison
        if n.contains("toxic") { return Some(Self::ToxicSpray); }
        if n.contains("plague") { return Some(Self::Plague); }
        if n.contains("venom") { return Some(Self::VenomStrike); }
        if n.contains("miasma") { return Some(Self::Miasma); }
        if n.contains("decay") { return Some(Self::Decay); }
        // Shadow
        if n.contains("shadow bolt") || n.contains("shadowbolt") { return Some(Self::ShadowBolt); }
        if n.contains("void zone") || n.contains("voidzone") { return Some(Self::VoidZone); }
        if n.contains("dark pact") || n.contains("darkpact") { return Some(Self::DarkPact); }
        if n.contains("soul drain") || n.contains("souldrain") { return Some(Self::SoulDrain); }
        if n.contains("eclipse") { return Some(Self::Eclipse); }
        // Holy
        if n.contains("smite") { return Some(Self::Smite); }
        if n.contains("divine") { return Some(Self::DivineLight); }
        if n.contains("sanctuary") { return Some(Self::Sanctuary); }
        if n.contains("resurrect") { return Some(Self::Resurrect); }
        if n.contains("judgment") || n.contains("judgement") { return Some(Self::Judgment); }
        // Arcane
        if n.contains("magic missile") || n.contains("magicmissile") { return Some(Self::MagicMissile); }
        if n.contains("arcane blast") || n.contains("arcaneblast") { return Some(Self::ArcaneBlast); }
        if n.contains("mana burn") || n.contains("manaburn") { return Some(Self::ManaBurn); }
        if n.contains("dispel") { return Some(Self::Dispel); }
        if n.contains("counterspell") || n.contains("counter spell") { return Some(Self::Counterspell); }
        // Chaos
        if n.contains("chaos bolt") || n.contains("chaosbolt") { return Some(Self::ChaosBolt); }
        if n.contains("reality tear") || n.contains("realitytear") { return Some(Self::RealityTear); }
        if n.contains("entropy") { return Some(Self::Entropy); }
        // Fallbacks by element keyword
        if n.contains("fire") || n.contains("burn") || n.contains("blaze") { return Some(Self::Fireball); }
        if n.contains("ice") || n.contains("frost") || n.contains("freeze") { return Some(Self::IceLance); }
        if n.contains("lightning") || n.contains("shock") { return Some(Self::ChainLightning); }
        if n.contains("poison") { return Some(Self::ToxicSpray); }
        if n.contains("shadow") || n.contains("dark") { return Some(Self::ShadowBolt); }
        if n.contains("holy") || n.contains("heal") || n.contains("light") { return Some(Self::Smite); }
        if n.contains("arcane") || n.contains("magic") { return Some(Self::MagicMissile); }
        if n.contains("chaos") { return Some(Self::ChaosBolt); }
        None
    }

    /// Screen shake intensity for this spell's impact.
    fn shake_intensity(&self) -> f32 {
        match self {
            Self::Meteor | Self::PhoenixStrike | Self::ThunderStrike => 0.6,
            Self::Inferno | Self::Blizzard | Self::Storm | Self::Judgment
            | Self::RealityTear | Self::Entropy => 0.5,
            Self::FrostNova | Self::Eclipse | Self::ArcaneBlast => 0.4,
            Self::Fireball | Self::ChainLightning | Self::ChaosBolt => 0.3,
            _ => 0.2,
        }
    }

    /// Whether this spell should flash the screen on impact.
    fn flash_on_impact(&self) -> bool {
        matches!(self,
            Self::Meteor | Self::PhoenixStrike | Self::ThunderStrike
            | Self::FrostNova | Self::DivineLight | Self::Judgment
            | Self::ArcaneBlast | Self::RealityTear | Self::Entropy
        )
    }
}

// ── Active spell instance ───────────────────────────────────────────────────

/// A single active spell animation being rendered.
#[derive(Clone, Debug)]
pub struct ActiveSpell {
    pub visual: SpellVisual,
    pub stage: SpellVisualStage,
    pub timer: f32,
    pub caster_pos: Vec3,
    pub target_pos: Vec3,
    pub frame_seed: u64,
}

impl ActiveSpell {
    pub fn new(visual: SpellVisual, caster: Vec3, target: Vec3, seed: u64) -> Self {
        Self {
            visual,
            stage: SpellVisualStage::Charge,
            timer: 0.0,
            caster_pos: caster,
            target_pos: target,
            frame_seed: seed,
        }
    }

    /// Duration of each stage in seconds.
    fn stage_duration(&self) -> f32 {
        match self.stage {
            SpellVisualStage::Charge    => 0.4,
            SpellVisualStage::Cast      => 0.15,
            SpellVisualStage::Travel    => 0.5,
            SpellVisualStage::Impact    => 0.3,
            SpellVisualStage::Aftermath => 0.8,
        }
    }

    /// Advance the timer and potentially move to the next stage.
    /// Returns true if the spell animation is still active.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.timer += dt;
        if self.timer >= self.stage_duration() {
            self.timer = 0.0;
            self.stage = match self.stage {
                SpellVisualStage::Charge    => SpellVisualStage::Cast,
                SpellVisualStage::Cast      => SpellVisualStage::Travel,
                SpellVisualStage::Travel    => SpellVisualStage::Impact,
                SpellVisualStage::Impact    => SpellVisualStage::Aftermath,
                SpellVisualStage::Aftermath => return false,
            };
        }
        true
    }

    /// Progress within the current stage (0.0 to 1.0).
    fn progress(&self) -> f32 {
        (self.timer / self.stage_duration()).clamp(0.0, 1.0)
    }
}

// ── Spell Visual Manager ────────────────────────────────────────────────────

/// Manages all active spell animations and renders them each frame.
pub struct SpellVisualManager {
    pub active_spells: Vec<ActiveSpell>,
}

impl SpellVisualManager {
    pub fn new() -> Self {
        Self { active_spells: Vec::new() }
    }

    /// Begin a new spell animation.
    pub fn cast_spell(&mut self, visual: SpellVisual, caster: Vec3, target: Vec3, seed: u64) {
        self.active_spells.push(ActiveSpell::new(visual, caster, target, seed));
    }

    /// Begin a spell animation from a spell name string.
    pub fn cast_by_name(&mut self, name: &str, caster: Vec3, target: Vec3, seed: u64) {
        if let Some(vis) = SpellVisual::from_name(name) {
            self.cast_spell(vis, caster, target, seed);
        }
    }

    /// Tick all active spells and remove finished ones.
    pub fn update(&mut self, dt: f32) {
        self.active_spells.retain_mut(|spell| spell.tick(dt));
    }

    /// Render all active spell visuals.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        for spell in &self.active_spells {
            render_spell(engine, spell, frame);
        }
    }
}

// ── Core rendering ──────────────────────────────────────────────────────────

fn render_spell(engine: &mut ProofEngine, spell: &ActiveSpell, frame: u64) {
    match spell.stage {
        SpellVisualStage::Charge    => render_charge(engine, spell, frame),
        SpellVisualStage::Cast      => render_cast(engine, spell, frame),
        SpellVisualStage::Travel    => render_travel(engine, spell, frame),
        SpellVisualStage::Impact    => render_impact(engine, spell, frame),
        SpellVisualStage::Aftermath => render_aftermath(engine, spell, frame),
    }
    // Dispatch spell-specific overlays
    render_spell_specific(engine, spell, frame);
}

// ── Stage: Charge ───────────────────────────────────────────────────────────

fn render_charge(engine: &mut ProofEngine, spell: &ActiveSpell, frame: u64) {
    let elem = spell.visual.element();
    let (c1, c2, _c3) = elem.colors();
    let glyphs = elem.glyphs();
    let t = spell.progress();
    let pos = spell.caster_pos;

    // Growing rune circle
    let radius = 0.5 + t * 2.0;
    let num_runes = 8 + (t * 8.0) as usize;
    for i in 0..num_runes {
        let angle_base = (i as f32 / num_runes as f32) * std::f32::consts::TAU;
        // Orbiting glyphs accelerate as charge progresses
        let spin_speed = 1.0 + t * 4.0;
        let angle = angle_base + frame as f32 * 0.05 * spin_speed;
        let rx = pos.x + angle.cos() * radius;
        let ry = pos.y + angle.sin() * radius;
        let glyph_char = glyphs[i % glyphs.len()];
        let color = lerp_color(c1, c2, (i as f32 / num_runes as f32 + t).fract());
        engine.spawn_glyph(Glyph {
            character: glyph_char,
            position: Vec3::new(rx, ry, 0.0),
            color: Vec4::new(color.x, color.y, color.z, 0.4 + t * 0.6),
            emission: 0.3 + t * 0.7,
            glow_color: Vec3::new(c1.x, c1.y, c1.z),
            glow_radius: t * 1.5,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Central gathering glyph
    let pulse = ((frame as f32 * 0.15).sin() * 0.3 + 0.7).max(0.0);
    engine.spawn_glyph(Glyph {
        character: glyphs[0],
        position: pos,
        scale: Vec2::splat(0.8 + t * 0.5),
        color: Vec4::new(c2.x, c2.y, c2.z, pulse),
        emission: 0.5 + t * 1.0,
        glow_color: Vec3::new(c2.x, c2.y, c2.z),
        glow_radius: t * 2.0,
        layer: RenderLayer::Particle,
        ..Default::default()
    });
}

// ── Stage: Cast ─────────────────────────────────────────────────────────────

fn render_cast(engine: &mut ProofEngine, spell: &ActiveSpell, frame: u64) {
    let elem = spell.visual.element();
    let (c1, _c2, c3) = elem.colors();
    let t = spell.progress();
    let pos = spell.caster_pos;

    // Flash burst at caster
    let flash_alpha = (1.0 - t).max(0.0);
    let flash_radius = 1.0 + t * 3.0;
    for i in 0..12 {
        let angle = (i as f32 / 12.0) * std::f32::consts::TAU;
        let r = flash_radius;
        engine.spawn_glyph(Glyph {
            character: '*',
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(c3.x, c3.y, c3.z, flash_alpha * 0.8),
            emission: flash_alpha * 1.5,
            glow_color: Vec3::new(c1.x, c1.y, c1.z),
            glow_radius: flash_alpha * 2.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Rune circle release — expanding ring
    let ring_r = 2.5 + t * 4.0;
    let ring_alpha = (1.0 - t * 0.7).max(0.0);
    let glyphs = elem.glyphs();
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU + frame as f32 * 0.1;
        engine.spawn_glyph(Glyph {
            character: glyphs[i % glyphs.len()],
            position: Vec3::new(pos.x + angle.cos() * ring_r, pos.y + angle.sin() * ring_r, 0.0),
            color: Vec4::new(c1.x, c1.y, c1.z, ring_alpha * 0.5),
            emission: ring_alpha * 0.6,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Apply screen shake for cast
    if t < 0.1 {
        engine.add_trauma(spell.visual.shake_intensity() * 0.3);
    }
}

// ── Stage: Travel ───────────────────────────────────────────────────────────

fn render_travel(engine: &mut ProofEngine, spell: &ActiveSpell, frame: u64) {
    let elem = spell.visual.element();
    let (c1, c2, c3) = elem.colors();
    let glyphs = elem.glyphs();
    let t = spell.progress();

    // Projectile position: lerp from caster to target
    let proj_pos = Vec3::new(
        spell.caster_pos.x + (spell.target_pos.x - spell.caster_pos.x) * t,
        spell.caster_pos.y + (spell.target_pos.y - spell.caster_pos.y) * t
            + (t * std::f32::consts::PI).sin() * 1.2, // arc
        0.0,
    );

    // Main projectile glyph
    engine.spawn_glyph(Glyph {
        character: glyphs[0],
        position: proj_pos,
        scale: Vec2::splat(1.2),
        color: c1,
        emission: 1.5,
        glow_color: Vec3::new(c1.x, c1.y, c1.z),
        glow_radius: 2.5,
        layer: RenderLayer::Particle,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    // Trail behind projectile (element-specific)
    let trail_len = elem.trail_length();
    for i in 1..=trail_len {
        let trail_t = (t - i as f32 * 0.03).max(0.0);
        let trail_pos = Vec3::new(
            spell.caster_pos.x + (spell.target_pos.x - spell.caster_pos.x) * trail_t,
            spell.caster_pos.y + (spell.target_pos.y - spell.caster_pos.y) * trail_t
                + (trail_t * std::f32::consts::PI).sin() * 1.2,
            0.0,
        );
        let fade = 1.0 - (i as f32 / trail_len as f32);
        let trail_char = match elem {
            Element::Fire      => ['·', '∿', '~', '*', '.', '\'', ',', '+'][i % 8],
            Element::Ice       => ['·', '✧', '*', '◇', '.', '+', '`', '-'][i % 8],
            Element::Lightning => ['·', '↝', '~', '|', '/', '\\', '-', '+'][i % 8],
            Element::Poison    => ['·', 'o', '.', '~', ',', '`', '\'', '+'][i % 8],
            Element::Shadow    => ['·', '◐', '.', ':', ',', ';', '`', '+'][i % 8],
            Element::Holy      => ['·', '✧', '+', '*', '.', '`', '\'', '-'][i % 8],
            Element::Arcane    => ['·', '✦', '*', '+', '.', '`', '\'', '-'][i % 8],
            Element::Chaos     => ['·', '✶', '?', '!', '#', '@', '&', '%'][i % 8],
        };
        let jitter_x = ((frame as f32 * 0.3 + i as f32 * 7.1).sin()) * 0.15;
        let jitter_y = ((frame as f32 * 0.4 + i as f32 * 3.7).cos()) * 0.15;
        let trail_color = lerp_color(c1, c2, i as f32 / trail_len as f32);
        engine.spawn_glyph(Glyph {
            character: trail_char,
            position: Vec3::new(trail_pos.x + jitter_x, trail_pos.y + jitter_y, 0.0),
            color: Vec4::new(trail_color.x, trail_color.y, trail_color.z, fade * 0.7),
            emission: fade * 0.8,
            glow_color: Vec3::new(c2.x, c2.y, c2.z),
            glow_radius: fade * 1.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Orbiting particles around projectile
    let density = elem.particle_density();
    for i in 0..density {
        let angle = (i as f32 / density as f32) * std::f32::consts::TAU + frame as f32 * 0.15;
        let r = 0.3 + ((frame as f32 * 0.1 + i as f32).sin().abs()) * 0.4;
        engine.spawn_glyph(Glyph {
            character: glyphs[(i + 1) % glyphs.len()],
            position: Vec3::new(proj_pos.x + angle.cos() * r, proj_pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(c3.x, c3.y, c3.z, 0.5),
            emission: 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── Stage: Impact ───────────────────────────────────────────────────────────

fn render_impact(engine: &mut ProofEngine, spell: &ActiveSpell, frame: u64) {
    let elem = spell.visual.element();
    let (c1, c2, c3) = elem.colors();
    let glyphs = elem.glyphs();
    let t = spell.progress();
    let pos = spell.target_pos;

    // Apply screen shake and flash
    if t < 0.1 {
        engine.add_trauma(spell.visual.shake_intensity());
    }

    // Element-specific impact shape
    match elem {
        Element::Fire => render_impact_fire(engine, pos, t, c1, c2, c3, glyphs, frame),
        Element::Ice => render_impact_ice(engine, pos, t, c1, c2, c3, glyphs, frame),
        Element::Lightning => render_impact_lightning(engine, pos, t, c1, c2, c3, glyphs, frame),
        Element::Poison => render_impact_poison(engine, pos, t, c1, c2, c3, glyphs, frame),
        Element::Shadow => render_impact_shadow(engine, pos, t, c1, c2, c3, glyphs, frame),
        Element::Holy => render_impact_holy(engine, pos, t, c1, c2, c3, glyphs, frame),
        Element::Arcane => render_impact_arcane(engine, pos, t, c1, c2, c3, glyphs, frame),
        Element::Chaos => render_impact_chaos(engine, pos, t, c1, c2, c3, glyphs, frame),
    }
}

// Fire impact: radial burst with embers scattering outward
fn render_impact_fire(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, c3: Vec4, glyphs: &[char], frame: u64,
) {
    let burst_r = 0.5 + t * 4.0;
    let alpha = (1.0 - t).max(0.0);
    for i in 0..16 {
        let angle = (i as f32 / 16.0) * std::f32::consts::TAU + t * 2.0;
        let r = burst_r + ((frame as f32 * 0.2 + i as f32).sin()) * 0.5;
        let color = lerp_color(c1, c3, t);
        engine.spawn_glyph(Glyph {
            character: glyphs[i % glyphs.len()],
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(color.x, color.y, color.z, alpha),
            emission: alpha * 1.2,
            glow_color: Vec3::new(c2.x, c2.y, c2.z),
            glow_radius: alpha * 2.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
    // Rising embers
    for i in 0..8 {
        let x_off = ((frame as f32 * 0.3 + i as f32 * 5.0).sin()) * 2.0;
        let y_off = t * 3.0 + i as f32 * 0.3;
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(pos.x + x_off, pos.y + y_off, 0.0),
            color: Vec4::new(1.0, 0.6, 0.1, alpha * 0.6),
            emission: alpha * 0.5,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// Ice impact: shatter pattern — shards fly outward then freeze
fn render_impact_ice(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, c3: Vec4, glyphs: &[char], frame: u64,
) {
    let shard_count = 12;
    let alpha = (1.0 - t * 0.8).max(0.0);
    for i in 0..shard_count {
        let angle = (i as f32 / shard_count as f32) * std::f32::consts::TAU;
        let r = t * 3.5;
        // Shards slow down (decelerate)
        let decel_r = r * (1.0 - t * 0.4);
        let shard_char = ['◇', '◆', '✧', '❄', '◈', '✱'][i % 6];
        let color = lerp_color(c1, c3, i as f32 / shard_count as f32);
        engine.spawn_glyph(Glyph {
            character: shard_char,
            position: Vec3::new(pos.x + angle.cos() * decel_r, pos.y + angle.sin() * decel_r, 0.0),
            color: Vec4::new(color.x, color.y, color.z, alpha),
            emission: alpha * 0.8,
            glow_color: Vec3::new(c2.x, c2.y, c2.z),
            glow_radius: alpha * 1.2,
            rotation: angle,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
    // Central frost flash
    let flash = (1.0 - t * 2.0).max(0.0);
    if flash > 0.0 {
        engine.spawn_glyph(Glyph {
            character: '❄',
            position: pos,
            scale: Vec2::splat(1.5 + t),
            color: Vec4::new(c3.x, c3.y, c3.z, flash),
            emission: flash * 2.0,
            glow_color: Vec3::new(1.0, 1.0, 1.0),
            glow_radius: flash * 3.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// Lightning impact: chain segments branching outward
fn render_impact_lightning(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, _c3: Vec4, _glyphs: &[char], frame: u64,
) {
    let alpha = (1.0 - t).max(0.0);
    let branches = 5;
    for b in 0..branches {
        let base_angle = (b as f32 / branches as f32) * std::f32::consts::TAU
            + ((frame as f32 * 0.5).sin()) * 0.3;
        let segs = 6;
        let mut cx = pos.x;
        let mut cy = pos.y;
        for s in 0..segs {
            let seg_angle = base_angle + ((frame as f32 * 0.7 + s as f32 * 13.0 + b as f32 * 7.0).sin()) * 0.5;
            let seg_len = 0.4 + t * 0.3;
            cx += seg_angle.cos() * seg_len;
            cy += seg_angle.sin() * seg_len;
            let bolt_char = ['/', '\\', '|', '-', '~'][s % 5];
            let fade = alpha * (1.0 - s as f32 / segs as f32);
            engine.spawn_glyph(Glyph {
                character: bolt_char,
                position: Vec3::new(cx, cy, 0.0),
                color: Vec4::new(c1.x, c1.y, c1.z, fade),
                emission: fade * 1.5,
                glow_color: Vec3::new(c2.x, c2.y, c2.z),
                glow_radius: fade * 1.8,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
    // Central flash
    let flash = (1.0 - t * 3.0).max(0.0);
    if flash > 0.0 {
        engine.spawn_glyph(Glyph {
            character: '⚡',
            position: pos,
            scale: Vec2::splat(1.3),
            color: Vec4::new(1.0, 1.0, 0.8, flash),
            emission: flash * 2.5,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// Poison impact: expanding cloud with dripping particles
fn render_impact_poison(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, _c3: Vec4, glyphs: &[char], frame: u64,
) {
    let alpha = (1.0 - t * 0.7).max(0.0);
    let cloud_r = 1.0 + t * 3.0;
    // Cloud particles
    for i in 0..14 {
        let seed = i as f32 * 17.3 + frame as f32 * 0.1;
        let x = pos.x + seed.sin() * cloud_r;
        let y = pos.y + seed.cos() * cloud_r * 0.7;
        let color = lerp_color(c1, c2, (seed * 0.1).fract());
        engine.spawn_glyph(Glyph {
            character: glyphs[i % glyphs.len()],
            position: Vec3::new(x, y, 0.0),
            color: Vec4::new(color.x, color.y, color.z, alpha * 0.6),
            emission: alpha * 0.5,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
    // Dripping particles falling
    for i in 0..6 {
        let x_off = ((i as f32 * 3.7 + frame as f32 * 0.2).sin()) * 1.5;
        let y_off = -t * 2.0 - i as f32 * 0.4;
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(pos.x + x_off, pos.y + y_off, 0.0),
            color: Vec4::new(c1.x, c1.y, c1.z, alpha * 0.5),
            emission: 0.2,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// Shadow impact: imploding void then dark ripple
fn render_impact_shadow(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, c3: Vec4, glyphs: &[char], frame: u64,
) {
    let alpha = (1.0 - t * 0.6).max(0.0);
    // Imploding ring (shrinks then expands)
    let r = if t < 0.4 { 3.0 * (1.0 - t / 0.4) } else { (t - 0.4) / 0.6 * 4.0 };
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * std::f32::consts::TAU;
        let color = lerp_color(c1, c3, t);
        engine.spawn_glyph(Glyph {
            character: glyphs[i % glyphs.len()],
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(color.x, color.y, color.z, alpha * 0.7),
            emission: alpha * 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
    // Central void
    engine.spawn_glyph(Glyph {
        character: '●',
        position: pos,
        scale: Vec2::splat(0.8 + (1.0 - r / 4.0).abs() * 0.5),
        color: Vec4::new(0.0, 0.0, 0.0, alpha),
        emission: 0.0,
        layer: RenderLayer::Particle,
        ..Default::default()
    });
}

// Holy impact: divine rays radiating outward
fn render_impact_holy(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, c3: Vec4, glyphs: &[char], frame: u64,
) {
    let alpha = (1.0 - t * 0.8).max(0.0);
    let rays = 8;
    for i in 0..rays {
        let angle = (i as f32 / rays as f32) * std::f32::consts::TAU;
        let ray_len = t * 5.0;
        for s in 0..5 {
            let r = s as f32 / 5.0 * ray_len;
            let fade = alpha * (1.0 - s as f32 / 5.0);
            let ray_char = ['✦', '✧', '+', '*', '.'][s % 5];
            engine.spawn_glyph(Glyph {
                character: ray_char,
                position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
                color: Vec4::new(c3.x, c3.y, c3.z, fade),
                emission: fade * 1.2,
                glow_color: Vec3::new(c1.x, c1.y, c1.z),
                glow_radius: fade * 1.5,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
    // Central cross
    let cross_chars = ['✝', '☆', '✦'];
    engine.spawn_glyph(Glyph {
        character: cross_chars[(frame as usize) % cross_chars.len()],
        position: pos,
        scale: Vec2::splat(1.5),
        color: Vec4::new(c2.x, c2.y, c2.z, alpha),
        emission: alpha * 2.0,
        glow_color: Vec3::new(1.0, 0.95, 0.5),
        glow_radius: alpha * 3.0,
        layer: RenderLayer::Particle,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });
}

// Arcane impact: geometric burst with rotating symbols
fn render_impact_arcane(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, c3: Vec4, glyphs: &[char], frame: u64,
) {
    let alpha = (1.0 - t).max(0.0);
    // Hexagonal burst
    let hex_r = 0.5 + t * 3.5;
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU + frame as f32 * 0.08;
        let color = lerp_color(c1, c3, (i as f32 / 6.0 + t * 0.5).fract());
        engine.spawn_glyph(Glyph {
            character: glyphs[i % glyphs.len()],
            position: Vec3::new(pos.x + angle.cos() * hex_r, pos.y + angle.sin() * hex_r, 0.0),
            color: Vec4::new(color.x, color.y, color.z, alpha),
            emission: alpha * 1.0,
            glow_color: Vec3::new(c2.x, c2.y, c2.z),
            glow_radius: alpha * 1.5,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
    // Inner rotating triangle
    let inner_r = hex_r * 0.5;
    for i in 0..3 {
        let angle = (i as f32 / 3.0) * std::f32::consts::TAU - frame as f32 * 0.12;
        engine.spawn_glyph(Glyph {
            character: '⊛',
            position: Vec3::new(pos.x + angle.cos() * inner_r, pos.y + angle.sin() * inner_r, 0.0),
            color: Vec4::new(c2.x, c2.y, c2.z, alpha * 0.8),
            emission: alpha * 0.8,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// Chaos impact: random element visual, screen distortion
fn render_impact_chaos(
    engine: &mut ProofEngine, pos: Vec3, t: f32,
    c1: Vec4, c2: Vec4, c3: Vec4, _glyphs: &[char], frame: u64,
) {
    let alpha = (1.0 - t).max(0.0);
    // Rapid color cycling through all elements
    let elements = [Element::Fire, Element::Ice, Element::Lightning, Element::Poison,
                    Element::Shadow, Element::Holy, Element::Arcane];
    let elem_idx = (frame as usize / 3) % elements.len();
    let chaos_elem = elements[elem_idx];
    let (ec1, ec2, _ec3) = chaos_elem.colors();
    let chaos_glyphs = chaos_elem.glyphs();

    // Multi-element burst
    let burst_r = 1.0 + t * 4.0;
    for i in 0..20 {
        let e_idx = (i + frame as usize) % elements.len();
        let (ecc1, _ecc2, _ecc3) = elements[e_idx].colors();
        let angle = (i as f32 / 20.0) * std::f32::consts::TAU + t * 3.0;
        let r = burst_r + ((i as f32 * 7.3 + frame as f32 * 0.4).sin()) * 0.8;
        engine.spawn_glyph(Glyph {
            character: chaos_glyphs[i % chaos_glyphs.len()],
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(ecc1.x, ecc1.y, ecc1.z, alpha * 0.7),
            emission: alpha * 1.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
    // Glitch characters at random positions
    let glitch_chars = ['!', '?', '#', '@', '%', '&', '~', '^'];
    for i in 0..8 {
        let seed = i as f32 * 31.7 + frame as f32 * 0.9;
        let gx = pos.x + seed.sin() * 3.0;
        let gy = pos.y + seed.cos() * 2.0;
        engine.spawn_glyph(Glyph {
            character: glitch_chars[(frame as usize + i) % glitch_chars.len()],
            position: Vec3::new(gx, gy, 0.0),
            color: Vec4::new(c1.x, c2.y, c3.z, alpha * 0.5),
            emission: alpha * 0.6,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── Stage: Aftermath ────────────────────────────────────────────────────────

fn render_aftermath(engine: &mut ProofEngine, spell: &ActiveSpell, frame: u64) {
    let elem = spell.visual.element();
    let (c1, c2, _c3) = elem.colors();
    let t = spell.progress();
    let pos = spell.target_pos;
    let alpha = (1.0 - t).max(0.0);

    match elem {
        Element::Fire => {
            // Ground flame — flickering fire chars on ground
            for i in 0..8 {
                let x_off = ((i as f32 * 3.1 + frame as f32 * 0.15).sin()) * 2.0;
                let flicker = ((frame as f32 * 0.3 + i as f32 * 5.0).sin() * 0.3 + 0.7).max(0.0);
                let fire_chars = ['∆', '▲', '∿', '~'];
                engine.spawn_glyph(Glyph {
                    character: fire_chars[i % fire_chars.len()],
                    position: Vec3::new(pos.x + x_off, pos.y - 0.5, 0.0),
                    color: Vec4::new(c1.x * flicker, c1.y * flicker, c1.z, alpha * 0.6),
                    emission: alpha * flicker * 0.5,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }
        Element::Ice => {
            // Frost patch — ice crystals on ground
            for i in 0..10 {
                let angle = (i as f32 / 10.0) * std::f32::consts::TAU;
                let r = 1.5 * (1.0 - t * 0.3);
                let ice_chars = ['❄', '✧', '◇', '·', '*'];
                engine.spawn_glyph(Glyph {
                    character: ice_chars[i % ice_chars.len()],
                    position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r * 0.4, 0.0),
                    color: Vec4::new(c2.x, c2.y, c2.z, alpha * 0.5),
                    emission: alpha * 0.3,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
        }
        Element::Lightning => {
            // Static sparks lingering
            for i in 0..6 {
                if (frame + i as u64) % 4 != 0 { continue; }
                let sx = pos.x + ((frame as f32 * 0.7 + i as f32 * 19.0).sin()) * 2.0;
                let sy = pos.y + ((frame as f32 * 0.5 + i as f32 * 13.0).cos()) * 1.5;
                engine.spawn_glyph(Glyph {
                    character: '⚡',
                    position: Vec3::new(sx, sy, 0.0),
                    color: Vec4::new(1.0, 1.0, 0.5, alpha * 0.4),
                    emission: alpha * 0.8,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
        Element::Poison => {
            // Lingering poison cloud
            for i in 0..10 {
                let seed = i as f32 * 11.3 + frame as f32 * 0.05;
                let x = pos.x + seed.sin() * 2.0;
                let y = pos.y + seed.cos() * 1.2 + t * 0.5;
                engine.spawn_glyph(Glyph {
                    character: '~',
                    position: Vec3::new(x, y, 0.0),
                    color: Vec4::new(c1.x, c1.y, c1.z, alpha * 0.3),
                    emission: alpha * 0.2,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }
        Element::Shadow => {
            // Void zone — dark patch that fades
            for i in 0..8 {
                let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                let r = 1.0 + (1.0 - t) * 1.5;
                engine.spawn_glyph(Glyph {
                    character: '.',
                    position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
                    color: Vec4::new(0.1, 0.0, 0.15, alpha * 0.5),
                    emission: 0.0,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
        }
        Element::Holy => {
            // Rising sparkle particles
            for i in 0..6 {
                let x_off = ((i as f32 * 5.0 + frame as f32 * 0.1).sin()) * 1.5;
                let y_off = t * 2.0 + i as f32 * 0.3;
                engine.spawn_glyph(Glyph {
                    character: '+',
                    position: Vec3::new(pos.x + x_off, pos.y + y_off, 0.0),
                    color: Vec4::new(1.0, 0.95, 0.5, alpha * 0.4),
                    emission: alpha * 0.5,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
        Element::Arcane => {
            // Residual arcane symbols rotating
            for i in 0..4 {
                let angle = (i as f32 / 4.0) * std::f32::consts::TAU + frame as f32 * 0.03;
                let r = 1.0;
                let arc_chars = ['✦', '⊛', '◈', '⊗'];
                engine.spawn_glyph(Glyph {
                    character: arc_chars[i % arc_chars.len()],
                    position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
                    color: Vec4::new(c1.x, c1.y, c1.z, alpha * 0.4),
                    emission: alpha * 0.3,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }
        Element::Chaos => {
            // Visual glitches fading out
            let glitch_chars = ['█', '▓', '▒', '░', '#', '!'];
            for i in 0..8 {
                let seed = i as f32 * 47.3 + frame as f32 * 0.3;
                let gx = pos.x + seed.sin() * 2.5;
                let gy = pos.y + seed.cos() * 1.8;
                engine.spawn_glyph(Glyph {
                    character: glitch_chars[(frame as usize + i) % glitch_chars.len()],
                    position: Vec3::new(gx, gy, 0.0),
                    color: Vec4::new(
                        (seed * 0.3).sin().abs(),
                        (seed * 0.7).cos().abs(),
                        (seed * 1.1).sin().abs(),
                        alpha * 0.3,
                    ),
                    emission: alpha * 0.2,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }
    }
}

// ── Spell-specific overlays ─────────────────────────────────────────────────

fn render_spell_specific(engine: &mut ProofEngine, spell: &ActiveSpell, frame: u64) {
    // Only render during Travel and Impact stages for spell-specific extras
    if spell.stage != SpellVisualStage::Travel && spell.stage != SpellVisualStage::Impact {
        return;
    }
    let t = spell.progress();

    match spell.visual {
        // ── Fire spells ─────────────────────────────────────────────
        SpellVisual::Meteor => {
            if spell.stage == SpellVisualStage::Travel {
                // Meteor comes from above — override Y arc
                let proj_x = spell.caster_pos.x + (spell.target_pos.x - spell.caster_pos.x) * t;
                let proj_y = spell.target_pos.y + 8.0 * (1.0 - t);
                // Large fiery body
                engine.spawn_glyph(Glyph {
                    character: '●',
                    position: Vec3::new(proj_x, proj_y, 0.0),
                    scale: Vec2::splat(2.0),
                    color: Vec4::new(1.0, 0.4, 0.0, 1.0),
                    emission: 2.0,
                    glow_color: Vec3::new(1.0, 0.3, 0.0),
                    glow_radius: 3.0,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
                // Smoke trail above
                for i in 0..6 {
                    let ty = proj_y + 0.5 + i as f32 * 0.4;
                    let jx = ((frame as f32 * 0.2 + i as f32 * 3.0).sin()) * 0.3;
                    let fade = 1.0 - i as f32 / 6.0;
                    engine.spawn_glyph(Glyph {
                        character: '░',
                        position: Vec3::new(proj_x + jx, ty, 0.0),
                        color: Vec4::new(0.5, 0.3, 0.1, fade * 0.4),
                        emission: fade * 0.2,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::FlameWall => {
            if spell.stage == SpellVisualStage::Impact {
                // Wall of fire across target area
                for i in 0..16 {
                    let wx = spell.target_pos.x - 3.0 + i as f32 * 0.4;
                    let wy = spell.target_pos.y;
                    let flicker = ((frame as f32 * 0.4 + i as f32 * 2.0).sin() * 0.3 + 0.7).max(0.0);
                    let fire_chars = ['∆', '▲', '∿', '~', '|'];
                    let h = ((frame as f32 * 0.2 + i as f32 * 1.5).sin().abs()) * 1.5 + 0.5;
                    engine.spawn_glyph(Glyph {
                        character: fire_chars[i % fire_chars.len()],
                        position: Vec3::new(wx, wy + h, 0.0),
                        color: Vec4::new(1.0 * flicker, 0.4 * flicker, 0.05, (1.0 - t) * 0.8),
                        emission: flicker * 0.8,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Inferno => {
            if spell.stage == SpellVisualStage::Impact {
                // Massive fire explosion — extra large burst
                let burst_r = 2.0 + t * 6.0;
                for i in 0..24 {
                    let angle = (i as f32 / 24.0) * std::f32::consts::TAU + frame as f32 * 0.1;
                    let r = burst_r + ((i as f32 * 3.0).sin()) * 1.0;
                    engine.spawn_glyph(Glyph {
                        character: '🜂',
                        position: Vec3::new(
                            spell.target_pos.x + angle.cos() * r,
                            spell.target_pos.y + angle.sin() * r,
                            0.0,
                        ),
                        color: Vec4::new(1.0, 0.5 - t * 0.3, 0.05, (1.0 - t) * 0.7),
                        emission: (1.0 - t) * 1.5,
                        glow_color: Vec3::new(1.0, 0.3, 0.0),
                        glow_radius: (1.0 - t) * 2.5,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Immolate => {
            if spell.stage == SpellVisualStage::Impact {
                // Flames wrap around target
                for i in 0..12 {
                    let angle = (i as f32 / 12.0) * std::f32::consts::TAU + frame as f32 * 0.2;
                    let r = 1.0 + ((frame as f32 * 0.15 + i as f32).sin()) * 0.3;
                    engine.spawn_glyph(Glyph {
                        character: '∿',
                        position: Vec3::new(
                            spell.target_pos.x + angle.cos() * r,
                            spell.target_pos.y + angle.sin() * r,
                            0.0,
                        ),
                        color: Vec4::new(1.0, 0.4, 0.1, (1.0 - t) * 0.8),
                        emission: (1.0 - t) * 0.7,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::FireShield => {
            if spell.stage == SpellVisualStage::Impact {
                // Fire shield around caster instead
                let pos = spell.caster_pos;
                for i in 0..16 {
                    let angle = (i as f32 / 16.0) * std::f32::consts::TAU + frame as f32 * 0.1;
                    let r = 2.0;
                    let flicker = ((frame as f32 * 0.3 + i as f32 * 2.0).sin() * 0.2 + 0.8).max(0.0);
                    engine.spawn_glyph(Glyph {
                        character: '▲',
                        position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
                        color: Vec4::new(1.0 * flicker, 0.3 * flicker, 0.05, (1.0 - t) * 0.7),
                        emission: flicker * 0.6,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::PhoenixStrike => {
            if spell.stage == SpellVisualStage::Impact {
                // Phoenix shape rising from impact
                let rise = t * 4.0;
                let phoenix_glyphs = ['V', '/', '\\', '^', '~', '`'];
                for i in 0..12 {
                    let x_off = ((i as f32 - 6.0) * 0.4).abs() * (if i < 6 { -1.0 } else { 1.0 });
                    let y_off = rise + (i as f32 * 0.3).abs();
                    let fade = (1.0 - t * 0.8).max(0.0);
                    engine.spawn_glyph(Glyph {
                        character: phoenix_glyphs[i % phoenix_glyphs.len()],
                        position: Vec3::new(
                            spell.target_pos.x + x_off,
                            spell.target_pos.y + y_off,
                            0.0,
                        ),
                        color: Vec4::new(1.0, 0.6, 0.0, fade),
                        emission: fade * 1.2,
                        glow_color: Vec3::new(1.0, 0.4, 0.0),
                        glow_radius: fade * 2.0,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }

        // ── Ice spells ──────────────────────────────────────────────
        SpellVisual::Blizzard => {
            if spell.stage == SpellVisualStage::Impact {
                // Snowflakes falling across wide area
                for i in 0..20 {
                    let x = spell.target_pos.x - 4.0 + ((frame as f32 * 0.1 + i as f32 * 3.7).sin() + 1.0) * 4.0;
                    let y = spell.target_pos.y + 3.0 - (frame as f32 * 0.05 + i as f32 * 0.7) % 6.0;
                    let snow_chars = ['*', '❄', '·', '✧', '.'];
                    engine.spawn_glyph(Glyph {
                        character: snow_chars[i % snow_chars.len()],
                        position: Vec3::new(x, y, 0.0),
                        color: Vec4::new(0.7, 0.9, 1.0, (1.0 - t) * 0.5),
                        emission: 0.3,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::FrostNova => {
            if spell.stage == SpellVisualStage::Impact {
                // Expanding ring of ice from center
                let nova_r = t * 5.0;
                for i in 0..20 {
                    let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
                    let ice_chars = ['❄', '✱', '◇', '❆'];
                    engine.spawn_glyph(Glyph {
                        character: ice_chars[i % ice_chars.len()],
                        position: Vec3::new(
                            spell.target_pos.x + angle.cos() * nova_r,
                            spell.target_pos.y + angle.sin() * nova_r,
                            0.0,
                        ),
                        color: Vec4::new(0.3, 0.7, 1.0, (1.0 - t) * 0.8),
                        emission: (1.0 - t) * 1.0,
                        glow_color: Vec3::new(0.4, 0.7, 1.0),
                        glow_radius: (1.0 - t) * 2.0,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::GlacialSpike => {
            if spell.stage == SpellVisualStage::Impact {
                // Large spike growing upward from target
                let spike_h = t * 4.0;
                for i in 0..8 {
                    let y_off = i as f32 * spike_h / 8.0;
                    let width = (1.0 - i as f32 / 8.0) * 0.8;
                    engine.spawn_glyph(Glyph {
                        character: '◆',
                        position: Vec3::new(spell.target_pos.x, spell.target_pos.y + y_off, 0.0),
                        scale: Vec2::new(width, 1.0),
                        color: Vec4::new(0.5, 0.85, 1.0, (1.0 - t * 0.5)),
                        emission: 0.8,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::IceWall => {
            if spell.stage == SpellVisualStage::Impact {
                // Wall of ice blocks
                for i in 0..10 {
                    for j in 0..3 {
                        let wx = spell.target_pos.x - 2.0 + i as f32 * 0.45;
                        let wy = spell.target_pos.y + j as f32 * 0.5;
                        let ice_alpha = ((1.0 - t) * 0.9).max(0.0);
                        engine.spawn_glyph(Glyph {
                            character: '█',
                            position: Vec3::new(wx, wy, 0.0),
                            color: Vec4::new(0.3, 0.65, 1.0, ice_alpha),
                            emission: ice_alpha * 0.4,
                            layer: RenderLayer::Particle,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        SpellVisual::FrozenTomb => {
            if spell.stage == SpellVisualStage::Impact {
                // Ice encasing the target
                let pos = spell.target_pos;
                for i in 0..16 {
                    let angle = (i as f32 / 16.0) * std::f32::consts::TAU;
                    let r = 1.2;
                    let ice_chars = ['█', '▓', '▒', '░'];
                    engine.spawn_glyph(Glyph {
                        character: ice_chars[i % ice_chars.len()],
                        position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
                        color: Vec4::new(0.4, 0.7, 1.0, (1.0 - t) * 0.9),
                        emission: (1.0 - t) * 0.5,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }

        // ── Lightning spells ────────────────────────────────────────
        SpellVisual::ChainLightning => {
            if spell.stage == SpellVisualStage::Impact {
                // Secondary chains bouncing to nearby positions
                let chain_targets = 3;
                for c in 0..chain_targets {
                    let chain_angle = (c as f32 / chain_targets as f32) * std::f32::consts::TAU + 0.5;
                    let cx = spell.target_pos.x + chain_angle.cos() * 3.0;
                    let cy = spell.target_pos.y + chain_angle.sin() * 2.0;
                    // Draw bolt segments from target to chain target
                    let segs = 4;
                    for s in 0..segs {
                        let st = s as f32 / segs as f32;
                        let sx = spell.target_pos.x + (cx - spell.target_pos.x) * st;
                        let sy = spell.target_pos.y + (cy - spell.target_pos.y) * st;
                        let jx = ((frame as f32 * 0.8 + s as f32 * 11.0 + c as f32 * 7.0).sin()) * 0.3;
                        let jy = ((frame as f32 * 0.6 + s as f32 * 13.0).cos()) * 0.2;
                        let bolt_chars = ['/', '\\', '|', '-'];
                        engine.spawn_glyph(Glyph {
                            character: bolt_chars[s % bolt_chars.len()],
                            position: Vec3::new(sx + jx, sy + jy, 0.0),
                            color: Vec4::new(1.0, 1.0, 0.4, (1.0 - t) * 0.7),
                            emission: (1.0 - t) * 1.2,
                            layer: RenderLayer::Particle,
                            blend_mode: BlendMode::Additive,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        SpellVisual::BallLightning => {
            if spell.stage == SpellVisualStage::Travel {
                // Orbiting sparks around the ball
                let proj_t = spell.progress();
                let px = spell.caster_pos.x + (spell.target_pos.x - spell.caster_pos.x) * proj_t;
                let py = spell.caster_pos.y + (spell.target_pos.y - spell.caster_pos.y) * proj_t;
                for i in 0..8 {
                    let angle = (i as f32 / 8.0) * std::f32::consts::TAU + frame as f32 * 0.3;
                    let r = 0.6;
                    let spark_chars = ['·', '*', '\'', '`'];
                    engine.spawn_glyph(Glyph {
                        character: spark_chars[i % spark_chars.len()],
                        position: Vec3::new(px + angle.cos() * r, py + angle.sin() * r, 0.0),
                        color: Vec4::new(0.8, 0.9, 1.0, 0.6),
                        emission: 0.8,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Storm => {
            if spell.stage == SpellVisualStage::Impact {
                // Multiple bolts striking from above
                let bolt_count = 4;
                for b in 0..bolt_count {
                    let bx = spell.target_pos.x - 2.0 + ((frame as f32 * 0.1 + b as f32 * 3.0).sin() + 1.0) * 2.0;
                    let top_y = spell.target_pos.y + 5.0;
                    let bot_y = spell.target_pos.y;
                    let segs = 5;
                    for s in 0..segs {
                        let sy = top_y - (top_y - bot_y) * s as f32 / segs as f32;
                        let jx = ((frame as f32 * 0.9 + s as f32 * 7.0 + b as f32 * 11.0).sin()) * 0.4;
                        engine.spawn_glyph(Glyph {
                            character: '|',
                            position: Vec3::new(bx + jx, sy, 0.0),
                            color: Vec4::new(1.0, 1.0, 0.5, (1.0 - t) * 0.6),
                            emission: (1.0 - t) * 1.0,
                            layer: RenderLayer::Particle,
                            blend_mode: BlendMode::Additive,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        SpellVisual::StaticField => {
            if spell.stage == SpellVisualStage::Impact {
                // Electric field — random sparks in area
                for i in 0..12 {
                    if (frame + i as u64) % 3 != 0 { continue; }
                    let sx = spell.target_pos.x + ((frame as f32 * 0.5 + i as f32 * 17.0).sin()) * 3.0;
                    let sy = spell.target_pos.y + ((frame as f32 * 0.4 + i as f32 * 13.0).cos()) * 2.0;
                    engine.spawn_glyph(Glyph {
                        character: '⚡',
                        position: Vec3::new(sx, sy, 0.0),
                        scale: Vec2::splat(0.6),
                        color: Vec4::new(1.0, 1.0, 0.3, (1.0 - t) * 0.5),
                        emission: (1.0 - t) * 0.7,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }

        // ── Poison spells ───────────────────────────────────────────
        SpellVisual::Plague => {
            if spell.stage == SpellVisualStage::Impact {
                // Spreading green circles
                let spread = t * 5.0;
                for ring in 0..3 {
                    let r = spread * (ring as f32 + 1.0) / 3.0;
                    let alpha = (1.0 - t) * (1.0 - ring as f32 / 3.0);
                    for i in 0..8 {
                        let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                        engine.spawn_glyph(Glyph {
                            character: '☠',
                            position: Vec3::new(
                                spell.target_pos.x + angle.cos() * r,
                                spell.target_pos.y + angle.sin() * r,
                                0.0,
                            ),
                            color: Vec4::new(0.2, 0.8, 0.1, alpha * 0.5),
                            emission: alpha * 0.4,
                            layer: RenderLayer::Particle,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        SpellVisual::Miasma => {
            if spell.stage == SpellVisualStage::Impact {
                // Dense green fog
                for i in 0..18 {
                    let seed = i as f32 * 7.7 + frame as f32 * 0.08;
                    let x = spell.target_pos.x + seed.sin() * 3.0;
                    let y = spell.target_pos.y + seed.cos() * 2.0;
                    engine.spawn_glyph(Glyph {
                        character: '~',
                        position: Vec3::new(x, y, 0.0),
                        color: Vec4::new(0.15, 0.7, 0.2, (1.0 - t) * 0.35),
                        emission: (1.0 - t) * 0.2,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Decay => {
            if spell.stage == SpellVisualStage::Impact {
                // Dark green particles falling
                for i in 0..10 {
                    let x_off = ((i as f32 * 4.3 + frame as f32 * 0.2).sin()) * 2.0;
                    let y_off = -t * 3.0 - i as f32 * 0.3;
                    engine.spawn_glyph(Glyph {
                        character: '.',
                        position: Vec3::new(spell.target_pos.x + x_off, spell.target_pos.y + y_off, 0.0),
                        color: Vec4::new(0.1, 0.5, 0.1, (1.0 - t) * 0.5),
                        emission: 0.1,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }

        // ── Shadow spells ───────────────────────────────────────────
        SpellVisual::VoidZone => {
            if spell.stage == SpellVisualStage::Impact {
                // Growing black hole at target
                let void_r = 0.5 + t * 3.0;
                for i in 0..12 {
                    let angle = (i as f32 / 12.0) * std::f32::consts::TAU + frame as f32 * 0.05;
                    let r = void_r;
                    engine.spawn_glyph(Glyph {
                        character: '●',
                        position: Vec3::new(
                            spell.target_pos.x + angle.cos() * r,
                            spell.target_pos.y + angle.sin() * r,
                            0.0,
                        ),
                        color: Vec4::new(0.1, 0.0, 0.15, (1.0 - t) * 0.8),
                        emission: 0.0,
                        layer: RenderLayer::Overlay,
                        ..Default::default()
                    });
                }
                // Center void
                engine.spawn_glyph(Glyph {
                    character: '⊘',
                    position: spell.target_pos,
                    scale: Vec2::splat(1.5),
                    color: Vec4::new(0.3, 0.0, 0.5, (1.0 - t)),
                    emission: (1.0 - t) * 0.3,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }
        SpellVisual::DarkPact => {
            if spell.stage == SpellVisualStage::Impact {
                // Dark energy flowing from target to caster (self-damage for power)
                for i in 0..8 {
                    let lt = (t + i as f32 * 0.1) % 1.0;
                    let px = spell.target_pos.x + (spell.caster_pos.x - spell.target_pos.x) * lt;
                    let py = spell.target_pos.y + (spell.caster_pos.y - spell.target_pos.y) * lt;
                    engine.spawn_glyph(Glyph {
                        character: '◐',
                        position: Vec3::new(px, py, 0.0),
                        color: Vec4::new(0.4, 0.0, 0.6, 0.6),
                        emission: 0.4,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::SoulDrain => {
            if spell.stage == SpellVisualStage::Impact {
                // Ghostly wisps flowing from target to caster
                for i in 0..6 {
                    let lt = (t + i as f32 * 0.15) % 1.0;
                    let px = spell.target_pos.x + (spell.caster_pos.x - spell.target_pos.x) * lt;
                    let py = spell.target_pos.y + (spell.caster_pos.y - spell.target_pos.y) * lt
                        + (lt * std::f32::consts::PI).sin() * 0.8;
                    engine.spawn_glyph(Glyph {
                        character: '~',
                        position: Vec3::new(px, py, 0.0),
                        color: Vec4::new(0.5, 0.2, 0.8, 0.5),
                        emission: 0.3,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Eclipse => {
            if spell.stage == SpellVisualStage::Impact {
                // Dark circle with corona
                let pos = spell.target_pos;
                // Dark center
                engine.spawn_glyph(Glyph {
                    character: '●',
                    position: pos,
                    scale: Vec2::splat(2.0),
                    color: Vec4::new(0.0, 0.0, 0.0, (1.0 - t)),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
                // Corona
                for i in 0..16 {
                    let angle = (i as f32 / 16.0) * std::f32::consts::TAU;
                    let r = 2.0 + ((frame as f32 * 0.1 + i as f32).sin()) * 0.3;
                    engine.spawn_glyph(Glyph {
                        character: '·',
                        position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
                        color: Vec4::new(0.6, 0.2, 0.8, (1.0 - t) * 0.6),
                        emission: (1.0 - t) * 0.8,
                        glow_color: Vec3::new(0.5, 0.1, 0.7),
                        glow_radius: (1.0 - t) * 1.5,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }

        // ── Holy spells ─────────────────────────────────────────────
        SpellVisual::DivineLight => {
            if spell.stage == SpellVisualStage::Impact {
                // Column of light from above
                let pos = spell.target_pos;
                for i in 0..10 {
                    let y_off = i as f32 * 0.8;
                    let width = 0.8 - i as f32 * 0.05;
                    let shimmer = ((frame as f32 * 0.2 + i as f32 * 1.5).sin() * 0.2 + 0.8).max(0.0);
                    engine.spawn_glyph(Glyph {
                        character: '|',
                        position: Vec3::new(pos.x, pos.y + y_off, 0.0),
                        scale: Vec2::new(width, 1.0),
                        color: Vec4::new(1.0, 0.95, 0.5, (1.0 - t) * shimmer),
                        emission: (1.0 - t) * 1.5,
                        glow_color: Vec3::new(1.0, 0.9, 0.4),
                        glow_radius: (1.0 - t) * 2.0,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Sanctuary => {
            if spell.stage == SpellVisualStage::Impact {
                // Protective dome around caster
                let pos = spell.caster_pos;
                let dome_r = 2.5;
                for i in 0..20 {
                    let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
                    let y_scale = ((i as f32 / 20.0) * std::f32::consts::PI).sin();
                    engine.spawn_glyph(Glyph {
                        character: '✧',
                        position: Vec3::new(
                            pos.x + angle.cos() * dome_r,
                            pos.y + y_scale * dome_r * 0.5 + 0.5,
                            0.0,
                        ),
                        color: Vec4::new(1.0, 0.95, 0.6, (1.0 - t) * 0.5),
                        emission: (1.0 - t) * 0.6,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Resurrect => {
            if spell.stage == SpellVisualStage::Impact {
                // Upward rising golden particles + cross
                let pos = spell.target_pos;
                for i in 0..10 {
                    let x_off = ((i as f32 * 3.7 + frame as f32 * 0.1).sin()) * 1.5;
                    let y_off = t * 4.0 + i as f32 * 0.3;
                    engine.spawn_glyph(Glyph {
                        character: '+',
                        position: Vec3::new(pos.x + x_off, pos.y + y_off, 0.0),
                        color: Vec4::new(1.0, 0.9, 0.3, (1.0 - t) * 0.6),
                        emission: (1.0 - t) * 0.8,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Judgment => {
            if spell.stage == SpellVisualStage::Impact {
                // Multiple beams from above converging on target
                let pos = spell.target_pos;
                for b in 0..5 {
                    let bx_off = (b as f32 - 2.0) * 1.5;
                    for s in 0..6 {
                        let st = s as f32 / 6.0;
                        let sx = pos.x + bx_off * (1.0 - st);
                        let sy = pos.y + 6.0 * (1.0 - st);
                        engine.spawn_glyph(Glyph {
                            character: '|',
                            position: Vec3::new(sx, sy, 0.0),
                            color: Vec4::new(1.0, 0.95, 0.4, (1.0 - t) * 0.6 * (1.0 - st * 0.5)),
                            emission: (1.0 - t) * 1.0,
                            layer: RenderLayer::Particle,
                            blend_mode: BlendMode::Additive,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // ── Arcane spells ───────────────────────────────────────────
        SpellVisual::MagicMissile => {
            if spell.stage == SpellVisualStage::Travel {
                // Multiple small homing projectiles
                let proj_t = spell.progress();
                for m in 0..3 {
                    let offset_angle = (m as f32 / 3.0) * std::f32::consts::TAU + frame as f32 * 0.2;
                    let wobble = offset_angle.sin() * 0.5;
                    let mx = spell.caster_pos.x + (spell.target_pos.x - spell.caster_pos.x) * proj_t + wobble;
                    let my = spell.caster_pos.y + (spell.target_pos.y - spell.caster_pos.y) * proj_t
                        + (proj_t * std::f32::consts::PI).sin() * (1.0 + m as f32 * 0.3);
                    engine.spawn_glyph(Glyph {
                        character: '✦',
                        position: Vec3::new(mx, my, 0.0),
                        scale: Vec2::splat(0.7),
                        color: Vec4::new(0.5, 0.3, 1.0, 0.8),
                        emission: 1.0,
                        glow_color: Vec3::new(0.4, 0.2, 0.9),
                        glow_radius: 1.0,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Additive,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::ManaBurn => {
            if spell.stage == SpellVisualStage::Impact {
                // Blue flame consuming mana visually
                let pos = spell.target_pos;
                for i in 0..10 {
                    let angle = (i as f32 / 10.0) * std::f32::consts::TAU + frame as f32 * 0.15;
                    let r = 1.0 + t * 1.5;
                    engine.spawn_glyph(Glyph {
                        character: '∿',
                        position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
                        color: Vec4::new(0.2, 0.3, 1.0, (1.0 - t) * 0.7),
                        emission: (1.0 - t) * 0.8,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Dispel => {
            if spell.stage == SpellVisualStage::Impact {
                // Expanding wave that clears effects
                let wave_r = t * 4.0;
                let wave_alpha = (1.0 - t).max(0.0);
                for i in 0..16 {
                    let angle = (i as f32 / 16.0) * std::f32::consts::TAU;
                    engine.spawn_glyph(Glyph {
                        character: '~',
                        position: Vec3::new(
                            spell.target_pos.x + angle.cos() * wave_r,
                            spell.target_pos.y + angle.sin() * wave_r,
                            0.0,
                        ),
                        color: Vec4::new(0.6, 0.4, 1.0, wave_alpha * 0.5),
                        emission: wave_alpha * 0.5,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Counterspell => {
            if spell.stage == SpellVisualStage::Impact {
                // Shield-like barrier flash + X mark
                let pos = spell.target_pos;
                engine.spawn_glyph(Glyph {
                    character: 'X',
                    position: pos,
                    scale: Vec2::splat(1.5),
                    color: Vec4::new(0.6, 0.3, 1.0, (1.0 - t)),
                    emission: (1.0 - t) * 1.5,
                    glow_color: Vec3::new(0.5, 0.2, 1.0),
                    glow_radius: (1.0 - t) * 2.0,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
                // Expanding barrier ring
                let br = 1.0 + t * 2.0;
                for i in 0..12 {
                    let angle = (i as f32 / 12.0) * std::f32::consts::TAU;
                    engine.spawn_glyph(Glyph {
                        character: '|',
                        position: Vec3::new(pos.x + angle.cos() * br, pos.y + angle.sin() * br, 0.0),
                        color: Vec4::new(0.7, 0.5, 1.0, (1.0 - t) * 0.5),
                        emission: (1.0 - t) * 0.6,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }

        // ── Chaos spells ────────────────────────────────────────────
        SpellVisual::RealityTear => {
            if spell.stage == SpellVisualStage::Impact {
                // Vertical tear in space with glitch effects
                let pos = spell.target_pos;
                let tear_h = 4.0;
                for i in 0..12 {
                    let y_off = -tear_h / 2.0 + i as f32 * tear_h / 12.0;
                    let jx = ((frame as f32 * 0.5 + i as f32 * 5.0).sin()) * 0.3;
                    let tear_chars = ['|', '/', '\\', ':', '!'];
                    engine.spawn_glyph(Glyph {
                        character: tear_chars[i % tear_chars.len()],
                        position: Vec3::new(pos.x + jx, pos.y + y_off, 0.0),
                        color: Vec4::new(
                            ((frame as f32 * 0.3 + i as f32).sin() * 0.5 + 0.5),
                            0.0,
                            ((frame as f32 * 0.5 + i as f32).cos() * 0.5 + 0.5),
                            (1.0 - t) * 0.8,
                        ),
                        emission: (1.0 - t) * 1.0,
                        layer: RenderLayer::Overlay,
                        ..Default::default()
                    });
                }
            }
        }
        SpellVisual::Entropy => {
            if spell.stage == SpellVisualStage::Impact {
                // Everything dissolves — random chars scattered
                for i in 0..25 {
                    let seed = i as f32 * 47.3 + frame as f32 * 0.7;
                    let x = spell.target_pos.x + seed.sin() * 4.0;
                    let y = spell.target_pos.y + seed.cos() * 3.0;
                    let entropy_chars = ['?', '!', '#', '.', ',', ';', ':', '~', '@'];
                    engine.spawn_glyph(Glyph {
                        character: entropy_chars[(frame as usize + i) % entropy_chars.len()],
                        position: Vec3::new(x, y, 0.0),
                        color: Vec4::new(
                            (seed * 0.2).sin().abs(),
                            (seed * 0.5).cos().abs(),
                            (seed * 0.8).sin().abs(),
                            (1.0 - t) * 0.4,
                        ),
                        emission: (1.0 - t) * 0.3,
                        layer: RenderLayer::Overlay,
                        ..Default::default()
                    });
                }
            }
        }

        // Default: no spell-specific overlay needed
        _ => {}
    }
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Linearly interpolate between two Vec4 colors.
fn lerp_color(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    let t = t.clamp(0.0, 1.0);
    Vec4::new(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t,
        a.w + (b.w - a.w) * t,
    )
}
