use std::collections::HashMap;

/// The kind of trap.
#[derive(Debug, Clone, PartialEq)]
pub enum TrapType {
    PitFall,
    Snare,
    Poison,
    Alarm,
    Explosion,
    Magic(String),
}

/// Lifecycle state of a trap.
#[derive(Debug, Clone, PartialEq)]
pub enum TrapState {
    Hidden,
    Detected,
    Triggered,
    Disarmed,
    Sprung,
}

/// A single trap in the world.
#[derive(Debug, Clone)]
pub struct Trap {
    pub id: u32,
    pub name: String,
    pub trap_type: TrapType,
    pub state: TrapState,
    pub detection_dc: u32,
    pub disarm_dc: u32,
    pub trigger_dc: u32,
    /// (num_dice, sides)
    pub damage_dice: (u8, u8),
    pub location: (u32, u32),
    pub triggered_by: Option<String>,
}

/// Result of an attempt to disarm a trap.
#[derive(Debug, Clone, PartialEq)]
pub enum DisarmResult {
    Success,
    Failure { damage: u32 },
    CriticalSuccess { looted_component: String },
    CriticalFailure { damage: u32, trap_springs: bool },
}

/// Linear congruential generator — returns a value in [1, 20].
pub fn lcg_next(state: &mut u64) -> u32 {
    *state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
    let high = (*state >> 33) as u32;
    (high % 20) + 1
}

/// Manages all traps in the game world.
pub struct TrapSystem {
    pub traps: HashMap<u32, Trap>,
    pub next_id: u32,
    pub lcg_state: u64,
}

impl TrapSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            traps: HashMap::new(),
            next_id: 1,
            lcg_state: seed,
        }
    }

    /// Place a new trap and return its id.
    pub fn place_trap(
        &mut self,
        trap_type: TrapType,
        location: (u32, u32),
        detection_dc: u32,
        disarm_dc: u32,
        damage_dice: (u8, u8),
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let name = format!("{:?} Trap #{}", trap_type, id);
        let trap = Trap {
            id,
            name,
            trap_type,
            state: TrapState::Hidden,
            detection_dc,
            disarm_dc,
            trigger_dc: 10,
            damage_dice,
            location,
            triggered_by: None,
        };
        self.traps.insert(id, trap);
        id
    }

    /// Roll perception vs detection DC for all traps at the given location.
    /// Returns IDs of traps that were newly detected.
    pub fn detect_traps(&mut self, searcher_skill: u32, location: (u32, u32)) -> Vec<u32> {
        let ids: Vec<u32> = self
            .traps
            .values()
            .filter(|t| t.location == location && t.state == TrapState::Hidden)
            .map(|t| t.id)
            .collect();

        let mut detected = Vec::new();
        for id in ids {
            let roll = lcg_next(&mut self.lcg_state);
            let total = roll + searcher_skill;
            if let Some(trap) = self.traps.get_mut(&id) {
                if total >= trap.detection_dc {
                    trap.state = TrapState::Detected;
                    detected.push(id);
                }
            }
        }
        detected
    }

    /// Attempt to disarm a trap by id.
    pub fn attempt_disarm(&mut self, trap_id: u32, skill: u32) -> DisarmResult {
        let roll = lcg_next(&mut self.lcg_state);
        let trap = match self.traps.get_mut(&trap_id) {
            Some(t) => t,
            None => return DisarmResult::Failure { damage: 0 },
        };

        if roll == 20 {
            trap.state = TrapState::Disarmed;
            return DisarmResult::CriticalSuccess {
                looted_component: format!("{:?} component", trap.trap_type),
            };
        }
        if roll == 1 {
            let dmg = roll_damage_lcg(&mut 0u64, trap.damage_dice);
            trap.state = TrapState::Sprung;
            return DisarmResult::CriticalFailure {
                damage: dmg,
                trap_springs: true,
            };
        }

        let total = roll + skill;
        if total >= trap.disarm_dc {
            trap.state = TrapState::Disarmed;
            DisarmResult::Success
        } else {
            let dmg = roll_damage_lcg(&mut 0u64, trap.damage_dice);
            DisarmResult::Failure { damage: dmg }
        }
    }

    /// Trigger a trap directly; returns damage dealt.
    pub fn trigger_trap(&mut self, trap_id: u32, entity: &str) -> u32 {
        let trap = match self.traps.get_mut(&trap_id) {
            Some(t) => t,
            None => return 0,
        };
        trap.state = TrapState::Sprung;
        trap.triggered_by = Some(entity.to_string());
        roll_damage_lcg(&mut 0u64, trap.damage_dice)
    }

    /// Check all traps at a location when an entity steps on them.
    /// Hidden traps: roll perception vs trigger_dc to avoid.
    /// Returns (trap_id, damage) pairs for each triggered trap.
    pub fn step_on_trap(
        &mut self,
        location: (u32, u32),
        entity: &str,
        perception: u32,
    ) -> Vec<(u32, u32)> {
        let ids: Vec<u32> = self
            .traps
            .values()
            .filter(|t| {
                t.location == location
                    && (t.state == TrapState::Hidden || t.state == TrapState::Detected)
            })
            .map(|t| t.id)
            .collect();

        let mut results = Vec::new();
        for id in ids {
            let roll = lcg_next(&mut self.lcg_state);
            let total = roll + perception;
            let (trigger_dc, damage_dice, is_hidden) = {
                let t = self.traps.get(&id).unwrap();
                (t.trigger_dc, t.damage_dice, t.state == TrapState::Hidden)
            };

            // For hidden traps, perception check avoids triggering.
            if is_hidden && total >= trigger_dc {
                // Spotted and avoided
                if let Some(t) = self.traps.get_mut(&id) {
                    t.state = TrapState::Detected;
                }
                continue;
            }

            // Triggered
            let dmg = roll_damage_lcg(&mut 0u64, damage_dice);
            if let Some(t) = self.traps.get_mut(&id) {
                t.state = TrapState::Sprung;
                t.triggered_by = Some(entity.to_string());
            }
            results.push((id, dmg));
        }
        results
    }

    /// Return all traps at a given location.
    pub fn traps_at(&self, location: (u32, u32)) -> Vec<&Trap> {
        self.traps
            .values()
            .filter(|t| t.location == location)
            .collect()
    }
}

/// Roll damage dice using a simple counter seed.
fn roll_damage_lcg(state: &mut u64, dice: (u8, u8)) -> u32 {
    let (num, sides) = dice;
    if sides == 0 {
        return 0;
    }
    let mut total = 0u32;
    for i in 0..num {
        let mut s = state.wrapping_add(i as u64).wrapping_add(12345678);
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        total += (s >> 33) as u32 % sides as u32 + 1;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_trap() {
        let mut sys = TrapSystem::new(42);
        let id = sys.place_trap(TrapType::PitFall, (5, 5), 12, 14, (1, 6));
        assert_eq!(id, 1);
        assert!(sys.traps.contains_key(&1));
        assert_eq!(sys.traps[&1].state, TrapState::Hidden);
    }

    #[test]
    fn test_detect_reveals_trap() {
        let mut sys = TrapSystem::new(99);
        let id = sys.place_trap(TrapType::Snare, (3, 3), 5, 12, (1, 4));
        // High perception skill guarantees detection (roll 1..20 + 30 >= 5 always)
        let detected = sys.detect_traps(30, (3, 3));
        assert!(detected.contains(&id));
        assert_eq!(sys.traps[&id].state, TrapState::Detected);
    }

    #[test]
    fn test_detect_misses_with_low_skill() {
        let mut sys = TrapSystem::new(7);
        sys.place_trap(TrapType::Alarm, (1, 1), 30, 12, (0, 0));
        // Even nat 20 + 0 = 20 < 30, so never detected
        let detected = sys.detect_traps(0, (1, 1));
        // With DC 30 and max roll of 20+0=20, should not detect
        assert!(detected.is_empty());
    }

    #[test]
    fn test_disarm_result_types() {
        // Just verify disarm returns a valid result
        let mut sys = TrapSystem::new(1234);
        let id = sys.place_trap(TrapType::Poison, (0, 0), 10, 10, (1, 6));
        let result = sys.attempt_disarm(id, 5);
        // Result should be one of the valid variants
        match result {
            DisarmResult::Success
            | DisarmResult::Failure { .. }
            | DisarmResult::CriticalSuccess { .. }
            | DisarmResult::CriticalFailure { .. } => {}
        }
    }

    #[test]
    fn test_trigger_returns_damage() {
        let mut sys = TrapSystem::new(555);
        let id = sys.place_trap(TrapType::Explosion, (2, 2), 10, 15, (2, 6));
        let dmg = sys.trigger_trap(id, "hero");
        // Damage should be > 0 for 2d6
        assert!(dmg >= 2 && dmg <= 12);
        assert_eq!(sys.traps[&id].state, TrapState::Sprung);
        assert_eq!(sys.traps[&id].triggered_by, Some("hero".to_string()));
    }

    #[test]
    fn test_step_on_triggers_hidden() {
        let mut sys = TrapSystem::new(77);
        // DC=30 means perception check (max 20+0=20) always fails, trap triggers
        let id = sys.place_trap(TrapType::PitFall, (4, 4), 10, 12, (1, 6));
        // Manually set trigger_dc very high so it always triggers
        sys.traps.get_mut(&id).unwrap().trigger_dc = 100;
        let hits = sys.step_on_trap((4, 4), "rogue", 0);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].0, id);
    }

    #[test]
    fn test_traps_at_location() {
        let mut sys = TrapSystem::new(321);
        let id1 = sys.place_trap(TrapType::Snare, (7, 7), 10, 10, (1, 4));
        let id2 = sys.place_trap(TrapType::Alarm, (7, 7), 10, 10, (0, 0));
        sys.place_trap(TrapType::Magic("Fire".into()), (9, 9), 10, 10, (1, 8));
        let at = sys.traps_at((7, 7));
        let at_ids: Vec<u32> = at.iter().map(|t| t.id).collect();
        assert!(at_ids.contains(&id1));
        assert!(at_ids.contains(&id2));
        assert_eq!(at.len(), 2);
    }
}
