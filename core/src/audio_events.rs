// Audio event definitions for CHAOS RPG.
// Pure type definitions — no playback, no external dependencies.
// Frontends consume AudioEvent queues and route them to the audio backend.

/// All discrete audio events the game can emit.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioEvent {
    // ── Navigation ────────────────────────────────────────────────────────────
    /// Entered a new floor. Carries floor number and seed for deterministic sfx.
    FloorEntered {
        /// Floor number.
        floor: u32,
        /// Floor seed.
        seed: u64,
    },
    /// Moved into a new room.
    RoomEntered {
        /// Index of the room on the floor.
        room_index: usize,
    },

    // ── Combat ────────────────────────────────────────────────────────────────
    /// Player executed a standard melee attack.
    PlayerAttack,
    /// Player executed a heavy / charged attack.
    PlayerHeavyAttack,
    /// Enemy attacked the player.
    EnemyAttack,
    /// A hit connected and dealt damage to any target.
    DamageDealt {
        /// Damage dealt.
        amount: i32,
        /// Whether it was a critical hit.
        is_crit: bool,
    },
    /// A heal was applied to any target.
    HealApplied {
        /// HP healed.
        amount: i32,
    },
    /// Player blocked / defended.
    PlayerDefend,
    /// Player cast a spell. Carries spell index for tonal variety.
    SpellCast {
        /// Index of the spell cast.
        spell_index: usize,
    },
    /// Player successfully fled combat.
    PlayerFled,
    /// An entity died. `is_player` distinguishes player vs enemy deaths.
    EntityDied {
        /// True if the player died, false for an enemy.
        is_player: bool,
    },
    /// Player levelled up.
    LevelUp,
    /// Status effect applied (burn, stun, etc.).
    StatusApplied,
    /// Boss fight started. `boss_tier` 1–3 sets intensity.
    BossEncounterStart {
        /// Boss intensity, 1 to 3.
        boss_tier: u8,
    },
    /// Three-stage gauntlet started.
    GauntletStart,
    /// One gauntlet stage cleared.
    GauntletStageClear {
        /// Stage just cleared.
        stage: u8,
    },

    // ── Math Engine / Chaos ───────────────────────────────────────────────────
    /// The chaos engine fired. `engine_id` 0–9 selects the sonic identity.
    ChaosEngineRoll {
        /// Engine that fired, 0 to 9.
        engine_id: u8,
    },
    /// A destiny roll occurred (separate from chaos roll).
    DestinyRoll,
    /// Engine result was a critical — chaotic modifier applied.
    EngineCritical,
    /// Chaos trace displayed — multiple engine layers resolved.
    ChaosCascade {
        /// Number of engine layers resolved.
        depth: u8,
    },

    // ── World / Exploration ───────────────────────────────────────────────────
    /// Trap room triggered. `disarmed` true = success.
    TrapTriggered {
        /// True if the trap was disarmed.
        disarmed: bool,
    },
    /// Shop room entered.
    ShopEntered,
    /// Player purchased an item.
    ItemPurchased,
    /// Shrine room: boon selected.
    BoonSelected,
    /// Rest room: player rested / healed.
    RestTaken,
    /// Mystery room event.
    MysteryRoom,
    /// Cursed floor activated this level.
    CursedFloorActivated,
    /// The Hunger triggered (floor 50+).
    HungerTriggered,
    /// BloodPact boon: HP drained on room entry.
    BloodPactDrain,

    // ── Items / Crafting ──────────────────────────────────────────────────────
    /// An item was picked up.
    ItemPickup,
    /// Crafting operation started.
    CraftStart {
        /// Which crafting operation started.
        op_index: usize,
    },
    /// Crafting operation succeeded.
    CraftSuccess,
    /// Crafting operation failed / cursed result.
    CraftFail,
    /// Item volatility reroll triggered.
    ItemVolatilityReroll,

    // ── Skill Checks ──────────────────────────────────────────────────────────
    /// Skill check resolved. `success` indicates outcome.
    SkillCheckResult {
        /// Whether the check passed.
        success: bool,
    },

    // ── Meta / UI ─────────────────────────────────────────────────────────────
    /// Menu / UI navigation.
    MenuNavigate,
    /// Menu item confirmed / selected.
    MenuConfirm,
    /// Menu / action cancelled.
    MenuCancel,
    /// Game over screen reached.
    GameOver,
    /// Victory screen reached (Story mode cleared).
    Victory,
    /// Daily seed game started.
    DailyStart,
    /// Nemesis spawned on this run.
    NemesisSpawned,
}

// ── Music state ───────────────────────────────────────────────────────────────

/// Music vibe preset — controls which generator and volume profile is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MusicVibe {
    /// Gentle evolving ambient — soft pads, pentatonic notes, builds and breathes.
    #[default]
    Chill,
    /// Original system — unchanged classic audio.
    Classic,
    /// Bass drone only — just a quiet sub-frequency hum, nothing else.
    Minimal,
    /// No music, only SFX.
    Off,
}

impl MusicVibe {
    /// Name shown in the settings menu.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Chill   => "Chill  (Evolving Ambient)",
            Self::Classic => "Classic",
            Self::Minimal => "Minimal (Drone Only)",
            Self::Off     => "Off",
        }
    }
    /// The next vibe in the settings cycle (Chill, Classic, Minimal, Off, then back).
    pub fn cycle(self) -> Self {
        match self {
            Self::Chill   => Self::Classic,
            Self::Classic => Self::Minimal,
            Self::Minimal => Self::Off,
            Self::Off     => Self::Chill,
        }
    }
    /// Parse a vibe name from the config file; unknown names give Chill.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "classic" => Self::Classic,
            "minimal" => Self::Minimal,
            "off"     => Self::Off,
            _         => Self::Chill,
        }
    }
    /// Name used in the config file, such as "chill".
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chill   => "chill",
            Self::Classic => "classic",
            Self::Minimal => "minimal",
            Self::Off     => "off",
        }
    }
}

/// High-level music state that the music system transitions between.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicState {
    /// Main menu / title screen.
    MainMenu,
    /// Exploring dungeon rooms — calm generative ambient.
    Exploration,
    /// Active combat — rhythm + chaos texture layers.
    Combat,
    /// Boss fight — full layer stack with boss theme.
    Boss,
    /// Shop / safe zone — lighter, warmer texture.
    Shop,
    /// Game over stinger + fade.
    GameOver,
    /// Victory fanfare.
    Victory,
    /// Cursed floor variant — dissonant, bitcrushed.
    CursedFloor,
    /// Silence (e.g. loading, transition).
    Silence,
}

/// Individual music layers that can be enabled / disabled independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MusicLayer {
    /// Low sustained bass hum.
    BassDrone,
    /// Rhythmic pulse.
    RhythmPulse,
    /// Short melodic phrases.
    MelodicFragment,
    /// Noisy chaos texture.
    ChaosTexture,
    /// Distortion that grows with corruption.
    CorruptionDistortion,
    /// Boss theme.
    BossTheme,
    /// Rising tension build.
    TensionRiser,
    /// Victory fanfare.
    VictoryFanfare,
    /// Funeral bell for a death.
    DeathKnell,
}

/// Ambient zone type — influences the tonal character of exploration music.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientZone {
    /// Standard dungeon floors 1–10.
    Dungeon,
    /// Deep floors 11–25: heavier bass.
    DeepDungeon,
    /// Abyss floors 26–49: dissonant, unstable.
    Abyss,
    /// Endgame floor 50+: The Hunger zone.
    Hunger,
    /// Boss arena.
    BossArena,
    /// Town / shop safe zone.
    SafeZone,
}

impl AmbientZone {
    /// The ambient zone for a floor number: 0 to 10 Dungeon, 11 to 25 DeepDungeon, 26 to 49 Abyss, 50 and up Hunger.
    pub fn for_floor(floor: u32) -> Self {
        match floor {
            0..=10 => Self::Dungeon,
            11..=25 => Self::DeepDungeon,
            26..=49 => Self::Abyss,
            _ => Self::Hunger,
        }
    }
}
