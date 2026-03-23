// CHAOS RPG — Daily Seed Leaderboard
//
// Local component: stores today's best daily-seed score in JSON.
// Remote component: submits to / fetches from a configurable HTTP endpoint.
//
// The server-side contract (Cloudflare Worker) expects:
//   POST /submit  { "date", "name", "class", "floor", "score", "seed", "kills" }
//   GET  /scores?date=YYYY-MM-DD  →  [{ "name", "class", "floor", "score", "seed", "kills", "rank" }]

use serde::{Deserialize, Serialize};

// ── Local daily entry ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyEntry {
    pub date:   String,
    pub name:   String,
    pub class:  String,
    pub floor:  u32,
    pub score:  u64,
    pub kills:  u64,
    pub seed:   u64,
    pub won:    bool,
}

// ── Remote leaderboard row ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeaderboardRow {
    pub rank:   u32,
    pub name:   String,
    pub class:  String,
    pub floor:  u32,
    pub score:  u64,
    pub kills:  u64,
    pub seed:   u64,
    pub won:    bool,
}

// ── Local store ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalDailyStore {
    pub entries: Vec<DailyEntry>,
}

impl LocalDailyStore {
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(s) = serde_json::from_str(&data) { return s; }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), json);
        }
    }

    /// Record a daily entry; keeps only the best score for each date.
    pub fn record(&mut self, entry: DailyEntry) -> bool {
        let date = entry.date.clone();
        if let Some(existing) = self.entries.iter_mut().find(|e| e.date == date) {
            if entry.score > existing.score {
                *existing = entry;
                self.save();
                return true; // new personal best
            }
            return false;
        }
        self.entries.insert(0, entry);
        self.entries.truncate(90); // keep ~3 months
        self.save();
        true
    }

    pub fn best_for_today(&self, today: &str) -> Option<&DailyEntry> {
        self.entries.iter().find(|e| e.date == today)
    }

    fn path() -> std::path::PathBuf {
        let mut p = std::env::current_exe().unwrap_or_default();
        p.pop();
        p.push("chaos_rpg_daily.json");
        p
    }
}

// ── HTTP client ───────────────────────────────────────────────────────────────

/// Submit score to the leaderboard. Returns Ok(rank) on success.
/// Non-blocking: times out after 5 seconds. Safe to call on game thread.
pub fn submit_score(endpoint: &str, entry: &DailyEntry) -> Result<u32, String> {
    let url = format!("{}/submit", endpoint.trim_end_matches('/'));
    let resp = ureq::post(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send_json(serde_json::json!({
            "date":  entry.date,
            "name":  entry.name,
            "class": entry.class,
            "floor": entry.floor,
            "score": entry.score,
            "kills": entry.kills,
            "seed":  entry.seed,
            "won":   entry.won,
        }))
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    Ok(body["rank"].as_u64().unwrap_or(0) as u32)
}

/// Fetch today's leaderboard. Times out after 5 seconds.
pub fn fetch_scores(endpoint: &str, date: &str) -> Result<Vec<LeaderboardRow>, String> {
    let url = format!("{}/scores?date={}", endpoint.trim_end_matches('/'), date);
    let resp = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .map_err(|e| e.to_string())?;

    let rows: Vec<LeaderboardRow> = resp.into_json().map_err(|e| e.to_string())?;
    Ok(rows)
}
