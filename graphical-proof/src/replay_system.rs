//! Replay system — recording, playback, sharing, ghost runs.
//!
//! Records player inputs + seed for deterministic replay.

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════════════════════
// REPLAY RECORDING
// ═══════════════════════════════════════════════════════════════════════════════

/// A single input frame in the replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFrame {
    pub frame: u64,
    pub action: ReplayAction,
}

/// Recorded player action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayAction {
    Attack,
    HeavyAttack,
    Defend,
    Flee,
    Taunt,
    CastSpell(usize),
    UseItem(usize),
    EnterRoom(usize),
    Descend,
    BuyItem(usize),
    BuyHeal,
    CraftOp(usize, usize),  // (item_idx, op_idx)
    AllocateNode(u32),
    SelectBoon(usize),
    PickupItem,
    LearnSpell,
    ScreenTransition(String),
}

/// Complete replay data for one run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayData {
    pub version: u32,
    pub seed: u64,
    pub class: String,
    pub background: String,
    pub difficulty: String,
    pub name: String,
    pub game_mode: String,
    pub frames: Vec<ReplayFrame>,
    /// Config hash to verify determinism.
    pub config_hash: u64,
    /// Final stats for preview.
    pub final_floor: u32,
    pub final_score: u64,
    pub won: bool,
    pub date: String,
}

impl ReplayData {
    pub fn new(seed: u64, class: &str, background: &str, difficulty: &str, name: &str, mode: &str) -> Self {
        Self {
            version: 1,
            seed,
            class: class.to_string(),
            background: background.to_string(),
            difficulty: difficulty.to_string(),
            name: name.to_string(),
            game_mode: mode.to_string(),
            frames: Vec::new(),
            config_hash: 0,
            final_floor: 0,
            final_score: 0,
            won: false,
            date: current_date_string(),
        }
    }

    pub fn record(&mut self, frame: u64, action: ReplayAction) {
        self.frames.push(ReplayFrame { frame, action });
    }

    pub fn finalize(&mut self, floor: u32, score: u64, won: bool) {
        self.final_floor = floor;
        self.final_score = score;
        self.won = won;
    }

    /// Save replay to disk.
    pub fn save(&self) -> Result<PathBuf, String> {
        let dir = replay_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let filename = format!("replay_{}.json", self.seed);
        let path = dir.join(&filename);
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(path)
    }

    /// Load replay from disk.
    pub fn load(seed: u64) -> Option<Self> {
        let path = replay_dir().join(format!("replay_{}.json", seed));
        let data = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Compressed size estimate (for display).
    pub fn estimated_size_bytes(&self) -> usize {
        // ~20 bytes per frame + header
        self.frames.len() * 20 + 200
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REPLAY PLAYBACK
// ═══════════════════════════════════════════════════════════════════════════════

pub struct ReplayPlayer {
    pub data: ReplayData,
    pub playback_frame: u64,
    pub frame_index: usize,    // index into data.frames
    pub speed: f32,            // 0.5, 1.0, 2.0, 4.0
    pub paused: bool,
    pub finished: bool,
}

impl ReplayPlayer {
    pub fn new(data: ReplayData) -> Self {
        Self {
            data,
            playback_frame: 0,
            frame_index: 0,
            speed: 1.0,
            paused: false,
            finished: false,
        }
    }

    /// Advance playback by dt seconds (at 60fps base).
    pub fn tick(&mut self, dt: f32) -> Vec<&ReplayAction> {
        if self.paused || self.finished { return Vec::new(); }

        let frames_to_advance = (dt * 60.0 * self.speed) as u64;
        let target_frame = self.playback_frame + frames_to_advance;

        let mut actions = Vec::new();
        while self.frame_index < self.data.frames.len() {
            let rf = &self.data.frames[self.frame_index];
            if rf.frame <= target_frame {
                actions.push(&rf.action);
                self.frame_index += 1;
            } else {
                break;
            }
        }

        self.playback_frame = target_frame;
        if self.frame_index >= self.data.frames.len() {
            self.finished = true;
        }

        actions
    }

    pub fn toggle_pause(&mut self) { self.paused = !self.paused; }
    pub fn set_speed(&mut self, speed: f32) { self.speed = speed.clamp(0.25, 8.0); }

    /// Jump to a specific floor (find the frame where that floor was entered).
    pub fn jump_to_floor(&mut self, floor: u32) {
        let target_label = format!("floor_{}", floor);
        for (i, rf) in self.data.frames.iter().enumerate() {
            if let ReplayAction::Descend = &rf.action {
                // Approximate — count descends
                self.frame_index = i;
                self.playback_frame = rf.frame;
                self.finished = false;
                return;
            }
        }
    }

    pub fn progress(&self) -> f32 {
        if self.data.frames.is_empty() { return 0.0; }
        self.frame_index as f32 / self.data.frames.len() as f32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GHOST RUN
// ═══════════════════════════════════════════════════════════════════════════════

/// Ghost run state — a semi-transparent replay running alongside the player.
pub struct GhostRun {
    pub player: ReplayPlayer,
    pub enabled: bool,
    pub opacity: f32,  // 0.5 = standard ghost
}

impl GhostRun {
    pub fn new(replay: ReplayData) -> Self {
        Self {
            player: ReplayPlayer::new(replay),
            enabled: true,
            opacity: 0.5,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.enabled {
            self.player.tick(dt);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REPLAY LISTING
// ═══════════════════════════════════════════════════════════════════════════════

/// List available replays.
pub fn list_replays() -> Vec<ReplayPreview> {
    let dir = replay_dir();
    let mut previews = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(data) = std::fs::read_to_string(&path) {
                    if let Ok(replay) = serde_json::from_str::<ReplayData>(&data) {
                        previews.push(ReplayPreview {
                            seed: replay.seed,
                            name: replay.name.clone(),
                            class: replay.class.clone(),
                            floor: replay.final_floor,
                            score: replay.final_score,
                            won: replay.won,
                            date: replay.date.clone(),
                            frame_count: replay.frames.len(),
                        });
                    }
                }
            }
        }
    }
    previews.sort_by(|a, b| b.date.cmp(&a.date));
    previews
}

pub struct ReplayPreview {
    pub seed: u64,
    pub name: String,
    pub class: String,
    pub floor: u32,
    pub score: u64,
    pub won: bool,
    pub date: String,
    pub frame_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn replay_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".chaos_rpg").join("replays")
}

fn current_date_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    format!("{:04}-{:02}-{:02}", year, day_of_year / 30 + 1, day_of_year % 30 + 1)
}
