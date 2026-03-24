//! 30+ particle presets using the proof_engine particle system.
//!
//! Each preset defines spawn count, velocity pattern, color gradient, lifetime,
//! emission, and MathFunction drive. The `ParticlePresetManager` spawns by name,
//! updates all, and renders all active preset instances.

use proof_engine::prelude::*;
use proof_engine::particle::{
    EmitterPreset, ParticleTemplate, EmitterShape, ColorGradient, FloatCurve,
    ScaleCurve, RangeParam, ParticleFlags,
};

// ---------------------------------------------------------------------------
// Hash helper (deterministic pseudo-random, no dep)
// ---------------------------------------------------------------------------

fn hash_f32(seed: u32) -> f32 {
    let mut s = seed;
    s ^= s >> 16;
    s = s.wrapping_mul(0x45d9f3b);
    s ^= s >> 16;
    (s & 0x00FF_FFFF) as f32 / 16_777_215.0
}

fn hash_range(seed: u32, lo: f32, hi: f32) -> f32 {
    lo + hash_f32(seed) * (hi - lo)
}

// ---------------------------------------------------------------------------
// Preset catalogue — returns (name, EmitterPreset) pairs.
// ---------------------------------------------------------------------------

/// Named preset identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PresetId {
    // ── Combat ──
    SwordSlashTrail,
    AxeChopSparks,
    ArrowTrail,
    SpellMissile,
    ShieldBlockSparks,
    CriticalHitBurst,
    ComboMilestoneExplosion,

    // ── Magic ──
    FireBallTrail,
    IceShardShatter,
    LightningChain,
    PoisonCloud,
    HolyRadiance,
    DarkTendrils,
    ArcaneRunesOrbit,

    // ── Ambient ──
    Campfire,
    TorchFlicker,
    ShrineGlow,
    PortalSwirl,
    TreasureSparkle,
    CorruptionSeep,
    VoidWisps,

    // ── UI ──
    LevelUpBurst,
    AchievementUnlock,
    ItemPickupSparkle,
    GoldCoinScatter,
    HealthRestoreGlow,
    ManaRestoreShimmer,

    // ── Extra ──
    BloodMist,
    SoulRelease,
    ChaosSurge,
}

impl PresetId {
    /// Human-readable label.
    pub fn name(self) -> &'static str {
        match self {
            Self::SwordSlashTrail => "sword_slash_trail",
            Self::AxeChopSparks => "axe_chop_sparks",
            Self::ArrowTrail => "arrow_trail",
            Self::SpellMissile => "spell_missile",
            Self::ShieldBlockSparks => "shield_block_sparks",
            Self::CriticalHitBurst => "critical_hit_burst",
            Self::ComboMilestoneExplosion => "combo_milestone_explosion",
            Self::FireBallTrail => "fire_ball_trail",
            Self::IceShardShatter => "ice_shard_shatter",
            Self::LightningChain => "lightning_chain",
            Self::PoisonCloud => "poison_cloud",
            Self::HolyRadiance => "holy_radiance",
            Self::DarkTendrils => "dark_tendrils",
            Self::ArcaneRunesOrbit => "arcane_runes_orbit",
            Self::Campfire => "campfire",
            Self::TorchFlicker => "torch_flicker",
            Self::ShrineGlow => "shrine_glow",
            Self::PortalSwirl => "portal_swirl",
            Self::TreasureSparkle => "treasure_sparkle",
            Self::CorruptionSeep => "corruption_seep",
            Self::VoidWisps => "void_wisps",
            Self::LevelUpBurst => "level_up_burst",
            Self::AchievementUnlock => "achievement_unlock",
            Self::ItemPickupSparkle => "item_pickup_sparkle",
            Self::GoldCoinScatter => "gold_coin_scatter",
            Self::HealthRestoreGlow => "health_restore_glow",
            Self::ManaRestoreShimmer => "mana_restore_shimmer",
            Self::BloodMist => "blood_mist",
            Self::SoulRelease => "soul_release",
            Self::ChaosSurge => "chaos_surge",
        }
    }

    /// All preset IDs for iteration.
    pub fn all() -> &'static [PresetId] {
        &[
            Self::SwordSlashTrail, Self::AxeChopSparks, Self::ArrowTrail,
            Self::SpellMissile, Self::ShieldBlockSparks, Self::CriticalHitBurst,
            Self::ComboMilestoneExplosion,
            Self::FireBallTrail, Self::IceShardShatter, Self::LightningChain,
            Self::PoisonCloud, Self::HolyRadiance, Self::DarkTendrils,
            Self::ArcaneRunesOrbit,
            Self::Campfire, Self::TorchFlicker, Self::ShrineGlow,
            Self::PortalSwirl, Self::TreasureSparkle, Self::CorruptionSeep,
            Self::VoidWisps,
            Self::LevelUpBurst, Self::AchievementUnlock, Self::ItemPickupSparkle,
            Self::GoldCoinScatter, Self::HealthRestoreGlow, Self::ManaRestoreShimmer,
            Self::BloodMist, Self::SoulRelease, Self::ChaosSurge,
        ]
    }
}

// ---------------------------------------------------------------------------
// Build the engine EmitterPreset for each id
// ---------------------------------------------------------------------------

fn build_template(
    character: char,
    lifetime: (f32, f32),
    speed: (f32, f32),
    size: (f32, f32),
    spread: f32,
    drag: f32,
    emission: f32,
    behavior: MathFunction,
    gradient: ColorGradient,
    flags: ParticleFlags,
) -> ParticleTemplate {
    ParticleTemplate {
        lifetime: RangeParam::range(lifetime.0, lifetime.1),
        speed: RangeParam::range(speed.0, speed.1),
        size: RangeParam::range(size.0, size.1),
        spread,
        drag,
        mass: 1.0,
        emission,
        spin: (0.0, 0.0),
        character,
        trail: false,
        trail_length: 0,
        trail_decay: 0.9,
        behavior,
        interaction: proof_engine::particle::ParticleInteraction::None,
        gradient,
        scale_over_life: None,
        color_over_life: None,
        size_over_life: None,
        group: None,
        sub_emitter: None,
        flags,
    }
}

fn gradient_2(c0: Vec4, c1: Vec4) -> ColorGradient {
    ColorGradient::new(vec![(0.0, c0), (1.0, c1)])
}

fn gradient_3(c0: Vec4, c1: Vec4, c2: Vec4) -> ColorGradient {
    ColorGradient::new(vec![(0.0, c0), (0.5, c1), (1.0, c2)])
}

/// Build the `EmitterPreset::Custom` for a given `PresetId`.
pub fn build_preset(id: PresetId) -> (EmitterPreset, u32) {
    // Returns (preset, count)
    match id {
        // ── COMBAT ──────────────────────────────────────────────────────────

        PresetId::SwordSlashTrail => {
            let t = build_template(
                '-',
                (0.2, 0.5),
                (4.0, 8.0),
                (0.15, 0.3),
                0.3,
                2.0,
                0.8,
                MathFunction::Linear { slope: -1.0, offset: 1.0 },
                gradient_2(
                    Vec4::new(0.9, 0.9, 1.0, 1.0),
                    Vec4::new(0.5, 0.5, 0.8, 0.0),
                ),
                ParticleFlags::STRETCH,
            );
            (EmitterPreset::Custom { template: t, count: 16, shape: EmitterShape::Point }, 16)
        }

        PresetId::AxeChopSparks => {
            let t = build_template(
                '*',
                (0.3, 0.7),
                (5.0, 12.0),
                (0.1, 0.2),
                1.2,
                1.5,
                1.2,
                MathFunction::Breathing { rate: 6.0, depth: 0.5 },
                gradient_2(
                    Vec4::new(1.0, 0.8, 0.3, 1.0),
                    Vec4::new(1.0, 0.3, 0.0, 0.0),
                ),
                ParticleFlags::GRAVITY,
            );
            (EmitterPreset::Custom { template: t, count: 24, shape: EmitterShape::Hemisphere { radius: 0.3 } }, 24)
        }

        PresetId::ArrowTrail => {
            let t = build_template(
                '.',
                (0.1, 0.3),
                (1.0, 2.0),
                (0.08, 0.12),
                0.15,
                3.0,
                0.4,
                MathFunction::Constant(0.5),
                gradient_2(
                    Vec4::new(0.6, 0.5, 0.3, 0.6),
                    Vec4::new(0.4, 0.35, 0.2, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 8, shape: EmitterShape::Point }, 8)
        }

        PresetId::SpellMissile => {
            let t = build_template(
                '\u{2726}',
                (0.3, 0.6),
                (3.0, 6.0),
                (0.2, 0.4),
                0.8,
                1.0,
                1.5,
                MathFunction::Sine { amplitude: 0.5, frequency: 4.0, phase: 0.0 },
                gradient_3(
                    Vec4::new(0.4, 0.6, 1.0, 1.0),
                    Vec4::new(0.7, 0.3, 1.0, 0.8),
                    Vec4::new(0.2, 0.1, 0.5, 0.0),
                ),
                ParticleFlags::TRAIL_EMITTER,
            );
            (EmitterPreset::Custom { template: t, count: 12, shape: EmitterShape::Sphere { radius: 0.2 } }, 12)
        }

        PresetId::ShieldBlockSparks => {
            let t = build_template(
                '+',
                (0.2, 0.4),
                (6.0, 10.0),
                (0.12, 0.2),
                1.5,
                2.5,
                1.0,
                MathFunction::Constant(1.0),
                gradient_2(
                    Vec4::new(0.7, 0.8, 1.0, 1.0),
                    Vec4::new(0.3, 0.4, 0.8, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 20, shape: EmitterShape::Hemisphere { radius: 0.5 } }, 20)
        }

        PresetId::CriticalHitBurst => {
            let t = build_template(
                '*',
                (0.4, 0.8),
                (8.0, 15.0),
                (0.2, 0.5),
                std::f32::consts::TAU,
                1.0,
                2.0,
                MathFunction::Breathing { rate: 8.0, depth: 1.0 },
                gradient_3(
                    Vec4::new(1.0, 1.0, 0.5, 1.0),
                    Vec4::new(1.0, 0.5, 0.0, 0.8),
                    Vec4::new(0.8, 0.1, 0.0, 0.0),
                ),
                ParticleFlags::GRAVITY,
            );
            (EmitterPreset::Custom { template: t, count: 32, shape: EmitterShape::Sphere { radius: 0.1 } }, 32)
        }

        PresetId::ComboMilestoneExplosion => {
            let t = build_template(
                '\u{2605}',
                (0.6, 1.2),
                (6.0, 14.0),
                (0.25, 0.6),
                std::f32::consts::TAU,
                0.8,
                2.5,
                MathFunction::Spiral { center: Vec3::ZERO, radius_rate: 2.0, speed: 3.0 },
                gradient_3(
                    Vec4::new(1.0, 0.9, 0.2, 1.0),
                    Vec4::new(1.0, 0.5, 0.1, 0.9),
                    Vec4::new(0.8, 0.2, 0.0, 0.0),
                ),
                ParticleFlags::GRAVITY,
            );
            (EmitterPreset::Custom { template: t, count: 48, shape: EmitterShape::Sphere { radius: 0.2 } }, 48)
        }

        // ── MAGIC ───────────────────────────────────────────────────────────

        PresetId::FireBallTrail => {
            let t = build_template(
                '\u{00b7}',
                (0.3, 0.7),
                (2.0, 5.0),
                (0.15, 0.35),
                0.8,
                1.5,
                1.8,
                MathFunction::Breathing { rate: 5.0, depth: 0.7 },
                gradient_3(
                    Vec4::new(1.0, 0.95, 0.6, 1.0),
                    Vec4::new(1.0, 0.5, 0.1, 0.8),
                    Vec4::new(0.3, 0.05, 0.0, 0.0),
                ),
                ParticleFlags::TRAIL_EMITTER,
            );
            (EmitterPreset::Custom { template: t, count: 20, shape: EmitterShape::Cone { angle: 0.4, length: 0.3 } }, 20)
        }

        PresetId::IceShardShatter => {
            let t = build_template(
                '\u{25c6}',
                (0.4, 0.9),
                (5.0, 10.0),
                (0.1, 0.25),
                std::f32::consts::PI,
                2.0,
                0.8,
                MathFunction::Constant(0.6),
                gradient_2(
                    Vec4::new(0.6, 0.85, 1.0, 1.0),
                    Vec4::new(0.2, 0.4, 0.8, 0.0),
                ),
                ParticleFlags::GRAVITY,
            );
            (EmitterPreset::Custom { template: t, count: 18, shape: EmitterShape::Sphere { radius: 0.3 } }, 18)
        }

        PresetId::LightningChain => {
            let t = build_template(
                '|',
                (0.05, 0.15),
                (15.0, 25.0),
                (0.1, 0.2),
                0.3,
                0.5,
                3.0,
                MathFunction::Square { amplitude: 1.0, frequency: 20.0, duty: 0.5 },
                gradient_2(
                    Vec4::new(1.0, 1.0, 0.9, 1.0),
                    Vec4::new(0.5, 0.7, 1.0, 0.0),
                ),
                ParticleFlags::STRETCH,
            );
            (EmitterPreset::Custom { template: t, count: 10, shape: EmitterShape::Point }, 10)
        }

        PresetId::PoisonCloud => {
            let t = build_template(
                'o',
                (1.0, 2.5),
                (0.5, 1.5),
                (0.2, 0.5),
                std::f32::consts::TAU,
                0.3,
                0.4,
                MathFunction::Perlin { frequency: 1.5, octaves: 2, amplitude: 0.8 },
                gradient_3(
                    Vec4::new(0.2, 0.7, 0.1, 0.7),
                    Vec4::new(0.3, 0.8, 0.2, 0.5),
                    Vec4::new(0.1, 0.3, 0.05, 0.0),
                ),
                ParticleFlags::AFFECTED_BY_FIELDS,
            );
            (EmitterPreset::Custom { template: t, count: 30, shape: EmitterShape::SphereVolume { radius: 1.5 } }, 30)
        }

        PresetId::HolyRadiance => {
            let t = build_template(
                '+',
                (0.5, 1.0),
                (2.0, 4.0),
                (0.15, 0.3),
                std::f32::consts::TAU,
                1.0,
                2.0,
                MathFunction::GoldenSpiral { center: Vec3::ZERO, scale: 0.5, speed: 2.0 },
                gradient_3(
                    Vec4::new(1.0, 1.0, 0.8, 1.0),
                    Vec4::new(1.0, 0.9, 0.4, 0.8),
                    Vec4::new(0.8, 0.7, 0.2, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 24, shape: EmitterShape::Disk { radius: 1.0 } }, 24)
        }

        PresetId::DarkTendrils => {
            let t = build_template(
                '~',
                (0.6, 1.5),
                (1.5, 4.0),
                (0.15, 0.35),
                1.0,
                0.5,
                0.6,
                MathFunction::Lorenz { sigma: 10.0, rho: 28.0, beta: 2.667, scale: 0.1 },
                gradient_2(
                    Vec4::new(0.3, 0.0, 0.4, 0.9),
                    Vec4::new(0.1, 0.0, 0.15, 0.0),
                ),
                ParticleFlags::TRAIL_EMITTER,
            );
            (EmitterPreset::Custom { template: t, count: 16, shape: EmitterShape::Point }, 16)
        }

        PresetId::ArcaneRunesOrbit => {
            let t = build_template(
                '\u{2609}',
                (1.0, 2.0),
                (1.0, 2.0),
                (0.2, 0.4),
                0.5,
                0.2,
                1.2,
                MathFunction::Orbit { center: Vec3::ZERO, radius: 2.0, speed: 1.5, eccentricity: 0.3 },
                gradient_3(
                    Vec4::new(0.5, 0.2, 1.0, 1.0),
                    Vec4::new(0.7, 0.4, 1.0, 0.8),
                    Vec4::new(0.3, 0.1, 0.6, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 8, shape: EmitterShape::Sphere { radius: 0.1 } }, 8)
        }

        // ── AMBIENT ─────────────────────────────────────────────────────────

        PresetId::Campfire => {
            let t = build_template(
                '\u{00b7}',
                (0.5, 1.2),
                (1.0, 3.0),
                (0.1, 0.25),
                0.6,
                0.8,
                1.5,
                MathFunction::Breathing { rate: 3.0, depth: 0.6 },
                gradient_3(
                    Vec4::new(1.0, 0.9, 0.3, 1.0),
                    Vec4::new(1.0, 0.5, 0.1, 0.7),
                    Vec4::new(0.3, 0.1, 0.0, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 15, shape: EmitterShape::Cone { angle: 0.5, length: 0.2 } }, 15)
        }

        PresetId::TorchFlicker => {
            let t = build_template(
                '.',
                (0.3, 0.8),
                (1.5, 3.5),
                (0.08, 0.18),
                0.4,
                1.0,
                1.2,
                MathFunction::Sine { amplitude: 0.3, frequency: 6.0, phase: 0.0 },
                gradient_2(
                    Vec4::new(1.0, 0.7, 0.2, 0.9),
                    Vec4::new(0.8, 0.3, 0.0, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 8, shape: EmitterShape::Point }, 8)
        }

        PresetId::ShrineGlow => {
            let t = build_template(
                '\u{2727}',
                (1.0, 2.5),
                (0.3, 0.8),
                (0.12, 0.25),
                std::f32::consts::TAU,
                0.1,
                1.0,
                MathFunction::Breathing { rate: 1.5, depth: 0.8 },
                gradient_2(
                    Vec4::new(0.3, 0.8, 1.0, 0.8),
                    Vec4::new(0.1, 0.4, 0.7, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 12, shape: EmitterShape::Disk { radius: 0.8 } }, 12)
        }

        PresetId::PortalSwirl => {
            let t = build_template(
                '\u{2588}',
                (0.8, 1.8),
                (2.0, 4.0),
                (0.1, 0.2),
                0.3,
                0.5,
                1.5,
                MathFunction::Spiral { center: Vec3::ZERO, radius_rate: -1.5, speed: 4.0 },
                gradient_3(
                    Vec4::new(0.3, 0.1, 0.8, 1.0),
                    Vec4::new(0.6, 0.2, 1.0, 0.7),
                    Vec4::new(0.1, 0.0, 0.3, 0.0),
                ),
                ParticleFlags::AFFECTED_BY_FIELDS,
            );
            (EmitterPreset::Custom { template: t, count: 20, shape: EmitterShape::Sphere { radius: 1.0 } }, 20)
        }

        PresetId::TreasureSparkle => {
            let t = build_template(
                '\u{2726}',
                (0.4, 1.0),
                (0.5, 1.5),
                (0.1, 0.2),
                std::f32::consts::TAU,
                0.3,
                1.8,
                MathFunction::Sine { amplitude: 0.2, frequency: 3.0, phase: 0.0 },
                gradient_2(
                    Vec4::new(1.0, 0.9, 0.3, 1.0),
                    Vec4::new(1.0, 0.8, 0.0, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 10, shape: EmitterShape::Hemisphere { radius: 0.5 } }, 10)
        }

        PresetId::CorruptionSeep => {
            let t = build_template(
                '\u{2591}',
                (1.0, 3.0),
                (0.3, 1.0),
                (0.2, 0.5),
                1.5,
                0.1,
                0.5,
                MathFunction::Perlin { frequency: 0.8, octaves: 3, amplitude: 1.0 },
                gradient_3(
                    Vec4::new(0.4, 0.0, 0.5, 0.8),
                    Vec4::new(0.3, 0.0, 0.4, 0.5),
                    Vec4::new(0.1, 0.0, 0.15, 0.0),
                ),
                ParticleFlags::AFFECTED_BY_FIELDS,
            );
            (EmitterPreset::Custom { template: t, count: 14, shape: EmitterShape::Disk { radius: 1.2 } }, 14)
        }

        PresetId::VoidWisps => {
            let t = build_template(
                'o',
                (1.5, 3.0),
                (0.5, 1.5),
                (0.1, 0.18),
                std::f32::consts::TAU,
                0.2,
                0.8,
                MathFunction::Lissajous { a: 3.0, b: 2.0, delta: 0.5, scale: 1.0 },
                gradient_2(
                    Vec4::new(0.1, 0.0, 0.2, 0.6),
                    Vec4::new(0.05, 0.0, 0.1, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 6, shape: EmitterShape::SphereVolume { radius: 2.0 } }, 6)
        }

        // ── UI ──────────────────────────────────────────────────────────────

        PresetId::LevelUpBurst => {
            let t = build_template(
                '\u{2605}',
                (0.6, 1.2),
                (5.0, 10.0),
                (0.2, 0.45),
                std::f32::consts::TAU,
                1.2,
                2.5,
                MathFunction::GoldenSpiral { center: Vec3::ZERO, scale: 0.3, speed: 2.0 },
                gradient_3(
                    Vec4::new(1.0, 1.0, 0.5, 1.0),
                    Vec4::new(0.3, 0.8, 1.0, 0.8),
                    Vec4::new(0.1, 0.3, 0.6, 0.0),
                ),
                ParticleFlags::GRAVITY,
            );
            (EmitterPreset::Custom { template: t, count: 40, shape: EmitterShape::Sphere { radius: 0.2 } }, 40)
        }

        PresetId::AchievementUnlock => {
            let t = build_template(
                '\u{2726}',
                (0.5, 1.0),
                (4.0, 8.0),
                (0.15, 0.3),
                std::f32::consts::TAU,
                1.5,
                2.0,
                MathFunction::Breathing { rate: 4.0, depth: 0.8 },
                gradient_3(
                    Vec4::new(1.0, 0.85, 0.0, 1.0),
                    Vec4::new(1.0, 0.6, 0.0, 0.7),
                    Vec4::new(0.6, 0.3, 0.0, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 30, shape: EmitterShape::Sphere { radius: 0.3 } }, 30)
        }

        PresetId::ItemPickupSparkle => {
            let t = build_template(
                '\u{00b7}',
                (0.3, 0.6),
                (2.0, 4.0),
                (0.08, 0.15),
                std::f32::consts::TAU,
                2.0,
                1.2,
                MathFunction::Constant(0.8),
                gradient_2(
                    Vec4::new(0.8, 0.9, 1.0, 1.0),
                    Vec4::new(0.4, 0.5, 0.8, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 12, shape: EmitterShape::Hemisphere { radius: 0.3 } }, 12)
        }

        PresetId::GoldCoinScatter => {
            let t = build_template(
                '\u{25c9}',
                (0.5, 1.0),
                (3.0, 7.0),
                (0.15, 0.25),
                1.2,
                1.5,
                1.0,
                MathFunction::Constant(0.7),
                gradient_2(
                    Vec4::new(1.0, 0.88, 0.0, 1.0),
                    Vec4::new(0.7, 0.55, 0.0, 0.0),
                ),
                ParticleFlags::GRAVITY,
            );
            (EmitterPreset::Custom { template: t, count: 16, shape: EmitterShape::Hemisphere { radius: 0.2 } }, 16)
        }

        PresetId::HealthRestoreGlow => {
            let t = build_template(
                '+',
                (0.6, 1.2),
                (1.0, 2.5),
                (0.12, 0.25),
                std::f32::consts::TAU,
                0.5,
                1.5,
                MathFunction::Breathing { rate: 2.0, depth: 0.6 },
                gradient_3(
                    Vec4::new(0.2, 1.0, 0.3, 1.0),
                    Vec4::new(0.4, 1.0, 0.5, 0.7),
                    Vec4::new(0.1, 0.5, 0.15, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 16, shape: EmitterShape::Sphere { radius: 0.5 } }, 16)
        }

        PresetId::ManaRestoreShimmer => {
            let t = build_template(
                '\u{2726}',
                (0.5, 1.0),
                (1.0, 2.5),
                (0.1, 0.2),
                std::f32::consts::TAU,
                0.5,
                1.3,
                MathFunction::Sine { amplitude: 0.4, frequency: 3.0, phase: 0.0 },
                gradient_3(
                    Vec4::new(0.3, 0.4, 1.0, 1.0),
                    Vec4::new(0.5, 0.6, 1.0, 0.7),
                    Vec4::new(0.15, 0.2, 0.6, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 14, shape: EmitterShape::Sphere { radius: 0.4 } }, 14)
        }

        // ── EXTRA ───────────────────────────────────────────────────────────

        PresetId::BloodMist => {
            let t = build_template(
                '\u{00b7}',
                (0.8, 1.8),
                (1.0, 3.0),
                (0.1, 0.3),
                std::f32::consts::PI,
                0.4,
                0.3,
                MathFunction::Perlin { frequency: 2.0, octaves: 2, amplitude: 0.6 },
                gradient_2(
                    Vec4::new(0.7, 0.05, 0.05, 0.8),
                    Vec4::new(0.3, 0.0, 0.0, 0.0),
                ),
                ParticleFlags::GRAVITY,
            );
            (EmitterPreset::Custom { template: t, count: 20, shape: EmitterShape::Hemisphere { radius: 0.5 } }, 20)
        }

        PresetId::SoulRelease => {
            let t = build_template(
                'o',
                (1.0, 2.5),
                (0.8, 2.0),
                (0.12, 0.25),
                0.8,
                0.1,
                1.0,
                MathFunction::Spiral { center: Vec3::ZERO, radius_rate: 0.5, speed: 2.0 },
                gradient_3(
                    Vec4::new(0.6, 0.8, 1.0, 0.9),
                    Vec4::new(0.8, 0.9, 1.0, 0.5),
                    Vec4::new(1.0, 1.0, 1.0, 0.0),
                ),
                ParticleFlags(0),
            );
            (EmitterPreset::Custom { template: t, count: 10, shape: EmitterShape::Point }, 10)
        }

        PresetId::ChaosSurge => {
            let t = build_template(
                '#',
                (0.4, 1.0),
                (6.0, 14.0),
                (0.15, 0.4),
                std::f32::consts::TAU,
                0.8,
                2.0,
                MathFunction::StrangeAttractor {
                    attractor_type: AttractorType::Lorenz,
                    scale: 0.05,
                    strength: 1.0,
                },
                gradient_3(
                    Vec4::new(0.8, 0.2, 1.0, 1.0),
                    Vec4::new(1.0, 0.4, 0.2, 0.8),
                    Vec4::new(0.2, 0.0, 0.4, 0.0),
                ),
                ParticleFlags::AFFECTED_BY_FIELDS,
            );
            (EmitterPreset::Custom { template: t, count: 36, shape: EmitterShape::Sphere { radius: 0.5 } }, 36)
        }
    }
}

// ---------------------------------------------------------------------------
// Active instance tracking
// ---------------------------------------------------------------------------

/// A spawned preset instance with position and remaining lifetime.
struct PresetInstance {
    id: PresetId,
    position: Vec3,
    age: f32,
    lifetime: f32,
    /// Frame counter seed for glyph-based fallback rendering.
    frame_seed: u64,
}

// ---------------------------------------------------------------------------
// ParticlePresetManager
// ---------------------------------------------------------------------------

/// Manages spawning, updating, and rendering of particle presets.
///
/// Presets that map to engine `EmitterPreset` are dispatched directly via
/// `engine.emit_particles()`. The manager also tracks active instances for
/// glyph-based fallback rendering of continuous / looping effects.
pub struct ParticlePresetManager {
    instances: Vec<PresetInstance>,
    frame: u64,
}

impl ParticlePresetManager {
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(64),
            frame: 0,
        }
    }

    /// Spawn a preset by `PresetId` at a world position.
    /// For burst presets, this fires once into the engine particle system.
    /// For ambient/looping presets, an instance is tracked internally.
    pub fn spawn(&mut self, engine: &mut ProofEngine, id: PresetId, position: Vec3) {
        let (preset, _count) = build_preset(id);
        engine.emit_particles(preset, position);

        // For ambient presets, also track for glyph rendering over time
        let lifetime = match id {
            PresetId::Campfire | PresetId::TorchFlicker | PresetId::ShrineGlow
            | PresetId::PortalSwirl | PresetId::CorruptionSeep | PresetId::VoidWisps => 5.0,
            PresetId::PoisonCloud => 3.0,
            PresetId::ArcaneRunesOrbit => 4.0,
            _ => 0.0, // burst-only, no tracking
        };

        if lifetime > 0.0 {
            self.instances.push(PresetInstance {
                id,
                position,
                age: 0.0,
                lifetime,
                frame_seed: self.frame,
            });
        }
    }

    /// Spawn a preset by string name. Returns false if name not found.
    pub fn spawn_by_name(&mut self, engine: &mut ProofEngine, name: &str, position: Vec3) -> bool {
        for &pid in PresetId::all() {
            if pid.name() == name {
                self.spawn(engine, pid, position);
                return true;
            }
        }
        false
    }

    /// Update all tracked instances, removing expired ones.
    pub fn update(&mut self, dt: f32) {
        self.frame += 1;
        self.instances.retain_mut(|inst| {
            inst.age += dt;
            inst.age < inst.lifetime
        });
    }

    /// Render glyph-based ambient effects for all tracked instances.
    pub fn render(&self, engine: &mut ProofEngine) {
        for inst in &self.instances {
            render_ambient_glyphs(engine, inst, self.frame);
        }
    }

    /// Convenience: update + render in one call.
    pub fn update_and_render(&mut self, engine: &mut ProofEngine, dt: f32) {
        self.update(dt);
        self.render(engine);
    }

    /// Number of active tracked instances.
    pub fn active_count(&self) -> usize {
        self.instances.len()
    }

    /// Clear all tracked instances.
    pub fn clear(&mut self) {
        self.instances.clear();
    }
}

// ---------------------------------------------------------------------------
// Glyph-based ambient rendering (supplements engine particles)
// ---------------------------------------------------------------------------

fn render_ambient_glyphs(engine: &mut ProofEngine, inst: &PresetInstance, frame: u64) {
    let t = inst.age;
    let pos = inst.position;
    let local_frame = frame.wrapping_sub(inst.frame_seed);

    match inst.id {
        PresetId::Campfire | PresetId::TorchFlicker => {
            // Flickering glow glyphs around position
            let count = if inst.id == PresetId::Campfire { 8 } else { 4 };
            for i in 0..count {
                let seed = (local_frame as u32).wrapping_mul(31).wrapping_add(i as u32);
                let ox = hash_range(seed, -0.6, 0.6);
                let oy = hash_range(seed.wrapping_add(1), 0.0, 1.5);
                let flicker = ((local_frame as f32 * 0.15 + i as f32 * 1.3).sin() * 0.3 + 0.7).max(0.0);
                let life_fade = 1.0 - (t / inst.lifetime);

                engine.spawn_glyph(Glyph {
                    character: if i % 2 == 0 { '\u{00b7}' } else { ',' },
                    position: pos + Vec3::new(ox, oy, 0.0),
                    color: Vec4::new(1.0, 0.6 * flicker, 0.15, 0.6 * life_fade * flicker),
                    scale: Vec2::splat(0.15),
                    emission: flicker * 1.2,
                    glow_color: Vec3::new(1.0, 0.5, 0.1),
                    glow_radius: 0.5,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        PresetId::ShrineGlow => {
            let count = 6;
            for i in 0..count {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU + t * 0.5;
                let r = 0.8;
                let ox = angle.cos() * r;
                let oy = angle.sin() * r * 0.5 + 0.5;
                let pulse = ((t * 2.0 + i as f32).sin() * 0.3 + 0.7).max(0.0);
                let life_fade = 1.0 - (t / inst.lifetime);

                engine.spawn_glyph(Glyph {
                    character: '\u{2727}',
                    position: pos + Vec3::new(ox, oy, 0.0),
                    color: Vec4::new(0.3, 0.8, 1.0, 0.5 * pulse * life_fade),
                    scale: Vec2::splat(0.18),
                    emission: pulse,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        PresetId::PortalSwirl => {
            let count = 10;
            for i in 0..count {
                let angle = (i as f32 / count as f32) * std::f32::consts::TAU + t * 3.0;
                let r = 1.0 + (t * 0.5).sin() * 0.3;
                let ox = angle.cos() * r;
                let oy = angle.sin() * r * 0.6;
                let life_fade = 1.0 - (t / inst.lifetime);

                engine.spawn_glyph(Glyph {
                    character: '\u{2588}',
                    position: pos + Vec3::new(ox, oy, 0.0),
                    color: Vec4::new(0.4, 0.15, 0.8, 0.5 * life_fade),
                    scale: Vec2::splat(0.12),
                    emission: 1.0,
                    glow_color: Vec3::new(0.5, 0.2, 0.9),
                    glow_radius: 0.6,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        PresetId::CorruptionSeep => {
            let count = 8;
            for i in 0..count {
                let seed = (local_frame as u32 / 4).wrapping_add(i as u32 * 71);
                let ox = hash_range(seed, -1.2, 1.2);
                let oy = hash_range(seed.wrapping_add(1), -0.3, 0.3);
                let rise = (local_frame as f32 * 0.01 + hash_f32(seed.wrapping_add(2)) * 5.0) % 2.0;
                let life_fade = 1.0 - (t / inst.lifetime);

                engine.spawn_glyph(Glyph {
                    character: '\u{2591}',
                    position: pos + Vec3::new(ox, oy + rise, 0.0),
                    color: Vec4::new(0.35, 0.0, 0.45, 0.4 * life_fade),
                    scale: Vec2::splat(0.25),
                    emission: 0.3,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }

        PresetId::VoidWisps => {
            let count = 3;
            for i in 0..count {
                let phase = i as f32 * 2.1;
                let ox = (t * 0.8 + phase).sin() * 2.0;
                let oy = (t * 0.6 + phase * 1.3).cos() * 1.2;
                let pulse = ((t * 1.5 + phase).sin() * 0.4 + 0.6).max(0.0);
                let life_fade = 1.0 - (t / inst.lifetime);

                engine.spawn_glyph(Glyph {
                    character: 'o',
                    position: pos + Vec3::new(ox, oy, 0.0),
                    color: Vec4::new(0.1, 0.0, 0.2, 0.4 * pulse * life_fade),
                    scale: Vec2::splat(0.12),
                    emission: 0.5 * pulse,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }

        PresetId::PoisonCloud => {
            let count = 10;
            for i in 0..count {
                let seed = (local_frame as u32 / 3).wrapping_add(i as u32 * 47);
                let ox = hash_range(seed, -1.5, 1.5);
                let oy = hash_range(seed.wrapping_add(1), -1.0, 1.0);
                let swell = ((t * 1.2 + i as f32 * 0.7).sin() * 0.3 + 0.7).max(0.0);
                let life_fade = 1.0 - (t / inst.lifetime);

                engine.spawn_glyph(Glyph {
                    character: 'o',
                    position: pos + Vec3::new(ox, oy, 0.0),
                    color: Vec4::new(0.2, 0.7, 0.15, 0.35 * swell * life_fade),
                    scale: Vec2::splat(0.2 + swell * 0.1),
                    emission: 0.3,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }

        PresetId::ArcaneRunesOrbit => {
            let runes = ['\u{2609}', '\u{2605}', '\u{2726}', '\u{2727}'];
            for (i, &ch) in runes.iter().enumerate() {
                let angle = (i as f32 / runes.len() as f32) * std::f32::consts::TAU + t * 1.5;
                let r = 1.8;
                let ox = angle.cos() * r;
                let oy = angle.sin() * r * 0.5;
                let life_fade = 1.0 - (t / inst.lifetime);

                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: pos + Vec3::new(ox, oy, 0.0),
                    color: Vec4::new(0.6, 0.3, 1.0, 0.7 * life_fade),
                    scale: Vec2::splat(0.25),
                    emission: 1.0,
                    glow_color: Vec3::new(0.5, 0.2, 0.8),
                    glow_radius: 0.8,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        _ => {} // Burst-only presets have no ambient glyph render
    }
}
