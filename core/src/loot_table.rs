//! Loot table: weighted random drops with rarity tiers and set bonuses

#[derive(Clone, Debug, PartialEq)]
/// How rare a drop is; rarer items have higher stats.
pub enum Rarity {
    /// Drop weight 100, stats x1.0.
    Common,
    /// Drop weight 40, stats x1.3.
    Uncommon,
    /// Drop weight 15, stats x1.7.
    Rare,
    /// Drop weight 5, stats x2.5.
    Epic,
    /// Drop weight 1, stats x4.0.
    Legendary,
}

impl Rarity {
    /// Relative drop weight of this rarity (Common 100 down to Legendary 1).
    pub fn weight(&self) -> u32 {
        match self {
            Rarity::Common => 100, Rarity::Uncommon => 40,
            Rarity::Rare => 15, Rarity::Epic => 5, Rarity::Legendary => 1,
        }
    }
    /// Multiplier applied to an item's base stats (1.0 to 4.0).
    pub fn stat_multiplier(&self) -> f32 {
        match self {
            Rarity::Common => 1.0, Rarity::Uncommon => 1.3,
            Rarity::Rare => 1.7, Rarity::Epic => 2.5, Rarity::Legendary => 4.0,
        }
    }
    /// Name shown to the player.
    pub fn name(&self) -> &'static str {
        match self { Rarity::Common => "Common", Rarity::Uncommon => "Uncommon",
                     Rarity::Rare => "Rare", Rarity::Epic => "Epic", Rarity::Legendary => "Legendary" }
    }
}

/// An item that can drop.
#[derive(Clone, Debug)]
pub struct LootItem {
    /// Unique item identifier.
    pub id: String,
    /// Name shown to the player.
    pub name: String,
    /// Rarity, which scales its stats.
    pub rarity: Rarity,
    /// Equipment set it belongs to, if any.
    pub set_id: Option<String>,
    /// Stats before the rarity multiplier, by stat name.
    pub base_stats: std::collections::HashMap<String, f32>,
}

impl LootItem {
    /// Base stats multiplied by the rarity multiplier.
    pub fn effective_stats(&self) -> std::collections::HashMap<String, f32> {
        let mult = self.rarity.stat_multiplier();
        self.base_stats.iter().map(|(k, &v)| (k.clone(), v * mult)).collect()
    }
}

/// An item in a loot table with its drop weight and level range.
#[derive(Clone)]
pub struct LootEntry {
    /// The item.
    pub item: LootItem,
    /// Relative chance of being picked among eligible entries.
    pub weight: u32,
    /// Lowest player level it can drop at.
    pub min_level: u32,
    /// Highest player level it can drop at.
    pub max_level: u32,
}

/// Extra stats for wearing enough pieces of one set.
pub struct SetBonus {
    /// Set identifier, matched against `LootItem::set_id`.
    pub set_id: String,
    /// Pieces needed for the bonus.
    pub pieces_required: usize,
    /// Stats added, by stat name.
    pub bonus_stats: std::collections::HashMap<String, f32>,
    /// Text shown to the player.
    pub description: String,
}

/// Weighted item drops, set bonuses and a gold range.
pub struct LootTable {
    /// Items that can drop.
    pub entries: Vec<LootEntry>,
    /// Set bonuses that can be earned.
    pub set_bonuses: Vec<SetBonus>,
    /// Gold drop range as (min, max); max itself is never rolled.
    pub gold_range: (u32, u32),
}

impl LootTable {
    /// An empty table with a gold range of 1 to 10.
    pub fn new() -> Self {
        LootTable { entries: Vec::new(), set_bonuses: Vec::new(), gold_range: (1, 10) }
    }

    /// Add an item with a drop weight and level range.
    pub fn add_item(mut self, item: LootItem, weight: u32, min_level: u32, max_level: u32) -> Self {
        self.entries.push(LootEntry { item, weight, min_level, max_level });
        self
    }

    /// Add a set bonus.
    pub fn add_set_bonus(mut self, bonus: SetBonus) -> Self {
        self.set_bonuses.push(bonus);
        self
    }

    /// Roll for items at given level, returning a list of drops
    pub fn roll(&self, level: u32, n_rolls: u32, seed: u64) -> Vec<LootItem> {
        let mut state = seed;
        let mut drops = Vec::new();

        let eligible: Vec<&LootEntry> = self.entries.iter()
            .filter(|e| e.min_level <= level && e.max_level >= level)
            .collect();
        if eligible.is_empty() { return drops; }

        let total_weight: u32 = eligible.iter().map(|e| e.weight).sum();

        for _ in 0..n_rolls {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let roll = (state >> 11) as u32 % total_weight;
            let mut cumulative = 0u32;
            for entry in &eligible {
                cumulative += entry.weight;
                if roll < cumulative {
                    drops.push(entry.item.clone());
                    break;
                }
            }
        }
        drops
    }

    /// Compute set bonuses for a collection of equipped items
    pub fn active_set_bonuses(&self, equipped: &[LootItem]) -> Vec<&SetBonus> {
        self.set_bonuses.iter().filter(|bonus| {
            let count = equipped.iter()
                .filter(|item| item.set_id.as_deref() == Some(&bonus.set_id))
                .count();
            count >= bonus.pieces_required
        }).collect()
    }

    /// Gold dropped for a seed, from `gold_range.0` up to but not including `gold_range.1`.
    pub fn gold_drop(&self, seed: u64) -> u32 {
        let range = self.gold_range.1 - self.gold_range.0;
        if range == 0 { return self.gold_range.0; }
        self.gold_range.0 + (seed >> 11) as u32 % range
    }
}

impl Default for LootTable {
    fn default() -> Self {
        Self::new()
    }
}

/// A sample loot table: three swords, a shield and a ring, with a two-piece Void set bonus (+25 void damage).
pub fn default_loot_table() -> LootTable {
    use std::collections::HashMap;
    let mut sword_stats = HashMap::new(); sword_stats.insert("attack".to_string(), 10.0f32);
    let mut shield_stats = HashMap::new(); shield_stats.insert("defense".to_string(), 8.0f32);
    let mut ring_stats = HashMap::new(); ring_stats.insert("magic".to_string(), 5.0f32); ring_stats.insert("defense".to_string(), 2.0f32);

    LootTable::new()
        .add_item(LootItem { id: "iron_sword".to_string(), name: "Iron Sword".to_string(), rarity: Rarity::Common, set_id: None, base_stats: sword_stats.clone() }, 80, 1, 99)
        .add_item(LootItem { id: "steel_sword".to_string(), name: "Steel Sword".to_string(), rarity: Rarity::Uncommon, set_id: Some("warrior".to_string()), base_stats: sword_stats.clone() }, 30, 5, 99)
        .add_item(LootItem { id: "void_sword".to_string(), name: "Void Blade".to_string(), rarity: Rarity::Legendary, set_id: Some("void_set".to_string()), base_stats: { let mut s = sword_stats.clone(); s.insert("lifesteal".to_string(), 5.0); s } }, 1, 15, 99)
        .add_item(LootItem { id: "iron_shield".to_string(), name: "Iron Shield".to_string(), rarity: Rarity::Common, set_id: None, base_stats: shield_stats.clone() }, 60, 1, 99)
        .add_item(LootItem { id: "void_ring".to_string(), name: "Void Ring".to_string(), rarity: Rarity::Epic, set_id: Some("void_set".to_string()), base_stats: ring_stats }, 5, 10, 99)
        .add_set_bonus(SetBonus { set_id: "void_set".to_string(), pieces_required: 2, bonus_stats: { let mut s = HashMap::new(); s.insert("void_damage".to_string(), 25.0f32); s }, description: "+25 Void Damage".to_string() })
}
