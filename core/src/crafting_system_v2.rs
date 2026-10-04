//! Advanced crafting with recipe discovery and ingredient substitution.

use std::collections::{HashMap, HashSet};

// ── IngredientQuality ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// Quality grade of a crafting ingredient.
pub enum IngredientQuality {
    /// Low quality; no bonus.
    Poor,
    /// Ordinary quality; no bonus.
    Normal,
    /// Good quality; +0.1 to crafting quality.
    Fine,
    /// Very good quality; +0.25 to crafting quality.
    Exceptional,
    /// Best quality; +0.5 to crafting quality.
    Masterwork,
}

impl IngredientQuality {
    /// Bonus this quality adds to the crafted item's quality score (0.0 to 0.5).
    pub fn quality_bonus(&self) -> f64 {
        match self {
            IngredientQuality::Poor => 0.0,
            IngredientQuality::Normal => 0.0,
            IngredientQuality::Fine => 0.1,
            IngredientQuality::Exceptional => 0.25,
            IngredientQuality::Masterwork => 0.5,
        }
    }
}

// ── Ingredient ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// One ingredient line of a recipe.
pub struct Ingredient {
    /// Item id of the ingredient.
    pub item_id: u32,
    /// Ingredient name.
    pub name: String,
    /// How many are needed.
    pub quantity: u32,
    /// Quality of the ingredient.
    pub quality: IngredientQuality,
}

// ── CraftingRecipe ────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// A crafting recipe.
pub struct CraftingRecipe {
    /// Recipe id.
    pub id: u32,
    /// Recipe name.
    pub name: String,
    /// Ingredients consumed.
    pub ingredients: Vec<Ingredient>,
    /// Item id of what gets made.
    pub output_item_id: u32,
    /// Name of what gets made.
    pub output_name: String,
    /// How many are made per craft.
    pub base_quantity: u32,
    /// Minimum crafting skill needed.
    pub skill_required: u32,
    /// Chance (0.0 to 1.0) of discovering the recipe when it is crafted or tried.
    pub discovery_chance: f64,
}

// ── SubstitutionRule ──────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Lets one item stand in for another in recipes.
pub struct SubstitutionRule {
    /// The item a recipe asks for.
    pub original_item_id: u32,
    /// The item that may be used instead.
    pub substitute_item_id: u32,
    /// Fraction (0.0 to 1.0) of the substitute's quality bonus that counts.
    pub efficiency: f64,
}

// ── CraftingResult ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
/// What a craft produced.
pub struct CraftingResult {
    /// True when the craft went through.
    pub success: bool,
    /// Item id of the output.
    pub output_item_id: u32,
    /// Name of the output.
    pub output_name: String,
    /// How many were made.
    pub quantity: u32,
    /// Average ingredient quality bonus plus 0.01 per skill point above the requirement.
    pub quality_score: f64,
    /// Recipe id if this craft discovered the recipe.
    pub discovered_recipe: Option<u32>,
}

// ── Inventory ─────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
/// Ingredients the crafter holds, by item id.
pub struct Inventory {
    /// id -> (name, quantity, quality)
    pub items: HashMap<u32, (String, u32, IngredientQuality)>,
}

impl Inventory {
    /// An empty inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `qty` of an item; a new stack keeps the given quality.
    pub fn add_item(&mut self, id: u32, name: &str, qty: u32, quality: IngredientQuality) {
        let entry = self.items.entry(id).or_insert((name.to_string(), 0, quality.clone()));
        entry.1 += qty;
    }

    /// True if at least `qty` of the item is held.
    pub fn has_item(&self, id: u32, qty: u32) -> bool {
        self.items.get(&id).map(|(_, q, _)| *q >= qty).unwrap_or(false)
    }

    /// Remove `qty` of an item; returns false (and removes nothing) if there are not enough.
    pub fn remove_item(&mut self, id: u32, qty: u32) -> bool {
        if let Some(entry) = self.items.get_mut(&id) {
            if entry.1 >= qty {
                entry.1 -= qty;
                return true;
            }
        }
        false
    }

    /// Quality of the item stack with this id (Normal if the item is not held).
    pub fn get_quality(&self, id: u32) -> IngredientQuality {
        self.items
            .get(&id)
            .map(|(_, _, q)| q.clone())
            .unwrap_or(IngredientQuality::Normal)
    }
}

// ── LCG RNG ───────────────────────────────────────────────────────────────

fn lcg_next(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*state >> 33) as f64 / (u32::MAX as f64)
}

// ── CraftingSystem ────────────────────────────────────────────────────────

#[derive(Debug)]
/// Recipe book plus the crafting rules and its own random state.
pub struct CraftingSystem {
    /// All recipes, by recipe id.
    pub recipes: HashMap<u32, CraftingRecipe>,
    /// Allowed ingredient substitutions.
    pub substitutions: Vec<SubstitutionRule>,
    /// Recipe ids the player has discovered.
    pub known_recipes: HashSet<u32>,
    /// State of the random generator used for discovery rolls.
    pub lcg_state: u64,
}

impl CraftingSystem {
    /// An empty crafting system whose rolls start from `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            recipes: HashMap::new(),
            substitutions: Vec::new(),
            known_recipes: HashSet::new(),
            lcg_state: seed,
        }
    }

    /// Add or replace a recipe (keyed by its id).
    pub fn add_recipe(&mut self, recipe: CraftingRecipe) {
        self.recipes.insert(recipe.id, recipe);
    }

    /// Add an ingredient substitution rule.
    pub fn add_substitution(&mut self, rule: SubstitutionRule) {
        self.substitutions.push(rule);
    }

    /// Roll to discover a recipe (chance is the recipe's `discovery_chance`); returns true if it was discovered.
    pub fn discover_recipe(&mut self, recipe_id: u32) -> bool {
        if let Some(recipe) = self.recipes.get(&recipe_id) {
            let roll = lcg_next(&mut self.lcg_state);
            if roll < recipe.discovery_chance {
                self.known_recipes.insert(recipe_id);
                return true;
            }
        }
        false
    }

    /// Substitution rules that can stand in for the given item.
    pub fn find_substitutes(&self, item_id: u32) -> Vec<&SubstitutionRule> {
        self.substitutions
            .iter()
            .filter(|r| r.original_item_id == item_id)
            .collect()
    }

    /// True if the inventory holds every ingredient, or an allowed substitute, in the needed quantity.
    pub fn can_craft(&self, recipe_id: u32, inventory: &Inventory) -> bool {
        let recipe = match self.recipes.get(&recipe_id) {
            Some(r) => r,
            None => return false,
        };

        for ingredient in &recipe.ingredients {
            let has_original = inventory.has_item(ingredient.item_id, ingredient.quantity);
            if !has_original {
                // Check substitutes
                let subs = self.find_substitutes(ingredient.item_id);
                let has_sub = subs.iter().any(|s| {
                    inventory.has_item(s.substitute_item_id, ingredient.quantity)
                });
                if !has_sub {
                    return false;
                }
            }
        }
        true
    }

    /// Craft the recipe: consume ingredients (or substitutes), add the output to the inventory and maybe discover the recipe; `None` if it cannot be crafted or the skill is too low.
    pub fn craft(
        &mut self,
        recipe_id: u32,
        inventory: &mut Inventory,
        player_skill: u32,
    ) -> Option<CraftingResult> {
        if !self.can_craft(recipe_id, inventory) {
            return None;
        }

        let recipe = self.recipes.get(&recipe_id)?.clone();

        if player_skill < recipe.skill_required {
            return None;
        }

        let mut quality_bonuses = Vec::new();
        let mut used_sub = false;

        for ingredient in &recipe.ingredients {
            let has_original = inventory.has_item(ingredient.item_id, ingredient.quantity);
            if has_original {
                let qual = inventory.get_quality(ingredient.item_id);
                quality_bonuses.push(qual.quality_bonus());
                inventory.remove_item(ingredient.item_id, ingredient.quantity);
            } else {
                // Use substitute
                let subs: Vec<SubstitutionRule> = self
                    .find_substitutes(ingredient.item_id)
                    .into_iter()
                    .cloned()
                    .collect();
                let sub = subs
                    .iter()
                    .find(|s| inventory.has_item(s.substitute_item_id, ingredient.quantity))?;
                let qual = inventory.get_quality(sub.substitute_item_id);
                quality_bonuses.push(qual.quality_bonus() * sub.efficiency);
                inventory.remove_item(sub.substitute_item_id, ingredient.quantity);
                used_sub = true;
            }
        }

        let _ = used_sub; // acknowledged

        let avg_quality = if quality_bonuses.is_empty() {
            0.0
        } else {
            quality_bonuses.iter().sum::<f64>() / quality_bonuses.len() as f64
        };

        let skill_bonus = (player_skill as f64 - recipe.skill_required as f64) * 0.01;
        let quality_score = avg_quality + skill_bonus.max(0.0);

        // Add output to inventory
        inventory.add_item(
            recipe.output_item_id,
            &recipe.output_name,
            recipe.base_quantity,
            IngredientQuality::Normal,
        );

        // Roll for recipe discovery on unknown recipes
        let was_unknown = !self.known_recipes.contains(&recipe_id);
        let discovered = if was_unknown {
            let roll = lcg_next(&mut self.lcg_state);
            if roll < recipe.discovery_chance {
                self.known_recipes.insert(recipe_id);
                Some(recipe_id)
            } else {
                None
            }
        } else {
            None
        };

        Some(CraftingResult {
            success: true,
            output_item_id: recipe.output_item_id,
            output_name: recipe.output_name.clone(),
            quantity: recipe.base_quantity,
            quality_score,
            discovered_recipe: discovered,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_system() -> (CraftingSystem, Inventory) {
        let mut sys = CraftingSystem::new(42);
        let recipe = CraftingRecipe {
            id: 1,
            name: "Iron Sword".to_string(),
            ingredients: vec![
                Ingredient {
                    item_id: 10,
                    name: "Iron Ore".to_string(),
                    quantity: 2,
                    quality: IngredientQuality::Normal,
                },
                Ingredient {
                    item_id: 11,
                    name: "Wood".to_string(),
                    quantity: 1,
                    quality: IngredientQuality::Normal,
                },
            ],
            output_item_id: 100,
            output_name: "Iron Sword".to_string(),
            base_quantity: 1,
            skill_required: 5,
            discovery_chance: 1.0,
        };
        sys.add_recipe(recipe);

        let mut inv = Inventory::new();
        inv.add_item(10, "Iron Ore", 3, IngredientQuality::Fine);
        inv.add_item(11, "Wood", 2, IngredientQuality::Normal);
        (sys, inv)
    }

    #[test]
    fn can_craft_with_exact_ingredients() {
        let (sys, inv) = make_system();
        assert!(sys.can_craft(1, &inv));
    }

    #[test]
    fn substitution_fallback() {
        let (mut sys, mut inv) = make_system();
        // Remove iron ore
        inv.remove_item(10, 3);
        // Add substitute
        inv.add_item(20, "Steel Ore", 2, IngredientQuality::Normal);
        sys.add_substitution(SubstitutionRule {
            original_item_id: 10,
            substitute_item_id: 20,
            efficiency: 0.8,
        });
        assert!(sys.can_craft(1, &inv));
        // Without substitute, should fail
        let (sys2, inv2) = make_system();
        let mut inv3 = inv2.clone();
        inv3.remove_item(10, 3);
        assert!(!sys2.can_craft(1, &inv3));
    }

    #[test]
    fn crafting_deducts_inventory() {
        let (mut sys, mut inv) = make_system();
        let ore_before = inv.items[&10].1;
        sys.craft(1, &mut inv, 10);
        let ore_after = inv.items.get(&10).map(|e| e.1).unwrap_or(0);
        assert_eq!(ore_before - ore_after, 2);
    }

    #[test]
    fn quality_score_calculation() {
        let (mut sys, mut inv) = make_system();
        // Iron Ore is Fine (0.1 bonus), Wood is Normal (0.0)
        let result = sys.craft(1, &mut inv, 5).unwrap();
        // avg = (0.1 + 0.0) / 2 = 0.05, skill_bonus = 0
        assert!((result.quality_score - 0.05).abs() < 1e-9);
    }

    #[test]
    fn recipe_discovery_roll() {
        let mut sys = CraftingSystem::new(42);
        let recipe = CraftingRecipe {
            id: 2,
            name: "Test Recipe".to_string(),
            ingredients: vec![],
            output_item_id: 200,
            output_name: "Test Item".to_string(),
            base_quantity: 1,
            skill_required: 0,
            discovery_chance: 1.0, // always discover
        };
        sys.add_recipe(recipe);
        // discover_recipe should work with 100% chance
        let discovered = sys.discover_recipe(2);
        assert!(discovered);
        assert!(sys.known_recipes.contains(&2));
    }
}
