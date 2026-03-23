//! Path of Exile-style passive skill tree.
//!
//! ~50 nodes arranged on a conceptual grid. 8 class starting positions branch
//! outward to shared nodes. Use [W/S/A/D] to navigate, [Enter] to allocate,
//! [Q] to exit.
//!
//! Node types:
//!   Stat     — flat stat bonus, amount chaos-rolled on allocation (you don't know until you commit)
//!   Engine   — modifies how a specific chaos engine behaves for this character
//!   Keystone — major build-defining choice; powerful but with a trade-off
//!   Synergy  — weak alone; allocate all nodes in a cluster to unlock a bonus

use crate::chaos_pipeline::chaos_roll_verbose;
use crate::character::CharacterClass;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ─── NODE TYPES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum NodeType {
    /// Flat bonus to a stat. The actual value is chaos-rolled when you allocate it.
    Stat {
        stat: &'static str,
        min: i64,
        max: i64,
    },
    /// Modifies how a specific chaos engine behaves.
    Engine {
        engine: &'static str,
        effect: &'static str,
    },
    /// Major keystone — powerful effect with a trade-off.
    Keystone { id: &'static str },
    /// Synergy cluster — weak alone, powerful together.
    Synergy {
        cluster: u8,
        bonus_desc: &'static str,
    },
}

// ─── TREE NODE ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: u16,
    /// Position on a 40×20 character grid for ASCII display.
    pub x: i16,
    pub y: i16,
    pub name: &'static str,
    pub short_desc: &'static str,
    pub node_type: NodeType,
    /// IDs of adjacent allocated nodes required to unlock this node
    /// (at least one must be allocated).
    pub requires: &'static [u16],
    /// If Some(class), this is the starting node for that class.
    pub class_start: Option<CharacterClass>,
}

// ─── KEYSTONE IDS ─────────────────────────────────────────────────────────────

pub const KS_CHAOS_IMMUNITY: &str = "ChaosImmunity";
pub const KS_ENTROPY_INVERSION: &str = "EntropyInversion";
pub const KS_MATH_CERTAINTY: &str = "MathCertainty";
pub const KS_GLASS_CANNON: &str = "GlassCannon";
pub const KS_RESONANCE_ECHO: &str = "ResonanceEcho";
pub const KS_PRIME_BLOOD: &str = "PrimeBloodKeystone";
pub const KS_VOID_STEP: &str = "VoidStep";
pub const KS_DEATH_PACT: &str = "DeathPact";

// ─── STATIC NODE TABLE ────────────────────────────────────────────────────────

pub const NODES: &[TreeNode] = &[
    // ── CLASS STARTING NODES ───────────────────────────────────────────────
    TreeNode { id: 0,  x: 4,  y: 10, name: "Arcane Origin",     short_desc: "+MANA +ENTROPY start",      node_type: NodeType::Stat { stat: "mana",      min: 10, max: 40 }, requires: &[],   class_start: Some(CharacterClass::Mage) },
    TreeNode { id: 1,  x: 36, y: 10, name: "Rage Origin",       short_desc: "+FORCE +VIT start",         node_type: NodeType::Stat { stat: "force",     min: 10, max: 40 }, requires: &[],   class_start: Some(CharacterClass::Berserker) },
    TreeNode { id: 2,  x: 20, y: 2,  name: "Prime Origin",      short_desc: "+PRECISION +LUCK start",    node_type: NodeType::Stat { stat: "precision", min: 10, max: 40 }, requires: &[],   class_start: Some(CharacterClass::Ranger) },
    TreeNode { id: 3,  x: 20, y: 18, name: "Shadow Origin",     short_desc: "+CUNNING +LUCK start",      node_type: NodeType::Stat { stat: "cunning",   min: 10, max: 40 }, requires: &[],   class_start: Some(CharacterClass::Thief) },
    TreeNode { id: 4,  x: 6,  y: 4,  name: "Death Origin",      short_desc: "+ENTROPY +MANA start",      node_type: NodeType::Stat { stat: "entropy",   min: 10, max: 40 }, requires: &[],   class_start: Some(CharacterClass::Necromancer) },
    TreeNode { id: 5,  x: 34, y: 4,  name: "Flask Origin",      short_desc: "+CUNNING +MANA start",      node_type: NodeType::Stat { stat: "cunning",   min: 10, max: 35 }, requires: &[],   class_start: Some(CharacterClass::Alchemist) },
    TreeNode { id: 6,  x: 6,  y: 16, name: "Divine Origin",     short_desc: "+VIT +FORCE start",         node_type: NodeType::Stat { stat: "vitality",  min: 10, max: 40 }, requires: &[],   class_start: Some(CharacterClass::Paladin) },
    TreeNode { id: 7,  x: 34, y: 16, name: "Void Origin",       short_desc: "+ENTROPY +LUCK start",      node_type: NodeType::Stat { stat: "entropy",   min: 8,  max: 35 }, requires: &[],   class_start: Some(CharacterClass::VoidWalker) },

    // ── MAGE BRANCH ────────────────────────────────────────────────────────
    TreeNode { id: 10, x: 7,  y: 9,  name: "Mana Surge",        short_desc: "+MANA (big range)",          node_type: NodeType::Stat { stat: "mana",    min: -5, max: 60 }, requires: &[0],  class_start: None },
    TreeNode { id: 11, x: 7,  y: 11, name: "Spell Weave",       short_desc: "+ENTROPY for spell depth",   node_type: NodeType::Stat { stat: "entropy", min: 5,  max: 30 }, requires: &[0],  class_start: None },
    TreeNode { id: 12, x: 9,  y: 8,  name: "Zeta Gambler",      short_desc: "Riemann Zeta uses s near 1 — higher ceiling, lower floor", node_type: NodeType::Engine { engine: "Riemann Zeta Partial", effect: "volatile" }, requires: &[10], class_start: None },
    TreeNode { id: 13, x: 9,  y: 12, name: "Lorenz Stabilizer", short_desc: "Lorenz outputs clamped to positive values for you",        node_type: NodeType::Engine { engine: "Lorenz Attractor",     effect: "stabilize" }, requires: &[11], class_start: None },
    TreeNode { id: 14, x: 11, y: 7,  name: "Glass Cannon",      short_desc: "HP always 1. Damage chain uses 15 engines.", node_type: NodeType::Keystone { id: KS_GLASS_CANNON },   requires: &[12], class_start: None },

    // ── BERSERKER BRANCH ───────────────────────────────────────────────────
    TreeNode { id: 20, x: 33, y: 9,  name: "Iron Skin",         short_desc: "+VITALITY (big range)",      node_type: NodeType::Stat { stat: "vitality", min: -5, max: 60 }, requires: &[1],  class_start: None },
    TreeNode { id: 21, x: 33, y: 11, name: "Reckless Power",    short_desc: "+FORCE, -PRECISION",         node_type: NodeType::Stat { stat: "force",    min: 5,  max: 50 }, requires: &[1],  class_start: None },
    TreeNode { id: 22, x: 31, y: 8,  name: "Collatz Shortcut",  short_desc: "Collatz chains capped at 100 steps (less pathological)", node_type: NodeType::Engine { engine: "Collatz Chain", effect: "cap100" }, requires: &[20], class_start: None },
    TreeNode { id: 23, x: 29, y: 7,  name: "Chaos Immunity",    short_desc: "Never take >50% max HP in one hit. Damage also capped at 50%.", node_type: NodeType::Keystone { id: KS_CHAOS_IMMUNITY }, requires: &[22], class_start: None },

    // ── RANGER BRANCH ──────────────────────────────────────────────────────
    TreeNode { id: 30, x: 19, y: 4,  name: "Prime Sight",       short_desc: "+PRECISION (wide range)",    node_type: NodeType::Stat { stat: "precision", min: 5,  max: 55 }, requires: &[2],  class_start: None },
    TreeNode { id: 31, x: 21, y: 4,  name: "Lucky Shot",        short_desc: "+LUCK (very wide range)",    node_type: NodeType::Stat { stat: "luck",      min: -10,max: 70 }, requires: &[2],  class_start: None },
    TreeNode { id: 32, x: 19, y: 6,  name: "Mandelbrot Magnet", short_desc: "Mandelbrot samples boundary — maximum chaos", node_type: NodeType::Engine { engine: "Mandelbrot Escape", effect: "boundary" }, requires: &[30], class_start: None },
    TreeNode { id: 33, x: 19, y: 8,  name: "Math Certainty",    short_desc: "Always exactly 4 engines. Always the same 4. No variance. No surprises.", node_type: NodeType::Keystone { id: KS_MATH_CERTAINTY }, requires: &[32], class_start: None },

    // ── THIEF BRANCH ───────────────────────────────────────────────────────
    TreeNode { id: 40, x: 19, y: 16, name: "Shadow Step",       short_desc: "+CUNNING (wide range)",      node_type: NodeType::Stat { stat: "cunning",   min: 5,  max: 50 }, requires: &[3],  class_start: None },
    TreeNode { id: 41, x: 21, y: 16, name: "Fortune's Favor",   short_desc: "+LUCK, chaotic",             node_type: NodeType::Stat { stat: "luck",      min: -15,max: 80 }, requires: &[3],  class_start: None },
    TreeNode { id: 42, x: 21, y: 14, name: "Fibonacci Harmony", short_desc: "Fibonacci engine outputs doubled", node_type: NodeType::Engine { engine: "Fibonacci Golden Spiral", effect: "double" }, requires: &[40], class_start: None },

    // ── NECROMANCER BRANCH ─────────────────────────────────────────────────
    TreeNode { id: 50, x: 7,  y: 5,  name: "Death Resonance",   short_desc: "+ENTROPY, negative side",    node_type: NodeType::Stat { stat: "entropy",   min: -5, max: 55 }, requires: &[4],  class_start: None },
    TreeNode { id: 51, x: 8,  y: 7,  name: "Void Drain",        short_desc: "+MANA from chaos",           node_type: NodeType::Stat { stat: "mana",      min: 5,  max: 45 }, requires: &[50], class_start: None },
    TreeNode { id: 52, x: 10, y: 6,  name: "Entropy Inversion", short_desc: "All negative rolls become positive. All positive become negative. You live in the mirror.", node_type: NodeType::Keystone { id: KS_ENTROPY_INVERSION }, requires: &[51], class_start: None },
    TreeNode { id: 53, x: 10, y: 9,  name: "Death Pact",        short_desc: "Die at 0 HP but deal your full remaining HP as a death strike.", node_type: NodeType::Keystone { id: KS_DEATH_PACT }, requires: &[51], class_start: None },

    // ── ALCHEMIST BRANCH ───────────────────────────────────────────────────
    TreeNode { id: 60, x: 33, y: 5,  name: "Flask Master",      short_desc: "+CUNNING for item potency",  node_type: NodeType::Stat { stat: "cunning",   min: 5,  max: 40 }, requires: &[5],  class_start: None },
    TreeNode { id: 61, x: 31, y: 6,  name: "Reagent Boost",     short_desc: "+MANA via alchemy",          node_type: NodeType::Stat { stat: "mana",      min: 5,  max: 35 }, requires: &[5],  class_start: None },
    TreeNode { id: 62, x: 30, y: 9,  name: "Collatz Optimizer", short_desc: "Collatz chains start even — immediate halving, less volatile", node_type: NodeType::Engine { engine: "Collatz Chain", effect: "even_start" }, requires: &[60], class_start: None },

    // ── PALADIN BRANCH ─────────────────────────────────────────────────────
    TreeNode { id: 70, x: 7,  y: 15, name: "Holy Fortitude",    short_desc: "+VITALITY (divine bonus)",   node_type: NodeType::Stat { stat: "vitality",  min: 5,  max: 45 }, requires: &[6],  class_start: None },
    TreeNode { id: 71, x: 8,  y: 13, name: "Sacred Force",      short_desc: "+FORCE from faith",          node_type: NodeType::Stat { stat: "force",     min: 5,  max: 40 }, requires: &[6],  class_start: None },
    TreeNode { id: 72, x: 10, y: 14, name: "Euler's Grace",     short_desc: "Euler Totient engine produces only positive outputs", node_type: NodeType::Engine { engine: "Euler's Totient", effect: "positive" }, requires: &[70], class_start: None },
    TreeNode { id: 73, x: 12, y: 13, name: "Resonance Echo",    short_desc: "Chaos roll output feeds into the next roll's input. Streaks emerge.", node_type: NodeType::Keystone { id: KS_RESONANCE_ECHO }, requires: &[72], class_start: None },

    // ── VOIDWALKER BRANCH ──────────────────────────────────────────────────
    TreeNode { id: 80, x: 33, y: 15, name: "Phase Mastery",     short_desc: "+ENTROPY for phasing",       node_type: NodeType::Stat { stat: "entropy",   min: 5,  max: 45 }, requires: &[7],  class_start: None },
    TreeNode { id: 81, x: 31, y: 14, name: "Void Luck",         short_desc: "+LUCK from the void",        node_type: NodeType::Stat { stat: "luck",      min: 5,  max: 50 }, requires: &[7],  class_start: None },
    TreeNode { id: 82, x: 29, y: 13, name: "Void Step",         short_desc: "Phase dodge triggers even when Phasing status has expired.", node_type: NodeType::Keystone { id: KS_VOID_STEP }, requires: &[80], class_start: None },

    // ── SHARED CENTER NODES ────────────────────────────────────────────────
    TreeNode { id: 90, x: 16, y: 10, name: "Chaos Attunement",  short_desc: "+ENTROPY (shared)",          node_type: NodeType::Stat { stat: "entropy",   min: 3,  max: 25 }, requires: &[0, 4, 6, 13, 51, 70], class_start: None },
    TreeNode { id: 91, x: 24, y: 10, name: "Force Conduit",     short_desc: "+FORCE (shared)",            node_type: NodeType::Stat { stat: "force",     min: 3,  max: 25 }, requires: &[1, 5, 7, 22, 60, 80], class_start: None },
    TreeNode { id: 92, x: 20, y: 10, name: "Mathematical Core", short_desc: "All stats +chaos-rolled bonus", node_type: NodeType::Stat { stat: "luck",  min: 5,  max: 20 }, requires: &[90, 91], class_start: None },

    // ── FRACTAL MASTERY SYNERGY CLUSTER (5 nodes, need all 5) ─────────────
    TreeNode { id: 100, x: 15, y: 7,  name: "Fractal I",   short_desc: "Fractal Mastery cluster 1/5 — Mandelbrot outputs ×2 when all 5 allocated", node_type: NodeType::Synergy { cluster: 1, bonus_desc: "Mandelbrot+Fibonacci outputs doubled" }, requires: &[0, 2, 90], class_start: None },
    TreeNode { id: 101, x: 17, y: 6,  name: "Fractal II",  short_desc: "Fractal Mastery cluster 2/5",                                               node_type: NodeType::Synergy { cluster: 1, bonus_desc: "Mandelbrot+Fibonacci outputs doubled" }, requires: &[100], class_start: None },
    TreeNode { id: 102, x: 20, y: 6,  name: "Fractal III", short_desc: "Fractal Mastery cluster 3/5",                                               node_type: NodeType::Synergy { cluster: 1, bonus_desc: "Mandelbrot+Fibonacci outputs doubled" }, requires: &[101], class_start: None },
    TreeNode { id: 103, x: 23, y: 6,  name: "Fractal IV",  short_desc: "Fractal Mastery cluster 4/5",                                               node_type: NodeType::Synergy { cluster: 1, bonus_desc: "Mandelbrot+Fibonacci outputs doubled" }, requires: &[102], class_start: None },
    TreeNode { id: 104, x: 25, y: 7,  name: "Fractal V",   short_desc: "Fractal Mastery cluster 5/5 — BONUS UNLOCKED",                              node_type: NodeType::Synergy { cluster: 1, bonus_desc: "Mandelbrot+Fibonacci outputs doubled" }, requires: &[103], class_start: None },

    // ── PRIME CLUSTER (5 nodes — prime density engine enhancements) ────────
    TreeNode { id: 110, x: 15, y: 13, name: "Prime I",     short_desc: "Prime cluster 1/5 — Prime Density outputs ×2 when all 5 allocated", node_type: NodeType::Synergy { cluster: 2, bonus_desc: "Prime Density Sieve outputs doubled" }, requires: &[3, 6, 90], class_start: None },
    TreeNode { id: 111, x: 17, y: 14, name: "Prime II",    short_desc: "Prime cluster 2/5", node_type: NodeType::Synergy { cluster: 2, bonus_desc: "Prime Density Sieve outputs doubled" }, requires: &[110], class_start: None },
    TreeNode { id: 112, x: 20, y: 14, name: "Prime III",   short_desc: "Prime cluster 3/5", node_type: NodeType::Synergy { cluster: 2, bonus_desc: "Prime Density Sieve outputs doubled" }, requires: &[111], class_start: None },
    TreeNode { id: 113, x: 23, y: 14, name: "Prime IV",    short_desc: "Prime cluster 4/5", node_type: NodeType::Synergy { cluster: 2, bonus_desc: "Prime Density Sieve outputs doubled" }, requires: &[112], class_start: None },
    TreeNode { id: 114, x: 25, y: 13, name: "Prime V",     short_desc: "Prime cluster 5/5 — BONUS UNLOCKED", node_type: NodeType::Synergy { cluster: 2, bonus_desc: "Prime Density Sieve outputs doubled" }, requires: &[113], class_start: None },
];

// ─── PLAYER PASSIVES ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerPassives {
    /// IDs of allocated nodes.
    pub allocated: HashSet<u16>,
    /// Cached: which stat bonuses have been locked in (node_id -> actual_value).
    pub stat_bonuses: std::collections::HashMap<u16, i64>,
    /// Skill points available to spend.
    pub points: u32,
    /// Which keystones are active.
    pub keystones: HashSet<String>,
    /// Which synergy clusters are fully completed.
    pub completed_synergies: HashSet<u8>,
    /// Cursor position for the tree navigator (node ID).
    pub cursor: u16,
}

impl PlayerPassives {
    /// Initialise for a given class — allocate the starting node for free.
    pub fn new_for_class(class: CharacterClass) -> Self {
        let mut p = PlayerPassives::default();
        if let Some(node) = NODES.iter().find(|n| n.class_start == Some(class)) {
            p.allocated.insert(node.id);
            p.cursor = node.id;
        }
        p
    }

    /// Is `node_id` available to allocate (adjacent to an allocated node and not yet allocated)?
    pub fn can_allocate(&self, node_id: u16) -> bool {
        if self.allocated.contains(&node_id) {
            return false;
        }
        let node = match NODES.iter().find(|n| n.id == node_id) {
            Some(n) => n,
            None => return false,
        };
        if node.requires.is_empty() {
            return true; // class start nodes
        }
        node.requires.iter().any(|req| self.allocated.contains(req))
    }

    /// Allocate a node. Rolls the stat value via chaos if it's a Stat node.
    /// Returns a description of what was gained.
    pub fn allocate(&mut self, node_id: u16, seed: u64) -> Option<String> {
        if !self.can_allocate(node_id) || self.points == 0 {
            return None;
        }
        let node = NODES.iter().find(|n| n.id == node_id)?;
        self.points -= 1;
        self.allocated.insert(node_id);

        let result = match &node.node_type {
            NodeType::Stat { stat, min, max } => {
                // Chaos-roll the bonus — you don't know until you commit.
                let roll = chaos_roll_verbose((*min + *max) as f64 * 0.005, seed);
                let range = max - min;
                let value =
                    min + ((roll.final_value * 0.5 + 0.5).clamp(0.0, 1.0) * range as f64) as i64;
                self.stat_bonuses.insert(node_id, value);
                format!(
                    "Allocated: {} → {} {}{}",
                    node.name,
                    stat.to_uppercase(),
                    if value >= 0 { "+" } else { "" },
                    value
                )
            }
            NodeType::Engine { engine, effect } => {
                format!("Allocated: {} — {} [{}]", node.name, engine, effect)
            }
            NodeType::Keystone { id } => {
                self.keystones.insert((*id).to_string());
                format!("KEYSTONE ACTIVATED: {} — {}", node.name, node.short_desc)
            }
            NodeType::Synergy {
                cluster,
                bonus_desc,
            } => {
                // Check if cluster is now complete.
                let cluster_size = NODES
                    .iter()
                    .filter(|n| {
                        matches!(&n.node_type, NodeType::Synergy { cluster: c, .. } if c == cluster)
                    })
                    .count();
                let allocated_in_cluster = NODES
                    .iter()
                    .filter(|n| {
                        matches!(&n.node_type, NodeType::Synergy { cluster: c, .. } if c == cluster)
                            && self.allocated.contains(&n.id)
                    })
                    .count();
                if allocated_in_cluster == cluster_size {
                    self.completed_synergies.insert(*cluster);
                    format!("SYNERGY COMPLETE: Cluster {} — {}", cluster, bonus_desc)
                } else {
                    format!(
                        "Synergy node {}/{}: {}",
                        allocated_in_cluster, cluster_size, node.name
                    )
                }
            }
        };

        // Update cursor to the newly allocated node.
        self.cursor = node_id;
        Some(result)
    }

    /// Sum of all stat bonuses from allocated Stat nodes.
    pub fn total_stat_bonus(&self, stat: &str) -> i64 {
        self.allocated
            .iter()
            .filter_map(|id| {
                let node = NODES.iter().find(|n| n.id == *id)?;
                if let NodeType::Stat { stat: s, .. } = &node.node_type {
                    if *s == stat {
                        return self.stat_bonuses.get(id).copied();
                    }
                }
                None
            })
            .sum()
    }

    /// Is a specific engine modification active?
    pub fn engine_mod(&self, engine: &str, effect: &str) -> bool {
        self.allocated.iter().any(|id| {
            NODES.iter().any(|n| {
                n.id == *id
                    && matches!(&n.node_type, NodeType::Engine { engine: e, effect: eff } if *e == engine && *eff == effect)
            })
        })
    }

    /// Is a keystone active?
    pub fn has_keystone(&self, id: &str) -> bool {
        self.keystones.contains(id)
    }

    /// Is a synergy cluster complete?
    pub fn synergy_active(&self, cluster: u8) -> bool {
        self.completed_synergies.contains(&cluster)
    }

    /// Move cursor to the nearest node in a cardinal direction.
    /// Returns the new cursor node ID.
    pub fn move_cursor(&mut self, dx: i16, dy: i16) -> u16 {
        let cur = match NODES.iter().find(|n| n.id == self.cursor) {
            Some(n) => n,
            None => {
                if let Some(n) = NODES.first() {
                    self.cursor = n.id;
                }
                return self.cursor;
            }
        };
        // Find the closest node strictly in the given direction.
        let best = NODES
            .iter()
            .filter(|n| n.id != self.cursor)
            .filter(|n| {
                let nx = n.x - cur.x;
                let ny = n.y - cur.y;
                // Must be in the dominant direction.
                if dx != 0 && dy == 0 {
                    nx * dx > 0 && nx.abs() >= ny.abs()
                } else if dy != 0 && dx == 0 {
                    ny * dy > 0 && ny.abs() >= nx.abs()
                } else {
                    false
                }
            })
            .min_by_key(|n| {
                let nx = n.x - cur.x;
                let ny = n.y - cur.y;
                nx * nx + ny * ny
            });

        if let Some(node) = best {
            self.cursor = node.id;
        }
        self.cursor
    }

    /// Render the passive tree as a compact ASCII grid.
    /// Returns a Vec of strings, one per row. Width ~82 chars.
    pub fn display_map(&self, class: CharacterClass) -> Vec<String> {
        const RESET: &str = "\x1b[0m";
        const DIM: &str = "\x1b[2m";
        const GREEN: &str = "\x1b[32m";
        const YELLOW: &str = "\x1b[33m";
        const CYAN: &str = "\x1b[36m";
        const MAGENTA: &str = "\x1b[35m";
        const BRIGHT_GREEN: &str = "\x1b[92m";
        const BRIGHT_CYAN: &str = "\x1b[96m";
        const WHITE: &str = "\x1b[97m";

        // Grid dimensions
        let cols = 41usize;
        let rows = 21usize;

        // Build a 2D grid: each cell is either empty or a node symbol.
        let mut grid: Vec<Vec<String>> = (0..rows)
            .map(|_| (0..cols).map(|_| format!("{}·{}", DIM, RESET)).collect())
            .collect();

        // Draw connection lines (simple horizontal/vertical segments).
        for node in NODES {
            for req_id in node.requires {
                let req = match NODES.iter().find(|n| n.id == *req_id) {
                    Some(r) => r,
                    None => continue,
                };
                // Draw a line from req to node using minimal segments.
                let (x0, y0) = (req.x as usize, req.y as usize);
                let (x1, y1) = (node.x as usize, node.y as usize);
                if x0 < cols && x1 < cols && y0 < rows && y1 < rows {
                    // Horizontal segment
                    let (lx, rx) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
                    for cell in &mut grid[y0][lx..=rx] {
                        if *cell == format!("{}·{}", DIM, RESET) {
                            *cell = format!("{}─{}", DIM, RESET);
                        }
                    }
                    // Vertical segment
                    let (ty, by) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
                    for row in &mut grid[ty..=by] {
                        if row[x1] == format!("{}·{}", DIM, RESET) {
                            row[x1] = format!("{}│{}", DIM, RESET);
                        }
                    }
                }
            }
        }

        // Place nodes on the grid.
        for node in NODES {
            let (x, y) = (node.x as usize, node.y as usize);
            if x >= cols || y >= rows {
                continue;
            }
            let is_cursor = node.id == self.cursor;
            let is_allocated = self.allocated.contains(&node.id);
            let can_alloc = self.can_allocate(node.id);
            let is_class_start = node.class_start == Some(class);

            let symbol = match &node.node_type {
                NodeType::Stat { .. } => "●",
                NodeType::Engine { .. } => "⚙",
                NodeType::Keystone { .. } => "★",
                NodeType::Synergy { .. } => "◆",
            };

            let cell = if is_cursor {
                format!("{}[{}]{}", WHITE, symbol, RESET)
            } else if is_allocated {
                format!("{}{}{}", BRIGHT_GREEN, symbol, RESET)
            } else if is_class_start {
                format!("{}{}{}", BRIGHT_CYAN, symbol, RESET)
            } else if can_alloc {
                format!("{}{}{}", YELLOW, symbol, RESET)
            } else {
                format!("{}{}{}", DIM, symbol, RESET)
            };

            grid[y][x] = cell;
        }

        // Flatten to strings.
        let mut lines = Vec::new();
        lines.push(format!(
            "  {}Passive Tree — {} pts available  [W/A/S/D]=move  [E]=allocate  [Q]=exit{}",
            CYAN, self.points, RESET
        ));
        lines.push(format!(
            "  {}{}●{}=stat  {}⚙{}=engine  {}★{}=keystone  {}◆{}=synergy  {}[●]{}=cursor  {}●{}=allocated",
            DIM, GREEN, DIM, DIM, MAGENTA, DIM, YELLOW, DIM, CYAN, DIM, WHITE, DIM, BRIGHT_GREEN
        ));
        lines.push(format!("  {}┌{}┐{}", CYAN, "─".repeat(cols + 2), RESET));
        for row in &grid {
            let row_str: String = row.iter().cloned().collect();
            lines.push(format!(
                "  {}│{} {} {}│{}",
                CYAN, RESET, row_str, CYAN, RESET
            ));
        }
        lines.push(format!("  {}└{}┘{}", CYAN, "─".repeat(cols + 2), RESET));

        // Show cursor node info.
        if let Some(cur) = NODES.iter().find(|n| n.id == self.cursor) {
            let status = if self.allocated.contains(&cur.id) {
                format!("{}ALLOCATED{}", BRIGHT_GREEN, RESET)
            } else if self.can_allocate(cur.id) {
                format!("{}AVAILABLE (costs 1 point){}", YELLOW, RESET)
            } else {
                format!("{}LOCKED{}", DIM, RESET)
            };
            lines.push(format!(
                "  {}▶ {} — {} | {}{}",
                CYAN, cur.name, cur.short_desc, status, RESET
            ));
        }

        lines
    }

    /// List view: show all nodes reachable from current position.
    pub fn list_available(&self) -> Vec<(u16, &'static str, &'static str)> {
        NODES
            .iter()
            .filter(|n| self.can_allocate(n.id))
            .map(|n| (n.id, n.name, n.short_desc))
            .collect()
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_starts_exist_for_all_classes() {
        for class in [
            CharacterClass::Mage,
            CharacterClass::Berserker,
            CharacterClass::Ranger,
            CharacterClass::Thief,
            CharacterClass::Necromancer,
            CharacterClass::Alchemist,
            CharacterClass::Paladin,
            CharacterClass::VoidWalker,
        ] {
            assert!(
                NODES.iter().any(|n| n.class_start == Some(class)),
                "No start node for {:?}",
                class
            );
        }
    }

    #[test]
    fn mage_can_allocate_starting_node() {
        let passives = PlayerPassives::new_for_class(CharacterClass::Mage);
        // Starting node (id=0) should be pre-allocated.
        assert!(passives.allocated.contains(&0));
    }

    #[test]
    fn allocate_stat_node_gives_bonus() {
        let mut p = PlayerPassives::new_for_class(CharacterClass::Mage);
        p.points = 5;
        let result = p.allocate(10, 42); // Mana Surge (requires node 0 which is allocated)
        assert!(result.is_some(), "should be able to allocate node 10");
        assert!(p.allocated.contains(&10));
        assert_eq!(p.points, 4);
    }

    #[test]
    fn cannot_allocate_locked_node() {
        let mut p = PlayerPassives::new_for_class(CharacterClass::Ranger);
        p.points = 5;
        // Node 52 (Entropy Inversion) requires necromancer path — should be locked.
        let result = p.allocate(52, 99);
        assert!(result.is_none());
        assert!(!p.allocated.contains(&52));
        assert_eq!(p.points, 5, "no points spent");
    }

    #[test]
    fn keystone_activates_on_allocation() {
        // Build up to the Glass Cannon keystone (0 → 10 → 12 → 14).
        let mut p = PlayerPassives::new_for_class(CharacterClass::Mage);
        p.points = 10;
        p.allocate(10, 1);
        p.allocate(12, 2);
        p.allocate(14, 3);
        assert!(p.has_keystone(KS_GLASS_CANNON));
    }
}
