//! Nemesis system — the enemy that killed you follows you to the next run.
//!
//! When you die, the killing enemy is saved. Next run, it appears with
//! boosted stats, resistance to whatever finished you, and a grudge.
//! Kill your Nemesis for bonus loot. Die again and it gets even stronger.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const NEMESIS_FILE: &str = "chaos_rpg_nemesis.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NemesisRecord {
    pub enemy_name: String,
    pub floor_killed_at: u32,
    pub times_killed_player: u32,
    pub killer_base_damage: i64,
    pub hp_bonus_pct: u64,
    pub damage_bonus_pct: u64,
    /// What damage type killed the player ("spell", "physical", "status")
    pub kill_method: String,
    pub killed_player_class: String,
}

impl NemesisRecord {
    pub fn new(
        enemy_name: String,
        floor: u32,
        base_damage: i64,
        player_class: String,
        kill_method: &str,
    ) -> Self {
        NemesisRecord {
            enemy_name,
            floor_killed_at: floor,
            times_killed_player: 1,
            killer_base_damage: base_damage,
            hp_bonus_pct: 30,
            damage_bonus_pct: 25,
            kill_method: kill_method.to_string(),
            killed_player_class: player_class,
        }
    }

    /// Called when the Nemesis kills the player again.
    pub fn escalate(&mut self) {
        self.times_killed_player += 1;
        self.hp_bonus_pct = (self.hp_bonus_pct + 30).min(300);
        self.damage_bonus_pct = (self.damage_bonus_pct + 25).min(200);
    }

    /// Resistance description for display.
    pub fn resistance_label(&self) -> &str {
        match self.kill_method.as_str() {
            "spell" => "Resistant: Spell Damage",
            "physical" => "Resistant: Physical Attacks",
            "status" => "Resistant: Status Effects",
            _ => "Resistant: Unknown",
        }
    }
}

fn nemesis_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(NEMESIS_FILE)))
        .unwrap_or_else(|| PathBuf::from(NEMESIS_FILE))
}

pub fn load_nemesis() -> Option<NemesisRecord> {
    let path = nemesis_path();
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_nemesis(record: &NemesisRecord) {
    let json = serde_json::to_string_pretty(record).unwrap_or_default();
    let _ = std::fs::write(nemesis_path(), json);
}

pub fn clear_nemesis() {
    let _ = std::fs::remove_file(nemesis_path());
}
