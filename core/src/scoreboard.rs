//! Scoreboard persistence — saves/loads top scores and Hall of Misery as JSON.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const SCOREBOARD_FILE: &str = "chaos_rpg_scores.json";
const MISERY_FILE: &str = "chaos_rpg_misery.json";
const MAX_ENTRIES: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// One run on the local high-score table.
pub struct ScoreEntry {
    /// Character name.
    pub name: String,
    /// Class name.
    pub class: String,
    /// Final score.
    pub score: u64,
    /// Deepest floor reached.
    pub floor_reached: u32,
    /// Enemies killed.
    pub enemies_defeated: u32,
    /// Integer overflows that happened during the run.
    pub overflow_events: u32,
    /// When the run ended, UTC, as "YYYY-MM-DD HH:MMZ".
    pub timestamp: String,
    // New fields — default for backward compat with old saves
    #[serde(default)]
    /// Power tier name at the end of the run (empty in saves from before tiers).
    pub power_tier: String,
    #[serde(default)]
    /// Misery index at the end of the run.
    pub misery_index: f64,
    #[serde(default)]
    /// Underdog score multiplier (1.0 for a new entry; 0.0 in older saves that lack it).
    pub underdog_mult: f64,
}

/// Hall of Misery entry — sorted by misery score, not power score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseryEntry {
    /// Character name.
    pub name: String,
    /// Class name.
    pub class: String,
    /// Misery index at death.
    pub misery_index: f64,
    /// Deepest floor reached.
    pub floor_reached: u32,
    /// Power tier name at death.
    pub power_tier: String,
    /// Spite spent during the run.
    pub spite_spent: f64,
    /// Defiance rolls made during the run.
    pub defiance_rolls: u64,
    /// What killed the character.
    pub cause_of_death: String,
    /// Ranking score: misery index times floor times underdog multiplier.
    pub misery_score: u64,  // misery × floor × underdog_mult
    /// When the run ended, UTC, as "YYYY-MM-DD HH:MMZ".
    pub timestamp: String,
}

impl ScoreEntry {
    /// A new entry stamped with the current time.
    pub fn new(
        name: impl Into<String>,
        class: impl Into<String>,
        score: u64,
        floor_reached: u32,
        enemies_defeated: u32,
        overflow_events: u32,
    ) -> Self {
        let timestamp = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format_unix_timestamp(secs)
        };
        Self {
            name: name.into(),
            class: class.into(),
            score,
            floor_reached,
            enemies_defeated,
            overflow_events,
            timestamp,
            power_tier: String::new(),
            misery_index: 0.0,
            underdog_mult: 1.0,
        }
    }

    /// Set the power tier name.
    pub fn with_tier(mut self, tier: impl Into<String>) -> Self {
        self.power_tier = tier.into(); self
    }
    /// Set the misery index and underdog multiplier.
    pub fn with_misery(mut self, misery: f64, underdog: f64) -> Self {
        self.misery_index = misery;
        self.underdog_mult = underdog;
        self
    }
}

impl MiseryEntry {
    /// A new Hall of Misery entry stamped with the current time; the misery score is computed here.
    pub fn new(
        name: impl Into<String>,
        class: impl Into<String>,
        misery_index: f64,
        floor_reached: u32,
        power_tier: impl Into<String>,
        spite_spent: f64,
        defiance_rolls: u64,
        cause_of_death: impl Into<String>,
        underdog_mult: f64,
    ) -> Self {
        let misery_score = (misery_index * floor_reached as f64 * underdog_mult) as u64;
        let timestamp = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format_unix_timestamp(secs)
        };
        Self {
            name: name.into(),
            class: class.into(),
            misery_index,
            floor_reached,
            power_tier: power_tier.into(),
            spite_spent,
            defiance_rolls,
            cause_of_death: cause_of_death.into(),
            misery_score,
            timestamp,
        }
    }
}

fn format_unix_timestamp(secs: u64) -> String {
    crate::time_util::timestamp_from_unix(secs as i64)
}

fn score_path() -> PathBuf {
    // Save next to executable, or in current dir
    crate::paths::data_file(SCOREBOARD_FILE)
}

/// Read the high-score table; empty if there is no readable file.
pub fn load_scores() -> Vec<ScoreEntry> {
    let path = score_path();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Add an entry, keep the best 20 by score, save, and return the table.
pub fn save_score(entry: ScoreEntry) -> Vec<ScoreEntry> {
    let mut scores = load_scores();
    scores.push(entry);
    scores.sort_by_key(|e| std::cmp::Reverse(e.score));
    scores.truncate(MAX_ENTRIES);
    let json = serde_json::to_string_pretty(&scores).unwrap_or_default();
    let _ = std::fs::write(score_path(), json);
    scores
}

fn misery_path() -> PathBuf {
    crate::paths::data_file(MISERY_FILE)
}

/// Read the Hall of Misery; empty if there is no readable file.
pub fn load_misery_scores() -> Vec<MiseryEntry> {
    let path = misery_path();
    let Ok(data) = std::fs::read_to_string(&path) else { return Vec::new(); };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Add an entry, keep the 20 most miserable, save, and return the list.
pub fn save_misery_score(entry: MiseryEntry) -> Vec<MiseryEntry> {
    let mut scores = load_misery_scores();
    scores.push(entry);
    scores.sort_by_key(|e| std::cmp::Reverse(e.misery_score));
    scores.truncate(MAX_ENTRIES);
    let json = serde_json::to_string_pretty(&scores).unwrap_or_default();
    let _ = std::fs::write(misery_path(), json);
    scores
}
