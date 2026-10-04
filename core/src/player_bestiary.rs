//! Player Bestiary — persistent record of all enemies and bosses encountered.
//!
//! Distinct from bestiary.rs (the combat AI system). This module tracks
//! encounter history across runs and stores it in bestiary.json.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// What the player has learned about one enemy type across all runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterRecord {
    /// Enemy name as it appears in game.
    pub name: String,
    /// Floor it was first seen on.
    pub first_seen_floor: u32,
    /// Date string of first encounter (e.g. "2026-03-23").
    pub first_seen_date: String,
    /// Whether this is a boss (unlocks lore on first kill, not first encounter).
    pub is_boss: bool,
    /// Total times fought.
    pub times_fought: u32,
    /// Total times the player killed it.
    pub times_killed: u32,
    /// Total times it killed the player.
    pub times_killed_player: u32,
    /// Minimum HP observed.
    pub min_hp_seen: i64,
    /// Maximum HP observed.
    pub max_hp_seen: i64,
    /// Minimum damage hit observed.
    pub min_damage_seen: i64,
    /// Maximum damage hit observed.
    pub max_damage_seen: i64,
    /// Whether lore entry is unlocked.
    pub lore_unlocked: bool,
    /// Whether strategy hint is unlocked (bosses only, after 3 encounters).
    pub strategy_unlocked: bool,
}

impl EncounterRecord {
    /// A record for a first encounter: fought once, nothing killed, HP and damage ranges unknown; normal enemies have their lore unlocked at once, bosses on first kill.
    pub fn new(name: String, floor: u32, date: String, is_boss: bool) -> Self {
        Self {
            name,
            first_seen_floor: floor,
            first_seen_date: date,
            is_boss,
            times_fought: 1,
            times_killed: 0,
            times_killed_player: 0,
            min_hp_seen: i64::MAX,
            max_hp_seen: i64::MIN,
            min_damage_seen: i64::MAX,
            max_damage_seen: i64::MIN,
            lore_unlocked: !is_boss, // normal enemies unlock lore on first encounter
            strategy_unlocked: false,
        }
    }

    /// Count a finished fight and widen the HP range; a boss kill unlocks its lore, and its strategy hint once fought 3 or more times.
    pub fn record_fight(&mut self, enemy_hp: i64, player_killed: bool, enemy_killed: bool) {
        self.times_fought += 1;
        if enemy_killed {
            self.times_killed += 1;
            if self.is_boss {
                self.lore_unlocked = true; // boss lore unlocks on first kill
            }
            if self.is_boss && self.times_fought >= 3 {
                self.strategy_unlocked = true;
            }
        }
        if player_killed {
            self.times_killed_player += 1;
        }
        self.min_hp_seen = self.min_hp_seen.min(enemy_hp);
        self.max_hp_seen = self.max_hp_seen.max(enemy_hp);
    }

    /// Widen the observed damage range with one hit (ignored if 0 or less).
    pub fn record_damage(&mut self, damage: i64) {
        if damage > 0 {
            self.min_damage_seen = self.min_damage_seen.min(damage);
            self.max_damage_seen = self.max_damage_seen.max(damage);
        }
    }

    /// Observed HP as text: one value, a "min to max" range, or "unknown".
    pub fn hp_range_display(&self) -> String {
        if self.min_hp_seen == i64::MAX {
            "unknown".to_string()
        } else if self.min_hp_seen == self.max_hp_seen {
            self.min_hp_seen.to_string()
        } else {
            format!("{} — {}", self.min_hp_seen, self.max_hp_seen)
        }
    }

    /// Observed damage per hit as text: one value, a "min to max" range, or "unknown".
    pub fn damage_range_display(&self) -> String {
        if self.min_damage_seen == i64::MAX {
            "unknown".to_string()
        } else if self.min_damage_seen == self.max_damage_seen {
            self.min_damage_seen.to_string()
        } else {
            format!("{} — {}", self.min_damage_seen, self.max_damage_seen)
        }
    }
}

/// All enemies the player has met, saved between runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerBestiary {
    /// Records keyed by enemy name.
    pub entries: HashMap<String, EncounterRecord>,
}

impl PlayerBestiary {
    /// Load from `chaos_rpg_bestiary.json` in the data folder, or start empty.
    pub fn load() -> Self {
        if let Ok(data) = std::fs::read_to_string(Self::path()) {
            if let Ok(b) = serde_json::from_str::<PlayerBestiary>(&data) {
                return b;
            }
        }
        Self::default()
    }

    /// Save to `chaos_rpg_bestiary.json` in the data folder, ignoring write errors.
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), json);
        }
    }

    /// Record a new encounter. Returns true if this is a first encounter.
    pub fn record_encounter(
        &mut self,
        name: &str,
        floor: u32,
        is_boss: bool,
        enemy_hp: i64,
    ) -> bool {
        let date = current_date_string();
        let is_new = !self.entries.contains_key(name);
        if is_new {
            self.entries.insert(
                name.to_string(),
                EncounterRecord::new(name.to_string(), floor, date, is_boss),
            );
        } else if let Some(rec) = self.entries.get_mut(name) {
            rec.times_fought += 1;
            rec.min_hp_seen = rec.min_hp_seen.min(enemy_hp);
            rec.max_hp_seen = rec.max_hp_seen.max(enemy_hp);
        }
        is_new
    }

    /// Record the end of a fight with a known enemy (ignored for unknown names).
    pub fn record_fight_result(
        &mut self,
        name: &str,
        enemy_hp: i64,
        player_killed: bool,
        enemy_killed: bool,
    ) {
        if let Some(rec) = self.entries.get_mut(name) {
            rec.record_fight(enemy_hp, player_killed, enemy_killed);
        }
    }

    /// Record a hit taken from a known enemy (ignored for unknown names).
    pub fn record_damage_received(&mut self, enemy_name: &str, damage: i64) {
        if let Some(rec) = self.entries.get_mut(enemy_name) {
            rec.record_damage(damage);
        }
    }

    /// Look up a record by enemy name.
    pub fn get(&self, name: &str) -> Option<&EncounterRecord> {
        self.entries.get(name)
    }

    /// Sorted list of all encountered names.
    pub fn sorted_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.entries.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Bosses first, then enemies alphabetically.
    pub fn sorted_for_display(&self) -> Vec<&EncounterRecord> {
        let mut records: Vec<&EncounterRecord> = self.entries.values().collect();
        records.sort_by(|a, b| {
            b.is_boss
                .cmp(&a.is_boss)
                .then(a.name.cmp(&b.name))
        });
        records
    }

    /// Number of different enemies met.
    pub fn total_encountered(&self) -> usize {
        self.entries.len()
    }

    /// Total kills across all enemies.
    pub fn total_killed(&self) -> u32 {
        self.entries.values().map(|r| r.times_killed).sum()
    }

    fn path() -> PathBuf {
        crate::paths::data_file("chaos_rpg_bestiary.json")
    }
}

fn current_date_string() -> String {
    crate::time_util::today_utc()
}
