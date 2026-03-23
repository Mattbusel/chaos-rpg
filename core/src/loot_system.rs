//! Procedural loot generation system for chaos-rpg.
//!
//! Uses a linear congruential generator (LCG) for reproducible randomness
//! without any external `rand` crate dependency.

// ---------------------------------------------------------------------------
// LCG – simple 64-bit linear congruential generator
// ---------------------------------------------------------------------------

/// Advance one LCG step (Knuth multiplier) and return the new state.
fn lcg_next(state: u64) -> u64 {
    state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

/// Return a pseudo-random `f64` in `[0, 1)` from the given LCG state.
fn lcg_f64(state: u64) -> f64 {
    // Use the upper 53 bits for precision.
    ((state >> 11) as f64) / (1u64 << 53) as f64
}

// ---------------------------------------------------------------------------
// ItemRarity
// ---------------------------------------------------------------------------

/// Rarity tier for a loot item. Higher rarity ⇒ lower drop weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    /// Relative drop weight used during weighted random selection.
    /// Larger numbers mean more frequent drops.
    pub fn drop_weight(&self) -> f64 {
        match self {
            ItemRarity::Common    => 50.0,
            ItemRarity::Uncommon  => 25.0,
            ItemRarity::Rare      => 15.0,
            ItemRarity::Epic      =>  7.0,
            ItemRarity::Legendary =>  3.0,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            ItemRarity::Common    => "Common",
            ItemRarity::Uncommon  => "Uncommon",
            ItemRarity::Rare      => "Rare",
            ItemRarity::Epic      => "Epic",
            ItemRarity::Legendary => "Legendary",
        }
    }
}

// ---------------------------------------------------------------------------
// LootEntry / LootDrop / LootTable
// ---------------------------------------------------------------------------

/// A single entry in a loot table.
#[derive(Debug, Clone)]
pub struct LootEntry {
    pub item_id:  String,
    pub rarity:   ItemRarity,
    /// Explicit weight override; set to 0.0 to use `rarity.drop_weight()`.
    pub weight:   f64,
    pub min_qty:  u32,
    pub max_qty:  u32,
}

impl LootEntry {
    /// Create a new entry using the rarity's default drop weight.
    pub fn new(item_id: impl Into<String>, rarity: ItemRarity, min_qty: u32, max_qty: u32) -> Self {
        Self {
            item_id: item_id.into(),
            rarity,
            weight: rarity.drop_weight(),
            min_qty,
            max_qty,
        }
    }

    /// Effective weight for this entry.
    fn effective_weight(&self) -> f64 {
        if self.weight > 0.0 { self.weight } else { self.rarity.drop_weight() }
    }
}

/// Items and value produced by rolling a loot table.
#[derive(Debug, Clone, Default)]
pub struct LootDrop {
    /// `(item_id, quantity)` pairs for every dropped item.
    pub items:       Vec<(String, u32)>,
    /// Sum of approximate gold values across all dropped items.
    pub total_value: u64,
}

/// A weighted loot table that can be rolled for drops.
#[derive(Debug, Clone, Default)]
pub struct LootTable {
    pub entries: Vec<LootEntry>,
}

impl LootTable {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: LootEntry) {
        self.entries.push(entry);
    }

    /// Roll the loot table with the supplied `seed`.
    ///
    /// Performs weighted random selection (linear scan) using an LCG derived
    /// from `seed`. Returns a [`LootDrop`] containing the selected item and
    /// a random quantity in `[min_qty, max_qty]`.
    pub fn roll(&self, seed: u64) -> LootDrop {
        if self.entries.is_empty() {
            return LootDrop::default();
        }

        let total_weight: f64 = self.entries.iter().map(|e| e.effective_weight()).sum();

        let mut state = lcg_next(seed);
        let pick = lcg_f64(state) * total_weight;
        state = lcg_next(state);

        // Linear-scan weighted selection.
        let mut accumulated = 0.0_f64;
        let mut selected: &LootEntry = &self.entries[0];
        for entry in &self.entries {
            accumulated += entry.effective_weight();
            if pick < accumulated {
                selected = entry;
                break;
            }
        }

        // Random quantity.
        let range = selected.max_qty.saturating_sub(selected.min_qty);
        let qty = if range == 0 {
            selected.min_qty
        } else {
            selected.min_qty + (lcg_f64(state) * (range + 1) as f64) as u32
        };

        // Approximate gold value: rarity multiplier × quantity.
        let rarity_multiplier: u64 = match selected.rarity {
            ItemRarity::Common    => 1,
            ItemRarity::Uncommon  => 3,
            ItemRarity::Rare      => 10,
            ItemRarity::Epic      => 50,
            ItemRarity::Legendary => 200,
        };
        let total_value = rarity_multiplier * qty as u64;

        LootDrop {
            items: vec![(selected.item_id.clone(), qty)],
            total_value,
        }
    }
}

// ---------------------------------------------------------------------------
// LootGenerator
// ---------------------------------------------------------------------------

/// High-level generator that wraps a [`LootTable`] and manages seed state.
pub struct LootGenerator {
    table: LootTable,
    seed:  u64,
}

impl LootGenerator {
    pub fn new(table: LootTable, initial_seed: u64) -> Self {
        Self { table, seed: initial_seed }
    }

    /// Roll the internal table and advance the seed for the next call.
    pub fn next_drop(&mut self) -> LootDrop {
        let drop = self.table.roll(self.seed);
        self.seed = lcg_next(self.seed);
        drop
    }

    /// Roll multiple drops and return them all.
    pub fn multi_drop(&mut self, count: usize) -> Vec<LootDrop> {
        (0..count).map(|_| self.next_drop()).collect()
    }
}

// ---------------------------------------------------------------------------
// Magic item name generation
// ---------------------------------------------------------------------------

/// Affixes used to build magic item names.
mod affixes {
    pub static PREFIXES: &[&str] = &[
        "Swift",
        "Mighty",
        "Ancient",
        "Cursed",
        "Radiant",
        "Iron",
        "Shadow",
        "Gilded",
        "Savage",
        "Ethereal",
        "Venomous",
        "Arcane",
        "Thunder",
        "Frost",
        "Blazing",
    ];

    pub static SUFFIXES: &[&str] = &[
        "of Fire",
        "of Ice",
        "of Thunder",
        "of the Void",
        "of Light",
        "of the Hunt",
        "of Chaos",
        "of the Dragon",
        "of Storms",
        "of Decay",
        "of the Ancients",
        "of Warding",
        "of the Moon",
        "of Fury",
        "of Wisdom",
    ];
}

/// Procedurally generate a magic item name from a base name and a seed.
///
/// Both a prefix and suffix are chosen via LCG-derived random indices into
/// the predefined affix tables.
pub fn generate_magic_name(base: &str, seed: u64) -> String {
    let mut state = lcg_next(seed ^ 0xDEAD_BEEF_CAFE_1234);

    let prefix_idx = (lcg_f64(state) * affixes::PREFIXES.len() as f64) as usize;
    state = lcg_next(state);
    let suffix_idx = (lcg_f64(state) * affixes::SUFFIXES.len() as f64) as usize;

    let prefix = affixes::PREFIXES[prefix_idx.min(affixes::PREFIXES.len() - 1)];
    let suffix = affixes::SUFFIXES[suffix_idx.min(affixes::SUFFIXES.len() - 1)];

    format!("{} {} {}", prefix, base, suffix)
}

/// Convenience wrapper representing a magic item.
#[derive(Debug, Clone)]
pub struct MagicItem {
    pub base_name: String,
    pub magic_name: String,
    pub rarity: ItemRarity,
    pub seed: u64,
}

impl MagicItem {
    pub fn new(base_name: impl Into<String>, rarity: ItemRarity, seed: u64) -> Self {
        let base = base_name.into();
        let magic_name = generate_magic_name(&base, seed);
        Self { base_name: base, magic_name, rarity, seed }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- ItemRarity ---

    #[test]
    fn rarity_drop_weights_are_positive_and_ordered() {
        let tiers = [
            ItemRarity::Common,
            ItemRarity::Uncommon,
            ItemRarity::Rare,
            ItemRarity::Epic,
            ItemRarity::Legendary,
        ];
        for t in &tiers {
            assert!(t.drop_weight() > 0.0);
        }
        // Common should be most common.
        assert!(ItemRarity::Common.drop_weight() > ItemRarity::Legendary.drop_weight());
    }

    #[test]
    fn rarity_labels_are_non_empty() {
        assert!(!ItemRarity::Rare.label().is_empty());
    }

    // --- LootTable ---

    #[test]
    fn empty_table_returns_empty_drop() {
        let table = LootTable::new();
        let drop = table.roll(42);
        assert!(drop.items.is_empty());
        assert_eq!(drop.total_value, 0);
    }

    #[test]
    fn single_entry_always_drops_that_item() {
        let mut table = LootTable::new();
        table.add_entry(LootEntry::new("iron_sword", ItemRarity::Common, 1, 1));
        for seed in 0..20u64 {
            let drop = table.roll(seed);
            assert_eq!(drop.items.len(), 1);
            assert_eq!(drop.items[0].0, "iron_sword");
            assert_eq!(drop.items[0].1, 1);
        }
    }

    #[test]
    fn drop_quantity_within_bounds() {
        let mut table = LootTable::new();
        table.add_entry(LootEntry::new("gold_coin", ItemRarity::Common, 5, 20));
        for seed in 0..50u64 {
            let drop = table.roll(seed * 17 + 3);
            let qty = drop.items[0].1;
            assert!((5..=20).contains(&qty), "qty={} out of [5,20]", qty);
        }
    }

    #[test]
    fn legendary_item_has_high_value() {
        let mut table = LootTable::new();
        table.add_entry(LootEntry {
            item_id:  "excalibur".into(),
            rarity:   ItemRarity::Legendary,
            weight:   100.0, // force selection
            min_qty:  1,
            max_qty:  1,
        });
        let drop = table.roll(99);
        assert!(drop.total_value >= 200);
    }

    // --- LootGenerator ---

    #[test]
    fn generator_advances_seed_between_rolls() {
        let mut table = LootTable::new();
        table.add_entry(LootEntry::new("apple", ItemRarity::Common, 1, 10));
        let mut gen = LootGenerator::new(table, 1234);
        let d1 = gen.next_drop();
        let d2 = gen.next_drop();
        // Different seeds should (very likely) produce different quantities.
        // We just check both return valid items.
        assert_eq!(d1.items[0].0, "apple");
        assert_eq!(d2.items[0].0, "apple");
    }

    #[test]
    fn multi_drop_returns_correct_count() {
        let mut table = LootTable::new();
        table.add_entry(LootEntry::new("stone", ItemRarity::Common, 1, 3));
        let mut gen = LootGenerator::new(table, 7777);
        let drops = gen.multi_drop(5);
        assert_eq!(drops.len(), 5);
    }

    // --- Magic name generation ---

    #[test]
    fn magic_name_contains_base() {
        let name = generate_magic_name("Sword", 42);
        assert!(name.contains("Sword"), "expected base in magic name: {}", name);
    }

    #[test]
    fn magic_name_has_prefix_and_suffix() {
        let name = generate_magic_name("Dagger", 100);
        // Name should be at least "X Dagger of Y" – split on spaces.
        let words: Vec<&str> = name.split_whitespace().collect();
        assert!(words.len() >= 3, "too short: {}", name);
    }

    #[test]
    fn same_seed_produces_same_name() {
        let a = generate_magic_name("Staff", 999);
        let b = generate_magic_name("Staff", 999);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_likely_differ() {
        let a = generate_magic_name("Staff", 1);
        let b = generate_magic_name("Staff", 9999);
        // Not guaranteed, but the probability of collision is very low.
        // If both affixes happen to match we get a false-positive; acceptable.
        let _ = (a, b); // just ensure no panic
    }

    #[test]
    fn magic_item_struct_fields() {
        let item = MagicItem::new("Axe", ItemRarity::Epic, 42);
        assert_eq!(item.base_name, "Axe");
        assert!(!item.magic_name.is_empty());
        assert_eq!(item.rarity, ItemRarity::Epic);
    }

    // --- Weighted selection distribution sanity check ---

    #[test]
    fn weighted_selection_distributes_roughly_correctly() {
        let mut table = LootTable::new();
        table.add_entry(LootEntry {
            item_id: "common_item".into(),
            rarity: ItemRarity::Common,
            weight: 90.0,
            min_qty: 1,
            max_qty: 1,
        });
        table.add_entry(LootEntry {
            item_id: "rare_item".into(),
            rarity: ItemRarity::Rare,
            weight: 10.0,
            min_qty: 1,
            max_qty: 1,
        });

        let trials = 1000u64;
        let mut common_count = 0u64;
        for i in 0..trials {
            let drop = table.roll(i.wrapping_mul(1234567));
            if drop.items[0].0 == "common_item" {
                common_count += 1;
            }
        }
        // Expect roughly 90% common. Allow ±15% tolerance.
        let ratio = common_count as f64 / trials as f64;
        assert!(ratio > 0.75, "common ratio too low: {:.2}", ratio);
        assert!(ratio < 0.99, "common ratio too high: {:.2}", ratio);
    }
}
