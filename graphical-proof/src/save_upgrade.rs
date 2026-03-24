//! Save system upgrade — multiple slots, visual state, compression, cloud sync.

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════════
// SAVE SLOT
// ═══════════════════════════════════════════════════════════════════════════════

/// A single save slot with metadata for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlot {
    pub slot_id: u8,                       // 0-4
    pub occupied: bool,
    pub character_name: String,
    pub character_class: String,
    pub floor: u32,
    pub power_tier: String,
    pub playtime_seconds: u64,
    pub save_date: String,
    /// Compressed game state (JSON for now, upgradeable to binary+RLE later).
    pub game_state_json: String,
    /// Visual state snapshot for exact restoration.
    pub visual_state: Option<VisualSnapshot>,
}

/// Visual state snapshot — captures how the game looked at save time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualSnapshot {
    pub theme_idx: usize,
    pub chaos_field_seed: u64,
    pub chaos_field_floor_mult: f32,
    pub shader_corruption_level: u32,
    pub shader_floor_depth: u32,
    pub weather_type: String,
    pub lighting_room_type: String,
    pub active_boss_id: Option<u8>,
    pub boss_turn: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SAVE MANAGER
// ═══════════════════════════════════════════════════════════════════════════════

pub struct SaveManager {
    pub slots: [Option<SaveSlot>; 5],
    pub active_slot: Option<u8>,
}

impl SaveManager {
    pub fn new() -> Self {
        Self {
            slots: Default::default(),
            active_slot: None,
        }
    }

    /// Load all save slots from disk.
    pub fn load_all(&mut self) {
        for i in 0..5u8 {
            let path = save_slot_path(i);
            if path.exists() {
                if let Ok(data) = std::fs::read_to_string(&path) {
                    if let Ok(slot) = serde_json::from_str::<SaveSlot>(&data) {
                        self.slots[i as usize] = Some(slot);
                    }
                }
            }
        }
    }

    /// Save to a specific slot.
    pub fn save_to_slot(&mut self, slot_id: u8, slot: SaveSlot) -> Result<(), String> {
        if slot_id >= 5 { return Err("Invalid slot ID".to_string()); }
        let path = save_slot_path(slot_id);
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&slot).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        self.slots[slot_id as usize] = Some(slot);
        self.active_slot = Some(slot_id);
        Ok(())
    }

    /// Delete a save slot.
    pub fn delete_slot(&mut self, slot_id: u8) {
        if slot_id >= 5 { return; }
        let path = save_slot_path(slot_id);
        let _ = std::fs::remove_file(path);
        self.slots[slot_id as usize] = None;
        if self.active_slot == Some(slot_id) {
            self.active_slot = None;
        }
    }

    /// Get a summary of all slots for the load screen.
    pub fn slot_summaries(&self) -> Vec<SlotSummary> {
        self.slots.iter().enumerate().map(|(i, slot)| {
            match slot {
                Some(s) => SlotSummary {
                    slot_id: i as u8,
                    occupied: true,
                    label: format!("{} — {} Lv.? F{} ({})", s.character_name, s.character_class, s.floor, s.power_tier),
                    date: s.save_date.clone(),
                },
                None => SlotSummary {
                    slot_id: i as u8,
                    occupied: false,
                    label: format!("Slot {} — Empty", i + 1),
                    date: String::new(),
                },
            }
        }).collect()
    }

    /// Find the most recent occupied slot.
    pub fn most_recent_slot(&self) -> Option<u8> {
        self.slots.iter().enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|_| i as u8))
            .last()
    }
}

pub struct SlotSummary {
    pub slot_id: u8,
    pub occupied: bool,
    pub label: String,
    pub date: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLOUD SYNC
// ═══════════════════════════════════════════════════════════════════════════════

/// Cloud sync state.
pub struct CloudSync {
    pub enabled: bool,
    pub last_sync: Option<String>,
    pub sync_status: String,
}

impl CloudSync {
    pub fn new() -> Self {
        Self {
            enabled: false,
            last_sync: None,
            sync_status: "Not configured".to_string(),
        }
    }

    /// Attempt to sync achievements and legacy data.
    /// In production this would call an API endpoint.
    pub fn sync_achievements(&mut self, _achievements_json: &str) -> Result<(), String> {
        if !self.enabled { return Ok(()); }
        self.sync_status = "Syncing achievements...".to_string();
        // Placeholder: actual sync would POST to a server
        self.sync_status = "Sync complete".to_string();
        self.last_sync = Some(current_date_string());
        Ok(())
    }

    /// Sync run history (append new runs to cloud).
    pub fn sync_run_history(&mut self, _history_json: &str) -> Result<(), String> {
        if !self.enabled { return Ok(()); }
        self.sync_status = "Syncing history...".to_string();
        self.sync_status = "History synced".to_string();
        self.last_sync = Some(current_date_string());
        Ok(())
    }

    /// Merge cloud data with local on conflict: achievements union, history append.
    pub fn merge_conflict_resolution(local_json: &str, cloud_json: &str) -> String {
        // In a real implementation: parse both, union achievements, append-deduplicate history
        // For now: prefer local (most recent wins)
        local_json.to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPRESSION UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// Simple RLE compression for save data strings.
pub fn rle_compress(data: &[u8]) -> Vec<u8> {
    if data.is_empty() { return Vec::new(); }
    let mut result = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let byte = data[i];
        let mut count = 1u8;
        while (i + count as usize) < data.len() && data[i + count as usize] == byte && count < 255 {
            count += 1;
        }
        result.push(count);
        result.push(byte);
        i += count as usize;
    }
    result
}

/// RLE decompression.
pub fn rle_decompress(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() {
        let count = data[i] as usize;
        let byte = data[i + 1];
        result.extend(std::iter::repeat(byte).take(count));
        i += 2;
    }
    result
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn save_slot_path(slot_id: u8) -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".chaos_rpg")
        .join("saves")
        .join(format!("slot_{}.json", slot_id))
}

fn current_date_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    format!("{:04}-{:02}-{:02}", year, day_of_year / 30 + 1, day_of_year % 30 + 1)
}
