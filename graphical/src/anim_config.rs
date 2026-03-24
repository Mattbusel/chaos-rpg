// Animation configuration — CHAOS RPG "Every Action Tells a Story"
//
// Loaded from chaos_config.toml if present, otherwise defaults apply.
// Controls animation speed multipliers and skip-flags for every category.

/// Animation speed and skip configuration.
/// All `_speed` values: 1.0 = normal, 2.0 = double speed, 0.5 = half speed.
pub struct AnimConfig {
    /// Global speed multiplier applied to all animations.
    pub animation_speed: f32,
    // Per-category multipliers (stacked on top of global)
    pub combat_speed:       f32,
    pub spell_speed:        f32,
    pub crafting_speed:     f32,
    pub transition_speed:   f32,
    pub achievement_speed:  f32,
    // Skip flags — true = skip the cinematic and go straight to result
    pub skip_boss_entrance:      bool,
    pub skip_floor_transition:   bool,
    pub skip_nemesis_reveal:     bool,
    pub skip_phase_transition:   bool,
    pub skip_achievement_banner: bool,
    // Hold durations in seconds (converted to frames at 30fps)
    pub boss_entrance_hold:    f32,
    pub floor_transition_hold: f32,
    pub death_sequence_hold:   f32,
    pub achievement_banner_hold: f32,
}

impl Default for AnimConfig {
    fn default() -> Self {
        Self {
            animation_speed:    1.0,
            combat_speed:       1.0,
            spell_speed:        1.0,
            crafting_speed:     1.0,
            transition_speed:   1.0,
            achievement_speed:  1.0,
            skip_boss_entrance:      false,
            skip_floor_transition:   false,
            skip_nemesis_reveal:     false,
            skip_phase_transition:   false,
            skip_achievement_banner: false,
            boss_entrance_hold:      3.0,
            floor_transition_hold:   2.5,
            death_sequence_hold:     3.0,
            achievement_banner_hold: 2.0,
        }
    }
}

impl AnimConfig {
    /// Load from chaos_config.toml in the current working dir.
    /// Falls back to defaults silently.
    pub fn load() -> Self {
        let mut cfg = Self::default();
        if let Ok(s) = std::fs::read_to_string("chaos_config.toml") {
            cfg.parse_toml(&s);
        }
        cfg
    }

    fn parse_toml(&mut self, src: &str) {
        // Minimal key=value parser — no dependency on toml crate.
        // Reads lines under [animation] section.
        let mut in_animation = false;
        for raw_line in src.lines() {
            let line = raw_line.trim();
            if line.starts_with('[') {
                in_animation = line == "[animation]";
                continue;
            }
            if !in_animation || line.starts_with('#') { continue; }
            if let Some(eq) = line.find('=') {
                let key = line[..eq].trim();
                let val = line[eq+1..].trim().trim_matches('"');
                match key {
                    "animation_speed"          => { if let Ok(v) = val.parse() { self.animation_speed    = v; } }
                    "combat_animation_speed"   => { if let Ok(v) = val.parse() { self.combat_speed       = v; } }
                    "spell_animation_speed"    => { if let Ok(v) = val.parse() { self.spell_speed        = v; } }
                    "crafting_animation_speed" => { if let Ok(v) = val.parse() { self.crafting_speed     = v; } }
                    "transition_animation_speed"   => { if let Ok(v) = val.parse() { self.transition_speed   = v; } }
                    "achievement_animation_speed"  => { if let Ok(v) = val.parse() { self.achievement_speed  = v; } }
                    "skip_boss_entrance"       => { self.skip_boss_entrance      = val == "true"; }
                    "skip_floor_transition"    => { self.skip_floor_transition   = val == "true"; }
                    "skip_nemesis_reveal"      => { self.skip_nemesis_reveal     = val == "true"; }
                    "skip_phase_transition"    => { self.skip_phase_transition   = val == "true"; }
                    "skip_achievement_banner"  => { self.skip_achievement_banner = val == "true"; }
                    "boss_entrance_hold"       => { if let Ok(v) = val.parse() { self.boss_entrance_hold      = v; } }
                    "floor_transition_hold"    => { if let Ok(v) = val.parse() { self.floor_transition_hold   = v; } }
                    "death_sequence_hold"      => { if let Ok(v) = val.parse() { self.death_sequence_hold     = v; } }
                    "achievement_banner_hold"  => { if let Ok(v) = val.parse() { self.achievement_banner_hold = v; } }
                    _ => {}
                }
            }
        }
    }

    /// Effective speed for combat animations (global × category).
    pub fn effective_combat(&self)      -> f32 { self.animation_speed * self.combat_speed }
    pub fn effective_spell(&self)       -> f32 { self.animation_speed * self.spell_speed }
    pub fn effective_crafting(&self)    -> f32 { self.animation_speed * self.crafting_speed }
    pub fn effective_transition(&self)  -> f32 { self.animation_speed * self.transition_speed }
    pub fn effective_achievement(&self) -> f32 { self.animation_speed * self.achievement_speed }

    /// Convert a duration in seconds to frames at 30fps, adjusted for speed.
    pub fn secs_to_frames(&self, secs: f32, speed: f32) -> u32 {
        ((secs * 30.0) / speed.max(0.1)) as u32
    }

    /// Frames for an animation duration, using effective combat speed.
    pub fn combat_frames(&self, base_frames: u32) -> u32 {
        (base_frames as f32 / self.effective_combat().max(0.1)) as u32 + 1
    }

    pub fn spell_frames(&self, base_frames: u32) -> u32 {
        (base_frames as f32 / self.effective_spell().max(0.1)) as u32 + 1
    }
}
