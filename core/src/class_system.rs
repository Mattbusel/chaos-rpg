//! Character class advancement: levels, abilities, and specializations.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ClassAbility
// ---------------------------------------------------------------------------

/// An active ability unlocked by advancing in a class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAbility {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level_required: u8,
    pub cooldown_turns: u32,
    pub mana_cost: u32,
    pub effect_type: String,
}

impl ClassAbility {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        level_required: u8,
        cooldown_turns: u32,
        mana_cost: u32,
        effect_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            level_required,
            cooldown_turns,
            mana_cost,
            effect_type: effect_type.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// ClassSpecialization
// ---------------------------------------------------------------------------

/// An optional specialization path available at higher levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassSpecialization {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlock_level: u8,
    pub bonus_abilities: Vec<ClassAbility>,
    pub passive_bonuses: Vec<String>,
}

impl ClassSpecialization {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        unlock_level: u8,
        bonus_abilities: Vec<ClassAbility>,
        passive_bonuses: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            unlock_level,
            bonus_abilities,
            passive_bonuses,
        }
    }
}

// ---------------------------------------------------------------------------
// CharacterClassDef
// ---------------------------------------------------------------------------

/// Full definition of a character class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterClassDef {
    pub id: String,
    pub name: String,
    /// Hit dice size (e.g. 10 for d10).
    pub hit_dice: u8,
    pub primary_attribute: String,
    pub abilities: Vec<ClassAbility>,
    pub specializations: Vec<ClassSpecialization>,
    /// `xp_curve[level]` = total XP required to reach that level.
    /// Index 0 is unused (or set to 0).
    pub xp_curve: Vec<u64>,
}

// ---------------------------------------------------------------------------
// ClassProgressionSystem
// ---------------------------------------------------------------------------

/// Stateless functions for class advancement logic.
pub struct ClassProgressionSystem;

impl ClassProgressionSystem {
    /// Add `gained` XP to `current_xp` and return `(new_xp, new_level)`.
    pub fn gain_xp(
        current_xp: u64,
        gained: u64,
        class: &CharacterClassDef,
    ) -> (u64, u8) {
        let new_xp = current_xp.saturating_add(gained);
        let new_level = Self::level_from_xp(new_xp, class);
        (new_xp, new_level)
    }

    /// Derive the current level from total accumulated XP.
    /// Returns the highest level whose XP threshold has been met.
    pub fn level_from_xp(xp: u64, class: &CharacterClassDef) -> u8 {
        let curve = &class.xp_curve;
        let mut level: u8 = 1;
        for (i, &threshold) in curve.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if xp >= threshold {
                level = i as u8;
            } else {
                break;
            }
        }
        level
    }

    /// Return all abilities available at or below `level`.
    pub fn abilities_at_level<'a>(
        class: &'a CharacterClassDef,
        level: u8,
    ) -> Vec<&'a ClassAbility> {
        class
            .abilities
            .iter()
            .filter(|a| a.level_required <= level)
            .collect()
    }

    /// Return specializations that can be chosen at `level`.
    pub fn available_specializations<'a>(
        class: &'a CharacterClassDef,
        level: u8,
    ) -> Vec<&'a ClassSpecialization> {
        class
            .specializations
            .iter()
            .filter(|s| s.unlock_level <= level)
            .collect()
    }

    /// Choose a specialization by id. Returns `false` if not available.
    /// On success, the specialization's bonus abilities are merged into the class.
    pub fn choose_specialization(
        class: &mut CharacterClassDef,
        spec_id: &str,
        level: u8,
    ) -> bool {
        // Find and clone the spec to avoid borrow conflicts
        let spec = class
            .specializations
            .iter()
            .find(|s| s.id == spec_id && s.unlock_level <= level)
            .cloned();

        match spec {
            None => false,
            Some(s) => {
                for ability in s.bonus_abilities {
                    class.abilities.push(ability);
                }
                true
            }
        }
    }

    // -----------------------------------------------------------------------
    // Default class factories
    // -----------------------------------------------------------------------

    /// Build a default Warrior class with 5 abilities and 2 specializations.
    pub fn default_warrior_class() -> CharacterClassDef {
        let abilities = vec![
            ClassAbility::new(
                "warrior_strike",
                "Power Strike",
                "A heavy melee blow dealing 150% weapon damage.",
                1,
                2,
                0,
                "damage",
            ),
            ClassAbility::new(
                "warrior_taunt",
                "Taunt",
                "Force nearby enemies to target the warrior for 2 turns.",
                2,
                4,
                0,
                "aggro",
            ),
            ClassAbility::new(
                "warrior_shield_wall",
                "Shield Wall",
                "Raise shield to reduce all incoming damage by 40% for 1 turn.",
                4,
                5,
                0,
                "defense",
            ),
            ClassAbility::new(
                "warrior_cleave",
                "Cleave",
                "Hit all adjacent enemies for 80% weapon damage.",
                6,
                3,
                10,
                "aoe_damage",
            ),
            ClassAbility::new(
                "warrior_battle_cry",
                "Battle Cry",
                "Boost all party members' attack by 20% for 3 turns.",
                10,
                8,
                20,
                "party_buff",
            ),
        ];

        let specs = vec![
            ClassSpecialization::new(
                "warrior_berserker",
                "Berserker",
                "Sacrifice defense for overwhelming offensive power.",
                5,
                vec![ClassAbility::new(
                    "berserk_rage",
                    "Berserk Rage",
                    "Double damage, halve defense for 3 turns.",
                    5,
                    10,
                    0,
                    "stance",
                )],
                vec![
                    "crit_chance +10%".into(),
                    "armor -15%".into(),
                ],
            ),
            ClassSpecialization::new(
                "warrior_guardian",
                "Guardian",
                "Become an unbreakable wall, protecting allies.",
                5,
                vec![ClassAbility::new(
                    "guardian_interpose",
                    "Interpose",
                    "Take a hit meant for an adjacent ally.",
                    5,
                    6,
                    0,
                    "redirect",
                )],
                vec![
                    "block_chance +15%".into(),
                    "hp_max +10%".into(),
                ],
            ),
        ];

        // xp_curve[level] = XP needed to BE that level
        // level 1 = 0 XP, level 2 = 300, level 3 = 900, ...
        let xp_curve: Vec<u64> = (0..=20)
            .map(|l: u64| if l <= 1 { 0 } else { 300 * l * l })
            .collect();

        CharacterClassDef {
            id: "warrior".into(),
            name: "Warrior".into(),
            hit_dice: 10,
            primary_attribute: "strength".into(),
            abilities,
            specializations: specs,
            xp_curve,
        }
    }

    /// Build a default Mage class with 5 abilities and 2 specializations.
    pub fn default_mage_class() -> CharacterClassDef {
        let abilities = vec![
            ClassAbility::new(
                "mage_bolt",
                "Magic Bolt",
                "Launch a bolt of arcane energy dealing 120% spell power.",
                1,
                1,
                15,
                "damage",
            ),
            ClassAbility::new(
                "mage_frost",
                "Frost Nova",
                "Freeze all nearby enemies for 1 turn.",
                2,
                5,
                30,
                "crowd_control",
            ),
            ClassAbility::new(
                "mage_shield",
                "Mana Shield",
                "Absorb up to 200 damage using mana instead of HP.",
                4,
                6,
                40,
                "defense",
            ),
            ClassAbility::new(
                "mage_fireball",
                "Fireball",
                "Launch an explosive fireball that deals AoE damage.",
                6,
                3,
                50,
                "aoe_damage",
            ),
            ClassAbility::new(
                "mage_time_warp",
                "Time Warp",
                "Reset all ability cooldowns for the caster.",
                10,
                20,
                100,
                "utility",
            ),
        ];

        let specs = vec![
            ClassSpecialization::new(
                "mage_pyromancer",
                "Pyromancer",
                "Channel destructive fire magic.",
                5,
                vec![ClassAbility::new(
                    "pyro_inferno",
                    "Inferno",
                    "Summon a column of fire dealing massive damage to a single target.",
                    5,
                    8,
                    80,
                    "damage",
                )],
                vec![
                    "fire_damage +20%".into(),
                    "frost_resistance -10%".into(),
                ],
            ),
            ClassSpecialization::new(
                "mage_arcane",
                "Arcane Scholar",
                "Master the pure arcane arts for versatile power.",
                5,
                vec![ClassAbility::new(
                    "arcane_surge",
                    "Arcane Surge",
                    "Empower next spell to deal double damage and ignore resistances.",
                    5,
                    7,
                    60,
                    "enhancement",
                )],
                vec![
                    "spell_power +15%".into(),
                    "mana_regen +20%".into(),
                ],
            ),
        ];

        let xp_curve: Vec<u64> = (0..=20)
            .map(|l: u64| if l <= 1 { 0 } else { 250 * l * l })
            .collect();

        CharacterClassDef {
            id: "mage".into(),
            name: "Mage".into(),
            hit_dice: 6,
            primary_attribute: "intelligence".into(),
            abilities,
            specializations: specs,
            xp_curve,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_from_xp_level1_at_zero() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        assert_eq!(ClassProgressionSystem::level_from_xp(0, &warrior), 1);
    }

    #[test]
    fn test_level_from_xp_advances() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        // curve[2] = 300 * 4 = 1200
        let lvl = ClassProgressionSystem::level_from_xp(1200, &warrior);
        assert!(lvl >= 2);
    }

    #[test]
    fn test_gain_xp_returns_new_level() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let (new_xp, new_level) = ClassProgressionSystem::gain_xp(0, 50_000, &warrior);
        assert!(new_xp == 50_000);
        assert!(new_level >= 10);
    }

    #[test]
    fn test_gain_xp_no_overflow() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let (new_xp, _) = ClassProgressionSystem::gain_xp(u64::MAX - 1, 10, &warrior);
        assert_eq!(new_xp, u64::MAX);
    }

    #[test]
    fn test_abilities_at_level_1() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let abilities = ClassProgressionSystem::abilities_at_level(&warrior, 1);
        assert!(!abilities.is_empty());
        for a in &abilities {
            assert!(a.level_required <= 1);
        }
    }

    #[test]
    fn test_abilities_at_level_10() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let abilities = ClassProgressionSystem::abilities_at_level(&warrior, 10);
        assert_eq!(abilities.len(), 5); // all 5 warrior abilities
    }

    #[test]
    fn test_abilities_at_level_3() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let abilities = ClassProgressionSystem::abilities_at_level(&warrior, 3);
        // level 1 and 2 abilities but not 4+
        for a in &abilities {
            assert!(a.level_required <= 3);
        }
    }

    #[test]
    fn test_available_specializations_below_unlock() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let specs = ClassProgressionSystem::available_specializations(&warrior, 4);
        assert!(specs.is_empty());
    }

    #[test]
    fn test_available_specializations_at_unlock() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let specs = ClassProgressionSystem::available_specializations(&warrior, 5);
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn test_choose_specialization_success() {
        let mut warrior = ClassProgressionSystem::default_warrior_class();
        let initial_count = warrior.abilities.len();
        let ok = ClassProgressionSystem::choose_specialization(&mut warrior, "warrior_berserker", 5);
        assert!(ok);
        assert!(warrior.abilities.len() > initial_count);
    }

    #[test]
    fn test_choose_specialization_too_low_level() {
        let mut warrior = ClassProgressionSystem::default_warrior_class();
        let ok = ClassProgressionSystem::choose_specialization(&mut warrior, "warrior_berserker", 4);
        assert!(!ok);
    }

    #[test]
    fn test_choose_specialization_invalid_id() {
        let mut warrior = ClassProgressionSystem::default_warrior_class();
        let ok = ClassProgressionSystem::choose_specialization(&mut warrior, "does_not_exist", 10);
        assert!(!ok);
    }

    #[test]
    fn test_default_warrior_has_correct_hit_dice() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        assert_eq!(warrior.hit_dice, 10);
        assert_eq!(warrior.primary_attribute, "strength");
    }

    #[test]
    fn test_default_mage_has_correct_hit_dice() {
        let mage = ClassProgressionSystem::default_mage_class();
        assert_eq!(mage.hit_dice, 6);
        assert_eq!(mage.primary_attribute, "intelligence");
    }

    #[test]
    fn test_mage_abilities_count() {
        let mage = ClassProgressionSystem::default_mage_class();
        assert_eq!(mage.abilities.len(), 5);
    }

    #[test]
    fn test_warrior_xp_curve_monotone() {
        let warrior = ClassProgressionSystem::default_warrior_class();
        let curve = &warrior.xp_curve;
        for i in 2..curve.len() {
            assert!(curve[i] >= curve[i - 1], "XP curve must be non-decreasing");
        }
    }

    #[test]
    fn test_mage_choose_specialization() {
        let mut mage = ClassProgressionSystem::default_mage_class();
        let ok = ClassProgressionSystem::choose_specialization(&mut mage, "mage_pyromancer", 5);
        assert!(ok);
    }
}
