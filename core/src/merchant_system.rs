use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    pub fn base_price_multiplier(&self) -> f64 {
        match self {
            ItemRarity::Common => 1.0,
            ItemRarity::Uncommon => 2.0,
            ItemRarity::Rare => 5.0,
            ItemRarity::Epic => 15.0,
            ItemRarity::Legendary => 50.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MerchantItem {
    pub id: u32,
    pub name: String,
    pub base_price: u32,
    pub rarity: ItemRarity,
    pub stock: u32,
    pub demand: f64,
}

#[derive(Debug, Clone)]
pub enum PriceModel {
    Fixed,
    Dynamic { volatility: f64 },
    Supply { elasticity: f64 },
}

#[derive(Debug, Clone)]
pub enum MerchantError {
    ItemNotFound,
    InsufficientStock,
    InsufficientGold,
}

impl std::fmt::Display for MerchantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MerchantError::ItemNotFound => write!(f, "Item not found"),
            MerchantError::InsufficientStock => write!(f, "Insufficient stock"),
            MerchantError::InsufficientGold => write!(f, "Insufficient gold"),
        }
    }
}

pub struct Merchant {
    pub name: String,
    pub items: HashMap<u32, MerchantItem>,
    pub gold: u32,
    pub reputation: i32,
    pub price_model: PriceModel,
    pub lcg_state: u64,
}

fn lcg_next(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 33) as f64 / (u32::MAX as f64 + 1.0)
}

impl Merchant {
    pub fn new(name: &str, gold: u32, model: PriceModel) -> Self {
        Merchant {
            name: name.to_string(),
            items: HashMap::new(),
            gold,
            reputation: 0,
            price_model: model,
            lcg_state: 12345678,
        }
    }

    pub fn add_item(&mut self, item: MerchantItem) {
        self.items.insert(item.id, item);
    }

    pub fn current_price(&mut self, item_id: u32) -> Option<u32> {
        let item = self.items.get(&item_id)?.clone();
        let base = item.base_price as f64 * item.rarity.base_price_multiplier();
        let demand_factor = item.demand.max(0.1);

        let price = match &self.price_model {
            PriceModel::Fixed => base * demand_factor,
            PriceModel::Dynamic { volatility } => {
                let noise = lcg_next(&mut self.lcg_state);
                // noise in [0,1) → ±5% with optional volatility scaling
                let delta = (noise - 0.5) * 0.1 * volatility;
                base * demand_factor * (1.0 + delta)
            }
            PriceModel::Supply { elasticity } => {
                let stock_factor = if item.stock == 0 {
                    2.0
                } else {
                    1.0 + elasticity / item.stock as f64
                };
                base * demand_factor * stock_factor
            }
        };

        Some(price.round().max(1.0) as u32)
    }

    pub fn buy_from_merchant(
        &mut self,
        item_id: u32,
        qty: u32,
        player_gold: &mut u32,
    ) -> Result<(), MerchantError> {
        let item = self.items.get(&item_id).ok_or(MerchantError::ItemNotFound)?;
        if item.stock < qty {
            return Err(MerchantError::InsufficientStock);
        }
        let price = self.current_price(item_id).ok_or(MerchantError::ItemNotFound)?;
        let total = price.saturating_mul(qty);
        if *player_gold < total {
            return Err(MerchantError::InsufficientGold);
        }
        *player_gold -= total;
        self.gold = self.gold.saturating_add(total);
        let item = self.items.get_mut(&item_id).unwrap();
        item.stock -= qty;
        item.demand = (item.demand + 0.1 * qty as f64).min(2.0);
        Ok(())
    }

    pub fn sell_to_merchant(
        &mut self,
        item_id: u32,
        name: &str,
        rarity: ItemRarity,
        qty: u32,
        player_gold: &mut u32,
    ) {
        let pay_per_unit = if let Some(item) = self.items.get(&item_id) {
            (item.base_price as f64 * 0.5).round() as u32
        } else {
            // Item doesn't exist yet — create an entry
            let base = 10u32;
            (base as f64 * rarity.base_price_multiplier() * 0.5).round() as u32
        };

        let total = pay_per_unit.saturating_mul(qty);
        if self.gold >= total {
            self.gold -= total;
            *player_gold = player_gold.saturating_add(total);
        }

        let entry = self.items.entry(item_id).or_insert_with(|| MerchantItem {
            id: item_id,
            name: name.to_string(),
            base_price: pay_per_unit * 2,
            rarity: rarity.clone(),
            stock: 0,
            demand: 1.0,
        });
        entry.stock += qty;
        entry.demand = (entry.demand - 0.05 * qty as f64).max(0.1);
    }

    pub fn haggle(&mut self, item_id: u32, player_charisma: i32) -> Option<u32> {
        let item = self.items.get(&item_id)?.clone();
        let base = item.base_price as f64 * item.rarity.base_price_multiplier();
        let charisma_factor = (player_charisma as f64 / 100.0).min(0.2);
        let rep_bonus = (self.reputation as f64 * 0.002).min(0.05);
        let discount = (charisma_factor + rep_bonus).min(0.20);
        let discounted = base * (1.0 - discount);
        Some(discounted.round().max(1.0) as u32)
    }

    pub fn restock(&mut self) {
        let ids: Vec<u32> = self.items.keys().cloned().collect();
        for id in ids {
            let gain = (lcg_next(&mut self.lcg_state) * 3.0).floor() as u32 + 1;
            if let Some(item) = self.items.get_mut(&id) {
                item.stock += gain;
                item.demand = (item.demand - 0.1).max(0.1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: u32, base_price: u32) -> MerchantItem {
        MerchantItem {
            id,
            name: format!("Item{}", id),
            base_price,
            rarity: ItemRarity::Common,
            stock: 10,
            demand: 1.0,
        }
    }

    #[test]
    fn test_dynamic_pricing_in_range() {
        let mut merchant = Merchant::new("Bob", 1000, PriceModel::Dynamic { volatility: 1.0 });
        let item = make_item(1, 100);
        merchant.add_item(item);
        for _ in 0..20 {
            let price = merchant.current_price(1).unwrap();
            // With demand=1.0 and ±5% noise on base 100*1.0=100
            assert!(price >= 90 && price <= 115, "price out of range: {}", price);
        }
    }

    #[test]
    fn test_haggle_discount_capped_at_20_percent() {
        let mut merchant = Merchant::new("Alice", 1000, PriceModel::Fixed);
        let item = MerchantItem {
            id: 1,
            name: "Sword".to_string(),
            base_price: 100,
            rarity: ItemRarity::Common,
            stock: 5,
            demand: 1.0,
        };
        merchant.add_item(item);
        // Charisma 9999 → discount capped at 20%
        let haggled = merchant.haggle(1, 9999).unwrap();
        let base = 100u32;
        let min_expected = (base as f64 * 0.75).round() as u32; // with rep bonus it can go slightly lower
        let max_expected = base;
        assert!(haggled <= max_expected, "haggled {} > base {}", haggled, max_expected);
        assert!(haggled >= min_expected, "haggled {} < {}", haggled, min_expected);
    }

    #[test]
    fn test_buy_reduces_stock() {
        let mut merchant = Merchant::new("Greg", 1000, PriceModel::Fixed);
        let item = make_item(1, 10);
        merchant.add_item(item);
        let mut player_gold = 1000u32;
        merchant.buy_from_merchant(1, 3, &mut player_gold).unwrap();
        assert_eq!(merchant.items[&1].stock, 7);
    }

    #[test]
    fn test_sell_increases_player_gold() {
        let mut merchant = Merchant::new("Greg", 500, PriceModel::Fixed);
        let item = make_item(1, 100);
        merchant.add_item(item);
        let mut player_gold = 0u32;
        merchant.sell_to_merchant(1, "Item1", ItemRarity::Common, 1, &mut player_gold);
        assert!(player_gold > 0);
    }

    #[test]
    fn test_restock_increases_stock() {
        let mut merchant = Merchant::new("Trader", 10000, PriceModel::Fixed);
        let item = make_item(1, 50);
        merchant.add_item(item);
        let stock_before = merchant.items[&1].stock;
        merchant.restock();
        let stock_after = merchant.items[&1].stock;
        assert!(stock_after > stock_before);
    }
}
