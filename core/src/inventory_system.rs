//! Full inventory management system.
//!
//! Tracks items, quantities, weight limits, and provides filtering/sorting.

use std::collections::HashMap;
use thiserror::Error;
use serde::{Deserialize, Serialize};

// ─── ERRORS ───────────────────────────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq)]
pub enum InventoryError {
    #[error("inventory is full (weight limit exceeded)")]
    Full,
    #[error("item '{0}' not found in inventory")]
    NotFound(String),
    #[error("stack limit ({0}) would be exceeded")]
    StackLimit(u32),
    #[error("item '{0}' is not stackable")]
    NotStackable(String),
    #[error("cannot remove {want} of '{id}': only {have} available")]
    InsufficientQuantity { id: String, want: u32, have: u32 },
}

// ─── ITEM TYPE ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Consumable,
    Material,
    QuestItem,
    Currency,
}

// ─── ITEM ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    /// Weight per unit in kilograms.
    pub weight: f64,
    /// Gold value per unit.
    pub value: u64,
    pub stackable: bool,
    pub max_stack: u32,
}

// ─── INVENTORY SLOT ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: Item,
    pub quantity: u32,
}

impl InventorySlot {
    pub fn total_weight(&self) -> f64 {
        self.item.weight * self.quantity as f64
    }

    pub fn total_value(&self) -> u64 {
        self.item.value.saturating_mul(self.quantity as u64)
    }
}

// ─── INVENTORY ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    /// Slots keyed by item ID.
    slots: HashMap<String, InventorySlot>,
    /// Maximum total weight this inventory can hold.
    pub max_weight: f64,
}

impl Inventory {
    pub fn new(max_weight: f64) -> Self {
        Self {
            slots: HashMap::new(),
            max_weight,
        }
    }

    /// Add `qty` units of `item` to the inventory.
    pub fn add_item(&mut self, item: Item, qty: u32) -> Result<(), InventoryError> {
        if qty == 0 {
            return Ok(());
        }
        let new_weight = item.weight * qty as f64;
        if self.total_weight() + new_weight > self.max_weight {
            return Err(InventoryError::Full);
        }

        if let Some(slot) = self.slots.get_mut(&item.id) {
            // Stacking
            if !item.stackable {
                return Err(InventoryError::NotStackable(item.id.clone()));
            }
            let new_qty = slot.quantity + qty;
            if new_qty > item.max_stack {
                return Err(InventoryError::StackLimit(item.max_stack));
            }
            slot.quantity = new_qty;
        } else {
            if !item.stackable && qty > 1 {
                return Err(InventoryError::NotStackable(item.id.clone()));
            }
            if item.stackable && qty > item.max_stack {
                return Err(InventoryError::StackLimit(item.max_stack));
            }
            self.slots.insert(
                item.id.clone(),
                InventorySlot { item, quantity: qty },
            );
        }
        Ok(())
    }

    /// Remove `qty` units of the item with `item_id`.
    pub fn remove_item(&mut self, item_id: &str, qty: u32) -> Result<(), InventoryError> {
        if qty == 0 {
            return Ok(());
        }
        let slot = self
            .slots
            .get_mut(item_id)
            .ok_or_else(|| InventoryError::NotFound(item_id.to_owned()))?;

        if slot.quantity < qty {
            return Err(InventoryError::InsufficientQuantity {
                id: item_id.to_owned(),
                want: qty,
                have: slot.quantity,
            });
        }
        slot.quantity -= qty;
        if slot.quantity == 0 {
            self.slots.remove(item_id);
        }
        Ok(())
    }

    /// Check whether the inventory contains at least `qty` of the given item.
    pub fn has_item(&self, item_id: &str, qty: u32) -> bool {
        self.slots
            .get(item_id)
            .map(|s| s.quantity >= qty)
            .unwrap_or(false)
    }

    /// Total weight of all items currently held.
    pub fn total_weight(&self) -> f64 {
        self.slots.values().map(|s| s.total_weight()).sum()
    }

    /// Total gold value of all items.
    pub fn total_value(&self) -> u64 {
        self.slots.values().map(|s| s.total_value()).sum()
    }

    /// Return all slots whose item type matches.
    pub fn items_of_type(&self, item_type: &ItemType) -> Vec<&InventorySlot> {
        self.slots
            .values()
            .filter(|s| &s.item.item_type == item_type)
            .collect()
    }

    /// Sort slots by descending unit value, returning them as a Vec.
    pub fn sort_by_value(&self) -> Vec<&InventorySlot> {
        let mut slots: Vec<&InventorySlot> = self.slots.values().collect();
        slots.sort_by(|a, b| {
            b.item.value.cmp(&a.item.value)
        });
        slots
    }

    /// Fraction of max_weight currently occupied (clamped to [0, 1]).
    pub fn capacity_used(&self) -> f64 {
        if self.max_weight <= 0.0 {
            return 1.0;
        }
        (self.total_weight() / self.max_weight).min(1.0)
    }

    /// Number of distinct item stacks.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Iterate over all slots.
    pub fn iter(&self) -> impl Iterator<Item = &InventorySlot> {
        self.slots.values()
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sword() -> Item {
        Item {
            id: "sword_001".to_owned(),
            name: "Iron Sword".to_owned(),
            item_type: ItemType::Weapon,
            weight: 3.0,
            value: 150,
            stackable: false,
            max_stack: 1,
        }
    }

    fn potion() -> Item {
        Item {
            id: "potion_hp".to_owned(),
            name: "Health Potion".to_owned(),
            item_type: ItemType::Consumable,
            weight: 0.5,
            value: 30,
            stackable: true,
            max_stack: 20,
        }
    }

    fn gold_coin() -> Item {
        Item {
            id: "gold".to_owned(),
            name: "Gold Coin".to_owned(),
            item_type: ItemType::Currency,
            weight: 0.01,
            value: 1,
            stackable: true,
            max_stack: 9999,
        }
    }

    fn ore() -> Item {
        Item {
            id: "iron_ore".to_owned(),
            name: "Iron Ore".to_owned(),
            item_type: ItemType::Material,
            weight: 1.0,
            value: 10,
            stackable: true,
            max_stack: 50,
        }
    }

    #[test]
    fn test_add_non_stackable_item() {
        let mut inv = Inventory::new(100.0);
        inv.add_item(sword(), 1).unwrap();
        assert!(inv.has_item("sword_001", 1));
    }

    #[test]
    fn test_add_stackable_items() {
        let mut inv = Inventory::new(100.0);
        inv.add_item(potion(), 5).unwrap();
        inv.add_item(potion(), 3).unwrap();
        assert!(inv.has_item("potion_hp", 8));
    }

    #[test]
    fn test_add_item_exceeds_weight() {
        let mut inv = Inventory::new(5.0);
        inv.add_item(sword(), 1).unwrap(); // 3kg
        let err = inv.add_item(sword(), 1).unwrap_err(); // would be 6kg total
        // Second sword: non-stackable but we test weight first... actually weight check happens first.
        // Let's use ore to force the weight limit.
        let mut inv2 = Inventory::new(2.0);
        let err2 = inv2.add_item(ore(), 3).unwrap_err(); // 3kg > 2kg
        assert!(matches!(err2, InventoryError::Full));
        let _ = err; // suppress unused warning
    }

    #[test]
    fn test_remove_item() {
        let mut inv = Inventory::new(100.0);
        inv.add_item(potion(), 10).unwrap();
        inv.remove_item("potion_hp", 4).unwrap();
        assert!(inv.has_item("potion_hp", 6));
        assert!(!inv.has_item("potion_hp", 7));
    }

    #[test]
    fn test_remove_all_items_clears_slot() {
        let mut inv = Inventory::new(100.0);
        inv.add_item(potion(), 5).unwrap();
        inv.remove_item("potion_hp", 5).unwrap();
        assert!(!inv.has_item("potion_hp", 1));
        assert_eq!(inv.slot_count(), 0);
    }

    #[test]
    fn test_remove_item_not_found() {
        let mut inv = Inventory::new(100.0);
        let err = inv.remove_item("ghost_item", 1).unwrap_err();
        assert!(matches!(err, InventoryError::NotFound(_)));
    }

    #[test]
    fn test_remove_item_insufficient_quantity() {
        let mut inv = Inventory::new(100.0);
        inv.add_item(potion(), 3).unwrap();
        let err = inv.remove_item("potion_hp", 10).unwrap_err();
        assert!(matches!(err, InventoryError::InsufficientQuantity { .. }));
    }

    #[test]
    fn test_total_weight() {
        let mut inv = Inventory::new(200.0);
        inv.add_item(sword(), 1).unwrap();   // 3.0
        inv.add_item(potion(), 4).unwrap();  // 2.0
        let w = inv.total_weight();
        assert!((w - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_total_value() {
        let mut inv = Inventory::new(200.0);
        inv.add_item(sword(), 1).unwrap();   // 150
        inv.add_item(potion(), 2).unwrap();  // 60
        assert_eq!(inv.total_value(), 210);
    }

    #[test]
    fn test_items_of_type() {
        let mut inv = Inventory::new(200.0);
        inv.add_item(sword(), 1).unwrap();
        inv.add_item(potion(), 5).unwrap();
        inv.add_item(gold_coin(), 100).unwrap();

        let consumables = inv.items_of_type(&ItemType::Consumable);
        assert_eq!(consumables.len(), 1);
        let weapons = inv.items_of_type(&ItemType::Weapon);
        assert_eq!(weapons.len(), 1);
        let armor = inv.items_of_type(&ItemType::Armor);
        assert_eq!(armor.len(), 0);
    }

    #[test]
    fn test_sort_by_value() {
        let mut inv = Inventory::new(200.0);
        inv.add_item(potion(), 1).unwrap();    // value 30
        inv.add_item(sword(), 1).unwrap();     // value 150
        inv.add_item(gold_coin(), 1).unwrap(); // value 1

        let sorted = inv.sort_by_value();
        assert_eq!(sorted[0].item.value, 150);
        assert_eq!(sorted[1].item.value, 30);
        assert_eq!(sorted[2].item.value, 1);
    }

    #[test]
    fn test_capacity_used() {
        let mut inv = Inventory::new(10.0);
        inv.add_item(sword(), 1).unwrap(); // 3kg / 10kg = 0.3
        let cap = inv.capacity_used();
        assert!((cap - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_stack_limit_error() {
        let mut inv = Inventory::new(200.0);
        inv.add_item(potion(), 19).unwrap();
        let err = inv.add_item(potion(), 5).unwrap_err(); // 19+5 = 24 > 20
        assert!(matches!(err, InventoryError::StackLimit(20)));
    }

    #[test]
    fn test_not_stackable_error() {
        let mut inv = Inventory::new(200.0);
        inv.add_item(sword(), 1).unwrap();
        let err = inv.add_item(sword(), 1).unwrap_err();
        assert!(matches!(err, InventoryError::NotStackable(_)));
    }
}
