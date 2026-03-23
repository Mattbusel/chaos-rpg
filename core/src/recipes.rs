//! Item crafting and recipe system.
//!
//! Provides a structured recipe system where players combine ingredients at
//! crafting stations to produce items. Distinct from the chaos-crafting system
//! in `crafting.rs` which handles item modification — this module handles
//! ingredient-based item creation.

use std::collections::HashMap;

// ─── INGREDIENT ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ingredient {
    pub item_name: String,
    pub quantity: u32,
}

impl Ingredient {
    pub fn new(item_name: impl Into<String>, quantity: u32) -> Self {
        Ingredient {
            item_name: item_name.into(),
            quantity,
        }
    }
}

// ─── CRAFTING STATION ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CraftingStation {
    Anvil,
    Alchemist,
    Workbench,
    MagicForge,
    Campfire,
}

impl CraftingStation {
    pub fn name(&self) -> &str {
        match self {
            CraftingStation::Anvil => "Anvil",
            CraftingStation::Alchemist => "Alchemist",
            CraftingStation::Workbench => "Workbench",
            CraftingStation::MagicForge => "Magic Forge",
            CraftingStation::Campfire => "Campfire",
        }
    }
}

// ─── RECIPE ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub result_item: String,
    pub result_quantity: u32,
    pub required_level: u32,
    pub crafting_time_turns: u32,
    pub station: CraftingStation,
}

impl Recipe {
    pub fn iron_sword() -> Self {
        Recipe {
            id: "iron_sword".to_string(),
            name: "Iron Sword".to_string(),
            ingredients: vec![
                Ingredient::new("Iron Ingot", 3),
                Ingredient::new("Wood Handle", 1),
            ],
            result_item: "Iron Sword".to_string(),
            result_quantity: 1,
            required_level: 1,
            crafting_time_turns: 3,
            station: CraftingStation::Anvil,
        }
    }

    pub fn health_potion() -> Self {
        Recipe {
            id: "health_potion".to_string(),
            name: "Health Potion".to_string(),
            ingredients: vec![
                Ingredient::new("Red Herb", 2),
                Ingredient::new("Water Vial", 1),
            ],
            result_item: "Health Potion".to_string(),
            result_quantity: 1,
            required_level: 1,
            crafting_time_turns: 2,
            station: CraftingStation::Alchemist,
        }
    }

    pub fn magic_staff() -> Self {
        Recipe {
            id: "magic_staff".to_string(),
            name: "Magic Staff".to_string(),
            ingredients: vec![
                Ingredient::new("Arcane Wood", 2),
                Ingredient::new("Mana Crystal", 3),
                Ingredient::new("Silver Wire", 1),
            ],
            result_item: "Magic Staff".to_string(),
            result_quantity: 1,
            required_level: 10,
            crafting_time_turns: 6,
            station: CraftingStation::MagicForge,
        }
    }

    pub fn arrow_bundle() -> Self {
        Recipe {
            id: "arrow_bundle".to_string(),
            name: "Arrow Bundle".to_string(),
            ingredients: vec![
                Ingredient::new("Feather", 5),
                Ingredient::new("Stick", 5),
                Ingredient::new("Flint", 2),
            ],
            result_item: "Arrow Bundle".to_string(),
            result_quantity: 20,
            required_level: 1,
            crafting_time_turns: 2,
            station: CraftingStation::Workbench,
        }
    }

    pub fn lockpick() -> Self {
        Recipe {
            id: "lockpick".to_string(),
            name: "Lockpick".to_string(),
            ingredients: vec![
                Ingredient::new("Wire".to_string(), 2),
                Ingredient::new("Small File".to_string(), 1),
            ],
            result_item: "Lockpick".to_string(),
            result_quantity: 3,
            required_level: 3,
            crafting_time_turns: 1,
            station: CraftingStation::Workbench,
        }
    }
}

// ─── CRAFTING ERROR ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CraftingError {
    UnknownRecipe,
    InsufficientMaterials { missing: Vec<Ingredient> },
    InsufficientLevel { required: u32, have: u32 },
    StationRequired(CraftingStation),
}

impl std::fmt::Display for CraftingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CraftingError::UnknownRecipe => write!(f, "Unknown recipe"),
            CraftingError::InsufficientMaterials { missing } => {
                write!(f, "Missing materials: ")?;
                for (i, ing) in missing.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}x {}", ing.quantity, ing.item_name)?;
                }
                Ok(())
            }
            CraftingError::InsufficientLevel { required, have } => {
                write!(f, "Level {} required, you are level {}", required, have)
            }
            CraftingError::StationRequired(station) => {
                write!(f, "Requires a {} to craft", station.name())
            }
        }
    }
}

// ─── CRAFTING RESULT ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct CraftingResult {
    pub item_name: String,
    pub quantity: u32,
    /// True if player level > recipe.required_level + 5 and the 10% bonus triggered.
    pub bonus_quality: bool,
}

// ─── RECIPE BOOK ──────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct RecipeBook {
    recipes: HashMap<String, Recipe>,
}

impl RecipeBook {
    pub fn new() -> Self {
        RecipeBook::default()
    }

    /// Add a recipe to the book.
    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Return all recipes usable at the given station.
    pub fn recipes_for_station(&self, station: CraftingStation) -> Vec<&Recipe> {
        self.recipes
            .values()
            .filter(|r| r.station == station)
            .collect()
    }

    /// Check whether crafting a recipe is possible given the inventory and player level.
    pub fn can_craft(
        &self,
        recipe_id: &str,
        inventory: &HashMap<String, u32>,
        player_level: u32,
    ) -> Result<(), CraftingError> {
        let recipe = self
            .recipes
            .get(recipe_id)
            .ok_or(CraftingError::UnknownRecipe)?;

        if player_level < recipe.required_level {
            return Err(CraftingError::InsufficientLevel {
                required: recipe.required_level,
                have: player_level,
            });
        }

        let mut missing = Vec::new();
        for ingredient in &recipe.ingredients {
            let have = inventory.get(&ingredient.item_name).copied().unwrap_or(0);
            if have < ingredient.quantity {
                missing.push(Ingredient::new(
                    &ingredient.item_name,
                    ingredient.quantity - have,
                ));
            }
        }

        if !missing.is_empty() {
            return Err(CraftingError::InsufficientMaterials { missing });
        }

        Ok(())
    }

    pub fn get(&self, recipe_id: &str) -> Option<&Recipe> {
        self.recipes.get(recipe_id)
    }
}

// ─── CRAFTING MANAGER ─────────────────────────────────────────────────────────

pub struct CraftingManager {
    pub recipe_book: RecipeBook,
    /// LCG state for bonus quality rolls.
    rng_state: u64,
}

impl CraftingManager {
    pub fn new(recipe_book: RecipeBook) -> Self {
        CraftingManager {
            recipe_book,
            rng_state: 12345678901234567,
        }
    }

    /// Create a manager with all built-in recipes pre-loaded.
    pub fn with_default_recipes() -> Self {
        let mut book = RecipeBook::new();
        book.add_recipe(Recipe::iron_sword());
        book.add_recipe(Recipe::health_potion());
        book.add_recipe(Recipe::magic_staff());
        book.add_recipe(Recipe::arrow_bundle());
        book.add_recipe(Recipe::lockpick());
        CraftingManager::new(book)
    }

    fn lcg_next(&mut self) -> u64 {
        // Linear congruential generator (Knuth)
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng_state
    }

    /// Attempt to craft a recipe. Deducts ingredients and adds the result to inventory.
    pub fn craft(
        &mut self,
        recipe_id: &str,
        inventory: &mut HashMap<String, u32>,
        player_level: u32,
    ) -> Result<CraftingResult, CraftingError> {
        self.recipe_book.can_craft(recipe_id, inventory, player_level)?;

        let recipe = self
            .recipe_book
            .get(recipe_id)
            .ok_or(CraftingError::UnknownRecipe)?
            .clone();

        // Deduct ingredients
        for ingredient in &recipe.ingredients {
            let entry = inventory.entry(ingredient.item_name.clone()).or_insert(0);
            *entry -= ingredient.quantity;
            // Clean up zero entries
            if *entry == 0 {
                inventory.remove(&ingredient.item_name);
            }
        }

        // Check for bonus quality: player is 5+ levels above requirement → 10% chance
        let bonus_quality = if player_level > recipe.required_level + 5 {
            let roll = self.lcg_next();
            (roll % 100) < 10
        } else {
            false
        };

        let quantity = recipe.result_quantity;
        let item_name = recipe.result_item.clone();

        // Add result to inventory
        *inventory.entry(item_name.clone()).or_insert(0) += quantity;

        Ok(CraftingResult {
            item_name,
            quantity,
            bonus_quality,
        })
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sword_inventory() -> HashMap<String, u32> {
        let mut inv = HashMap::new();
        inv.insert("Iron Ingot".to_string(), 5);
        inv.insert("Wood Handle".to_string(), 2);
        inv
    }

    fn make_manager() -> CraftingManager {
        CraftingManager::with_default_recipes()
    }

    #[test]
    fn successful_crafting_consumes_ingredients() {
        let mut manager = make_manager();
        let mut inv = make_sword_inventory();

        let result = manager.craft("iron_sword", &mut inv, 5).unwrap();

        assert_eq!(result.item_name, "Iron Sword");
        assert_eq!(result.quantity, 1);

        // Ingredients consumed
        assert_eq!(inv.get("Iron Ingot").copied().unwrap_or(0), 2); // 5 - 3
        assert_eq!(inv.get("Wood Handle").copied().unwrap_or(0), 1); // 2 - 1
    }

    #[test]
    fn result_added_to_inventory() {
        let mut manager = make_manager();
        let mut inv = make_sword_inventory();
        assert!(!inv.contains_key("Iron Sword"));

        manager.craft("iron_sword", &mut inv, 5).unwrap();

        assert_eq!(inv.get("Iron Sword").copied().unwrap_or(0), 1);
    }

    #[test]
    fn insufficient_materials_error() {
        let mut manager = make_manager();
        let mut inv: HashMap<String, u32> = HashMap::new();
        inv.insert("Iron Ingot".to_string(), 1); // Need 3

        let result = manager.craft("iron_sword", &mut inv, 5);
        match result {
            Err(CraftingError::InsufficientMaterials { missing }) => {
                // Should mention Iron Ingot (need 2 more) and Wood Handle (need 1)
                assert!(missing.iter().any(|m| m.item_name == "Iron Ingot"));
                assert!(missing.iter().any(|m| m.item_name == "Wood Handle"));
            }
            _ => panic!("Expected InsufficientMaterials error"),
        }
    }

    #[test]
    fn level_check_returns_error() {
        let mut manager = make_manager();
        let mut inv: HashMap<String, u32> = HashMap::new();
        inv.insert("Arcane Wood".to_string(), 2);
        inv.insert("Mana Crystal".to_string(), 3);
        inv.insert("Silver Wire".to_string(), 1);

        // magic_staff requires level 10, player is level 5
        let result = manager.craft("magic_staff", &mut inv, 5);
        assert_eq!(
            result,
            Err(CraftingError::InsufficientLevel {
                required: 10,
                have: 5
            })
        );
    }

    #[test]
    fn unknown_recipe_error() {
        let mut manager = make_manager();
        let mut inv = HashMap::new();
        let result = manager.craft("nonexistent_recipe", &mut inv, 99);
        assert_eq!(result, Err(CraftingError::UnknownRecipe));
    }

    #[test]
    fn arrow_bundle_yields_correct_quantity() {
        let mut manager = make_manager();
        let mut inv: HashMap<String, u32> = HashMap::new();
        inv.insert("Feather".to_string(), 10);
        inv.insert("Stick".to_string(), 10);
        inv.insert("Flint".to_string(), 5);

        let result = manager.craft("arrow_bundle", &mut inv, 5).unwrap();
        assert_eq!(result.quantity, 20);
        assert_eq!(inv.get("Arrow Bundle").copied().unwrap_or(0), 20);
    }

    #[test]
    fn recipes_for_station_filters_correctly() {
        let manager = make_manager();
        let anvil_recipes = manager.recipe_book.recipes_for_station(CraftingStation::Anvil);
        assert!(anvil_recipes.iter().any(|r| r.id == "iron_sword"));
        assert!(!anvil_recipes.iter().any(|r| r.id == "health_potion"));
    }

    #[test]
    fn can_craft_checks_level_before_materials() {
        let manager = make_manager();
        let inv = make_sword_inventory();
        // Player level 0 should fail on level check
        let result = manager.recipe_book.can_craft("iron_sword", &inv, 0);
        // iron_sword required_level is 1, so level 0 fails
        // Note: required_level is 1 which is > 0
        // Actually iron_sword required_level = 1, player level = 0 → fail
        assert!(matches!(result, Err(CraftingError::InsufficientLevel { .. })));
    }

    #[test]
    fn crafting_exact_ingredients_leaves_zero_in_inventory() {
        let mut manager = make_manager();
        let mut inv: HashMap<String, u32> = HashMap::new();
        inv.insert("Iron Ingot".to_string(), 3); // exactly enough
        inv.insert("Wood Handle".to_string(), 1); // exactly enough

        manager.craft("iron_sword", &mut inv, 5).unwrap();

        // Both should be consumed completely (removed from map)
        assert!(!inv.contains_key("Iron Ingot"));
        assert!(!inv.contains_key("Wood Handle"));
    }
}
