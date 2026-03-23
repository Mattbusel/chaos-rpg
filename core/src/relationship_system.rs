//! NPC relationship tracking: trust, affection, and interaction history.

use std::collections::HashMap;

/// Classification of the relationship between two entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipType {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    Ally,
    Rival,
    Enemy,
    Romantic,
    Family,
}

/// The type of interaction that occurred between two entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionType {
    Greeting,
    Gift,
    Trade,
    QuestComplete,
    Betrayal,
    Combat,
    Rescue,
    Conversation,
}

/// A single recorded interaction between two entities.
#[derive(Debug, Clone)]
pub struct Interaction {
    pub interaction_type: InteractionType,
    pub delta_trust: i32,
    pub delta_affection: i32,
    pub timestamp_ms: u64,
    pub note: String,
}

/// A bilateral relationship between two entities.
#[derive(Debug, Clone)]
pub struct Relationship {
    pub entity_a: String,
    pub entity_b: String,
    /// Trust score in the range [-100, 100].
    pub trust: i32,
    /// Affection score in the range [-100, 100].
    pub affection: i32,
    pub history: Vec<Interaction>,
    pub relationship_type: RelationshipType,
}

/// Classify a relationship from trust and affection scores.
pub fn relationship_type_from_scores(trust: i32, affection: i32) -> RelationshipType {
    if trust <= -60 || affection <= -60 {
        return RelationshipType::Enemy;
    }
    if trust <= -20 && affection <= -20 {
        return RelationshipType::Rival;
    }
    if trust >= 80 && affection >= 80 {
        return RelationshipType::Romantic;
    }
    if trust >= 70 && affection >= 50 {
        return RelationshipType::CloseFriend;
    }
    if trust >= 50 && affection >= 30 {
        return RelationshipType::Friend;
    }
    if trust >= 60 {
        return RelationshipType::Ally;
    }
    if trust >= 10 || affection >= 10 {
        return RelationshipType::Acquaintance;
    }
    RelationshipType::Stranger
}

/// Saturate a value to [-100, 100].
fn saturate(v: i32) -> i32 {
    v.clamp(-100, 100)
}

/// The relationship manager for all entity pairs.
pub struct RelationshipSystem {
    pub relationships: HashMap<(String, String), Relationship>,
}

impl RelationshipSystem {
    pub fn new() -> Self {
        Self { relationships: HashMap::new() }
    }

    /// Canonical lexicographic key so (A,B) and (B,A) map to the same entry.
    pub fn relationship_key(a: &str, b: &str) -> (String, String) {
        if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        }
    }

    /// Get or create the relationship between two entities.
    pub fn get_or_create(&mut self, a: &str, b: &str) -> &mut Relationship {
        let key = Self::relationship_key(a, b);
        self.relationships.entry(key.clone()).or_insert_with(|| Relationship {
            entity_a: key.0.clone(),
            entity_b: key.1.clone(),
            trust: 0,
            affection: 0,
            history: Vec::new(),
            relationship_type: RelationshipType::Stranger,
        })
    }

    /// Record an interaction and update trust/affection with saturation.
    pub fn record_interaction(&mut self, a: &str, b: &str, interaction: Interaction) {
        let rel = self.get_or_create(a, b);
        rel.trust = saturate(rel.trust + interaction.delta_trust);
        rel.affection = saturate(rel.affection + interaction.delta_affection);
        rel.relationship_type = relationship_type_from_scores(rel.trust, rel.affection);
        rel.history.push(interaction);
    }

    pub fn relationship_between(&self, a: &str, b: &str) -> Option<&Relationship> {
        let key = Self::relationship_key(a, b);
        self.relationships.get(&key)
    }

    /// All relationships involving the given entity.
    pub fn relationships_of(&self, entity: &str) -> Vec<&Relationship> {
        self.relationships.values()
            .filter(|r| r.entity_a == entity || r.entity_b == entity)
            .collect()
    }

    /// Names of entities with Ally, Friend, or CloseFriend relationship to `entity`.
    pub fn allies_of(&self, entity: &str) -> Vec<String> {
        self.relationships_of(entity).into_iter()
            .filter(|r| matches!(
                r.relationship_type,
                RelationshipType::Ally | RelationshipType::Friend | RelationshipType::CloseFriend
            ))
            .map(|r| {
                if r.entity_a == entity { r.entity_b.clone() } else { r.entity_a.clone() }
            })
            .collect()
    }

    /// Names of entities with Enemy relationship to `entity`.
    pub fn enemies_of(&self, entity: &str) -> Vec<String> {
        self.relationships_of(entity).into_iter()
            .filter(|r| r.relationship_type == RelationshipType::Enemy)
            .map(|r| {
                if r.entity_a == entity { r.entity_b.clone() } else { r.entity_a.clone() }
            })
            .collect()
    }

    /// Names of entities whose trust score towards `entity` is >= `min_trust`.
    pub fn trust_network(&self, entity: &str, min_trust: i32) -> Vec<String> {
        self.relationships_of(entity).into_iter()
            .filter(|r| r.trust >= min_trust)
            .map(|r| {
                if r.entity_a == entity { r.entity_b.clone() } else { r.entity_a.clone() }
            })
            .collect()
    }
}

impl Default for RelationshipSystem {
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

    fn greeting(dt: i32, da: i32) -> Interaction {
        Interaction {
            interaction_type: InteractionType::Greeting,
            delta_trust: dt,
            delta_affection: da,
            timestamp_ms: 0,
            note: String::new(),
        }
    }

    #[test]
    fn create_relationship() {
        let mut sys = RelationshipSystem::new();
        let rel = sys.get_or_create("Alice", "Bob");
        assert_eq!(rel.trust, 0);
        assert_eq!(rel.affection, 0);
        assert_eq!(rel.relationship_type, RelationshipType::Stranger);
    }

    #[test]
    fn interaction_updates_scores() {
        let mut sys = RelationshipSystem::new();
        sys.record_interaction("Alice", "Bob", greeting(20, 15));
        let rel = sys.relationship_between("Alice", "Bob").unwrap();
        assert_eq!(rel.trust, 20);
        assert_eq!(rel.affection, 15);
    }

    #[test]
    fn type_classified_correctly() {
        // Friend threshold: trust >= 50, affection >= 30
        assert_eq!(
            relationship_type_from_scores(55, 35),
            RelationshipType::Friend
        );
        assert_eq!(
            relationship_type_from_scores(-80, -80),
            RelationshipType::Enemy
        );
        assert_eq!(
            relationship_type_from_scores(0, 0),
            RelationshipType::Stranger
        );
        assert_eq!(
            relationship_type_from_scores(80, 85),
            RelationshipType::Romantic
        );
        assert_eq!(
            relationship_type_from_scores(70, 55),
            RelationshipType::CloseFriend
        );
    }

    #[test]
    fn allies_filter() {
        let mut sys = RelationshipSystem::new();
        // Make Alice and Bob friends.
        sys.record_interaction("Alice", "Bob", greeting(55, 35));
        // Make Alice and Charlie strangers.
        sys.get_or_create("Alice", "Charlie");

        let allies = sys.allies_of("Alice");
        assert!(allies.contains(&"Bob".to_string()));
        assert!(!allies.contains(&"Charlie".to_string()));
    }

    #[test]
    fn enemy_filter() {
        let mut sys = RelationshipSystem::new();
        sys.record_interaction("Alice", "Dave", greeting(-70, -70));
        let enemies = sys.enemies_of("Alice");
        assert!(enemies.contains(&"Dave".to_string()));
    }

    #[test]
    fn trust_saturation_at_100() {
        let mut sys = RelationshipSystem::new();
        // Push trust far above 100.
        sys.record_interaction("Alice", "Bob", greeting(80, 0));
        sys.record_interaction("Alice", "Bob", greeting(80, 0));
        let rel = sys.relationship_between("Alice", "Bob").unwrap();
        assert_eq!(rel.trust, 100, "trust must be capped at 100");
    }
}
