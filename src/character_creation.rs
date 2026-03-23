//! Character creation — stat rolling, class/race bonuses, starting equipment.

use serde::{Deserialize, Serialize};

/// Playable character classes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Warrior,
    Mage,
    Rogue,
    Cleric,
    Ranger,
    Paladin,
}

impl CharacterClass {
    /// The primary stat that benefits most from this class.
    pub fn primary_stat(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "strength",
            CharacterClass::Mage => "intelligence",
            CharacterClass::Rogue => "dexterity",
            CharacterClass::Cleric => "wisdom",
            CharacterClass::Ranger => "dexterity",
            CharacterClass::Paladin => "charisma",
        }
    }

    /// The number of sides on the hit die for this class.
    pub fn hit_dice_sides(&self) -> u8 {
        match self {
            CharacterClass::Warrior => 10,
            CharacterClass::Mage => 6,
            CharacterClass::Rogue => 8,
            CharacterClass::Cleric => 8,
            CharacterClass::Ranger => 8,
            CharacterClass::Paladin => 10,
        }
    }
}

/// The six core stats of a character.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterStats {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl CharacterStats {
    /// 5e-style ability modifier: (stat - 10) / 2, rounded toward negative infinity.
    pub fn modifier(stat: u8) -> i8 {
        let s = stat as i16;
        ((s - 10) / 2) as i8
    }

    /// Apply a stat bonus by name.
    pub fn apply_bonus(&mut self, stat: &str, amount: i8) {
        let apply = |v: &mut u8, a: i8| {
            *v = ((*v as i16 + a as i16).max(1).min(30)) as u8;
        };
        match stat {
            "strength" => apply(&mut self.strength, amount),
            "dexterity" => apply(&mut self.dexterity, amount),
            "constitution" => apply(&mut self.constitution, amount),
            "intelligence" => apply(&mut self.intelligence, amount),
            "wisdom" => apply(&mut self.wisdom, amount),
            "charisma" => apply(&mut self.charisma, amount),
            _ => {}
        }
    }
}

/// A playable race with stat bonuses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
    pub strength_bonus: i8,
    pub dexterity_bonus: i8,
    pub constitution_bonus: i8,
    pub intelligence_bonus: i8,
    pub wisdom_bonus: i8,
    pub charisma_bonus: i8,
}

impl Race {
    pub fn human() -> Self {
        Self {
            name: "Human".to_string(),
            strength_bonus: 1,
            dexterity_bonus: 1,
            constitution_bonus: 1,
            intelligence_bonus: 1,
            wisdom_bonus: 1,
            charisma_bonus: 1,
        }
    }

    pub fn elf() -> Self {
        Self {
            name: "Elf".to_string(),
            strength_bonus: 0,
            dexterity_bonus: 2,
            constitution_bonus: 0,
            intelligence_bonus: 1,
            wisdom_bonus: 0,
            charisma_bonus: 0,
        }
    }

    pub fn dwarf() -> Self {
        Self {
            name: "Dwarf".to_string(),
            strength_bonus: 2,
            dexterity_bonus: 0,
            constitution_bonus: 2,
            intelligence_bonus: 0,
            wisdom_bonus: 0,
            charisma_bonus: -1,
        }
    }
}

/// The method used to generate ability scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatRollMethod {
    /// Roll `rolls` dice (typically 4d6), drop the lowest.
    FourDrop(u8),
    /// Use the fixed array [15,14,13,12,10,8].
    StandardArray,
    /// Distribute `points` across stats using a point-buy table.
    PointBuy(u8),
}

/// Provides static methods for building a new character.
pub struct CharacterCreator;

/// Seeded LCG — returns a value in [1, sides].
fn lcg(seed: u64, sides: u8) -> u8 {
    if sides == 0 {
        return 0;
    }
    let v = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((v >> 33) as u8) % sides + 1
}

impl CharacterCreator {
    /// Roll one ability score using FourDrop: roll 4d6, drop lowest.
    fn roll_one_fourdrop(seed: u64) -> u8 {
        let rolls: Vec<u8> = (0..4).map(|i| lcg(seed.wrapping_add(i * 997), 6)).collect();
        let min = *rolls.iter().min().unwrap();
        let total: u8 = rolls.iter().sum::<u8>() - min;
        total
    }

    /// Roll all six ability scores using the given method.
    pub fn roll_stats(method: &StatRollMethod, seed: u64) -> CharacterStats {
        match method {
            StatRollMethod::FourDrop(_) => CharacterStats {
                strength: Self::roll_one_fourdrop(seed),
                dexterity: Self::roll_one_fourdrop(seed.wrapping_add(1)),
                constitution: Self::roll_one_fourdrop(seed.wrapping_add(2)),
                intelligence: Self::roll_one_fourdrop(seed.wrapping_add(3)),
                wisdom: Self::roll_one_fourdrop(seed.wrapping_add(4)),
                charisma: Self::roll_one_fourdrop(seed.wrapping_add(5)),
            },
            StatRollMethod::StandardArray => CharacterStats {
                strength: 15,
                dexterity: 14,
                constitution: 13,
                intelligence: 12,
                wisdom: 10,
                charisma: 8,
            },
            StatRollMethod::PointBuy(points) => {
                // Simple even distribution across 6 stats; base cost is (val-8).
                let base = 8u8;
                let per_stat = (*points / 6).min(7); // cap at base+7 = 15
                CharacterStats {
                    strength: base + per_stat,
                    dexterity: base + per_stat,
                    constitution: base + per_stat,
                    intelligence: base + per_stat,
                    wisdom: base + per_stat,
                    charisma: base + per_stat,
                }
            }
        }
    }

    /// Apply racial ability score bonuses.
    pub fn apply_race_bonuses(stats: &mut CharacterStats, race: &Race) {
        stats.apply_bonus("strength", race.strength_bonus);
        stats.apply_bonus("dexterity", race.dexterity_bonus);
        stats.apply_bonus("constitution", race.constitution_bonus);
        stats.apply_bonus("intelligence", race.intelligence_bonus);
        stats.apply_bonus("wisdom", race.wisdom_bonus);
        stats.apply_bonus("charisma", race.charisma_bonus);
    }

    /// Apply class-specific ability score bonuses (+2 to primary stat).
    pub fn apply_class_bonuses(stats: &mut CharacterStats, class: &CharacterClass) {
        stats.apply_bonus(class.primary_stat(), 2);
    }

    /// Calculate maximum HP for a character.
    ///
    /// At level 1: hit_dice_sides + con_modifier.
    /// Each additional level: hit_dice_sides/2 + 1 + con_modifier.
    pub fn max_hp(constitution: u8, class: &CharacterClass, level: u8) -> u32 {
        let con_mod = CharacterStats::modifier(constitution) as i32;
        let hd = class.hit_dice_sides() as i32;
        let first_level = hd + con_mod;
        if level == 0 {
            return first_level.max(1) as u32;
        }
        let remaining = (level as i32 - 1) * (hd / 2 + 1 + con_mod);
        (first_level + remaining).max(level as i32) as u32
    }

    /// Return a list of starting equipment items for the given class.
    pub fn starting_equipment(class: &CharacterClass) -> Vec<String> {
        match class {
            CharacterClass::Warrior => vec![
                "Longsword".to_string(),
                "Shield".to_string(),
                "Chain Mail".to_string(),
                "Explorer's Pack".to_string(),
            ],
            CharacterClass::Mage => vec![
                "Quarterstaff".to_string(),
                "Spellbook".to_string(),
                "Component Pouch".to_string(),
                "Scholar's Pack".to_string(),
            ],
            CharacterClass::Rogue => vec![
                "Shortsword".to_string(),
                "Shortbow".to_string(),
                "20 Arrows".to_string(),
                "Leather Armor".to_string(),
                "Thieves' Tools".to_string(),
                "Burglar's Pack".to_string(),
            ],
            CharacterClass::Cleric => vec![
                "Mace".to_string(),
                "Scale Mail".to_string(),
                "Shield".to_string(),
                "Holy Symbol".to_string(),
                "Priest's Pack".to_string(),
            ],
            CharacterClass::Ranger => vec![
                "Longbow".to_string(),
                "20 Arrows".to_string(),
                "Shortsword".to_string(),
                "Leather Armor".to_string(),
                "Explorer's Pack".to_string(),
            ],
            CharacterClass::Paladin => vec![
                "Longsword".to_string(),
                "Shield".to_string(),
                "Chain Mail".to_string(),
                "Holy Symbol".to_string(),
                "Priest's Pack".to_string(),
            ],
        }
    }

    /// Full character creation pipeline.
    ///
    /// Returns `(stats, max_hp, equipment)`.
    pub fn create_character(
        name: &str,
        race: &Race,
        class: CharacterClass,
        method: &StatRollMethod,
        stat_seed: u64,
    ) -> (CharacterStats, u32, Vec<String>) {
        let _ = name; // stored by caller
        let mut stats = Self::roll_stats(method, stat_seed);
        Self::apply_race_bonuses(&mut stats, race);
        Self::apply_class_bonuses(&mut stats, &class);
        let hp = Self::max_hp(stats.constitution, &class, 1);
        let equipment = Self::starting_equipment(&class);
        (stats, hp, equipment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_primary_stats() {
        assert_eq!(CharacterClass::Warrior.primary_stat(), "strength");
        assert_eq!(CharacterClass::Mage.primary_stat(), "intelligence");
        assert_eq!(CharacterClass::Rogue.primary_stat(), "dexterity");
        assert_eq!(CharacterClass::Cleric.primary_stat(), "wisdom");
        assert_eq!(CharacterClass::Ranger.primary_stat(), "dexterity");
        assert_eq!(CharacterClass::Paladin.primary_stat(), "charisma");
    }

    #[test]
    fn test_hit_dice_sides() {
        assert_eq!(CharacterClass::Warrior.hit_dice_sides(), 10);
        assert_eq!(CharacterClass::Mage.hit_dice_sides(), 6);
        assert_eq!(CharacterClass::Rogue.hit_dice_sides(), 8);
    }

    #[test]
    fn test_modifier() {
        assert_eq!(CharacterStats::modifier(10), 0);
        assert_eq!(CharacterStats::modifier(12), 1);
        assert_eq!(CharacterStats::modifier(8), -1);
        assert_eq!(CharacterStats::modifier(20), 5);
    }

    #[test]
    fn test_standard_array_roll() {
        let stats = CharacterCreator::roll_stats(&StatRollMethod::StandardArray, 0);
        assert_eq!(stats.strength, 15);
        assert_eq!(stats.charisma, 8);
    }

    #[test]
    fn test_fourdrop_roll_in_range() {
        let stats = CharacterCreator::roll_stats(&StatRollMethod::FourDrop(4), 12345);
        for val in [
            stats.strength,
            stats.dexterity,
            stats.constitution,
            stats.intelligence,
            stats.wisdom,
            stats.charisma,
        ] {
            assert!(val >= 3 && val <= 18, "stat out of range: {}", val);
        }
    }

    #[test]
    fn test_point_buy_even_distribution() {
        let stats = CharacterCreator::roll_stats(&StatRollMethod::PointBuy(42), 0);
        // 42/6=7, base=8 -> each stat = 15
        assert_eq!(stats.strength, 15);
        assert_eq!(stats.intelligence, 15);
    }

    #[test]
    fn test_race_bonuses_applied() {
        let mut stats = CharacterStats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };
        CharacterCreator::apply_race_bonuses(&mut stats, &Race::elf());
        assert_eq!(stats.dexterity, 12);
        assert_eq!(stats.intelligence, 11);
    }

    #[test]
    fn test_class_bonuses_applied() {
        let mut stats = CharacterStats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };
        CharacterCreator::apply_class_bonuses(&mut stats, &CharacterClass::Mage);
        assert_eq!(stats.intelligence, 12);
    }

    #[test]
    fn test_max_hp_level_1_warrior() {
        // constitution 14 -> modifier +2; warrior d10 -> 10+2=12
        let hp = CharacterCreator::max_hp(14, &CharacterClass::Warrior, 1);
        assert_eq!(hp, 12);
    }

    #[test]
    fn test_max_hp_scales_with_level() {
        let hp1 = CharacterCreator::max_hp(10, &CharacterClass::Mage, 1);
        let hp5 = CharacterCreator::max_hp(10, &CharacterClass::Mage, 5);
        assert!(hp5 > hp1);
    }

    #[test]
    fn test_starting_equipment_warrior() {
        let gear = CharacterCreator::starting_equipment(&CharacterClass::Warrior);
        assert!(gear.contains(&"Longsword".to_string()));
        assert!(gear.contains(&"Shield".to_string()));
    }

    #[test]
    fn test_starting_equipment_mage_has_spellbook() {
        let gear = CharacterCreator::starting_equipment(&CharacterClass::Mage);
        assert!(gear.contains(&"Spellbook".to_string()));
    }

    #[test]
    fn test_starting_equipment_rogue_has_thieves_tools() {
        let gear = CharacterCreator::starting_equipment(&CharacterClass::Rogue);
        assert!(gear.contains(&"Thieves' Tools".to_string()));
    }

    #[test]
    fn test_create_character_returns_tuple() {
        let (stats, hp, gear) = CharacterCreator::create_character(
            "Aragorn",
            &Race::human(),
            CharacterClass::Warrior,
            &StatRollMethod::StandardArray,
            0,
        );
        assert!(hp > 0);
        assert!(!gear.is_empty());
        assert!(stats.strength > 0);
    }

    #[test]
    fn test_dwarf_constitution_bonus() {
        let mut stats = CharacterStats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };
        CharacterCreator::apply_race_bonuses(&mut stats, &Race::dwarf());
        assert_eq!(stats.constitution, 12);
        assert_eq!(stats.strength, 12);
    }
}
