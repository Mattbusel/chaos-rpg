//! Codex progress — tracks which lore entries the player has unlocked.
//!
//! Stored in chaos_rpg_codex.json alongside the other persistent files.

use crate::lore::codex::{self, CodexCategory, CodexEntry};
use crate::lore::fragments::{self, Fragment};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexProgress {
    /// Set of unlocked codex entry IDs.
    pub unlocked_entries: HashSet<String>,
    /// Set of unlocked fragment IDs (1-indexed).
    pub unlocked_fragments: HashSet<u8>,
    /// Total engine unlock counts per engine name (for the "dominate 50 rolls" condition).
    pub engine_roll_counts: std::collections::HashMap<String, u32>,
}

impl CodexProgress {
    pub fn load() -> Self {
        if let Ok(data) = std::fs::read_to_string(Self::path()) {
            if let Ok(p) = serde_json::from_str::<CodexProgress>(&data) {
                return p;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), json);
        }
    }

    /// Trigger an event and unlock any associated entries/fragments.
    /// Returns the list of newly unlocked entry IDs and fragment IDs.
    pub fn trigger_event(&mut self, event: &str) -> (Vec<String>, Vec<u8>) {
        let mut new_entries: Vec<String> = Vec::new();
        let mut new_fragments: Vec<u8> = Vec::new();

        // Unlock codex entries
        for id in codex::entries_unlocked_by(event) {
            if self.unlocked_entries.insert(id.to_string()) {
                new_entries.push(id.to_string());
            }
        }

        // Unlock fragments
        if let Some(frag_id) = fragments::check_fragment_unlock(event) {
            if self.unlocked_fragments.insert(frag_id) {
                new_fragments.push(frag_id);
                // Check if all 7 non-final fragments are unlocked → unlock fragment 8
                if (1u8..=7).all(|id| self.unlocked_fragments.contains(&id)) {
                    if self.unlocked_fragments.insert(8) {
                        new_fragments.push(8);
                    }
                }
            }
        }

        if !new_entries.is_empty() || !new_fragments.is_empty() {
            self.save();
        }

        (new_entries, new_fragments)
    }

    /// Record an engine appearing in a roll chain and check if codex should unlock.
    pub fn record_engine_roll(&mut self, engine_name: &str) -> bool {
        let count = self
            .engine_roll_counts
            .entry(engine_to_codex_id(engine_name).to_string())
            .or_insert(0);
        *count += 1;
        if *count >= 50 {
            let event_key = format!("engine_{}", engine_to_codex_id(engine_name));
            let (new_e, _) = self.trigger_event(&event_key);
            !new_e.is_empty()
        } else {
            false
        }
    }

    /// Record an EngineLock event.
    pub fn record_engine_lock(&mut self, engine_name: &str) {
        let event_key = format!("engine_{}", engine_to_codex_id(engine_name));
        self.trigger_event(&event_key);
    }

    pub fn is_unlocked(&self, id: &str) -> bool {
        self.unlocked_entries.contains(id)
    }

    pub fn fragment_unlocked(&self, id: u8) -> bool {
        self.unlocked_fragments.contains(&id)
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked_entries.len()
    }

    pub fn total_count(&self) -> usize {
        codex::CODEX_ENTRIES.len()
    }

    pub fn fragment_unlocked_count(&self) -> usize {
        self.unlocked_fragments.len()
    }

    pub fn total_fragment_count(&self) -> usize {
        fragments::FRAGMENTS.len()
    }

    /// Get all unlocked entries for a category, sorted by title.
    pub fn unlocked_in_category(&self, cat: CodexCategory) -> Vec<&'static CodexEntry> {
        let mut entries: Vec<&CodexEntry> = codex::entries_by_category(cat)
            .into_iter()
            .filter(|e| self.unlocked_entries.contains(e.id))
            .collect();
        entries.sort_by_key(|e| e.title);
        entries
    }

    /// Get all locked entries for a category (shows unlock hint).
    pub fn locked_in_category(&self, cat: CodexCategory) -> Vec<&'static CodexEntry> {
        let mut entries: Vec<&CodexEntry> = codex::entries_by_category(cat)
            .into_iter()
            .filter(|e| !self.unlocked_entries.contains(e.id))
            .collect();
        entries.sort_by_key(|e| e.title);
        entries
    }

    /// All unlocked fragments, sorted by ID.
    pub fn unlocked_fragments_sorted(&self) -> Vec<&'static Fragment> {
        let mut frags: Vec<&Fragment> = fragments::FRAGMENTS
            .iter()
            .filter(|f| self.unlocked_fragments.contains(&f.id))
            .collect();
        frags.sort_by_key(|f| f.id);
        frags
    }

    fn path() -> PathBuf {
        let mut p = std::env::current_exe().unwrap_or_default();
        p.pop();
        p.push("chaos_rpg_codex.json");
        p
    }
}

fn engine_to_codex_id(engine_name: &str) -> &'static str {
    match engine_name {
        "Lorenz Attractor" => "lorenz",
        "Fourier Harmonic" => "fourier",
        "Prime Density Sieve" => "prime",
        "Riemann Zeta Partial" => "zeta",
        "Fibonacci Golden Spiral" => "fibonacci",
        "Mandelbrot Escape" => "mandelbrot",
        "Logistic Map" => "logistic",
        "Euler's Totient" => "euler",
        "Collatz Chain" => "collatz",
        "Modular Exp Hash" => "modexp",
        _ => "unknown",
    }
}
