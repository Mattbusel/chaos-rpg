//! Music bridge — maps chaos-rpg game events to proof-engine's MusicDirector.
//!
//! Owns a `MusicDirector` and translates high-level game events (screen
//! changes, combat start/end, floor changes, corruption, boss encounters)
//! into the procedural music system. Returns `AudioVisualEffects` each
//! frame for the renderer to apply (FOV pulse, glow, vignette, etc.).

use std::f32::consts::{PI, TAU};

use proof_engine::game::music::{
    BossMusic, EnemyTier, GameVibe, GameVisuals, MusicDirector,
    RoomType as MusicRoomType,
};
use crate::state::{AppScreen, GameState};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Audio-Visual Effects (returned to the renderer each frame)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Visual effects derived from the current audio state, applied by the
/// renderer on top of the normal scene.
#[derive(Debug, Clone)]
pub struct AudioVisualEffects {
    /// Multiplicative speed factor for chaos-field particles (1.0 = normal).
    pub particle_speed_mult: f32,
    /// Additive camera FOV offset in degrees (positive = wider, negative = narrower).
    pub camera_fov_offset: f32,
    /// Force-field strength multiplier for entity cohesion visual.
    pub force_field_strength: f32,
    /// Per-entity emission pulse intensity (0.0 = no pulse).
    pub entity_emission_pulse: f32,
    /// Screen vignette intensity (0.0 = none, 1.0 = full).
    pub vignette_intensity: f32,
    /// Whether a beat was detected this frame (useful for one-shot VFX).
    pub beat_detected: bool,
    /// Smoothed bass energy [0, 1] — drives low-frequency visual rumble.
    pub bass_energy: f32,
    /// Smoothed high-frequency energy [0, 1] — drives sparkle / shimmer.
    pub high_energy: f32,
}

impl Default for AudioVisualEffects {
    fn default() -> Self {
        Self {
            particle_speed_mult: 1.0,
            camera_fov_offset: 0.0,
            force_field_strength: 0.0,
            entity_emission_pulse: 0.0,
            vignette_intensity: 0.5,
            beat_detected: false,
            bass_energy: 0.0,
            high_energy: 0.0,
        }
    }
}

impl AudioVisualEffects {
    /// Build from the engine's `GameVisuals` struct.
    fn from_game_visuals(v: &GameVisuals) -> Self {
        Self {
            particle_speed_mult: v.chaos_particle_speed_mult,
            camera_fov_offset: v.camera_fov_offset,
            force_field_strength: v.force_field_strength,
            entity_emission_pulse: v.entity_emission_pulse,
            vignette_intensity: v.vignette_intensity,
            beat_detected: false,
            bass_energy: 0.0,
            high_energy: 0.0,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Screen → GameVibe mapping
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Map a chaos-rpg `AppScreen` to a proof-engine `GameVibe`.
fn screen_to_vibe(screen: &AppScreen, is_boss: bool) -> GameVibe {
    match screen {
        // Title / menus
        AppScreen::Title
        | AppScreen::Tutorial
        | AppScreen::ModeSelect
        | AppScreen::CharacterCreation
        | AppScreen::BoonSelect
        | AppScreen::Settings
        | AppScreen::Scoreboard
        | AppScreen::Achievements
        | AppScreen::RunHistory
        | AppScreen::DailyLeaderboard
        | AppScreen::Bestiary
        | AppScreen::Codex => GameVibe::TitleScreen,

        // Exploration / navigation
        AppScreen::FloorNav
        | AppScreen::RoomView
        | AppScreen::CharacterSheet
        | AppScreen::BodyChart
        | AppScreen::PassiveTree => GameVibe::Exploration,

        // Shop
        AppScreen::Shop => GameVibe::Shop,

        // Crafting — uses Shrine vibe (calm, focused)
        AppScreen::Crafting => GameVibe::Shrine,

        // Combat
        AppScreen::Combat => {
            if is_boss {
                GameVibe::Boss
            } else {
                GameVibe::Combat
            }
        }

        // End states
        AppScreen::GameOver => GameVibe::Death,
        AppScreen::Victory => GameVibe::Victory,
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Enemy tier mapping
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Map a chaos-rpg `EnemyTier` to a proof-engine `EnemyTier`.
fn map_enemy_tier(tier: &chaos_rpg_core::enemy::EnemyTier) -> EnemyTier {
    use chaos_rpg_core::enemy::EnemyTier as CoreTier;
    match tier {
        CoreTier::Minion => EnemyTier::Fodder,
        CoreTier::Elite => EnemyTier::Standard,
        CoreTier::Champion => EnemyTier::Elite,
        CoreTier::Boss | CoreTier::Abomination => EnemyTier::MiniBoss,
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Boss name → BossMusic mapping
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Map a boss name string from chaos-rpg to a proof-engine `BossMusic` variant.
fn boss_name_to_music(name: &str) -> BossMusic {
    let lower = name.to_ascii_lowercase();
    if lower.contains("mirror") || lower.contains("reflection") || lower.contains("doppel") {
        BossMusic::Mirror
    } else if lower.contains("null") || lower.contains("void") || lower.contains("nothing") {
        BossMusic::Null
    } else if lower.contains("committee") || lower.contains("council") || lower.contains("jury")
        || lower.contains("tribunal") || lower.contains("judges")
    {
        BossMusic::Committee
    } else if lower.contains("algorithm") || lower.contains("machine") || lower.contains("code")
        || lower.contains("program") || lower.contains("reborn")
    {
        BossMusic::AlgorithmReborn
    } else {
        // Default: choose based on name hash to give each unknown boss a
        // consistent music style.
        let hash: u32 = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        match hash % 4 {
            0 => BossMusic::Mirror,
            1 => BossMusic::Null,
            2 => BossMusic::Committee,
            _ => BossMusic::AlgorithmReborn,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Low HP detection
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Fraction of HP below which the "low HP" vibe activates.
const LOW_HP_THRESHOLD: f32 = 0.25;

/// Check whether the player is currently at low HP.
fn is_player_low_hp(state: &GameState) -> bool {
    state.player.as_ref().map_or(false, |p| {
        let frac = p.current_hp as f32 / p.max_hp.max(1) as f32;
        frac <= LOW_HP_THRESHOLD && frac > 0.0
    })
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MusicBridge
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Top-level bridge between chaos-rpg game state and proof-engine's
/// procedural music system. Owns the `MusicDirector` and drives it from
/// game events.
pub struct MusicBridge {
    /// The proof-engine music director that owns all subsystems.
    director: MusicDirector,

    /// Previous screen — used to detect screen changes.
    prev_screen: AppScreen,

    /// Whether we were in a boss fight last frame.
    prev_boss_fight: bool,

    /// Whether the player was at low HP last frame.
    prev_low_hp: bool,

    /// Previous floor number — used to detect floor changes.
    prev_floor: u32,

    /// Previous corruption level (0..100) — used to detect changes.
    prev_corruption: u32,

    /// Whether combat was active last frame.
    prev_in_combat: bool,

    /// Synthetic audio buffer used when no real audio data is available.
    /// Filled with a simple sine wave derived from the current vibe's BPM
    /// so that the audio-visual bridge has *something* to analyze.
    synth_buffer: Vec<f32>,

    /// Phase accumulator for the synthetic sine wave.
    synth_phase: f32,

    /// Cached visual effects from the last update.
    cached_effects: AudioVisualEffects,

    /// Accumulated time for the synthetic buffer generation.
    time_acc: f32,
}

impl MusicBridge {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Create a new `MusicBridge` in its initial (title screen) state.
    pub fn init() -> Self {
        let director = MusicDirector::new();

        // Pre-allocate a synthetic audio buffer (1024 samples at 48 kHz ≈ 21 ms).
        let synth_buffer = vec![0.0f32; 1024];

        Self {
            director,
            prev_screen: AppScreen::Title,
            prev_boss_fight: false,
            prev_low_hp: false,
            prev_floor: 1,
            prev_corruption: 0,
            prev_in_combat: false,
            synth_buffer,
            synth_phase: 0.0,
            cached_effects: AudioVisualEffects::default(),
            time_acc: 0.0,
        }
    }

    // ── Event handlers (called explicitly by game logic) ─────────────────────

    /// React to a screen change. Maps the chaos-rpg screen enum to a
    /// `GameVibe` and tells the director to transition.
    pub fn on_screen_change(&mut self, screen: &AppScreen, is_boss: bool) {
        let vibe = screen_to_vibe(screen, is_boss);
        if vibe != self.director.current_vibe {
            match vibe {
                GameVibe::Death => self.director.on_player_death(),
                GameVibe::Victory => self.director.on_victory(),
                _ => {
                    // Use on_enter_room for exploration-family vibes when we
                    // can map to a MusicRoomType, otherwise let the director
                    // handle it via its internal transition.
                    let room = match screen {
                        AppScreen::Shop => Some(MusicRoomType::Shop),
                        AppScreen::Crafting => Some(MusicRoomType::Shrine),
                        _ => None,
                    };
                    if let Some(rt) = room {
                        self.director.on_enter_room(rt, self.director.current_floor);
                    }
                    // For vibes not covered by on_enter_room, the per-frame
                    // auto-detect in `update` will catch it via screen_to_vibe.
                }
            }
        }
        self.prev_screen = screen.clone();
    }

    /// Called when combat starts. `enemy_tier` is the chaos-rpg `EnemyTier`.
    pub fn on_combat_start(&mut self, enemy_tier: &chaos_rpg_core::enemy::EnemyTier) {
        let tier = map_enemy_tier(enemy_tier);
        self.director.on_combat_start(tier);
        self.prev_in_combat = true;
    }

    /// Called when combat ends (enemy killed or player fled).
    pub fn on_combat_end(&mut self) {
        self.director.on_combat_end();
        self.prev_in_combat = false;
    }

    /// Called when the player changes floors.
    pub fn on_floor_change(&mut self, floor: u32) {
        self.director.on_floor_change(floor);
        self.prev_floor = floor;
    }

    /// Called when the corruption level changes. `level` is 0..100.
    pub fn on_corruption_change(&mut self, level: u32) {
        self.director.on_corruption_change(level);
        self.prev_corruption = level;
    }

    /// Called when a boss encounter begins. `boss_name` is the display name
    /// of the boss from chaos-rpg-core.
    pub fn on_boss_encounter(&mut self, boss_name: &str) {
        let boss_music = boss_name_to_music(boss_name);
        self.director.on_boss_encounter(boss_music);
        self.prev_boss_fight = true;
    }

    /// Called when the player's HP crosses the low-HP threshold.
    pub fn on_player_low_hp(&mut self, is_low: bool) {
        if is_low && !self.prev_low_hp {
            self.director.on_player_low_hp();
        }
        // When HP recovers above threshold during combat, go back to combat
        // vibe (the director doesn't have an "un-low-hp" event, so we
        // re-trigger combat start to restore the combat vibe).
        if !is_low && self.prev_low_hp && self.prev_in_combat {
            self.director.on_combat_start(EnemyTier::Fodder);
        }
        self.prev_low_hp = is_low;
    }

    // ── Per-frame update ─────────────────────────────────────────────────────

    /// Tick the music director and return audio-visual effects for the
    /// renderer. Must be called every frame.
    pub fn update(&mut self, dt: f32, state: &GameState) -> AudioVisualEffects {
        // ── Auto-detect state changes ────────────────────────────────────────

        // Screen change detection
        if state.screen != self.prev_screen {
            self.on_screen_change(&state.screen, state.is_boss_fight);
        }

        // Boss fight start detection
        if state.is_boss_fight && !self.prev_boss_fight {
            self.on_boss_encounter(&state.boss_entrance_name);
        }
        if !state.is_boss_fight && self.prev_boss_fight {
            self.prev_boss_fight = false;
        }

        // Combat detection (screen-based)
        let in_combat = state.screen == AppScreen::Combat;
        if in_combat && !self.prev_in_combat {
            let default_tier = chaos_rpg_core::enemy::EnemyTier::Minion;
            let tier = state
                .enemy
                .as_ref()
                .map(|e| &e.tier)
                .unwrap_or(&default_tier);
            self.on_combat_start(tier);
        }
        if !in_combat && self.prev_in_combat {
            self.on_combat_end();
        }

        // Floor change detection
        if state.floor_num != self.prev_floor {
            self.on_floor_change(state.floor_num);
        }

        // Corruption change detection (quantized to integer 0..100)
        let corruption = (state.corruption_frac() * 100.0) as u32;
        if corruption != self.prev_corruption {
            self.on_corruption_change(corruption);
        }

        // Low HP detection
        let low_hp = is_player_low_hp(state);
        if low_hp != self.prev_low_hp {
            self.on_player_low_hp(low_hp);
        }

        // ── Generate synthetic audio buffer ──────────────────────────────────
        //
        // The MusicDirector::update expects an audio buffer for its
        // audio-visual bridge analysis. Since we don't have real audio
        // samples in the game loop, we generate a simple synthetic signal
        // that mirrors the current vibe's energy.
        self.generate_synth_buffer(dt);

        // ── Tick the director ────────────────────────────────────────────────
        self.director.update(dt, &self.synth_buffer, 48_000);

        // ── Extract visual effects ───────────────────────────────────────────
        let visuals = self.director.visuals();
        let mut effects = AudioVisualEffects::from_game_visuals(visuals);

        // Augment with combat intensity
        if in_combat {
            effects.vignette_intensity += 0.1;
            if state.is_boss_fight {
                effects.entity_emission_pulse += 0.15;
            }
        }

        // Augment with low-HP urgency
        if low_hp {
            let pulse = (self.time_acc * 4.0 * PI).sin().abs() * 0.2;
            effects.vignette_intensity += pulse;
            effects.camera_fov_offset -= 0.5; // slight tunnel vision
        }

        // Augment with corruption visual degradation
        let corr_frac = state.corruption_frac();
        if corr_frac > 0.3 {
            let intensity = (corr_frac - 0.3) / 0.7; // 0..1 over 0.3..1.0
            effects.particle_speed_mult += intensity * 0.5;
            effects.force_field_strength += intensity * 0.3;
        }

        // Floor depth increases vignette subtly
        let floor_factor = (state.floor_num as f32 / 50.0).min(1.0);
        effects.vignette_intensity += floor_factor * 0.1;

        self.time_acc += dt;
        self.cached_effects = effects.clone();
        effects
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// Return the last computed visual effects without re-ticking.
    pub fn cached_effects(&self) -> &AudioVisualEffects {
        &self.cached_effects
    }

    /// Return the current `GameVibe` for debug display.
    pub fn current_vibe(&self) -> GameVibe {
        self.director.current_vibe
    }

    /// Return the current floor as known by the music director.
    pub fn current_floor(&self) -> u32 {
        self.director.current_floor
    }

    /// Provide direct access to the underlying director (for advanced use).
    pub fn director(&self) -> &MusicDirector {
        &self.director
    }

    /// Provide mutable access to the underlying director.
    pub fn director_mut(&mut self) -> &mut MusicDirector {
        &mut self.director
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    /// Fill `self.synth_buffer` with a synthetic signal that the audio-visual
    /// bridge can analyze. The signal is a mix of:
    ///   - A bass sine wave at ~80 Hz (scaled by vibe energy)
    ///   - A mid sine at ~440 Hz (scaled by vibe energy)
    ///   - Noise component that increases with corruption
    fn generate_synth_buffer(&mut self, dt: f32) {
        let sample_rate = 48_000.0f32;
        let samples = self.synth_buffer.len();

        // Energy profile per vibe
        let (bass_amp, mid_amp, noise_amp) = match self.director.current_vibe {
            GameVibe::TitleScreen => (0.2, 0.1, 0.0),
            GameVibe::Exploration => (0.3, 0.2, 0.02),
            GameVibe::Combat => (0.6, 0.4, 0.05),
            GameVibe::Boss => (0.8, 0.5, 0.1),
            GameVibe::Shop => (0.15, 0.25, 0.0),
            GameVibe::Shrine => (0.1, 0.3, 0.0),
            GameVibe::ChaosRift => (0.5, 0.3, 0.3),
            GameVibe::LowHP => (0.7, 0.2, 0.15),
            GameVibe::Death => (0.3, 0.05, 0.2),
            GameVibe::Victory => (0.4, 0.5, 0.0),
        };

        let bass_freq = 80.0;
        let mid_freq = 440.0;

        // Simple LCG for noise (deterministic, no external dependency)
        let mut noise_state = (self.time_acc * 1000.0) as u32;

        for i in 0..samples {
            let t = self.synth_phase + (i as f32 / sample_rate);
            let bass = bass_amp * (t * bass_freq * TAU).sin();
            let mid = mid_amp * (t * mid_freq * TAU).sin();

            // Simple noise via LCG
            noise_state = noise_state.wrapping_mul(1103515245).wrapping_add(12345);
            let noise_val = ((noise_state >> 16) as f32 / 32768.0) - 1.0;
            let noise = noise_amp * noise_val;

            self.synth_buffer[i] = (bass + mid + noise).clamp(-1.0, 1.0);
        }

        self.synth_phase += samples as f32 / sample_rate;
        // Keep phase from growing unbounded
        if self.synth_phase > 1000.0 {
            self.synth_phase -= 1000.0;
        }
    }
}

impl Default for MusicBridge {
    fn default() -> Self {
        Self::init()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Floor music profile helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Describe the musical character of a floor range for debug/UI display.
pub fn floor_music_description(floor: u32) -> &'static str {
    match floor {
        1..=10 => "Bright major key, sparse arrangement",
        11..=20 => "Minor key, building density",
        21..=30 => "Harmonic minor, dense layers",
        31..=40 => "Dorian mode, aggressive rhythms",
        41..=50 => "Phrygian, heavy corruption influence",
        _ => "Chromatic chaos, maximum density",
    }
}

/// Return a suggested BPM modifier for the given floor depth.
pub fn floor_tempo_modifier(floor: u32) -> f32 {
    match floor {
        1..=10 => 1.0,
        11..=20 => 1.05,
        21..=30 => 1.1,
        31..=40 => 1.15,
        41..=50 => 1.2,
        _ => 1.25,
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Corruption audio helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Map a chaos-rpg corruption fraction (0.0..1.0) to the integer level
/// expected by the music director (0..100).
pub fn corruption_frac_to_level(frac: f32) -> u32 {
    (frac.clamp(0.0, 1.0) * 100.0) as u32
}

/// Describe the audio degradation at a given corruption level.
pub fn corruption_audio_description(level: u32) -> &'static str {
    match level {
        0..=10 => "Clean audio",
        11..=25 => "Subtle bitcrush artifacts",
        26..=50 => "Noticeable distortion and timing drift",
        51..=75 => "Heavy degradation, rhythm instability",
        76..=90 => "Extreme corruption, audio barely coherent",
        _ => "Total chaos — audio fully corrupted",
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Combat music intensity helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Compute a combat intensity factor from the current game state.
/// Returns a value in [0.0, 1.0] that can drive music layer density.
pub fn combat_intensity(state: &GameState) -> f32 {
    if state.screen != AppScreen::Combat {
        return 0.0;
    }

    let mut intensity = 0.3; // Base combat intensity

    // Increase with boss fight
    if state.is_boss_fight {
        intensity += 0.3;
    }

    // Increase when player HP is low
    if let Some(ref p) = state.player {
        let hp_frac = p.current_hp as f32 / p.max_hp.max(1) as f32;
        if hp_frac < 0.5 {
            intensity += (0.5 - hp_frac) * 0.4;
        }
    }

    // Increase when enemy HP is low (about to kill)
    if let Some(ref e) = state.enemy {
        let ehp_frac = e.hp as f32 / e.max_hp.max(1) as f32;
        if ehp_frac < 0.2 {
            intensity += 0.15;
        }
    }

    // Increase with floor depth
    let floor_bonus = (state.floor_num as f32 / 100.0).min(0.15);
    intensity += floor_bonus;

    intensity.clamp(0.0, 1.0)
}

/// Compute a visual "urgency" pulse frequency in Hz based on combat intensity.
pub fn urgency_pulse_hz(intensity: f32) -> f32 {
    // Ramps from 0.5 Hz (calm combat) to 4 Hz (extreme combat)
    0.5 + intensity * 3.5
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Boss music helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Return a human-readable name for the boss music type (for debug UI).
pub fn boss_music_name(boss: BossMusic) -> &'static str {
    match boss {
        BossMusic::Mirror => "Mirror — reversed melody playback",
        BossMusic::Null => "Null — progressive silence",
        BossMusic::Committee => "Committee — 5/4 time signature chaos",
        BossMusic::AlgorithmReborn => "Algorithm Reborn — adaptive phases",
    }
}

/// Return a visual theme hint for the given boss music.
pub fn boss_visual_hint(boss: BossMusic) -> (f32, f32, f32) {
    // (emission_boost, vignette_boost, particle_speed_boost)
    match boss {
        BossMusic::Mirror => (0.2, 0.15, 0.0),
        BossMusic::Null => (0.0, 0.3, -0.3),
        BossMusic::Committee => (0.1, 0.1, 0.2),
        BossMusic::AlgorithmReborn => (0.3, 0.2, 0.4),
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Audio-visual effect application
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Apply `AudioVisualEffects` to the engine's camera and global state.
/// Called from the main game loop after `MusicBridge::update`.
pub fn apply_effects_to_engine(
    effects: &AudioVisualEffects,
    engine: &mut proof_engine::ProofEngine,
) {
    // Beat-driven screen shake (subtle)
    if effects.beat_detected {
        engine.add_trauma(0.02);
    }

    // Bass-driven low-frequency rumble
    if effects.bass_energy > 0.3 {
        let rumble = (effects.bass_energy - 0.3) * 0.015;
        engine.add_trauma(rumble);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Vibe transition smoothing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Crossfade duration recommendations per vibe transition type.
pub fn recommended_crossfade(from: GameVibe, to: GameVibe) -> f32 {
    match (from, to) {
        // Combat transitions should be snappy
        (_, GameVibe::Combat) | (_, GameVibe::Boss) => 0.3,
        // Death is immediate
        (_, GameVibe::Death) => 0.1,
        // Victory fanfare should cut in quickly
        (_, GameVibe::Victory) => 0.2,
        // Low HP overlay blends in
        (_, GameVibe::LowHP) => 0.5,
        // ChaosRift is jarring by design
        (_, GameVibe::ChaosRift) => 0.15,
        // Everything else gets a comfortable crossfade
        _ => 0.75,
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_to_vibe_mapping() {
        assert_eq!(screen_to_vibe(&AppScreen::Title, false), GameVibe::TitleScreen);
        assert_eq!(screen_to_vibe(&AppScreen::Combat, false), GameVibe::Combat);
        assert_eq!(screen_to_vibe(&AppScreen::Combat, true), GameVibe::Boss);
        assert_eq!(screen_to_vibe(&AppScreen::Shop, false), GameVibe::Shop);
        assert_eq!(screen_to_vibe(&AppScreen::GameOver, false), GameVibe::Death);
        assert_eq!(screen_to_vibe(&AppScreen::Victory, false), GameVibe::Victory);
    }

    #[test]
    fn boss_name_mapping() {
        assert_eq!(boss_name_to_music("The Mirror"), BossMusic::Mirror);
        assert_eq!(boss_name_to_music("Null Void"), BossMusic::Null);
        assert_eq!(boss_name_to_music("The Committee of Chaos"), BossMusic::Committee);
        assert_eq!(boss_name_to_music("Algorithm Reborn"), BossMusic::AlgorithmReborn);
    }

    #[test]
    fn enemy_tier_mapping() {
        assert_eq!(map_enemy_tier(0), EnemyTier::Fodder);
        assert_eq!(map_enemy_tier(1), EnemyTier::Standard);
        assert_eq!(map_enemy_tier(2), EnemyTier::Elite);
        assert_eq!(map_enemy_tier(5), EnemyTier::MiniBoss);
    }

    #[test]
    fn corruption_level_conversion() {
        assert_eq!(corruption_frac_to_level(0.0), 0);
        assert_eq!(corruption_frac_to_level(0.5), 50);
        assert_eq!(corruption_frac_to_level(1.0), 100);
        assert_eq!(corruption_frac_to_level(1.5), 100); // clamped
    }

    #[test]
    fn crossfade_recommendations() {
        let combat_fade = recommended_crossfade(GameVibe::Exploration, GameVibe::Combat);
        assert!(combat_fade < 0.5, "combat transitions should be snappy");
        let death_fade = recommended_crossfade(GameVibe::Combat, GameVibe::Death);
        assert!(death_fade < 0.2, "death should be near-instant");
    }
}
