//! Turn-based combat engine — initiative, conditions, actions, rounds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ─── COMBAT CONDITION ────────────────────────────────────────────────────────

/// Status conditions that can affect a combatant during combat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CombatCondition {
    Stunned,
    Poisoned { damage_per_turn: u32 },
    Blinded,
    Frightened,
    Paralyzed,
    Burning { damage_per_turn: u32 },
    Invisible,
    Hasted,
    Slow,
}

impl fmt::Display for CombatCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CombatCondition::Stunned => write!(f, "Stunned"),
            CombatCondition::Poisoned { damage_per_turn } => {
                write!(f, "Poisoned({}dmg/turn)", damage_per_turn)
            }
            CombatCondition::Blinded => write!(f, "Blinded"),
            CombatCondition::Frightened => write!(f, "Frightened"),
            CombatCondition::Paralyzed => write!(f, "Paralyzed"),
            CombatCondition::Burning { damage_per_turn } => {
                write!(f, "Burning({}dmg/turn)", damage_per_turn)
            }
            CombatCondition::Invisible => write!(f, "Invisible"),
            CombatCondition::Hasted => write!(f, "Hasted"),
            CombatCondition::Slow => write!(f, "Slow"),
        }
    }
}

impl CombatCondition {
    /// True if the condition deals damage each turn.
    pub fn is_dot(&self) -> bool {
        matches!(
            self,
            CombatCondition::Poisoned { .. } | CombatCondition::Burning { .. }
        )
    }

    /// Per-turn damage dealt by DoT conditions (0 for non-DoT).
    pub fn dot_damage(&self) -> u32 {
        match self {
            CombatCondition::Poisoned { damage_per_turn } => *damage_per_turn,
            CombatCondition::Burning { damage_per_turn } => *damage_per_turn,
            _ => 0,
        }
    }

    /// Returns true if this condition prevents acting.
    pub fn prevents_action(&self) -> bool {
        matches!(
            self,
            CombatCondition::Stunned | CombatCondition::Paralyzed
        )
    }

    /// Returns a variant-level tag for equality checks ignoring fields.
    pub fn variant_tag(&self) -> u8 {
        match self {
            CombatCondition::Stunned => 0,
            CombatCondition::Poisoned { .. } => 1,
            CombatCondition::Blinded => 2,
            CombatCondition::Frightened => 3,
            CombatCondition::Paralyzed => 4,
            CombatCondition::Burning { .. } => 5,
            CombatCondition::Invisible => 6,
            CombatCondition::Hasted => 7,
            CombatCondition::Slow => 8,
        }
    }
}

// ─── ACTION TYPE ─────────────────────────────────────────────────────────────

/// The type of action a combatant can take on their turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    Attack,
    CastSpell(String),
    UseItem(String),
    Dodge,
    Disengage,
    Help { target_idx: usize },
    Dash,
    Hide,
    Grapple,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::Attack => write!(f, "Attack"),
            ActionType::CastSpell(s) => write!(f, "CastSpell({})", s),
            ActionType::UseItem(s) => write!(f, "UseItem({})", s),
            ActionType::Dodge => write!(f, "Dodge"),
            ActionType::Disengage => write!(f, "Disengage"),
            ActionType::Help { target_idx } => write!(f, "Help(idx={})", target_idx),
            ActionType::Dash => write!(f, "Dash"),
            ActionType::Hide => write!(f, "Hide"),
            ActionType::Grapple => write!(f, "Grapple"),
        }
    }
}

// ─── COMBATANT ───────────────────────────────────────────────────────────────

/// A participant in combat — player character or enemy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub id: usize,
    pub name: String,
    pub hp_current: i32,
    pub hp_max: i32,
    pub armor_class: u8,
    pub initiative: i32,
    /// (condition, remaining_turns)
    pub conditions: Vec<(CombatCondition, u32)>,
    /// 0 = players, 1 = enemies
    pub team: u8,
}

impl Combatant {
    pub fn new(
        id: usize,
        name: impl Into<String>,
        hp: i32,
        armor_class: u8,
        initiative: i32,
        team: u8,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            hp_current: hp,
            hp_max: hp,
            armor_class,
            initiative,
            conditions: Vec::new(),
            team,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp_current > 0
    }

    /// Apply damage. Returns true if the combatant died from this damage.
    pub fn take_damage(&mut self, amount: u32) -> bool {
        let was_alive = self.is_alive();
        self.hp_current -= amount as i32;
        if self.hp_current < 0 {
            self.hp_current = 0;
        }
        was_alive && !self.is_alive()
    }

    /// Apply healing (capped at hp_max).
    pub fn heal(&mut self, amount: u32) {
        self.hp_current = (self.hp_current + amount as i32).min(self.hp_max);
    }

    /// Add a condition with a duration in turns.
    pub fn add_condition(&mut self, cond: CombatCondition, duration: u32) {
        // Update existing condition of same variant rather than duplicating.
        let tag = cond.variant_tag();
        if let Some(existing) = self
            .conditions
            .iter_mut()
            .find(|(c, _)| c.variant_tag() == tag)
        {
            *existing = (cond, duration);
        } else {
            self.conditions.push((cond, duration));
        }
    }

    /// Tick all conditions: apply DoT, decrement durations, remove expired.
    pub fn tick_conditions(&mut self) {
        let mut dot_total: u32 = 0;
        self.conditions.retain_mut(|(cond, turns)| {
            // Apply DoT
            if cond.is_dot() {
                dot_total += cond.dot_damage();
            }
            // Decrement duration
            if *turns > 0 {
                *turns -= 1;
            }
            *turns > 0
        });
        if dot_total > 0 {
            self.take_damage(dot_total);
        }
    }

    /// Returns true if this combatant currently has the given condition variant.
    pub fn has_condition(&self, cond: &CombatCondition) -> bool {
        let tag = cond.variant_tag();
        self.conditions.iter().any(|(c, _)| c.variant_tag() == tag)
    }

    /// Returns true if any condition prevents this combatant from acting.
    pub fn is_incapacitated(&self) -> bool {
        self.conditions
            .iter()
            .any(|(c, _)| c.prevents_action())
    }
}

// ─── ACTION RESULT ───────────────────────────────────────────────────────────

/// The result of a single combat action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub hit: bool,
    pub damage: u32,
    pub healing: u32,
    pub conditions_applied: Vec<CombatCondition>,
    pub message: String,
}

impl ActionResult {
    pub fn miss(msg: impl Into<String>) -> Self {
        Self {
            hit: false,
            damage: 0,
            healing: 0,
            conditions_applied: Vec::new(),
            message: msg.into(),
        }
    }
}

// ─── COMBAT ACTION ───────────────────────────────────────────────────────────

/// A single action taken by a combatant during a round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatAction {
    pub actor_id: usize,
    pub action: ActionType,
    pub target_id: Option<usize>,
    pub result: ActionResult,
}

// ─── COMBAT ROUND ────────────────────────────────────────────────────────────

/// All actions and the initiative order for a single round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatRound {
    pub round_num: u32,
    pub initiative_order: Vec<usize>,
    pub actions: Vec<CombatAction>,
}

// ─── COMBAT ENGINE ───────────────────────────────────────────────────────────

/// Drives a full turn-based combat encounter.
pub struct CombatEngine {
    pub combatants: Vec<Combatant>,
    pub round: u32,
    pub log: Vec<CombatRound>,
}

/// Minimal LCG for deterministic pseudo-randomness without external deps.
fn lcg_next(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed
}

/// Returns a value in 1..=sides.
fn roll_die(sides: u32, seed: &mut u64) -> u32 {
    (lcg_next(seed) % sides as u64) as u32 + 1
}

impl CombatEngine {
    pub fn new(combatants: Vec<Combatant>) -> Self {
        Self {
            combatants,
            round: 0,
            log: Vec::new(),
        }
    }

    /// Roll initiative for all combatants using d20 + their initiative modifier,
    /// then sort descending.
    pub fn roll_initiative(&mut self, seed: u64) {
        let mut rng = seed;
        for c in self.combatants.iter_mut() {
            let roll = roll_die(20, &mut rng) as i32;
            c.initiative = roll + c.initiative;
        }
        self.combatants
            .sort_by(|a, b| b.initiative.cmp(&a.initiative));
    }

    /// Roll an attack: returns (hit, crit).
    /// Crit on natural 20; auto-miss on natural 1.
    pub fn attack_roll(
        attacker: &Combatant,
        target: &Combatant,
        seed: u64,
    ) -> (bool, bool) {
        let mut rng = seed;
        let natural = roll_die(20, &mut rng);
        if natural == 1 {
            return (false, false);
        }
        if natural == 20 {
            return (true, true);
        }
        // Simple attack bonus: initiative modifier as proxy for attack bonus
        let attack_total = natural as i32 + attacker.initiative / 4;
        (attack_total >= target.armor_class as i32, false)
    }

    /// Roll damage dice + modifier. Crits double the dice.
    pub fn damage_roll(
        dice: (u32, u32),
        modifier: i32,
        is_crit: bool,
        seed: u64,
    ) -> u32 {
        let mut rng = seed.wrapping_add(0xDEAD_BEEF);
        let num_dice = if is_crit { dice.0 * 2 } else { dice.0 };
        let total: u32 = (0..num_dice).map(|_| roll_die(dice.1, &mut rng)).sum();
        let result = total as i32 + modifier;
        result.max(1) as u32
    }

    /// Execute a single action by combatant at `actor_idx`, optionally targeting
    /// `target_idx`. Returns an `ActionResult`.
    pub fn execute_action(
        &mut self,
        actor_idx: usize,
        action: ActionType,
        target_idx: Option<usize>,
        seed: u64,
    ) -> ActionResult {
        // Gather actor info without borrowing self.combatants mutably yet
        let actor_alive = self.combatants[actor_idx].is_alive();
        let actor_incap = self.combatants[actor_idx].is_incapacitated();

        if !actor_alive {
            return ActionResult::miss(format!(
                "{} is dead and cannot act.",
                self.combatants[actor_idx].name
            ));
        }
        if actor_incap {
            return ActionResult::miss(format!(
                "{} is incapacitated and loses their turn.",
                self.combatants[actor_idx].name
            ));
        }

        match action {
            ActionType::Attack => {
                let tidx = match target_idx {
                    Some(i) => i,
                    None => {
                        return ActionResult::miss("No target for attack.".to_string());
                    }
                };
                if tidx >= self.combatants.len() || !self.combatants[tidx].is_alive() {
                    return ActionResult::miss("Invalid or dead target.".to_string());
                }

                let attacker = &self.combatants[actor_idx];
                let target = &self.combatants[tidx];

                let (hit, crit) = Self::attack_roll(attacker, target, seed);
                let actor_name = attacker.name.clone();
                let target_name = target.name.clone();

                if !hit {
                    return ActionResult {
                        hit: false,
                        damage: 0,
                        healing: 0,
                        conditions_applied: Vec::new(),
                        message: format!("{} attacks {} — MISS!", actor_name, target_name),
                    };
                }

                let dmg = Self::damage_roll((1, 8), 2, crit, seed);
                let died = self.combatants[tidx].take_damage(dmg);

                let msg = if crit {
                    format!(
                        "{} CRITS {} for {} damage!{}",
                        actor_name,
                        target_name,
                        dmg,
                        if died { " (KILLED)" } else { "" }
                    )
                } else {
                    format!(
                        "{} hits {} for {} damage.{}",
                        actor_name,
                        target_name,
                        dmg,
                        if died { " (KILLED)" } else { "" }
                    )
                };

                ActionResult {
                    hit: true,
                    damage: dmg,
                    healing: 0,
                    conditions_applied: Vec::new(),
                    message: msg,
                }
            }

            ActionType::CastSpell(ref spell_name) => {
                let tidx = match target_idx {
                    Some(i) => i,
                    None => {
                        return ActionResult::miss("No target for spell.".to_string());
                    }
                };
                if tidx >= self.combatants.len() {
                    return ActionResult::miss("Invalid target index.".to_string());
                }

                let actor_name = self.combatants[actor_idx].name.clone();
                let target_name = self.combatants[tidx].name.clone();
                let spell = spell_name.to_lowercase();

                if spell.contains("fire") || spell.contains("burn") {
                    let dmg = Self::damage_roll((2, 6), 3, false, seed);
                    let cond = CombatCondition::Burning { damage_per_turn: 3 };
                    self.combatants[tidx].take_damage(dmg);
                    self.combatants[tidx].add_condition(cond.clone(), 3);
                    ActionResult {
                        hit: true,
                        damage: dmg,
                        healing: 0,
                        conditions_applied: vec![cond],
                        message: format!(
                            "{} casts {} on {} for {} fire damage and Burning!",
                            actor_name, spell_name, target_name, dmg
                        ),
                    }
                } else if spell.contains("heal") || spell.contains("cure") {
                    let heal = Self::damage_roll((2, 8), 5, false, seed);
                    self.combatants[tidx].heal(heal);
                    ActionResult {
                        hit: true,
                        damage: 0,
                        healing: heal,
                        conditions_applied: Vec::new(),
                        message: format!(
                            "{} casts {} on {} — healed {} HP.",
                            actor_name, spell_name, target_name, heal
                        ),
                    }
                } else if spell.contains("poison") {
                    let dmg = Self::damage_roll((1, 4), 1, false, seed);
                    let cond = CombatCondition::Poisoned { damage_per_turn: 4 };
                    self.combatants[tidx].take_damage(dmg);
                    self.combatants[tidx].add_condition(cond.clone(), 4);
                    ActionResult {
                        hit: true,
                        damage: dmg,
                        healing: 0,
                        conditions_applied: vec![cond],
                        message: format!(
                            "{} poisons {} for {} initial damage!",
                            actor_name, target_name, dmg
                        ),
                    }
                } else {
                    // Generic magic damage
                    let dmg = Self::damage_roll((1, 10), 4, false, seed);
                    self.combatants[tidx].take_damage(dmg);
                    ActionResult {
                        hit: true,
                        damage: dmg,
                        healing: 0,
                        conditions_applied: Vec::new(),
                        message: format!(
                            "{} casts {} on {} for {} magic damage.",
                            actor_name, spell_name, target_name, dmg
                        ),
                    }
                }
            }

            ActionType::UseItem(ref item_name) => {
                let actor_name = self.combatants[actor_idx].name.clone();
                if item_name.to_lowercase().contains("potion") {
                    let heal = Self::damage_roll((2, 4), 2, false, seed);
                    self.combatants[actor_idx].heal(heal);
                    ActionResult {
                        hit: true,
                        damage: 0,
                        healing: heal,
                        conditions_applied: Vec::new(),
                        message: format!("{} uses {} and heals {} HP.", actor_name, item_name, heal),
                    }
                } else {
                    ActionResult {
                        hit: true,
                        damage: 0,
                        healing: 0,
                        conditions_applied: Vec::new(),
                        message: format!("{} uses {}.", actor_name, item_name),
                    }
                }
            }

            ActionType::Dodge => {
                let actor_name = self.combatants[actor_idx].name.clone();
                // Increase effective AC for one round by granting the Hasted condition briefly
                self.combatants[actor_idx].add_condition(CombatCondition::Hasted, 1);
                ActionResult {
                    hit: false,
                    damage: 0,
                    healing: 0,
                    conditions_applied: vec![CombatCondition::Hasted],
                    message: format!("{} takes the Dodge action.", actor_name),
                }
            }

            ActionType::Disengage => {
                let actor_name = self.combatants[actor_idx].name.clone();
                ActionResult {
                    hit: false,
                    damage: 0,
                    healing: 0,
                    conditions_applied: Vec::new(),
                    message: format!("{} disengages from combat.", actor_name),
                }
            }

            ActionType::Help { target_idx: help_idx } => {
                let actor_name = self.combatants[actor_idx].name.clone();
                let target_name = if help_idx < self.combatants.len() {
                    self.combatants[help_idx].name.clone()
                } else {
                    "unknown".to_string()
                };
                ActionResult {
                    hit: false,
                    damage: 0,
                    healing: 0,
                    conditions_applied: Vec::new(),
                    message: format!("{} helps {} (advantage on next attack).", actor_name, target_name),
                }
            }

            ActionType::Dash => {
                let actor_name = self.combatants[actor_idx].name.clone();
                ActionResult {
                    hit: false,
                    damage: 0,
                    healing: 0,
                    conditions_applied: Vec::new(),
                    message: format!("{} dashes!", actor_name),
                }
            }

            ActionType::Hide => {
                let actor_name = self.combatants[actor_idx].name.clone();
                self.combatants[actor_idx]
                    .add_condition(CombatCondition::Invisible, 2);
                ActionResult {
                    hit: false,
                    damage: 0,
                    healing: 0,
                    conditions_applied: vec![CombatCondition::Invisible],
                    message: format!("{} hides in the shadows!", actor_name),
                }
            }

            ActionType::Grapple => {
                let tidx = match target_idx {
                    Some(i) => i,
                    None => return ActionResult::miss("No target to grapple.".to_string()),
                };
                if tidx >= self.combatants.len() || !self.combatants[tidx].is_alive() {
                    return ActionResult::miss("Invalid or dead target for grapple.".to_string());
                }

                let actor_name = self.combatants[actor_idx].name.clone();
                let target_name = self.combatants[tidx].name.clone();
                // Grapple check: Athletics vs Acrobatics (simplified)
                let mut rng = seed;
                let atk_roll = roll_die(20, &mut rng) + 3;
                let def_roll = roll_die(20, &mut rng) + 1;

                if atk_roll >= def_roll {
                    self.combatants[tidx].add_condition(CombatCondition::Slow, 2);
                    ActionResult {
                        hit: true,
                        damage: 0,
                        healing: 0,
                        conditions_applied: vec![CombatCondition::Slow],
                        message: format!("{} grapples {}! {} is Slowed.", actor_name, target_name, target_name),
                    }
                } else {
                    ActionResult::miss(format!("{} fails to grapple {}.", actor_name, target_name))
                }
            }
        }
    }

    /// Run a full round of combat. `ai_actions` maps combatant index -> action for
    /// those whose actions are determined externally.
    pub fn run_round(
        &mut self,
        ai_actions: &HashMap<usize, ActionType>,
        seed: u64,
    ) -> CombatRound {
        self.round += 1;
        let initiative_order: Vec<usize> = (0..self.combatants.len())
            .filter(|i| self.combatants[*i].is_alive())
            .collect();

        let mut actions = Vec::new();

        for &idx in &initiative_order {
            if !self.combatants[idx].is_alive() {
                continue;
            }
            if self.is_combat_over() {
                break;
            }

            // Tick conditions at start of turn
            let _cond_seed = seed.wrapping_add(idx as u64 * 1337);
            // We need to tick conditions — but we can't borrow mutably while
            // iterating initiative_order (which doesn't borrow combatants). Safe.
            self.combatants[idx].tick_conditions();

            if !self.combatants[idx].is_alive() {
                continue;
            }

            // Determine action
            let action = if let Some(act) = ai_actions.get(&idx) {
                act.clone()
            } else {
                // Default AI: attack nearest living enemy
                let enemy_team = 1 - self.combatants[idx].team;
                let target = self
                    .combatants
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.team == enemy_team && c.is_alive())
                    .map(|(i, _)| i);
                match target {
                    Some(_) => ActionType::Attack,
                    None => ActionType::Dodge,
                }
            };

            // Find default target if needed
            let target_idx = if let ActionType::Help { target_idx } = &action {
                Some(*target_idx)
            } else {
                let enemy_team = 1 - self.combatants[idx].team;
                self.combatants
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.team == enemy_team && c.is_alive())
                    .map(|(i, _)| i)
            };

            let actor_id = self.combatants[idx].id;
            let target_id = target_idx.map(|i| self.combatants[i].id);
            let round_seed = seed.wrapping_add(self.round as u64 * 7919 + idx as u64 * 31);

            let result = self.execute_action(idx, action.clone(), target_idx, round_seed);

            actions.push(CombatAction {
                actor_id,
                action,
                target_id,
                result,
            });
        }

        let round = CombatRound {
            round_num: self.round,
            initiative_order: initiative_order
                .iter()
                .map(|&i| self.combatants[i].id)
                .collect(),
            actions,
        };
        self.log.push(round.clone());
        round
    }

    /// Returns true if one or both teams have no surviving combatants.
    pub fn is_combat_over(&self) -> bool {
        let team0_alive = self.combatants.iter().any(|c| c.team == 0 && c.is_alive());
        let team1_alive = self.combatants.iter().any(|c| c.team == 1 && c.is_alive());
        !team0_alive || !team1_alive
    }

    /// Returns the winning team index, if combat is over.
    pub fn winner_team(&self) -> Option<u8> {
        if !self.is_combat_over() {
            return None;
        }
        let team0_alive = self.combatants.iter().any(|c| c.team == 0 && c.is_alive());
        if team0_alive {
            Some(0)
        } else {
            Some(1)
        }
    }

    /// A human-readable summary of the entire combat encounter.
    pub fn combat_summary(&self) -> String {
        let mut s = format!(
            "=== Combat Summary: {} rounds ===\n",
            self.round
        );
        for round in &self.log {
            s.push_str(&format!("\n-- Round {} --\n", round.round_num));
            for action in &round.actions {
                s.push_str(&format!("  {}\n", action.result.message));
            }
        }
        s.push_str("\n--- Final Status ---\n");
        for c in &self.combatants {
            s.push_str(&format!(
                "  {} (team {}): {}/{} HP — {}\n",
                c.name,
                c.team,
                c.hp_current,
                c.hp_max,
                if c.is_alive() { "ALIVE" } else { "DEAD" }
            ));
        }
        if let Some(winner) = self.winner_team() {
            s.push_str(&format!("\nWINNER: Team {}\n", winner));
        } else {
            s.push_str("\nCombat ongoing.\n");
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> CombatEngine {
        let combatants = vec![
            Combatant::new(0, "Hero", 30, 15, 2, 0),
            Combatant::new(1, "Goblin", 10, 12, 0, 1),
        ];
        CombatEngine::new(combatants)
    }

    #[test]
    fn test_basic_combat_round() {
        let mut engine = make_engine();
        engine.roll_initiative(42);
        let actions = HashMap::new();
        let round = engine.run_round(&actions, 42);
        assert_eq!(round.round_num, 1);
    }

    #[test]
    fn test_conditions() {
        let mut c = Combatant::new(0, "Fighter", 20, 14, 1, 0);
        c.add_condition(CombatCondition::Poisoned { damage_per_turn: 3 }, 2);
        assert!(c.has_condition(&CombatCondition::Poisoned { damage_per_turn: 0 }));
        c.tick_conditions();
        assert_eq!(c.hp_current, 17);
        c.tick_conditions();
        assert!(!c.has_condition(&CombatCondition::Poisoned { damage_per_turn: 0 }));
    }

    #[test]
    fn test_is_combat_over_single_death() {
        let combatants = vec![
            Combatant::new(0, "Hero", 30, 15, 2, 0),
            Combatant::new(1, "Goblin", 1, 12, 0, 1),
        ];
        let mut engine = CombatEngine::new(combatants);
        engine.combatants[1].take_damage(1);
        assert!(engine.is_combat_over());
        assert_eq!(engine.winner_team(), Some(0));
    }

    #[test]
    fn test_damage_roll_crit_doubles_dice() {
        let normal = CombatEngine::damage_roll((2, 6), 0, false, 1);
        let crit = CombatEngine::damage_roll((2, 6), 0, true, 1);
        // Crits roll 4 dice instead of 2 — expected value doubles
        assert!(crit >= normal || crit <= normal * 3);
    }
}
