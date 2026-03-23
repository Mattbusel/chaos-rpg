//! In-game economy with supply/demand dynamics.
//!
//! Price formula: `price = base_price * sqrt(demand / supply)`,
//! clamped to `[base / 4, base * 8]`.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Good
// ---------------------------------------------------------------------------

/// Tradeable goods available in markets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Good {
    Food,
    Wood,
    Iron,
    Gold,
    Gems,
    MagicComponents,
}

impl Good {
    /// Base price in copper coins.
    pub fn base_price(&self) -> u64 {
        match self {
            Good::Food           => 10,
            Good::Wood           => 8,
            Good::Iron           => 25,
            Good::Gold           => 200,
            Good::Gems           => 500,
            Good::MagicComponents => 1000,
        }
    }

    pub fn all() -> &'static [Good] {
        &[
            Good::Food,
            Good::Wood,
            Good::Iron,
            Good::Gold,
            Good::Gems,
            Good::MagicComponents,
        ]
    }
}

// ---------------------------------------------------------------------------
// MarketGood
// ---------------------------------------------------------------------------

/// Per-good market state including price history.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketGood {
    pub supply: u64,
    pub demand: u64,
    pub current_price: u64,
    pub price_history: Vec<u64>,
}

impl MarketGood {
    pub fn new(supply: u64, demand: u64, base: u64) -> Self {
        let price = compute_price(base, supply, demand);
        MarketGood {
            supply,
            demand,
            current_price: price,
            price_history: vec![price],
        }
    }

    pub fn update_price(&mut self, base: u64) {
        let price = compute_price(base, self.supply, self.demand);
        self.current_price = price;
        self.price_history.push(price);
    }
}

fn compute_price(base: u64, supply: u64, demand: u64) -> u64 {
    if supply == 0 {
        return base * 8;
    }
    let ratio = (demand as f64) / (supply as f64);
    let raw = (base as f64) * ratio.sqrt();
    let min_price = base / 4;
    let max_price = base * 8;
    (raw as u64).clamp(min_price.max(1), max_price)
}

// ---------------------------------------------------------------------------
// Market
// ---------------------------------------------------------------------------

/// A collection of goods with their market state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Market {
    pub goods: HashMap<Good, MarketGood>,
}

impl Market {
    pub fn new() -> Self {
        let mut goods = HashMap::new();
        for g in Good::all() {
            goods.insert(*g, MarketGood::new(100, 100, g.base_price()));
        }
        Market { goods }
    }
}

impl Default for Market {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// EconomyError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum EconomyError {
    #[error("insufficient supply: need {needed}, have {available}")]
    InsufficientSupply { needed: u64, available: u64 },

    #[error("unknown good")]
    UnknownGood,
}

// ---------------------------------------------------------------------------
// Trend
// ---------------------------------------------------------------------------

/// Direction of recent price movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Rising,
    Falling,
    Stable,
}

// ---------------------------------------------------------------------------
// Economy
// ---------------------------------------------------------------------------

/// Top-level economy manager.
pub struct Economy {
    pub market: Market,
    lcg_state: u64,
    tick_count: u64,
}

const LCG_A: u64 = 1664525;
const LCG_C: u64 = 1013904223;
const LCG_M: u64 = 1 << 32;

fn lcg_next(state: u64) -> u64 {
    (LCG_A.wrapping_mul(state).wrapping_add(LCG_C)) % LCG_M
}

impl Economy {
    pub fn new() -> Self {
        Economy {
            market: Market::new(),
            lcg_state: 1,
            tick_count: 0,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Economy {
            market: Market::new(),
            lcg_state: seed,
            tick_count: 0,
        }
    }

    /// Purchase `qty` units of `good`. Returns total cost. Reduces supply.
    pub fn buy(&mut self, good: Good, qty: u64) -> Result<u64, EconomyError> {
        let mg = self.market.goods.get_mut(&good).ok_or(EconomyError::UnknownGood)?;
        if mg.supply < qty {
            return Err(EconomyError::InsufficientSupply {
                needed: qty,
                available: mg.supply,
            });
        }
        let cost = mg.current_price * qty;
        mg.supply -= qty;
        // Buying increases demand slightly (reflects market signal).
        mg.demand = mg.demand.saturating_add(qty / 2);
        mg.update_price(good.base_price());
        Ok(cost)
    }

    /// Sell `qty` units of `good`. Returns total revenue. Increases supply.
    pub fn sell(&mut self, good: Good, qty: u64) -> u64 {
        let mg = self.market.goods.entry(good).or_insert_with(|| {
            MarketGood::new(100, 100, good.base_price())
        });
        let revenue = mg.current_price * qty;
        mg.supply = mg.supply.saturating_add(qty);
        // Selling decreases demand slightly.
        mg.demand = mg.demand.saturating_sub(qty / 4);
        mg.update_price(good.base_price());
        revenue
    }

    /// Advance the economy by one tick — random demand fluctuations, price update.
    pub fn tick(&mut self) {
        self.tick_count += 1;
        for good in Good::all() {
            self.lcg_state = lcg_next(self.lcg_state ^ self.tick_count);
            let r = (self.lcg_state as f64) / (LCG_M as f64);

            // Demand fluctuates ±20 % of current demand.
            if let Some(mg) = self.market.goods.get_mut(good) {
                let delta = (mg.demand as f64 * 0.20 * (r * 2.0 - 1.0)) as i64;
                mg.demand = ((mg.demand as i64) + delta).max(1) as u64;
                mg.update_price(good.base_price());
            }
        }
    }

    /// Returns the price trend for a good based on recent history.
    pub fn price_trend(&self, good: Good) -> Trend {
        let mg = match self.market.goods.get(&good) {
            Some(g) => g,
            None => return Trend::Stable,
        };
        let history = &mg.price_history;
        if history.len() < 3 {
            return Trend::Stable;
        }
        let recent = history[history.len() - 1] as i64;
        let older  = history[history.len() - 3] as i64;
        let diff = recent - older;
        if diff > (older / 10) {
            Trend::Rising
        } else if diff < -(older / 10) {
            Trend::Falling
        } else {
            Trend::Stable
        }
    }

    /// Goods whose current price deviates more than 20 % from their base price.
    pub fn arbitrage_opportunities(&self) -> Vec<(Good, f64)> {
        let mut opps = Vec::new();
        for good in Good::all() {
            if let Some(mg) = self.market.goods.get(good) {
                let base = good.base_price() as f64;
                let current = mg.current_price as f64;
                let deviation = (current - base) / base;
                if deviation.abs() > 0.20 {
                    opps.push((*good, deviation));
                }
            }
        }
        opps
    }
}

impl Default for Economy {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_prices_positive() {
        for good in Good::all() {
            assert!(good.base_price() > 0, "{:?} base price must be positive", good);
        }
    }

    #[test]
    fn test_compute_price_balanced_supply_demand() {
        let price = compute_price(100, 100, 100);
        assert_eq!(price, 100);
    }

    #[test]
    fn test_compute_price_high_demand_raises_price() {
        let low_demand  = compute_price(100, 100, 50);
        let high_demand = compute_price(100, 100, 200);
        assert!(high_demand > low_demand);
    }

    #[test]
    fn test_compute_price_clamps_to_max() {
        let price = compute_price(100, 1, 1_000_000);
        assert_eq!(price, 800); // base * 8
    }

    #[test]
    fn test_compute_price_clamps_to_min() {
        let price = compute_price(100, 1_000_000, 1);
        assert_eq!(price, 25); // base / 4
    }

    #[test]
    fn test_buy_reduces_supply() {
        let mut econ = Economy::new();
        let initial_supply = econ.market.goods[&Good::Food].supply;
        econ.buy(Good::Food, 10).unwrap();
        assert_eq!(econ.market.goods[&Good::Food].supply, initial_supply - 10);
    }

    #[test]
    fn test_buy_returns_cost() {
        let mut econ = Economy::new();
        let price = econ.market.goods[&Good::Iron].current_price;
        let cost = econ.buy(Good::Iron, 5).unwrap();
        // Cost may differ slightly after demand bump, but should be close.
        assert!(cost >= price * 5 / 2);
    }

    #[test]
    fn test_buy_insufficient_supply_returns_error() {
        let mut econ = Economy::new();
        let supply = econ.market.goods[&Good::Gems].supply;
        let result = econ.buy(Good::Gems, supply + 1);
        assert!(result.is_err());
        match result {
            Err(EconomyError::InsufficientSupply { .. }) => {}
            other => panic!("Expected InsufficientSupply, got {:?}", other),
        }
    }

    #[test]
    fn test_sell_increases_supply() {
        let mut econ = Economy::new();
        let before = econ.market.goods[&Good::Wood].supply;
        econ.sell(Good::Wood, 20);
        assert_eq!(econ.market.goods[&Good::Wood].supply, before + 20);
    }

    #[test]
    fn test_tick_updates_prices() {
        let mut econ = Economy::with_seed(42);
        let before: Vec<u64> = Good::all().iter().map(|g| econ.market.goods[g].current_price).collect();
        for _ in 0..5 {
            econ.tick();
        }
        let after: Vec<u64> = Good::all().iter().map(|g| econ.market.goods[g].current_price).collect();
        // At least one price must have changed.
        assert_ne!(before, after);
    }

    #[test]
    fn test_price_trend_rising() {
        let mut econ = Economy::new();
        let mg = econ.market.goods.get_mut(&Good::Gold).unwrap();
        mg.price_history = vec![100, 110, 130];
        mg.current_price = 130;
        assert_eq!(econ.price_trend(Good::Gold), Trend::Rising);
    }

    #[test]
    fn test_price_trend_falling() {
        let mut econ = Economy::new();
        let mg = econ.market.goods.get_mut(&Good::Gold).unwrap();
        mg.price_history = vec![130, 110, 100];
        mg.current_price = 100;
        assert_eq!(econ.price_trend(Good::Gold), Trend::Falling);
    }

    #[test]
    fn test_price_trend_stable() {
        let mut econ = Economy::new();
        let mg = econ.market.goods.get_mut(&Good::Gold).unwrap();
        mg.price_history = vec![100, 101, 102];
        mg.current_price = 102;
        assert_eq!(econ.price_trend(Good::Gold), Trend::Stable);
    }

    #[test]
    fn test_arbitrage_opportunities_returns_deviating_goods() {
        let mut econ = Economy::new();
        // Force price deviation by manipulating supply.
        let mg = econ.market.goods.get_mut(&Good::MagicComponents).unwrap();
        mg.current_price = mg.current_price * 3; // +200% above base
        let opps = econ.arbitrage_opportunities();
        assert!(!opps.is_empty(), "Expected at least one arbitrage opportunity");
        let found = opps.iter().any(|(g, _)| *g == Good::MagicComponents);
        assert!(found);
    }

    #[test]
    fn test_arbitrage_opportunities_deviation_above_20pct() {
        let econ = Economy::new();
        // At start all prices are near base, so no opportunities initially.
        // (supply == demand == 100 => price == base)
        let opps = econ.arbitrage_opportunities();
        for (_, dev) in &opps {
            assert!(dev.abs() > 0.20, "deviation must exceed 20%");
        }
    }

    #[test]
    fn test_market_good_price_history_grows_on_update() {
        let mut mg = MarketGood::new(100, 100, 50);
        let initial_len = mg.price_history.len();
        mg.update_price(50);
        assert_eq!(mg.price_history.len(), initial_len + 1);
    }

    #[test]
    fn test_all_goods_present_in_market() {
        let econ = Economy::new();
        for g in Good::all() {
            assert!(econ.market.goods.contains_key(g), "{:?} missing from market", g);
        }
    }
}
