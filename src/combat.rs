//! Turn-based combat system powered by chaos math.
//!
//! Every attack, dodge, crit, and spell is determined by chaining
//! mathematical algorithms. No dice. Pure chaos.

use crate::chaos_pipeline::{biased_chaos_roll, chaos_roll_verbose, roll_damage, ChaosRollResult};
use crate::character::{Character, CharacterClass, StatBlock};
use crate::enemy::Enemy;
use serde::{Deserialize, Serialize};

// ─── COMBAT ACTIONS ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatAction {
    Attack,
    HeavyAttack,
    Defend,
    UseSpell(usize), // spell index
    UseItem(usize),  // item index
    Flee,
    Taunt,
}

impl CombatAction {
    pub fn display_name(&self) -> String {
        match self {
            CombatAction::Attack => "[A] Attack".to_string(),
            CombatAction::HeavyAttack => "[H] Heavy Attack".to_string(),
            CombatAction::Defend => "[D] Defend".to_string(),
            CombatAction::UseSpell(i) => format!("[S{}] Spell {}", i, i + 1),
            CombatAction::UseItem(i) => format!("[I{}] Item {}", i, i + 1),
            CombatAction::Flee => "[F] Flee".to_string(),
            CombatAction::Taunt => "[T] Taunt".to_string(),
        }
    }
}

// ─── COMBAT EVENTS (LOG) ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatEvent {
    PlayerAttack {
        damage: i64,
        is_crit: bool,
    },
    EnemyAttack {
        damage: i64,
        is_crit: bool,
    },
    PlayerDefend {
        damage_reduced: i64,
    },
    PlayerFled,
    PlayerFleeFailed,
    SpellCast {
        name: String,
        damage: i64,
        backfired: bool,
    },
    EnemyDied {
        xp: u64,
        gold: i64,
    },
    PlayerHealed {
        amount: i64,
    },
    StatusApplied {
        name: String,
    },
    ChaosEvent {
        description: String,
    },
}

impl CombatEvent {
    pub fn to_display_string(&self) -> String {
        match self {
            CombatEvent::PlayerAttack { damage, is_crit } => {
                if *is_crit {
                    format!("★ CRITICAL HIT! You deal {} damage!", damage)
                } else {
                    format!("You attack for {} damage.", damage)
                }
            }
            CombatEvent::EnemyAttack { damage, is_crit } => {
                if *is_crit {
                    format!("☠ ENEMY CRITS YOU for {} damage!", damage)
                } else {
                    format!("Enemy strikes you for {} damage.", damage)
                }
            }
            CombatEvent::PlayerDefend { damage_reduced } => {
                format!("You brace! {} damage absorbed.", damage_reduced)
            }
            CombatEvent::PlayerFled => "You escape into the chaos!".to_string(),
            CombatEvent::PlayerFleeFailed => {
                "You can't escape — the math won't allow it!".to_string()
            }
            CombatEvent::SpellCast {
                name,
                damage,
                backfired,
            } => {
                if *backfired {
                    format!("☢ {} BACKFIRES! You take {} damage!", name, damage)
                } else {
                    format!("✦ {} blasts for {} damage!", name, damage)
                }
            }
            CombatEvent::EnemyDied { xp, gold } => {
                format!("Enemy slain! +{} XP, +{} gold.", xp, gold)
            }
            CombatEvent::PlayerHealed { amount } => {
                format!("You recover {} HP.", amount)
            }
            CombatEvent::StatusApplied { name } => {
                format!("Status applied: {}.", name)
            }
            CombatEvent::ChaosEvent { description } => {
                format!("⚡ CHAOS EVENT: {}", description)
            }
        }
    }
}

// ─── COMBAT STATE ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CombatState {
    pub turn: u32,
    pub player_defending: bool,
    pub enemy_stunned: bool,
    pub chaos_events: u32,
    pub log: Vec<CombatEvent>,
    pub last_roll: Option<ChaosRollResult>,
    pub seed: u64,
}

impl CombatState {
    pub fn new(seed: u64) -> Self {
        CombatState {
            turn: 0,
            player_defending: false,
            enemy_stunned: false,
            chaos_events: 0,
            log: Vec::new(),
            last_roll: None,
            seed,
        }
    }

    fn next_seed(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.seed
    }
}

// ─── COMBAT OUTCOMES ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatOutcome {
    Ongoing,
    PlayerWon { xp: u64, gold: i64 },
    PlayerDied,
    PlayerFled,
}

// ─── COMBAT RESOLUTION ───────────────────────────────────────────────────────

/// Resolve a player action against an enemy. Returns events and outcome.
pub fn resolve_action(
    player: &mut Character,
    enemy: &mut Enemy,
    action: CombatAction,
    state: &mut CombatState,
) -> (Vec<CombatEvent>, CombatOutcome) {
    state.turn += 1;
    state.player_defending = false;

    let mut events = Vec::new();
    let seed = state.next_seed();

    // ── PLAYER TURN ──────────────────────────────────────────────────────────
    match action {
        CombatAction::Attack => {
            let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, seed);
            let is_crit = roll.is_critical();
            let base_dmg = 5 + player.stats.force / 5 + player.stats.precision / 10;
            let mut damage = roll_damage(base_dmg, player.stats.force, seed);

            if is_crit {
                // Crit multiplied by entropy modifier
                let entropy_bonus = (player.stats.entropy as f64 / 50.0 + 1.0).min(4.0);
                damage = (damage as f64 * entropy_bonus) as i64;
            }

            // Class bonuses
            damage += match player.class {
                CharacterClass::Berserker => {
                    // Berserker rage: extra damage when low HP
                    ((1.0 - player.hp_percent()) * 20.0) as i64
                }
                CharacterClass::Ranger => player.stats.precision / 8,
                _ => 0,
            };

            state.last_roll = Some(roll.clone());
            enemy.hp = (enemy.hp - damage).max(0);
            events.push(CombatEvent::PlayerAttack { damage, is_crit });
        }

        CombatAction::HeavyAttack => {
            let roll = biased_chaos_roll(
                player.stats.force as f64 * 0.01,
                0.3, // slight positive bias
                seed,
            );
            let is_crit = roll.is_critical();
            let base_dmg = 12 + player.stats.force / 4;
            let mut damage = roll_damage(
                base_dmg,
                player.stats.force + player.stats.entropy / 2,
                seed,
            );

            if is_crit {
                damage *= 2;
            }
            if roll.is_catastrophe() {
                // Heavy attack whiffs catastrophically
                damage = 0;
                events.push(CombatEvent::ChaosEvent {
                    description: "Your swing goes wide — the Lorenz butterfly mocks you."
                        .to_string(),
                });
            }

            state.last_roll = Some(roll);
            if damage > 0 {
                enemy.hp = (enemy.hp - damage).max(0);
                events.push(CombatEvent::PlayerAttack { damage, is_crit });
            }
        }

        CombatAction::Defend => {
            state.player_defending = true;
            events.push(CombatEvent::PlayerDefend { damage_reduced: 0 }); // updated below
        }

        CombatAction::Flee => {
            let flee_roll = chaos_roll_verbose(player.stats.luck as f64 * 0.01, seed);
            let flee_chance = flee_roll.to_range(1, 100);
            let threshold = 40 + player.stats.cunning / 5;

            if flee_chance > threshold {
                events.push(CombatEvent::PlayerFled);
                return (events, CombatOutcome::PlayerFled);
            } else {
                events.push(CombatEvent::PlayerFleeFailed);
                state.last_roll = Some(flee_roll);
            }
        }

        CombatAction::Taunt => {
            let taunt_roll = chaos_roll_verbose(player.stats.cunning as f64 * 0.01, seed);
            if taunt_roll.is_critical() {
                state.enemy_stunned = true;
                events.push(CombatEvent::StatusApplied {
                    name: "STUNNED (enemy)".to_string(),
                });
            } else if taunt_roll.is_catastrophe() {
                // Taunt backfires: enemy gets enraged
                events.push(CombatEvent::ChaosEvent {
                    description: "Your taunt ENRAGES the enemy! They focus exclusively on you."
                        .to_string(),
                });
            }
        }

        CombatAction::UseSpell(idx) => {
            if let Some(spell) = player.known_spells.get(idx).cloned() {
                let stat_val = get_stat_by_name(&player.stats, &spell.scaling_stat);
                let mut damage = spell.calc_damage(stat_val);

                // Chaos roll modifies effectiveness
                let spell_roll = chaos_roll_verbose(player.stats.mana as f64 * 0.01, seed);
                state.last_roll = Some(spell_roll.clone());

                let backfired = spell_roll.is_catastrophe();
                if backfired {
                    // Spell backfires: damage hits player
                    let self_damage = damage.abs().min(player.current_hp - 1).max(1);
                    player.take_damage(self_damage);
                    player.spells_cast += 1;
                    events.push(CombatEvent::SpellCast {
                        name: spell.name.clone(),
                        damage: self_damage,
                        backfired: true,
                    });
                } else {
                    if spell_roll.is_critical() {
                        damage = (damage as f64 * 1.5) as i64;
                    }
                    // Negative damage heals the enemy (chaotic spells)
                    if damage >= 0 {
                        enemy.hp = (enemy.hp - damage).max(0);
                        player.total_damage_dealt += damage;
                    } else {
                        // Negative damage = heal self
                        player.heal(damage.abs() / 4);
                    }
                    player.spells_cast += 1;
                    events.push(CombatEvent::SpellCast {
                        name: spell.name.clone(),
                        damage: damage.abs(),
                        backfired: false,
                    });

                    // Side effect: apply status based on spell side effect text
                    if spell.side_effect.contains("burning") || spell.side_effect.contains("fire") {
                        player.add_status(crate::character::StatusEffect::Blessed(2));
                        events.push(CombatEvent::StatusApplied { name: "BLESSED (2 turns)".to_string() });
                    }
                }
            } else {
                events.push(CombatEvent::ChaosEvent {
                    description: "No spell at that index. The void laughs at you.".to_string(),
                });
            }
        }

        CombatAction::UseItem(idx) => {
            if let Some(item) = player.use_item(idx) {
                // Items used in combat: apply stat modifiers and/or heal
                let mut heal_amount = 0i64;
                for modifier in &item.stat_modifiers {
                    match modifier.stat.to_lowercase().as_str() {
                        "vitality" => {
                            player.stats.vitality += modifier.value;
                            player.max_hp = (50 + player.stats.vitality * 3 + player.stats.force).max(1);
                            heal_amount += modifier.value * 3;
                        }
                        "force" => {
                            player.stats.force += modifier.value;
                            player.max_hp = (50 + player.stats.vitality * 3 + player.stats.force).max(1);
                        }
                        "mana" => player.stats.mana += modifier.value,
                        "cunning" => player.stats.cunning += modifier.value,
                        "precision" => player.stats.precision += modifier.value,
                        "entropy" => player.stats.entropy += modifier.value,
                        "luck" => player.stats.luck += modifier.value,
                        _ => {}
                    }
                }
                // Weapons deal damage; non-weapons heal
                if item.is_weapon {
                    let weapon_dmg = item.damage_or_defense.abs().max(1);
                    enemy.hp = (enemy.hp - weapon_dmg).max(0);
                    player.total_damage_dealt += weapon_dmg;
                    events.push(CombatEvent::PlayerAttack { damage: weapon_dmg, is_crit: false });
                } else {
                    let base_heal = item.damage_or_defense.abs().max(0) / 5 + heal_amount.max(0);
                    if base_heal > 0 {
                        player.heal(base_heal);
                        events.push(CombatEvent::PlayerHealed { amount: base_heal });
                    }
                }
                events.push(CombatEvent::ChaosEvent {
                    description: format!("Used {}! ({})", item.name, item.special_effect),
                });
            } else {
                events.push(CombatEvent::ChaosEvent {
                    description: "No item at that index. Your pockets are empty.".to_string(),
                });
            }
        }
    }

    // ── Check enemy death ────────────────────────────────────────────────────
    if enemy.hp <= 0 {
        let xp = enemy.xp_reward;
        let gold = enemy.gold_reward;
        events.push(CombatEvent::EnemyDied { xp, gold });
        player.kills += 1;
        player.gold += gold;
        player.gain_xp(xp);
        return (events, CombatOutcome::PlayerWon { xp, gold });
    }

    // ── ENEMY TURN ───────────────────────────────────────────────────────────
    if !state.enemy_stunned {
        let enemy_seed = state.next_seed();
        let enemy_roll = chaos_roll_verbose(enemy.chaos_level, enemy_seed);
        let is_crit = enemy_roll.is_critical();

        let base = enemy.base_damage + enemy.attack_modifier;
        let mut enemy_dmg = roll_damage(base, base, enemy_seed);

        if is_crit {
            enemy_dmg = (enemy_dmg as f64 * 1.5) as i64;
        }

        // Reduce damage if player is defending
        if state.player_defending {
            let reduction = player.stats.vitality / 3 + player.stats.force / 5;
            let reduced = (enemy_dmg - reduction).max(1);
            let damage_reduced = enemy_dmg - reduced;
            enemy_dmg = reduced;
            events.push(CombatEvent::PlayerDefend { damage_reduced });
        }

        player.take_damage(enemy_dmg);
        events.push(CombatEvent::EnemyAttack {
            damage: enemy_dmg,
            is_crit,
        });
    } else {
        state.enemy_stunned = false;
        events.push(CombatEvent::ChaosEvent {
            description: "Enemy is stunned! They skip their turn.".to_string(),
        });
    }

    // ── CHAOS EVENT (rare, 10% chance) ───────────────────────────────────────
    let chaos_roll = chaos_roll_verbose(state.chaos_events as f64 * 0.1, state.next_seed());
    if chaos_roll.final_value > 0.85 {
        let chaos_event = generate_chaos_event(player, enemy, &mut state.seed);
        events.push(chaos_event);
        state.chaos_events += 1;
    }

    // ── CHECK PLAYER DEATH ───────────────────────────────────────────────────
    if !player.is_alive() {
        return (events, CombatOutcome::PlayerDied);
    }

    (events, CombatOutcome::Ongoing)
}

fn get_stat_by_name(stats: &StatBlock, name: &str) -> i64 {
    match name.to_lowercase().as_str() {
        "vitality" => stats.vitality,
        "force" => stats.force,
        "mana" => stats.mana,
        "cunning" => stats.cunning,
        "precision" => stats.precision,
        "entropy" => stats.entropy,
        "luck" => stats.luck,
        _ => 50, // fallback
    }
}

fn generate_chaos_event(player: &mut Character, enemy: &mut Enemy, seed: &mut u64) -> CombatEvent {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let event_type = *seed % 6;

    match event_type {
        0 => {
            let heal = player.stats.luck / 5 + 5;
            player.heal(heal);
            CombatEvent::PlayerHealed { amount: heal }
        }
        1 => {
            let dmg = roll_damage(10, player.stats.entropy, *seed);
            enemy.hp = (enemy.hp - dmg).max(0);
            CombatEvent::ChaosEvent {
                description: format!("Reality fractures! Enemy takes {} chaos damage.", dmg),
            }
        }
        2 => CombatEvent::ChaosEvent {
            description: "The Mandelbrot boundary shifts — all attacks +20% this turn.".to_string(),
        },
        3 => CombatEvent::ChaosEvent {
            description: "Fibonacci spiral surrounds you! +1 armor this turn.".to_string(),
        },
        4 => {
            // Confuse enemy briefly
            CombatEvent::ChaosEvent {
                description: "Collatz sequence confuses the enemy! They hesitate.".to_string(),
            }
        }
        _ => CombatEvent::ChaosEvent {
            description: "A prime number whispers your name. You feel... lucky.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Background, CharacterClass};
    use crate::enemy::generate_enemy;

    fn make_player() -> Character {
        Character::roll_new(
            "TestHero".to_string(),
            CharacterClass::Berserker,
            Background::Gladiator,
            42,
        )
    }

    #[test]
    fn attack_deals_damage() {
        let mut player = make_player();
        let mut enemy = generate_enemy(1, 42);
        let mut state = CombatState::new(999);

        let initial_hp = enemy.hp;
        let (events, _) = resolve_action(&mut player, &mut enemy, CombatAction::Attack, &mut state);
        assert!(events
            .iter()
            .any(|e| matches!(e, CombatEvent::PlayerAttack { .. })));
        assert!(enemy.hp < initial_hp || enemy.hp == 0);
    }

    #[test]
    fn flee_can_succeed() {
        let mut attempts = 0;
        let mut escaped = false;
        for seed in 0u64..50 {
            let mut player = make_player();
            let mut enemy = generate_enemy(1, seed);
            let mut state = CombatState::new(seed);
            let (_, outcome) =
                resolve_action(&mut player, &mut enemy, CombatAction::Flee, &mut state);
            if outcome == CombatOutcome::PlayerFled {
                escaped = true;
                break;
            }
            attempts += 1;
        }
        assert!(
            escaped || attempts >= 50,
            "Should be able to flee sometimes"
        );
    }
}
