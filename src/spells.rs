//! Spell generation — 7 schools, 140 named spells, cooldowns, spell levels.
//!
//! Schools map to primary stats:
//!   Fire → Force | Ice → Precision | Lightning → Entropy
//!   Arcane → Mana | Nature → Vitality | Shadow → Cunning | Chaos → Luck

use crate::chaos_pipeline::{chaos_roll_verbose, roll_stat};
use serde::{Deserialize, Serialize};

// ─── SPELL SCHOOL ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellSchool {
    Fire,
    Ice,
    Lightning,
    Arcane,
    Nature,
    Shadow,
    Chaos,
}

impl SpellSchool {
    pub fn name(&self) -> &'static str {
        match self {
            SpellSchool::Fire => "Fire",
            SpellSchool::Ice => "Ice",
            SpellSchool::Lightning => "Lightning",
            SpellSchool::Arcane => "Arcane",
            SpellSchool::Nature => "Nature",
            SpellSchool::Shadow => "Shadow",
            SpellSchool::Chaos => "CHAOS",
        }
    }

    pub fn scaling_stat(&self) -> &'static str {
        match self {
            SpellSchool::Fire => "Force",
            SpellSchool::Ice => "Precision",
            SpellSchool::Lightning => "Entropy",
            SpellSchool::Arcane => "Mana",
            SpellSchool::Nature => "Vitality",
            SpellSchool::Shadow => "Cunning",
            SpellSchool::Chaos => "Luck",
        }
    }

    /// ANSI color code for the school
    pub fn color(&self) -> &'static str {
        match self {
            SpellSchool::Fire => "\x1b[91m",      // bright red
            SpellSchool::Ice => "\x1b[96m",        // bright cyan
            SpellSchool::Lightning => "\x1b[93m",  // bright yellow
            SpellSchool::Arcane => "\x1b[95m",     // bright magenta
            SpellSchool::Nature => "\x1b[92m",     // bright green
            SpellSchool::Shadow => "\x1b[35m",     // dim magenta
            SpellSchool::Chaos => "\x1b[97m",      // bright white
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SpellSchool::Fire => "🔥",
            SpellSchool::Ice => "❄",
            SpellSchool::Lightning => "⚡",
            SpellSchool::Arcane => "✦",
            SpellSchool::Nature => "☘",
            SpellSchool::Shadow => "☽",
            SpellSchool::Chaos => "∞",
        }
    }

    fn from_idx(i: u64) -> Self {
        match i % 7 {
            0 => SpellSchool::Fire,
            1 => SpellSchool::Ice,
            2 => SpellSchool::Lightning,
            3 => SpellSchool::Arcane,
            4 => SpellSchool::Nature,
            5 => SpellSchool::Shadow,
            _ => SpellSchool::Chaos,
        }
    }
}

// ─── NAMED SPELL TEMPLATES ───────────────────────────────────────────────────

/// Static blueprint for a named spell.
/// `base_damage` and `base_mana` are scaled by chaos roll at generation time.
struct SpellTemplate {
    name: &'static str,
    base_damage: i64,
    base_mana: i64,
    aoe: i64,       // 0=single target, >0=radius, <0=self-inflicted
    cooldown: u32,  // turns
    fx_idx: usize,  // index into SIDE_EFFECTS
}

// fx_idx constants for readability
const FX_NONE: usize = 14; // "No visible side effect (suspicious)"
const FX_SELF_DMG: usize = 0; // "caster takes equal damage"
const FX_TELE: usize = 1; // "caster is teleported 1d6 rooms away"
const FX_GRAV: usize = 2; // "gravity reverses for 3 turns"
const FX_GOLD: usize = 15; // "you gain 1 gold"
const FX_OLDER: usize = 9; // "you age 10 years"
const FX_YOUNGER: usize = 10; // "you age -10 years"
const FX_EMBAR: usize = 13; // "nearby enemies are mildly embarrassed"
const FX_LEARN: usize = 8; // "the spell learns from this experience"
const FX_RAND: usize = 7; // "the spell targets a random entity"
const FX_FIRE: usize = 16; // "caster is briefly on fire (cosmetic only)"
const FX_NPC: usize = 6; // "a new NPC spawns and is immediately confused"
const FX_STATS: usize = 4; // "nothing visible happens but your stats secretly change"
const FX_DOOR: usize = 11; // "a door appears"
const FX_SHAKE: usize = 5; // "the screen shakes violently"
const FX_HOSTILE: usize = 3; // "all NPCs in the area become temporarily hostile"

static FIRE_SPELLS: &[SpellTemplate] = &[
    SpellTemplate { name: "Fireball",           base_damage: 280, base_mana: 30, aoe: 2, cooldown: 2, fx_idx: FX_FIRE },
    SpellTemplate { name: "Immolate",           base_damage: 200, base_mana: 20, aoe: 0, cooldown: 1, fx_idx: FX_FIRE },
    SpellTemplate { name: "Magma Surge",        base_damage: 350, base_mana: 45, aoe: 3, cooldown: 3, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Cinder Storm",       base_damage: 180, base_mana: 25, aoe: 4, cooldown: 3, fx_idx: FX_FIRE },
    SpellTemplate { name: "Pyroclasm",          base_damage: 420, base_mana: 55, aoe: 0, cooldown: 4, fx_idx: FX_SELF_DMG },
    SpellTemplate { name: "Infernal Wave",      base_damage: 260, base_mana: 35, aoe: 5, cooldown: 4, fx_idx: FX_HOSTILE },
    SpellTemplate { name: "Combustion",         base_damage: 150, base_mana: 15, aoe: 0, cooldown: 1, fx_idx: FX_NONE },
    SpellTemplate { name: "Ember Swarm",        base_damage: 120, base_mana: 18, aoe: 3, cooldown: 2, fx_idx: FX_EMBAR },
    SpellTemplate { name: "Molten Fist",        base_damage: 310, base_mana: 30, aoe: 0, cooldown: 2, fx_idx: FX_FIRE },
    SpellTemplate { name: "Solar Flare",        base_damage: 380, base_mana: 50, aoe: 4, cooldown: 4, fx_idx: FX_GRAV },
    SpellTemplate { name: "Ashen Breath",       base_damage: 170, base_mana: 22, aoe: 2, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Eruption",           base_damage: 450, base_mana: 60, aoe: 5, cooldown: 5, fx_idx: FX_TELE },
    SpellTemplate { name: "Phoenix Strike",     base_damage: 300, base_mana: 40, aoe: 0, cooldown: 3, fx_idx: FX_LEARN },
    SpellTemplate { name: "Slag Barrage",       base_damage: 200, base_mana: 28, aoe: 1, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Thermal Collapse",   base_damage: 400, base_mana: 55, aoe: 3, cooldown: 4, fx_idx: FX_SELF_DMG },
    SpellTemplate { name: "Wildfire Hex",       base_damage: 220, base_mana: 30, aoe: 2, cooldown: 2, fx_idx: FX_HOSTILE },
    SpellTemplate { name: "Flamewall",          base_damage: 190, base_mana: 25, aoe: 6, cooldown: 3, fx_idx: FX_NONE },
    SpellTemplate { name: "Scorched Earth",     base_damage: 240, base_mana: 32, aoe: 4, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Hellfire Bolt",      base_damage: 320, base_mana: 42, aoe: 0, cooldown: 2, fx_idx: FX_FIRE },
    SpellTemplate { name: "Ignite Protocol",    base_damage: 160, base_mana: 20, aoe: 0, cooldown: 1, fx_idx: FX_LEARN },
    SpellTemplate { name: "Cataclysm Surge",    base_damage: 500, base_mana: 70, aoe: 7, cooldown: 6, fx_idx: FX_TELE },
];

static ICE_SPELLS: &[SpellTemplate] = &[
    SpellTemplate { name: "Frostbolt",          base_damage: 220, base_mana: 20, aoe: 0, cooldown: 1, fx_idx: FX_NONE },
    SpellTemplate { name: "Glacial Spike",      base_damage: 310, base_mana: 35, aoe: 0, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Blizzard",           base_damage: 180, base_mana: 30, aoe: 5, cooldown: 4, fx_idx: FX_GRAV },
    SpellTemplate { name: "Ice Lance",          base_damage: 270, base_mana: 25, aoe: 0, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Permafrost",         base_damage: 140, base_mana: 18, aoe: 3, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Shatter",            base_damage: 380, base_mana: 45, aoe: 2, cooldown: 3, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Frozen Tomb",        base_damage: 200, base_mana: 35, aoe: 0, cooldown: 4, fx_idx: FX_STATS },
    SpellTemplate { name: "Crystallize",        base_damage: 160, base_mana: 22, aoe: 1, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Arctic Wind",        base_damage: 150, base_mana: 20, aoe: 4, cooldown: 3, fx_idx: FX_GRAV },
    SpellTemplate { name: "Avalanche",          base_damage: 420, base_mana: 55, aoe: 5, cooldown: 5, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Frost Nova",         base_damage: 200, base_mana: 28, aoe: 3, cooldown: 3, fx_idx: FX_EMBAR },
    SpellTemplate { name: "Cryogenic Pulse",    base_damage: 260, base_mana: 32, aoe: 2, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Absolute Zero",      base_damage: 480, base_mana: 65, aoe: 0, cooldown: 5, fx_idx: FX_TELE },
    SpellTemplate { name: "Hailstorm",          base_damage: 190, base_mana: 25, aoe: 6, cooldown: 4, fx_idx: FX_NONE },
    SpellTemplate { name: "Glacial Rift",       base_damage: 340, base_mana: 45, aoe: 1, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Ice Spear",          base_damage: 290, base_mana: 30, aoe: 0, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Deep Freeze",        base_damage: 230, base_mana: 38, aoe: 0, cooldown: 3, fx_idx: FX_OLDER },
    SpellTemplate { name: "Polar Vortex",       base_damage: 350, base_mana: 50, aoe: 5, cooldown: 5, fx_idx: FX_GRAV },
    SpellTemplate { name: "Winterbound",        base_damage: 170, base_mana: 24, aoe: 2, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Frost Sigil",        base_damage: 120, base_mana: 15, aoe: 0, cooldown: 1, fx_idx: FX_LEARN },
    SpellTemplate { name: "Cryo Cascade",       base_damage: 440, base_mana: 60, aoe: 4, cooldown: 5, fx_idx: FX_TELE },
];

static LIGHTNING_SPELLS: &[SpellTemplate] = &[
    SpellTemplate { name: "Chain Lightning",    base_damage: 250, base_mana: 28, aoe: 2, cooldown: 2, fx_idx: FX_RAND },
    SpellTemplate { name: "Thunderstrike",      base_damage: 330, base_mana: 38, aoe: 0, cooldown: 2, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Arc Pulse",          base_damage: 180, base_mana: 20, aoe: 1, cooldown: 1, fx_idx: FX_STATS },
    SpellTemplate { name: "Storm Surge",        base_damage: 290, base_mana: 35, aoe: 3, cooldown: 3, fx_idx: FX_GRAV },
    SpellTemplate { name: "Static Nova",        base_damage: 210, base_mana: 25, aoe: 4, cooldown: 3, fx_idx: FX_RAND },
    SpellTemplate { name: "Ball Lightning",     base_damage: 270, base_mana: 30, aoe: 2, cooldown: 2, fx_idx: FX_RAND },
    SpellTemplate { name: "Shock Wave",         base_damage: 360, base_mana: 45, aoe: 5, cooldown: 4, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Overcharge",         base_damage: 400, base_mana: 50, aoe: 0, cooldown: 3, fx_idx: FX_SELF_DMG },
    SpellTemplate { name: "Galvanic Burst",     base_damage: 230, base_mana: 28, aoe: 2, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Discharge",          base_damage: 310, base_mana: 40, aoe: 3, cooldown: 3, fx_idx: FX_SELF_DMG },
    SpellTemplate { name: "Lightning Conduit",  base_damage: 200, base_mana: 22, aoe: 1, cooldown: 1, fx_idx: FX_STATS },
    SpellTemplate { name: "Stormcall",          base_damage: 420, base_mana: 55, aoe: 6, cooldown: 5, fx_idx: FX_GRAV },
    SpellTemplate { name: "Voltage Spike",      base_damage: 160, base_mana: 18, aoe: 0, cooldown: 1, fx_idx: FX_NONE },
    SpellTemplate { name: "Arcing Death",       base_damage: 350, base_mana: 48, aoe: 3, cooldown: 3, fx_idx: FX_RAND },
    SpellTemplate { name: "Ionize",             base_damage: 140, base_mana: 15, aoe: 0, cooldown: 1, fx_idx: FX_STATS },
    SpellTemplate { name: "Mjolnir Drop",       base_damage: 490, base_mana: 65, aoe: 4, cooldown: 5, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Electron Cascade",   base_damage: 240, base_mana: 30, aoe: 2, cooldown: 2, fx_idx: FX_LEARN },
    SpellTemplate { name: "Plasma Burst",       base_damage: 300, base_mana: 40, aoe: 1, cooldown: 2, fx_idx: FX_FIRE },
    SpellTemplate { name: "Tempest Wrath",      base_damage: 380, base_mana: 52, aoe: 5, cooldown: 4, fx_idx: FX_GRAV },
    SpellTemplate { name: "Thunderclap",        base_damage: 220, base_mana: 25, aoe: 3, cooldown: 2, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Static Field",       base_damage: 170, base_mana: 22, aoe: 4, cooldown: 3, fx_idx: FX_STATS },
];

static ARCANE_SPELLS: &[SpellTemplate] = &[
    SpellTemplate { name: "Arcane Missile",     base_damage: 260, base_mana: 35, aoe: 0, cooldown: 1, fx_idx: FX_NONE },
    SpellTemplate { name: "Mana Void",          base_damage: 180, base_mana: -40, aoe: 0, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Void Surge",         base_damage: 320, base_mana: 45, aoe: 1, cooldown: 2, fx_idx: FX_TELE },
    SpellTemplate { name: "Prismatic Ray",      base_damage: 300, base_mana: 40, aoe: 0, cooldown: 2, fx_idx: FX_RAND },
    SpellTemplate { name: "Force Construct",    base_damage: 240, base_mana: 32, aoe: 2, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Arcane Torrent",     base_damage: 200, base_mana: 28, aoe: 3, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Reality Tear",       base_damage: 450, base_mana: 65, aoe: 0, cooldown: 5, fx_idx: FX_TELE },
    SpellTemplate { name: "Nullfield",          base_damage: 150, base_mana: 20, aoe: 5, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Arcane Resonance",   base_damage: 280, base_mana: 38, aoe: 2, cooldown: 2, fx_idx: FX_LEARN },
    SpellTemplate { name: "Singularity",        base_damage: 500, base_mana: 80, aoe: 6, cooldown: 6, fx_idx: FX_GRAV },
    SpellTemplate { name: "Mirror Shard",       base_damage: 220, base_mana: 30, aoe: 0, cooldown: 2, fx_idx: FX_RAND },
    SpellTemplate { name: "Time Fracture",      base_damage: 340, base_mana: 50, aoe: 0, cooldown: 4, fx_idx: FX_OLDER },
    SpellTemplate { name: "Arcane Overload",    base_damage: 420, base_mana: 60, aoe: 2, cooldown: 4, fx_idx: FX_SELF_DMG },
    SpellTemplate { name: "Phase Blast",        base_damage: 290, base_mana: 40, aoe: 1, cooldown: 2, fx_idx: FX_TELE },
    SpellTemplate { name: "Mana Siphon",        base_damage: 160, base_mana: -30, aoe: 0, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Dimensional Rift",   base_damage: 380, base_mana: 55, aoe: 3, cooldown: 4, fx_idx: FX_TELE },
    SpellTemplate { name: "Spectral Bolt",      base_damage: 230, base_mana: 28, aoe: 0, cooldown: 1, fx_idx: FX_NONE },
    SpellTemplate { name: "Arcane Infusion",    base_damage: -100, base_mana: -50, aoe: 0, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Mindburn",           base_damage: 350, base_mana: 48, aoe: 0, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Entropy Lens",       base_damage: 270, base_mana: 35, aoe: 0, cooldown: 2, fx_idx: FX_LEARN },
    SpellTemplate { name: "Spell Echo",         base_damage: 200, base_mana: 25, aoe: 0, cooldown: 2, fx_idx: FX_LEARN },
];

static NATURE_SPELLS: &[SpellTemplate] = &[
    SpellTemplate { name: "Thorn Volley",       base_damage: 180, base_mana: 20, aoe: 2, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Regrowth",           base_damage: -250, base_mana: 25, aoe: 0, cooldown: 3, fx_idx: FX_NONE },
    SpellTemplate { name: "Verdant Surge",      base_damage: 220, base_mana: 28, aoe: 3, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Spore Cloud",        base_damage: 140, base_mana: 18, aoe: 5, cooldown: 3, fx_idx: FX_EMBAR },
    SpellTemplate { name: "Root Grasp",         base_damage: 200, base_mana: 25, aoe: 1, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Pollen Burst",       base_damage: 120, base_mana: 16, aoe: 4, cooldown: 2, fx_idx: FX_EMBAR },
    SpellTemplate { name: "Overgrowth",         base_damage: 300, base_mana: 40, aoe: 4, cooldown: 4, fx_idx: FX_NPC },
    SpellTemplate { name: "Chloro Blast",       base_damage: 260, base_mana: 32, aoe: 0, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Healing Tide",       base_damage: -350, base_mana: 35, aoe: 3, cooldown: 4, fx_idx: FX_NONE },
    SpellTemplate { name: "Primal Roar",        base_damage: 280, base_mana: 35, aoe: 5, cooldown: 4, fx_idx: FX_HOSTILE },
    SpellTemplate { name: "Barkskin Crush",     base_damage: 240, base_mana: 28, aoe: 0, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Fungal Wall",        base_damage: 160, base_mana: 22, aoe: 2, cooldown: 2, fx_idx: FX_NPC },
    SpellTemplate { name: "Vine Whip",          base_damage: 190, base_mana: 22, aoe: 0, cooldown: 1, fx_idx: FX_NONE },
    SpellTemplate { name: "Nourishing Rain",    base_damage: -200, base_mana: 30, aoe: 4, cooldown: 3, fx_idx: FX_GOLD },
    SpellTemplate { name: "Photosynthesis Bolt",base_damage: 170, base_mana: 20, aoe: 0, cooldown: 1, fx_idx: FX_STATS },
    SpellTemplate { name: "Natural Selection",  base_damage: 350, base_mana: 50, aoe: 0, cooldown: 4, fx_idx: FX_RAND },
    SpellTemplate { name: "Mycelium Net",       base_damage: 130, base_mana: 18, aoe: 3, cooldown: 2, fx_idx: FX_NPC },
    SpellTemplate { name: "Gaia's Wrath",       base_damage: 460, base_mana: 65, aoe: 7, cooldown: 6, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Seed Bomb",          base_damage: 200, base_mana: 24, aoe: 3, cooldown: 2, fx_idx: FX_DOOR },
    SpellTemplate { name: "Living Shield",      base_damage: -150, base_mana: 20, aoe: 0, cooldown: 2, fx_idx: FX_NONE },
    SpellTemplate { name: "Druidic Pulse",      base_damage: 210, base_mana: 27, aoe: 2, cooldown: 2, fx_idx: FX_LEARN },
];

static SHADOW_SPELLS: &[SpellTemplate] = &[
    SpellTemplate { name: "Shadowbolt",         base_damage: 240, base_mana: 25, aoe: 0, cooldown: 1, fx_idx: FX_NONE },
    SpellTemplate { name: "Soul Drain",         base_damage: 200, base_mana: 22, aoe: 0, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Void Strike",        base_damage: 300, base_mana: 35, aoe: 0, cooldown: 2, fx_idx: FX_TELE },
    SpellTemplate { name: "Curse of Agony",     base_damage: 160, base_mana: 20, aoe: 0, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Necrotic Pulse",     base_damage: 220, base_mana: 28, aoe: 2, cooldown: 2, fx_idx: FX_HOSTILE },
    SpellTemplate { name: "Shadow Tendril",     base_damage: 180, base_mana: 22, aoe: 1, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Death Mark",         base_damage: 350, base_mana: 45, aoe: 0, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Entropy Leech",      base_damage: 150, base_mana: -20, aoe: 0, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Spectral Wound",     base_damage: 280, base_mana: 35, aoe: 0, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Umbral Surge",       base_damage: 320, base_mana: 42, aoe: 2, cooldown: 3, fx_idx: FX_TELE },
    SpellTemplate { name: "Black Hole",         base_damage: 460, base_mana: 65, aoe: 5, cooldown: 5, fx_idx: FX_GRAV },
    SpellTemplate { name: "Doombolt",           base_damage: 390, base_mana: 50, aoe: 0, cooldown: 3, fx_idx: FX_OLDER },
    SpellTemplate { name: "Lifetap",            base_damage: 130, base_mana: -45, aoe: 0, cooldown: 2, fx_idx: FX_SELF_DMG },
    SpellTemplate { name: "Soul Rend",          base_damage: 340, base_mana: 45, aoe: 0, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Whisper of Oblivion",base_damage: 200, base_mana: 28, aoe: 3, cooldown: 3, fx_idx: FX_HOSTILE },
    SpellTemplate { name: "Dread Sigil",        base_damage: 260, base_mana: 33, aoe: 0, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Midnight Surge",     base_damage: 290, base_mana: 38, aoe: 1, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Abyssal Cry",        base_damage: 250, base_mana: 32, aoe: 4, cooldown: 3, fx_idx: FX_HOSTILE },
    SpellTemplate { name: "Phantom Strike",     base_damage: 310, base_mana: 38, aoe: 0, cooldown: 2, fx_idx: FX_TELE },
    SpellTemplate { name: "Hex Bolt",           base_damage: 170, base_mana: 20, aoe: 0, cooldown: 1, fx_idx: FX_EMBAR },
    SpellTemplate { name: "Voidweave",          base_damage: 420, base_mana: 60, aoe: 3, cooldown: 4, fx_idx: FX_TELE },
];

static CHAOS_SPELLS: &[SpellTemplate] = &[
    SpellTemplate { name: "Chaos Bolt",             base_damage: 0,    base_mana: 20,  aoe: 0, cooldown: 0, fx_idx: FX_RAND },
    SpellTemplate { name: "Random Catastrophe",     base_damage: 500,  base_mana: -50, aoe: 7, cooldown: 6, fx_idx: FX_SELF_DMG },
    SpellTemplate { name: "Probability Collapse",   base_damage: 300,  base_mana: 0,   aoe: 3, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Schrodinger's Fireball", base_damage: 400,  base_mana: 40,  aoe: 4, cooldown: 4, fx_idx: FX_RAND },
    SpellTemplate { name: "Butterfly Effect",       base_damage: 10,   base_mana: 5,   aoe: 0, cooldown: 1, fx_idx: FX_LEARN },
    SpellTemplate { name: "Murphy's Law",           base_damage: -200, base_mana: -20, aoe: 2, cooldown: 2, fx_idx: FX_HOSTILE },
    SpellTemplate { name: "Undefined Behavior",     base_damage: 9999, base_mana: 9999,aoe: 0, cooldown: 9, fx_idx: FX_TELE },
    SpellTemplate { name: "Recursive Paradox",      base_damage: 100,  base_mana: 10,  aoe: 0, cooldown: 2, fx_idx: FX_LEARN },
    SpellTemplate { name: "Null Pointer Exception", base_damage: 0,    base_mana: 0,   aoe: 0, cooldown: 0, fx_idx: FX_STATS },
    SpellTemplate { name: "Stack Overflow",         base_damage: 450,  base_mana: 999, aoe: 9, cooldown: 8, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Integer Overflow",       base_damage: -9999,base_mana: 0,   aoe: 0, cooldown: 3, fx_idx: FX_STATS },
    SpellTemplate { name: "Divide By Zero",         base_damage: 1,    base_mana: 1,   aoe: 0, cooldown: 1, fx_idx: FX_SHAKE },
    SpellTemplate { name: "Heisenberg Uncertainty", base_damage: 250,  base_mana: 25,  aoe: 0, cooldown: 2, fx_idx: FX_RAND },
    SpellTemplate { name: "Quantum Fluctuation",    base_damage: 150,  base_mana: 15,  aoe: 1, cooldown: 1, fx_idx: FX_RAND },
    SpellTemplate { name: "Eigenvector Eruption",   base_damage: 360,  base_mana: 45,  aoe: 3, cooldown: 3, fx_idx: FX_GRAV },
    SpellTemplate { name: "Lorenz Attractor",       base_damage: 200,  base_mana: 20,  aoe: 2, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Julia Set Burst",        base_damage: 280,  base_mana: 30,  aoe: 2, cooldown: 2, fx_idx: FX_YOUNGER },
    SpellTemplate { name: "Mandelbrot Cascade",     base_damage: 340,  base_mana: 40,  aoe: 4, cooldown: 4, fx_idx: FX_DOOR },
    SpellTemplate { name: "Cantor Dust Storm",      base_damage: 190,  base_mana: 22,  aoe: 5, cooldown: 3, fx_idx: FX_RAND },
    SpellTemplate { name: "Bifurcation Point",      base_damage: 260,  base_mana: 28,  aoe: 0, cooldown: 2, fx_idx: FX_STATS },
    SpellTemplate { name: "Chaos Singularity",      base_damage: 700,  base_mana: 100, aoe: 10,cooldown: 9, fx_idx: FX_SELF_DMG },
];

fn school_templates(school: SpellSchool) -> &'static [SpellTemplate] {
    match school {
        SpellSchool::Fire => FIRE_SPELLS,
        SpellSchool::Ice => ICE_SPELLS,
        SpellSchool::Lightning => LIGHTNING_SPELLS,
        SpellSchool::Arcane => ARCANE_SPELLS,
        SpellSchool::Nature => NATURE_SPELLS,
        SpellSchool::Shadow => SHADOW_SPELLS,
        SpellSchool::Chaos => CHAOS_SPELLS,
    }
}

// ─── SIDE EFFECTS ────────────────────────────────────────────────────────────

const SIDE_EFFECTS: &[&str] = &[
    "caster takes equal damage",               // 0  FX_SELF_DMG
    "caster is teleported 1d6 rooms away",     // 1  FX_TELE
    "gravity reverses for 3 turns",            // 2  FX_GRAV
    "all NPCs in the area become temporarily hostile", // 3  FX_HOSTILE
    "nothing visible happens but your stats secretly change", // 4  FX_STATS
    "the screen shakes violently",             // 5  FX_SHAKE
    "a new NPC spawns and is immediately confused",   // 6  FX_NPC
    "the spell targets a random entity",       // 7  FX_RAND
    "the spell learns from this experience and grows slightly stronger", // 8  FX_LEARN
    "you age 10 years",                        // 9  FX_OLDER
    "you age -10 years (become younger)",      // 10 FX_YOUNGER
    "a door appears",                          // 11 FX_DOOR
    "a door disappears",                       // 12
    "nearby enemies are mildly embarrassed",   // 13 FX_EMBAR
    "No visible side effect (suspicious)",     // 14 FX_NONE
    "you gain 1 gold",                         // 15 FX_GOLD
    "caster is briefly on fire (cosmetic only)", // 16 FX_FIRE
];

// ─── SPELL STRUCT ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub name: String,
    pub school: SpellSchool,
    pub damage: i64,      // can be negative (heals)
    pub mana_cost: i64,   // can be negative (gives mana)
    pub aoe_radius: i64,  // 0=single, >0=radius, <0=hits own party
    pub side_effect: String,
    pub scaling_stat: String,
    pub scaling_factor: f64,
    pub engines_used: Vec<String>,
    pub cooldown: u32,         // max cooldown in turns (0 = no cooldown)
    pub current_cooldown: u32, // turns until ready
    pub spell_level: u32,      // 1-based; increases every 5 casts
    pub casts: u32,            // total times cast
}

impl Spell {
    /// Generate a spell from a specific school.
    pub fn generate_from_school(seed: u64, school: SpellSchool) -> Self {
        let templates = school_templates(school);
        let t_idx = (seed % templates.len() as u64) as usize;
        let tmpl = &templates[t_idx];

        // Chaos roll for damage variance (±40% from base)
        let dmg_roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(1));
        let variance = 1.0 + dmg_roll.final_value * 0.4;
        let damage = (tmpl.base_damage as f64 * variance) as i64;

        // Mana cost variance (±20%)
        let mana_roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(3));
        let mana_var = 1.0 + mana_roll.final_value * 0.2;
        let mana_cost = (tmpl.base_mana as f64 * mana_var) as i64;

        // Scaling factor for this school (1.0–2.5, chaos-modulated)
        let scale_roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(6));
        let scaling_factor = 1.0 + (scale_roll.final_value.abs() * 1.5);

        let fx_idx = tmpl.fx_idx.min(SIDE_EFFECTS.len() - 1);
        let side_effect = SIDE_EFFECTS[fx_idx].to_string();

        let engines_used: Vec<String> = dmg_roll
            .chain
            .iter()
            .map(|s| s.engine_name.to_string())
            .collect();

        Spell {
            name: tmpl.name.to_string(),
            school,
            damage,
            mana_cost,
            aoe_radius: tmpl.aoe,
            side_effect,
            scaling_stat: school.scaling_stat().to_string(),
            scaling_factor,
            engines_used,
            cooldown: tmpl.cooldown,
            current_cooldown: 0,
            spell_level: 1,
            casts: 0,
        }
    }

    /// Generate a spell picking the school from the seed.
    pub fn generate(seed: u64) -> Self {
        let school = SpellSchool::from_idx(seed.wrapping_mul(6364136223846793005));
        Self::generate_from_school(seed, school)
    }

    // ─── Cooldown / Level ────────────────────────────────────────────────────

    pub fn is_ready(&self) -> bool {
        self.current_cooldown == 0
    }

    pub fn tick_cooldown(&mut self) {
        if self.current_cooldown > 0 {
            self.current_cooldown -= 1;
        }
    }

    /// Call after successfully casting this spell.
    pub fn on_cast(&mut self) {
        self.casts += 1;
        self.spell_level = 1 + self.casts / 5;
        if self.cooldown > 0 {
            self.current_cooldown = self.cooldown;
        }
    }

    // ─── Damage calculation ──────────────────────────────────────────────────

    /// Damage including stat scaling (no level bonus).
    pub fn calc_damage(&self, stat_value: i64) -> i64 {
        let bonus = (stat_value as f64 * self.scaling_factor) as i64;
        self.damage + bonus
    }

    /// Damage including both stat scaling and spell level multiplier.
    pub fn calc_damage_leveled(&self, stat_value: i64) -> i64 {
        let base = self.calc_damage(stat_value);
        let level_mult = 1.0 + (self.spell_level.saturating_sub(1)) as f64 * 0.1;
        (base as f64 * level_mult) as i64
    }

    // ─── Display ─────────────────────────────────────────────────────────────

    pub fn display_box(&self) -> Vec<String> {
        let color = self.school.color();
        let reset = "\x1b[0m";
        let width = 46usize;
        let inner = width - 2;

        let mut lines = Vec::new();
        lines.push(format!("{}┌{}┐{}", color, "─".repeat(width), reset));

        // Name line with school icon
        let name_display: String = self.name.chars().take(inner - 5).collect();
        lines.push(format!(
            "{}│ {} {:<width$}│{}",
            color,
            self.school.icon(),
            name_display,
            reset,
            width = inner - 5
        ));

        // School + level + cooldown status
        let cd_str = if self.current_cooldown > 0 {
            format!("CD:{}", self.current_cooldown)
        } else {
            "READY".to_string()
        };
        let school_line = format!(
            "{} Lv.{}  {}",
            self.school.name(),
            self.spell_level,
            cd_str
        );
        lines.push(format!(
            "{}│   {:<width$}│{}",
            color,
            school_line,
            reset,
            width = inner - 3
        ));

        // Damage
        let dmg_sign = if self.damage >= 0 { "+" } else { "" };
        lines.push(format!(
            "{}│   Damage: {}{:<width$}│{}",
            color,
            dmg_sign,
            self.damage,
            reset,
            width = inner - 11
        ));

        // Mana
        let mana_note = if self.mana_cost < 0 { " (GIVES mana!)" } else { "" };
        lines.push(format!(
            "{}│   Mana Cost: {}{:<w$}│{}",
            color,
            self.mana_cost,
            mana_note,
            reset,
            w = (inner - 14 - mana_note.len()).max(1)
        ));

        // AoE
        let aoe_desc = match self.aoe_radius.cmp(&0) {
            std::cmp::Ordering::Equal => "single target".to_string(),
            std::cmp::Ordering::Greater => format!("{} tile radius", self.aoe_radius),
            std::cmp::Ordering::Less => format!("hits own party ({})", self.aoe_radius),
        };
        lines.push(format!(
            "{}│   AoE: {:<width$}│{}",
            color,
            aoe_desc,
            reset,
            width = inner - 9
        ));

        // Scaling
        lines.push(format!(
            "{}│   Scales: {} ×{:.2}{:<w$}│{}",
            color,
            self.scaling_stat,
            self.scaling_factor,
            "",
            reset,
            w = (inner.saturating_sub(self.scaling_stat.len() + 14)).max(1)
        ));

        // Side effect
        let fx = &self.side_effect;
        let fx_display = if fx.len() > inner - 14 {
            format!("{}...", &fx[..inner.saturating_sub(17)])
        } else {
            fx.clone()
        };
        lines.push(format!(
            "{}│   Side FX: {:<w$}│{}",
            color,
            fx_display,
            reset,
            w = (inner - 13).max(1)
        ));

        // Engine chain
        let engines_str: String = self
            .engines_used
            .iter()
            .take(4)
            .map(|e| e.split_whitespace().next().unwrap_or(e).to_string())
            .collect::<Vec<_>>()
            .join("→");
        lines.push(format!(
            "{}│   [{:<width$}]│{}",
            color,
            engines_str,
            reset,
            width = inner - 4
        ));

        lines.push(format!("{}└{}┘{}", color, "─".repeat(width), reset));
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_generation_valid() {
        for seed in 0..10u64 {
            let spell = Spell::generate(seed);
            assert!(!spell.name.is_empty());
            assert!(!spell.scaling_stat.is_empty());
        }
    }

    #[test]
    fn all_schools_generate() {
        let schools = [
            SpellSchool::Fire,
            SpellSchool::Ice,
            SpellSchool::Lightning,
            SpellSchool::Arcane,
            SpellSchool::Nature,
            SpellSchool::Shadow,
            SpellSchool::Chaos,
        ];
        for &school in &schools {
            let spell = Spell::generate_from_school(42, school);
            assert_eq!(spell.school, school);
            assert_eq!(spell.scaling_stat, school.scaling_stat());
        }
    }

    #[test]
    fn spell_levels_up_every_5_casts() {
        let mut spell = Spell::generate(99);
        assert_eq!(spell.spell_level, 1);
        for _ in 0..5 {
            spell.on_cast();
        }
        assert_eq!(spell.spell_level, 2);
    }

    #[test]
    fn cooldown_ticks_down() {
        let mut spell = Spell::generate_from_school(1, SpellSchool::Fire);
        spell.on_cast();
        let cd = spell.current_cooldown;
        if cd > 0 {
            spell.tick_cooldown();
            assert_eq!(spell.current_cooldown, cd - 1);
        }
    }

    #[test]
    fn negative_mana_spell_exists_in_distribution() {
        let mut found_negative = false;
        for seed in 0..200u64 {
            let spell = Spell::generate(seed);
            if spell.mana_cost < 0 {
                found_negative = true;
                break;
            }
        }
        assert!(found_negative, "Should occasionally generate negative-cost spells");
    }
}
