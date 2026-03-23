//! Monster bestiary with AI behaviors for CHAOS RPG.

use std::collections::HashMap;

/// Creature size categories.
#[derive(Debug, Clone, PartialEq)]
pub enum CreatureSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

/// Creature type categories.
#[derive(Debug, Clone, PartialEq)]
pub enum CreatureType {
    Beast,
    Undead,
    Humanoid,
    Dragon,
    Elemental,
    Construct,
    Fiend,
    Celestial,
    Plant,
    Aberration,
}

/// The six core ability scores.
#[derive(Debug, Clone)]
pub struct AbilityScores {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl AbilityScores {
    /// Compute ability modifier: (score - 10) / 2 (floor division).
    pub fn modifier(score: u8) -> i8 {
        let s = score as i16;
        ((s - 10) / 2) as i8
    }

    /// Saving throw = ability modifier + proficiency bonus.
    pub fn saving_throw(ability: u8, proficiency: i8) -> i8 {
        Self::modifier(ability).saturating_add(proficiency)
    }
}

/// An entry in a creature's loot table.
#[derive(Debug, Clone)]
pub struct LootEntry {
    pub item_name: String,
    /// (min, max) inclusive quantity range.
    pub quantity_range: (u32, u32),
    /// Drop chance 0–100.
    pub drop_chance_pct: u8,
    pub gold_value: u32,
}

/// AI behavior mode for a creature.
#[derive(Debug, Clone)]
pub enum AiBehavior {
    Aggressive,
    Defensive,
    Skirmisher,
    Supporter,
    Coward { flee_threshold_hp_pct: f64 },
    Pack { min_allies: usize },
    Territorial { range: f64 },
    Ambush,
}

/// A single attack profile.
#[derive(Debug, Clone)]
pub struct AttackProfile {
    pub name: String,
    pub hit_bonus: i8,
    /// (count, sides) e.g. (2, 6) = 2d6.
    pub damage_dice: (u32, u32),
    pub damage_bonus: i32,
    pub reach_ft: u32,
}

/// Result of an attack roll.
#[derive(Debug, Clone)]
pub struct AttackResult {
    pub roll: u32,
    pub total: i32,
    pub is_hit: bool,
    pub is_critical: bool,
}

/// Action chosen by the AI.
#[derive(Debug, Clone)]
pub enum CombatAction {
    Attack { target_idx: usize },
    Flee,
    UseAbility(String),
    Defend,
    Retreat,
}

/// A creature instance.
#[derive(Debug, Clone)]
pub struct Creature {
    pub id: String,
    pub name: String,
    pub creature_type: CreatureType,
    pub size: CreatureSize,
    pub challenge_rating: f64,
    pub hp_max: u32,
    pub hp_current: u32,
    pub armor_class: u8,
    pub speed_ft: u32,
    pub ability_scores: AbilityScores,
    pub attacks: Vec<AttackProfile>,
    pub loot_table: Vec<LootEntry>,
    pub ai_behavior: AiBehavior,
    pub xp_reward: u32,
}

/// Simple LCG random helper (seed-based, no external deps).
fn lcg_next(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

fn lcg_range(seed: u64, lo: u64, hi: u64) -> (u64, u64) {
    let s = lcg_next(seed);
    let val = lo + (s % (hi - lo + 1));
    (val, s)
}

impl Creature {
    /// Returns true if the creature has positive HP.
    pub fn is_alive(&self) -> bool {
        self.hp_current > 0
    }

    /// Apply damage. Returns true if the creature is killed.
    pub fn take_damage(&mut self, dmg: u32) -> bool {
        if dmg >= self.hp_current {
            self.hp_current = 0;
            true
        } else {
            self.hp_current -= dmg;
            false
        }
    }

    /// Roll an attack for the given attack index. Uses seed for determinism.
    /// Returns an `AttackResult` with the d20 roll and whether it hits AC 10 (placeholder).
    pub fn attack_roll(&self, attack_idx: usize, seed: u64) -> AttackResult {
        let (roll, _) = lcg_range(seed, 1, 20);
        let roll = roll as u32;
        let is_critical = roll == 20;
        let atk = self.attacks.get(attack_idx);
        let hit_bonus = atk.map(|a| a.hit_bonus as i32).unwrap_or(0);
        let total = roll as i32 + hit_bonus;
        // is_hit if total >= 10 or critical (simplified; caller provides target AC)
        let is_hit = is_critical || total >= 10;
        AttackResult { roll, total, is_hit, is_critical }
    }

    /// Roll damage for the given attack index.
    pub fn damage_roll(&self, attack_idx: usize, seed: u64) -> u32 {
        let atk = match self.attacks.get(attack_idx) {
            Some(a) => a,
            None => return 0,
        };
        let (count, sides) = atk.damage_dice;
        let mut total: i32 = 0;
        let mut s = seed;
        for _ in 0..count {
            let (roll, ns) = lcg_range(s, 1, sides as u64);
            total += roll as i32;
            s = ns;
        }
        total = total + atk.damage_bonus;
        if total < 0 { 0 } else { total as u32 }
    }

    /// Decide what action to take based on AI behavior, allies, and HP.
    pub fn decide_action(&self, allies_nearby: usize, hp_pct: f64) -> CombatAction {
        match &self.ai_behavior {
            AiBehavior::Aggressive => CombatAction::Attack { target_idx: 0 },
            AiBehavior::Defensive => {
                if hp_pct < 0.3 {
                    CombatAction::Defend
                } else {
                    CombatAction::Attack { target_idx: 0 }
                }
            }
            AiBehavior::Skirmisher => {
                if hp_pct < 0.5 {
                    CombatAction::Retreat
                } else {
                    CombatAction::Attack { target_idx: 0 }
                }
            }
            AiBehavior::Supporter => {
                if allies_nearby > 0 {
                    CombatAction::UseAbility("support".to_string())
                } else {
                    CombatAction::Attack { target_idx: 0 }
                }
            }
            AiBehavior::Coward { flee_threshold_hp_pct } => {
                if hp_pct <= *flee_threshold_hp_pct {
                    CombatAction::Flee
                } else {
                    CombatAction::Attack { target_idx: 0 }
                }
            }
            AiBehavior::Pack { min_allies } => {
                if allies_nearby >= *min_allies {
                    CombatAction::Attack { target_idx: 0 }
                } else {
                    CombatAction::Retreat
                }
            }
            AiBehavior::Territorial { .. } => CombatAction::Attack { target_idx: 0 },
            AiBehavior::Ambush => CombatAction::UseAbility("ambush_strike".to_string()),
        }
    }

    /// Generate loot drops based on loot table, using seed for randomness.
    pub fn generate_loot(&self, seed: u64) -> Vec<(String, u32)> {
        let mut result = Vec::new();
        let mut s = seed;
        for entry in &self.loot_table {
            let (roll, ns) = lcg_range(s, 1, 100);
            s = ns;
            if roll <= entry.drop_chance_pct as u64 {
                let (qty, ns2) = lcg_range(
                    s,
                    entry.quantity_range.0 as u64,
                    entry.quantity_range.1 as u64,
                );
                s = ns2;
                result.push((entry.item_name.clone(), qty as u32));
            }
        }
        result
    }
}

/// A collection of creature templates.
#[derive(Debug, Default)]
pub struct Bestiary {
    creatures: HashMap<String, Creature>,
}

impl Bestiary {
    /// Create an empty bestiary.
    pub fn new() -> Self {
        Self { creatures: HashMap::new() }
    }

    /// Add or replace a creature template.
    pub fn add_creature(&mut self, creature: Creature) {
        self.creatures.insert(creature.id.clone(), creature);
    }

    /// Look up a creature by id.
    pub fn get(&self, id: &str) -> Option<&Creature> {
        self.creatures.get(id)
    }

    /// Return all creatures with CR in [min_cr, max_cr].
    pub fn by_challenge_rating(&self, min_cr: f64, max_cr: f64) -> Vec<&Creature> {
        self.creatures
            .values()
            .filter(|c| c.challenge_rating >= min_cr && c.challenge_rating <= max_cr)
            .collect()
    }

    /// Greedy random encounter: pick `count` creatures whose total CR <= budget.
    pub fn random_encounter(&self, cr_budget: f64, count: usize, seed: u64) -> Vec<&Creature> {
        let mut pool: Vec<&Creature> = self.creatures.values().collect();
        // Sort by id for determinism before applying seed
        pool.sort_by(|a, b| a.id.cmp(&b.id));

        let mut selected: Vec<&Creature> = Vec::new();
        let mut remaining_budget = cr_budget;
        let mut s = seed;

        for _ in 0..count {
            let affordable: Vec<&Creature> =
                pool.iter().copied().filter(|c| c.challenge_rating <= remaining_budget).collect();
            if affordable.is_empty() {
                break;
            }
            let (idx, ns) = lcg_range(s, 0, affordable.len() as u64 - 1);
            s = ns;
            let chosen = affordable[idx as usize];
            remaining_budget -= chosen.challenge_rating;
            selected.push(chosen);
        }
        selected
    }

    /// Clone a creature template from the bestiary with full HP restored.
    pub fn spawn(&self, id: &str) -> Option<Creature> {
        self.creatures.get(id).map(|c| {
            let mut clone = c.clone();
            clone.hp_current = clone.hp_max;
            clone
        })
    }
}
