//! Faction reputation system.
//!
//! Manages multiple named factions, their alliances and enmities, and the
//! player's standing with each one.  Reputation cascades automatically to
//! allied and enemy factions when modified.

use std::collections::HashMap;

// ─── FACTION ──────────────────────────────────────────────────────────────────

/// A playable or NPC faction with social relationships to other factions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Faction {
    /// Unique machine-readable identifier.
    pub id: String,
    /// Display name shown to the player.
    pub name: String,
    /// Flavour text describing the faction.
    pub description: String,
    /// Ids of factions that share a positive relationship with this one.
    pub allied_factions: Vec<String>,
    /// Ids of factions that are at odds with this one.
    pub enemy_factions: Vec<String>,
}

// ─── REPUTATION LEVEL ─────────────────────────────────────────────────────────

/// Qualitative standing derived from a numeric reputation score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum ReputationLevel {
    /// Score in [-100, -50).
    Hostile,
    /// Score in [-50, 0).
    Unfriendly,
    /// Score in [0, 25).
    Neutral,
    /// Score in [25, 50).
    Friendly,
    /// Score in [50, 75).
    Honored,
    /// Score in [75, 90).
    Revered,
    /// Score in [90, 100].
    Exalted,
}

impl ReputationLevel {
    /// Map a numeric score to a reputation level.
    pub fn from_score(score: i32) -> Self {
        match score {
            i32::MIN..=-50 => ReputationLevel::Hostile,
            -49..=-1 => ReputationLevel::Unfriendly,
            0..=24 => ReputationLevel::Neutral,
            25..=49 => ReputationLevel::Friendly,
            50..=74 => ReputationLevel::Honored,
            75..=89 => ReputationLevel::Revered,
            _ => ReputationLevel::Exalted,
        }
    }

    /// Human-readable title for this reputation tier.
    pub fn title(&self) -> &str {
        match self {
            ReputationLevel::Hostile => "Hostile",
            ReputationLevel::Unfriendly => "Unfriendly",
            ReputationLevel::Neutral => "Neutral",
            ReputationLevel::Friendly => "Friendly",
            ReputationLevel::Honored => "Honored",
            ReputationLevel::Revered => "Revered",
            ReputationLevel::Exalted => "Exalted",
        }
    }
}

// ─── FACTION REGISTRY ─────────────────────────────────────────────────────────

/// Central store for all factions and the player's reputation with each.
#[derive(Debug, Default)]
pub struct FactionRegistry {
    /// All registered factions keyed by id.
    pub factions: HashMap<String, Faction>,
    /// Current reputation scores keyed by faction id.
    pub reputations: HashMap<String, i32>,
}

impl FactionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a faction.  Initialises its reputation to 0 if not already set.
    pub fn add_faction(&mut self, faction: Faction) {
        self.reputations.entry(faction.id.clone()).or_insert(0);
        self.factions.insert(faction.id.clone(), faction);
    }

    /// Return the current reputation score for a faction (0 if unknown).
    pub fn get_reputation(&self, faction_id: &str) -> i32 {
        self.reputations.get(faction_id).copied().unwrap_or(0)
    }

    /// Modify reputation by `delta`, clamping to [-100, 100].
    ///
    /// Cascade rules:
    /// * Allied factions gain **half** the delta (rounded toward zero).
    /// * Enemy factions lose **half** the delta (rounded toward zero).
    pub fn modify_reputation(&mut self, faction_id: &str, delta: i32) {
        // Apply to primary faction.
        let primary = self.reputations.entry(faction_id.to_owned()).or_insert(0);
        *primary = (*primary + delta).clamp(-100, 100);

        // Collect cascade targets to avoid borrow issues.
        let (allied, enemy) = if let Some(f) = self.factions.get(faction_id) {
            (f.allied_factions.clone(), f.enemy_factions.clone())
        } else {
            return;
        };

        let half = delta / 2; // integer truncation toward zero

        for ally_id in allied {
            let ally = self.reputations.entry(ally_id).or_insert(0);
            *ally = (*ally + half).clamp(-100, 100);
        }

        for enemy_id in enemy {
            let enemy = self.reputations.entry(enemy_id).or_insert(0);
            *enemy = (*enemy - half).clamp(-100, 100);
        }
    }

    /// Return the reputation level for a faction.
    pub fn reputation_level(&self, faction_id: &str) -> ReputationLevel {
        ReputationLevel::from_score(self.get_reputation(faction_id))
    }

    /// Return the discount fraction available from a faction's vendors.
    ///
    /// | Level    | Discount |
    /// |----------|----------|
    /// | Friendly | 5 %      |
    /// | Honored  | 10 %     |
    /// | Revered  | 15 %     |
    /// | Exalted  | 20 %     |
    /// | below    | 0 %      |
    pub fn available_discounts(&self, faction_id: &str) -> f64 {
        match self.reputation_level(faction_id) {
            ReputationLevel::Friendly => 0.05,
            ReputationLevel::Honored => 0.10,
            ReputationLevel::Revered => 0.15,
            ReputationLevel::Exalted => 0.20,
            _ => 0.0,
        }
    }

    /// Return the ids of all factions with which the player has at least
    /// Friendly standing (unlocked vendors).
    pub fn unlocked_vendors(&self) -> Vec<String> {
        self.factions
            .keys()
            .filter(|id| {
                self.get_reputation(id) >= 25 // Friendly threshold
            })
            .cloned()
            .collect()
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry() -> FactionRegistry {
        let mut reg = FactionRegistry::new();

        reg.add_faction(Faction {
            id: "merchants".into(),
            name: "Merchant Guild".into(),
            description: "Traders of the realm.".into(),
            allied_factions: vec!["artisans".into()],
            enemy_factions: vec!["thieves".into()],
        });
        reg.add_faction(Faction {
            id: "artisans".into(),
            name: "Artisan League".into(),
            description: "Crafters united.".into(),
            allied_factions: vec![],
            enemy_factions: vec![],
        });
        reg.add_faction(Faction {
            id: "thieves".into(),
            name: "Thieves Guild".into(),
            description: "Shadows of the city.".into(),
            allied_factions: vec![],
            enemy_factions: vec![],
        });

        reg
    }

    #[test]
    fn initial_reputation_is_zero() {
        let reg = make_registry();
        assert_eq!(reg.get_reputation("merchants"), 0);
        assert_eq!(reg.get_reputation("artisans"), 0);
    }

    #[test]
    fn cascade_to_allied_faction() {
        let mut reg = make_registry();
        reg.modify_reputation("merchants", 20);
        // merchants: 20, artisans gets half = 10
        assert_eq!(reg.get_reputation("merchants"), 20);
        assert_eq!(reg.get_reputation("artisans"), 10);
    }

    #[test]
    fn cascade_to_enemy_faction() {
        let mut reg = make_registry();
        reg.modify_reputation("merchants", 20);
        // thieves loses half = -10
        assert_eq!(reg.get_reputation("thieves"), -10);
    }

    #[test]
    fn clamp_at_positive_100() {
        let mut reg = make_registry();
        reg.modify_reputation("merchants", 80);
        reg.modify_reputation("merchants", 80);
        assert_eq!(reg.get_reputation("merchants"), 100);
    }

    #[test]
    fn clamp_at_negative_100() {
        let mut reg = make_registry();
        reg.modify_reputation("merchants", -80);
        reg.modify_reputation("merchants", -80);
        assert_eq!(reg.get_reputation("merchants"), -100);
    }

    #[test]
    fn reputation_level_mapping() {
        assert_eq!(ReputationLevel::from_score(-75), ReputationLevel::Hostile);
        assert_eq!(ReputationLevel::from_score(-25), ReputationLevel::Unfriendly);
        assert_eq!(ReputationLevel::from_score(10), ReputationLevel::Neutral);
        assert_eq!(ReputationLevel::from_score(30), ReputationLevel::Friendly);
        assert_eq!(ReputationLevel::from_score(60), ReputationLevel::Honored);
        assert_eq!(ReputationLevel::from_score(80), ReputationLevel::Revered);
        assert_eq!(ReputationLevel::from_score(95), ReputationLevel::Exalted);
    }

    #[test]
    fn discount_tiers() {
        let mut reg = make_registry();
        // Below Friendly: no discount
        assert_eq!(reg.available_discounts("merchants"), 0.0);

        reg.modify_reputation("merchants", 30); // Friendly
        assert!((reg.available_discounts("merchants") - 0.05).abs() < f64::EPSILON);

        reg.modify_reputation("merchants", 25); // Honored (55 total)
        assert!((reg.available_discounts("merchants") - 0.10).abs() < f64::EPSILON);

        reg.modify_reputation("merchants", 25); // Revered (80 total)
        assert!((reg.available_discounts("merchants") - 0.15).abs() < f64::EPSILON);

        reg.modify_reputation("merchants", 15); // Exalted (95 total)
        assert!((reg.available_discounts("merchants") - 0.20).abs() < f64::EPSILON);
    }

    #[test]
    fn unlocked_vendors_returns_friendly_and_above() {
        let mut reg = make_registry();
        assert!(reg.unlocked_vendors().is_empty());

        reg.modify_reputation("artisans", 30); // Friendly
        let vendors = reg.unlocked_vendors();
        assert!(vendors.contains(&"artisans".to_string()));
        assert!(!vendors.contains(&"merchants".to_string()));
    }

    #[test]
    fn reputation_level_title() {
        assert_eq!(ReputationLevel::Hostile.title(), "Hostile");
        assert_eq!(ReputationLevel::Exalted.title(), "Exalted");
    }

    #[test]
    fn negative_delta_cascades_correctly() {
        let mut reg = make_registry();
        reg.modify_reputation("merchants", -20);
        // allied artisans lose half = -10
        assert_eq!(reg.get_reputation("artisans"), -10);
        // enemy thieves gain half = +10
        assert_eq!(reg.get_reputation("thieves"), 10);
    }
}
