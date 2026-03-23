//! Enemy AI state machine with behavior archetypes.
//!
//! Provides a self-contained finite state machine for enemy AI that handles
//! state transitions based on player proximity, visibility, and health thresholds.

use serde::{Deserialize, Serialize};

// ─── AI STATE ─────────────────────────────────────────────────────────────────

/// Current behavioral state of an enemy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiState {
    /// Unaware of the player, standing still.
    Idle,
    /// Moving between patrol waypoints.
    Patrol { waypoints: Vec<(u32, u32)>, current: usize },
    /// Heard or briefly glimpsed the player; searching.
    Alerted { source: (u32, u32) },
    /// Actively pursuing the player.
    Chasing { target: (u32, u32) },
    /// Player is adjacent; melee engagement.
    Attacking,
    /// Retreating to survive.
    Fleeing { threshold_hp_pct: f64 },
    /// Enemy has been killed.
    Dead,
}

// ─── AI BEHAVIOR ──────────────────────────────────────────────────────────────

/// Behavioral parameters for an enemy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBehavior {
    pub state: AiState,
    /// Aggression factor in [0, 1]. Higher values mean the enemy fights longer.
    pub aggression: f64,
    /// Player must be within this many tiles to be detected.
    pub detection_radius: u32,
    /// HP percentage below which the enemy considers fleeing.
    pub flee_threshold: f64,
}

// ─── AI DECISION ──────────────────────────────────────────────────────────────

/// Action the AI has decided to take this tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiDecision {
    MoveToward((u32, u32)),
    MoveAway((u32, u32)),
    Attack,
    UseAbility(String),
    Idle,
    Patrol,
}

// ─── ENEMY AI ─────────────────────────────────────────────────────────────────

/// The full enemy AI entity combining behavior with stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyAi {
    pub behavior: AiBehavior,
    pub hp: u32,
    pub max_hp: u32,
    pub position: (u32, u32),
    /// Internal counter used for Alerted → Chasing transition.
    alerted_turns: u32,
}

impl EnemyAi {
    /// Create a new enemy AI with the given parameters.
    pub fn new(
        position: (u32, u32),
        hp: u32,
        aggression: f64,
        detection_radius: u32,
        flee_threshold: f64,
    ) -> Self {
        Self {
            behavior: AiBehavior {
                state: AiState::Idle,
                aggression,
                detection_radius,
                flee_threshold,
            },
            hp,
            max_hp: hp,
            position,
            alerted_turns: 0,
        }
    }

    /// Current HP as a fraction of max HP.
    pub fn hp_pct(&self) -> f64 {
        if self.max_hp == 0 {
            return 0.0;
        }
        self.hp as f64 / self.max_hp as f64
    }

    /// Manhattan distance between two grid positions.
    pub fn manhattan_distance(a: (u32, u32), b: (u32, u32)) -> u32 {
        let dx = if a.0 > b.0 { a.0 - b.0 } else { b.0 - a.0 };
        let dy = if a.1 > b.1 { a.1 - b.1 } else { b.1 - a.1 };
        dx + dy
    }

    /// Advance the AI state machine by one turn and return the decision made.
    ///
    /// # Parameters
    /// - `player_pos`: The player's current position, or `None` if unknown.
    /// - `player_visible`: Whether the player is in line of sight this tick.
    pub fn tick(&mut self, player_pos: Option<(u32, u32)>, player_visible: bool) -> AiDecision {
        // Terminal state — no further transitions.
        if self.hp == 0 {
            self.behavior.state = AiState::Dead;
            return AiDecision::Idle;
        }

        let current_hp_pct = self.hp_pct();
        let flee_th = self.behavior.flee_threshold;
        let aggression = self.behavior.aggression;
        let detection_radius = self.behavior.detection_radius;

        // Clone to avoid borrow issues with self.
        let state = std::mem::replace(&mut self.behavior.state, AiState::Idle);

        let (new_state, decision) = match state {
            AiState::Dead => (AiState::Dead, AiDecision::Idle),

            AiState::Idle => {
                if player_visible {
                    if let Some(pos) = player_pos {
                        if Self::manhattan_distance(self.position, pos) <= detection_radius {
                            self.alerted_turns = 0;
                            return self.transition_to_alerted_or_chase(pos, player_visible);
                        }
                    }
                }
                (AiState::Idle, AiDecision::Idle)
            }

            AiState::Patrol { ref waypoints, ref current } => {
                if player_visible {
                    if let Some(pos) = player_pos {
                        if Self::manhattan_distance(self.position, pos) <= detection_radius {
                            self.alerted_turns = 0;
                            let waypoints_clone = waypoints.clone();
                            let _ = waypoints_clone; // drop
                            return self.transition_to_alerted_or_chase(pos, player_visible);
                        }
                    }
                }
                // Continue patrol.
                if waypoints.is_empty() {
                    (AiState::Patrol { waypoints: vec![], current: 0 }, AiDecision::Idle)
                } else {
                    let _dest = waypoints[*current];
                    let next = (*current + 1) % waypoints.len();
                    let waypoints_clone = waypoints.clone();
                    (AiState::Patrol { waypoints: waypoints_clone, current: next }, AiDecision::Patrol)
                }
            }

            AiState::Alerted { source } => {
                self.alerted_turns += 1;
                if player_visible {
                    if let Some(pos) = player_pos {
                        // Seen player — immediately chase.
                        (AiState::Chasing { target: pos }, AiDecision::MoveToward(pos))
                    } else {
                        if self.alerted_turns >= 2 {
                            (AiState::Chasing { target: source }, AiDecision::MoveToward(source))
                        } else {
                            (AiState::Alerted { source }, AiDecision::MoveToward(source))
                        }
                    }
                } else if self.alerted_turns >= 2 {
                    (AiState::Chasing { target: source }, AiDecision::MoveToward(source))
                } else {
                    (AiState::Alerted { source }, AiDecision::MoveToward(source))
                }
            }

            AiState::Chasing { target } => {
                let effective_target = player_pos.unwrap_or(target);
                let dist = Self::manhattan_distance(self.position, effective_target);

                if dist <= 1 {
                    // Adjacent — begin attacking.
                    (AiState::Attacking, AiDecision::Attack)
                } else {
                    (AiState::Chasing { target: effective_target }, AiDecision::MoveToward(effective_target))
                }
            }

            AiState::Attacking => {
                // Should we flee?
                if current_hp_pct < flee_th && aggression < 0.5 {
                    let flee_from = player_pos.unwrap_or(self.position);
                    return self.start_fleeing(flee_from, flee_th);
                }

                if let Some(pos) = player_pos {
                    let dist = Self::manhattan_distance(self.position, pos);
                    if dist <= 1 {
                        (AiState::Attacking, AiDecision::Attack)
                    } else {
                        (AiState::Chasing { target: pos }, AiDecision::MoveToward(pos))
                    }
                } else {
                    (AiState::Idle, AiDecision::Idle)
                }
            }

            AiState::Fleeing { threshold_hp_pct } => {
                if self.hp == 0 {
                    (AiState::Dead, AiDecision::Idle)
                } else if current_hp_pct >= threshold_hp_pct {
                    // Recovered enough to reconsider.
                    (AiState::Idle, AiDecision::Idle)
                } else {
                    let flee_from = player_pos.unwrap_or(self.position);
                    (AiState::Fleeing { threshold_hp_pct }, AiDecision::MoveAway(flee_from))
                }
            }
        };

        self.behavior.state = new_state;
        decision
    }

    fn transition_to_alerted_or_chase(&mut self, pos: (u32, u32), _visible: bool) -> AiDecision {
        self.behavior.state = AiState::Alerted { source: pos };
        AiDecision::MoveToward(pos)
    }

    fn start_fleeing(&mut self, flee_from: (u32, u32), threshold: f64) -> AiDecision {
        self.behavior.state = AiState::Fleeing { threshold_hp_pct: threshold };
        AiDecision::MoveAway(flee_from)
    }
}

// ─── AI ARCHETYPE ─────────────────────────────────────────────────────────────

/// Pre-defined AI behavior templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiArchetype {
    /// Always attacks; never flees.
    Aggressive,
    /// Balanced; flees at low HP.
    Defensive,
    /// Flees at moderate HP loss.
    Coward,
    /// Fights harder when wounded.
    Berserk,
    /// Patrols until player spotted.
    Patrol,
}

impl AiArchetype {
    /// Build an [`EnemyAi`] from this archetype at the given position with the given HP.
    pub fn build(archetype: AiArchetype, position: (u32, u32), hp: u32) -> EnemyAi {
        match archetype {
            AiArchetype::Aggressive => EnemyAi::new(position, hp, 1.0, 8, 0.0),
            AiArchetype::Defensive => EnemyAi::new(position, hp, 0.4, 6, 0.25),
            AiArchetype::Coward => EnemyAi::new(position, hp, 0.2, 10, 0.6),
            AiArchetype::Berserk => {
                let mut ai = EnemyAi::new(position, hp, 1.0, 7, 0.0);
                ai.behavior.aggression = 1.0;
                ai
            }
            AiArchetype::Patrol => {
                let mut ai = EnemyAi::new(position, hp, 0.5, 6, 0.2);
                ai.behavior.state = AiState::Patrol {
                    waypoints: vec![
                        position,
                        (position.0 + 3, position.1),
                        (position.0 + 3, position.1 + 3),
                        (position.0, position.1 + 3),
                    ],
                    current: 0,
                };
                ai
            }
        }
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_becomes_alerted_when_player_visible() {
        let mut ai = EnemyAi::new((0, 0), 100, 0.5, 8, 0.2);
        let player_pos = Some((3, 0)); // within detection radius
        let decision = ai.tick(player_pos, true);

        assert!(matches!(decision, AiDecision::MoveToward(_)));
        assert!(matches!(ai.behavior.state, AiState::Alerted { .. }));
    }

    #[test]
    fn idle_stays_idle_when_player_out_of_range() {
        let mut ai = EnemyAi::new((0, 0), 100, 0.5, 4, 0.2);
        let player_pos = Some((10, 0)); // outside detection radius
        let decision = ai.tick(player_pos, true);

        assert_eq!(decision, AiDecision::Idle);
        assert!(matches!(ai.behavior.state, AiState::Idle));
    }

    #[test]
    fn chasing_becomes_attacking_when_adjacent() {
        let mut ai = EnemyAi::new((0, 0), 100, 0.8, 8, 0.1);
        ai.behavior.state = AiState::Chasing { target: (1, 0) };

        let decision = ai.tick(Some((1, 0)), true);
        assert_eq!(decision, AiDecision::Attack);
        assert!(matches!(ai.behavior.state, AiState::Attacking));
    }

    #[test]
    fn flees_when_hp_low_and_aggression_low() {
        let mut ai = EnemyAi::new((5, 5), 100, 0.3, 6, 0.3); // aggression < 0.5
        ai.behavior.state = AiState::Attacking;
        ai.hp = 20; // 20% HP, below flee threshold 0.3

        let decision = ai.tick(Some((4, 5)), true);
        assert!(matches!(decision, AiDecision::MoveAway(_)));
        assert!(matches!(ai.behavior.state, AiState::Fleeing { .. }));
    }

    #[test]
    fn dead_state_is_terminal() {
        let mut ai = EnemyAi::new((0, 0), 100, 0.8, 8, 0.1);
        ai.hp = 0;

        let decision1 = ai.tick(Some((1, 0)), true);
        let decision2 = ai.tick(Some((1, 0)), true);

        assert_eq!(decision1, AiDecision::Idle);
        assert_eq!(decision2, AiDecision::Idle);
        assert!(matches!(ai.behavior.state, AiState::Dead));
    }

    #[test]
    fn manhattan_distance_correct() {
        assert_eq!(EnemyAi::manhattan_distance((0, 0), (3, 4)), 7);
        assert_eq!(EnemyAi::manhattan_distance((5, 5), (5, 5)), 0);
        assert_eq!(EnemyAi::manhattan_distance((0, 0), (1, 0)), 1);
    }

    #[test]
    fn archetype_builds_correctly() {
        let ai = AiArchetype::build(AiArchetype::Coward, (10, 10), 50);
        assert!(ai.behavior.aggression < 0.5);
        assert!(ai.behavior.flee_threshold > 0.4);

        let ai2 = AiArchetype::build(AiArchetype::Aggressive, (0, 0), 100);
        assert!((ai2.behavior.aggression - 1.0).abs() < f64::EPSILON);
    }
}
