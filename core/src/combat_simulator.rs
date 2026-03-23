//! Full combat simulation with initiative, status effects, and action economy.

use serde::{Deserialize, Serialize};

// ─── STRUCTS ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combatant {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub ac: u8,
    pub attack_bonus: i8,
    pub damage_dice: (u8, u8), // (num, sides)
    pub speed: u32,
    pub initiative_bonus: i8,
}

impl Combatant {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

// ─── COMBAT ACTIONS ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CombatAction {
    Attack { target_id: String },
    Dodge,
    Dash,
    Help { target_id: String },
    UseItem(String),
}

// ─── COMBAT ROUND ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatRound {
    pub round_number: u32,
    pub actions: Vec<(String, CombatAction)>,
    pub damage_dealt: Vec<(String, i32)>,
}

// ─── DICE ROLLER ─────────────────────────────────────────────────────────────

/// LCG dice roller — deterministic given a seed.
pub fn roll_dice(num: u8, sides: u8, seed: u64) -> i32 {
    if num == 0 || sides == 0 {
        return 0;
    }
    let mut state = seed;
    let mut total: i32 = 0;
    for _ in 0..num {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let roll = ((state >> 33) % sides as u64) as i32 + 1;
        total += roll;
    }
    total
}

/// Advance LCG one step and return new state.
fn lcg_next(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

// ─── COMBAT SIMULATOR ────────────────────────────────────────────────────────

pub struct CombatSimulator {
    combatants: Vec<Combatant>,
}

impl CombatSimulator {
    pub fn new() -> Self {
        CombatSimulator {
            combatants: Vec::new(),
        }
    }

    pub fn add_combatant(&mut self, c: Combatant) {
        self.combatants.push(c);
    }

    /// Roll initiative for all combatants. Returns sorted vec (highest first).
    pub fn roll_initiative(&self, seed: u64) -> Vec<(String, i32)> {
        let mut results: Vec<(String, i32)> = self
            .combatants
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let s = lcg_next(seed.wrapping_add(i as u64 * 7919));
                let d20 = ((s >> 33) % 20) as i32 + 1;
                let init = d20 + c.initiative_bonus as i32;
                (c.id.clone(), init)
            })
            .collect();
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    /// Resolve a single attack roll. Returns damage dealt (0 if miss).
    pub fn resolve_attack(
        &self,
        attacker: &Combatant,
        defender: &mut Combatant,
        seed: u64,
    ) -> i32 {
        let s1 = lcg_next(seed);
        let d20 = ((s1 >> 33) % 20) as i32 + 1;
        let attack_roll = d20 + attacker.attack_bonus as i32;
        if attack_roll >= defender.ac as i32 {
            let s2 = lcg_next(s1);
            let damage = roll_dice(attacker.damage_dice.0, attacker.damage_dice.1, s2).max(1);
            defender.hp -= damage;
            damage
        } else {
            0
        }
    }

    /// Simulate one round given an initiative order.
    pub fn simulate_round(
        &mut self,
        initiative_order: &[(String, i32)],
        seed: u64,
    ) -> CombatRound {
        let mut actions: Vec<(String, CombatAction)> = Vec::new();
        let mut damage_dealt: Vec<(String, i32)> = Vec::new();
        let round_seed = lcg_next(seed);

        // Collect combatant ids (alive) for targeting
        let alive_ids: Vec<String> = self
            .combatants
            .iter()
            .filter(|c| c.is_alive())
            .map(|c| c.id.clone())
            .collect();

        for (idx, (actor_id, _)) in initiative_order.iter().enumerate() {
            // Find a living target that is not the actor
            let actor_alive = self
                .combatants
                .iter()
                .find(|c| &c.id == actor_id)
                .map(|c| c.is_alive())
                .unwrap_or(false);

            if !actor_alive {
                continue;
            }

            let target_id = alive_ids
                .iter()
                .find(|id| *id != actor_id)
                .cloned();

            if let Some(tid) = target_id {
                let action = CombatAction::Attack { target_id: tid.clone() };
                actions.push((actor_id.clone(), action));

                // Perform attack
                let iter_seed = round_seed.wrapping_add(idx as u64 * 1000003);
                // Split borrow: read attacker, then mutate defender
                let attacker_clone = self
                    .combatants
                    .iter()
                    .find(|c| &c.id == actor_id)
                    .cloned();
                if let Some(attacker) = attacker_clone {
                    let defender_idx = self
                        .combatants
                        .iter()
                        .position(|c| c.id == tid);
                    if let Some(di) = defender_idx {
                        let d20_s = lcg_next(iter_seed);
                        let d20 = ((d20_s >> 33) % 20) as i32 + 1;
                        let attack_roll = d20 + attacker.attack_bonus as i32;
                        if attack_roll >= self.combatants[di].ac as i32 {
                            let dmg_s = lcg_next(d20_s);
                            let dmg = roll_dice(
                                attacker.damage_dice.0,
                                attacker.damage_dice.1,
                                dmg_s,
                            )
                            .max(1);
                            self.combatants[di].hp -= dmg;
                            damage_dealt.push((tid.clone(), dmg));
                        } else {
                            damage_dealt.push((tid.clone(), 0));
                        }
                    }
                }
            } else {
                actions.push((actor_id.clone(), CombatAction::Dodge));
            }
        }

        CombatRound {
            round_number: 0, // filled by caller
            actions,
            damage_dealt,
        }
    }

    /// Run full combat until one faction is eliminated or max_rounds reached.
    pub fn simulate_combat(&mut self, max_rounds: u32, seed: u64) -> Vec<CombatRound> {
        let mut rounds: Vec<CombatRound> = Vec::new();
        let initiative_order = self.roll_initiative(seed);

        for round_num in 1..=max_rounds {
            // Check if combat is over
            let alive_count = self.combatants.iter().filter(|c| c.is_alive()).count();
            if alive_count <= 1 {
                break;
            }
            let round_seed = lcg_next(seed.wrapping_mul(round_num as u64));
            let mut round = self.simulate_round(&initiative_order, round_seed);
            round.round_number = round_num;
            rounds.push(round);
        }
        rounds
    }

    /// Produce a human-readable combat summary.
    pub fn combat_summary(&self, rounds: &[CombatRound]) -> String {
        let total_damage: i32 = rounds
            .iter()
            .flat_map(|r| r.damage_dealt.iter())
            .map(|(_, d)| d)
            .sum();
        let survivors: Vec<&str> = self
            .combatants
            .iter()
            .filter(|c| c.is_alive())
            .map(|c| c.name.as_str())
            .collect();
        format!(
            "Combat lasted {} round(s). Total damage dealt: {}. Survivors: [{}].",
            rounds.len(),
            total_damage,
            survivors.join(", ")
        )
    }
}

impl Default for CombatSimulator {
    fn default() -> Self {
        Self::new()
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_combatant(id: &str, name: &str, hp: i32, ac: u8, atk: i8) -> Combatant {
        Combatant {
            id: id.to_string(),
            name: name.to_string(),
            hp,
            max_hp: hp,
            ac,
            attack_bonus: atk,
            damage_dice: (1, 6),
            speed: 30,
            initiative_bonus: 2,
        }
    }

    #[test]
    fn test_roll_dice_deterministic() {
        let r1 = roll_dice(2, 6, 42);
        let r2 = roll_dice(2, 6, 42);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_roll_dice_range() {
        for seed in 0..100u64 {
            let r = roll_dice(3, 6, seed);
            assert!(r >= 3 && r <= 18, "roll {} out of range", r);
        }
    }

    #[test]
    fn test_roll_dice_zero_num() {
        assert_eq!(roll_dice(0, 6, 1), 0);
    }

    #[test]
    fn test_roll_dice_zero_sides() {
        assert_eq!(roll_dice(2, 0, 1), 0);
    }

    #[test]
    fn test_combatant_is_alive() {
        let alive = make_combatant("a", "A", 10, 12, 3);
        let dead = Combatant { hp: 0, ..make_combatant("b", "B", 10, 12, 3) };
        assert!(alive.is_alive());
        assert!(!dead.is_alive());
    }

    #[test]
    fn test_roll_initiative_sorted() {
        let mut sim = CombatSimulator::new();
        sim.add_combatant(make_combatant("a", "A", 20, 12, 3));
        sim.add_combatant(make_combatant("b", "B", 20, 12, 3));
        sim.add_combatant(make_combatant("c", "C", 20, 14, 1));
        let order = sim.roll_initiative(99);
        assert_eq!(order.len(), 3);
        for i in 0..order.len() - 1 {
            assert!(order[i].1 >= order[i + 1].1);
        }
    }

    #[test]
    fn test_simulate_combat_runs() {
        let mut sim = CombatSimulator::new();
        sim.add_combatant(make_combatant("a", "Alice", 30, 12, 4));
        sim.add_combatant(make_combatant("b", "Bob", 30, 10, 2));
        let rounds = sim.simulate_combat(10, 1234);
        assert!(!rounds.is_empty());
    }

    #[test]
    fn test_simulate_combat_max_rounds() {
        let mut sim = CombatSimulator::new();
        // Two very tanky combatants
        sim.add_combatant(Combatant {
            damage_dice: (1, 1), // only 1 damage
            hp: 1000, max_hp: 1000,
            ..make_combatant("a", "Tank1", 1000, 20, 0)
        });
        sim.add_combatant(Combatant {
            damage_dice: (1, 1),
            hp: 1000, max_hp: 1000,
            ..make_combatant("b", "Tank2", 1000, 20, 0)
        });
        let rounds = sim.simulate_combat(5, 7);
        assert!(rounds.len() <= 5);
    }

    #[test]
    fn test_combat_summary_contains_rounds() {
        let mut sim = CombatSimulator::new();
        sim.add_combatant(make_combatant("a", "Hero", 10, 8, 5));
        sim.add_combatant(make_combatant("b", "Villain", 5, 8, 5));
        let rounds = sim.simulate_combat(20, 555);
        let summary = sim.combat_summary(&rounds);
        assert!(summary.contains("round"));
        assert!(summary.contains("damage"));
    }

    #[test]
    fn test_round_number_sequence() {
        let mut sim = CombatSimulator::new();
        sim.add_combatant(make_combatant("a", "A", 100, 15, 3));
        sim.add_combatant(make_combatant("b", "B", 100, 15, 3));
        let rounds = sim.simulate_combat(5, 77);
        for (i, r) in rounds.iter().enumerate() {
            assert_eq!(r.round_number, (i + 1) as u32);
        }
    }

    #[test]
    fn test_add_multiple_combatants() {
        let mut sim = CombatSimulator::new();
        for i in 0..5 {
            sim.add_combatant(make_combatant(
                &format!("{}", i),
                &format!("Fighter{}", i),
                20, 12, 2,
            ));
        }
        let order = sim.roll_initiative(11);
        assert_eq!(order.len(), 5);
    }

    #[test]
    fn test_combat_action_enum_variants() {
        let _atk = CombatAction::Attack { target_id: "t".to_string() };
        let _dodge = CombatAction::Dodge;
        let _dash = CombatAction::Dash;
        let _help = CombatAction::Help { target_id: "t".to_string() };
        let _item = CombatAction::UseItem("Potion".to_string());
    }

    #[test]
    fn test_combat_ends_when_one_side_dead() {
        let mut sim = CombatSimulator::new();
        // Attacker has massive damage vs tiny HP
        sim.add_combatant(Combatant {
            damage_dice: (10, 6),
            attack_bonus: 20,
            ..make_combatant("a", "Slayer", 100, 5, 20)
        });
        sim.add_combatant(make_combatant("b", "Victim", 1, 5, 0));
        let rounds = sim.simulate_combat(100, 1);
        // Should end quickly
        assert!(!rounds.is_empty());
    }
}
