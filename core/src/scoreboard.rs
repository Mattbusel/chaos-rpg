//! Scoreboard persistence — saves/loads top scores and Hall of Misery as JSON.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const SCOREBOARD_FILE: &str = "chaos_rpg_scores.json";
const MISERY_FILE: &str = "chaos_rpg_misery.json";
const MAX_ENTRIES: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub class: String,
    pub score: u64,
    pub floor_reached: u32,
    pub enemies_defeated: u32,
    pub overflow_events: u32,
    pub timestamp: String,
    // New fields — default for backward compat with old saves
    #[serde(default)]
    pub power_tier: String,
    #[serde(default)]
    pub misery_index: f64,
    #[serde(default)]
    pub underdog_mult: f64,
}

/// Hall of Misery entry — sorted by misery score, not power score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseryEntry {
    pub name: String,
    pub class: String,
    pub misery_index: f64,
    pub floor_reached: u32,
    pub power_tier: String,
    pub spite_spent: f64,
    pub defiance_rolls: u64,
    pub cause_of_death: String,
    pub misery_score: u64,  // misery × floor × underdog_mult
    pub timestamp: String,
}

impl ScoreEntry {
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

    pub fn with_tier(mut self, tier: impl Into<String>) -> Self {
        self.power_tier = tier.into(); self
    }
    pub fn with_misery(mut self, misery: f64, underdog: f64) -> Self {
        self.misery_index = misery;
        self.underdog_mult = underdog;
        self
    }
}

impl MiseryEntry {
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
    // Rough UTC date from unix timestamp (no chrono dependency)
    let days_since_epoch = secs / 86400;
    let mut year = 1970u32;
    let mut days = days_since_epoch as u32;
    loop {
        let year_days =
            if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) {
                366
            } else {
                365
            };
        if days < year_days {
            break;
        }
        days -= year_days;
        year += 1;
    }
    let month_days = [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let mut month = 1u32;
    for &md in &month_days {
        let md = if month == 2 && is_leap { 29 } else { md };
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;
    let hour = (secs % 86400) / 3600;
    let minute = (secs % 3600) / 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}Z")
}

fn score_path() -> PathBuf {
    // Save next to executable, or in current dir
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(SCOREBOARD_FILE)))
        .unwrap_or_else(|| PathBuf::from(SCOREBOARD_FILE))
}

pub fn load_scores() -> Vec<ScoreEntry> {
    let path = score_path();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_score(entry: ScoreEntry) -> Vec<ScoreEntry> {
    let mut scores = load_scores();
    scores.push(entry);
    scores.sort_by(|a, b| b.score.cmp(&a.score));
    scores.truncate(MAX_ENTRIES);
    let json = serde_json::to_string_pretty(&scores).unwrap_or_default();
    let _ = std::fs::write(score_path(), json);
    scores
}

fn misery_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(MISERY_FILE)))
        .unwrap_or_else(|| PathBuf::from(MISERY_FILE))
}

pub fn load_misery_scores() -> Vec<MiseryEntry> {
    let path = misery_path();
    let Ok(data) = std::fs::read_to_string(&path) else { return Vec::new(); };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_misery_score(entry: MiseryEntry) -> Vec<MiseryEntry> {
    let mut scores = load_misery_scores();
    scores.push(entry);
    scores.sort_by(|a, b| b.misery_score.cmp(&a.misery_score));
    scores.truncate(MAX_ENTRIES);
    let json = serde_json::to_string_pretty(&scores).unwrap_or_default();
    let _ = std::fs::write(misery_path(), json);
    scores
}
