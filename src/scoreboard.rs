//! Scoreboard persistence — saves/loads top scores as JSON.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const SCOREBOARD_FILE: &str = "chaos_rpg_scores.json";
const MAX_ENTRIES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub class: String,
    pub score: u64,
    pub floor_reached: u32,
    pub enemies_defeated: u32,
    pub overflow_events: u32,
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
            // Simple timestamp from system time
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
