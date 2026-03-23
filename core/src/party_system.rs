//! Party management system: formation, shared resources, group bonuses.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// PartyRole
// ---------------------------------------------------------------------------

/// Role a character fills in the party.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyRole {
    Tank,
    Healer,
    DamageDealer,
    Support,
    Scout,
}

impl PartyRole {
    /// Recommended row position: 1 = front, 2 = mid, 3 = back.
    pub fn recommended_position(&self) -> u8 {
        match self {
            PartyRole::Tank => 1,
            PartyRole::DamageDealer => 1,
            PartyRole::Healer => 3,
            PartyRole::Support => 2,
            PartyRole::Scout => 2,
        }
    }
}

// ---------------------------------------------------------------------------
// PartyMember
// ---------------------------------------------------------------------------

/// A single member of a party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMember {
    pub character_id: String,
    pub name: String,
    pub role: PartyRole,
    pub level: u8,
    pub hp: u32,
    pub max_hp: u32,
    pub is_leader: bool,
}

impl PartyMember {
    /// Create a new party member.
    pub fn new(
        character_id: impl Into<String>,
        name: impl Into<String>,
        role: PartyRole,
        level: u8,
        max_hp: u32,
    ) -> Self {
        Self {
            character_id: character_id.into(),
            name: name.into(),
            role,
            level,
            hp: max_hp,
            max_hp,
            is_leader: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Formation
// ---------------------------------------------------------------------------

/// Tactical formation that affects combat stats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Formation {
    /// Single-file line. Good defense, poor mobility.
    Line,
    /// V-shape with tank at tip. Balanced.
    Wedge,
    /// Protective ring. High defense, no mobility.
    Circle,
    /// Spread out. Poor defense, high mobility.
    Loose,
}

impl Formation {
    /// Defense bonus multiplier (1.0 = neutral).
    pub fn defense_bonus(&self) -> f64 {
        match self {
            Formation::Line => 1.15,
            Formation::Wedge => 1.05,
            Formation::Circle => 1.30,
            Formation::Loose => 0.85,
        }
    }

    /// Mobility penalty (0.0 = none, 1.0 = fully immobilized).
    pub fn mobility_penalty(&self) -> f64 {
        match self {
            Formation::Line => 0.20,
            Formation::Wedge => 0.10,
            Formation::Circle => 0.40,
            Formation::Loose => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// PartyBonus
// ---------------------------------------------------------------------------

/// A synergy bonus granted to the whole party.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyBonus {
    pub name: String,
    pub effect: String,
    pub value: f64,
}

impl PartyBonus {
    fn new(name: impl Into<String>, effect: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            effect: effect.into(),
            value,
        }
    }
}

// ---------------------------------------------------------------------------
// Party
// ---------------------------------------------------------------------------

/// A group of up to 6 adventurers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub members: Vec<PartyMember>,
    pub formation: Formation,
}

impl Default for Party {
    fn default() -> Self {
        Self::new()
    }
}

impl Party {
    /// Create an empty party with a Line formation.
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
            formation: Formation::Line,
        }
    }

    /// Add a member. Returns `false` if the party is already full (6 members).
    pub fn add_member(&mut self, member: PartyMember) -> bool {
        if self.members.len() >= 6 {
            return false;
        }
        self.members.push(member);
        true
    }

    /// Remove a member by character_id. Returns `true` if found and removed.
    pub fn remove_member(&mut self, character_id: &str) -> bool {
        let before = self.members.len();
        self.members.retain(|m| m.character_id != character_id);
        self.members.len() < before
    }

    /// Change the party's tactical formation.
    pub fn set_formation(&mut self, formation: Formation) {
        self.formation = formation;
    }

    /// Designate a new leader by character_id. Returns `false` if not found.
    pub fn set_leader(&mut self, character_id: &str) -> bool {
        let found = self.members.iter().any(|m| m.character_id == character_id);
        if !found {
            return false;
        }
        for m in &mut self.members {
            m.is_leader = m.character_id == character_id;
        }
        true
    }

    /// Compute active group synergy bonuses.
    pub fn compute_bonuses(&self) -> Vec<PartyBonus> {
        let mut bonuses = Vec::new();

        let has_tank = self.members.iter().any(|m| m.role == PartyRole::Tank);
        let has_healer = self.members.iter().any(|m| m.role == PartyRole::Healer);
        let has_dd = self.members.iter().any(|m| m.role == PartyRole::DamageDealer);
        let has_support = self.members.iter().any(|m| m.role == PartyRole::Support);
        let has_scout = self.members.iter().any(|m| m.role == PartyRole::Scout);

        if has_tank && has_healer && has_dd && has_support && has_scout {
            bonuses.push(PartyBonus::new(
                "Balanced Party",
                "all_stats_pct",
                0.10,
            ));
        }

        if has_healer {
            bonuses.push(PartyBonus::new(
                "Full Healer Bonus",
                "hp_regen_pct",
                0.05,
            ));
        }

        if has_tank {
            bonuses.push(PartyBonus::new(
                "Stalwart Vanguard",
                "damage_reduction_flat",
                2.0,
            ));
        }

        if has_scout {
            bonuses.push(PartyBonus::new(
                "Scout Awareness",
                "surprise_immunity",
                1.0,
            ));
        }

        // Large party bonus
        if self.members.len() >= 5 {
            bonuses.push(PartyBonus::new(
                "Full Company",
                "xp_bonus_pct",
                0.05,
            ));
        }

        bonuses
    }

    /// Average level of all members (0.0 if empty).
    pub fn average_level(&self) -> f64 {
        if self.members.is_empty() {
            return 0.0;
        }
        let total: u32 = self.members.iter().map(|m| m.level as u32).sum();
        total as f64 / self.members.len() as f64
    }

    /// Sum of current HP across all members.
    pub fn total_hp(&self) -> u32 {
        self.members.iter().map(|m| m.hp).sum()
    }

    /// Members that are still alive (hp > 0).
    pub fn living_members(&self) -> Vec<&PartyMember> {
        self.members.iter().filter(|m| m.hp > 0).collect()
    }

    /// Distribute `total_xp` weighted by member level.
    /// Returns a list of `(character_id, xp_awarded)`.
    pub fn share_experience(&self, total_xp: u64) -> Vec<(String, u64)> {
        let living = self.living_members();
        if living.is_empty() {
            return Vec::new();
        }

        let weight_sum: u32 = living.iter().map(|m| m.level as u32).sum();
        if weight_sum == 0 {
            // All level 0 — split equally
            let share = total_xp / living.len() as u64;
            return living
                .iter()
                .map(|m| (m.character_id.clone(), share))
                .collect();
        }

        let mut result = Vec::new();
        let mut distributed: u64 = 0;
        for (i, member) in living.iter().enumerate() {
            let share = if i == living.len() - 1 {
                // Give remainder to last member to avoid rounding loss
                total_xp - distributed
            } else {
                (total_xp * member.level as u64) / weight_sum as u64
            };
            distributed += share;
            result.push((member.character_id.clone(), share));
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tank() -> PartyMember {
        PartyMember::new("t1", "Gorok", PartyRole::Tank, 5, 200)
    }
    fn healer() -> PartyMember {
        PartyMember::new("h1", "Lyria", PartyRole::Healer, 4, 120)
    }
    fn dd() -> PartyMember {
        PartyMember::new("d1", "Slash", PartyRole::DamageDealer, 6, 100)
    }
    fn support() -> PartyMember {
        PartyMember::new("s1", "Aria", PartyRole::Support, 3, 80)
    }
    fn scout() -> PartyMember {
        PartyMember::new("sc1", "Finn", PartyRole::Scout, 2, 90)
    }

    #[test]
    fn test_role_positions() {
        assert_eq!(PartyRole::Tank.recommended_position(), 1);
        assert_eq!(PartyRole::Healer.recommended_position(), 3);
        assert_eq!(PartyRole::Support.recommended_position(), 2);
        assert_eq!(PartyRole::Scout.recommended_position(), 2);
        assert_eq!(PartyRole::DamageDealer.recommended_position(), 1);
    }

    #[test]
    fn test_formation_stats() {
        assert!(Formation::Circle.defense_bonus() > Formation::Line.defense_bonus());
        assert!(Formation::Loose.mobility_penalty() < Formation::Circle.mobility_penalty());
        assert_eq!(Formation::Loose.mobility_penalty(), 0.0);
    }

    #[test]
    fn test_add_member() {
        let mut party = Party::new();
        assert!(party.add_member(tank()));
        assert_eq!(party.members.len(), 1);
    }

    #[test]
    fn test_max_six_members() {
        let mut party = Party::new();
        for i in 0..6 {
            let m = PartyMember::new(
                format!("x{i}"),
                format!("Member {i}"),
                PartyRole::Tank,
                1,
                100,
            );
            assert!(party.add_member(m));
        }
        let extra = PartyMember::new("x7", "Extra", PartyRole::Tank, 1, 100);
        assert!(!party.add_member(extra));
        assert_eq!(party.members.len(), 6);
    }

    #[test]
    fn test_remove_member() {
        let mut party = Party::new();
        party.add_member(tank());
        assert!(party.remove_member("t1"));
        assert!(party.members.is_empty());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut party = Party::new();
        party.add_member(tank());
        assert!(!party.remove_member("nobody"));
    }

    #[test]
    fn test_set_leader() {
        let mut party = Party::new();
        party.add_member(tank());
        party.add_member(healer());
        assert!(party.set_leader("h1"));
        assert!(!party.members[0].is_leader);
        assert!(party.members[1].is_leader);
    }

    #[test]
    fn test_set_leader_not_found() {
        let mut party = Party::new();
        party.add_member(tank());
        assert!(!party.set_leader("ghost"));
    }

    #[test]
    fn test_average_level_empty() {
        let party = Party::new();
        assert_eq!(party.average_level(), 0.0);
    }

    #[test]
    fn test_average_level() {
        let mut party = Party::new();
        party.add_member(tank()); // level 5
        party.add_member(healer()); // level 4
        let avg = party.average_level();
        assert!((avg - 4.5).abs() < 1e-9);
    }

    #[test]
    fn test_total_hp() {
        let mut party = Party::new();
        party.add_member(tank()); // max_hp=200
        party.add_member(healer()); // max_hp=120
        assert_eq!(party.total_hp(), 320);
    }

    #[test]
    fn test_living_members() {
        let mut party = Party::new();
        let mut dead = PartyMember::new("dead", "Ghost", PartyRole::Tank, 1, 100);
        dead.hp = 0;
        party.add_member(dead);
        party.add_member(healer());
        let living = party.living_members();
        assert_eq!(living.len(), 1);
        assert_eq!(living[0].character_id, "h1");
    }

    #[test]
    fn test_balanced_party_bonus() {
        let mut party = Party::new();
        party.add_member(tank());
        party.add_member(healer());
        party.add_member(dd());
        party.add_member(support());
        party.add_member(scout());
        let bonuses = party.compute_bonuses();
        let names: Vec<&str> = bonuses.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"Balanced Party"));
        assert!(names.contains(&"Full Healer Bonus"));
    }

    #[test]
    fn test_healer_bonus_without_balance() {
        let mut party = Party::new();
        party.add_member(healer());
        let bonuses = party.compute_bonuses();
        let names: Vec<&str> = bonuses.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"Full Healer Bonus"));
        assert!(!names.contains(&"Balanced Party"));
    }

    #[test]
    fn test_share_experience_weighted() {
        let mut party = Party::new();
        party.add_member(tank()); // level 5
        party.add_member(healer()); // level 4
        // total weight = 9, total_xp = 900
        let shares = party.share_experience(900);
        assert_eq!(shares.len(), 2);
        let tank_share = shares.iter().find(|(id, _)| id == "t1").unwrap().1;
        let heal_share = shares.iter().find(|(id, _)| id == "h1").unwrap().1;
        assert!(tank_share > heal_share);
        assert_eq!(tank_share + heal_share, 900);
    }

    #[test]
    fn test_share_experience_no_living() {
        let mut party = Party::new();
        let mut dead = PartyMember::new("d", "D", PartyRole::Tank, 1, 100);
        dead.hp = 0;
        party.add_member(dead);
        let shares = party.share_experience(1000);
        assert!(shares.is_empty());
    }

    #[test]
    fn test_formation_change() {
        let mut party = Party::new();
        party.set_formation(Formation::Circle);
        assert_eq!(party.formation, Formation::Circle);
    }
}
