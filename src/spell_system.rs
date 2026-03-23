//! Spell system — mana, cooldowns, schools, and AoE effects.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for spell casting operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpellError {
    #[error("spell '{0}' not found in spellbook")]
    SpellNotFound(String),
    #[error("insufficient mana: need {need}, have {have}")]
    InsufficientMana { need: u32, have: u32 },
    #[error("spell '{0}' is on cooldown for {1} more turns")]
    OnCooldown(String, u32),
}

/// The school of magic a spell belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellSchool {
    Evocation,
    Conjuration,
    Transmutation,
    Illusion,
    Necromancy,
    Divination,
}

impl SpellSchool {
    /// Mana cost multiplier for spells of this school.
    pub fn mana_multiplier(&self) -> f64 {
        match self {
            SpellSchool::Evocation => 1.0,
            SpellSchool::Conjuration => 1.2,
            SpellSchool::Transmutation => 1.1,
            SpellSchool::Illusion => 0.9,
            SpellSchool::Necromancy => 1.3,
            SpellSchool::Divination => 0.7,
        }
    }
}

/// The effect produced when a spell is cast.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellEffect {
    /// Deal damage using a dice roll plus bonus.
    Damage { dice: (u8, u8), bonus: i32 },
    /// Heal hit points using a dice roll.
    Heal { dice: (u8, u8) },
    /// Buff a stat by a flat amount for a number of turns.
    Buff {
        stat: String,
        amount: i32,
        duration_turns: u32,
    },
    /// Area-of-effect damage in a radius.
    AoE { radius: u32, damage_dice: (u8, u8) },
    /// Inflict a named status condition.
    StatusInflict(String),
    /// Teleport the caster up to max_range squares.
    Teleport { max_range: u32 },
}

/// A spell that can be placed in a spellbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub id: String,
    pub name: String,
    pub school: SpellSchool,
    pub mana_cost: u32,
    pub cooldown_turns: u32,
    pub range: u32,
    pub effects: Vec<SpellEffect>,
    pub description: String,
}

impl Spell {
    /// Effective mana cost after applying school multiplier.
    pub fn effective_mana_cost(&self) -> u32 {
        ((self.mana_cost as f64) * self.school.mana_multiplier()).ceil() as u32
    }
}

/// A collection of known spells plus runtime mana and cooldown state.
#[derive(Debug, Default)]
pub struct SpellBook {
    pub spells: HashMap<String, Spell>,
    pub mana: u32,
    pub max_mana: u32,
    /// Remaining cooldown turns keyed by spell ID.
    pub cooldowns: HashMap<String, u32>,
}

/// Simple seeded LCG for deterministic rolls inside the spell system.
fn lcg_roll(seed: u64, sides: u8) -> u32 {
    if sides == 0 {
        return 0;
    }
    let val = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((val >> 33) as u32) % (sides as u32) + 1
}

impl SpellBook {
    /// Create a new spellbook with given mana pool.
    pub fn new(max_mana: u32) -> Self {
        Self {
            spells: HashMap::new(),
            mana: max_mana,
            max_mana,
            cooldowns: HashMap::new(),
        }
    }

    /// Add a spell to the spellbook.
    pub fn learn(&mut self, spell: Spell) {
        self.spells.insert(spell.id.clone(), spell);
    }

    /// Cast a spell by ID. Returns the resolved effects on success.
    pub fn cast(&mut self, spell_id: &str, seed: u64) -> Result<Vec<SpellEffect>, SpellError> {
        let spell = self
            .spells
            .get(spell_id)
            .ok_or_else(|| SpellError::SpellNotFound(spell_id.to_string()))?
            .clone();

        // Check cooldown.
        if let Some(&remaining) = self.cooldowns.get(spell_id) {
            if remaining > 0 {
                return Err(SpellError::OnCooldown(spell_id.to_string(), remaining));
            }
        }

        let cost = spell.effective_mana_cost();
        if self.mana < cost {
            return Err(SpellError::InsufficientMana {
                need: cost,
                have: self.mana,
            });
        }

        self.mana -= cost;
        if spell.cooldown_turns > 0 {
            self.cooldowns
                .insert(spell_id.to_string(), spell.cooldown_turns);
        }

        // Resolve effects (roll dice for Damage / Heal / AoE).
        let mut resolved = Vec::new();
        for (i, effect) in spell.effects.iter().enumerate() {
            let effect_seed = seed.wrapping_add(i as u64 * 31337);
            let resolved_effect = match effect {
                SpellEffect::Damage { dice: (count, sides), bonus } => {
                    let total: u32 = (0..*count)
                        .map(|r| lcg_roll(effect_seed.wrapping_add(r as u64), *sides))
                        .sum();
                    SpellEffect::Damage {
                        dice: (*count, *sides),
                        bonus: total as i32 + bonus,
                    }
                }
                SpellEffect::Heal { dice: (count, sides) } => {
                    let total: u32 = (0..*count)
                        .map(|r| lcg_roll(effect_seed.wrapping_add(r as u64), *sides))
                        .sum();
                    SpellEffect::Heal {
                        dice: (*count, total as u8),
                    }
                }
                other => other.clone(),
            };
            resolved.push(resolved_effect);
        }

        Ok(resolved)
    }

    /// Decrement all active cooldowns by one turn.
    pub fn tick_cooldowns(&mut self) {
        for val in self.cooldowns.values_mut() {
            *val = val.saturating_sub(1);
        }
    }

    /// Restore mana up to max.
    pub fn regenerate_mana(&mut self, amount: u32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }

    /// Return spells that are off cooldown and affordable.
    pub fn available_spells(&self) -> Vec<&Spell> {
        self.spells
            .values()
            .filter(|s| {
                let cd = self.cooldowns.get(&s.id).copied().unwrap_or(0);
                cd == 0 && self.mana >= s.effective_mana_cost()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fireball() -> Spell {
        Spell {
            id: "fireball".to_string(),
            name: "Fireball".to_string(),
            school: SpellSchool::Evocation,
            mana_cost: 10,
            cooldown_turns: 2,
            range: 6,
            effects: vec![SpellEffect::Damage {
                dice: (3, 6),
                bonus: 0,
            }],
            description: "A classic explosion of fire.".to_string(),
        }
    }

    fn heal_spell() -> Spell {
        Spell {
            id: "cure".to_string(),
            name: "Cure Wounds".to_string(),
            school: SpellSchool::Divination,
            mana_cost: 5,
            cooldown_turns: 1,
            range: 1,
            effects: vec![SpellEffect::Heal { dice: (2, 8) }],
            description: "Restore hit points.".to_string(),
        }
    }

    #[test]
    fn test_spell_school_multipliers() {
        assert!((SpellSchool::Evocation.mana_multiplier() - 1.0).abs() < 1e-9);
        assert!(SpellSchool::Necromancy.mana_multiplier() > 1.0);
        assert!(SpellSchool::Divination.mana_multiplier() < 1.0);
    }

    #[test]
    fn test_learn_and_cast() {
        let mut book = SpellBook::new(50);
        book.learn(fireball());
        let result = book.cast("fireball", 42).unwrap();
        assert_eq!(result.len(), 1);
        assert!(book.mana < 50);
    }

    #[test]
    fn test_cast_unknown_spell() {
        let mut book = SpellBook::new(50);
        let err = book.cast("unknown", 1).unwrap_err();
        assert!(matches!(err, SpellError::SpellNotFound(_)));
    }

    #[test]
    fn test_insufficient_mana() {
        let mut book = SpellBook::new(5);
        book.learn(fireball());
        let err = book.cast("fireball", 1).unwrap_err();
        assert!(matches!(err, SpellError::InsufficientMana { .. }));
    }

    #[test]
    fn test_cooldown_enforced() {
        let mut book = SpellBook::new(100);
        book.learn(fireball());
        book.cast("fireball", 1).unwrap();
        let err = book.cast("fireball", 2).unwrap_err();
        assert!(matches!(err, SpellError::OnCooldown(_, _)));
    }

    #[test]
    fn test_tick_reduces_cooldown() {
        let mut book = SpellBook::new(100);
        book.learn(fireball());
        book.cast("fireball", 1).unwrap();
        book.tick_cooldowns();
        book.tick_cooldowns();
        // After 2 ticks the cooldown should be gone.
        assert!(book.cast("fireball", 2).is_ok());
    }

    #[test]
    fn test_regenerate_mana_capped_at_max() {
        let mut book = SpellBook::new(20);
        book.mana = 10;
        book.regenerate_mana(100);
        assert_eq!(book.mana, 20);
    }

    #[test]
    fn test_available_spells_empty_when_no_mana() {
        let mut book = SpellBook::new(0);
        book.learn(fireball());
        assert!(book.available_spells().is_empty());
    }

    #[test]
    fn test_available_spells_excludes_on_cooldown() {
        let mut book = SpellBook::new(100);
        book.learn(fireball());
        book.cast("fireball", 1).unwrap();
        let avail = book.available_spells();
        assert!(avail.is_empty());
    }

    #[test]
    fn test_aoe_effect_stored() {
        let mut book = SpellBook::new(100);
        book.learn(Spell {
            id: "nova".to_string(),
            name: "Nova".to_string(),
            school: SpellSchool::Evocation,
            mana_cost: 15,
            cooldown_turns: 3,
            range: 4,
            effects: vec![SpellEffect::AoE {
                radius: 3,
                damage_dice: (4, 6),
            }],
            description: "Blast everything nearby.".to_string(),
        });
        let res = book.cast("nova", 7).unwrap();
        assert!(matches!(res[0], SpellEffect::AoE { .. }));
    }

    #[test]
    fn test_effective_mana_cost_school_multiplier() {
        let spell = Spell {
            id: "x".to_string(),
            name: "X".to_string(),
            school: SpellSchool::Necromancy, // 1.3x
            mana_cost: 10,
            cooldown_turns: 0,
            range: 1,
            effects: vec![],
            description: "".to_string(),
        };
        assert_eq!(spell.effective_mana_cost(), 13);
    }

    #[test]
    fn test_status_inflict_effect() {
        let mut book = SpellBook::new(50);
        book.learn(Spell {
            id: "curse".to_string(),
            name: "Curse".to_string(),
            school: SpellSchool::Necromancy,
            mana_cost: 5,
            cooldown_turns: 0,
            range: 3,
            effects: vec![SpellEffect::StatusInflict("Poisoned".to_string())],
            description: "Poison the target.".to_string(),
        });
        let res = book.cast("curse", 0).unwrap();
        assert!(matches!(res[0], SpellEffect::StatusInflict(_)));
    }

    #[test]
    fn test_heal_spell_casts() {
        let mut book = SpellBook::new(50);
        book.learn(heal_spell());
        let res = book.cast("cure", 99).unwrap();
        assert!(matches!(res[0], SpellEffect::Heal { .. }));
    }

    #[test]
    fn test_buff_effect_passthrough() {
        let mut book = SpellBook::new(50);
        book.learn(Spell {
            id: "haste".to_string(),
            name: "Haste".to_string(),
            school: SpellSchool::Transmutation,
            mana_cost: 8,
            cooldown_turns: 0,
            range: 1,
            effects: vec![SpellEffect::Buff {
                stat: "dexterity".to_string(),
                amount: 4,
                duration_turns: 3,
            }],
            description: "Increase dexterity.".to_string(),
        });
        let res = book.cast("haste", 0).unwrap();
        assert!(matches!(res[0], SpellEffect::Buff { .. }));
    }

    #[test]
    fn test_teleport_effect_passthrough() {
        let mut book = SpellBook::new(50);
        book.learn(Spell {
            id: "blink".to_string(),
            name: "Blink".to_string(),
            school: SpellSchool::Conjuration,
            mana_cost: 6,
            cooldown_turns: 1,
            range: 0,
            effects: vec![SpellEffect::Teleport { max_range: 8 }],
            description: "Short-range teleport.".to_string(),
        });
        let res = book.cast("blink", 0).unwrap();
        assert!(matches!(res[0], SpellEffect::Teleport { .. }));
    }

    #[test]
    fn test_multiple_spells_in_book() {
        let mut book = SpellBook::new(200);
        book.learn(fireball());
        book.learn(heal_spell());
        assert_eq!(book.spells.len(), 2);
        let avail = book.available_spells();
        assert_eq!(avail.len(), 2);
    }
}
