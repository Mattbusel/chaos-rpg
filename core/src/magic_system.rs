//! Magic system: spell crafting, mana management, and magical effects.

use std::collections::HashMap;

/// Schools of magic available to spellcasters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MagicSchool {
    Evocation,
    Conjuration,
    Necromancy,
    Illusion,
    Transmutation,
    Divination,
    Enchantment,
    Abjuration,
}

/// The effect produced when a spell is cast.
#[derive(Debug, Clone, PartialEq)]
pub enum SpellEffect {
    Damage { amount: u32, element: String },
    Heal(u32),
    Buff { stat: String, bonus: i32, duration_turns: u32 },
    Summon(String),
    Teleport,
    Reveal,
}

/// Components required to cast a spell.
#[derive(Debug, Clone, PartialEq)]
pub enum SpellComponent {
    Verbal,
    Somatic,
    Material(String),
    Focus(String),
}

/// A spell definition.
#[derive(Debug, Clone)]
pub struct Spell {
    pub id: u32,
    pub name: String,
    pub school: MagicSchool,
    pub mana_cost: u32,
    pub cast_time_turns: u32,
    pub range: u32,
    pub duration_turns: u32,
    pub effects: Vec<SpellEffect>,
    pub components: Vec<SpellComponent>,
    pub level: u8,
}

/// A caster's mana pool with regeneration.
#[derive(Debug, Clone)]
pub struct ManaPool {
    pub current: u32,
    pub maximum: u32,
    pub regen_per_turn: u32,
}

impl ManaPool {
    /// Create a new mana pool at full capacity.
    pub fn new(max: u32, regen: u32) -> Self {
        Self { current: max, maximum: max, regen_per_turn: regen }
    }

    /// Attempt to consume `amount` mana. Returns `true` on success.
    pub fn consume(&mut self, amount: u32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// Regenerate mana by `regen_per_turn`, capped at maximum.
    pub fn regenerate(&mut self) {
        self.current = (self.current + self.regen_per_turn).min(self.maximum);
    }

    /// Returns `true` when current mana is zero.
    pub fn is_depleted(&self) -> bool {
        self.current == 0
    }

    /// Returns current mana as a fraction of maximum [0.0, 1.0].
    pub fn fill_pct(&self) -> f32 {
        if self.maximum == 0 {
            return 0.0;
        }
        self.current as f32 / self.maximum as f32
    }
}

/// Result produced after attempting to cast a spell.
#[derive(Debug, Clone)]
pub struct SpellcastResult {
    pub success: bool,
    pub mana_used: u32,
    pub effects_applied: Vec<SpellEffect>,
    pub concentration_required: bool,
}

/// A collection of known spells, keyed by id.
#[derive(Debug, Clone)]
pub struct SpellBook {
    pub spells: HashMap<u32, Spell>,
    pub next_id: u32,
}

impl SpellBook {
    pub fn new() -> Self {
        Self { spells: HashMap::new(), next_id: 1 }
    }

    /// Add a spell; assigns and returns its new id.
    pub fn add_spell(&mut self, mut spell: Spell) -> u32 {
        let id = self.next_id;
        spell.id = id;
        self.spells.insert(id, spell);
        self.next_id += 1;
        id
    }

    pub fn get_spell(&self, id: u32) -> Option<&Spell> {
        self.spells.get(&id)
    }

    /// All spells belonging to the given school.
    pub fn spells_of_school(&self, school: &MagicSchool) -> Vec<&Spell> {
        self.spells.values().filter(|s| &s.school == school).collect()
    }

    /// Spells whose mana cost does not exceed `mana`.
    pub fn affordable_spells(&self, mana: u32) -> Vec<&Spell> {
        self.spells.values().filter(|s| s.mana_cost <= mana).collect()
    }
}

impl Default for SpellBook {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during spellcasting.
#[derive(Debug, Clone, PartialEq)]
pub enum MagicError {
    InsufficientMana { needed: u32, have: u32 },
    SpellNotFound,
    AlreadyConcentrating,
    ComponentMissing(String),
}

impl std::fmt::Display for MagicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MagicError::InsufficientMana { needed, have } =>
                write!(f, "Insufficient mana: need {needed}, have {have}"),
            MagicError::SpellNotFound => write!(f, "Spell not found"),
            MagicError::AlreadyConcentrating => write!(f, "Already concentrating on a spell"),
            MagicError::ComponentMissing(c) => write!(f, "Missing component: {c}"),
        }
    }
}

impl std::error::Error for MagicError {}

/// The top-level magic system: holds a spellbook, mana pool, and concentration state.
pub struct MagicSystem {
    pub spellbook: SpellBook,
    pub mana_pool: ManaPool,
    /// Id of the spell currently being concentrated upon, if any.
    pub concentration_spell: Option<u32>,
}

impl MagicSystem {
    pub fn new(max_mana: u32, regen: u32) -> Self {
        Self {
            spellbook: SpellBook::new(),
            mana_pool: ManaPool::new(max_mana, regen),
            concentration_spell: None,
        }
    }

    /// Attempt to cast a spell. Concentration spells replace the current one.
    pub fn cast_spell(&mut self, spell_id: u32) -> Result<SpellcastResult, MagicError> {
        let spell = self.spellbook.get_spell(spell_id)
            .ok_or(MagicError::SpellNotFound)?
            .clone();

        if self.mana_pool.current < spell.mana_cost {
            return Err(MagicError::InsufficientMana {
                needed: spell.mana_cost,
                have: self.mana_pool.current,
            });
        }

        // Determine if this is a concentration spell (duration > 1 turn and has non-instant effects).
        let concentration_required = spell.duration_turns > 1;

        // Replace concentration if needed — no error, just swap.
        if concentration_required {
            self.concentration_spell = Some(spell_id);
        }

        self.mana_pool.consume(spell.mana_cost);

        Ok(SpellcastResult {
            success: true,
            mana_used: spell.mana_cost,
            effects_applied: spell.effects.clone(),
            concentration_required,
        })
    }

    /// Drop the current concentration spell.
    pub fn end_concentration(&mut self) {
        self.concentration_spell = None;
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_system() -> MagicSystem {
        MagicSystem::new(100, 5)
    }

    fn fireball() -> Spell {
        Spell {
            id: 0,
            name: "Fireball".to_string(),
            school: MagicSchool::Evocation,
            mana_cost: 30,
            cast_time_turns: 1,
            range: 20,
            duration_turns: 1,
            effects: vec![SpellEffect::Damage { amount: 50, element: "fire".to_string() }],
            components: vec![SpellComponent::Verbal, SpellComponent::Somatic],
            level: 3,
        }
    }

    fn concentration_spell() -> Spell {
        Spell {
            id: 0,
            name: "Haste".to_string(),
            school: MagicSchool::Transmutation,
            mana_cost: 20,
            cast_time_turns: 1,
            range: 5,
            duration_turns: 10,
            effects: vec![SpellEffect::Buff { stat: "speed".to_string(), bonus: 10, duration_turns: 10 }],
            components: vec![SpellComponent::Verbal],
            level: 3,
        }
    }

    #[test]
    fn cast_consumes_mana() {
        let mut sys = make_system();
        let id = sys.spellbook.add_spell(fireball());
        let result = sys.cast_spell(id).unwrap();
        assert!(result.success);
        assert_eq!(result.mana_used, 30);
        assert_eq!(sys.mana_pool.current, 70);
    }

    #[test]
    fn insufficient_mana_error() {
        let mut sys = MagicSystem::new(10, 1);
        let id = sys.spellbook.add_spell(fireball());
        let err = sys.cast_spell(id).unwrap_err();
        assert_eq!(err, MagicError::InsufficientMana { needed: 30, have: 10 });
    }

    #[test]
    fn mana_regeneration() {
        let mut pool = ManaPool::new(100, 5);
        pool.consume(20);
        assert_eq!(pool.current, 80);
        pool.regenerate();
        assert_eq!(pool.current, 85);
        // Does not exceed maximum.
        pool.current = 98;
        pool.regenerate();
        assert_eq!(pool.current, 100);
    }

    #[test]
    fn concentration_replaces_on_new_spell() {
        let mut sys = make_system();
        let haste_id = sys.spellbook.add_spell(concentration_spell());
        let mut haste2 = concentration_spell();
        haste2.name = "Slow".to_string();
        let slow_id = sys.spellbook.add_spell(haste2);

        sys.cast_spell(haste_id).unwrap();
        assert_eq!(sys.concentration_spell, Some(haste_id));

        sys.cast_spell(slow_id).unwrap();
        // Replaced by the new concentration spell.
        assert_eq!(sys.concentration_spell, Some(slow_id));
    }

    #[test]
    fn spells_by_school_filter() {
        let mut book = SpellBook::new();
        book.add_spell(fireball());
        let mut c = concentration_spell();
        c.school = MagicSchool::Conjuration;
        book.add_spell(c);

        let evoc = book.spells_of_school(&MagicSchool::Evocation);
        assert_eq!(evoc.len(), 1);
        assert_eq!(evoc[0].name, "Fireball");

        let conj = book.spells_of_school(&MagicSchool::Conjuration);
        assert_eq!(conj.len(), 1);

        let necro = book.spells_of_school(&MagicSchool::Necromancy);
        assert!(necro.is_empty());
    }
}
