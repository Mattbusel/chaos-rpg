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

/// One row of `GET /scores`.
///
/// The reference server (`server/worker.js`) stores `seed` as a string
/// (`String(body.seed)`), so `seed` accepts a JSON string or number. Missing
/// fields default instead of failing the whole list.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct LeaderboardRow {
    pub rank:   u32,
    pub name:   String,
    pub class:  String,
    pub floor:  u32,
    pub score:  u64,
    pub kills:  u64,
    #[serde(deserialize_with = "de_u64_lenient")]
    pub seed:   u64,
    pub won:    bool,
}

/// Accept `123`, `"123"` or `null` for a u64 field.
fn de_u64_lenient<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    use serde::de::Error;
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Number(n) => n
            .as_u64()
            .ok_or_else(|| D::Error::custom(format!("seed {n} is not a u64"))),
        serde_json::Value::String(s) => s
            .trim()
            .parse::<u64>()
            .map_err(|e| D::Error::custom(format!("seed {s:?}: {e}"))),
        serde_json::Value::Null => Ok(0),
        other => Err(D::Error::custom(format!("seed has unexpected type: {other}"))),
    }
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
        crate::paths::data_file("chaos_rpg_daily.json")
    }
}

// ── HTTP client ───────────────────────────────────────────────────────────────

/// Submit score to the leaderboard. Returns Ok(rank) on success.
/// Non-blocking: times out after 5 seconds. Safe to call on game thread.
pub fn submit_score(endpoint: &str, entry: &DailyEntry) -> Result<u32, String> {
    if endpoint.trim().is_empty() {
        return Err("no leaderboard endpoint configured".to_string());
    }
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
    if endpoint.trim().is_empty() {
        return Err("no leaderboard endpoint configured".to_string());
    }
    let url = format!("{}/scores?date={}", endpoint.trim_end_matches('/'), date);
    let resp = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .map_err(|e| e.to_string())?;

    let rows: Vec<LeaderboardRow> = resp.into_json().map_err(|e| e.to_string())?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;

    /// Serve exactly one HTTP request with `body`, returning (request line,
    /// request body) to the test.
    fn one_shot_server(status: &'static str, body: &'static str) -> (String, std::thread::JoinHandle<(String, String)>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" || line.is_empty() {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = v.trim().parse().unwrap();
                }
            }
            let mut req_body = vec![0u8; content_length];
            reader.read_exact(&mut req_body).unwrap();
            let mut stream = stream;
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            (request_line.trim().to_string(), String::from_utf8(req_body).unwrap())
        });
        (url, handle)
    }

    /// Rows exactly as `server/worker.js` stores them: `seed` is a string and
    /// there is an extra `ts` field. Before 2.3.0 this failed to parse
    /// ("invalid type: string, expected u64"), so every non-empty board showed
    /// a fetch error.
    #[test]
    fn fetch_parses_worker_rows() {
        let (url, h) = one_shot_server(
            "200 OK",
            r#"[{"name":"Ada","class":"Mage","floor":7,"score":4200,"kills":31,"seed":"9876543210","won":false,"ts":1759449600000,"rank":1},
                {"name":"Bo","class":"Rogue","floor":3,"score":900,"kills":8,"seed":42,"won":true,"rank":2}]"#,
        );
        let rows = fetch_scores(&format!("{url}/"), "2025-10-03").unwrap();
        let (req, _) = h.join().unwrap();
        assert_eq!(req, "GET /scores?date=2025-10-03 HTTP/1.1");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].seed, 9_876_543_210);
        assert_eq!(rows[0].name, "Ada");
        assert_eq!(rows[1].seed, 42);
        assert!(rows[1].won);
    }

    #[test]
    fn submit_sends_entry_and_reads_rank() {
        let (url, h) = one_shot_server("200 OK", r#"{"ok":true,"rank":3}"#);
        let entry = DailyEntry {
            date: "2025-10-03".into(),
            name: "Ada".into(),
            class: "Mage".into(),
            floor: 7,
            score: 4200,
            kills: 31,
            seed: 12345,
            won: true,
        };
        assert_eq!(submit_score(&url, &entry).unwrap(), 3);
        let (req, body) = h.join().unwrap();
        assert_eq!(req, "POST /submit HTTP/1.1");
        let sent: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(sent["date"], "2025-10-03");
        assert_eq!(sent["score"], 4200);
        assert_eq!(sent["seed"], 12345);
        assert_eq!(sent["won"], true);
    }

    #[test]
    fn server_error_is_reported() {
        let (url, h) = one_shot_server("400 Bad Request", r#"{"error":"invalid date"}"#);
        let err = fetch_scores(&url, "nope").unwrap_err();
        h.join().unwrap();
        assert!(err.contains("400"), "{err}");
    }

    #[test]
    fn empty_endpoint_is_an_error_without_network() {
        assert!(fetch_scores("", "2025-10-03").is_err());
        assert!(submit_score("  ", &DailyEntry {
            date: String::new(), name: String::new(), class: String::new(),
            floor: 0, score: 0, kills: 0, seed: 0, won: false,
        }).is_err());
    }

    #[test]
    fn bad_seed_is_rejected() {
        let r: Result<Vec<LeaderboardRow>, _> = serde_json::from_str(r#"[{"seed":"abc"}]"#);
        assert!(r.is_err());
        let r: Vec<LeaderboardRow> = serde_json::from_str(r#"[{"seed":null,"name":"x"}]"#).unwrap();
        assert_eq!(r[0].seed, 0);
    }
}
