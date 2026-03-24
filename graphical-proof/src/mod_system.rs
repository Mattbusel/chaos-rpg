//! Scripting/modding integration — mod loader, script hooks, hot-reload.
//!
//! Scans chaos-rpg/mods/ directory for mod packages with mod.toml manifests.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════════════════════
// MOD MANIFEST
// ═══════════════════════════════════════════════════════════════════════════════

/// Mod manifest (from mod.toml).
#[derive(Debug, Clone)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub scripts: Vec<String>,       // script file paths relative to mod root
    pub enabled: bool,
}

impl ModManifest {
    pub fn from_toml(content: &str, mod_dir: &Path) -> Option<Self> {
        // Simple TOML-like parsing (no dependency on toml crate in this context)
        let mut id = String::new();
        let mut name = String::new();
        let mut author = String::new();
        let mut version = String::from("0.1.0");
        let mut description = String::new();
        let mut scripts = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("id = ") {
                id = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("name = ") {
                name = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("author = ") {
                author = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("version = ") {
                version = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("description = ") {
                description = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("scripts = ") {
                // Parse simple array: ["file1.lua", "file2.lua"]
                let stripped = val.trim_matches(|c| c == '[' || c == ']');
                for s in stripped.split(',') {
                    let s = s.trim().trim_matches('"').to_string();
                    if !s.is_empty() { scripts.push(s); }
                }
            }
        }

        if id.is_empty() { return None; }
        Some(Self { id, name, author, version, description, scripts, enabled: true })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCRIPT HOOKS
// ═══════════════════════════════════════════════════════════════════════════════

/// Events that can trigger mod scripts.
#[derive(Debug, Clone)]
pub enum ScriptEvent {
    CombatStart { player_class: String, enemy_name: String, floor: u32 },
    DamageDealt { attacker: String, defender: String, damage: i64, engine_id: String },
    EnemyDeath { enemy_name: String, killing_engine: String, floor: u32 },
    BossPhaseChange { boss_id: u8, phase: u8 },
    FloorEnter { floor_number: u32, epoch: String },
    ItemCraft { item_name: String, operation: String, result: String },
    AchievementUnlock { achievement_id: String, rarity: String },
    CorruptionMilestone { stacks: u32 },
    MiseryMilestone { index: f64 },
    NemesisEncounter { nemesis_name: String },
    PlayerDeath { floor: u32, killer: String },
}

impl ScriptEvent {
    pub fn hook_name(&self) -> &'static str {
        match self {
            ScriptEvent::CombatStart { .. } => "on_combat_start",
            ScriptEvent::DamageDealt { .. } => "on_damage_dealt",
            ScriptEvent::EnemyDeath { .. } => "on_enemy_death",
            ScriptEvent::BossPhaseChange { .. } => "on_boss_phase_change",
            ScriptEvent::FloorEnter { .. } => "on_floor_enter",
            ScriptEvent::ItemCraft { .. } => "on_item_craft",
            ScriptEvent::AchievementUnlock { .. } => "on_achievement_unlock",
            ScriptEvent::CorruptionMilestone { .. } => "on_corruption_milestone",
            ScriptEvent::MiseryMilestone { .. } => "on_misery_milestone",
            ScriptEvent::NemesisEncounter { .. } => "on_nemesis_encounter",
            ScriptEvent::PlayerDeath { .. } => "on_player_death",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MOD LOADER
// ═══════════════════════════════════════════════════════════════════════════════

pub struct ModLoader {
    pub mods_dir: PathBuf,
    pub loaded_mods: Vec<LoadedMod>,
    pub script_log: Vec<String>,
    /// File modification times for hot-reload detection.
    file_mtimes: HashMap<PathBuf, std::time::SystemTime>,
}

pub struct LoadedMod {
    pub manifest: ModManifest,
    pub dir: PathBuf,
    pub script_sources: Vec<(String, String)>, // (filename, source code)
    /// Hook registrations: which hooks this mod listens to.
    pub hooks: Vec<String>,
}

impl ModLoader {
    pub fn new() -> Self {
        let mods_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("mods")))
            .unwrap_or_else(|| PathBuf::from("mods"));

        Self {
            mods_dir,
            loaded_mods: Vec::new(),
            script_log: Vec::new(),
            file_mtimes: HashMap::new(),
        }
    }

    /// Scan mods directory and load all valid mods.
    pub fn scan_and_load(&mut self) {
        self.loaded_mods.clear();
        self.script_log.clear();

        if !self.mods_dir.exists() {
            self.script_log.push(format!("Mods directory not found: {:?}", self.mods_dir));
            return;
        }

        let entries = match std::fs::read_dir(&self.mods_dir) {
            Ok(e) => e,
            Err(e) => {
                self.script_log.push(format!("Failed to read mods dir: {}", e));
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let manifest_path = path.join("mod.toml");
            if !manifest_path.exists() { continue; }

            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(e) => {
                    self.script_log.push(format!("Failed to read {:?}: {}", manifest_path, e));
                    continue;
                }
            };

            let manifest = match ModManifest::from_toml(&content, &path) {
                Some(m) => m,
                None => {
                    self.script_log.push(format!("Invalid mod.toml in {:?}", path));
                    continue;
                }
            };

            // Load script files
            let mut script_sources = Vec::new();
            let mut hooks = Vec::new();
            for script_name in &manifest.scripts {
                let script_path = path.join(script_name);
                match std::fs::read_to_string(&script_path) {
                    Ok(source) => {
                        // Detect hooks by scanning for function names
                        for hook in &[
                            "on_combat_start", "on_damage_dealt", "on_enemy_death",
                            "on_boss_phase_change", "on_floor_enter", "on_item_craft",
                            "on_achievement_unlock", "on_corruption_milestone",
                            "on_misery_milestone", "on_nemesis_encounter", "on_player_death",
                        ] {
                            if source.contains(hook) && !hooks.contains(&hook.to_string()) {
                                hooks.push(hook.to_string());
                            }
                        }
                        // Track mtime for hot-reload
                        if let Ok(meta) = std::fs::metadata(&script_path) {
                            if let Ok(mtime) = meta.modified() {
                                self.file_mtimes.insert(script_path, mtime);
                            }
                        }
                        script_sources.push((script_name.clone(), source));
                    }
                    Err(e) => {
                        self.script_log.push(format!("Failed to load script {}: {}", script_name, e));
                    }
                }
            }

            self.script_log.push(format!(
                "Loaded mod: {} v{} by {} ({} scripts, {} hooks)",
                manifest.name, manifest.version, manifest.author,
                script_sources.len(), hooks.len(),
            ));

            self.loaded_mods.push(LoadedMod {
                manifest,
                dir: path,
                script_sources,
                hooks,
            });
        }

        self.script_log.push(format!("Total mods loaded: {}", self.loaded_mods.len()));
    }

    /// Check for file modifications and reload changed scripts.
    pub fn check_hot_reload(&mut self) -> Vec<String> {
        let mut reloaded = Vec::new();
        for loaded_mod in &mut self.loaded_mods {
            for (script_name, source) in &mut loaded_mod.script_sources {
                let sname = script_name.clone();
                let script_path = loaded_mod.dir.join(&sname);
                let current_mtime = std::fs::metadata(&script_path)
                    .ok()
                    .and_then(|m| m.modified().ok());

                if let Some(current) = current_mtime {
                    let changed = self.file_mtimes.get(&script_path)
                        .map(|prev| current != *prev)
                        .unwrap_or(true);

                    if changed {
                        if let Ok(new_source) = std::fs::read_to_string(&script_path) {
                            *source = new_source;
                            self.file_mtimes.insert(script_path, current);
                            reloaded.push(format!("{}:{}", loaded_mod.manifest.id, sname));
                        }
                    }
                }
            }
        }
        reloaded
    }

    /// Dispatch a script event to all mods that listen for it.
    pub fn dispatch_event(&self, event: &ScriptEvent) -> Vec<String> {
        let hook = event.hook_name();
        let mut results = Vec::new();

        for loaded_mod in &self.loaded_mods {
            if !loaded_mod.manifest.enabled { continue; }
            if !loaded_mod.hooks.contains(&hook.to_string()) { continue; }

            // In a full implementation, this would execute the script in the VM.
            // For now, log that the hook was triggered.
            results.push(format!("[{}] {} triggered", loaded_mod.manifest.id, hook));
        }
        results
    }

    /// Check if any mods are loaded (for MODDED scoreboard tag).
    pub fn has_active_mods(&self) -> bool {
        self.loaded_mods.iter().any(|m| m.manifest.enabled)
    }

    /// Get mod count and names for display.
    pub fn mod_summary(&self) -> String {
        if self.loaded_mods.is_empty() {
            "No mods loaded.".to_string()
        } else {
            let names: Vec<&str> = self.loaded_mods.iter()
                .filter(|m| m.manifest.enabled)
                .map(|m| m.manifest.name.as_str())
                .collect();
            format!("{} mods: {}", names.len(), names.join(", "))
        }
    }
}
