//! Dynamic economy system — supply/demand pricing, faction economics.
//!
//! Bridges proof-engine's economy module to chaos-rpg shop/crafting pricing.

use std::collections::HashMap;
use crate::state::GameState;

// ═══════════════════════════════════════════════════════════════════════════════
// SUPPLY/DEMAND TRACKER
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks item purchase history for supply/demand pricing.
pub struct DemandTracker {
    /// How many times each item base type has been purchased this run.
    pub purchase_counts: HashMap<String, u32>,
    /// Price modifier per item (starts at 1.0, increases with demand).
    pub price_modifiers: HashMap<String, f32>,
    /// Rounds since last purchase (for supply decay).
    pub rounds_since_purchase: HashMap<String, u32>,
}

impl DemandTracker {
    pub fn new() -> Self {
        Self {
            purchase_counts: HashMap::new(),
            price_modifiers: HashMap::new(),
            rounds_since_purchase: HashMap::new(),
        }
    }

    /// Record a purchase. Increases demand for that item type.
    pub fn record_purchase(&mut self, item_type: &str) {
        let count = self.purchase_counts.entry(item_type.to_string()).or_insert(0);
        *count += 1;
        // Price increases: +10% per purchase, max 2.5x
        let modifier = self.price_modifiers.entry(item_type.to_string()).or_insert(1.0);
        *modifier = (*modifier + 0.1).min(2.5);
        // Reset rounds counter
        self.rounds_since_purchase.insert(item_type.to_string(), 0);
    }

    /// Tick supply decay — items not purchased become cheaper over time.
    pub fn tick_floor(&mut self) {
        for (item_type, rounds) in self.rounds_since_purchase.iter_mut() {
            *rounds += 1;
            if *rounds > 3 {
                // Supply exceeds demand: reduce price by 5% per floor without purchase
                if let Some(modifier) = self.price_modifiers.get_mut(item_type) {
                    *modifier = (*modifier - 0.05).max(0.5); // minimum 50% of base price
                }
            }
        }
    }

    /// Get the price modifier for an item type.
    pub fn price_modifier(&self, item_type: &str) -> f32 {
        self.price_modifiers.get(item_type).copied().unwrap_or(1.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FACTION ECONOMY
// ═══════════════════════════════════════════════════════════════════════════════

/// Faction economic state.
pub struct FactionEconomy {
    pub treasuries: HashMap<String, i64>,
    pub embargoes: Vec<(String, String)>, // (faction_a, faction_b) embargo pairs
}

impl FactionEconomy {
    pub fn new() -> Self {
        let mut treasuries = HashMap::new();
        // Default faction treasuries
        treasuries.insert("Archivist".to_string(), 10000);
        treasuries.insert("Forge".to_string(), 8000);
        treasuries.insert("Void".to_string(), 5000);
        treasuries.insert("Chaos".to_string(), 3000);

        Self {
            treasuries,
            embargoes: Vec::new(),
        }
    }

    /// Get the shop inventory quality modifier for a faction.
    /// Rich factions (treasury > 8000) have better shops.
    pub fn inventory_quality(&self, faction: &str) -> f32 {
        let treasury = self.treasuries.get(faction).copied().unwrap_or(5000);
        if treasury > 12000 { 1.5 }      // Rich: 50% better items
        else if treasury > 8000 { 1.2 }   // Comfortable: 20% better
        else if treasury > 4000 { 1.0 }   // Normal
        else if treasury > 1000 { 0.8 }   // Poor: 20% worse
        else { 0.6 }                       // Desperate: 40% worse
    }

    /// Get the price discount from faction treasury (poor factions = cheaper but worse items).
    pub fn price_factor(&self, faction: &str) -> f32 {
        let treasury = self.treasuries.get(faction).copied().unwrap_or(5000);
        if treasury > 10000 { 1.3 }       // Rich: 30% markup
        else if treasury > 6000 { 1.0 }   // Normal pricing
        else if treasury > 2000 { 0.8 }   // Discount: 20% off
        else { 0.6 }                       // Desperate prices: 40% off
    }

    /// Check if a faction has an embargo against another.
    pub fn is_embargoed(&self, faction_a: &str, faction_b: &str) -> bool {
        self.embargoes.iter().any(|(a, b)|
            (a == faction_a && b == faction_b) || (a == faction_b && b == faction_a)
        )
    }

    /// Record a kill of a faction member — reduces their treasury.
    pub fn on_faction_kill(&mut self, faction: &str, value: i64) {
        if let Some(treasury) = self.treasuries.get_mut(faction) {
            *treasury = (*treasury - value).max(0);
        }
    }

    /// Record a purchase from a faction shop — increases their treasury.
    pub fn on_purchase(&mut self, faction: &str, amount: i64) {
        if let Some(treasury) = self.treasuries.get_mut(faction) {
            *treasury += amount;
        }
    }

    /// Tick faction economy each floor (trade income, maintenance costs).
    pub fn tick_floor(&mut self) {
        for (_faction, treasury) in self.treasuries.iter_mut() {
            // Passive income
            *treasury += 200;
            // Maintenance cost
            *treasury = (*treasury - 100).max(0);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REPUTATION PRICING
// ═══════════════════════════════════════════════════════════════════════════════

/// Get the reputation-based price discount for a player with a faction.
/// Returns a multiplier (1.0 = no discount, 0.7 = 30% off).
pub fn reputation_discount(faction_rep: i32) -> f32 {
    if faction_rep > 100 { 0.7 }       // Exalted: 30% off
    else if faction_rep > 50 { 0.8 }   // Honored: 20% off
    else if faction_rep > 20 { 0.9 }   // Friendly: 10% off
    else if faction_rep > 0 { 0.95 }   // Neutral+: 5% off
    else if faction_rep > -20 { 1.0 }  // Neutral: no change
    else if faction_rep > -50 { 1.1 }  // Disliked: 10% markup
    else { 1.3 }                        // Hated: 30% markup
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITE PRICING
// ═══════════════════════════════════════════════════════════════════════════════

/// Full economy state for the game.
pub struct GameEconomy {
    pub demand: DemandTracker,
    pub factions: FactionEconomy,
}

impl GameEconomy {
    pub fn new() -> Self {
        Self {
            demand: DemandTracker::new(),
            factions: FactionEconomy::new(),
        }
    }

    /// Calculate the final price of an item considering all economic factors.
    pub fn calculate_price(
        &self,
        base_price: i64,
        item_type: &str,
        shop_faction: &str,
        player_rep: i32,
    ) -> i64 {
        let demand_mult = self.demand.price_modifier(item_type);
        let faction_mult = self.factions.price_factor(shop_faction);
        let rep_mult = reputation_discount(player_rep);

        let final_price = base_price as f32 * demand_mult * faction_mult * rep_mult;
        (final_price as i64).max(1)
    }

    /// Generate Archivist price announcement based on recent changes.
    pub fn price_announcement(&self) -> Option<String> {
        // Find the most inflated item
        let mut most_inflated: Option<(&str, f32)> = None;
        for (item_type, modifier) in &self.demand.price_modifiers {
            if *modifier > 1.3 {
                if most_inflated.is_none() || *modifier > most_inflated.unwrap().1 {
                    most_inflated = Some((item_type, *modifier));
                }
            }
        }
        // Find the most deflated item
        let mut most_deflated: Option<(&str, f32)> = None;
        for (item_type, modifier) in &self.demand.price_modifiers {
            if *modifier < 0.8 {
                if most_deflated.is_none() || *modifier < most_deflated.unwrap().1 {
                    most_deflated = Some((item_type, *modifier));
                }
            }
        }

        if let Some((item, mult)) = most_inflated {
            Some(format!(
                "\"{}s have increased in value. The proof demands more weight.\" (+{:.0}%)",
                item, (mult - 1.0) * 100.0
            ))
        } else if let Some((item, mult)) = most_deflated {
            Some(format!(
                "\"{}s gather dust. Perhaps a bargain awaits.\" (-{:.0}%)",
                item, (1.0 - mult) * 100.0
            ))
        } else {
            None
        }
    }

    /// Tick economy each floor.
    pub fn tick_floor(&mut self) {
        self.demand.tick_floor();
        self.factions.tick_floor();
    }
}
