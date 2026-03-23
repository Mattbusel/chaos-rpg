//! Faction reputation tracking with events and consequences.

use std::collections::HashMap;

/// Moral and ethical alignment of a faction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alignment {
    LawfulGood,
    NeutralGood,
    ChaoticGood,
    LawfulNeutral,
    TrueNeutral,
    ChaoticNeutral,
    LawfulEvil,
    NeutralEvil,
    ChaoticEvil,
}

impl Alignment {
    /// Returns the broad axis: "Good", "Neutral", or "Evil".
    pub fn moral_axis(&self) -> &'static str {
        match self {
            Alignment::LawfulGood | Alignment::NeutralGood | Alignment::ChaoticGood => "Good",
            Alignment::LawfulNeutral | Alignment::TrueNeutral | Alignment::ChaoticNeutral => "Neutral",
            Alignment::LawfulEvil | Alignment::NeutralEvil | Alignment::ChaoticEvil => "Evil",
        }
    }

    /// Returns true if the two alignments are broadly opposing.
    pub fn opposes(&self, other: &Alignment) -> bool {
        let a = self.moral_axis();
        let b = other.moral_axis();
        matches!((a, b), ("Good", "Evil") | ("Evil", "Good"))
    }

    /// Returns true if the two alignments share the same broad moral axis.
    pub fn allies_with(&self, other: &Alignment) -> bool {
        self.moral_axis() == other.moral_axis()
    }
}

/// A faction in the game world.
#[derive(Debug, Clone)]
pub struct Faction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub alignment: Alignment,
}

/// Standing level derived from numeric reputation score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReputationLevel {
    /// Score > 800
    Exalted,
    /// 600 ..= 800
    Revered,
    /// 400 ..= 599
    Honored,
    /// 200 ..= 399
    Friendly,
    /// 0 ..= 199
    Neutral,
    /// -199 ..= -1
    Unfriendly,
    /// -399 ..= -200
    Hostile,
    /// < -400
    Hated,
}

impl ReputationLevel {
    /// Derive the level from a raw numeric score.
    pub fn from_score(score: i32) -> Self {
        match score {
            s if s > 800 => ReputationLevel::Exalted,
            s if s >= 600 => ReputationLevel::Revered,
            s if s >= 400 => ReputationLevel::Honored,
            s if s >= 200 => ReputationLevel::Friendly,
            s if s >= 0 => ReputationLevel::Neutral,
            s if s >= -199 => ReputationLevel::Unfriendly,
            s if s >= -400 => ReputationLevel::Hostile,
            _ => ReputationLevel::Hated,
        }
    }
}

/// A single reputation-altering event.
#[derive(Debug, Clone)]
pub struct ReputationEvent {
    pub event_id: String,
    pub faction_id: String,
    /// Positive = reputation gain, negative = reputation loss.
    pub delta: i32,
    pub reason: String,
    pub timestamp: u64,
}

/// Central reputation tracker.
#[derive(Debug, Default)]
pub struct ReputationSystem {
    factions: HashMap<String, Faction>,
    events: Vec<ReputationEvent>,
}

impl ReputationSystem {
    /// Create a new, empty reputation system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a faction.
    pub fn add_faction(&mut self, faction: Faction) {
        self.factions.insert(faction.id.clone(), faction);
    }

    /// Record a reputation-changing event.
    pub fn record_event(&mut self, event: ReputationEvent) {
        self.events.push(event);
    }

    /// Compute the current reputation score for a faction (sum of all deltas).
    pub fn reputation(&self, faction_id: &str) -> i32 {
        self.events
            .iter()
            .filter(|e| e.faction_id == faction_id)
            .map(|e| e.delta)
            .sum()
    }

    /// Derive the reputation level for a faction.
    pub fn level(&self, faction_id: &str) -> ReputationLevel {
        ReputationLevel::from_score(self.reputation(faction_id))
    }

    /// Return a list of reward descriptions available at the current reputation level.
    pub fn available_rewards(&self, faction_id: &str) -> Vec<String> {
        let level = self.level(faction_id);
        let name = self
            .factions
            .get(faction_id)
            .map(|f| f.name.as_str())
            .unwrap_or(faction_id);

        match level {
            ReputationLevel::Exalted => vec![
                format!("{}: Champion title", name),
                format!("{}: Legendary weapon discount (50%)", name),
                format!("{}: Access to secret vault", name),
                format!("{}: Personal escort service", name),
            ],
            ReputationLevel::Revered => vec![
                format!("{}: Elite armor discount (30%)", name),
                format!("{}: Exclusive questline unlocked", name),
                format!("{}: Guild tabard", name),
            ],
            ReputationLevel::Honored => vec![
                format!("{}: Access to special shop", name),
                format!("{}: 10% discount on services", name),
            ],
            ReputationLevel::Friendly => vec![
                format!("{}: Warm welcome at faction halls", name),
                format!("{}: Basic quests available", name),
            ],
            ReputationLevel::Neutral => vec![
                format!("{}: Standard trading permitted", name),
            ],
            ReputationLevel::Unfriendly => vec![
                format!("{}: Restricted access — improve reputation", name),
            ],
            ReputationLevel::Hostile => vec![],
            ReputationLevel::Hated => vec![],
        }
    }

    /// Return factions that share the same broad alignment as the given faction.
    pub fn allied_factions(&self, faction_id: &str) -> Vec<&Faction> {
        let alignment = match self.factions.get(faction_id) {
            Some(f) => &f.alignment,
            None => return vec![],
        };
        self.factions
            .values()
            .filter(|f| f.id != faction_id && f.alignment.allies_with(alignment))
            .collect()
    }

    /// Return factions with opposing alignments.
    pub fn enemy_factions(&self, faction_id: &str) -> Vec<&Faction> {
        let alignment = match self.factions.get(faction_id) {
            Some(f) => &f.alignment,
            None => return vec![],
        };
        self.factions
            .values()
            .filter(|f| f.id != faction_id && f.alignment.opposes(alignment))
            .collect()
    }

    /// Return the last `last_n` events for a faction (oldest first within the slice).
    pub fn history<'a>(&'a self, faction_id: &str, last_n: usize) -> Vec<&'a ReputationEvent> {
        let events: Vec<&'a ReputationEvent> = self
            .events
            .iter()
            .filter(|e| e.faction_id == faction_id)
            .collect();
        let skip = events.len().saturating_sub(last_n);
        events.into_iter().skip(skip).collect()
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_faction(id: &str, name: &str, alignment: Alignment) -> Faction {
        Faction {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{} faction", name),
            alignment,
        }
    }

    fn make_event(faction_id: &str, delta: i32) -> ReputationEvent {
        ReputationEvent {
            event_id: uuid_str(),
            faction_id: faction_id.to_string(),
            delta,
            reason: "test".to_string(),
            timestamp: 0,
        }
    }

    fn uuid_str() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
            .to_string()
    }

    #[test]
    fn test_empty_reputation_is_zero() {
        let sys = ReputationSystem::new();
        assert_eq!(sys.reputation("unknown"), 0);
    }

    #[test]
    fn test_single_positive_event() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "Knights", Alignment::LawfulGood));
        sys.record_event(ReputationEvent {
            event_id: "e1".to_string(),
            faction_id: "f1".to_string(),
            delta: 300,
            reason: "Helped villagers".to_string(),
            timestamp: 100,
        });
        assert_eq!(sys.reputation("f1"), 300);
    }

    #[test]
    fn test_multiple_events_sum() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "Knights", Alignment::LawfulGood));
        sys.record_event(make_event("f1", 400));
        sys.record_event(make_event("f1", -100));
        sys.record_event(make_event("f1", 50));
        assert_eq!(sys.reputation("f1"), 350);
    }

    #[test]
    fn test_reputation_level_exalted() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::TrueNeutral));
        sys.record_event(make_event("f1", 850));
        assert_eq!(sys.level("f1"), ReputationLevel::Exalted);
    }

    #[test]
    fn test_reputation_level_revered() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::TrueNeutral));
        sys.record_event(make_event("f1", 700));
        assert_eq!(sys.level("f1"), ReputationLevel::Revered);
    }

    #[test]
    fn test_reputation_level_honored() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::TrueNeutral));
        sys.record_event(make_event("f1", 450));
        assert_eq!(sys.level("f1"), ReputationLevel::Honored);
    }

    #[test]
    fn test_reputation_level_hostile() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::TrueNeutral));
        sys.record_event(make_event("f1", -350));
        assert_eq!(sys.level("f1"), ReputationLevel::Hostile);
    }

    #[test]
    fn test_reputation_level_hated() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::TrueNeutral));
        sys.record_event(make_event("f1", -500));
        assert_eq!(sys.level("f1"), ReputationLevel::Hated);
    }

    #[test]
    fn test_available_rewards_honored() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "Merchants", Alignment::LawfulNeutral));
        sys.record_event(make_event("f1", 450));
        let rewards = sys.available_rewards("f1");
        assert!(rewards.iter().any(|r| r.contains("special shop")));
    }

    #[test]
    fn test_available_rewards_hostile_is_empty() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "Bandits", Alignment::ChaoticEvil));
        sys.record_event(make_event("f1", -300));
        let rewards = sys.available_rewards("f1");
        assert!(rewards.is_empty());
    }

    #[test]
    fn test_allied_factions_same_axis() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("a", "Paladins", Alignment::LawfulGood));
        sys.add_faction(make_faction("b", "Rangers", Alignment::NeutralGood));
        sys.add_faction(make_faction("c", "Rogues", Alignment::ChaoticNeutral));

        let allies = sys.allied_factions("a");
        assert!(allies.iter().any(|f| f.id == "b"));
        assert!(!allies.iter().any(|f| f.id == "c"));
    }

    #[test]
    fn test_enemy_factions_opposing_axis() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("a", "Paladins", Alignment::LawfulGood));
        sys.add_faction(make_faction("b", "Demons", Alignment::ChaoticEvil));
        sys.add_faction(make_faction("c", "Merchants", Alignment::TrueNeutral));

        let enemies = sys.enemy_factions("a");
        assert!(enemies.iter().any(|f| f.id == "b"));
        assert!(!enemies.iter().any(|f| f.id == "c"));
    }

    #[test]
    fn test_history_last_n() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::TrueNeutral));
        for i in 0..5_i32 {
            sys.record_event(ReputationEvent {
                event_id: format!("e{}", i),
                faction_id: "f1".to_string(),
                delta: i * 10,
                reason: format!("event {}", i),
                timestamp: i as u64,
            });
        }
        let hist = sys.history("f1", 3);
        assert_eq!(hist.len(), 3);
        assert_eq!(hist[0].delta, 20);
        assert_eq!(hist[2].delta, 40);
    }

    #[test]
    fn test_history_fewer_events_than_n() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::TrueNeutral));
        sys.record_event(make_event("f1", 100));
        let hist = sys.history("f1", 10);
        assert_eq!(hist.len(), 1);
    }

    #[test]
    fn test_events_isolated_per_faction() {
        let mut sys = ReputationSystem::new();
        sys.add_faction(make_faction("f1", "A", Alignment::LawfulGood));
        sys.add_faction(make_faction("f2", "B", Alignment::LawfulEvil));
        sys.record_event(make_event("f1", 500));
        sys.record_event(make_event("f2", -200));
        assert_eq!(sys.reputation("f1"), 500);
        assert_eq!(sys.reputation("f2"), -200);
    }
}
