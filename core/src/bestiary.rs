//! Monster bestiary with AI behaviors for CHAOS RPG.

use std::collections::HashMap;

/// Creature size categories.
#[derive(Debug, Clone, PartialEq)]
pub enum CreatureSize {
    /// Smaller than a cat.
    Tiny,
    /// About child sized.
    Small,
    /// About human sized.
    Medium,
    /// Horse sized.
    Large,
    /// Giant sized.
    Huge,
    /// Larger than a giant.
    Gargantuan,
}

/// Creature type categories.
#[derive(Debug, Clone, PartialEq)]
pub enum CreatureType {
    /// Natural animals and monsters.
    Beast,
    /// Risen dead.
    Undead,
    /// People and humanlike creatures.
    Humanoid,
    /// Dragons.
    Dragon,
    /// Living elements such as fire or water.
    Elemental,
    /// Built or animated objects.
    Construct,
    /// Demons and devils.
    Fiend,
    /// Heavenly beings.
    Celestial,
    /// Living plants.
    Plant,
    /// Alien, unnatural creatures.
    Aberration,
}

/// The six core ability scores.
#[derive(Debug, Clone)]
pub struct AbilityScores {
    /// Strength score (raw physical power).
    pub strength: u8,
    /// Dexterity score (agility and reflexes).
    pub dexterity: u8,
    /// Constitution score (toughness).
    pub constitution: u8,
    /// Intelligence score.
    pub intelligence: u8,
    /// Wisdom score.
    pub wisdom: u8,
    /// Charisma score.
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
    /// Name of the item that can drop.
    pub item_name: String,
    /// (min, max) inclusive quantity range.
    pub quantity_range: (u32, u32),
    /// Drop chance 0–100.
    pub drop_chance_pct: u8,
    /// Value of one item in gold.
    pub gold_value: u32,
}

/// AI behavior mode for a creature.
#[derive(Debug, Clone)]
pub enum AiBehavior {
    /// Always attacks.
    Aggressive,
    /// Attacks, but defends when below 30% HP.
    Defensive,
    /// Attacks, but retreats when below 50% HP.
    Skirmisher,
    /// Uses a support ability when allies are near, otherwise attacks.
    Supporter,
    /// Attacks until HP falls to a threshold, then flees.
    Coward {
        /// HP fraction (0.0 to 1.0) at or below which it flees.
        flee_threshold_hp_pct: f64,
    },
    /// Attacks only with enough allies nearby, otherwise retreats.
    Pack {
        /// Allies needed nearby before it attacks.
        min_allies: usize,
    },
    /// Guards an area and attacks intruders.
    Territorial {
        /// Size of the guarded area (not used by `decide_action` yet, which always attacks).
        range: f64,
    },
    /// Opens with an ambush strike.
    Ambush,
}

/// A single attack profile.
#[derive(Debug, Clone)]
pub struct AttackProfile {
    /// Name of the attack.
    pub name: String,
    /// Bonus added to the d20 attack roll.
    pub hit_bonus: i8,
    /// (count, sides) e.g. (2, 6) = 2d6.
    pub damage_dice: (u32, u32),
    /// Flat damage added to the dice.
    pub damage_bonus: i32,
    /// Reach in feet.
    pub reach_ft: u32,
}

/// Result of an attack roll.
#[derive(Debug, Clone)]
pub struct AttackResult {
    /// The natural d20 roll (1 to 20).
    pub roll: u32,
    /// Roll plus hit bonus.
    pub total: i32,
    /// Whether the attack hit.
    pub is_hit: bool,
    /// Whether it was a natural 20.
    pub is_critical: bool,
}

/// Action chosen by the AI.
#[derive(Debug, Clone)]
pub enum CombatAction {
    /// Attack a target.
    Attack {
        /// Index of the target in the encounter.
        target_idx: usize,
    },
    /// Run away from the fight.
    Flee,
    /// Use the named special ability.
    UseAbility(String),
    /// Brace and defend this turn.
    Defend,
    /// Back off without leaving the fight.
    Retreat,
}

/// A creature instance.
#[derive(Debug, Clone)]
pub struct Creature {
    /// Unique id of the creature type.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Creature type (beast, undead and so on).
    pub creature_type: CreatureType,
    /// Size category.
    pub size: CreatureSize,
    /// Challenge rating: how dangerous it is, used to budget encounters.
    pub challenge_rating: f64,
    /// Maximum hit points.
    pub hp_max: u32,
    /// Current hit points.
    pub hp_current: u32,
    /// Armor class an attack roll must reach to hit.
    pub armor_class: u8,
    /// Walking speed in feet per round.
    pub speed_ft: u32,
    /// The six ability scores.
    pub ability_scores: AbilityScores,
    /// Attacks it can make.
    pub attacks: Vec<AttackProfile>,
    /// What it can drop when killed.
    pub loot_table: Vec<LootEntry>,
    /// How it behaves in combat.
    pub ai_behavior: AiBehavior,
    /// Experience awarded for defeating it.
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
        total += atk.damage_bonus;
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
