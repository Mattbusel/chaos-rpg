//! Spell casting and mana system for CHAOS RPG.
//!
//! Provides a complete magic system including spell definitions, a spellbook
//! for tracking known spells and cooldowns, and a mana pool for resource management.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ─── SPELL SCHOOL ─────────────────────────────────────────────────────────────

/// The magical school a spell belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpellSchool {
    Fire,
    Ice,
    Lightning,
    Arcane,
    Nature,
    Shadow,
}

// ─── SPELL EFFECT ─────────────────────────────────────────────────────────────

/// The effect produced when a spell is cast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellEffect {
    Damage { base: u32, variance: u32 },
    Heal { amount: u32 },
    Buff { effect_id: String, duration_turns: u32 },
    Summon { entity: String },
    Teleport,
    AoE { radius: u32, damage: u32 },
}

// ─── SPELL ────────────────────────────────────────────────────────────────────

/// A single spell that can be learned and cast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub id: String,
    pub name: String,
    pub school: SpellSchool,
    pub mana_cost: u32,
    pub cooldown_turns: u32,
    pub cast_time_turns: u32,
    pub range: u32,
    pub effects: Vec<SpellEffect>,
    pub description: String,
}

impl Spell {
    /// Classic fireball — high damage, moderate variance.
    pub fn fireball() -> Self {
        Self {
            id: "fireball".to_string(),
            name: "Fireball".to_string(),
            school: SpellSchool::Fire,
            mana_cost: 30,
            cooldown_turns: 3,
            cast_time_turns: 1,
            range: 8,
            effects: vec![SpellEffect::Damage { base: 40, variance: 20 }],
            description: "Hurls a blazing sphere of fire at the target.".to_string(),
        }
    }

    /// Frost bolt — moderate damage, slows the target.
    pub fn frost_bolt() -> Self {
        Self {
            id: "frost_bolt".to_string(),
            name: "Frost Bolt".to_string(),
            school: SpellSchool::Ice,
            mana_cost: 20,
            cooldown_turns: 2,
            cast_time_turns: 1,
            range: 10,
            effects: vec![
                SpellEffect::Damage { base: 25, variance: 10 },
                SpellEffect::Buff { effect_id: "slowed".to_string(), duration_turns: 2 },
            ],
            description: "A shard of ice that chills and slows the target.".to_string(),
        }
    }

    /// Chain lightning — fast, high damage.
    pub fn lightning() -> Self {
        Self {
            id: "lightning".to_string(),
            name: "Chain Lightning".to_string(),
            school: SpellSchool::Lightning,
            mana_cost: 40,
            cooldown_turns: 4,
            cast_time_turns: 1,
            range: 12,
            effects: vec![SpellEffect::Damage { base: 60, variance: 30 }],
            description: "A bolt of lightning that arcs between enemies.".to_string(),
        }
    }

    /// Heal — restores health points.
    pub fn heal() -> Self {
        Self {
            id: "heal".to_string(),
            name: "Healing Light".to_string(),
            school: SpellSchool::Nature,
            mana_cost: 25,
            cooldown_turns: 2,
            cast_time_turns: 1,
            range: 0,
            effects: vec![SpellEffect::Heal { amount: 35 }],
            description: "Channels restorative energy to mend wounds.".to_string(),
        }
    }

    /// Blink — instant teleport to nearby location.
    pub fn blink() -> Self {
        Self {
            id: "blink".to_string(),
            name: "Blink".to_string(),
            school: SpellSchool::Arcane,
            mana_cost: 15,
            cooldown_turns: 3,
            cast_time_turns: 0,
            range: 5,
            effects: vec![SpellEffect::Teleport],
            description: "Instantly teleports the caster a short distance.".to_string(),
        }
    }

    /// Blizzard — AoE ice storm.
    pub fn blizzard() -> Self {
        Self {
            id: "blizzard".to_string(),
            name: "Blizzard".to_string(),
            school: SpellSchool::Ice,
            mana_cost: 60,
            cooldown_turns: 5,
            cast_time_turns: 2,
            range: 15,
            effects: vec![SpellEffect::AoE { radius: 4, damage: 30 }],
            description: "Calls down a storm of ice and snow over a wide area.".to_string(),
        }
    }
}

// ─── SPELL ERROR ──────────────────────────────────────────────────────────────

/// Errors that can occur during spell operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpellError {
    #[error("unknown spell: {0}")]
    UnknownSpell(String),
    #[error("insufficient mana: have {have}, need {need}")]
    InsufficientMana { have: u32, need: u32 },
    #[error("spell is on cooldown: {turns_remaining} turns remaining")]
    OnCooldown { turns_remaining: u32 },
    #[error("spell already known")]
    SpellAlreadyKnown,
}

// ─── SPELL BOOK ───────────────────────────────────────────────────────────────

/// Tracks known spells and their cooldowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellBook {
    pub known_spells: Vec<Spell>,
    pub cooldowns: HashMap<String, u32>,
}

impl SpellBook {
    /// Create an empty spellbook.
    pub fn new() -> Self {
        Self {
            known_spells: Vec::new(),
            cooldowns: HashMap::new(),
        }
    }

    /// Learn a new spell. Returns an error if the spell is already known.
    pub fn learn(&mut self, spell: Spell) -> Result<(), SpellError> {
        if self.known_spells.iter().any(|s| s.id == spell.id) {
            return Err(SpellError::SpellAlreadyKnown);
        }
        self.known_spells.push(spell);
        Ok(())
    }

    /// Check whether a spell can be cast given current mana.
    pub fn can_cast(&self, spell_id: &str, mana: u32) -> Result<(), SpellError> {
        let spell = self.known_spells.iter()
            .find(|s| s.id == spell_id)
            .ok_or_else(|| SpellError::UnknownSpell(spell_id.to_string()))?;

        if let Some(&remaining) = self.cooldowns.get(spell_id) {
            if remaining > 0 {
                return Err(SpellError::OnCooldown { turns_remaining: remaining });
            }
        }

        if mana < spell.mana_cost {
            return Err(SpellError::InsufficientMana { have: mana, need: spell.mana_cost });
        }

        Ok(())
    }

    /// Begin casting a spell: starts its cooldown timer. The caller is responsible
    /// for spending mana via [`ManaPool::spend`].
    pub fn start_cast(&mut self, spell_id: &str) -> Result<&Spell, SpellError> {
        let idx = self.known_spells.iter()
            .position(|s| s.id == spell_id)
            .ok_or_else(|| SpellError::UnknownSpell(spell_id.to_string()))?;

        let cooldown = self.known_spells[idx].cooldown_turns;
        if cooldown > 0 {
            self.cooldowns.insert(spell_id.to_string(), cooldown);
        }

        Ok(&self.known_spells[idx])
    }

    /// Decrement all active cooldowns by 1, removing any that reach 0.
    pub fn tick_cooldowns(&mut self) {
        for val in self.cooldowns.values_mut() {
            if *val > 0 {
                *val -= 1;
            }
        }
        self.cooldowns.retain(|_, v| *v > 0);
    }

    /// Return all spells that are not on cooldown and affordable with `mana`.
    pub fn available_spells(&self, mana: u32) -> Vec<&Spell> {
        self.known_spells.iter()
            .filter(|s| {
                let on_cooldown = self.cooldowns.get(&s.id).copied().unwrap_or(0) > 0;
                !on_cooldown && s.mana_cost <= mana
            })
            .collect()
    }
}

impl Default for SpellBook {
    fn default() -> Self {
        Self::new()
    }
}

// ─── MANA POOL ────────────────────────────────────────────────────────────────

/// Manages a caster's mana resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaPool {
    pub current: u32,
    pub max: u32,
    pub regen_per_turn: u32,
}

impl ManaPool {
    /// Create a new mana pool at full capacity.
    pub fn new(max: u32, regen_per_turn: u32) -> Self {
        Self { current: max, max, regen_per_turn }
    }

    /// Spend mana. Returns [`SpellError::InsufficientMana`] if not enough.
    pub fn spend(&mut self, amount: u32) -> Result<(), SpellError> {
        if self.current < amount {
            return Err(SpellError::InsufficientMana { have: self.current, need: amount });
        }
        self.current -= amount;
        Ok(())
    }

    /// Regenerate mana at the configured rate (capped at max).
    pub fn regenerate(&mut self) {
        self.current = (self.current + self.regen_per_turn).min(self.max);
    }

    /// Restore a fixed amount of mana (capped at max).
    pub fn restore(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn book_with_fireball() -> SpellBook {
        let mut book = SpellBook::new();
        book.learn(Spell::fireball()).unwrap();
        book
    }

    #[test]
    fn learn_and_cast() {
        let mut book = book_with_fireball();
        let mut mana = ManaPool::new(100, 5);

        // Can cast when mana is sufficient and not on cooldown.
        assert!(book.can_cast("fireball", mana.current).is_ok());

        // Start cast, spend mana.
        let cost = {
            let spell = book.start_cast("fireball").unwrap();
            spell.mana_cost
        };
        mana.spend(cost).unwrap();

        // Cooldown should now be active.
        assert!(book.cooldowns.contains_key("fireball"));
    }

    #[test]
    fn cooldown_countdown() {
        let mut book = book_with_fireball();
        let mut mana = ManaPool::new(200, 10);

        // Cast the spell.
        let cost = {
            let spell = book.start_cast("fireball").unwrap();
            spell.mana_cost
        };
        mana.spend(cost).unwrap();

        let initial_cd = *book.cooldowns.get("fireball").unwrap();
        assert!(initial_cd > 0);

        // Tick down to zero.
        for _ in 0..initial_cd {
            book.tick_cooldowns();
        }
        assert!(!book.cooldowns.contains_key("fireball"));
    }

    #[test]
    fn insufficient_mana_error() {
        let book = book_with_fireball();
        let result = book.can_cast("fireball", 5); // fireball costs 30
        assert!(matches!(result, Err(SpellError::InsufficientMana { have: 5, need: 30 })));
    }

    #[test]
    fn available_spells_filters_correctly() {
        let mut book = SpellBook::new();
        book.learn(Spell::fireball()).unwrap();  // costs 30
        book.learn(Spell::heal()).unwrap();      // costs 25
        book.learn(Spell::blizzard()).unwrap();  // costs 60

        // Start fireball cast to put it on cooldown.
        book.start_cast("fireball").unwrap();

        // With 50 mana: heal is available (25), fireball on cooldown, blizzard too expensive.
        let available = book.available_spells(50);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "heal");
    }

    #[test]
    fn spell_already_known_error() {
        let mut book = book_with_fireball();
        let result = book.learn(Spell::fireball());
        assert!(matches!(result, Err(SpellError::SpellAlreadyKnown)));
    }

    #[test]
    fn mana_pool_regen_and_cap() {
        let mut pool = ManaPool::new(100, 10);
        pool.spend(50).unwrap();
        assert_eq!(pool.current, 50);
        pool.regenerate();
        assert_eq!(pool.current, 60);
        pool.restore(100); // should cap at max
        assert_eq!(pool.current, 100);
    }
}
