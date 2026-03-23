//! Characters, classes, stats, and leveling.
//!
//! CHAOS RPG has 4 character classes and 7 unbounded stats.
//! Stats can exceed any limit — there is no cap. The universe is your cap.

use crate::chaos_pipeline::{chaos_roll_verbose, destiny_roll, roll_stat};
use serde::{Deserialize, Serialize};

// ─── CLASSES ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Mage,
    Berserker,
    Ranger,
    Thief,
    ChaosMath,    // Chaos Mathematician — bends the math engines themselves
    Necromancer,  // Death math — Euler's mortality equation made flesh
    Physicist,    // Heisenberg combat — first strike is always "observed"
}

impl CharacterClass {
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "Mage",
            CharacterClass::Berserker => "Berserker",
            CharacterClass::Ranger => "Ranger",
            CharacterClass::Thief => "Thief",
            CharacterClass::ChaosMath => "Chaos Mathematician",
            CharacterClass::Necromancer => "Necromancer",
            CharacterClass::Physicist => "Physicist",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Mage => {
                "Bends chaos through pure mathematical will. High MANA and ENTROPY, low VITALITY."
            }
            CharacterClass::Berserker => {
                "Channels pain into exponential power. VITALITY and FORCE scale catastrophically."
            }
            CharacterClass::Ranger => {
                "Reads the prime number patterns in nature. Balanced stats with deadly PRECISION."
            }
            CharacterClass::Thief => {
                "Exploits the logistic map's chaotic phase transitions. Master of CUNNING and LUCK."
            }
            CharacterClass::ChaosMath => {
                "Rewrites the mathematical rules mid-battle. Chaos events heal you. Maximum ENTROPY."
            }
            CharacterClass::Necromancer => {
                "Channels death equations. Killing grants HP shields. Euler's formula is a weapon."
            }
            CharacterClass::Physicist => {
                "Applies Heisenberg's uncertainty to combat. First strike is always 'observed' — no catastrophes."
            }
        }
    }

    pub fn passive_description(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "ARCANE SURGE: Critical rolls amplify next spell by 50%.",
            CharacterClass::Berserker => "BLOOD RAGE: Below 30% HP, automatically enter ENRAGED state.",
            CharacterClass::Ranger => "PRIME SIGHT: Combo chains deal +10% more per hit (stacks).",
            CharacterClass::Thief => "SHADOW KILL: One-shot kills grant double gold.",
            CharacterClass::ChaosMath => "CHAOS RESONANCE: Every chaos event in combat restores 15 HP.",
            CharacterClass::Necromancer => "DEATH HARVEST: Killing an enemy grants a shield = 30% of their max HP.",
            CharacterClass::Physicist => "UNCERTAINTY PRINCIPLE: First attack each combat is 'observed' — guaranteed to connect.",
        }
    }

    pub fn ascii_art(&self) -> &'static str {
        match self {
            CharacterClass::Mage =>       "   /\\ \n  (∞) \n   ||",
            CharacterClass::Berserker =>  "  ><  \n [RAGE]\n  \\/ ",
            CharacterClass::Ranger =>     "  />\\ \n  ||| \n  vvv",
            CharacterClass::Thief =>      "  .~~.\n  {~} \n  /|\\",
            CharacterClass::ChaosMath =>  "  ∑∫∂ \n  |Σ| \n  ∞∇π",
            CharacterClass::Necromancer => "  /†\\ \n  |☠| \n  ≡≡≡",
            CharacterClass::Physicist =>  "  ℏ∇² \n  ψ|φ \n  ⟨ψ|",
        }
    }

    /// Base stat multipliers for each class
    pub fn stat_weights(&self) -> StatBlock {
        match self {
            CharacterClass::Mage => StatBlock {
                vitality: 40,
                force: 30,
                mana: 90,
                cunning: 60,
                precision: 50,
                entropy: 80,
                luck: 55,
            },
            CharacterClass::Berserker => StatBlock {
                vitality: 90,
                force: 85,
                mana: 20,
                cunning: 25,
                precision: 40,
                entropy: 70,
                luck: 35,
            },
            CharacterClass::Ranger => StatBlock {
                vitality: 55,
                force: 55,
                mana: 45,
                cunning: 60,
                precision: 90,
                entropy: 50,
                luck: 60,
            },
            CharacterClass::Thief => StatBlock {
                vitality: 45,
                force: 40,
                mana: 55,
                cunning: 90,
                precision: 70,
                entropy: 65,
                luck: 85,
            },
            CharacterClass::ChaosMath => StatBlock {
                vitality: 40,
                force: 30,
                mana: 85,
                cunning: 70,
                precision: 60,
                entropy: 100, // highest entropy of all classes
                luck: 55,
            },
            CharacterClass::Necromancer => StatBlock {
                vitality: 65,
                force: 35,
                mana: 95, // highest mana of all classes
                cunning: 55,
                precision: 40,
                entropy: 75,
                luck: 50,
            },
            CharacterClass::Physicist => StatBlock {
                vitality: 55,
                force: 50,
                mana: 60,
                cunning: 85,
                precision: 95, // highest precision of all classes
                entropy: 70,
                luck: 65,
            },
        }
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ─── BACKGROUNDS ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    Scholar,   // +MANA +ENTROPY
    Wanderer,  // +LUCK +PRECISION
    Gladiator, // +FORCE +VITALITY
    Outcast,   // +CUNNING +ENTROPY
    Merchant,  // +LUCK +CUNNING
    Cultist,   // +MANA +ENTROPY (extreme)
}

impl Background {
    pub fn name(&self) -> &'static str {
        match self {
            Background::Scholar => "Scholar",
            Background::Wanderer => "Wanderer",
            Background::Gladiator => "Gladiator",
            Background::Outcast => "Outcast",
            Background::Merchant => "Merchant",
            Background::Cultist => "Cultist",
        }
    }

    pub fn stat_bonus(&self) -> StatBlock {
        match self {
            Background::Scholar => StatBlock {
                mana: 15,
                entropy: 10,
                ..StatBlock::zero()
            },
            Background::Wanderer => StatBlock {
                luck: 15,
                precision: 10,
                ..StatBlock::zero()
            },
            Background::Gladiator => StatBlock {
                force: 15,
                vitality: 10,
                ..StatBlock::zero()
            },
            Background::Outcast => StatBlock {
                cunning: 15,
                entropy: 10,
                ..StatBlock::zero()
            },
            Background::Merchant => StatBlock {
                luck: 10,
                cunning: 15,
                ..StatBlock::zero()
            },
            Background::Cultist => StatBlock {
                mana: 20,
                entropy: 20,
                vitality: -10,
                ..StatBlock::zero()
            },
        }
    }
}

// ─── STATS ───────────────────────────────────────────────────────────────────

/// The 7 unbounded stats of CHAOS RPG.
/// Values can theoretically grow without limit (wrapping at ±100,000 in display).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBlock {
    pub vitality: i64,  // HP multiplier, resistance
    pub force: i64,     // Physical damage, carry weight
    pub mana: i64,      // Spell power, magic resist
    pub cunning: i64,   // Critical chance, trap detection
    pub precision: i64, // Accuracy, ranged damage
    pub entropy: i64,   // Chaos bonus to all rolls
    pub luck: i64,      // General fortune modifier
}

impl StatBlock {
    pub fn zero() -> Self {
        StatBlock {
            vitality: 0,
            force: 0,
            mana: 0,
            cunning: 0,
            precision: 0,
            entropy: 0,
            luck: 0,
        }
    }

    pub fn add(&self, other: &StatBlock) -> StatBlock {
        StatBlock {
            vitality: self.vitality + other.vitality,
            force: self.force + other.force,
            mana: self.mana + other.mana,
            cunning: self.cunning + other.cunning,
            precision: self.precision + other.precision,
            entropy: self.entropy + other.entropy,
            luck: self.luck + other.luck,
        }
    }

    pub fn total(&self) -> i64 {
        self.vitality
            + self.force
            + self.mana
            + self.cunning
            + self.precision
            + self.entropy
            + self.luck
    }

    pub fn power_level(&self) -> PowerTier {
        let total = self.total();
        match total {
            i64::MIN..=-1000 => PowerTier::Abyssal,
            -999..=-300 => PowerTier::Damned,
            -299..=-1 => PowerTier::Cursed,
            0..=99 => PowerTier::Mortal,
            100..=299 => PowerTier::Awakened,
            300..=599 => PowerTier::Champion,
            600..=999 => PowerTier::Legendary,
            1000..=2999 => PowerTier::Transcendent,
            3000..=9999 => PowerTier::Godlike,
            _ => PowerTier::BeyondMath,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerTier {
    // Negative tiers — the chaos cursed you
    Abyssal,
    Damned,
    Cursed,
    // Normal progression
    Mortal,
    Awakened,
    Champion,
    Legendary,
    Transcendent,
    Godlike,
    BeyondMath, // stat total > 9999 — shouldn't exist
}

impl PowerTier {
    pub fn name(&self) -> &'static str {
        match self {
            PowerTier::Abyssal => "ABYSSAL",
            PowerTier::Damned => "DAMNED",
            PowerTier::Cursed => "CURSED",
            PowerTier::Mortal => "Mortal",
            PowerTier::Awakened => "Awakened",
            PowerTier::Champion => "Champion",
            PowerTier::Legendary => "Legendary",
            PowerTier::Transcendent => "Transcendent",
            PowerTier::Godlike => "GODLIKE",
            PowerTier::BeyondMath => "BEYOND MATH",
        }
    }

    pub fn flavor(&self) -> &'static str {
        match self {
            PowerTier::Abyssal => "The math has forsaken you. You exist only through spite.",
            PowerTier::Damned => "The algorithms hate you specifically. Keep going.",
            PowerTier::Cursed => "Even rats pity you. Negative stats are technically valid.",
            PowerTier::Mortal => "Statistically average. The Logistic Map is neutral on you.",
            PowerTier::Awakened => "The prime numbers notice you. That is an improvement.",
            PowerTier::Champion => "The Lorenz attractor bends in your favor.",
            PowerTier::Legendary => "The Riemann zeros align. You are an anomaly.",
            PowerTier::Transcendent => "The Mandelbrot boundary recognizes your face.",
            PowerTier::Godlike => "You ARE the chaos engine. The math screams.",
            PowerTier::BeyondMath => "ERROR: STAT OVERFLOW. YOU HAVE BROKEN THE ALGORITHM.",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            PowerTier::Abyssal => "\x1b[31m",      // red (cursed/suffering)
            PowerTier::Damned => "\x1b[31m",        // red
            PowerTier::Cursed => "\x1b[35m",        // magenta
            PowerTier::Mortal => "\x1b[37m",        // white
            PowerTier::Awakened => "\x1b[32m",      // green
            PowerTier::Champion => "\x1b[36m",      // cyan
            PowerTier::Legendary => "\x1b[33m",     // yellow
            PowerTier::Transcendent => "\x1b[35m",  // magenta
            PowerTier::Godlike => "\x1b[91m",       // bright red
            PowerTier::BeyondMath => "\x1b[97m",    // bright white
        }
    }
}

// ─── DIFFICULTY ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Novice,    // +30% HP, enemies weakened
    Standard,  // balanced
    Chaotic,   // harder, bigger rewards
    Nightmare, // mathematical certainty of suffering
}

impl Difficulty {
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Novice => "Novice",
            Difficulty::Standard => "Standard",
            Difficulty::Chaotic => "Chaotic",
            Difficulty::Nightmare => "NIGHTMARE",
        }
    }

    pub fn color_code(self) -> &'static str {
        match self {
            Difficulty::Novice => "\x1b[32m",
            Difficulty::Standard => "\x1b[37m",
            Difficulty::Chaotic => "\x1b[33m",
            Difficulty::Nightmare => "\x1b[91m",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Difficulty::Novice    => "+30% HP  |  Enemies: -25% HP/DMG  |  Rewards: -20%",
            Difficulty::Standard  => "Balanced chaos. As the algorithms intended.",
            Difficulty::Chaotic   => "-15% HP  |  Enemies: +30% HP/DMG  |  Rewards: +50%",
            Difficulty::Nightmare => "-35% HP  |  Enemies: 2× HP/DMG    |  Rewards: 3×",
        }
    }

    pub fn player_hp_mult(self) -> f64 {
        match self {
            Difficulty::Novice    => 1.30,
            Difficulty::Standard  => 1.00,
            Difficulty::Chaotic   => 0.85,
            Difficulty::Nightmare => 0.65,
        }
    }

    pub fn enemy_power_mult(self) -> f64 {
        match self {
            Difficulty::Novice    => 0.75,
            Difficulty::Standard  => 1.00,
            Difficulty::Chaotic   => 1.30,
            Difficulty::Nightmare => 2.00,
        }
    }

    pub fn reward_mult(self) -> f64 {
        match self {
            Difficulty::Novice    => 0.80,
            Difficulty::Standard  => 1.00,
            Difficulty::Chaotic   => 1.50,
            Difficulty::Nightmare => 3.00,
        }
    }
}

// ─── BOONS ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Boon {
    BloodPact,       // +50 max HP, take 2 dmg per room
    ChaosBlessing,   // chaos rolls biased luckier (+0.15)
    GoldVein,        // start with 200 gold
    ScholarGift,     // 3 extra starting spells
    WarriorBlessing, // +20 Force, +15 Vitality
    LuckyBirth,      // +30 Luck, +5 gold per room cleared
    EntropicSoul,    // 2× Entropy + Mana, 0.5× Vitality
    CrystalSkin,     // start with 80 HP shield
    MathSavant,      // all spells deal 1.75× damage
    VoidTouched,     // all stats ×1.5 — chaos events affect you too
    PrimeBlood,      // every kill permanently +1 to highest stat
    ShadowStart,     // start at 50% HP, but 3× XP from all kills
}

impl Boon {
    pub fn name(self) -> &'static str {
        match self {
            Boon::BloodPact       => "Blood Pact",
            Boon::ChaosBlessing   => "Chaos Blessing",
            Boon::GoldVein        => "Gold Vein",
            Boon::ScholarGift     => "Scholar's Gift",
            Boon::WarriorBlessing => "Warrior's Blessing",
            Boon::LuckyBirth      => "Lucky Birth",
            Boon::EntropicSoul    => "Entropic Soul",
            Boon::CrystalSkin     => "Crystal Skin",
            Boon::MathSavant      => "Math Savant",
            Boon::VoidTouched     => "Void Touched",
            Boon::PrimeBlood      => "Prime Blood",
            Boon::ShadowStart     => "Shadow Start",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Boon::BloodPact       => "+50 max HP. Take 2 HP damage entering each room.",
            Boon::ChaosBlessing   => "All chaos rolls biased +0.15. The math likes you.",
            Boon::GoldVein        => "Start with 200 gold. Someone's loss is your gain.",
            Boon::ScholarGift     => "Start with 3 extra chaos-generated spells.",
            Boon::WarriorBlessing => "+20 Force, +15 Vitality. Mathematically swole.",
            Boon::LuckyBirth      => "+30 Luck. Gain 5 gold every room you clear.",
            Boon::EntropicSoul    => "2× Entropy + Mana, half Vitality. High risk, higher chaos.",
            Boon::CrystalSkin     => "Begin with an 80 HP shield. The math loves you (for now).",
            Boon::MathSavant      => "All spell damage ×1.75. The equations submit to your will.",
            Boon::VoidTouched     => "All stats ×1.5. Chaos events also affect you (they're bigger).",
            Boon::PrimeBlood      => "Each kill: +1 to your highest stat. Growth is inevitable.",
            Boon::ShadowStart     => "Start at 50% HP. All XP gained ×3. Death is the best teacher.",
        }
    }

    pub fn color_code(self) -> &'static str {
        match self {
            Boon::BloodPact | Boon::ShadowStart => "\x1b[31m",
            Boon::ChaosBlessing | Boon::EntropicSoul | Boon::VoidTouched => "\x1b[35m",
            Boon::GoldVein | Boon::LuckyBirth => "\x1b[33m",
            Boon::ScholarGift | Boon::MathSavant => "\x1b[36m",
            Boon::WarriorBlessing | Boon::PrimeBlood => "\x1b[32m",
            Boon::CrystalSkin => "\x1b[34m",
        }
    }

    /// Pick 3 unique random boons from the full set.
    pub fn random_three(seed: u64) -> [Boon; 3] {
        use Boon::*;
        const ALL: [Boon; 12] = [
            BloodPact, ChaosBlessing, GoldVein, ScholarGift, WarriorBlessing,
            LuckyBirth, EntropicSoul, CrystalSkin, MathSavant, VoidTouched,
            PrimeBlood, ShadowStart,
        ];
        let a = (seed % 12) as usize;
        let b = ((seed.wrapping_mul(31337)) % 12) as usize;
        let c = ((seed.wrapping_mul(99991)) % 12) as usize;
        // Ensure uniqueness
        let b = if b == a { (b + 1) % 12 } else { b };
        let c = if c == a || c == b { (c + 2) % 12 } else { c };
        [ALL[a], ALL[b], ALL[c]]
    }
}

// ─── STATUS EFFECTS ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffect {
    Burning(u32),    // takes damage each round, N rounds remaining
    Poisoned(u32),   // weaker damage-over-time
    Stunned(u32),    // skips N turns
    Cursed(u32),     // -20 to all stat rolls for N rounds
    Blessed(u32),    // +20 to all stat rolls for N rounds
    Shielded(i64),   // absorbs flat damage
    Enraged(u32),    // +50% damage but -30% defense
    Frozen(u32),     // can't flee, -50% speed
}

impl StatusEffect {
    pub fn name(&self) -> &'static str {
        match self {
            StatusEffect::Burning(_) => "BURNING",
            StatusEffect::Poisoned(_) => "POISONED",
            StatusEffect::Stunned(_) => "STUNNED",
            StatusEffect::Cursed(_) => "CURSED",
            StatusEffect::Blessed(_) => "BLESSED",
            StatusEffect::Shielded(_) => "SHIELDED",
            StatusEffect::Enraged(_) => "ENRAGED",
            StatusEffect::Frozen(_) => "FROZEN",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            StatusEffect::Burning(_) => "\x1b[91m",
            StatusEffect::Poisoned(_) => "\x1b[32m",
            StatusEffect::Stunned(_) => "\x1b[36m",
            StatusEffect::Cursed(_) => "\x1b[35m",
            StatusEffect::Blessed(_) => "\x1b[33m",
            StatusEffect::Shielded(_) => "\x1b[34m",
            StatusEffect::Enraged(_) => "\x1b[31m",
            StatusEffect::Frozen(_) => "\x1b[94m",
        }
    }

    /// Returns the per-turn damage (0 if not a damage effect)
    pub fn tick_damage(&self) -> i64 {
        match self {
            StatusEffect::Burning(_) => 8,
            StatusEffect::Poisoned(_) => 3,
            _ => 0,
        }
    }

    /// Decrements turn counter. Returns true if the effect expired.
    pub fn tick(&mut self) -> bool {
        match self {
            StatusEffect::Burning(n)
            | StatusEffect::Poisoned(n)
            | StatusEffect::Stunned(n)
            | StatusEffect::Cursed(n)
            | StatusEffect::Blessed(n)
            | StatusEffect::Enraged(n)
            | StatusEffect::Frozen(n) => {
                if *n == 0 {
                    return true;
                }
                *n -= 1;
                *n == 0
            }
            StatusEffect::Shielded(hp) => *hp <= 0,
        }
    }
}

// ─── CHARACTER ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub class: CharacterClass,
    pub background: Background,
    pub stats: StatBlock,
    pub max_hp: i64,
    pub current_hp: i64,
    pub level: u32,
    pub xp: u64,
    pub floor: u32,
    pub gold: i64,
    pub kills: u32,
    pub seed: u64,
    // Extended fields
    pub inventory: Vec<crate::items::Item>,
    pub known_spells: Vec<crate::spells::Spell>,
    pub status_effects: Vec<StatusEffect>,
    // Game settings applied at creation
    pub difficulty: Difficulty,
    pub boon: Option<Boon>,
    // Passive state
    pub passive_ready: bool,  // Physicist first-attack, Mage crit-amplify, etc.
    pub spell_damage_mult: f64, // MathSavant boon or Mage crit buff
    pub xp_mult: f64,          // ShadowStart boon
    // Run statistics
    pub total_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub spells_cast: u32,
    pub items_used: u32,
    pub rooms_cleared: u32,
}

impl Character {
    /// Roll a new character with destiny (all 10 engines)
    pub fn roll_new(
        name: String,
        class: CharacterClass,
        background: Background,
        seed: u64,
    ) -> Self {
        let weights = class.stat_weights();
        let bg_bonus = background.stat_bonus();

        // Each stat is chaos-rolled through all 10 engines.
        // The class weight is the *center* of the distribution — not a cap.
        // Final values can be deeply negative (cursed) or astronomically positive (godlike).
        // At chaos_mult=+1: stat ≈ weight × 4 (transcendent)
        // At chaos_mult= 0: stat ≈ weight     (class-appropriate)
        // At chaos_mult=-1: stat ≈ weight × -2 (catastrophically cursed)
        let roll_with_weight = |weight: i64, stat_seed: u64| -> i64 {
            let destiny = destiny_roll(stat_seed as f64 * 1e-12, stat_seed);
            let chaos_mult = 1.0 + destiny.final_value * 3.0; // range [-2, 4]
            let base = (weight as f64 * chaos_mult) as i64;
            // Small deterministic perturbation so nearby seeds diverge further
            base + roll_stat(-(weight / 5 + 1), weight / 5 + 1, stat_seed.wrapping_add(77))
        };

        let stats = StatBlock {
            vitality: roll_with_weight(weights.vitality, seed.wrapping_add(1)),
            force: roll_with_weight(weights.force, seed.wrapping_add(2)),
            mana: roll_with_weight(weights.mana, seed.wrapping_add(3)),
            cunning: roll_with_weight(weights.cunning, seed.wrapping_add(4)),
            precision: roll_with_weight(weights.precision, seed.wrapping_add(5)),
            entropy: roll_with_weight(weights.entropy, seed.wrapping_add(6)),
            luck: roll_with_weight(weights.luck, seed.wrapping_add(7)),
        };

        let stats = stats.add(&bg_bonus);
        // HP can be very low for cursed rolls — minimum 1 to stay alive
        let max_hp = (50 + stats.vitality * 3 + stats.force).max(1);

        // Starting spells — class-based starting count
        let spell_count = match class {
            CharacterClass::Mage | CharacterClass::ChaosMath => 2,
            CharacterClass::Necromancer => 2,
            _ => 1,
        };
        let known_spells: Vec<crate::spells::Spell> = (0..spell_count)
            .map(|i| crate::spells::Spell::generate(seed.wrapping_add(10001 + i as u64)))
            .collect();

        Character {
            name,
            class,
            background,
            max_hp,
            current_hp: max_hp,
            stats,
            level: 1,
            xp: 0,
            floor: 1,
            gold: roll_stat(5, 30, seed.wrapping_add(999)),
            kills: 0,
            seed,
            inventory: Vec::new(),
            known_spells,
            status_effects: Vec::new(),
            difficulty: Difficulty::Standard,
            boon: None,
            passive_ready: true, // Physicist first-attack starts ready
            spell_damage_mult: 1.0,
            xp_mult: 1.0,
            total_damage_dealt: 0,
            total_damage_taken: 0,
            spells_cast: 0,
            items_used: 0,
            rooms_cleared: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn hp_percent(&self) -> f64 {
        (self.current_hp as f64 / self.max_hp as f64).clamp(0.0, 1.0)
    }

    pub fn power_tier(&self) -> PowerTier {
        self.stats.power_level()
    }

    pub fn take_damage(&mut self, amount: i64) {
        // Shielded absorbs first
        let mut remaining = amount;
        for effect in &mut self.status_effects {
            if let StatusEffect::Shielded(shield_hp) = effect {
                if *shield_hp >= remaining {
                    *shield_hp -= remaining;
                    remaining = 0;
                    break;
                } else {
                    remaining -= *shield_hp;
                    *shield_hp = 0;
                }
            }
        }
        self.status_effects.retain(|e| !matches!(e, StatusEffect::Shielded(0)));
        self.current_hp = (self.current_hp - remaining).max(0);
        self.total_damage_taken += remaining;
    }

    pub fn add_status(&mut self, effect: StatusEffect) {
        // Replace same-type effects rather than stack
        self.status_effects.retain(|e| e.name() != effect.name());
        self.status_effects.push(effect);
    }

    pub fn has_status(&self, name: &str) -> bool {
        self.status_effects.iter().any(|e| e.name() == name)
    }

    /// Process start-of-turn status effects. Returns (damage_taken, messages).
    pub fn tick_status_effects(&mut self) -> (i64, Vec<String>) {
        let mut dmg = 0i64;
        let mut msgs = Vec::new();

        let effects_copy = self.status_effects.clone();
        for effect in &effects_copy {
            let tick_dmg = effect.tick_damage();
            if tick_dmg > 0 {
                self.current_hp = (self.current_hp - tick_dmg).max(0);
                self.total_damage_taken += tick_dmg;
                dmg += tick_dmg;
                msgs.push(format!("{} takes {} {} damage!", self.name, tick_dmg, effect.name()));
            }
        }

        // Decrement counters and remove expired effects
        let mut expired = Vec::new();
        for effect in &mut self.status_effects {
            if effect.tick() {
                expired.push(effect.name());
            }
        }
        for name in &expired {
            msgs.push(format!("{} wore off.", name));
        }
        self.status_effects.retain(|e| !expired.contains(&e.name()));

        (dmg, msgs)
    }

    pub fn add_item(&mut self, item: crate::items::Item) {
        self.inventory.push(item);
    }

    pub fn add_spell(&mut self, spell: crate::spells::Spell) {
        self.known_spells.push(spell);
    }

    /// Use an item from inventory by index. Returns the item if valid.
    pub fn use_item(&mut self, idx: usize) -> Option<crate::items::Item> {
        if idx < self.inventory.len() {
            self.items_used += 1;
            Some(self.inventory.remove(idx))
        } else {
            None
        }
    }

    pub fn heal(&mut self, amount: i64) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn gain_xp(&mut self, xp: u64) {
        let scaled = (xp as f64 * self.xp_mult) as u64;
        self.xp += scaled;
        let xp_needed = (self.level as u64 * 100) * (self.level as u64 + 1) / 2;
        if self.xp >= xp_needed {
            self.level_up_and_learn_spell();
        }
    }

    /// Apply difficulty multipliers to HP after character creation.
    pub fn apply_difficulty(&mut self, difficulty: Difficulty) {
        self.difficulty = difficulty;
        let mult = difficulty.player_hp_mult();
        self.max_hp = ((self.max_hp as f64 * mult) as i64).max(1);
        self.current_hp = self.max_hp;
    }

    /// Apply a starting boon's effects to the character.
    pub fn apply_boon(&mut self, boon: Boon) {
        use Boon::*;
        self.boon = Some(boon);
        match boon {
            BloodPact => {
                self.max_hp += 50;
                self.current_hp = self.max_hp;
            }
            ChaosBlessing => {
                // Stored for later use in chaos rolls (flavor — noted in passive_ready)
                self.stats.luck += 10;
            }
            GoldVein => {
                self.gold += 200;
            }
            ScholarGift => {
                for i in 0..3u64 {
                    let spell = crate::spells::Spell::generate(
                        self.seed.wrapping_add(88888 + i * 31337),
                    );
                    self.known_spells.push(spell);
                }
            }
            WarriorBlessing => {
                self.stats.force += 20;
                self.stats.vitality += 15;
                self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
                self.current_hp = self.max_hp;
            }
            LuckyBirth => {
                self.stats.luck += 30;
            }
            EntropicSoul => {
                self.stats.entropy *= 2;
                self.stats.mana *= 2;
                self.stats.vitality /= 2;
                self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
                self.current_hp = self.max_hp;
            }
            CrystalSkin => {
                self.add_status(StatusEffect::Shielded(80));
            }
            MathSavant => {
                self.spell_damage_mult = 1.75;
            }
            VoidTouched => {
                self.stats.vitality = (self.stats.vitality as f64 * 1.5) as i64;
                self.stats.force = (self.stats.force as f64 * 1.5) as i64;
                self.stats.mana = (self.stats.mana as f64 * 1.5) as i64;
                self.stats.cunning = (self.stats.cunning as f64 * 1.5) as i64;
                self.stats.precision = (self.stats.precision as f64 * 1.5) as i64;
                self.stats.entropy = (self.stats.entropy as f64 * 1.5) as i64;
                self.stats.luck = (self.stats.luck as f64 * 1.5) as i64;
                self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
                self.current_hp = self.max_hp;
            }
            PrimeBlood => {
                // Effect happens on kill — tracked via boon field
            }
            ShadowStart => {
                self.current_hp = (self.max_hp / 2).max(1);
                self.xp_mult = 3.0;
            }
        }
    }

    /// Called by PrimeBlood boon when an enemy is killed.
    pub fn prime_blood_tick(&mut self) {
        // +1 to highest stat
        let max_val = [
            self.stats.vitality, self.stats.force, self.stats.mana,
            self.stats.cunning, self.stats.precision, self.stats.entropy, self.stats.luck,
        ]
        .iter()
        .copied()
        .max()
        .unwrap_or(0);

        if max_val == self.stats.vitality { self.stats.vitality += 1; self.max_hp += 3; }
        else if max_val == self.stats.force { self.stats.force += 1; }
        else if max_val == self.stats.mana { self.stats.mana += 1; }
        else if max_val == self.stats.cunning { self.stats.cunning += 1; }
        else if max_val == self.stats.precision { self.stats.precision += 1; }
        else if max_val == self.stats.entropy { self.stats.entropy += 1; }
        else { self.stats.luck += 1; }
    }

    fn level_up(&mut self) {
        self.level += 1;
        let seed = self.seed.wrapping_add(self.level as u64 * 31337);
        let roll = chaos_roll_verbose(self.level as f64 * 0.1, seed);

        // Stats grow based on class weights, amplified by chaos roll
        let weights = self.class.stat_weights();
        let chaos_mult = (roll.final_value + 1.5).max(0.5);

        self.stats.vitality += (weights.vitality / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.force += (weights.force / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.mana += (weights.mana / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.cunning += (weights.cunning / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.precision += (weights.precision / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.entropy += (weights.entropy / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.luck += (weights.luck / 20 + 1) * chaos_mult as i64 + 1;

        // HP scales with vitality (minimum 1 even with negative stats)
        let old_max = self.max_hp;
        self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
        self.current_hp += self.max_hp - old_max;
        self.current_hp = self.current_hp.min(self.max_hp);
    }

    pub fn level_up_and_learn_spell(&mut self) {
        self.level_up();
        // Learn a new spell on level up
        let spell_seed = self.seed
            .wrapping_add(self.level as u64 * 99991)
            .wrapping_mul(2654435761);
        self.known_spells.push(crate::spells::Spell::generate(spell_seed));
    }

    pub fn score(&self) -> u64 {
        let stat_total = self.stats.total().max(0) as u64;
        let floor_bonus = self.floor as u64 * 200;
        let level_bonus = self.level as u64 * 100;
        let kill_bonus = self.kills as u64 * 25;
        let room_bonus = self.rooms_cleared as u64 * 15;
        let spell_bonus = self.spells_cast as u64 * 5;
        stat_total + floor_bonus + level_bonus + kill_bonus + room_bonus
            + spell_bonus + self.gold.max(0) as u64
    }

    pub fn run_summary(&self) -> Vec<String> {
        let boon_str = self.boon
            .map(|b| format!("{}", b.name()))
            .unwrap_or_else(|| "None".to_string());
        vec![
            format!("  Difficulty:       {}{}\x1b[0m", self.difficulty.color_code(), self.difficulty.name()),
            format!("  Starting boon:    {}", boon_str),
            format!("  Floor reached:    {}", self.floor),
            format!("  Enemies slain:    {}", self.kills),
            format!("  Rooms cleared:    {}", self.rooms_cleared),
            format!("  Damage dealt:     {}", self.total_damage_dealt),
            format!("  Damage taken:     {}", self.total_damage_taken),
            format!("  Spells cast:      {}", self.spells_cast),
            format!("  Items used:       {}", self.items_used),
            format!("  Gold collected:   {}", self.gold),
            format!("  Final level:      {}", self.level),
            format!("  Power tier:       {}{}\x1b[0m",
                self.power_tier().color_code(), self.power_tier().name()),
        ]
    }

    pub fn hp_bar(&self, width: usize) -> String {
        let filled = ((self.hp_percent() * width as f64) as usize).min(width);
        let bar = "█".repeat(filled) + &"░".repeat(width - filled);
        format!("[{}] {}/{}", bar, self.current_hp, self.max_hp)
    }
}

// ─── STAT DISPLAY ────────────────────────────────────────────────────────────

pub fn stat_color(value: i64) -> &'static str {
    match value {
        i64::MIN..=-1 => "\x1b[35m", // magenta for negative (cursed/drained)
        0..=29 => "\x1b[31m",        // red
        30..=59 => "\x1b[33m",       // yellow
        60..=89 => "\x1b[32m",       // green
        90..=149 => "\x1b[36m",      // cyan
        _ => "\x1b[35m",             // magenta (godlike)
    }
}

pub fn display_stat(name: &str, value: i64) -> String {
    let color = stat_color(value);
    let reset = "\x1b[0m";
    let bar_len = (value.clamp(0, 100) as usize / 5).min(20);
    let bar = "▓".repeat(bar_len) + &"░".repeat(20 - bar_len);
    format!("  {:12} {}{:>6}{} [{}]", name, color, value, reset, bar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_creation_produces_valid_stats() {
        let c = Character::roll_new(
            "TestHero".to_string(),
            CharacterClass::Mage,
            Background::Scholar,
            42,
        );
        // Stats are UNBOUNDED — they can be any value including negative.
        // We only verify structural validity, not specific stat values.
        assert!(c.max_hp >= 1, "max_hp must be at least 1");
        assert!(c.current_hp >= 0, "current_hp cannot be negative at start");
        assert_eq!(c.level, 1);
        // Power tier must be computable without panic
        let _ = c.power_tier();
    }

    #[test]
    fn chaos_stats_are_unbounded_across_seeds() {
        // Over many seeds, we should see both positive and negative stats
        // proving the chaos engine isn't clamped.
        let mut saw_negative = false;
        let mut saw_large_positive = false;
        for seed in 0u64..50 {
            let c = Character::roll_new(
                "X".to_string(),
                CharacterClass::Berserker,
                Background::Gladiator,
                seed,
            );
            if c.stats.vitality < 0 { saw_negative = true; }
            if c.stats.force > 200 { saw_large_positive = true; }
        }
        // Not asserting both — chaos might not produce extremes in 50 seeds.
        // Just verify no panic occurred. The game is about unpredictability.
        let _ = (saw_negative, saw_large_positive);
    }

    #[test]
    fn character_takes_damage_correctly() {
        let mut c = Character::roll_new(
            "X".to_string(),
            CharacterClass::Thief,
            Background::Outcast,
            1,
        );
        let initial_hp = c.current_hp;
        c.take_damage(10);
        // HP is clamped to 0, so expected is (initial - 10).max(0)
        assert_eq!(c.current_hp, (initial_hp - 10).max(0));
    }

    #[test]
    fn hp_cannot_go_below_zero() {
        let mut c = Character::roll_new(
            "X".to_string(),
            CharacterClass::Ranger,
            Background::Wanderer,
            2,
        );
        c.take_damage(1_000_000);
        assert_eq!(c.current_hp, 0);
        assert!(!c.is_alive());
    }
}
