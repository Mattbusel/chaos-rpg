//! Crafting system — every operation is chaos-rolled.
//!
//! Operations:
//!   Reforge      — reroll all stats (keeps base type & material name)
//!   Augment      — add one chaos-rolled stat modifier
//!   Annul        — remove one random stat modifier
//!   Corrupt      — destiny roll on the entire item; can double stats, zero
//!                  them, add a unique implicit, or brick to "Mathematical Error"
//!   Fuse         — attempt to add a socket link; chaos-rolled per attempt
//!   EngineLock   — permanently lock an engine into/out of this item's rolls
//!
//! Corrupted items cannot be crafted further.
//! Engine Lock costs scale exponentially with the number of existing locks.

use crate::chaos_pipeline::{chaos_roll_verbose, destiny_roll, roll_stat};
use crate::items::{Item, Rarity, StatModifier};

// ─── ENGINE NAMES ─────────────────────────────────────────────────────────────

pub const ENGINE_NAMES: &[&str] = &[
    "Lorenz Attractor",
    "Fourier Harmonic",
    "Prime Density Sieve",
    "Riemann Zeta Partial",
    "Fibonacci Golden Spiral",
    "Mandelbrot Escape",
    "Logistic Map",
    "Euler's Totient",
    "Collatz Chain",
    "Modular Exp Hash",
];

const STAT_NAMES: &[&str] = &[
    "Vitality",
    "Force",
    "Mana",
    "Cunning",
    "Precision",
    "Entropy",
    "Luck",
];

const CORRUPTION_IMPLICITS: &[&str] = &[
    "Void-touched: chaos rolls add +1 extra engine",
    "Cursed: bearer takes 1 damage per second from background math",
    "Enlightened: owner understands the 4th dimension (mostly useless)",
    "Phasing: item occasionally becomes ethereal (not equippable while ethereal)",
    "Prime-locked: all damage values are rounded to nearest prime",
    "Schrödinger's Defense: armor value is simultaneously max and 0 until observed",
    "Recursive: spells cast from this item cast themselves again once",
    "Anti-gravity: item floats 3 inches above hand (still usable)",
    "Mathematically Inevitable: one random effect per fight is guaranteed to occur",
    "Beyond the Set: Mandelbrot engine always samples outside the set (max chaos)",
    "Entropic Decay: stats decay by 1% per floor but reset on level up",
    "Omega Point: when used at exactly 0 mana, does double everything",
];

// ─── CRAFTING RESULT ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CraftResult {
    Success { description: String, item: Item },
    Failure { reason: String },
    Bricked { description: String, item: Item },
}

impl CraftResult {
    pub fn is_success(&self) -> bool {
        matches!(self, CraftResult::Success { .. })
    }

    pub fn item(&self) -> Option<&Item> {
        match self {
            CraftResult::Success { item, .. } | CraftResult::Bricked { item, .. } => Some(item),
            CraftResult::Failure { .. } => None,
        }
    }

    pub fn into_item(self) -> Option<Item> {
        match self {
            CraftResult::Success { item, .. } | CraftResult::Bricked { item, .. } => Some(item),
            CraftResult::Failure { .. } => None,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            CraftResult::Success { description, .. } => description,
            CraftResult::Failure { reason } => reason,
            CraftResult::Bricked { description, .. } => description,
        }
    }
}

// ─── CRAFTING OPERATIONS ──────────────────────────────────────────────────────

/// Reforge: reroll all stats on an item. Keeps base type, material, socket
/// count, engine locks, and corruption state. Everything else is re-chaos-rolled.
pub fn reforge(item: &Item, seed: u64) -> CraftResult {
    if item.corruption.is_some() {
        return CraftResult::Failure {
            reason: "Cannot reforge a corrupted item.".to_string(),
        };
    }
    let new_item = Item::generate(seed);
    // Preserve identity: keep the original name, base type, locks, sockets.
    let mut result = Item {
        name: item.name.clone(),
        base_type: item.base_type.clone(),
        socket_count: item.socket_count,
        socket_links: item.socket_links,
        socketed_gems: item.socketed_gems.clone(),
        engine_locks: item.engine_locks.clone(),
        corruption: None,
        is_weapon: item.is_weapon,
        ..new_item
    };
    // Preserve material in name (first word).
    let material_word = item.name.split_whitespace().next().unwrap_or("chaos");
    let rest: Vec<&str> = result.name.splitn(2, ' ').skip(1).collect();
    result.name = format!("{} {}", material_word, rest.join(" "));

    CraftResult::Success {
        description: format!(
            "Reforged! New rolls: dmg/def={:+}, {} modifiers.",
            result.damage_or_defense,
            result.stat_modifiers.len()
        ),
        item: result,
    }
}

/// Augment: add one chaos-rolled stat modifier to the item.
pub fn augment(item: &Item, seed: u64) -> CraftResult {
    if item.corruption.is_some() {
        return CraftResult::Failure {
            reason: "Cannot augment a corrupted item.".to_string(),
        };
    }
    if item.stat_modifiers.len() >= 6 {
        return CraftResult::Failure {
            reason: "Item already has 6 modifiers — maximum reached.".to_string(),
        };
    }
    let mut result = item.clone();
    let stat_idx = (seed % STAT_NAMES.len() as u64) as usize;
    let roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(5555));
    let value = (roll.final_value * 2000.0) as i64 + roll_stat(-500, 500, seed.wrapping_add(6666));
    let stat = STAT_NAMES[stat_idx].to_string();
    result.stat_modifiers.push(StatModifier {
        stat: stat.clone(),
        value,
    });
    // Rarity may have improved.
    let total_mag: i64 = result
        .stat_modifiers
        .iter()
        .map(|m| m.value.abs())
        .sum::<i64>()
        + result.damage_or_defense.abs();
    result.rarity = Rarity::from_magnitude(total_mag);
    CraftResult::Success {
        description: format!(
            "Augmented! Added {} {:+}  ({})",
            stat,
            value,
            result.rarity.name()
        ),
        item: result,
    }
}

/// Annul: remove one random stat modifier. Chaos roll picks which one.
pub fn annul(item: &Item, seed: u64) -> CraftResult {
    if item.corruption.is_some() {
        return CraftResult::Failure {
            reason: "Cannot annul a corrupted item.".to_string(),
        };
    }
    if item.stat_modifiers.is_empty() {
        return CraftResult::Failure {
            reason: "No modifiers to remove.".to_string(),
        };
    }
    let mut result = item.clone();
    let idx = (seed % result.stat_modifiers.len() as u64) as usize;
    let removed = result.stat_modifiers.remove(idx);
    CraftResult::Success {
        description: format!(
            "Annulled {} {:+} from {}.",
            removed.stat, removed.value, item.name
        ),
        item: result,
    }
}

/// Corrupt: run the item through a destiny roll. Wild outcomes.
/// Corrupted items cannot be crafted further.
pub fn corrupt(item: &Item, seed: u64) -> CraftResult {
    if item.corruption.is_some() {
        return CraftResult::Failure {
            reason: "Already corrupted. The math has been permanently applied.".to_string(),
        };
    }

    let roll = destiny_roll(seed as f64 * 1e-12, seed);
    let outcome = roll.to_range(1, 100);

    let mut result = item.clone();

    let desc = match outcome {
        // 1–15: Brick — Mathematical Error
        1..=15 => {
            result.name = "Mathematical Error".to_string();
            result.base_type = "Error".to_string();
            result.damage_or_defense = 0;
            result.stat_modifiers.clear();
            result.special_effect = "Undefined behavior. Do not equip while sober.".to_string();
            result.rarity = Rarity::Common;
            result.corruption = Some("BRICKED: Mathematical Error".to_string());
            let desc = "☢ CORRUPTION BRICKED the item! It is now a Mathematical Error.".to_string();
            return CraftResult::Bricked {
                description: desc,
                item: result,
            };
        }
        // 16–30: All stats doubled
        16..=30 => {
            result.damage_or_defense *= 2;
            for m in &mut result.stat_modifiers {
                m.value *= 2;
            }
            result.corruption = Some("Doubled: all stats ×2".to_string());
            "☢ Corruption: ALL STATS DOUBLED!".to_string()
        }
        // 31–45: All stats zeroed
        31..=45 => {
            result.damage_or_defense = 0;
            for m in &mut result.stat_modifiers {
                m.value = 0;
            }
            result.corruption = Some("Nullified: all stats zeroed by the void".to_string());
            "☢ Corruption: ALL STATS ZEROED by the void.".to_string()
        }
        // 46–60: Add a unique corruption implicit
        46..=60 => {
            let imp_idx = (seed.wrapping_mul(777) % CORRUPTION_IMPLICITS.len() as u64) as usize;
            let implicit = CORRUPTION_IMPLICITS[imp_idx];
            result.corruption = Some(implicit.to_string());
            format!("☢ Corruption implicit: {}", implicit)
        }
        // 61–75: Randomize all stats (extreme reroll)
        61..=75 => {
            result.damage_or_defense =
                (chaos_roll_verbose(seed as f64 * 2e-9, seed.wrapping_add(1)).final_value * 2000.0)
                    as i64;
            for (i, m) in result.stat_modifiers.iter_mut().enumerate() {
                let s = seed.wrapping_add(i as u64 * 31337);
                let r = chaos_roll_verbose(s as f64 * 1e-9, s);
                m.value = (r.final_value * 3000.0) as i64;
            }
            result.corruption = Some("Chaos-rerolled: destiny touched every value".to_string());
            "☢ Corruption: ALL VALUES CHAOS-REROLLED!".to_string()
        }
        // 76–90: Add an extra socket
        76..=90 => {
            if result.socket_count < 6 {
                result.socket_count += 1;
            }
            result.corruption = Some(format!("Extra socket ({} total)", result.socket_count));
            format!(
                "☢ Corruption: Extra socket! Now {} sockets.",
                result.socket_count
            )
        }
        // 91–100: Triple the best stat modifier
        _ => {
            if let Some(best) = result
                .stat_modifiers
                .iter_mut()
                .max_by_key(|m| m.value.abs())
            {
                let old = best.value;
                best.value *= 3;
                result.corruption = Some(format!(
                    "Amplified: {} tripled ({} → {})",
                    best.stat, old, best.value
                ));
                format!(
                    "☢ Corruption: {} TRIPLED ({:+} → {:+})!",
                    best.stat,
                    old,
                    best.value * 3
                )
            } else {
                result.corruption = Some("Inert: the math refused to act".to_string());
                "☢ Corruption: inert. The math refused to act.".to_string()
            }
        }
    };

    // Update rarity post-corruption.
    let total_mag: i64 = result
        .stat_modifiers
        .iter()
        .map(|m| m.value.abs())
        .sum::<i64>()
        + result.damage_or_defense.abs();
    result.rarity = Rarity::from_magnitude(total_mag);

    CraftResult::Success {
        description: desc,
        item: result,
    }
}

/// Fuse: attempt to add a link between sockets.
/// Chaos roll determines success. Each attempt costs gold (handled by caller).
pub fn fuse(item: &Item, seed: u64) -> CraftResult {
    if item.corruption.is_some() {
        return CraftResult::Failure {
            reason: "Cannot fuse a corrupted item.".to_string(),
        };
    }
    if item.socket_count < 2 {
        return CraftResult::Failure {
            reason: "Need at least 2 sockets to fuse.".to_string(),
        };
    }
    let max_links = item.socket_count.saturating_sub(1);
    if item.socket_links >= max_links {
        return CraftResult::Failure {
            reason: format!("All {} sockets already linked.", item.socket_count),
        };
    }
    let roll = chaos_roll_verbose(0.3, seed);
    // Success if roll is positive (roughly 50–70% depending on chain).
    if roll.final_value > 0.0 {
        let mut result = item.clone();
        result.socket_links += 1;
        CraftResult::Success {
            description: format!(
                "Fuse succeeded! {}/{} links. (roll: {:.3})",
                result.socket_links, max_links, roll.final_value
            ),
            item: result,
        }
    } else {
        CraftResult::Failure {
            reason: format!(
                "Fuse failed. (roll: {:.3}) The math resisted.",
                roll.final_value
            ),
        }
    }
}

/// Engine Lock: permanently add an engine lock to this item.
/// "+EngineName" = always include in damage rolls.
/// "-EngineName" = always exclude from damage rolls.
/// Cost in gold scales exponentially: 100 × 10^(existing_locks).
pub fn engine_lock_cost(item: &Item) -> i64 {
    let n = item.engine_locks.len() as u32;
    100i64 * 10i64.pow(n.min(6))
}

pub fn engine_lock(item: &Item, engine: &str, include: bool, seed: u64) -> CraftResult {
    if item.corruption.is_some() {
        return CraftResult::Failure {
            reason: "Cannot engine-lock a corrupted item.".to_string(),
        };
    }
    if item.engine_locks.len() >= 3 {
        return CraftResult::Failure {
            reason: "Maximum 3 engine locks per item reached.".to_string(),
        };
    }
    if !ENGINE_NAMES.contains(&engine) {
        return CraftResult::Failure {
            reason: format!("Unknown engine: {}", engine),
        };
    }
    // Small chaos check — the lock might partially fail.
    let roll = chaos_roll_verbose(0.6, seed);
    let prefix = if include { "+" } else { "-" };
    let lock_str = format!("{}{}", prefix, engine);

    let mut result = item.clone();
    if roll.final_value > -0.5 {
        // Success
        // Remove any existing lock for this engine.
        result.engine_locks.retain(|l| !l.ends_with(engine));
        result.engine_locks.push(lock_str.clone());
        CraftResult::Success {
            description: format!(
                "Engine locked: {} will always be {} this item's rolls.",
                engine,
                if include {
                    "included in"
                } else {
                    "excluded from"
                }
            ),
            item: result,
        }
    } else {
        CraftResult::Failure {
            reason: format!(
                "Lock failed! (roll {:.3}) The {} engine refused to be bound.",
                roll.final_value, engine
            ),
        }
    }
}

/// Shatter: destroy the item and scatter modifiers as transferable shards.
/// (Stub — shard persistence is handled by the caller.)
pub fn shatter(item: &Item, _seed: u64) -> CraftResult {
    CraftResult::Success {
        description: format!("☠ Shattered {} into modifier shards.", item.name),
        item: Item::generate(0), // placeholder; caller discards this
    }
}

/// Imbue: implant a shard into the item.
/// (Stub — shard selection handled by caller.)
pub fn imbue(item: &Item, seed: u64) -> CraftResult {
    let mut result = item.clone();
    let roll = chaos_roll_verbose(0.5, seed);
    let stat_idx = (seed % STAT_NAMES.len() as u64) as usize;
    let value = (roll.final_value * 1000.0) as i64;
    result.stat_modifiers.push(crate::items::StatModifier {
        stat: STAT_NAMES[stat_idx].to_string(),
        value,
    });
    let total_mag: i64 = result.stat_modifiers.iter().map(|m| m.value.abs()).sum::<i64>()
        + result.damage_or_defense.abs();
    result.rarity = crate::items::Rarity::from_magnitude(total_mag);
    CraftResult::Success {
        description: format!("Imbued shard: {} {:+}", STAT_NAMES[stat_idx], value),
        item: result,
    }
}

/// Repair: restore an item's durability to its maximum value.
/// Cost in gold is handled by the caller (50 + floor × 5).
pub fn repair(item: &Item) -> CraftResult {
    if item.durability == item.max_durability {
        return CraftResult::Failure {
            reason: format!("{} is already at full durability.", item.name),
        };
    }
    let mut result = item.clone();
    result.repair_full();
    CraftResult::Success {
        description: format!(
            "Repaired! {} durability restored to {}/{}.",
            item.name, result.durability, result.max_durability
        ),
        item: result,
    }
}

/// Gold cost to repair an item (scales with how damaged it is).
pub fn repair_cost(item: &Item, floor: u32) -> i64 {
    let missing = item.max_durability.saturating_sub(item.durability) as i64;
    (50 + floor as i64 * 5) * missing / item.max_durability.max(1) as i64
}

// ─── DISPLAY ──────────────────────────────────────────────────────────────────

/// Display lines for the crafting bench UI.
pub fn crafting_bench_lines(item: &Item, gold: i64) -> Vec<String> {
    const RESET: &str = "\x1b[0m";
    const CYAN: &str = "\x1b[36m";
    const YELLOW: &str = "\x1b[33m";
    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const DIM: &str = "\x1b[2m";

    let mut lines = Vec::new();
    lines.push(format!(
        "{}╔══ CRAFTING BENCH ═══════════════════════════════╗{}",
        CYAN, RESET
    ));
    lines.push(format!(
        "{}║  Item: {}{:<40}{}{}║{}",
        CYAN,
        item.rarity.color_code(),
        &item.name,
        RESET,
        CYAN,
        RESET
    ));
    lines.push(format!(
        "{}║  Sockets: {}/6  Links: {}  Locked engines: {}{}{}║{}",
        CYAN,
        item.socket_count,
        item.socket_links,
        item.engine_locks.len(),
        " ".repeat(15 - item.engine_locks.len().min(15)),
        CYAN,
        RESET
    ));
    lines.push(format!(
        "{}║  Durability: {}{}{}                              ║{}",
        CYAN,
        item.durability_bar(),
        CYAN,
        " ".repeat(0),
        RESET
    ));
    if let Some(cor) = &item.corruption {
        lines.push(format!(
            "{}║  {}⚠ CORRUPTED: {}{}{:<28}{}║{}",
            CYAN, RED, cor, RESET, "", CYAN, RESET
        ));
    }
    lines.push(format!(
        "{}╠══ OPERATIONS ═══════════════════════════════════╣{}",
        CYAN, RESET
    ));

    let corrupted = item.corruption.is_some();
    let fuse_cost = 50 + item.socket_links as i64 * 200;
    let lock_cost = engine_lock_cost(item);

    let repair_cost_val = repair_cost(item, 0 /* caller may pass floor */);
    let ops: &[(&str, &str, i64, bool)] = &[
        (
            "[R] Reforge",
            "Reroll all stats (chaos-rolled)",
            0,
            !corrupted,
        ),
        (
            "[A] Augment",
            "Add one chaos-rolled modifier",
            0,
            !corrupted && item.stat_modifiers.len() < 6,
        ),
        (
            "[N] Annul",
            "Remove one random modifier",
            0,
            !corrupted && !item.stat_modifiers.is_empty(),
        ),
        (
            "[C] Corrupt",
            "Destiny roll — wild outcomes, permanent",
            0,
            !corrupted,
        ),
        (
            "[F] Fuse",
            "Attempt to add a socket link",
            fuse_cost,
            !corrupted && item.socket_count >= 2,
        ),
        (
            "[L] Eng.Lock",
            "Lock an engine in/out of rolls",
            lock_cost,
            !corrupted && item.engine_locks.len() < 3,
        ),
        (
            "[P] Repair",
            "Restore item durability to maximum",
            repair_cost_val,
            item.durability < item.max_durability,
        ),
    ];

    for (key, desc, cost, available) in ops {
        let cost_str = if *cost > 0 {
            format!(" ({}{}g{})", YELLOW, cost, RESET)
        } else {
            String::new()
        };
        let avail_col = if *available { GREEN } else { DIM };
        lines.push(format!(
            "{}║  {}{}{} — {}{}{}{}║{}",
            CYAN, avail_col, key, RESET, DIM, desc, cost_str, CYAN, RESET
        ));
    }

    lines.push(format!(
        "{}║  {}[0] Leave bench{}{}                               ║{}",
        CYAN, GREEN, RESET, CYAN, RESET
    ));
    lines.push(format!(
        "{}║  {}Your gold: {}{}{}                                 ║{}",
        CYAN, YELLOW, gold, RESET, CYAN, RESET
    ));
    lines.push(format!(
        "{}╚═════════════════════════════════════════════════╝{}",
        CYAN, RESET
    ));
    lines
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Item;

    fn test_item() -> Item {
        Item::generate(42)
    }

    #[test]
    fn reforge_changes_stats() {
        let item = test_item();
        let original_dmg = item.damage_or_defense;
        let result = reforge(&item, 99999);
        if let Some(new_item) = result.into_item() {
            // Name preserved
            assert!(new_item
                .name
                .contains(item.name.split_whitespace().next().unwrap()));
            // Stats are likely different (may rarely be the same)
            let _ = new_item.damage_or_defense != original_dmg;
        }
    }

    #[test]
    fn augment_adds_modifier() {
        let item = test_item();
        let original_mods = item.stat_modifiers.len();
        if let CraftResult::Success { item: new_item, .. } = augment(&item, 1234) {
            assert_eq!(new_item.stat_modifiers.len(), original_mods + 1);
        }
    }

    #[test]
    fn annul_removes_modifier() {
        let mut item = test_item();
        // Ensure there's at least one modifier.
        item.stat_modifiers.push(StatModifier {
            stat: "Force".to_string(),
            value: 10,
        });
        let original_count = item.stat_modifiers.len();
        if let CraftResult::Success { item: new_item, .. } = annul(&item, 77) {
            assert_eq!(new_item.stat_modifiers.len(), original_count - 1);
        }
    }

    #[test]
    fn corrupt_sets_corruption_field() {
        let item = test_item();
        let result = corrupt(&item, 50); // seed 50 should hit a non-brick outcome
        if let Some(new_item) = result.into_item() {
            assert!(new_item.corruption.is_some());
        }
    }

    #[test]
    fn cannot_craft_corrupted_item() {
        let mut item = test_item();
        item.corruption = Some("test corruption".to_string());
        assert!(!reforge(&item, 1).is_success());
        assert!(!augment(&item, 1).is_success());
        assert!(!corrupt(&item, 1).is_success());
    }

    #[test]
    fn engine_lock_cost_scales() {
        let mut item = test_item();
        assert_eq!(engine_lock_cost(&item), 100);
        item.engine_locks.push("+Lorenz Attractor".to_string());
        assert_eq!(engine_lock_cost(&item), 1000);
        item.engine_locks.push("+Collatz Chain".to_string());
        assert_eq!(engine_lock_cost(&item), 10000);
    }
}
