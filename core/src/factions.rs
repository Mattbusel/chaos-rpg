//! Faction and reputation system.
//!
//! Three factions, each aligned with a mathematical philosophy:
//!   Order of Convergence  -- taming chaos, stability, predictability
//!   Cult of Divergence    -- maximizing chaos, variance, extremes
//!   Watchers of Boundary  -- the Mandelbrot boundary, threshold effects, crits

use serde::{Deserialize, Serialize};

// ─── FACTION ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    OrderOfConvergence,
    CultOfDivergence,
    WatchersOfBoundary,
}

impl Faction {
    pub fn name(self) -> &'static str {
        match self {
            Faction::OrderOfConvergence => "Order of Convergence",
            Faction::CultOfDivergence => "Cult of Divergence",
            Faction::WatchersOfBoundary => "Watchers of the Boundary",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Faction::OrderOfConvergence => "ORDER",
            Faction::CultOfDivergence => "CULT",
            Faction::WatchersOfBoundary => "WATCH",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Faction::OrderOfConvergence => "\x1b[34m", // blue -- order
            Faction::CultOfDivergence => "\x1b[31m",   // red -- chaos
            Faction::WatchersOfBoundary => "\x1b[35m", // purple -- boundary
        }
    }

    pub fn philosophy(self) -> &'static str {
        match self {
            Faction::OrderOfConvergence => {
                "Chaos is a disease. We are the cure. Every algorithm must be tamed."
            }
            Faction::CultOfDivergence => {
                "The ceiling is a suggestion. We remove suggestions. Maximum engines, always."
            }
            Faction::WatchersOfBoundary => {
                "The boundary between order and chaos is where everything interesting happens."
            }
        }
    }

    pub fn all() -> [Faction; 3] {
        [
            Faction::OrderOfConvergence,
            Faction::CultOfDivergence,
            Faction::WatchersOfBoundary,
        ]
    }
}

// ─── REPUTATION TIER ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReputationTier {
    Hostile,    // < -200
    Neutral,    // -200 to 0
    Recognized, // 1 to 200
    Trusted,    // 201 to 500
    Exalted,    // > 500
}

impl ReputationTier {
    pub fn from_rep(rep: i32) -> Self {
        match rep {
            i32::MIN..=-201 => ReputationTier::Hostile,
            -200..=0 => ReputationTier::Neutral,
            1..=200 => ReputationTier::Recognized,
            201..=500 => ReputationTier::Trusted,
            _ => ReputationTier::Exalted,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ReputationTier::Hostile => "HOSTILE",
            ReputationTier::Neutral => "Neutral",
            ReputationTier::Recognized => "Recognized",
            ReputationTier::Trusted => "Trusted",
            ReputationTier::Exalted => "EXALTED",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            ReputationTier::Hostile => "\x1b[31m",
            ReputationTier::Neutral => "\x1b[37m",
            ReputationTier::Recognized => "\x1b[32m",
            ReputationTier::Trusted => "\x1b[33m",
            ReputationTier::Exalted => "\x1b[97m",
        }
    }
}

// ─── FACTION REP STATE ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionRep {
    pub order: i32,
    pub cult: i32,
    pub watchers: i32,
}

impl FactionRep {
    pub fn get(&self, faction: Faction) -> i32 {
        match faction {
            Faction::OrderOfConvergence => self.order,
            Faction::CultOfDivergence => self.cult,
            Faction::WatchersOfBoundary => self.watchers,
        }
    }

    pub fn add(&mut self, faction: Faction, amount: i32) {
        let other_penalty = -(amount / 3).max(1);
        match faction {
            Faction::OrderOfConvergence => {
                self.order += amount;
                self.cult += other_penalty;
            }
            Faction::CultOfDivergence => {
                self.cult += amount;
                self.order += other_penalty;
            }
            Faction::WatchersOfBoundary => {
                self.watchers += amount;
                // Watchers don't anger the others as much
            }
        }
    }

    pub fn tier(&self, faction: Faction) -> ReputationTier {
        ReputationTier::from_rep(self.get(faction))
    }

    /// Returns faction-specific passive bonus description (if Trusted+)
    pub fn passive_bonus(faction: Faction, tier: ReputationTier) -> Option<&'static str> {
        if tier < ReputationTier::Trusted {
            return None;
        }
        Some(match faction {
            Faction::OrderOfConvergence => {
                "ORDER PACT: Chain length reduced by 1 (min 3). More predictable rolls."
            }
            Faction::CultOfDivergence => {
                "CULT PACT: Chain length increased by 2. Higher ceiling, lower floor."
            }
            Faction::WatchersOfBoundary => {
                "WATCHER PACT: Crit threshold lowered to 65. Near-zero rolls trigger bonus effects."
            }
        })
    }
}

// ─── FACTION VENDOR DIALOGUE ──────────────────────────────────────────────────

pub fn vendor_greeting(faction: Faction, tier: ReputationTier) -> &'static str {
    match (faction, tier) {
        (Faction::OrderOfConvergence, ReputationTier::Hostile) => {
            "The Order does not associate with chaos-embracers. Leave."
        }
        (Faction::OrderOfConvergence, ReputationTier::Neutral) => {
            "We sell tools of convergence. Keep your chaos away from our wares."
        }
        (Faction::OrderOfConvergence, ReputationTier::Recognized) => {
            "Ah. You show some potential for discipline. Browse our convergence tools."
        }
        (Faction::OrderOfConvergence, ReputationTier::Trusted) => {
            "Brother. The algorithms can be tamed. We have the tools."
        }
        (Faction::OrderOfConvergence, ReputationTier::Exalted) => {
            "Champion of Convergence. The prime numbers bow to your will. Take what you need."
        }
        (Faction::CultOfDivergence, ReputationTier::Hostile) => {
            "Your rolls are too small. You fear the ceiling. Come back when you've lost everything."
        }
        (Faction::CultOfDivergence, ReputationTier::Neutral) => {
            "Most people fear what we sell. Smart people buy everything."
        }
        (Faction::CultOfDivergence, ReputationTier::Recognized) => {
            "You've seen the beautiful chaos. Let us show you more."
        }
        (Faction::CultOfDivergence, ReputationTier::Trusted) => {
            "True believer. The Lorenz butterfly knows your name. Buy freely."
        }
        (Faction::CultOfDivergence, ReputationTier::Exalted) => {
            "The DIVERGENT ONE arrives. Reality is your dice. The shop is yours."
        }
        (Faction::WatchersOfBoundary, ReputationTier::Hostile) => {
            "...You haven't seen the boundary. You don't even know it exists."
        }
        (Faction::WatchersOfBoundary, ReputationTier::Neutral) => {
            "The boundary between order and chaos. That's where we live. Care to visit?"
        }
        (Faction::WatchersOfBoundary, ReputationTier::Recognized) => {
            "You begin to see the boundary. The threshold effects. The critical moments."
        }
        (Faction::WatchersOfBoundary, ReputationTier::Trusted) => {
            "The Mandelbrot set's edge is your home now. Welcome."
        }
        (Faction::WatchersOfBoundary, ReputationTier::Exalted) => {
            "You ARE the boundary. Neither chaos nor order. Something better."
        }
    }
}

// ─── FACTION QUEST ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionQuest {
    pub faction: Faction,
    pub description: String,
    pub reward_rep: i32,
    pub reward_gold: i64,
    pub completed: bool,
}

impl FactionQuest {
    pub fn generate(faction: Faction, floor: u32, seed: u64) -> Self {
        let (desc, rep, gold) = match faction {
            Faction::OrderOfConvergence => {
                let quests = [
                    (
                        "Kill 3 enemies using only Defend actions (stability demonstration)",
                        50,
                        150,
                    ),
                    (
                        "Complete a floor without using any spells (discipline trial)",
                        75,
                        200,
                    ),
                    (
                        "Survive 10 rounds against a boss (endurance through convergence)",
                        100,
                        300,
                    ),
                ];
                let idx = (seed % quests.len() as u64) as usize;
                quests[idx]
            }
            Faction::CultOfDivergence => {
                let quests = [
                    (
                        "Deal a single hit exceeding 1000 damage (chaos multiplication)",
                        50,
                        100,
                    ),
                    (
                        "Die and continue 3 times in one run (embrace divergence)",
                        60,
                        80,
                    ),
                    (
                        "Roll a critical hit using a spell (divergence amplified)",
                        80,
                        200,
                    ),
                ];
                let idx = (seed % quests.len() as u64) as usize;
                quests[idx]
            }
            Faction::WatchersOfBoundary => {
                let quests = [
                    (
                        "Land exactly 0 damage on an enemy (touch the boundary)",
                        60,
                        175,
                    ),
                    (
                        "Survive with exactly 1 HP at end of a fight (boundary survival)",
                        80,
                        250,
                    ),
                    (
                        "Roll a chaos value between -0.05 and 0.05 (touch zero)",
                        100,
                        300,
                    ),
                ];
                let idx = (seed % quests.len() as u64) as usize;
                quests[idx]
            }
        };
        FactionQuest {
            faction,
            description: format!("[Floor {}] {}", floor, desc),
            reward_rep: rep,
            reward_gold: gold + (floor as i64 * 20),
            completed: false,
        }
    }
}
