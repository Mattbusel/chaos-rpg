// Audio event definitions for CHAOS RPG.
// Pure type definitions — no playback, no external dependencies.
// Frontends consume AudioEvent queues and route them to the audio backend.

/// All discrete audio events the game can emit.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioEvent {
    // ── Navigation ────────────────────────────────────────────────────────────
    /// Entered a new floor. Carries floor number and seed for deterministic sfx.
    FloorEntered { floor: u32, seed: u64 },
    /// Moved into a new room.
    RoomEntered { room_index: usize },

    // ── Combat ────────────────────────────────────────────────────────────────
    /// Player executed a standard melee attack.
    PlayerAttack,
    /// Player executed a heavy / charged attack.
    PlayerHeavyAttack,
    /// Enemy attacked the player.
    EnemyAttack,
    /// A hit connected and dealt damage to any target.
    DamageDealt { amount: i32, is_crit: bool },
    /// A heal was applied to any target.
    HealApplied { amount: i32 },
    /// Player blocked / defended.
    PlayerDefend,
    /// Player cast a spell. Carries spell index for tonal variety.
    SpellCast { spell_index: usize },
    /// Player successfully fled combat.
    PlayerFled,
    /// An entity died. `is_player` distinguishes player vs enemy deaths.
    EntityDied { is_player: bool },
    /// Player levelled up.
    LevelUp,
    /// Status effect applied (burn, stun, etc.).
    StatusApplied,
    /// Boss fight started. `boss_tier` 1–3 sets intensity.
    BossEncounterStart { boss_tier: u8 },
    /// Three-stage gauntlet started.
    GauntletStart,
    /// One gauntlet stage cleared.
    GauntletStageClear { stage: u8 },

    // ── Math Engine / Chaos ───────────────────────────────────────────────────
    /// The chaos engine fired. `engine_id` 0–9 selects the sonic identity.
    ChaosEngineRoll { engine_id: u8 },
    /// A destiny roll occurred (separate from chaos roll).
    DestinyRoll,
    /// Engine result was a critical — chaotic modifier applied.
    EngineCritical,
    /// Chaos trace displayed — multiple engine layers resolved.
    ChaosCascade { depth: u8 },

    // ── World / Exploration ───────────────────────────────────────────────────
    /// Trap room triggered. `disarmed` true = success.
    TrapTriggered { disarmed: bool },
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
    CraftStart { op_index: usize },
    /// Crafting operation succeeded.
    CraftSuccess,
    /// Crafting operation failed / cursed result.
    CraftFail,
    /// Item volatility reroll triggered.
    ItemVolatilityReroll,

    // ── Skill Checks ──────────────────────────────────────────────────────────
    /// Skill check resolved. `success` indicates outcome.
    SkillCheckResult { success: bool },

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
    BassDrone,
    RhythmPulse,
    MelodicFragment,
    ChaosTexture,
    CorruptionDistortion,
    BossTheme,
    TensionRiser,
    VictoryFanfare,
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
    pub fn for_floor(floor: u32) -> Self {
        match floor {
            0..=10 => Self::Dungeon,
            11..=25 => Self::DeepDungeon,
            26..=49 => Self::Abyss,
            _ => Self::Hunger,
        }
    }
}
