//! Faction politics, diplomacy, and player reputation.
//!
//! Manages factions, diplomatic relations, and per-player standing.

use std::collections::HashMap;

// ── Diplomatic Relations ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DiplomaticRelation {
    Allied,
    Friendly,
    Neutral,
    Hostile,
    AtWar,
}

// ── Faction ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Faction {
    pub id: u32,
    pub name: String,
    pub power: f64,
    pub gold: u64,
    pub alignment: String,
}

// ── FactionRelation ───────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactionRelation {
    pub faction_a: u32,
    pub faction_b: u32,
    pub relation: DiplomaticRelation,
    pub score: i32,
}

// ── PlayerFactionStanding ─────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerFactionStanding {
    pub player_id: String,
    pub faction_id: u32,
    pub reputation: i32,
    pub title: String,
}

// ── Reputation Title ──────────────────────────────────────────────────────

pub fn reputation_title(rep: i32) -> &'static str {
    if rep >= 90 {
        "Exalted"
    } else if rep >= 60 {
        "Revered"
    } else if rep >= 30 {
        "Honored"
    } else if rep >= 10 {
        "Friendly"
    } else if rep >= 0 {
        "Neutral"
    } else if rep >= -10 {
        "Unfriendly"
    } else if rep >= -30 {
        "Hostile"
    } else {
        "Hated"
    }
}

// ── FactionSystem ─────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct FactionSystem {
    pub factions: HashMap<u32, Faction>,
    pub relations: Vec<FactionRelation>,
    pub standings: HashMap<(String, u32), PlayerFactionStanding>,
    pub next_id: u32,
}

impl FactionSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_faction(&mut self, name: &str, power: f64, alignment: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.factions.insert(
            id,
            Faction {
                id,
                name: name.to_string(),
                power,
                gold: 0,
                alignment: alignment.to_string(),
            },
        );
        id
    }

    pub fn set_relation(&mut self, a: u32, b: u32, relation: DiplomaticRelation, score: i32) {
        // Remove existing relation between a and b if present
        self.relations.retain(|r| {
            !((r.faction_a == a && r.faction_b == b)
                || (r.faction_a == b && r.faction_b == a))
        });
        self.relations.push(FactionRelation {
            faction_a: a,
            faction_b: b,
            relation,
            score: score.clamp(-100, 100),
        });
    }

    pub fn get_relation(&self, a: u32, b: u32) -> Option<&FactionRelation> {
        self.relations.iter().find(|r| {
            (r.faction_a == a && r.faction_b == b)
                || (r.faction_a == b && r.faction_b == a)
        })
    }

    pub fn modify_standing(&mut self, player_id: &str, faction_id: u32, delta: i32) {
        let key = (player_id.to_string(), faction_id);
        let entry = self.standings.entry(key).or_insert_with(|| PlayerFactionStanding {
            player_id: player_id.to_string(),
            faction_id,
            reputation: 0,
            title: reputation_title(0).to_string(),
        });
        entry.reputation = (entry.reputation + delta).clamp(-100, 100);
        entry.title = reputation_title(entry.reputation).to_string();
    }

    pub fn player_standing(&self, player_id: &str, faction_id: u32) -> Option<&PlayerFactionStanding> {
        self.standings.get(&(player_id.to_string(), faction_id))
    }

    pub fn allied_factions(&self, faction_id: u32) -> Vec<u32> {
        self.relations
            .iter()
            .filter(|r| r.relation == DiplomaticRelation::Allied)
            .filter_map(|r| {
                if r.faction_a == faction_id {
                    Some(r.faction_b)
                } else if r.faction_b == faction_id {
                    Some(r.faction_a)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn declare_war(&mut self, a: u32, b: u32) {
        // Set A-B to AtWar
        self.set_relation(a, b, DiplomaticRelation::AtWar, -100);

        // Allies of A become Hostile to allies of B
        let allies_a = self.allied_factions(a);
        let allies_b = self.allied_factions(b);

        for &ally_a in &allies_a {
            for &ally_b in &allies_b {
                if ally_a != ally_b {
                    self.set_relation(ally_a, ally_b, DiplomaticRelation::Hostile, -50);
                }
            }
        }
    }

    pub fn make_peace(&mut self, a: u32, b: u32) {
        self.set_relation(a, b, DiplomaticRelation::Neutral, 0);
    }

    pub fn faction_strength(&self, faction_id: u32) -> f64 {
        if let Some(faction) = self.factions.get(&faction_id) {
            let ally_count = self.allied_factions(faction_id).len() as f64;
            faction.power * (1.0 + ally_count * 0.1)
        } else {
            0.0
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_get_faction() {
        let mut sys = FactionSystem::new();
        let id = sys.add_faction("Merchant Guild", 10.0, "Neutral");
        assert_eq!(id, 0);
        assert!(sys.factions.contains_key(&id));
        assert_eq!(sys.factions[&id].name, "Merchant Guild");
    }

    #[test]
    fn set_and_get_relation() {
        let mut sys = FactionSystem::new();
        let a = sys.add_faction("A", 5.0, "Good");
        let b = sys.add_faction("B", 5.0, "Evil");
        sys.set_relation(a, b, DiplomaticRelation::Hostile, -40);
        let rel = sys.get_relation(a, b).unwrap();
        assert_eq!(rel.relation, DiplomaticRelation::Hostile);
        assert_eq!(rel.score, -40);
        // Symmetric
        let rel2 = sys.get_relation(b, a).unwrap();
        assert_eq!(rel2.relation, DiplomaticRelation::Hostile);
    }

    #[test]
    fn standing_modification() {
        let mut sys = FactionSystem::new();
        let fid = sys.add_faction("Artisans", 3.0, "Lawful");
        sys.modify_standing("player1", fid, 50);
        let standing = sys.player_standing("player1", fid).unwrap();
        assert_eq!(standing.reputation, 50);
        sys.modify_standing("player1", fid, 60); // clamp to 100
        let standing = sys.player_standing("player1", fid).unwrap();
        assert_eq!(standing.reputation, 100);
    }

    #[test]
    fn reputation_title_thresholds() {
        assert_eq!(reputation_title(95), "Exalted");
        assert_eq!(reputation_title(60), "Revered");
        assert_eq!(reputation_title(30), "Honored");
        assert_eq!(reputation_title(10), "Friendly");
        assert_eq!(reputation_title(0), "Neutral");
        assert_eq!(reputation_title(-5), "Unfriendly");
        assert_eq!(reputation_title(-20), "Hostile");
        assert_eq!(reputation_title(-50), "Hated");
    }

    #[test]
    fn declare_war_sets_at_war() {
        let mut sys = FactionSystem::new();
        let a = sys.add_faction("Kingdom", 20.0, "Good");
        let b = sys.add_faction("Empire", 18.0, "Neutral");
        sys.declare_war(a, b);
        let rel = sys.get_relation(a, b).unwrap();
        assert_eq!(rel.relation, DiplomaticRelation::AtWar);
        assert_eq!(rel.score, -100);
    }

    #[test]
    fn declare_war_propagates_to_allies() {
        let mut sys = FactionSystem::new();
        let a = sys.add_faction("Kingdom", 20.0, "Good");
        let b = sys.add_faction("Empire", 18.0, "Neutral");
        let ally_a = sys.add_faction("KingdomAlly", 5.0, "Good");
        let ally_b = sys.add_faction("EmpireAlly", 5.0, "Neutral");
        sys.set_relation(a, ally_a, DiplomaticRelation::Allied, 80);
        sys.set_relation(b, ally_b, DiplomaticRelation::Allied, 80);
        sys.declare_war(a, b);
        // Allies of A should be hostile to allies of B
        let rel = sys.get_relation(ally_a, ally_b).unwrap();
        assert_eq!(rel.relation, DiplomaticRelation::Hostile);
    }

    #[test]
    fn make_peace() {
        let mut sys = FactionSystem::new();
        let a = sys.add_faction("A", 10.0, "Good");
        let b = sys.add_faction("B", 10.0, "Evil");
        sys.declare_war(a, b);
        sys.make_peace(a, b);
        let rel = sys.get_relation(a, b).unwrap();
        assert_eq!(rel.relation, DiplomaticRelation::Neutral);
        assert_eq!(rel.score, 0);
    }

    #[test]
    fn faction_strength_with_allies() {
        let mut sys = FactionSystem::new();
        let a = sys.add_faction("A", 10.0, "Good");
        let b = sys.add_faction("B", 5.0, "Good");
        // Without allies
        let strength_a = sys.faction_strength(a);
        assert!((strength_a - 10.0).abs() < 1e-9);
        // With one ally
        sys.set_relation(a, b, DiplomaticRelation::Allied, 80);
        let strength_a_allied = sys.faction_strength(a);
        assert!((strength_a_allied - 11.0).abs() < 1e-9);
    }
}
