//! Animation bridge — maps chaos-rpg game events to proof-engine's AnimationManager.
//!
//! Owns an `AnimationManager` and translates high-level game actions (player
//! movement, attacks, spell casts, damage taken, deaths) into animation state
//! machine transitions. Returns per-entity glyph transforms each frame that
//! the renderer applies on top of the base entity formations.

use std::collections::HashMap;

use proof_engine::entity::EntityId;
use proof_engine::game::animation::{
    AnimationManager, BossAnimController,
    EnemyAnimState, GlyphTransform, IKTarget,
    PlayerAnimState,
};
use proof_engine::prelude::Vec3;

use crate::state::{AppScreen, GameState};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Entity ID allocation
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Well-known entity ID for the player. Constant across the entire session.
const PLAYER_ENTITY_ID: EntityId = EntityId(1);

/// Base entity ID for enemies. Each enemy in a gauntlet gets sequential IDs.
const ENEMY_ENTITY_BASE: u32 = 1000;

/// Generate an `EntityId` for an enemy at the given index in a gauntlet.
fn enemy_entity_id(index: u32) -> EntityId {
    EntityId(ENEMY_ENTITY_BASE + index)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Boss type mapping
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Boss animation archetypes for chaos-rpg bosses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BossAnimType {
    /// Multi-headed / splitting boss (like the Hydra).
    Hydra,
    /// Panel of judges that vote (like the Committee).
    Committee { judge_count: usize },
    /// Phase-based adaptive boss (like the Algorithm Reborn).
    Algorithm,
}

/// Map a boss name to an animation archetype.
fn boss_name_to_anim_type(name: &str) -> BossAnimType {
    let lower = name.to_ascii_lowercase();
    if lower.contains("hydra") || lower.contains("dragon") || lower.contains("serpent")
        || lower.contains("cerberus") || lower.contains("chimera")
    {
        BossAnimType::Hydra
    } else if lower.contains("committee") || lower.contains("council") || lower.contains("jury")
        || lower.contains("tribunal") || lower.contains("judges")
    {
        BossAnimType::Committee { judge_count: 5 }
    } else if lower.contains("algorithm") || lower.contains("machine") || lower.contains("code")
        || lower.contains("reborn")
    {
        BossAnimType::Algorithm
    } else {
        // Default: hash-based selection for consistency
        let hash: u32 = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        match hash % 3 {
            0 => BossAnimType::Hydra,
            1 => BossAnimType::Committee { judge_count: 3 },
            _ => BossAnimType::Algorithm,
        }
    }
}

/// Create a `BossAnimController` for the given archetype and glyph count.
fn create_boss_controller(anim_type: BossAnimType, glyph_count: usize) -> BossAnimController {
    match anim_type {
        BossAnimType::Hydra => BossAnimController::new_hydra(glyph_count),
        BossAnimType::Committee { judge_count } => {
            BossAnimController::new_committee(glyph_count, judge_count)
        }
        BossAnimType::Algorithm => BossAnimController::new_algorithm(glyph_count),
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Direction helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Compute a direction vector from source to target, defaulting to forward
/// if positions are identical.
fn direction_to(source: Vec3, target: Vec3) -> Vec3 {
    let d = target - source;
    if d.length_squared() < 0.001 {
        Vec3::new(1.0, 0.0, 0.0) // default forward
    } else {
        d.normalize()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Transform output
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A bundle of glyph transforms for a single entity, ready for the renderer.
#[derive(Debug, Clone)]
pub struct EntityTransforms {
    pub entity_id: EntityId,
    pub transforms: Vec<GlyphTransform>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AnimBridge
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Top-level bridge between chaos-rpg game actions and proof-engine's
/// animation state machines. Owns the `AnimationManager` and drives it
/// each frame.
pub struct AnimBridge {
    /// The proof-engine animation manager.
    manager: AnimationManager,

    /// Whether the player entity has been registered.
    player_registered: bool,

    /// Set of currently registered enemy entity IDs.
    registered_enemies: HashMap<u32, EntityId>,

    /// Previous screen — used to detect combat entry/exit.
    prev_screen: AppScreen,

    /// Previous player action type — for idle return detection.
    prev_action_type: u8,

    /// Timer for returning to idle after one-shot animations.
    idle_return_timer: f32,

    /// Whether the player is currently in a one-shot animation.
    player_in_oneshot: bool,

    /// Player position (updated each frame).
    player_pos: Vec3,

    /// Enemy position (updated each frame for the primary enemy).
    enemy_pos: Vec3,

    /// Accumulated time for procedural effects.
    time_acc: f32,
}

impl AnimBridge {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Create a new `AnimBridge` with an empty animation manager.
    pub fn init() -> Self {
        Self {
            manager: AnimationManager::new(),
            player_registered: false,
            registered_enemies: HashMap::new(),
            prev_screen: AppScreen::Title,
            prev_action_type: 0,
            idle_return_timer: 0.0,
            player_in_oneshot: false,
            player_pos: Vec3::new(-5.0, 0.0, 0.0),
            enemy_pos: Vec3::new(5.0, 0.0, 0.0),
            time_acc: 0.0,
        }
    }

    // ── Entity registration ──────────────────────────────────────────────────

    /// Register the player entity for animation. `glyph_count` is the number
    /// of glyphs in the player's AmorphousEntity formation.
    pub fn register_player(&mut self, glyph_count: usize) {
        if self.player_registered {
            // Re-register: remove old and create new
            self.manager.remove_entity(PLAYER_ENTITY_ID);
        }
        self.manager.register_player(PLAYER_ENTITY_ID, glyph_count);
        self.manager
            .entity_positions
            .insert(PLAYER_ENTITY_ID, self.player_pos);
        self.player_registered = true;
    }

    /// Register an enemy entity for animation. Returns the assigned `EntityId`.
    ///
    /// - `index`: position in the gauntlet (0 for single enemies)
    /// - `glyph_count`: number of glyphs in the enemy's formation
    /// - `is_boss`: whether this is a boss entity
    /// - `boss_name`: boss display name (only used when `is_boss` is true)
    pub fn register_enemy(
        &mut self,
        index: u32,
        glyph_count: usize,
        is_boss: bool,
        boss_name: &str,
    ) -> EntityId {
        let id = enemy_entity_id(index);

        // Remove any existing registration for this slot
        if self.registered_enemies.contains_key(&index) {
            self.manager.remove_entity(id);
        }

        if is_boss {
            let anim_type = boss_name_to_anim_type(boss_name);
            let controller = create_boss_controller(anim_type, glyph_count);
            self.manager.register_boss(id, controller);
        } else {
            self.manager.register_enemy(id, glyph_count);
        }

        self.manager
            .entity_positions
            .insert(id, self.enemy_pos);
        self.registered_enemies.insert(index, id);
        id
    }

    /// Unregister an enemy entity (e.g., after death animation completes).
    pub fn unregister_enemy(&mut self, index: u32) {
        if let Some(id) = self.registered_enemies.remove(&index) {
            self.manager.remove_entity(id);
        }
    }

    /// Unregister all enemies (e.g., when leaving combat).
    pub fn unregister_all_enemies(&mut self) {
        let indices: Vec<u32> = self.registered_enemies.keys().copied().collect();
        for idx in indices {
            self.unregister_enemy(idx);
        }
    }

    // ── Player animation events ──────────────────────────────────────────────

    /// Trigger walk/idle blend based on player movement speed.
    /// `speed` is 0.0 for idle, > 0.0 for walking.
    pub fn on_player_move(&mut self, speed: f32) {
        if !self.player_registered || self.player_in_oneshot {
            return;
        }

        let state = if speed > 0.1 {
            PlayerAnimState::Walk
        } else {
            PlayerAnimState::Idle
        };
        self.manager.trigger_player_state(PLAYER_ENTITY_ID, state);
    }

    /// Trigger attack animation.
    /// `is_heavy`: true for heavy/charged attacks.
    /// `target_pos`: world position of the target (for IK aiming).
    pub fn on_player_attack(&mut self, is_heavy: bool, target_pos: Vec3) {
        if !self.player_registered {
            return;
        }

        let state = if is_heavy {
            PlayerAnimState::HeavyAttack
        } else {
            PlayerAnimState::Attack
        };
        self.manager.trigger_player_state(PLAYER_ENTITY_ID, state);

        // Set IK target toward the enemy
        self.manager.set_ik_target(
            PLAYER_ENTITY_ID,
            "weapon_arm",
            IKTarget::Position(target_pos),
        );

        // Schedule return to idle
        let duration = if is_heavy { 1.0 } else { 0.3 };
        self.idle_return_timer = duration;
        self.player_in_oneshot = true;
    }

    /// Trigger spell cast animation.
    /// `mana_cost`: cost of the spell (affects intensity).
    /// `target_pos`: world position of the target.
    pub fn on_player_cast(&mut self, _mana_cost: i64, target_pos: Vec3) {
        if !self.player_registered {
            return;
        }

        self.manager
            .trigger_player_state(PLAYER_ENTITY_ID, PlayerAnimState::Cast);

        // Aim staff/hand toward target
        self.manager.set_ik_target(
            PLAYER_ENTITY_ID,
            "weapon_arm",
            IKTarget::Position(target_pos),
        );

        self.idle_return_timer = 0.75;
        self.player_in_oneshot = true;
    }

    /// Trigger defend animation.
    pub fn on_player_defend(&mut self) {
        if !self.player_registered {
            return;
        }

        self.manager
            .trigger_player_state(PLAYER_ENTITY_ID, PlayerAnimState::Defend);

        self.idle_return_timer = 0.6;
        self.player_in_oneshot = true;
    }

    // ── Entity damage / death events ─────────────────────────────────────────

    /// Trigger hurt animation on an entity.
    /// `entity_index`: 0 for player, 1000+ for enemies (or use the enemy index).
    /// `damage`: amount of damage taken (affects hurt intensity).
    /// `source_dir`: direction the damage came from.
    pub fn on_entity_hurt(&mut self, entity_index: u32, _damage: i64, _source_dir: Vec3) {
        if entity_index == 0 {
            // Player hurt
            if self.player_registered {
                self.manager
                    .trigger_player_state(PLAYER_ENTITY_ID, PlayerAnimState::Hurt);
                self.idle_return_timer = 0.2;
                self.player_in_oneshot = true;
            }
        } else {
            // Enemy hurt
            if let Some(&id) = self.registered_enemies.get(&entity_index) {
                self.manager
                    .trigger_enemy_state(id, EnemyAnimState::Hurt);
            }
        }
    }

    /// Trigger death/dissolution animation on an entity.
    /// `entity_index`: 0 for player, other values for enemies.
    pub fn on_entity_death(&mut self, entity_index: u32) {
        if entity_index == 0 {
            // Player death — trigger a hurt animation (no explicit death state
            // in PlayerAnimState; the renderer handles the cinematic).
            if self.player_registered {
                self.manager
                    .trigger_player_state(PLAYER_ENTITY_ID, PlayerAnimState::Hurt);
            }
        } else {
            // Enemy death dissolution
            if let Some(&id) = self.registered_enemies.get(&entity_index) {
                self.manager
                    .trigger_enemy_state(id, EnemyAnimState::Die);
            }
        }
    }

    // ── Enemy animation events ───────────────────────────────────────────────

    /// Trigger approach animation on an enemy.
    pub fn on_enemy_approach(&mut self, index: u32) {
        if let Some(&id) = self.registered_enemies.get(&index) {
            self.manager
                .trigger_enemy_state(id, EnemyAnimState::Approach);
        }
    }

    /// Trigger attack animation on an enemy.
    pub fn on_enemy_attack(&mut self, index: u32) {
        if let Some(&id) = self.registered_enemies.get(&index) {
            self.manager
                .trigger_enemy_state(id, EnemyAnimState::Attack);
        }
    }

    /// Trigger special animation on an enemy (boss abilities, etc.).
    pub fn on_enemy_special(&mut self, index: u32) {
        if let Some(&id) = self.registered_enemies.get(&index) {
            self.manager
                .trigger_enemy_state(id, EnemyAnimState::Special);
        }
    }

    // ── Per-frame update ─────────────────────────────────────────────────────

    /// Tick all animation controllers and return per-entity glyph transforms.
    /// Must be called every frame.
    pub fn update(&mut self, dt: f32, state: &GameState) -> Vec<EntityTransforms> {
        self.time_acc += dt;

        // ── Auto-detect combat entry/exit ────────────────────────────────────
        if state.screen == AppScreen::Combat && self.prev_screen != AppScreen::Combat {
            self.on_combat_enter(state);
        }
        if state.screen != AppScreen::Combat && self.prev_screen == AppScreen::Combat {
            self.on_combat_exit();
        }
        self.prev_screen = state.screen.clone();

        // ── Auto-detect action changes ───────────────────────────────────────
        if state.last_action_type != self.prev_action_type && state.screen == AppScreen::Combat {
            self.handle_action_change(state);
            self.prev_action_type = state.last_action_type;
        }

        // ── Idle return timer ────────────────────────────────────────────────
        if self.player_in_oneshot {
            self.idle_return_timer -= dt;
            if self.idle_return_timer <= 0.0 {
                self.player_in_oneshot = false;
                if self.player_registered {
                    self.manager
                        .trigger_player_state(PLAYER_ENTITY_ID, PlayerAnimState::Idle);
                    // Relax IK
                    self.manager.set_ik_target(
                        PLAYER_ENTITY_ID,
                        "weapon_arm",
                        IKTarget::None,
                    );
                }
            }
        }

        // ── Update entity positions ──────────────────────────────────────────
        self.manager
            .entity_positions
            .insert(PLAYER_ENTITY_ID, self.player_pos);
        for (&_index, &id) in &self.registered_enemies {
            self.manager
                .entity_positions
                .insert(id, self.enemy_pos);
        }

        // ── Tick the animation manager ───────────────────────────────────────
        self.manager.update(dt);

        // ── Collect transforms ───────────────────────────────────────────────
        let mut results = Vec::new();

        if self.player_registered {
            let transforms = self.manager.get_glyph_transforms(PLAYER_ENTITY_ID);
            if !transforms.is_empty() {
                results.push(EntityTransforms {
                    entity_id: PLAYER_ENTITY_ID,
                    transforms,
                });
            }
        }

        for (&_index, &id) in &self.registered_enemies {
            let transforms = self.manager.get_glyph_transforms(id);
            if !transforms.is_empty() {
                results.push(EntityTransforms {
                    entity_id: id,
                    transforms,
                });
            }
        }

        results
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// Return the player entity ID.
    pub fn player_entity_id(&self) -> EntityId {
        PLAYER_ENTITY_ID
    }

    /// Return whether the player is registered.
    pub fn is_player_registered(&self) -> bool {
        self.player_registered
    }

    /// Return the number of currently registered enemies.
    pub fn enemy_count(&self) -> usize {
        self.registered_enemies.len()
    }

    /// Set the player's world position (used for IK and entity position tracking).
    pub fn set_player_pos(&mut self, pos: Vec3) {
        self.player_pos = pos;
    }

    /// Set the primary enemy's world position.
    pub fn set_enemy_pos(&mut self, pos: Vec3) {
        self.enemy_pos = pos;
    }

    /// Direct access to the underlying manager (for advanced use).
    pub fn manager(&self) -> &AnimationManager {
        &self.manager
    }

    /// Mutable access to the underlying manager.
    pub fn manager_mut(&mut self) -> &mut AnimationManager {
        &mut self.manager
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    /// Called when combat screen is entered. Registers enemy entity if needed.
    fn on_combat_enter(&mut self, state: &GameState) {
        // Register the enemy if one exists
        if let Some(ref enemy) = state.enemy {
            let glyph_count = estimate_enemy_glyph_count(&enemy.tier);
            self.register_enemy(
                0,
                glyph_count,
                state.is_boss_fight,
                &state.boss_entrance_name,
            );

            // Set enemy to idle
            let id = enemy_entity_id(0);
            self.manager
                .trigger_enemy_state(id, EnemyAnimState::Idle);
        }

        // Make player face the enemy
        if self.player_registered {
            let dir = direction_to(self.player_pos, self.enemy_pos);
            self.manager.set_ik_target(
                PLAYER_ENTITY_ID,
                "weapon_arm",
                IKTarget::LookDirection(dir),
            );
        }
    }

    /// Called when leaving combat. Unregisters all enemies.
    fn on_combat_exit(&mut self) {
        self.unregister_all_enemies();
        // Return player to idle
        if self.player_registered {
            self.manager
                .trigger_player_state(PLAYER_ENTITY_ID, PlayerAnimState::Idle);
            self.manager.set_ik_target(
                PLAYER_ENTITY_ID,
                "weapon_arm",
                IKTarget::None,
            );
        }
        self.player_in_oneshot = false;
    }

    /// React to a change in `state.last_action_type`.
    fn handle_action_change(&mut self, state: &GameState) {
        match state.last_action_type {
            1 => {
                // Light attack
                self.on_player_attack(false, self.enemy_pos);
                // Enemy gets hurt (simplified — real damage events come separately)
                self.on_enemy_approach(0);
            }
            2 => {
                // Heavy attack
                self.on_player_attack(true, self.enemy_pos);
            }
            3 => {
                // Spell cast
                let mana_cost = 20; // Approximate; real cost comes from game logic
                self.on_player_cast(mana_cost, self.enemy_pos);
            }
            4 => {
                // Defend
                self.on_player_defend();
            }
            _ => {}
        }
    }
}

impl Default for AnimBridge {
    fn default() -> Self {
        Self::init()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Glyph count estimation
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Estimate the number of glyphs an enemy entity will have based on tier.
/// Must match the logic in `entities::enemies::build_enemy_entity`.
fn estimate_enemy_glyph_count(tier: &chaos_rpg_core::enemy::EnemyTier) -> usize {
    use chaos_rpg_core::enemy::EnemyTier;
    match tier {
        EnemyTier::Minion => 15,
        EnemyTier::Elite => 25,
        EnemyTier::Champion => 35,
        EnemyTier::Boss | EnemyTier::Abomination => 50,
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Transform application
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Apply glyph transforms from the animation bridge to an `AmorphousEntity`'s
/// formation positions. This modifies the entity's visual state in place.
pub fn apply_transforms_to_entity(
    entity: &mut proof_engine::AmorphousEntity,
    transforms: &[GlyphTransform],
) {
    let count = entity.formation.len().min(transforms.len());
    for i in 0..count {
        let t = &transforms[i];
        // Apply position offset
        entity.formation[i] += t.position_offset;
        // Apply scale by scaling the distance from center
        if (t.scale - 1.0).abs() > 0.001 {
            entity.formation[i] *= t.scale;
        }
        // Apply emission to colors (alpha channel as emission proxy)
        if t.emission > 0.0 && i < entity.formation_colors.len() {
            let c = &mut entity.formation_colors[i];
            // Boost brightness proportional to emission
            let boost = 1.0 + t.emission;
            *c = proof_engine::prelude::Vec4::new(
                (c.x * boost).min(1.0),
                (c.y * boost).min(1.0),
                (c.z * boost).min(1.0),
                c.w,
            );
        }
    }
}

/// Apply transforms from a batch of `EntityTransforms` to the engine's
/// entities. This is the main integration point called from the game loop.
pub fn apply_all_transforms(
    results: &[EntityTransforms],
    mut player_entity: Option<&mut proof_engine::AmorphousEntity>,
    mut enemy_entity: Option<&mut proof_engine::AmorphousEntity>,
) {
    for et in results {
        if et.entity_id == PLAYER_ENTITY_ID {
            if let Some(ref mut pe) = player_entity {
                apply_transforms_to_entity(pe, &et.transforms);
            }
        } else {
            // Enemy entity
            if let Some(ref mut ee) = enemy_entity {
                apply_transforms_to_entity(ee, &et.transforms);
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Debug / diagnostics
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Return a diagnostic summary of the animation bridge state.
pub fn debug_summary(bridge: &AnimBridge) -> String {
    let player_state = if bridge.player_registered {
        if bridge.player_in_oneshot {
            format!("oneshot (return in {:.2}s)", bridge.idle_return_timer)
        } else {
            "idle".to_string()
        }
    } else {
        "not registered".to_string()
    };

    format!(
        "AnimBridge: player={}, enemies={}, time={:.1}s",
        player_state,
        bridge.enemy_count(),
        bridge.time_acc,
    )
}

/// Return a human-readable label for a `PlayerAnimState`.
pub fn player_state_label(state: PlayerAnimState) -> &'static str {
    match state {
        PlayerAnimState::Idle => "Idle",
        PlayerAnimState::Walk => "Walk",
        PlayerAnimState::Attack => "Attack",
        PlayerAnimState::HeavyAttack => "Heavy Attack",
        PlayerAnimState::Cast => "Cast",
        PlayerAnimState::Defend => "Defend",
        PlayerAnimState::Hurt => "Hurt",
        PlayerAnimState::Flee => "Flee",
        PlayerAnimState::Channel => "Channel",
        PlayerAnimState::Dodge => "Dodge",
        PlayerAnimState::Interact => "Interact",
    }
}

/// Return a human-readable label for an `EnemyAnimState`.
pub fn enemy_state_label(state: EnemyAnimState) -> &'static str {
    match state {
        EnemyAnimState::Idle => "Idle",
        EnemyAnimState::Approach => "Approach",
        EnemyAnimState::Attack => "Attack",
        EnemyAnimState::Hurt => "Hurt",
        EnemyAnimState::Die => "Die",
        EnemyAnimState::Special => "Special",
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Procedural idle overlay
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Generate a subtle procedural "breathing" overlay that can be added on top
/// of the base animation. Returns per-glyph transforms.
pub fn breathing_overlay(glyph_count: usize, time: f32) -> Vec<GlyphTransform> {
    let mut transforms = Vec::with_capacity(glyph_count);
    for i in 0..glyph_count {
        let phase = i as f32 * 0.3 + time * 1.5;
        let breath = phase.sin() * 0.02;
        let emission = ((time * 2.0 + i as f32 * 0.5).sin() * 0.5 + 0.5) * 0.05;
        transforms.push(GlyphTransform {
            position_offset: Vec3::new(0.0, breath, 0.0),
            scale: 1.0 + breath * 0.5,
            rotation_z: 0.0,
            emission,
        });
    }
    transforms
}

/// Generate a "corruption shimmer" overlay based on corruption level.
pub fn corruption_shimmer(glyph_count: usize, time: f32, corruption: f32) -> Vec<GlyphTransform> {
    if corruption < 0.1 {
        return vec![GlyphTransform::default(); glyph_count];
    }

    let intensity = corruption.clamp(0.0, 1.0);
    let mut transforms = Vec::with_capacity(glyph_count);

    for i in 0..glyph_count {
        let hash = (i as f32 * 7.31 + time * 3.0).sin();
        let offset = if hash > (1.0 - intensity) {
            Vec3::new(
                (time * 5.0 + i as f32).sin() * intensity * 0.1,
                (time * 7.0 + i as f32 * 1.3).cos() * intensity * 0.08,
                0.0,
            )
        } else {
            Vec3::ZERO
        };

        let emission = if hash > (1.0 - intensity * 0.5) {
            intensity * 0.3
        } else {
            0.0
        };

        transforms.push(GlyphTransform {
            position_offset: offset,
            scale: 1.0,
            rotation_z: if intensity > 0.7 {
                hash * intensity * 0.1
            } else {
                0.0
            },
            emission,
        });
    }
    transforms
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boss_name_mapping() {
        assert_eq!(boss_name_to_anim_type("The Hydra"), BossAnimType::Hydra);
        assert_eq!(
            boss_name_to_anim_type("Committee of Chaos"),
            BossAnimType::Committee { judge_count: 5 }
        );
        assert_eq!(
            boss_name_to_anim_type("Algorithm Reborn"),
            BossAnimType::Algorithm
        );
    }

    #[test]
    fn enemy_glyph_estimation() {
        assert_eq!(estimate_enemy_glyph_count(0), 15);
        assert_eq!(estimate_enemy_glyph_count(1), 15);
        assert_eq!(estimate_enemy_glyph_count(3), 25);
        assert_eq!(estimate_enemy_glyph_count(5), 35);
        assert_eq!(estimate_enemy_glyph_count(10), 50);
    }

    #[test]
    fn breathing_overlay_length() {
        let overlay = breathing_overlay(20, 1.0);
        assert_eq!(overlay.len(), 20);
    }

    #[test]
    fn corruption_shimmer_zero() {
        let shimmer = corruption_shimmer(10, 0.0, 0.0);
        assert_eq!(shimmer.len(), 10);
        for t in &shimmer {
            assert_eq!(t.position_offset, Vec3::ZERO);
        }
    }

    #[test]
    fn entity_id_generation() {
        assert_eq!(enemy_entity_id(0), EntityId(1000));
        assert_eq!(enemy_entity_id(5), EntityId(1005));
    }

    #[test]
    fn direction_to_default() {
        let d = direction_to(Vec3::ZERO, Vec3::ZERO);
        assert!((d.x - 1.0).abs() < 0.01, "default direction should be forward");
    }
}
