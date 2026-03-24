//! Bridge between proof-engine's BossEncounterManager and chaos-rpg's combat.
//!
//! Translates proof-engine boss encounter lifecycle (phases, mechanics,
//! events) into events the chaos-rpg frontend can process for UI, combat
//! state, and visual effects.

use proof_engine::game::bosses::{
    ArenaMod, BossEncounter, BossEncounterManager, BossEvent, BossLootEntry,
    BossType, CommitteeAction, EraseTarget, MusicType, PhaseTransition,
    PlayerActionType, QuantumForm, RecordedAction, SpecialAbility,
};
use proof_engine::combat::{CombatStats, Element as PeElement};

// ═══════════════════════════════════════════════════════════════════════════════
// Game-facing event types
// ═══════════════════════════════════════════════════════════════════════════════

/// Events the game loop should act on after a boss update tick.
#[derive(Debug, Clone)]
pub enum BossGameEvent {
    /// Boss entered a new phase.
    PhaseChanged {
        phase: u32,
        transition_style: TransitionStyle,
    },
    /// Boss used a special ability.
    SpecialAbility { description: String },
    /// Boss dialogue to show on screen.
    Dialogue(String),
    /// Music should change to match new mood.
    MusicChange(MusicVibe),
    /// Arena-wide visual effect.
    ArenaEffect(ArenaEffectKind),
    /// Raw damage number for the player to process.
    Damage(f32),
    /// Boss was defeated. Contains loot descriptions and XP.
    BossDied {
        loot: Vec<LootDrop>,
        xp: u64,
    },
    /// A UI element was erased by the Null boss.
    UiErased(ErasedElement),
    /// UI elements restored after Null boss death.
    UiRestored(Vec<ErasedElement>),
    /// Hydra head split into two.
    HydraSplit {
        parent_id: u32,
        child_a: u32,
        child_b: u32,
    },
    /// Quantum form collapsed.
    QuantumCollapse(String),
    /// Game rules changed (Ouroboros / ChaosWeaver).
    RulesChanged(String),
    /// Player ability slot locked.
    AbilityLocked(u32),
    /// Committee vote result.
    CommitteeVote(String),
    /// Arena edges consumed (Void Serpent).
    ArenaShrunk {
        direction: String,
        remaining_pct: f32,
    },
    /// Arithmetic puzzle generated (PrimeFactorial).
    PuzzleGenerated(Vec<u32>),
    /// Puzzle solved.
    PuzzleSolved,
    /// Combat log message.
    CombatLog(String),
}

/// Simplified transition animation style for the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionStyle {
    Reorganize,
    Dissolve,
    Split,
    Merge,
    Teleport,
    PowerUp,
}

/// Music mood for the frontend audio system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicVibe {
    Ominous,
    Frenetic,
    Orchestral,
    Glitch,
    Silence,
    Reversed,
    Algorithmic,
    Chaotic,
    Crescendo,
    MinimalDrone,
}

/// Arena effect that the renderer should display.
#[derive(Debug, Clone)]
pub enum ArenaEffectKind {
    ShrinkEdges { rate: u32 },
    HazardTiles { element: String, count: u32 },
    DarkenVision { reduction: f32 },
    InvertControls,
    SlipperyFloor { friction: f32 },
    TeleportTraps { count: u32 },
    None,
}

/// Simplified loot drop.
#[derive(Debug, Clone)]
pub struct LootDrop {
    pub name: String,
    pub quantity: u32,
}

/// Which UI element was erased by the Null boss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErasedElement {
    PlayerBuffs,
    HpBar,
    MiniMap,
    AbilitySlot,
    InventorySlot,
    DamageNumbers,
    BossHpBar,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Player action mapping
// ═══════════════════════════════════════════════════════════════════════════════

/// Actions the player can take during a boss fight, mapped from chaos-rpg
/// combat actions to proof-engine's `PlayerActionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossPlayerAction {
    Attack,
    HeavyAttack,
    Defend,
    Flee,
    Taunt,
    Spell,
    UseItem,
}

impl BossPlayerAction {
    fn to_pe(self) -> PlayerActionType {
        match self {
            BossPlayerAction::Attack => PlayerActionType::Attack,
            BossPlayerAction::HeavyAttack => PlayerActionType::Attack, // mapped to Attack (heavy is a chaos-rpg concept)
            BossPlayerAction::Defend => PlayerActionType::Defend,
            BossPlayerAction::Flee => PlayerActionType::Move,          // mapped to Move
            BossPlayerAction::Taunt => PlayerActionType::Wait,         // mapped to Wait
            BossPlayerAction::Spell => PlayerActionType::UseAbility(0),
            BossPlayerAction::UseItem => PlayerActionType::UseItem,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Boss state snapshot (for UI)
// ═══════════════════════════════════════════════════════════════════════════════

/// Snapshot of boss state for the UI to display.
#[derive(Debug, Clone)]
pub struct BossStateSnapshot {
    /// Boss name.
    pub name: String,
    /// Boss title / subtitle.
    pub title: String,
    /// Current HP as a fraction [0.0, 1.0].
    pub hp_fraction: f32,
    /// Current phase number (1-based).
    pub phase: u32,
    /// Total number of phases.
    pub phase_count: usize,
    /// Turn counter.
    pub turn: u32,
    /// Whether the boss is currently transitioning between phases.
    pub is_transitioning: bool,
    /// Transition progress [0.0, 1.0] (0 when not transitioning).
    pub transition_progress: f32,
    /// Speed multiplier from current phase.
    pub speed_mult: f32,
    /// Damage multiplier from current phase.
    pub damage_mult: f32,
    /// Boss tier (1-5).
    pub tier: u32,
    /// Active boss-specific mechanic descriptions.
    pub active_mechanics: Vec<String>,
    /// Whether the fight is over.
    pub finished: bool,
    /// Boss visual ID (for boss_visuals overlay selection).
    pub visual_id: u8,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Conversion helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn transition_to_bridge(t: PhaseTransition) -> TransitionStyle {
    match t {
        PhaseTransition::GlyphReorganize => TransitionStyle::Reorganize,
        PhaseTransition::Dissolve => TransitionStyle::Dissolve,
        PhaseTransition::Split => TransitionStyle::Split,
        PhaseTransition::Merge => TransitionStyle::Merge,
        PhaseTransition::Teleport => TransitionStyle::Teleport,
        PhaseTransition::PowerUp => TransitionStyle::PowerUp,
    }
}

fn music_to_bridge(m: MusicType) -> MusicVibe {
    match m {
        MusicType::Ominous => MusicVibe::Ominous,
        MusicType::Frenetic => MusicVibe::Frenetic,
        MusicType::Orchestral => MusicVibe::Orchestral,
        MusicType::Glitch => MusicVibe::Glitch,
        MusicType::Silence => MusicVibe::Silence,
        MusicType::Reversed => MusicVibe::Reversed,
        MusicType::Algorithmic => MusicVibe::Algorithmic,
        MusicType::Chaotic => MusicVibe::Chaotic,
        MusicType::Crescendo => MusicVibe::Crescendo,
        MusicType::MinimalDrone => MusicVibe::MinimalDrone,
    }
}

fn arena_mod_to_bridge(a: &ArenaMod) -> ArenaEffectKind {
    match a {
        ArenaMod::ShrinkEdges { rate_per_turn } => ArenaEffectKind::ShrinkEdges {
            rate: *rate_per_turn,
        },
        ArenaMod::HazardTiles { element, count } => ArenaEffectKind::HazardTiles {
            element: format!("{:?}", element),
            count: *count,
        },
        ArenaMod::DarkenVision { radius_reduction } => ArenaEffectKind::DarkenVision {
            reduction: *radius_reduction,
        },
        ArenaMod::InvertControls => ArenaEffectKind::InvertControls,
        ArenaMod::SlipperyFloor { friction } => ArenaEffectKind::SlipperyFloor {
            friction: *friction,
        },
        ArenaMod::TeleportTraps { count } => ArenaEffectKind::TeleportTraps { count: *count },
        ArenaMod::None => ArenaEffectKind::None,
    }
}

fn erase_to_bridge(e: &EraseTarget) -> ErasedElement {
    match e {
        EraseTarget::PlayerBuffs => ErasedElement::PlayerBuffs,
        EraseTarget::HpBar => ErasedElement::HpBar,
        EraseTarget::MiniMap => ErasedElement::MiniMap,
        EraseTarget::AbilitySlot => ErasedElement::AbilitySlot,
        EraseTarget::InventorySlot => ErasedElement::InventorySlot,
        EraseTarget::DamageNumbers => ErasedElement::DamageNumbers,
        EraseTarget::BossHpBar => ErasedElement::BossHpBar,
    }
}

fn loot_to_bridge(entries: &[BossLootEntry]) -> Vec<LootDrop> {
    entries
        .iter()
        .map(|e| LootDrop {
            name: e.item_name.clone(),
            quantity: (e.min_quantity + e.max_quantity) / 2,
        })
        .collect()
}

/// Map a chaos-rpg boss name (from the enemy system) to a proof-engine BossType.
fn name_to_boss_type(boss_name: &str) -> Option<BossType> {
    let lower = boss_name.to_lowercase();
    if lower.contains("mirror") {
        Some(BossType::Mirror)
    } else if lower.contains("null") {
        Some(BossType::Null)
    } else if lower.contains("committee") {
        Some(BossType::Committee)
    } else if lower.contains("hydra") || lower.contains("fibonacci") {
        Some(BossType::FibonacciHydra)
    } else if lower.contains("eigenstate") || lower.contains("quantum") {
        Some(BossType::Eigenstate)
    } else if lower.contains("ouroboros") || lower.contains("serpent that devours") {
        Some(BossType::Ouroboros)
    } else if lower.contains("algorithm") || lower.contains("reborn") {
        Some(BossType::AlgorithmReborn)
    } else if lower.contains("chaos weaver") || lower.contains("weaver") {
        Some(BossType::ChaosWeaver)
    } else if lower.contains("void serpent") || lower.contains("void") {
        Some(BossType::VoidSerpent)
    } else if lower.contains("prime") || lower.contains("factorial") {
        Some(BossType::PrimeFactorial)
    } else {
        None
    }
}

/// Map a BossType to the visual_id used by boss_visuals.rs (1-12 system).
fn boss_type_to_visual_id(bt: BossType) -> u8 {
    match bt {
        BossType::Mirror => 1,
        BossType::FibonacciHydra => 3,
        BossType::Eigenstate => 4,
        BossType::Null => 6,
        BossType::Ouroboros => 7,
        BossType::Committee => 9,
        BossType::AlgorithmReborn => 12,
        BossType::ChaosWeaver => 5,
        BossType::VoidSerpent => 11,
        BossType::PrimeFactorial => 8,
    }
}

/// Map a BossType to a floor-tier-appropriate selection.
pub fn boss_for_floor(floor: u32) -> BossType {
    let tier = match floor {
        0..=5 => 1,
        6..=15 => 2,
        16..=30 => 3,
        31..=50 => 4,
        _ => 5,
    };
    let candidates: Vec<BossType> = BossType::all()
        .iter()
        .copied()
        .filter(|b| b.tier() <= tier)
        .collect();
    if candidates.is_empty() {
        BossType::Mirror
    } else {
        // Deterministic pick based on floor.
        candidates[(floor as usize) % candidates.len()]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BossBridge
// ═══════════════════════════════════════════════════════════════════════════════

/// Bridge between proof-engine's boss encounter system and chaos-rpg combat.
///
/// Owns an active `BossEncounter` and translates its events into
/// `BossGameEvent`s that the frontend can process.
pub struct BossBridge {
    /// The live encounter (if any).
    encounter: Option<BossEncounter>,
    /// Accumulated events from the last update tick.
    pending_events: Vec<BossGameEvent>,
    /// Queued player actions to feed into the next update.
    queued_actions: Vec<RecordedAction>,
    /// Turn counter local to the bridge.
    turn: u32,
}

impl BossBridge {
    /// Create a new, idle boss bridge.
    pub fn new() -> Self {
        Self {
            encounter: None,
            pending_events: Vec::new(),
            queued_actions: Vec::new(),
            turn: 0,
        }
    }

    // ── Encounter lifecycle ──────────────────────────────────────────────────

    /// Start a boss encounter.
    ///
    /// `boss_name` is matched against known boss types. If unrecognised, a
    /// tier-appropriate boss is chosen based on floor number.
    ///
    /// Returns the visual_id for `boss_visuals.rs` overlay selection.
    pub fn start_boss(
        &mut self,
        boss_name: &str,
        floor: u32,
        player_attack: f32,
        player_hp: f32,
        player_level: u32,
    ) -> u8 {
        let boss_type = name_to_boss_type(boss_name).unwrap_or_else(|| boss_for_floor(floor));

        let player_stats = CombatStats {
            attack: player_attack,
            max_hp: player_hp,
            hp: player_hp,
            level: player_level,
            ..CombatStats::default()
        };

        let encounter = BossEncounterManager::start_encounter(boss_type, floor, &player_stats);

        let visual_id = boss_type_to_visual_id(boss_type);
        self.encounter = Some(encounter);
        self.pending_events.clear();
        self.queued_actions.clear();
        self.turn = 0;

        // Push initial dialogue.
        let profile_name = boss_type.name().to_string();
        let profile_title = boss_type.title().to_string();
        self.pending_events.push(BossGameEvent::Dialogue(format!(
            "{} — {}",
            profile_name, profile_title
        )));

        visual_id
    }

    /// Queue a player action for the next update tick.
    pub fn queue_action(&mut self, action: BossPlayerAction) {
        self.turn += 1;
        self.queued_actions.push(RecordedAction {
            action_type: action.to_pe(),
            turn: self.turn,
            damage_dealt: 0.0,
            element: None,
        });
    }

    /// Tick the boss AI and return events for the game to process.
    pub fn update(&mut self, dt: f32) -> Vec<BossGameEvent> {
        let mut result = std::mem::take(&mut self.pending_events);

        if let Some(enc) = &mut self.encounter {
            let actions: Vec<RecordedAction> = std::mem::take(&mut self.queued_actions);
            let pe_events = enc.update(dt, &actions);

            for event in pe_events {
                match event {
                    BossEvent::PhaseChange {
                        new_phase,
                        transition,
                        dialogue,
                    } => {
                        result.push(BossGameEvent::PhaseChanged {
                            phase: new_phase,
                            transition_style: transition_to_bridge(transition),
                        });
                        if !dialogue.is_empty() {
                            result.push(BossGameEvent::Dialogue(dialogue));
                        }
                    }
                    BossEvent::SpecialAbility {
                        ability: _,
                        description,
                    } => {
                        result.push(BossGameEvent::SpecialAbility { description });
                    }
                    BossEvent::Dialogue(text) => {
                        result.push(BossGameEvent::Dialogue(text));
                    }
                    BossEvent::MusicChange(music) => {
                        result.push(BossGameEvent::MusicChange(music_to_bridge(music)));
                    }
                    BossEvent::ArenaModification(arena_mod) => {
                        result.push(BossGameEvent::ArenaEffect(arena_mod_to_bridge(&arena_mod)));
                    }
                    BossEvent::VictoryReward {
                        boss_type: _,
                        loot,
                        xp_reward,
                    } => {
                        result.push(BossGameEvent::BossDied {
                            loot: loot_to_bridge(&loot),
                            xp: xp_reward,
                        });
                    }
                    BossEvent::UiErased(target) => {
                        result.push(BossGameEvent::UiErased(erase_to_bridge(&target)));
                    }
                    BossEvent::UiRestored(targets) => {
                        result.push(BossGameEvent::UiRestored(
                            targets.iter().map(erase_to_bridge).collect(),
                        ));
                    }
                    BossEvent::HydraSplit {
                        parent_id,
                        child_ids,
                    } => {
                        result.push(BossGameEvent::HydraSplit {
                            parent_id,
                            child_a: child_ids.0,
                            child_b: child_ids.1,
                        });
                        result.push(BossGameEvent::CombatLog(format!(
                            "Hydra head {} splits into {} and {}!",
                            parent_id, child_ids.0, child_ids.1
                        )));
                    }
                    BossEvent::QuantumCollapse(form) => {
                        result.push(BossGameEvent::QuantumCollapse(format!("{:?}", form)));
                    }
                    BossEvent::RulesChanged(desc) => {
                        result.push(BossGameEvent::RulesChanged(desc));
                    }
                    BossEvent::AbilityLocked(slot) => {
                        result.push(BossGameEvent::AbilityLocked(slot));
                        result.push(BossGameEvent::CombatLog(format!(
                            "Ability slot {} has been locked!",
                            slot
                        )));
                    }
                    BossEvent::CommitteeVoteResult(action) => {
                        let desc = format!("{:?}", action);
                        result.push(BossGameEvent::CommitteeVote(desc));
                    }
                    BossEvent::ArenaShrunk {
                        direction,
                        remaining_fraction,
                    } => {
                        result.push(BossGameEvent::ArenaShrunk {
                            direction,
                            remaining_pct: remaining_fraction,
                        });
                    }
                    BossEvent::PuzzleGenerated(factors) => {
                        result.push(BossGameEvent::PuzzleGenerated(factors));
                    }
                    BossEvent::PuzzleSolved => {
                        result.push(BossGameEvent::PuzzleSolved);
                    }
                    BossEvent::BossDefeated(bt) => {
                        result.push(BossGameEvent::CombatLog(format!(
                            "{} has been defeated!",
                            bt.name()
                        )));
                    }
                }
            }
        }

        result
    }

    // ── State queries ────────────────────────────────────────────────────────

    /// Get a snapshot of the current boss state for UI rendering.
    pub fn get_boss_state(&self) -> Option<BossStateSnapshot> {
        let enc = self.encounter.as_ref()?;
        let profile = &enc.profile;

        let active_mechanics = profile.special_mechanics.clone();

        Some(BossStateSnapshot {
            name: profile.name.clone(),
            title: profile.title.clone(),
            hp_fraction: enc.entity.hp_frac(),
            phase: enc.phase_controller.current_phase_number(),
            phase_count: enc.phase_controller.phase_count(),
            turn: enc.turn_count,
            is_transitioning: enc.phase_controller.is_transitioning(),
            transition_progress: enc.phase_controller.transition_progress(),
            speed_mult: enc.phase_controller.speed_mult(),
            damage_mult: enc.phase_controller.damage_mult(),
            tier: profile.tier,
            active_mechanics,
            finished: enc.finished,
            visual_id: boss_type_to_visual_id(profile.boss_type),
        })
    }

    /// Whether a boss encounter is currently active.
    pub fn is_active(&self) -> bool {
        self.encounter
            .as_ref()
            .map(|e| !e.finished)
            .unwrap_or(false)
    }

    /// Whether the boss is currently transitioning between phases.
    pub fn is_transitioning(&self) -> bool {
        self.encounter
            .as_ref()
            .map(|e| e.phase_controller.is_transitioning())
            .unwrap_or(false)
    }

    /// Get the boss's current HP fraction (for HP bar rendering).
    pub fn hp_fraction(&self) -> f32 {
        self.encounter
            .as_ref()
            .map(|e| e.entity.hp_frac())
            .unwrap_or(0.0)
    }

    /// Get the current phase number.
    pub fn current_phase(&self) -> u32 {
        self.encounter
            .as_ref()
            .map(|e| e.phase_controller.current_phase_number())
            .unwrap_or(0)
    }

    /// Apply damage to the boss (called after chaos-rpg combat resolution).
    pub fn apply_damage(&mut self, amount: f32) {
        if let Some(enc) = &mut self.encounter {
            enc.entity.hp = (enc.entity.hp - amount).max(0.0);
            enc.damage_log.push(amount);
        }
    }

    /// End the encounter (cleanup).
    pub fn end_encounter(&mut self) {
        self.encounter = None;
        self.pending_events.clear();
        self.queued_actions.clear();
        self.turn = 0;
    }

    /// Boss visual id for the boss_visuals overlay system.
    pub fn visual_id(&self) -> Option<u8> {
        self.encounter
            .as_ref()
            .map(|e| boss_type_to_visual_id(e.profile.boss_type))
    }
}
