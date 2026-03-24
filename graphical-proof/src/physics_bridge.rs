//! Physics bridge — wires proof-engine's game physics systems into Chaos RPG.
//!
//! Owns all physics managers (debris, fluids, cloth/rope, arena, weapons) and
//! exposes high-level event methods that the combat system calls. Each event
//! translates game-logic happenings (enemy death, spell cast, weapon swing)
//! into concrete physics spawns and simulations.

use proof_engine::prelude::{Vec2, Vec3};

use proof_engine::game::debris::{
    DebrisPool, DebrisRenderer, DebrisSimulator, DebrisSpawner, DebrisType,
    EntityDeathEvent,
};
use proof_engine::game::fluids::{FluidManager, FluidSpriteData};
use proof_engine::game::cloth_rope::{
    BossCape, ClothId, ClothRopeManager, RopeChain, RopeId,
    SoftBodyId,
};
use proof_engine::game::arena_physics::{
    ArenaPhysicsManager, ArenaRoom, ChaosRiftRoom, DamageEvent, ObjectId,
    RoomType as ArenaRoomType, TrapSystem, TreasureRoom, AABB,
};
use proof_engine::game::weapon_physics::{
    SwingArc, TrailVertex, WeaponPhysicsSystem, WeaponType,
};
use proof_engine::glyph::batch::GlyphInstance;
use proof_engine::combat::Element;

use chaos_rpg_core::world::RoomType as CrpgRoomType;

// ── Render data collected each frame ────────────────────────────────────────

/// All visual output from the physics bridge, ready for the renderer.
pub struct PhysicsRenderData {
    /// Instanced glyph data for debris particles.
    pub debris_instances: Vec<GlyphInstance>,
    /// Point-sprite data for fluid particles.
    pub fluid_sprites: Vec<FluidSpriteData>,
    /// Point-sprite data for fluid pools on the ground.
    pub fluid_pool_sprites: Vec<FluidSpriteData>,
    /// Cloth strip render points (id, positions).
    pub cloth_points: Vec<(ClothId, Vec<[f32; 3]>)>,
    /// Rope chain render points (id, positions).
    pub rope_points: Vec<(RopeId, Vec<Vec3>)>,
    /// Soft body hull points (id, positions).
    pub soft_body_hulls: Vec<(SoftBodyId, Vec<Vec3>)>,
    /// Weapon trail ribbon vertices.
    pub trail_vertices: Vec<TrailVertex>,
    /// Damage events from arena traps this frame.
    pub arena_damage_events: Vec<DamageEvent>,
}

impl Default for PhysicsRenderData {
    fn default() -> Self {
        Self {
            debris_instances: Vec::new(),
            fluid_sprites: Vec::new(),
            fluid_pool_sprites: Vec::new(),
            cloth_points: Vec::new(),
            rope_points: Vec::new(),
            soft_body_hulls: Vec::new(),
            trail_vertices: Vec::new(),
            arena_damage_events: Vec::new(),
        }
    }
}

// ── Element mapping helpers ─────────────────────────────────────────────────

/// Map a spell/damage element string to a `DebrisType`.
fn element_to_debris_type(element: &str) -> DebrisType {
    match element.to_lowercase().as_str() {
        "fire" => DebrisType::Fire,
        "ice" | "frost" | "cold" => DebrisType::Ice,
        "lightning" | "electric" | "shock" => DebrisType::Lightning,
        "poison" | "toxic" => DebrisType::Poison,
        "holy" | "light" | "radiant" => DebrisType::Holy,
        "dark" | "shadow" | "void" | "necrotic" => DebrisType::Dark,
        "bleed" | "blood" => DebrisType::Bleed,
        _ => DebrisType::Normal,
    }
}

/// Map a spell element string to the proof-engine `Element` enum.
fn element_str_to_combat_element(element: &str) -> Element {
    match element.to_lowercase().as_str() {
        "fire" => Element::Fire,
        "ice" | "frost" | "cold" => Element::Ice,
        "lightning" | "electric" | "shock" => Element::Lightning,
        "void" => Element::Void,
        "radiant" | "holy" | "light" => Element::Radiant,
        "shadow" | "dark" | "necrotic" => Element::Shadow,
        "gravity" => Element::Gravity,
        "entropy" | "chaos" => Element::Entropy,
        "temporal" | "time" => Element::Temporal,
        _ => Element::Physical,
    }
}

/// Map a weapon name string to a `WeaponType`.
fn weapon_name_to_type(weapon: &str) -> WeaponType {
    match weapon.to_lowercase().as_str() {
        "sword" | "blade" | "longsword" | "greatsword" => WeaponType::Sword,
        "axe" | "hatchet" | "battleaxe" => WeaponType::Axe,
        "mace" | "hammer" | "warhammer" | "club" => WeaponType::Mace,
        "staff" | "rod" | "wand" => WeaponType::Staff,
        "dagger" | "knife" | "shiv" | "stiletto" => WeaponType::Dagger,
        "spear" | "lance" | "pike" | "javelin" => WeaponType::Spear,
        "bow" | "crossbow" | "shortbow" | "longbow" => WeaponType::Bow,
        "fist" | "gauntlet" | "knuckle" | "claw" => WeaponType::Fist,
        "scythe" | "sickle" | "reaper" => WeaponType::Scythe,
        "whip" | "flail" | "chain" => WeaponType::Whip,
        _ => WeaponType::Sword,
    }
}

/// Convert a chaos-rpg `RoomType` to the arena physics `RoomType`.
fn crpg_room_to_arena(room: &CrpgRoomType) -> ArenaRoomType {
    match room {
        CrpgRoomType::Combat => ArenaRoomType::Normal,
        CrpgRoomType::Trap => ArenaRoomType::Trap,
        CrpgRoomType::Treasure => ArenaRoomType::Treasure,
        CrpgRoomType::Boss => ArenaRoomType::Boss,
        CrpgRoomType::Shop => ArenaRoomType::Shop,
        CrpgRoomType::ChaosRift => ArenaRoomType::ChaosRift,
        _ => ArenaRoomType::Normal,
    }
}

// ── Boss type classification ────────────────────────────────────────────────

/// Boss visual archetype — determines which cloth/rope physics to attach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossArchetype {
    /// Robed/caped boss — gets a flowing cloth cape.
    Caped,
    /// Tentacle/hydra boss — gets rope-based tendrils.
    Tentacled,
    /// Slime/amorphous boss — gets a soft-body blob.
    Amorphous,
    /// Standard boss — no special cloth/rope attachment.
    Standard,
}

/// Classify a boss name/id into a visual archetype.
fn classify_boss(boss_name: &str) -> BossArchetype {
    let lower = boss_name.to_lowercase();
    if lower.contains("lich") || lower.contains("wizard") || lower.contains("sorcerer")
        || lower.contains("mage") || lower.contains("warlock") || lower.contains("vampire")
        || lower.contains("dracula") || lower.contains("reaper") || lower.contains("phantom")
        || lower.contains("wraith")
    {
        BossArchetype::Caped
    } else if lower.contains("hydra") || lower.contains("kraken") || lower.contains("tentacle")
        || lower.contains("cthulhu") || lower.contains("beholder") || lower.contains("squid")
        || lower.contains("serpent") || lower.contains("eldritch")
    {
        BossArchetype::Tentacled
    } else if lower.contains("slime") || lower.contains("ooze") || lower.contains("blob")
        || lower.contains("jelly") || lower.contains("gelatinous") || lower.contains("pudding")
        || lower.contains("amoeba")
    {
        BossArchetype::Amorphous
    } else {
        BossArchetype::Standard
    }
}

// ── Active boss physics state ───────────────────────────────────────────────

/// Tracks the physics objects attached to the currently active boss.
struct ActiveBossPhysics {
    archetype: BossArchetype,
    /// ClothId for the boss cape (if Caped).
    cape_cloth_id: Option<ClothId>,
    /// RopeIds for tendrils (if Tentacled).
    tendril_rope_ids: Vec<RopeId>,
    /// SoftBodyId for the blob (if Amorphous).
    blob_id: Option<SoftBodyId>,
    /// Boss world position.
    position: Vec3,
    /// Boss name (for reference).
    name: String,
}

impl ActiveBossPhysics {
    fn new(name: &str, archetype: BossArchetype, position: Vec3) -> Self {
        Self {
            archetype,
            cape_cloth_id: None,
            tendril_rope_ids: Vec::new(),
            blob_id: None,
            position,
            name: name.to_string(),
        }
    }
}

// ── Room counter for unique IDs ─────────────────────────────────────────────

static ROOM_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

fn next_room_id() -> u32 {
    ROOM_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

// ── PhysicsBridge ───────────────────────────────────────────────────────────

/// Bridges proof-engine's game physics systems into the Chaos RPG game loop.
///
/// Owns debris, fluid, cloth/rope, arena, and weapon physics managers. Exposes
/// high-level event methods called by the combat and exploration systems.
pub struct PhysicsBridge {
    // ── Debris system ──
    pub debris_pool: DebrisPool,
    pub debris_spawner: DebrisSpawner,
    pub debris_simulator: DebrisSimulator,
    pub debris_renderer: DebrisRenderer,

    // ── Fluid system ──
    pub fluid_manager: FluidManager,

    // ── Cloth/rope system ──
    pub cloth_rope_manager: ClothRopeManager,

    // ── Arena physics ──
    pub arena_manager: ArenaPhysicsManager,

    // ── Weapon physics ──
    pub weapon_system: WeaponPhysicsSystem,

    // ── Active boss state ──
    active_boss: Option<ActiveBossPhysics>,

    // ── Cached render data ──
    render_data: PhysicsRenderData,

    // ── Accumulated time ──
    total_time: f32,

    // ── Death corpse positions for necro effects ──
    corpse_positions: Vec<Vec3>,

    // ── Arena room tracking ──
    current_room_id: Option<u32>,
}

impl PhysicsBridge {
    // ════════════════════════════════════════════════════════════════════════
    // Initialization
    // ════════════════════════════════════════════════════════════════════════

    /// Create a new PhysicsBridge with all sub-systems initialized to defaults.
    pub fn init() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);

        Self {
            debris_pool: DebrisPool::new(),
            debris_spawner: DebrisSpawner::new(seed),
            debris_simulator: DebrisSimulator::default(),
            debris_renderer: DebrisRenderer::new(),

            fluid_manager: FluidManager::new(),

            cloth_rope_manager: ClothRopeManager::new(),

            arena_manager: ArenaPhysicsManager::new(),

            weapon_system: WeaponPhysicsSystem::new(WeaponType::Sword),

            active_boss: None,

            render_data: PhysicsRenderData::default(),

            total_time: 0.0,

            corpse_positions: Vec::new(),

            current_room_id: None,
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // Combat events
    // ════════════════════════════════════════════════════════════════════════

    /// Called when an enemy dies. Spawns debris from the enemy's glyph
    /// formation, with element-specific death behaviour.
    ///
    /// * `pos` — world position of the dying enemy
    /// * `glyphs` — the characters that composed the enemy entity
    /// * `colors` — per-glyph RGBA colours
    /// * `element` — element of the killing blow (e.g. "fire", "ice", "normal")
    pub fn on_enemy_death(
        &mut self,
        pos: Vec3,
        glyphs: &[char],
        colors: &[[f32; 4]],
        element: &str,
    ) {
        let death_type = element_to_debris_type(element);

        let event = EntityDeathEvent {
            position: pos,
            glyphs: glyphs.to_vec(),
            colors: colors.to_vec(),
            death_type,
        };

        self.debris_spawner.spawn(&event, &mut self.debris_pool);

        // Element-specific death effects via fluids
        match death_type {
            DebrisType::Fire => {
                self.fluid_manager.spawn_fire_pool(pos, 2.0, 40);
            }
            DebrisType::Ice => {
                self.fluid_manager.spawn_ice_spread(pos, 3.0, 50);
            }
            DebrisType::Poison => {
                self.fluid_manager.spawn_poison_bubbles(pos, 30);
            }
            DebrisType::Holy => {
                self.fluid_manager.spawn_holy_rise(pos, 25);
            }
            DebrisType::Dark => {
                // Dark energy pools on the floor
                self.fluid_manager
                    .spawn_ouroboros_flow(pos, pos + Vec3::new(0.0, -2.0, 0.0), 20);
            }
            DebrisType::Bleed => {
                self.fluid_manager
                    .spawn_bleed(pos, Vec3::new(0.0, -1.0, 0.0), 35);
            }
            DebrisType::Lightning => {
                // Lightning has no fluid; debris scatter handles it
            }
            DebrisType::Normal => {
                // Small blood splat on normal kills
                self.fluid_manager
                    .spawn_bleed(pos, Vec3::new(0.0, -1.0, 0.0), 15);
            }
        }

        // Track corpse position for necromancer effects
        self.corpse_positions.push(pos);
        // Limit stored corpse positions
        if self.corpse_positions.len() > 20 {
            self.corpse_positions.remove(0);
        }
    }

    /// Called on a bleed status tick. Spawns blood fluid dripping from
    /// the entity.
    pub fn on_bleed_tick(&mut self, entity_pos: Vec3) {
        // Drip blood downward from the entity
        let direction = Vec3::new(0.0, -1.0, 0.0);
        self.fluid_manager.spawn_bleed(entity_pos, direction, 8);
    }

    /// Called when a spell is cast. Spawns the appropriate fluid effect
    /// at the target position based on the spell's element.
    ///
    /// * `spell_element` — element string (e.g. "fire", "ice", "poison")
    /// * `target_pos` — world position where the spell lands
    pub fn on_spell_cast(&mut self, spell_element: &str, target_pos: Vec3) {
        match spell_element.to_lowercase().as_str() {
            "fire" => {
                self.fluid_manager.spawn_fire_pool(target_pos, 2.5, 60);
            }
            "ice" | "frost" | "cold" => {
                self.fluid_manager.spawn_ice_spread(target_pos, 3.0, 50);
            }
            "poison" | "toxic" => {
                self.fluid_manager.spawn_poison_bubbles(target_pos, 40);
            }
            "holy" | "light" | "radiant" => {
                self.fluid_manager.spawn_holy_rise(target_pos, 30);
            }
            "dark" | "shadow" | "void" | "necrotic" => {
                // Dark energy crawls outward from impact
                self.fluid_manager.spawn_ouroboros_flow(
                    target_pos,
                    target_pos + Vec3::new(0.0, -1.5, 0.0),
                    35,
                );
            }
            "healing" | "heal" | "restoration" => {
                self.fluid_manager
                    .spawn_healing_fountain(target_pos, 25);
            }
            "necro" | "necromancy" | "undeath" => {
                // Necro energy crawls toward existing corpse positions
                if !self.corpse_positions.is_empty() {
                    self.fluid_manager.spawn_necro_crawl(
                        target_pos,
                        &self.corpse_positions,
                        6,
                    );
                } else {
                    // No corpses — just pool on the floor
                    self.fluid_manager.spawn_ouroboros_flow(
                        target_pos,
                        target_pos + Vec3::new(0.0, -1.0, 0.0),
                        20,
                    );
                }
            }
            _ => {
                // Generic magic — small healing-like fountain with neutral tint
                self.fluid_manager
                    .spawn_healing_fountain(target_pos, 15);
            }
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // Weapon events
    // ════════════════════════════════════════════════════════════════════════

    /// Called when the player begins a weapon swing. Creates a weapon trail
    /// arc from `arc_start` to `arc_end` pivoting at `origin`.
    ///
    /// * `weapon_type` — name of the weapon (e.g. "Sword", "Axe")
    /// * `arc_start` — starting angle in radians
    /// * `arc_end` — ending angle in radians
    /// * `origin` — pivot point (player position)
    pub fn on_weapon_swing(
        &mut self,
        weapon_type: &str,
        arc_start: f32,
        arc_end: f32,
        origin: Vec3,
    ) {
        let wt = weapon_name_to_type(weapon_type);

        // Switch weapon profile if it changed
        self.weapon_system.switch_weapon(wt);

        let arc = SwingArc {
            start_angle: arc_start,
            end_angle: arc_end,
            duration: 0.25 / self.weapon_system.trail.profile.swing_speed,
            elapsed: 0.0,
            origin,
            radius: self.weapon_system.trail.profile.length,
        };

        self.weapon_system.begin_swing(arc);
    }

    /// Called when a weapon connects with an entity. Triggers impact effects,
    /// debris, camera shake, and damage number.
    ///
    /// * `contact_point` — world position of the hit
    /// * `weapon_type` — weapon name string
    /// * `damage` — integer damage dealt
    /// * `is_crit` — whether this was a critical hit
    pub fn on_weapon_impact(
        &mut self,
        contact_point: Vec3,
        weapon_type: &str,
        damage: i64,
        is_crit: bool,
    ) {
        let wt = weapon_name_to_type(weapon_type);
        let profile = proof_engine::game::weapon_physics::WeaponProfiles::get(wt);

        // Compute velocity magnitude from weapon profile
        let velocity_mag = profile.swing_speed * profile.impact_force;

        // Hit direction (from player toward enemy, approximated as +X)
        let hit_dir = Vec3::new(1.0, 0.0, 0.0);

        // Screen position approximation (center-right of screen)
        let screen_pos = Vec2::new(0.6, 0.5);

        self.weapon_system.on_hit(
            contact_point,
            velocity_mag,
            damage as i32,
            is_crit,
            screen_pos,
            hit_dir,
        );

        // Spawn impact debris (small glyph fragments)
        let _debris_count = if is_crit { 8 } else { 4 };
        let debris_type = match profile.element {
            Some(Element::Fire) => DebrisType::Fire,
            Some(Element::Ice) => DebrisType::Ice,
            Some(Element::Lightning) => DebrisType::Lightning,
            Some(Element::Radiant) => DebrisType::Holy,
            Some(Element::Shadow) | Some(Element::Void) => DebrisType::Dark,
            _ => DebrisType::Normal,
        };
        let impact_event = EntityDeathEvent {
            position: contact_point,
            glyphs: vec!['*', '+', '#', '~', '^', '%', '!', '?'],
            colors: vec![[1.0, 0.8, 0.3, 1.0]; 8],
            death_type: debris_type,
        };
        // Only spawn a small burst — not a full death explosion
        let _ = self.debris_spawner.spawn(&impact_event, &mut self.debris_pool);
    }

    /// Update the combo count for weapon trail visual scaling.
    pub fn on_combo_update(&mut self, combo_count: u32) {
        self.weapon_system.update_combo(combo_count);
    }

    /// Handle an attack being blocked.
    pub fn on_block(
        &mut self,
        contact_point: Vec3,
        attacker_direction: Vec3,
        velocity_magnitude: f32,
    ) {
        self.weapon_system
            .on_block(contact_point, attacker_direction, velocity_magnitude);
    }

    /// Handle a perfect parry.
    pub fn on_parry(&mut self, contact_point: Vec3) {
        self.weapon_system.on_parry(contact_point);
    }

    // ════════════════════════════════════════════════════════════════════════
    // Room / exploration events
    // ════════════════════════════════════════════════════════════════════════

    /// Called when the player enters a new room. Initialises arena physics
    /// appropriate to the room type: traps for trap rooms, treasure for
    /// treasure rooms, chaos rift for rift rooms, etc.
    ///
    /// * `room_type` — the chaos-rpg `RoomType` variant
    pub fn on_enter_room(&mut self, room_type: &CrpgRoomType) {
        // Clear previous room state
        self.clear_room();

        let arena_type = crpg_room_to_arena(room_type);
        let room_id = next_room_id();
        self.current_room_id = Some(room_id);

        // Standard arena bounds (20x20 combat arena)
        let bounds = AABB::new(Vec2::new(-10.0, -10.0), Vec2::new(10.0, 10.0));
        let room = ArenaRoom::new(room_id, arena_type, bounds);
        self.arena_manager.add_room(room);

        match room_type {
            CrpgRoomType::Trap => {
                self.setup_trap_room(room_id);
            }
            CrpgRoomType::Treasure => {
                self.setup_treasure_room(room_id);
            }
            CrpgRoomType::ChaosRift => {
                self.setup_chaos_rift_room(room_id);
            }
            CrpgRoomType::Boss => {
                // Boss room setup is handled by `on_boss_spawn`
            }
            _ => {
                // Normal, Shop, Shrine, etc. — no special physics
            }
        }
    }

    /// Set up trap room physics: pendulums, spike pits, flame jets.
    fn setup_trap_room(&mut self, room_id: u32) {
        let traps = TrapSystem::new();
        // Trap details are configured by the arena_physics module;
        // we register an empty system that the arena manager will step.
        // In a full implementation, traps would be procedurally generated
        // based on floor number and seed.
        self.arena_manager.register_trap_system(room_id, traps);
    }

    /// Set up treasure room physics: animated chests, pedestal items.
    fn setup_treasure_room(&mut self, room_id: u32) {
        let treasure = TreasureRoom::new();
        self.arena_manager
            .register_treasure_room(room_id, treasure);
    }

    /// Set up chaos rift room: vortex portal spawning random physics objects.
    fn setup_chaos_rift_room(&mut self, room_id: u32) {
        let rift = ChaosRiftRoom::new(Vec2::ZERO, 3.0);
        self.arena_manager.register_chaos_rift(room_id, rift);
    }

    /// Clear the current room's physics state.
    fn clear_room(&mut self) {
        // Reset arena manager
        self.arena_manager = ArenaPhysicsManager::new();
        self.current_room_id = None;

        // Clear fluids (room-specific pools)
        self.fluid_manager.clear();

        // Debris persists across rooms for visual continuity (fades naturally)
    }

    // ════════════════════════════════════════════════════════════════════════
    // Boss events
    // ════════════════════════════════════════════════════════════════════════

    /// Called when a boss spawns. Creates cloth capes, rope tendrils, or
    /// soft-body blobs depending on the boss archetype.
    ///
    /// * `boss_name` — the boss's display name (used to classify archetype)
    /// * `boss_pos` — world position of the boss entity
    pub fn on_boss_spawn(&mut self, boss_name: &str, boss_pos: Vec3) {
        // Clear any previous boss physics
        self.clear_boss_physics();

        let archetype = classify_boss(boss_name);
        let mut boss_state = ActiveBossPhysics::new(boss_name, archetype, boss_pos);

        match archetype {
            BossArchetype::Caped => {
                // Create a flowing cloth cape: 8 points wide, 12 points tall
                let cape = BossCape::new(boss_pos, 8, 12, 0.3);
                // Add the cape's cloth to the manager
                let cloth = cape.cloth.clone();
                if let Some(id) = self.cloth_rope_manager.add_cloth(cloth) {
                    boss_state.cape_cloth_id = Some(id);
                }
            }
            BossArchetype::Tentacled => {
                // Create 4-6 rope tendrils radiating from the boss
                let tendril_count = 5;
                for i in 0..tendril_count {
                    let angle = (i as f32 / tendril_count as f32)
                        * std::f32::consts::TAU;
                    let length = 4.0;
                    let end_pos = boss_pos
                        + Vec3::new(angle.cos() * length, -1.0, angle.sin() * length);
                    let rope = RopeChain::new(boss_pos, end_pos, 10);
                    if let Some(id) = self.cloth_rope_manager.add_rope(rope) {
                        boss_state.tendril_rope_ids.push(id);
                    }
                }
            }
            BossArchetype::Amorphous => {
                // Create a soft-body blob at the boss position
                let blob =
                    proof_engine::game::cloth_rope::SoftBodyBlob::new(boss_pos, 2.0, 16);
                if let Some(id) = self.cloth_rope_manager.add_soft_body(blob) {
                    boss_state.blob_id = Some(id);
                }
            }
            BossArchetype::Standard => {
                // No special cloth/rope physics for standard bosses
            }
        }

        self.active_boss = Some(boss_state);
    }

    /// Update the boss's world position (call each frame when boss moves).
    pub fn update_boss_position(&mut self, new_pos: Vec3) {
        if let Some(ref mut boss) = self.active_boss {
            boss.position = new_pos;

            match boss.archetype {
                BossArchetype::Caped => {
                    // Update cape anchor points to follow boss
                    if let Some(cloth_id) = boss.cape_cloth_id {
                        if let Some(cloth) = self.cloth_rope_manager.get_cloth_mut(cloth_id) {
                            // Move top-row pinned points to follow boss
                            // The cape was 8 points wide; update pin positions
                            for col in 0..8 {
                                let offset = Vec3::new(col as f32 * 0.3 - 1.05, 0.0, -0.5);
                                cloth.set_point_position(col, new_pos + offset);
                            }
                        }
                    }
                }
                BossArchetype::Tentacled => {
                    // Update tendril start points to follow boss
                    for (_i, rope_id) in boss.tendril_rope_ids.iter().enumerate() {
                        if let Some(rope) =
                            self.cloth_rope_manager.get_rope_mut(*rope_id)
                        {
                            rope.attach_start(new_pos);
                        }
                    }
                }
                BossArchetype::Amorphous => {
                    // Soft body follows boss via gentle force toward new position
                    if let Some(blob_id) = boss.blob_id {
                        if let Some(blob) =
                            self.cloth_rope_manager.get_soft_body_mut(blob_id)
                        {
                            let current = blob.center_position();
                            let delta = new_pos - current;
                            if delta.length_squared() > 0.01 {
                                blob.apply_hit(delta.normalize(), delta.length() * 5.0);
                            }
                        }
                    }
                }
                BossArchetype::Standard => {}
            }
        }
    }

    /// Apply a spell vortex effect on the boss cape (if applicable).
    pub fn apply_boss_vortex(&mut self, origin: Vec3, strength: f32, radius: f32) {
        if let Some(ref boss) = self.active_boss {
            if boss.archetype == BossArchetype::Caped {
                if let Some(cloth_id) = boss.cape_cloth_id {
                    if let Some(cloth) = self.cloth_rope_manager.get_cloth_mut(cloth_id) {
                        // Apply vortex force to all unpinned cloth points
                        for p in &mut cloth.points {
                            if p.pinned {
                                continue;
                            }
                            let to_point = p.position - origin;
                            let dist = to_point.length();
                            if dist < radius && dist > 1e-4 {
                                let falloff = 1.0 - dist / radius;
                                let tangent = Vec3::new(-to_point.y, to_point.x, 0.0)
                                    .normalize_or_zero();
                                p.apply_force(tangent * strength * falloff);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Clear boss-specific physics objects.
    fn clear_boss_physics(&mut self) {
        if let Some(boss) = self.active_boss.take() {
            if let Some(id) = boss.cape_cloth_id {
                self.cloth_rope_manager.remove_cloth(id);
            }
            for id in &boss.tendril_rope_ids {
                self.cloth_rope_manager.remove_rope(*id);
            }
            if let Some(id) = boss.blob_id {
                self.cloth_rope_manager.remove_soft_body(id);
            }
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // Per-frame update
    // ════════════════════════════════════════════════════════════════════════

    /// Step all physics systems forward by `dt` seconds. Call once per frame.
    pub fn update(&mut self, dt: f32) {
        self.total_time += dt;

        // ── Debris ──
        self.debris_simulator.step(dt, &mut self.debris_pool);
        self.debris_pool.reclaim_dead();

        // ── Fluids ──
        self.fluid_manager.update(dt);

        // ── Cloth / Rope / Soft body ──
        self.cloth_rope_manager.step_all(dt);

        // ── Arena ──
        let damage_events = self.arena_manager.step(dt);
        self.render_data.arena_damage_events = damage_events;

        // ── Weapons ──
        self.weapon_system.update(dt);
    }

    // ════════════════════════════════════════════════════════════════════════
    // Render data collection
    // ════════════════════════════════════════════════════════════════════════

    /// Collect all visual data from every physics subsystem into a single
    /// `PhysicsRenderData` struct for the renderer.
    pub fn get_render_data(&mut self) -> &PhysicsRenderData {
        // ── Debris instances ──
        let debris_slice = self.debris_renderer.build_instances(&self.debris_pool);
        self.render_data.debris_instances = debris_slice.to_vec();

        // ── Fluid sprites ──
        self.render_data.fluid_sprites = self
            .fluid_manager
            .renderer
            .extract_sprites(&self.fluid_manager.particles);
        self.render_data.fluid_pool_sprites = self
            .fluid_manager
            .renderer
            .extract_pool_sprites(&self.fluid_manager.pools);

        // ── Cloth / Rope / Soft body ──
        self.render_data.cloth_points = self.cloth_rope_manager.cloth_render_data();
        self.render_data.rope_points = self.cloth_rope_manager.rope_render_data();
        self.render_data.soft_body_hulls = self.cloth_rope_manager.soft_body_render_data();

        // ── Weapon trail ──
        self.render_data.trail_vertices = self.weapon_system.trail_vertices();

        &self.render_data
    }

    // ════════════════════════════════════════════════════════════════════════
    // Utility / query methods
    // ════════════════════════════════════════════════════════════════════════

    /// Query whether the weapon time-scale is slowed (parry effect active).
    pub fn time_scale(&self) -> f32 {
        self.weapon_system.time_scale
    }

    /// Total number of live debris particles.
    pub fn debris_count(&self) -> usize {
        self.debris_pool.alive_count()
    }

    /// Total number of live fluid particles.
    pub fn fluid_particle_count(&self) -> usize {
        self.fluid_manager.particle_count()
    }

    /// Total number of fluid pools on the floor.
    pub fn fluid_pool_count(&self) -> usize {
        self.fluid_manager.pool_count()
    }

    /// Check trap damage for an entity at a given position.
    pub fn check_trap_damage(&self, entity_pos: Vec2, entity_id: u32) -> Vec<DamageEvent> {
        self.arena_manager
            .check_entity_damage(entity_pos, ObjectId(entity_id))
    }

    /// Get the current boss archetype (if any).
    pub fn boss_archetype(&self) -> Option<BossArchetype> {
        self.active_boss.as_ref().map(|b| b.archetype)
    }

    /// Whether any boss physics are active.
    pub fn has_active_boss(&self) -> bool {
        self.active_boss.is_some()
    }

    /// Get corpse positions (for necro effects).
    pub fn corpse_positions(&self) -> &[Vec3] {
        &self.corpse_positions
    }

    /// Clear corpse history (e.g., on floor transition).
    pub fn clear_corpses(&mut self) {
        self.corpse_positions.clear();
    }

    /// Full reset — clears all physics state for a new run.
    pub fn reset(&mut self) {
        self.debris_pool.clear();
        self.fluid_manager.clear();
        self.clear_boss_physics();
        self.cloth_rope_manager = ClothRopeManager::new();
        self.arena_manager = ArenaPhysicsManager::new();
        self.weapon_system = WeaponPhysicsSystem::new(WeaponType::Sword);
        self.corpse_positions.clear();
        self.current_room_id = None;
        self.total_time = 0.0;
        self.render_data = PhysicsRenderData::default();
    }
}
