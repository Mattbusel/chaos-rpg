// CHAOS RPG — chaos_config.toml loader
//
// Place chaos_config.toml next to the executable to override defaults.
// Unknown keys are ignored. Missing keys fall back to defaults.
// Reloaded every time ChaosConfig::load() is called.
//
// Example chaos_config.toml:
//   [display]
//   particle_speed_mult = 1.5
//   kill_linger_frames  = 30
//
//   [gameplay]
//   starting_gold_bonus  = 500
//   difficulty_modifier  = 1.0   # 1.0 = normal, 2.0 = double enemy HP
//   infinite_seed_override = 0   # 0 = random each run
//
//   [leaderboard]
//   url = "https://your-worker.your-name.workers.dev"
//   submit_daily = true
//
//   [meta]
//   player_name = "Anonymous"

use serde::{Deserialize, Serialize};

fn default_one() -> f64 { 1.0 }
fn default_true() -> bool { true }
fn default_max_particles() -> u32 { 2000 }
fn default_field_density() -> f64 { 1.0 }
fn default_url() -> String { "https://chaos-rpg-leaderboard.mfletcherdev.workers.dev".to_string() }
fn default_music_vibe() -> String { "chill".to_string() }
fn default_volume() -> f64 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Music vibe preset: "chill" (default), "classic", "minimal", "off"
    #[serde(default = "default_music_vibe")]
    pub music_vibe: String,
    /// Master music volume multiplier (0.0–2.0, default 1.0)
    #[serde(default = "default_volume")]
    pub music_volume: f64,
    /// Master SFX volume multiplier (0.0–2.0, default 1.0)
    #[serde(default = "default_volume")]
    pub sfx_volume: f64,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self { music_vibe: default_music_vibe(), music_volume: 1.0, sfx_volume: 1.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_one")]
    pub particle_speed_mult: f64,
    #[serde(default)]
    pub kill_linger_frames: u32,   // 0 = use engine default
    #[serde(default)]
    pub fast_mode: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self { particle_speed_mult: 1.0, kill_linger_frames: 0, fast_mode: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayConfig {
    #[serde(default)]
    pub starting_gold_bonus: i64,
    #[serde(default = "default_one")]
    pub difficulty_modifier: f64,
    #[serde(default)]
    pub infinite_seed_override: u64,  // 0 = random
    #[serde(default)]
    pub disable_hunger: bool,
    #[serde(default)]
    pub disable_nemesis: bool,
    #[serde(default)]
    pub disable_corruption: bool,
    #[serde(default)]
    pub extra_inventory_slots: u32,
    #[serde(default)]
    pub xp_multiplier: f64,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            starting_gold_bonus: 0,
            difficulty_modifier: 1.0,
            infinite_seed_override: 0,
            disable_hunger: false,
            disable_nemesis: false,
            disable_corruption: false,
            extra_inventory_slots: 0,
            xp_multiplier: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardConfig {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_true")]
    pub submit_daily: bool,
    #[serde(default = "default_true")]
    pub fetch_on_open: bool,
}

impl Default for LeaderboardConfig {
    fn default() -> Self {
        Self { url: default_url(), submit_daily: true, fetch_on_open: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaConfig {
    #[serde(default)]
    pub player_name: String,
    #[serde(default)]
    pub custom_seed_label: String,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self { player_name: String::new(), custom_seed_label: String::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualsConfig {
    /// Enable the chaos field animated background.
    #[serde(default = "default_true")]
    pub enable_chaos_field: bool,
    /// Enable particle effects (damage numbers, explosions, etc.)
    #[serde(default = "default_true")]
    pub enable_particles: bool,
    /// Enable screen shake on heavy hits.
    #[serde(default = "default_true")]
    pub enable_screen_shake: bool,
    /// Enable HP ghost bars (show recent damage taken).
    #[serde(default = "default_true")]
    pub enable_hp_ghost: bool,
    /// Maximum live particles (reduce to 500 for low-end machines).
    #[serde(default = "default_max_particles")]
    pub max_particles: u32,
    /// Chaos field density multiplier (0.0=off, 0.5=sparse, 1.0=normal, 2.0=dense).
    #[serde(default = "default_field_density")]
    pub chaos_field_density: f64,
    /// Global animation speed multiplier.
    #[serde(default = "default_one")]
    pub animation_speed: f64,
    /// Reduce motion: disables shake, reduces particles 75%, slows animations.
    #[serde(default)]
    pub reduce_motion: bool,
    /// Reduce flashing: caps brightness changes, disables rapid flicker.
    #[serde(default)]
    pub reduce_flashing: bool,
    /// Screen inversion for The Paradox boss fight.
    #[serde(default = "default_true")]
    pub invert_screen_for_paradox: bool,
    /// Eigenstate boss rapid flicker effect.
    #[serde(default = "default_true")]
    pub enable_eigenstate_flicker: bool,
}

impl Default for VisualsConfig {
    fn default() -> Self {
        Self {
            enable_chaos_field: true,
            enable_particles: true,
            enable_screen_shake: true,
            enable_hp_ghost: true,
            max_particles: 2000,
            chaos_field_density: 1.0,
            animation_speed: 1.0,
            reduce_motion: false,
            reduce_flashing: false,
            invert_screen_for_paradox: true,
            enable_eigenstate_flicker: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChaosConfig {
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub gameplay: GameplayConfig,
    #[serde(default)]
    pub leaderboard: LeaderboardConfig,
    #[serde(default)]
    pub meta: MetaConfig,
    #[serde(default)]
    pub visuals: VisualsConfig,
    /// True if a config file was actually found and loaded.
    #[serde(skip)]
    pub loaded_from_file: bool,
}

impl ChaosConfig {
    /// Load from chaos_config.toml next to the executable.
    /// Returns default config if file is absent or malformed.
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(text) = std::fs::read_to_string(&path) {
            match toml::from_str::<ChaosConfig>(&text) {
                Ok(mut cfg) => {
                    cfg.loaded_from_file = true;
                    return cfg;
                }
                Err(e) => {
                    eprintln!("[chaos_config] parse error: {}", e);
                }
            }
        }
        Self::default()
    }

    /// Write out a fully-documented example config next to the exe.
    pub fn write_example() {
        let example = r#"# CHAOS RPG — mod configuration
# Place this file next to chaos-rpg-graphical.exe and edit to taste.
# Restart the game to apply changes.

[audio]
# Music vibe: chill (default) | classic | minimal | off
music_vibe = "chill"
# Master music volume (0.0 = silent, 1.0 = default, 2.0 = double)
music_volume = 1.0
# Master SFX volume (0.0 = silent, 1.0 = default)
sfx_volume = 1.0

[display]
# Multiply particle drift speed (1.0 = default, 2.0 = double speed)
particle_speed_mult = 1.0
# Override kill-linger frame count (0 = use engine default ~45)
kill_linger_frames  = 0
# Halve all visual timings (same as FAST_MODE=1 env var)
fast_mode = false

[gameplay]
# Bonus gold at run start (0 = none)
starting_gold_bonus  = 0
# Scale all enemy HP and damage (1.0 = normal, 2.0 = double)
difficulty_modifier  = 1.0
# Force a specific seed for Infinite mode (0 = random)
infinite_seed_override = 0
# Disable mechanics
disable_hunger    = false
disable_nemesis   = false
disable_corruption = false
# Extra inventory slots (0-20)
extra_inventory_slots = 0
# XP multiplier bonus (0.0 = none, 1.0 = double XP)
xp_multiplier = 0.0

[leaderboard]
# Daily seed leaderboard endpoint
url = "https://chaos-rpg-leaderboard.mfletcherdev.workers.dev"
# Auto-submit your daily seed score after each run
submit_daily = true
# Fetch leaderboard on open
fetch_on_open = true

[meta]
# Override player name shown in leaderboard submissions
player_name = ""

[visuals]
# Animated mathematical background (the Proof computing behind every screen)
enable_chaos_field = true
# Particle effects: damage numbers, explosions, crits, death bursts
enable_particles = true
# Screen shake on heavy hits and boss attacks
enable_screen_shake = true
# Ghost HP bars showing recent damage taken (combat only)
enable_hp_ghost = true
# Maximum live particles (reduce to 500 for low-end machines)
max_particles = 2000
# Chaos field density (0.0=off, 0.5=sparse, 1.0=default, 2.0=dense)
chaos_field_density = 1.0
# Reduce motion: disable shake, cut particles 75%, slow animations
reduce_motion = false
# Reduce flashing: cap brightness spikes, disable rapid flicker (accessibility)
reduce_flashing = false
"#;
        let path = Self::path();
        let _ = std::fs::write(path, example);
    }

    fn path() -> std::path::PathBuf {
        let mut p = std::env::current_exe().unwrap_or_default();
        p.pop();
        p.push("chaos_config.toml");
        p
    }
}
