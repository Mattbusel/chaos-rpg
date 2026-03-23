//! XP-gated skill tree with prerequisites, leveling, and effect aggregation.
//!
//! This module provides a separate skill tree that uses XP costs and string-based
//! prerequisite references, distinct from the passive tree in `skill_tree.rs`.

use std::collections::HashMap;
use thiserror::Error;
use serde::{Deserialize, Serialize};

// ─── ERRORS ───────────────────────────────────────────────────────────────────

#[derive(Debug, Error, Clone, PartialEq)]
pub enum SkillError {
    #[error("skill '{0}' not found")]
    NotFound(String),
    #[error("skill '{0}' is already unlocked")]
    AlreadyUnlocked(String),
    #[error("insufficient XP: need {need}, have {have}")]
    InsufficientXp { need: u32, have: u32 },
    #[error("prerequisites not met for skill '{0}'")]
    PrerequisitesNotMet(String),
    #[error("skill '{0}' is already at max level")]
    AlreadyMaxLevel(String),
    #[error("skill '{0}' is not unlocked")]
    NotUnlocked(String),
}

// ─── SKILL EFFECT ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkillEffect {
    /// Flat additive bonus to damage (multiplicative factor, e.g. 0.1 = +10%).
    DamageBonus(f64),
    /// Flat additive bonus to defense.
    DefenseBonus(f64),
    /// Movement speed multiplier bonus.
    SpeedBonus(f64),
    /// Unlocks a named ability string (ability id).
    UnlockAbility(String),
    /// Multiplier applied to a named resource (e.g. "gold", "mana").
    ResourceMultiplier { resource: String, mult: f64 },
}

// ─── SKILL ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub xp_cost: u32,
    pub max_level: u8,
    pub prerequisites: Vec<String>,
}

// ─── SKILL NODE ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillNode {
    pub skill: Skill,
    pub current_level: u8,
    pub unlocked: bool,
    pub effects: Vec<SkillEffect>,
}

// ─── SKILL TREE ───────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SkillTree {
    nodes: HashMap<String, SkillNode>,
}

impl SkillTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a skill with its associated effects to the tree.
    pub fn add_skill(&mut self, skill: Skill, effects: Vec<SkillEffect>) {
        let id = skill.id.clone();
        let node = SkillNode {
            skill,
            current_level: 0,
            unlocked: false,
            effects,
        };
        self.nodes.insert(id, node);
    }

    /// Check whether a skill can be unlocked given current XP and the set of already-unlocked skill IDs.
    pub fn can_unlock(
        &self,
        skill_id: &str,
        player_xp: u32,
        unlocked_skills: &[String],
    ) -> bool {
        let Some(node) = self.nodes.get(skill_id) else {
            return false;
        };
        if node.unlocked {
            return false;
        }
        if player_xp < node.skill.xp_cost {
            return false;
        }
        for prereq in &node.skill.prerequisites {
            if !unlocked_skills.contains(prereq) {
                return false;
            }
        }
        true
    }

    /// Unlock a skill, deducting XP cost.  Returns remaining XP on success.
    pub fn unlock(
        &mut self,
        skill_id: &str,
        player_xp: u32,
        unlocked: &[String],
    ) -> Result<u32, SkillError> {
        let node = self
            .nodes
            .get(skill_id)
            .ok_or_else(|| SkillError::NotFound(skill_id.to_owned()))?;

        if node.unlocked {
            return Err(SkillError::AlreadyUnlocked(skill_id.to_owned()));
        }
        if player_xp < node.skill.xp_cost {
            return Err(SkillError::InsufficientXp {
                need: node.skill.xp_cost,
                have: player_xp,
            });
        }
        for prereq in &node.skill.prerequisites {
            if !unlocked.contains(prereq) {
                return Err(SkillError::PrerequisitesNotMet(skill_id.to_owned()));
            }
        }

        let cost = node.skill.xp_cost;
        let node_mut = self.nodes.get_mut(skill_id).unwrap();
        node_mut.unlocked = true;
        node_mut.current_level = 1;
        Ok(player_xp - cost)
    }

    /// Increase the level of an already-unlocked skill by 1, up to max_level.
    pub fn upgrade(
        &mut self,
        skill_id: &str,
        unlocked: &[String],
    ) -> Result<(), SkillError> {
        let node = self
            .nodes
            .get(skill_id)
            .ok_or_else(|| SkillError::NotFound(skill_id.to_owned()))?;

        if !node.unlocked || !unlocked.contains(&skill_id.to_owned()) {
            return Err(SkillError::NotUnlocked(skill_id.to_owned()));
        }
        if node.current_level >= node.skill.max_level {
            return Err(SkillError::AlreadyMaxLevel(skill_id.to_owned()));
        }

        let node_mut = self.nodes.get_mut(skill_id).unwrap();
        node_mut.current_level += 1;
        Ok(())
    }

    /// Aggregate all effects from unlocked skills.
    pub fn active_effects(&self, unlocked: &[String]) -> Vec<SkillEffect> {
        let mut effects = Vec::new();
        for id in unlocked {
            if let Some(node) = self.nodes.get(id.as_str()) {
                if node.unlocked {
                    // Repeat effects for each level above 1.
                    for _ in 0..node.current_level.max(1) {
                        effects.extend(node.effects.clone());
                    }
                }
            }
        }
        effects
    }

    /// Return all skill nodes whose prerequisites are satisfied.
    pub fn available_skills(&self, unlocked: &[String]) -> Vec<&SkillNode> {
        self.nodes
            .values()
            .filter(|node| {
                node.skill
                    .prerequisites
                    .iter()
                    .all(|p| unlocked.contains(p))
            })
            .collect()
    }

    /// Get a reference to a node by id.
    pub fn get(&self, skill_id: &str) -> Option<&SkillNode> {
        self.nodes.get(skill_id)
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_basic_skill(id: &str, cost: u32) -> Skill {
        Skill {
            id: id.to_owned(),
            name: format!("Skill {id}"),
            description: "A test skill".to_owned(),
            xp_cost: cost,
            max_level: 3,
            prerequisites: vec![],
        }
    }

    fn make_skill_with_prereqs(id: &str, cost: u32, prereqs: Vec<&str>) -> Skill {
        Skill {
            id: id.to_owned(),
            name: format!("Skill {id}"),
            description: "A test skill with prereqs".to_owned(),
            xp_cost: cost,
            max_level: 5,
            prerequisites: prereqs.into_iter().map(|s| s.to_owned()).collect(),
        }
    }

    #[test]
    fn test_add_and_get_skill() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![SkillEffect::DamageBonus(0.1)]);
        let node = tree.get("fire").unwrap();
        assert_eq!(node.skill.id, "fire");
        assert!(!node.unlocked);
        assert_eq!(node.current_level, 0);
    }

    #[test]
    fn test_can_unlock_sufficient_xp() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        assert!(tree.can_unlock("fire", 200, &[]));
    }

    #[test]
    fn test_can_unlock_insufficient_xp() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        assert!(!tree.can_unlock("fire", 50, &[]));
    }

    #[test]
    fn test_can_unlock_missing_prereqs() {
        let mut tree = SkillTree::new();
        tree.add_skill(
            make_skill_with_prereqs("ice", 100, vec!["fire"]),
            vec![],
        );
        assert!(!tree.can_unlock("ice", 500, &[]));
    }

    #[test]
    fn test_can_unlock_with_prereqs_met() {
        let mut tree = SkillTree::new();
        tree.add_skill(
            make_skill_with_prereqs("ice", 100, vec!["fire"]),
            vec![],
        );
        let unlocked = vec!["fire".to_owned()];
        assert!(tree.can_unlock("ice", 500, &unlocked));
    }

    #[test]
    fn test_unlock_returns_remaining_xp() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        let remaining = tree.unlock("fire", 500, &[]).unwrap();
        assert_eq!(remaining, 400);
    }

    #[test]
    fn test_unlock_already_unlocked_error() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        tree.unlock("fire", 500, &[]).unwrap();
        let err = tree.unlock("fire", 500, &[]).unwrap_err();
        assert!(matches!(err, SkillError::AlreadyUnlocked(_)));
    }

    #[test]
    fn test_unlock_insufficient_xp_error() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        let err = tree.unlock("fire", 50, &[]).unwrap_err();
        assert!(matches!(err, SkillError::InsufficientXp { .. }));
    }

    #[test]
    fn test_unlock_prerequisites_not_met_error() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_skill_with_prereqs("ice", 100, vec!["fire"]), vec![]);
        let err = tree.unlock("ice", 500, &[]).unwrap_err();
        assert!(matches!(err, SkillError::PrerequisitesNotMet(_)));
    }

    #[test]
    fn test_upgrade_increases_level() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        tree.unlock("fire", 500, &[]).unwrap();
        let unlocked = vec!["fire".to_owned()];
        tree.upgrade("fire", &unlocked).unwrap();
        assert_eq!(tree.get("fire").unwrap().current_level, 2);
    }

    #[test]
    fn test_upgrade_at_max_level_error() {
        let mut tree = SkillTree::new();
        let mut skill = make_basic_skill("fire", 100);
        skill.max_level = 1;
        tree.add_skill(skill, vec![]);
        tree.unlock("fire", 500, &[]).unwrap();
        let unlocked = vec!["fire".to_owned()];
        let err = tree.upgrade("fire", &unlocked).unwrap_err();
        assert!(matches!(err, SkillError::AlreadyMaxLevel(_)));
    }

    #[test]
    fn test_active_effects_aggregation() {
        let mut tree = SkillTree::new();
        tree.add_skill(
            make_basic_skill("fire", 100),
            vec![SkillEffect::DamageBonus(0.15)],
        );
        tree.add_skill(
            make_basic_skill("shield", 80),
            vec![SkillEffect::DefenseBonus(5.0)],
        );
        tree.unlock("fire", 500, &[]).unwrap();
        tree.unlock("shield", 500, &[]).unwrap();
        let unlocked = vec!["fire".to_owned(), "shield".to_owned()];
        let effects = tree.active_effects(&unlocked);
        assert!(effects.iter().any(|e| matches!(e, SkillEffect::DamageBonus(_))));
        assert!(effects.iter().any(|e| matches!(e, SkillEffect::DefenseBonus(_))));
    }

    #[test]
    fn test_available_skills_no_prereqs() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        tree.add_skill(make_basic_skill("ice", 100), vec![]);
        let available = tree.available_skills(&[]);
        assert_eq!(available.len(), 2);
    }

    #[test]
    fn test_available_skills_with_prereqs_not_met() {
        let mut tree = SkillTree::new();
        tree.add_skill(make_basic_skill("fire", 100), vec![]);
        tree.add_skill(make_skill_with_prereqs("inferno", 200, vec!["fire"]), vec![]);
        let available = tree.available_skills(&[]);
        // Only fire is available (inferno requires fire).
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].skill.id, "fire");
    }

    #[test]
    fn test_unlock_ability_effect() {
        let mut tree = SkillTree::new();
        tree.add_skill(
            make_basic_skill("arcane", 150),
            vec![SkillEffect::UnlockAbility("arcane_blast".to_owned())],
        );
        tree.unlock("arcane", 500, &[]).unwrap();
        let unlocked = vec!["arcane".to_owned()];
        let effects = tree.active_effects(&unlocked);
        assert!(effects
            .iter()
            .any(|e| matches!(e, SkillEffect::UnlockAbility(a) if a == "arcane_blast")));
    }

    #[test]
    fn test_resource_multiplier_effect() {
        let mut tree = SkillTree::new();
        tree.add_skill(
            make_basic_skill("merchant", 200),
            vec![SkillEffect::ResourceMultiplier {
                resource: "gold".to_owned(),
                mult: 1.25,
            }],
        );
        tree.unlock("merchant", 500, &[]).unwrap();
        let unlocked = vec!["merchant".to_owned()];
        let effects = tree.active_effects(&unlocked);
        assert!(effects.iter().any(|e| matches!(
            e,
            SkillEffect::ResourceMultiplier { resource, mult }
            if resource == "gold" && (*mult - 1.25).abs() < 1e-9
        )));
    }

    #[test]
    fn test_unlock_not_found_error() {
        let mut tree = SkillTree::new();
        let err = tree.unlock("ghost", 500, &[]).unwrap_err();
        assert!(matches!(err, SkillError::NotFound(_)));
    }
}
