//! Enemy AI systems — steering behaviors, behavior trees, GOAP, utility AI.
//!
//! Bridges proof-engine's AI module to chaos-rpg enemy behavior.

use proof_engine::prelude::*;
use crate::state::GameState;

// ═══════════════════════════════════════════════════════════════════════════════
// STEERING BEHAVIORS — enemy movement patterns in the combat arena
// ═══════════════════════════════════════════════════════════════════════════════

/// Enemy archetype determines steering behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyArchetype {
    Melee,      // seek + arrive
    Ranged,     // maintain distance
    Swarm,      // flock (separation, alignment, cohesion)
    Minion,     // formation around boss
    Fleeing,    // evade when low HP, wander otherwise
    Patrol,     // wander between waypoints
}

/// Steering output: desired velocity delta.
pub struct SteeringOutput {
    pub dx: f32,
    pub dy: f32,
}

impl SteeringOutput {
    pub fn zero() -> Self { Self { dx: 0.0, dy: 0.0 } }
}

/// Compute steering for an enemy based on archetype and positions.
pub fn compute_steering(
    archetype: EnemyArchetype,
    enemy_pos: Vec3,
    player_pos: Vec3,
    hp_frac: f32,
    dt: f32,
    frame: u64,
) -> SteeringOutput {
    match archetype {
        EnemyArchetype::Melee => steer_seek_arrive(enemy_pos, player_pos, 2.0, dt),
        EnemyArchetype::Ranged => steer_maintain_distance(enemy_pos, player_pos, 8.0, 3.0, dt),
        EnemyArchetype::Swarm => steer_flock(enemy_pos, player_pos, frame, dt),
        EnemyArchetype::Minion => steer_formation(enemy_pos, Vec3::new(6.0, 0.0, 0.0), frame, dt),
        EnemyArchetype::Fleeing => {
            if hp_frac < 0.2 {
                steer_evade(enemy_pos, player_pos, dt)
            } else {
                steer_wander(enemy_pos, frame, dt)
            }
        }
        EnemyArchetype::Patrol => steer_wander(enemy_pos, frame, dt),
    }
}

fn steer_seek_arrive(pos: Vec3, target: Vec3, arrive_radius: f32, dt: f32) -> SteeringOutput {
    let to_target = target - pos;
    let dist = (to_target.x * to_target.x + to_target.y * to_target.y).sqrt();
    if dist < 0.01 { return SteeringOutput::zero(); }
    let speed = if dist < arrive_radius { dist / arrive_radius } else { 1.0 };
    let nx = to_target.x / dist;
    let ny = to_target.y / dist;
    SteeringOutput { dx: nx * speed * dt * 3.0, dy: ny * speed * dt * 3.0 }
}

fn steer_maintain_distance(pos: Vec3, target: Vec3, ideal: f32, tolerance: f32, dt: f32) -> SteeringOutput {
    let to_target = target - pos;
    let dist = (to_target.x * to_target.x + to_target.y * to_target.y).sqrt();
    if dist < 0.01 { return SteeringOutput::zero(); }
    let nx = to_target.x / dist;
    let ny = to_target.y / dist;
    if dist < ideal - tolerance {
        // Too close: flee
        SteeringOutput { dx: -nx * dt * 2.0, dy: -ny * dt * 2.0 }
    } else if dist > ideal + tolerance {
        // Too far: pursue
        SteeringOutput { dx: nx * dt * 2.0, dy: ny * dt * 2.0 }
    } else {
        // In range: slight orbit
        SteeringOutput { dx: -ny * dt * 0.5, dy: nx * dt * 0.5 }
    }
}

fn steer_flock(pos: Vec3, player_pos: Vec3, frame: u64, dt: f32) -> SteeringOutput {
    // Simplified flock: seek player with oscillation for pack movement
    let to_player = player_pos - pos;
    let dist = (to_player.x * to_player.x + to_player.y * to_player.y).sqrt().max(0.01);
    let nx = to_player.x / dist;
    let ny = to_player.y / dist;
    let wobble_x = (frame as f32 * 0.07 + pos.x * 0.5).sin() * 0.5;
    let wobble_y = (frame as f32 * 0.09 + pos.y * 0.3).cos() * 0.5;
    SteeringOutput {
        dx: (nx * 0.6 + wobble_x) * dt * 2.0,
        dy: (ny * 0.6 + wobble_y) * dt * 2.0,
    }
}

fn steer_formation(pos: Vec3, leader_pos: Vec3, frame: u64, dt: f32) -> SteeringOutput {
    // Orbit around leader
    let angle = (frame as f32 * 0.02 + pos.x * 0.3 + pos.y * 0.7);
    let orbit_r = 3.0;
    let target = Vec3::new(
        leader_pos.x + angle.cos() * orbit_r,
        leader_pos.y + angle.sin() * orbit_r,
        0.0,
    );
    steer_seek_arrive(pos, target, 0.5, dt)
}

fn steer_evade(pos: Vec3, threat: Vec3, dt: f32) -> SteeringOutput {
    let away = pos - threat;
    let dist = (away.x * away.x + away.y * away.y).sqrt().max(0.01);
    SteeringOutput { dx: (away.x / dist) * dt * 4.0, dy: (away.y / dist) * dt * 4.0 }
}

fn steer_wander(pos: Vec3, frame: u64, dt: f32) -> SteeringOutput {
    let angle = (frame as f32 * 0.01 + pos.x * 1.7 + pos.y * 2.3).sin() * std::f32::consts::TAU;
    SteeringOutput { dx: angle.cos() * dt * 1.0, dy: angle.sin() * dt * 1.0 }
}

/// Classify an enemy name into an archetype.
pub fn classify_enemy(name: &str, is_boss: bool, hp_frac: f32) -> EnemyArchetype {
    if is_boss { return EnemyArchetype::Melee; } // Bosses use custom AI
    if hp_frac < 0.2 { return EnemyArchetype::Fleeing; }
    let name_lower = name.to_lowercase();
    if name_lower.contains("swarm") || name_lower.contains("pack") || name_lower.contains("horde") {
        EnemyArchetype::Swarm
    } else if name_lower.contains("archer") || name_lower.contains("mage") || name_lower.contains("caster") {
        EnemyArchetype::Ranged
    } else if name_lower.contains("guard") || name_lower.contains("sentinel") || name_lower.contains("patrol") {
        EnemyArchetype::Patrol
    } else {
        EnemyArchetype::Melee
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BEHAVIOR TREES — boss-specific decision making
// ═══════════════════════════════════════════════════════════════════════════════

/// Simplified behavior tree node result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BtResult { Success, Failure, Running }

/// The Accountant behavior tree — bill management.
pub fn accountant_tick(
    player_damage_total: i64,
    boss_hp_frac: f32,
    bill_ready: &mut bool,
    turn: u32,
) -> &'static str {
    // Check if bill is ready to send
    if *bill_ready {
        *bill_ready = false;
        return "send_bill";
    }
    // Check if player has dealt enough damage to trigger a bill
    let threshold = 100 + turn as i64 * 50;
    if player_damage_total > threshold {
        *bill_ready = true;
        return "prepare_bill";
    }
    // Low HP: defensive stance
    if boss_hp_frac < 0.3 {
        return "defensive_stance";
    }
    "basic_attack"
}

/// The Committee behavior tree — voting mechanic.
pub fn committee_tick(turn: u32, seed: u64) -> (Vec<bool>, bool, &'static str) {
    // 5 judges vote
    let mut votes = Vec::with_capacity(5);
    for j in 0..5u64 {
        let hash = seed.wrapping_mul(j + 1).wrapping_add(turn as u64 * 7919);
        votes.push((hash >> 16) % 3 != 0); // ~67% chance of approval
    }
    let approved = votes.iter().filter(|&&v| v).count();
    let passed = approved >= 3;
    let action = if passed { "execute_attack" } else { "attack_fails" };
    (votes, passed, action)
}

// ═══════════════════════════════════════════════════════════════════════════════
// GOAP — Algorithm Reborn goal-oriented planning
// ═══════════════════════════════════════════════════════════════════════════════

/// Algorithm Reborn action selection via GOAP-style scoring.
pub fn algorithm_reborn_plan(
    phase: u8,
    boss_hp_frac: f32,
    player_attack_counts: &[u32; 6], // attack, heavy, spell, defend, flee, taunt
    turn: u32,
) -> &'static str {
    // Identify player's most-used action
    let max_action = player_attack_counts.iter().enumerate()
        .max_by_key(|(_, &count)| count)
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Phase-dependent planning
    match phase {
        1 => {
            // Evaluation: basic attacks while observing
            if boss_hp_frac < 0.7 { return "phase_shift"; }
            "basic_attack"
        }
        2 => {
            // Adaptation: counter the player's strategy
            if boss_hp_frac < 0.4 { return "phase_shift"; }
            if boss_hp_frac < 0.5 { return "regenerate"; }
            match max_action {
                0 | 1 => "adapt_defense_physical",  // Player attacks a lot: boost physical resist
                2 => "adapt_defense_magic",          // Player uses spells: boost magic resist
                3 => "counter_defend",               // Player defends: use piercing attack
                _ => "counter_attack",               // Default: heavy counter
            }
        }
        _ => {
            // Phase 3: Counter-specialization — all-out
            if boss_hp_frac < 0.3 { return "regenerate"; }
            if turn % 3 == 0 { return "counter_attack"; }
            "heavy_counter"
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITY AI — general enemy action scoring
// ═══════════════════════════════════════════════════════════════════════════════

/// Score combat actions for a general enemy.
pub struct ActionScores {
    pub attack: f32,
    pub defend: f32,
    pub special: f32,
    pub flee: f32,
}

/// Compute utility scores for enemy combat actions.
pub fn score_enemy_actions(
    enemy_hp_frac: f32,
    player_hp_frac: f32,
    has_special: bool,
    special_cooldown_ready: bool,
    turn: u32,
) -> ActionScores {
    // Attack: score increases when player HP is low (logistic curve)
    let attack_score = logistic(1.0 - player_hp_frac, 0.5, 8.0) * 0.8 + 0.2;

    // Defend: score increases when own HP is low
    let defend_score = logistic(1.0 - enemy_hp_frac, 0.6, 6.0) * 0.6;

    // Special: score when cooldown ready and situation is advantageous
    let special_score = if has_special && special_cooldown_ready {
        let advantage = (1.0 - player_hp_frac) * enemy_hp_frac;
        advantage * 0.9 + 0.1
    } else {
        0.0
    };

    // Flee: score when HP < 20% and player HP > 50%
    let flee_score = if enemy_hp_frac < 0.2 && player_hp_frac > 0.5 {
        (1.0 - enemy_hp_frac) * 0.7
    } else {
        0.0
    };

    ActionScores {
        attack: attack_score,
        defend: defend_score,
        special: special_score,
        flee: flee_score,
    }
}

/// Select the highest-scoring action.
pub fn select_best_action(scores: &ActionScores) -> &'static str {
    let mut best = ("attack", scores.attack);
    if scores.defend > best.1 { best = ("defend", scores.defend); }
    if scores.special > best.1 { best = ("special", scores.special); }
    if scores.flee > best.1 { best = ("flee", scores.flee); }
    best.0
}

/// Logistic consideration curve: smooth threshold at `center` with steepness `k`.
fn logistic(x: f32, center: f32, k: f32) -> f32 {
    1.0 / (1.0 + (-k * (x - center)).exp())
}
