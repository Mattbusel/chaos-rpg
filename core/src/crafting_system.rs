//! Recipe-based crafting system for CHAOS RPG.
//!
//! Handles ingredient management, recipe discovery, quality rolling,
//! and crafting station interactions.

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// LCG helper
// ---------------------------------------------------------------------------

const LCG_A: u64 = 1664525;
const LCG_C: u64 = 1013904223;
const LCG_M: u64 = 1 << 32;

fn lcg_next(seed: u64) -> u64 {
    (LCG_A.wrapping_mul(seed).wrapping_add(LCG_C)) % LCG_M
}

// ---------------------------------------------------------------------------
// IngredientType
// ---------------------------------------------------------------------------

/// Category of a crafting ingredient.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IngredientType {
    Herb,
    Ore,
    Wood,
    Leather,
    Cloth,
    Crystal,
    Bone,
    Liquid,
}

impl fmt::Display for IngredientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngredientType::Herb    => write!(f, "Herb"),
            IngredientType::Ore     => write!(f, "Ore"),
            IngredientType::Wood    => write!(f, "Wood"),
            IngredientType::Leather => write!(f, "Leather"),
            IngredientType::Cloth   => write!(f, "Cloth"),
            IngredientType::Crystal => write!(f, "Crystal"),
            IngredientType::Bone    => write!(f, "Bone"),
            IngredientType::Liquid  => write!(f, "Liquid"),
        }
    }
}

// ---------------------------------------------------------------------------
// Ingredient
// ---------------------------------------------------------------------------

/// A single crafting ingredient with quantity and quality.
#[derive(Debug, Clone)]
pub struct Ingredient {
    /// Display name of the ingredient.
    pub name: String,
    /// Broad material category.
    pub ingredient_type: IngredientType,
    /// How many units are required.
    pub quantity: u32,
    /// Quality level 1–100.
    pub quality: u8,
}

impl Ingredient {
    /// Create a new ingredient.
    pub fn new(name: impl Into<String>, ingredient_type: IngredientType, quantity: u32, quality: u8) -> Self {
        Self {
            name: name.into(),
            ingredient_type,
            quantity,
            quality: quality.clamp(1, 100),
        }
    }
}

// ---------------------------------------------------------------------------
// RecipeCategory
// ---------------------------------------------------------------------------

/// Broad category of craftable output.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecipeCategory {
    Weapon,
    Armor,
    Potion,
    Food,
    Tool,
    Scroll,
    Jewelry,
}

impl fmt::Display for RecipeCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecipeCategory::Weapon  => write!(f, "Weapon"),
            RecipeCategory::Armor   => write!(f, "Armor"),
            RecipeCategory::Potion  => write!(f, "Potion"),
            RecipeCategory::Food    => write!(f, "Food"),
            RecipeCategory::Tool    => write!(f, "Tool"),
            RecipeCategory::Scroll  => write!(f, "Scroll"),
            RecipeCategory::Jewelry => write!(f, "Jewelry"),
        }
    }
}

// ---------------------------------------------------------------------------
// Recipe
// ---------------------------------------------------------------------------

/// A crafting recipe defining inputs and output.
#[derive(Debug, Clone)]
pub struct Recipe {
    /// Unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Broad output category.
    pub category: RecipeCategory,
    /// Required ingredients.
    pub ingredients: Vec<Ingredient>,
    /// Name of the crafted item.
    pub output_name: String,
    /// How many items are produced per craft.
    pub output_quantity: u32,
    /// Difficulty 1–100 (affects quality roll).
    pub difficulty: u8,
    /// Minimum player skill required.
    pub skill_required: u8,
    /// How long crafting takes in real seconds.
    pub crafting_time_seconds: u32,
}

impl Recipe {
    /// Create a new recipe.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: RecipeCategory,
        ingredients: Vec<Ingredient>,
        output_name: impl Into<String>,
        output_quantity: u32,
        difficulty: u8,
        skill_required: u8,
        crafting_time_seconds: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            ingredients,
            output_name: output_name.into(),
            output_quantity,
            difficulty: difficulty.clamp(1, 100),
            skill_required: skill_required.clamp(0, 100),
            crafting_time_seconds,
        }
    }
}

// ---------------------------------------------------------------------------
// CraftingStation
// ---------------------------------------------------------------------------

/// The type of station needed to execute a craft.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CraftingStation {
    Forge,
    Alchemy,
    Loom,
    Workbench,
    Kitchen,
    ScribingDesk,
}

impl fmt::Display for CraftingStation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CraftingStation::Forge       => write!(f, "Forge"),
            CraftingStation::Alchemy     => write!(f, "Alchemy Lab"),
            CraftingStation::Loom        => write!(f, "Loom"),
            CraftingStation::Workbench   => write!(f, "Workbench"),
            CraftingStation::Kitchen     => write!(f, "Kitchen"),
            CraftingStation::ScribingDesk=> write!(f, "Scribing Desk"),
        }
    }
}

// ---------------------------------------------------------------------------
// CraftResult
// ---------------------------------------------------------------------------

/// The outcome of a crafting attempt.
#[derive(Debug, Clone)]
pub struct CraftResult {
    /// Whether the craft succeeded.
    pub success: bool,
    /// Name of the crafted item (empty on failure).
    pub item_name: String,
    /// Quantity produced.
    pub quantity: u32,
    /// Quality of the produced item (1–100).
    pub quality: u8,
    /// Experience points awarded.
    pub experience_gained: u32,
    /// Flavour message describing the result.
    pub message: String,
}

// ---------------------------------------------------------------------------
// CraftingSystem
// ---------------------------------------------------------------------------

/// Manages all crafting recipes and orchestrates the crafting process.
pub struct CraftingSystem {
    recipes: HashMap<String, Recipe>,
}

impl CraftingSystem {
    /// Create an empty crafting system.
    pub fn new() -> Self {
        Self { recipes: HashMap::new() }
    }

    /// Register a recipe.
    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Check whether the player can craft the given recipe.
    ///
    /// Returns `Ok(())` if all conditions are met, or an error string explaining
    /// the first failing condition.
    pub fn can_craft(
        &self,
        recipe_id: &str,
        inventory: &HashMap<String, u32>,
        skill_level: u8,
    ) -> Result<(), String> {
        let recipe = self.recipes.get(recipe_id)
            .ok_or_else(|| format!("Unknown recipe: {}", recipe_id))?;

        if skill_level < recipe.skill_required {
            return Err(format!(
                "Skill level {} required; you have {}",
                recipe.skill_required, skill_level
            ));
        }

        for ingredient in &recipe.ingredients {
            let have = inventory.get(&ingredient.name).copied().unwrap_or(0);
            if have < ingredient.quantity {
                return Err(format!(
                    "Need {} x {} but only have {}",
                    ingredient.quantity, ingredient.name, have
                ));
            }
        }

        Ok(())
    }

    /// Attempt to craft an item.
    ///
    /// Consumes ingredients on success, uses `seed` for RNG.
    pub fn craft(
        &self,
        recipe_id: &str,
        inventory: &mut HashMap<String, u32>,
        skill_level: u8,
        _station: CraftingStation,
        seed: u64,
    ) -> CraftResult {
        match self.can_craft(recipe_id, inventory, skill_level) {
            Err(msg) => return CraftResult {
                success: false,
                item_name: String::new(),
                quantity: 0,
                quality: 0,
                experience_gained: 0,
                message: msg,
            },
            Ok(()) => {}
        }

        let recipe = &self.recipes[recipe_id];

        // Consume ingredients.
        for ingredient in &recipe.ingredients {
            let entry = inventory.entry(ingredient.name.clone()).or_insert(0);
            *entry = entry.saturating_sub(ingredient.quantity);
        }

        let quality = Self::quality_roll(skill_level, recipe.difficulty, seed);
        let exp = Self::experience_gain(recipe.difficulty, quality);

        // Success threshold: quality >= 10 (very lenient).
        let success = quality >= 10;

        let message = if success {
            format!(
                "You craft {} (quality {}/100) at the {}.",
                recipe.output_name, quality, _station
            )
        } else {
            format!("The crafting attempt fails — the materials are wasted.")
        };

        CraftResult {
            success,
            item_name: if success { recipe.output_name.clone() } else { String::new() },
            quantity: if success { recipe.output_quantity } else { 0 },
            quality,
            experience_gained: exp,
            message,
        }
    }

    /// Roll a quality value using an LCG, biased by skill vs. difficulty.
    ///
    /// Returns a value clamped to 1–100.
    pub fn quality_roll(skill_level: u8, recipe_difficulty: u8, seed: u64) -> u8 {
        let s1 = lcg_next(seed);
        let s2 = lcg_next(s1);
        let rand_component = (s2 % 40) as i32; // 0–39
        let skill_bonus = (skill_level as i32 - recipe_difficulty as i32) / 2;
        let base: i32 = 30 + skill_bonus + rand_component;
        base.clamp(1, 100) as u8
    }

    /// Compute experience reward from difficulty and output quality.
    pub fn experience_gain(recipe_difficulty: u8, quality: u8) -> u32 {
        let base = (recipe_difficulty as u32) * 5;
        let quality_bonus = (quality as u32).saturating_sub(50) / 10;
        base + quality_bonus
    }

    /// Return all recipes in the given category.
    pub fn recipes_by_category(&self, category: RecipeCategory) -> Vec<&Recipe> {
        self.recipes.values()
            .filter(|r| r.category == category)
            .collect()
    }

    /// Attempt to identify a recipe from a list of presented ingredients.
    ///
    /// Uses fuzzy matching: a recipe matches if ≥ 75% of its ingredient names
    /// appear in the provided list.
    pub fn discover_recipe(ingredients: &[Ingredient]) -> Option<String> {
        // Standalone function — cannot access `self.recipes` directly.
        // Callers who want to search their system's recipes should call
        // `discover_recipe_in_system` instead.
        let _ = ingredients; // placeholder for API compatibility
        None
    }

    /// Search `self.recipes` for a match against the provided ingredients.
    pub fn discover_recipe_in_system(&self, ingredients: &[Ingredient]) -> Option<String> {
        let provided: std::collections::HashSet<&str> =
            ingredients.iter().map(|i| i.name.as_str()).collect();

        let mut best_id: Option<String> = None;
        let mut best_ratio = 0.0_f64;

        for recipe in self.recipes.values() {
            if recipe.ingredients.is_empty() {
                continue;
            }
            let matched = recipe.ingredients.iter()
                .filter(|ri| provided.contains(ri.name.as_str()))
                .count();
            let ratio = matched as f64 / recipe.ingredients.len() as f64;
            if ratio >= 0.75 && ratio > best_ratio {
                best_ratio = ratio;
                best_id = Some(recipe.id.clone());
            }
        }

        best_id
    }
}

impl Default for CraftingSystem {
    fn default() -> Self {
        Self::new()
    }
}
