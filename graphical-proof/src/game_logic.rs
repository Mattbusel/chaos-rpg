//! Core game logic ported from `graphical/src/main.rs`.
//!
//! Every piece of floor generation, room entry, combat resolution, death
//! handling, and descend logic lives here as free functions operating on
//! `GameState`. The screen modules call into these instead of duplicating
//! the logic.

use chaos_rpg_core::{
    character::{Boon, Character},
    chaos_pipeline::{chaos_roll_verbose, destiny_roll},
    combat::CombatState,
    enemy::{generate_enemy, FloorAbility},
    items::Item,
    nemesis::{clear_nemesis, load_nemesis, save_nemesis, NemesisRecord},
    npcs::shop_npc,
    skill_checks::{perform_skill_check, Difficulty as SkillDiff, SkillType},
    spells::Spell,
    world::{generate_floor as core_generate_floor, room_enemy, RoomType},
    bosses::{boss_name, random_unique_boss},
    scoreboard::{save_score, ScoreEntry},
};

use crate::state::{AppScreen, CraftPhase, GameState, GameMode, RoomEvent};

// ─── Helper: push_log ────────────────────────────────────────────────────────

/// Push a message to the combat log, capping at 300 entries.
fn push_log(state: &mut GameState, msg: impl Into<String>) {
    state.combat_log.push(msg.into());
    if state.combat_log.len() > 300 {
        state.combat_log.remove(0);
    }
}

/// Max mana for the current player.
fn max_mana(state: &GameState) -> i64 {
    state
        .player
        .as_ref()
        .map(|p| (p.stats.mana + 50).max(50))
        .unwrap_or(50)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. generate_floor
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate a floor for the current `floor_num`. Handles:
/// - Floor seed advancement
/// - Item volatility every 20 floors
/// - Cursed floor detection every 25 floors
/// - Core floor generation
pub fn generate_floor(state: &mut GameState) {
    // Advance floor seed
    state.floor_seed = state
        .floor_seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(state.floor_num as u64 * 31337);

    // Item volatility: every 20 floors, re-roll a random item
    if state.floor_num > 1 && state.floor_num % 20 == 0 {
        if let Some(ref mut p) = state.player {
            if !p.inventory.is_empty() {
                let vol_idx = (state.floor_seed % p.inventory.len() as u64) as usize;
                let old = p.inventory[vol_idx].name.clone();
                p.inventory[vol_idx] =
                    Item::generate(state.floor_seed.wrapping_add(0x766F6C));
                let new_name = p.inventory[vol_idx].name.clone();
                push_log(
                    state,
                    format!("ITEM VOLATILITY: {} -> {}", old, new_name),
                );
            }
        }
    }

    // Cursed floor: every 25 floors
    state.is_cursed_floor = state.floor_num > 0 && state.floor_num % 25 == 0;
    if state.is_cursed_floor {
        push_log(
            state,
            "CURSED FLOOR! All engine outputs INVERTED this floor.".to_string(),
        );
    }

    // Generate the floor via core
    let fl = core_generate_floor(state.floor_num, state.floor_seed);
    state.floor = Some(fl);

    // Sync player floor number
    if let Some(ref mut p) = state.player {
        p.floor = state.floor_num;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. can_descend
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns true if the player can descend to the next floor.
/// Requires that all rooms are visited OR all combat/boss rooms are beaten.
pub fn can_descend(state: &GameState) -> bool {
    let Some(ref floor) = state.floor else {
        return false;
    };

    // Check if we are at the last room
    let at_end = floor.current_room + 1 >= floor.rooms.len();
    if at_end {
        return true;
    }

    // Alternative: all combat rooms visited
    let all_combat_done = floor
        .rooms
        .iter()
        .filter(|r| matches!(r.room_type, RoomType::Combat | RoomType::Boss))
        .all(|r| r.visited);
    all_combat_done
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. descend
// ═══════════════════════════════════════════════════════════════════════════════

/// Advance to the next floor. Handles:
/// - Victory condition (floor >= max_floor)
/// - Floor number increment
/// - Floor transition timer
/// - Floor generation
/// - rooms_cleared increment
/// - The Hunger mechanic (floor 50+)
pub fn descend(state: &mut GameState) {
    let at_end = state
        .floor
        .as_ref()
        .map(|f| f.current_room + 1 >= f.rooms.len())
        .unwrap_or(true);

    if at_end {
        // Victory check
        if state.floor_num >= state.max_floor {
            state.screen = AppScreen::Victory;
            save_score_now(state);
            return;
        }

        state.floor_num += 1;

        // Floor transition overlay
        state.floor_transition_floor = state.floor_num;
        state.floor_transition_timer = 2.5; // seconds-based in proof-engine

        generate_floor(state);
    } else {
        // Advance to the next room within the floor
        if let Some(ref mut f) = state.floor {
            f.advance();
        }
    }

    // Track rooms cleared
    if let Some(ref mut p) = state.player {
        p.rooms_cleared += 1;
    }

    // The Hunger (floor 50+): if 5+ rooms without a kill, lose max HP
    let hunger_trigger = state
        .player
        .as_ref()
        .map(|p| {
            p.floor >= 50
                && p.rooms_without_kill >= 5
                && state.screen != AppScreen::Combat
        })
        .unwrap_or(false);

    if hunger_trigger {
        let loss = state
            .player
            .as_ref()
            .map(|p| (p.max_hp / 20).max(1))
            .unwrap_or(1);
        if let Some(ref mut p) = state.player {
            p.max_hp = (p.max_hp - loss).max(1);
            if p.current_hp > p.max_hp {
                p.current_hp = p.max_hp;
            }
            p.rooms_without_kill = 0;
        }
        push_log(
            state,
            format!("THE HUNGER: -{} max HP permanently!", loss),
        );

        if state
            .player
            .as_ref()
            .map(|p| !p.is_alive())
            .unwrap_or(false)
        {
            state.screen = AppScreen::GameOver;
            save_score_now(state);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. enter_room  (the big one)
// ═══════════════════════════════════════════════════════════════════════════════

/// Enter the current room. Handles ALL room types:
/// - Combat / Boss (with nemesis, gauntlet, unique boss, stat mirror, etc.)
/// - Treasure
/// - Shop
/// - Shrine
/// - Trap
/// - Portal
/// - Empty
/// - ChaosRift
/// - CraftingBench
pub fn enter_room(state: &mut GameState) {
    let floor_num = state.floor_num;
    let room_seed = state.floor_seed.wrapping_add(
        state
            .floor
            .as_ref()
            .map(|f| f.current_room as u64 * 9973)
            .unwrap_or(0),
    );

    // BloodPact boon: take 2 HP each room
    let has_blood_pact = state
        .player
        .as_ref()
        .map(|p| matches!(p.boon, Some(Boon::BloodPact)))
        .unwrap_or(false);
    if has_blood_pact {
        if let Some(ref mut p) = state.player {
            p.take_damage(2);
        }
        push_log(state, "Blood Pact: -2 HP".to_string());
        if state
            .player
            .as_ref()
            .map(|p| !p.is_alive())
            .unwrap_or(false)
        {
            state.screen = AppScreen::GameOver;
            save_score_now(state);
            return;
        }
    }

    let room_type = state
        .floor
        .as_ref()
        .map(|f| f.current().room_type.clone())
        .unwrap_or(RoomType::Empty);
    let room_desc = state
        .floor
        .as_ref()
        .map(|f| f.current().description.clone())
        .unwrap_or_default();

    match room_type {
        RoomType::Combat | RoomType::Boss => {
            enter_combat_room(state, floor_num, room_seed, room_desc);
        }
        RoomType::Treasure => {
            enter_treasure_room(state, floor_num, room_seed, room_desc);
        }
        RoomType::Shop => {
            enter_shop_room(state, floor_num, room_seed);
        }
        RoomType::Shrine => {
            enter_shrine_room(state, floor_num, room_seed, room_desc);
        }
        RoomType::Trap => {
            enter_trap_room(state, floor_num, room_seed, room_desc);
        }
        RoomType::Portal => {
            enter_portal_room(state, room_desc);
        }
        RoomType::Empty => {
            enter_empty_room(state, floor_num, room_desc);
        }
        RoomType::ChaosRift => {
            enter_chaos_rift(state, floor_num, room_seed, room_desc);
        }
        RoomType::CraftingBench => {
            state.craft_phase = CraftPhase::SelectItem;
            state.craft_item_cursor = 0;
            state.craft_op_cursor = 0;
            state.craft_message = "Choose an item to craft.".to_string();
            state.screen = AppScreen::Crafting;
        }
    }
}

// ── Combat / Boss room entry ─────────────────────────────────────────────────

fn enter_combat_room(
    state: &mut GameState,
    floor_num: u32,
    room_seed: u64,
    _room_desc: String,
) {
    let is_boss = state
        .floor
        .as_ref()
        .map(|f| f.current().room_type == RoomType::Boss)
        .unwrap_or(false);

    // ── Nemesis spawn check ──────────────────────────────────────────────────
    if !state.nemesis_spawned {
        if let Some(ref nemesis) = state.nemesis_record.clone() {
            let spawn_roll =
                room_seed.wrapping_mul(0x6E656D6573697300) % 100;
            let spawn_chance = if floor_num >= nemesis.floor_killed_at {
                40
            } else {
                20
            };
            if floor_num >= 3 && spawn_roll < spawn_chance {
                state.nemesis_spawned = true;
                let base_floor = nemesis.floor_killed_at;
                let mut nem_enemy =
                    generate_enemy(base_floor.max(1), room_seed);
                nem_enemy.name = format!("* {}", nemesis.enemy_name);
                nem_enemy.hp = (nem_enemy.hp
                    * (100 + nemesis.hp_bonus_pct as i64)
                    / 100)
                    .max(1);
                nem_enemy.max_hp = nem_enemy.hp;
                nem_enemy.base_damage = (nem_enemy.base_damage
                    * (100 + nemesis.damage_bonus_pct as i64)
                    / 100)
                    .max(1);
                nem_enemy.xp_reward *= 5;
                nem_enemy.gold_reward *= 3;

                push_log(
                    state,
                    format!("NEMESIS RETURNS: {}!", nem_enemy.name),
                );
                push_log(
                    state,
                    format!(
                        "HP +{}%  DMG +{}%",
                        nemesis.hp_bonus_pct, nemesis.damage_bonus_pct
                    ),
                );

                state.enemy = Some(nem_enemy);
                state.is_boss_fight = true;
                state.gauntlet_stage = 0;
                state.combat_state = Some(CombatState::new(room_seed));
                if let Some(ref mut cs) = state.combat_state {
                    cs.is_cursed = state.is_cursed_floor;
                }
                state.combat_log.clear();
                state.display_enemy_hp = 1.0;
                state.screen = AppScreen::Combat;
                return;
            }
        }
    }

    // ── Boss gauntlet: every 10 floors, boss room = 3-fight gauntlet ─────
    if is_boss && floor_num % 10 == 0 {
        let mut enemies = Vec::new();

        let mut e1 = generate_enemy(floor_num, room_seed.wrapping_add(1));
        e1.hp = (e1.hp as f64 * 2.0) as i64;
        e1.max_hp = e1.hp;

        let mut e2 = generate_enemy(floor_num, room_seed.wrapping_add(2));
        e2.hp = (e2.hp as f64 * 3.0) as i64;
        e2.max_hp = e2.hp;
        e2.base_damage = (e2.base_damage as f64 * 1.5) as i64;

        let dr = destiny_roll(0.5, room_seed.wrapping_add(31337));
        let pm = (dr.final_value + 1.5).max(0.5);
        let mut e3 = generate_enemy(floor_num, room_seed.wrapping_add(3));
        e3.hp = ((e3.hp as f64 * 4.0 * pm) as i64).max(1);
        e3.max_hp = e3.hp;
        e3.base_damage = ((e3.base_damage as f64 * 2.0 * pm) as i64).max(1);
        e3.xp_reward *= 5;
        e3.gold_reward *= 5;

        enemies.push(e1);
        enemies.push(e2);
        enemies.push(e3);

        state.gauntlet_enemies = enemies;
        state.gauntlet_stage = 1;
        let first = state.gauntlet_enemies.remove(0);
        state.enemy = Some(first);
        state.is_boss_fight = false;
        push_log(
            state,
            "BOSS GAUNTLET! 3 fights. No healing.".to_string(),
        );
        push_log(state, "Fight 1/3".to_string());
        state.combat_state = Some(CombatState::new(room_seed));
        if let Some(ref mut cs) = state.combat_state {
            cs.is_cursed = state.is_cursed_floor;
        }
        state.combat_log.clear();
        state.display_enemy_hp = 1.0;
        state.screen = AppScreen::Combat;
        return;
    }

    // ── Unique boss spawn ────────────────────────────────────────────────────
    // floor 5+: boss rooms every 5 floors
    // floor 50+: 20% random in non-boss combat rooms
    // floor 100+: every 3rd room
    let unique_roll =
        room_seed.wrapping_mul(0x756E697175650000) % 100;
    let current_room_idx = state
        .floor
        .as_ref()
        .map(|f| f.current_room)
        .unwrap_or(0);
    let spawn_unique = (floor_num >= 100 && current_room_idx % 3 == 0)
        || (floor_num >= 50 && !is_boss && unique_roll < 20)
        || (is_boss && floor_num % 5 == 0);

    if spawn_unique {
        if let Some(boss_id) = random_unique_boss(floor_num, room_seed) {
            start_unique_boss(state, boss_id, floor_num, room_seed);
            return;
        }
    }

    // ── Normal enemy ─────────────────────────────────────────────────────────
    let room = state
        .floor
        .as_ref()
        .map(|f| f.current().clone())
        .unwrap();
    let mut enemy = room_enemy(&room);

    // StatMirror
    if enemy.floor_ability == FloorAbility::StatMirror {
        let (sname, sval) = state
            .player
            .as_ref()
            .map(|p| p.highest_stat())
            .unwrap_or(("force", 10));
        enemy.hp = sval.max(1);
        enemy.max_hp = enemy.hp;
        push_log(
            state,
            format!(
                "STAT MIRROR: enemy HP = your {} ({})",
                sname, sval
            ),
        );
    }
    if enemy.floor_ability == FloorAbility::NullifyAura {
        push_log(
            state,
            "NULLIFY AURA: first action returns 0.0!".to_string(),
        );
    }
    if enemy.floor_ability == FloorAbility::EngineTheft {
        push_log(
            state,
            "ENGINE THEFT: each hit steals 1 engine!".to_string(),
        );
    }

    if is_boss {
        enemy.hp = (enemy.hp as f64 * 2.5) as i64;
        enemy.max_hp = enemy.hp;
        enemy.base_damage = (enemy.base_damage as f64 * 1.8) as i64;
        enemy.xp_reward *= 3;
        enemy.gold_reward *= 3;
        push_log(state, "BOSS BATTLE".to_string());
    }

    state.enemy = Some(enemy);
    state.is_boss_fight = is_boss;
    state.gauntlet_stage = 0;
    state.combat_state = Some(CombatState::new(room_seed));
    if let Some(ref mut cs) = state.combat_state {
        cs.is_cursed = state.is_cursed_floor;
    }
    state.room_entry_type = if is_boss { 5 } else { 1 };
    state.room_entry_timer = if is_boss { 0.5 } else { 0.5 };
    state.combat_log.clear();
    state.display_enemy_hp = 1.0;
    state.screen = AppScreen::Combat;
}

// ── Unique boss setup (all 12 bosses) ────────────────────────────────────────

fn start_unique_boss(
    state: &mut GameState,
    boss_id: u8,
    floor_num: u32,
    room_seed: u64,
) {
    let bname = boss_name(boss_id);
    let mut enemy = generate_enemy(floor_num + 2, room_seed);
    enemy.name = bname.to_string();
    enemy.xp_reward *= 5;
    enemy.gold_reward *= 5;
    state.boss_id = Some(boss_id);
    state.boss_turn = 0;

    match boss_id {
        1 => {
            // THE MIRROR
            let (max_hp, force, prec) = state
                .player
                .as_ref()
                .map(|p| (p.max_hp, p.stats.force, p.stats.precision))
                .unwrap_or((100, 10, 10));
            enemy.hp = max_hp;
            enemy.max_hp = max_hp;
            enemy.base_damage = 5 + force / 5 + prec / 10;
            state.boss_extra = 0;
            state.boss_extra2 = 0;
            push_log(
                state,
                "THE MIRROR: Your exact reflection -- same HP, same force."
                    .to_string(),
            );
            push_log(
                state,
                "Your class passive still applies. Find the asymmetry."
                    .to_string(),
            );
        }
        2 => {
            // THE ACCOUNTANT
            let lifetime = state
                .player
                .as_ref()
                .map(|p| p.total_damage_dealt)
                .unwrap_or(0);
            enemy.hp = 999_999;
            enemy.max_hp = 999_999;
            enemy.base_damage = 0;
            state.boss_extra = 0;
            state.boss_extra2 = 0;
            push_log(
                state,
                format!(
                    "THE ACCOUNTANT: Lifetime damage on record: {}.",
                    lifetime
                ),
            );
            push_log(
                state,
                "5 rounds, then THE BILL. [D] Defend reduces it 20%/round."
                    .to_string(),
            );
        }
        3 => {
            // FIBONACCI HYDRA
            let hp = 200 + floor_num as i64 * 30;
            enemy.hp = hp;
            enemy.max_hp = hp;
            enemy.base_damage = 8 + floor_num as i64 * 2;
            state.boss_extra = 0;
            state.boss_extra2 = 0;
            push_log(
                state,
                "FIBONACCI HYDRA: Kill it -- it splits. 10 splits = victory."
                    .to_string(),
            );
            push_log(
                state,
                "Splits: 1,1,2,3,5,8,13. Burst damage wins.".to_string(),
            );
        }
        4 => {
            // THE EIGENSTATE
            let oneshot = state
                .player
                .as_ref()
                .map(|p| p.max_hp + 1)
                .unwrap_or(101);
            let tanky_max = 500 + floor_num as i64 * 100;
            enemy.hp = tanky_max;
            enemy.max_hp = tanky_max;
            enemy.base_damage = oneshot;
            state.boss_extra = tanky_max;
            state.boss_extra2 = 0;
            push_log(
                state,
                "THE EIGENSTATE: Form A = huge HP no attack; Form B = 1 HP one-shot."
                    .to_string(),
            );
            push_log(
                state,
                "[T] Taunt reveals form safely. [D] Defend survives Form B."
                    .to_string(),
            );
        }
        5 => {
            // THE TAXMAN
            let stolen = state
                .player
                .as_ref()
                .map(|p| p.gold)
                .unwrap_or(0);
            if let Some(ref mut p) = state.player {
                p.gold = 0;
            }
            let hp = stolen.max(1);
            enemy.hp = hp;
            enemy.max_hp = hp;
            enemy.base_damage = 1;
            state.boss_extra = stolen;
            state.boss_extra2 = 0;
            push_log(
                state,
                format!(
                    "THE TAXMAN: Your {} gold SEIZED! HP = gold owed.",
                    stolen
                ),
            );
            push_log(
                state,
                "Damage = gold recovered. He bills you 1% HP/round."
                    .to_string(),
            );
        }
        6 => {
            // THE NULL
            let hp = 300 + floor_num as i64 * 80;
            enemy.hp = hp;
            enemy.max_hp = hp;
            enemy.base_damage = 20 + floor_num as i64 * 5;
            state.boss_extra = 0;
            state.boss_extra2 = 0;
            push_log(
                state,
                "THE NULL: Chaos suppressed. Your damage is flat. No crits."
                    .to_string(),
            );
            push_log(
                state,
                "Enemy uses full 10-engine destiny rolls.".to_string(),
            );
        }
        7 => {
            // THE OUROBOROS
            let (total_dmg, kills) = state
                .player
                .as_ref()
                .map(|p| (p.total_damage_dealt, p.kills.max(1) as i64))
                .unwrap_or((0, 1));
            let avg = total_dmg / kills;
            let hp = (avg * 3).max(500 + floor_num as i64 * 60);
            enemy.hp = hp;
            enemy.max_hp = hp;
            enemy.base_damage = 15 + floor_num as i64 * 4;
            state.boss_extra = hp;
            state.boss_extra2 = 0;
            push_log(
                state,
                format!(
                    "THE OUROBOROS: Heals to full every 3 turns! HP: {}.",
                    hp
                ),
            );
            push_log(
                state,
                "Kill it within 3 turns. Heavy attacks.".to_string(),
            );
        }
        8 => {
            // THE COLLATZ TITAN
            let start_hp = chaos_roll_verbose(0.5, room_seed)
                .to_range(1000, 9999)
                .max(1000);
            enemy.hp = start_hp;
            enemy.max_hp = start_hp;
            enemy.base_damage = 10 + floor_num as i64 * 3;
            state.boss_extra = start_hp;
            state.boss_extra2 = 0;
            push_log(
                state,
                format!(
                    "THE COLLATZ TITAN: HP follows Collatz. Start: {}.",
                    start_hp
                ),
            );
            push_log(
                state,
                "Each turn: even->HP/2, odd->HP*3+1. Attack when HP is low!"
                    .to_string(),
            );
        }
        9 => {
            // THE COMMITTEE
            let hp_each = 200 + floor_num as i64 * 40;
            enemy.hp = hp_each * 5;
            enemy.max_hp = hp_each * 5;
            enemy.base_damage = 8 + floor_num as i64;
            state.boss_extra = 0b11111;
            state.boss_extra2 = hp_each;
            push_log(
                state,
                "THE COMMITTEE: 5 members, each immune to a different attack type."
                    .to_string(),
            );
            push_log(
                state,
                "[T] Taunt bypasses all immunities. Vary attacks.".to_string(),
            );
        }
        10 => {
            // THE RECURSION
            let hp = state
                .player
                .as_ref()
                .map(|p| p.max_hp)
                .unwrap_or(100);
            enemy.hp = hp;
            enemy.max_hp = hp;
            enemy.base_damage = 5;
            state.boss_extra = 0;
            state.boss_extra2 = 0;
            push_log(
                state,
                format!(
                    "THE RECURSION: HP = your max HP ({}). Every hit reflects!",
                    hp
                ),
            );
            push_log(
                state,
                "[D] Defend reduces reflection by VIT/2.".to_string(),
            );
        }
        11 => {
            // THE PARADOX
            enemy.hp = 999_999;
            enemy.max_hp = 999_999;
            enemy.base_damage = 10 + floor_num as i64 / 2;
            state.boss_extra = 0;
            state.boss_extra2 = 0;
            push_log(
                state,
                "THE PARADOX: Immune to damage. Cannot flee.".to_string(),
            );
            push_log(
                state,
                "[T] Taunt = Talk (CUNNING roll). [D] Defend = +5 CUN bonus."
                    .to_string(),
            );
        }
        12 => {
            // THE ALGORITHM REBORN
            let hp = 2000 + floor_num as i64 * 200;
            enemy.hp = hp;
            enemy.max_hp = hp;
            enemy.base_damage = 25 + floor_num as i64 * 5;
            state.boss_extra = 1;
            state.boss_extra2 = 0;
            push_log(
                state,
                "THE ALGORITHM REBORN: 3 phases. Adapts at 66% and 33% HP."
                    .to_string(),
            );
            push_log(
                state,
                "Vary attack types -- it learns patterns.".to_string(),
            );
        }
        _ => {
            // Generic unique boss fallback
            enemy.hp = (enemy.hp as f64 * 3.0) as i64;
            enemy.max_hp = enemy.hp;
            enemy.base_damage = (enemy.base_damage as f64 * 2.0) as i64;
            state.boss_extra = 0;
            state.boss_extra2 = 0;
        }
    }

    state.enemy = Some(enemy);
    state.is_boss_fight = true;
    state.gauntlet_stage = 0;
    state.combat_state = Some(CombatState::new(room_seed));
    if let Some(ref mut cs) = state.combat_state {
        cs.is_cursed = state.is_cursed_floor;
    }
    // Boss entrance animation
    state.boss_entrance_timer = 3.0;
    state.boss_entrance_name = boss_name(boss_id).to_string();
    state.combat_log.clear();
    state.display_enemy_hp = 1.0;
    state.screen = AppScreen::Combat;
}

// ── Treasure room ────────────────────────────────────────────────────────────

fn enter_treasure_room(
    state: &mut GameState,
    floor_num: u32,
    room_seed: u64,
    room_desc: String,
) {
    let item = Item::generate(room_seed);
    let gold_bonus = ((room_seed % 30 + 10) as i64) * floor_num as i64;
    let mut ev = RoomEvent::empty();
    ev.title = "TREASURE ROOM".to_string();
    ev.lines = vec![
        room_desc,
        String::new(),
        format!("You find {} gold!", gold_bonus),
        String::new(),
        format!("Item: {}", item.name),
        format!("Rarity: {}", item.rarity.name()),
    ];
    for m in &item.stat_modifiers {
        ev.lines.push(format!("  {:+} {}", m.value, m.stat));
    }
    ev.lines.push(String::new());
    ev.lines
        .push("[P] Pick up   [Enter] Leave".to_string());
    ev.gold_delta = gold_bonus;
    ev.pending_item = Some(item);

    // 25% chance for a spell scroll
    if room_seed % 4 == 0 {
        let spell = Spell::generate(room_seed.wrapping_add(54321));
        ev.lines.push(String::new());
        ev.lines
            .push(format!("+ SPELL SCROLL: {}", spell.name));
        ev.lines.push(format!(
            "  {}mp  x{:.1} scaling",
            spell.mana_cost,
            spell.scaling_factor.abs()
        ));
        ev.lines.push(
            "[L] Learn spell   [Enter] Leave scroll".to_string(),
        );
        ev.pending_spell = Some(spell);
    }

    state.room_event = ev;
    state.screen = AppScreen::RoomView;
}

// ── Shop room ────────────────────────────────────────────────────────────────

fn enter_shop_room(state: &mut GameState, floor_num: u32, room_seed: u64) {
    let mut npc = shop_npc(floor_num, room_seed);
    let heal_cost = 15 + floor_num as i64 * 2;
    let cunning = state
        .player
        .as_ref()
        .map(|p| p.stats.cunning)
        .unwrap_or(0);
    let npc_items: Vec<Item> = npc.inventory.drain(..).collect();
    let shop: Vec<(Item, i64)> = npc_items
        .into_iter()
        .map(|item| {
            let price = npc.sale_price(item.value, cunning);
            (item, price)
        })
        .collect();
    state.shop_items = shop;
    state.shop_heal_cost = heal_cost;
    state.shop_cursor = 0;
    state.room_entry_type = 2;
    state.room_entry_timer = 0.33;
    state.screen = AppScreen::Shop;
}

// ── Shrine room ──────────────────────────────────────────────────────────────

fn enter_shrine_room(
    state: &mut GameState,
    floor_num: u32,
    room_seed: u64,
    room_desc: String,
) {
    let entropy = state
        .player
        .as_ref()
        .map(|p| p.stats.entropy as f64 * 0.01)
        .unwrap_or(0.1);
    let roll = chaos_roll_verbose(entropy, room_seed);
    state.last_roll = Some(roll.clone());

    let stats: &[&'static str] = &[
        "vitality",
        "force",
        "mana",
        "cunning",
        "precision",
        "entropy",
        "luck",
    ];
    let stat_name = stats[(room_seed % stats.len() as u64) as usize];
    let buff = 3 + (roll.to_range(1, 10) as i64) + floor_num as i64 / 2;
    let hp_restore = state
        .player
        .as_ref()
        .map(|p| p.max_hp / 5)
        .unwrap_or(10);

    let mut ev = RoomEvent::empty();
    ev.title = "SHRINE".to_string();
    ev.lines = vec![
        room_desc,
        String::new(),
        format!("Chaos value: {:.4}", roll.final_value),
        String::new(),
        format!("The shrine blesses you! +{} {}", buff, stat_name),
        format!("You feel restored. +{} HP", hp_restore),
        String::new(),
        "[Enter] Accept blessing".to_string(),
    ];
    ev.stat_bonuses = vec![(stat_name, buff)];
    ev.hp_delta = hp_restore;
    state.room_event = ev;
    state.room_entry_type = 3;
    state.room_entry_timer = 0.42;
    state.screen = AppScreen::RoomView;
}

// ── Trap room ────────────────────────────────────────────────────────────────

fn enter_trap_room(
    state: &mut GameState,
    floor_num: u32,
    room_seed: u64,
    room_desc: String,
) {
    let player_ref = state.player.as_ref().unwrap();
    let diff = match floor_num {
        1..=3 => SkillDiff::Easy,
        4..=7 => SkillDiff::Medium,
        _ => SkillDiff::Hard,
    };
    let check =
        perform_skill_check(player_ref, SkillType::Perception, diff, room_seed);
    state.last_roll = Some(check.chaos_result.clone());

    let trap_damage = if check.passed {
        0
    } else {
        5 + floor_num as i64 * 3 + (room_seed % 10) as i64
    };

    let mut ev = RoomEvent::empty();
    ev.title = "! TRAP ROOM !".to_string();
    let mut lines = vec![room_desc, String::new()];
    for line in check.display_lines() {
        lines.push(line);
    }
    lines.push(String::new());
    if check.passed {
        lines.push("You spot and dodge the trap!".to_string());
    } else {
        lines.push(format!("TRAP TRIGGERED! -{} HP!", trap_damage));
    }
    lines.push(String::new());
    lines.push("[Enter] Continue".to_string());
    ev.lines = lines;
    ev.damage_taken = trap_damage;
    state.room_event = ev;
    state.screen = AppScreen::RoomView;
}

// ── Portal room ──────────────────────────────────────────────────────────────

fn enter_portal_room(state: &mut GameState, room_desc: String) {
    let mut ev = RoomEvent::empty();
    ev.title = "PORTAL".to_string();
    ev.lines = vec![
        room_desc,
        String::new(),
        "A shimmering rift to the next floor.".to_string(),
        String::new(),
        "[P] Step through portal   [Enter] Resist".to_string(),
    ];
    ev.portal_available = true;
    state.room_event = ev;
    state.screen = AppScreen::RoomView;
}

// ── Empty room ───────────────────────────────────────────────────────────────

fn enter_empty_room(
    state: &mut GameState,
    floor_num: u32,
    room_desc: String,
) {
    let hp_gain = 5 + floor_num as i64 * 2;
    let mut ev = RoomEvent::empty();
    ev.title = "EMPTY ROOM".to_string();
    ev.lines = vec![
        room_desc,
        String::new(),
        format!("The stillness restores you. +{} HP", hp_gain),
        String::new(),
        "[Enter] Continue".to_string(),
    ];
    ev.hp_delta = hp_gain;
    state.room_event = ev;
    state.screen = AppScreen::RoomView;
}

// ── Chaos Rift room ──────────────────────────────────────────────────────────

fn enter_chaos_rift(
    state: &mut GameState,
    floor_num: u32,
    room_seed: u64,
    _room_desc: String,
) {
    let entropy = state
        .player
        .as_ref()
        .map(|p| p.stats.entropy as f64 * 0.015)
        .unwrap_or(0.1);
    let roll = chaos_roll_verbose(entropy, room_seed);
    state.last_roll = Some(roll.clone());

    let outcome =
        room_seed.wrapping_mul(floor_num as u64 * 7 + 1) % 6;
    let mut ev = RoomEvent::empty();
    ev.title = "CHAOS RIFT".to_string();
    ev.lines = vec![
        "REALITY ERROR. MATHEMATICAL EXCEPTION.".to_string(),
        String::new(),
        format!("Chaos value: {:.4}", roll.final_value),
        String::new(),
    ];

    match outcome {
        0 => {
            let gold =
                ((room_seed % 100 + 50) as i64) * floor_num as i64;
            ev.lines.push(format!("CHAOS BOUNTY: +{} gold!", gold));
            ev.gold_delta = gold;
        }
        1 => {
            let dmg = state
                .player
                .as_ref()
                .map(|p| (p.max_hp / 4).max(1))
                .unwrap_or(10);
            ev.lines
                .push(format!("CHAOS PUNISHMENT: -{} HP!", dmg));
            ev.damage_taken = dmg;
        }
        2 => {
            let bonus = 5 + floor_num as i64;
            ev.lines
                .push(format!("CHAOS ASCENSION: +{} Entropy!", bonus));
            ev.stat_bonuses = vec![("entropy", bonus)];
        }
        3 => {
            let heal = state
                .player
                .as_ref()
                .map(|p| p.max_hp / 3)
                .unwrap_or(20);
            ev.lines
                .push(format!("CHAOS BLESSING: +{} HP!", heal));
            ev.hp_delta = heal;
        }
        4 => {
            let gold_loss = state
                .player
                .as_ref()
                .map(|p| p.gold / 4)
                .unwrap_or(0);
            let luck = 10 + floor_num as i64;
            ev.lines.push(format!(
                "CHAOS TRADE: -{} gold, +{} Luck!",
                gold_loss, luck
            ));
            ev.gold_delta = -gold_loss;
            ev.stat_bonuses = vec![("luck", luck)];
        }
        _ => {
            ev.lines
                .push("CHAOS HARMONY: All stats +1!".to_string());
            ev.stat_bonuses = vec![
                ("vitality", 1),
                ("force", 1),
                ("mana", 1),
                ("cunning", 1),
                ("precision", 1),
                ("entropy", 1),
                ("luck", 1),
            ];
        }
    }
    ev.lines.push(String::new());
    ev.lines.push("[Enter] Accept fate".to_string());
    state.room_event = ev;
    state.room_entry_type = 4;
    state.room_entry_timer = 0.5;
    state.screen = AppScreen::RoomView;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. on_combat_victory
// ═══════════════════════════════════════════════════════════════════════════════

/// Called after combat resolves with a player win. Handles:
/// - XP + gold from the enemy
/// - Kill tracking, rooms_without_kill reset
/// - Nemesis kill reward
/// - Loot drop
/// - Gauntlet progression
/// - Boss win bonuses (Taxman gold return, Ouroboros burst bonus)
/// - Advancing the floor / showing loot screen
pub fn on_combat_victory(state: &mut GameState, xp: u64, gold: i64) {
    // Boss win bonus adjustments
    let (xp, gold) = if let Some(bid) = state.boss_id.take() {
        state.boss_turn = 0;
        state.boss_extra = 0;
        state.boss_extra2 = 0;
        boss_win_bonus(state, bid, xp, gold)
    } else {
        (xp, gold)
    };

    push_log(state, format!("Victory! +{} XP  +{} gold", xp, gold));

    if let Some(ref mut p) = state.player {
        p.kills += 1;
        if p.floor >= 50 {
            p.rooms_without_kill = 0;
        }
    }

    // Nemesis tracking: if nemesis killed, clear it and reward
    if let Some(ref nem) = state.nemesis_record.clone() {
        if state
            .enemy
            .as_ref()
            .map(|e| e.name.contains(&nem.enemy_name))
            .unwrap_or(false)
        {
            clear_nemesis();
            state.nemesis_record = None;
            push_log(
                state,
                "Nemesis defeated! Grudge settled.".to_string(),
            );
            if let Some(ref mut p) = state.player {
                let (sname, _) = p.highest_stat();
                match sname {
                    "Vitality" => p.stats.vitality += 50,
                    "Force" => p.stats.force += 50,
                    "Mana" => p.stats.mana += 50,
                    "Cunning" => p.stats.cunning += 50,
                    "Precision" => p.stats.precision += 50,
                    "Entropy" => p.stats.entropy += 50,
                    _ => p.stats.luck += 50,
                }
            }
        }
    }

    // Loot drop
    let loot_seed = state
        .floor_seed
        .wrapping_add(state.frame)
        .wrapping_mul(6364136223846793005);
    let drop_chance: u64 = if state.is_boss_fight { 100 } else { 40 };
    if loot_seed % 100 < drop_chance {
        let loot = Item::generate(loot_seed);
        push_log(
            state,
            format!("Item dropped: {}!", loot.name),
        );
        state.loot_pending = Some(loot);
    }

    // Boss gauntlet: advance to next fight
    if state.gauntlet_stage > 0 && !state.gauntlet_enemies.is_empty() {
        state.gauntlet_stage += 1;
        let next = state.gauntlet_enemies.remove(0);
        push_log(
            state,
            format!("GAUNTLET: Fight {}/3", state.gauntlet_stage),
        );
        state.enemy = Some(next);
        let ns = state
            .floor_seed
            .wrapping_add(state.gauntlet_stage as u64 * 1337);
        state.combat_state = Some(CombatState::new(ns));
        if let Some(ref mut cs) = state.combat_state {
            cs.is_cursed = state.is_cursed_floor;
        }
        return; // Stay in combat
    }
    state.gauntlet_stage = 0;

    // Show loot screen if pending, otherwise advance floor
    let next_screen = if state.loot_pending.is_some() {
        let loot = state.loot_pending.take().unwrap();
        state.room_event = RoomEvent::empty();
        state.room_event.title = "LOOT DROPPED".to_string();
        state.room_event.lines = vec![
            format!("Enemy dropped: {}", loot.name),
            format!("Rarity: {}", loot.rarity.name()),
        ];
        for m in &loot.stat_modifiers {
            state
                .room_event
                .lines
                .push(format!("  {:+} {}", m.value, m.stat));
        }
        state.room_event.lines.push(String::new());
        state
            .room_event
            .lines
            .push("[P] Pick up   [Enter] Leave".to_string());
        state.room_event.pending_item = Some(loot);
        AppScreen::RoomView
    } else {
        descend(state);
        if state.screen == AppScreen::GameOver
            || state.screen == AppScreen::Victory
        {
            state.screen.clone()
        } else {
            AppScreen::FloorNav
        }
    };

    // Kill linger: stay on combat screen briefly before transitioning
    state.kill_linger = 0.5;
    state.post_combat_screen = Some(next_screen);
}

/// Boss-specific win bonuses.
fn boss_win_bonus(
    state: &mut GameState,
    bid: u8,
    xp: u64,
    gold: i64,
) -> (u64, i64) {
    match bid {
        5 => {
            // Taxman: return seized gold + 20% interest
            let stolen = state.boss_extra;
            let returned = stolen + stolen / 5;
            if let Some(ref mut p) = state.player {
                p.gold += returned;
            }
            push_log(
                state,
                format!(
                    "Gold returned: {} + 20% interest = {}!",
                    stolen, returned
                ),
            );
            (xp, gold + returned)
        }
        7 if state.boss_turn <= 3 => {
            // Ouroboros: burst kill bonus
            push_log(
                state,
                "BURST KILL -- Ouroboros down before its first reset!"
                    .to_string(),
            );
            (xp + 200, gold + 50)
        }
        _ => (xp, gold),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. on_player_death
// ═══════════════════════════════════════════════════════════════════════════════

/// Called when the player dies. Handles:
/// - Boss state reset
/// - Nemesis promotion (the killing enemy becomes your Nemesis)
/// - Score saving
/// - Screen transition to GameOver
pub fn on_player_death(state: &mut GameState) {
    state.boss_id = None;
    state.boss_turn = 0;
    state.boss_extra = 0;
    state.boss_extra2 = 0;

    state.player_flash = 0.0;
    state.enemy_flash = 0.0;
    state.hit_shake = 0.0;

    // Save nemesis
    let enemy_name = state
        .enemy
        .as_ref()
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let enemy_dmg = state
        .enemy
        .as_ref()
        .map(|e| e.base_damage)
        .unwrap_or(5);

    if let Some(ref p) = state.player {
        let method = if p.spells_cast > p.kills * 2 {
            "spell"
        } else {
            "physical"
        };
        let nem = NemesisRecord::new(
            enemy_name.clone(),
            p.floor,
            enemy_dmg,
            p.class.name().to_string(),
            method,
        );
        save_nemesis(&nem);
        push_log(
            state,
            format!("{} is now your Nemesis.", enemy_name),
        );
    }

    save_score_now(state);

    // Death cinematic setup
    state.death_cinematic_done = false;
    state.screen = AppScreen::GameOver;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. apply_room_event
// ═══════════════════════════════════════════════════════════════════════════════

/// Apply the stat bonuses, HP/gold from a room event. Called when the player
/// presses Enter/confirms on a RoomView screen.
pub fn apply_room_event(state: &mut GameState) {
    let ev = &state.room_event;

    // Apply gold delta
    if ev.gold_delta != 0 {
        if let Some(ref mut p) = state.player {
            p.gold = (p.gold + ev.gold_delta).max(0);
        }
    }

    // Apply HP delta (heal)
    if ev.hp_delta != 0 {
        if let Some(ref mut p) = state.player {
            p.current_hp = (p.current_hp + ev.hp_delta).min(p.max_hp).max(0);
        }
    }

    // Apply damage
    if ev.damage_taken > 0 {
        if let Some(ref mut p) = state.player {
            p.take_damage(ev.damage_taken);
        }
    }

    // Apply stat bonuses
    // We need to clone stat_bonuses to avoid borrow issues
    let bonuses: Vec<(&'static str, i64)> =
        state.room_event.stat_bonuses.clone();
    for (stat, val) in &bonuses {
        if let Some(ref mut p) = state.player {
            match *stat {
                "vitality" => p.stats.vitality += val,
                "force" => p.stats.force += val,
                "mana" => p.stats.mana += val,
                "cunning" => p.stats.cunning += val,
                "precision" => p.stats.precision += val,
                "entropy" => p.stats.entropy += val,
                "luck" => p.stats.luck += val,
                _ => {}
            }
        }
    }

    // Mark resolved
    state.room_event.resolved = true;

    // Check if player died from trap/rift damage
    if state
        .player
        .as_ref()
        .map(|p| !p.is_alive())
        .unwrap_or(false)
    {
        state.screen = AppScreen::GameOver;
        save_score_now(state);
        return;
    }

    // If this was a portal and player chose to go through, descend
    if state.room_event.portal_available {
        // Portal logic: calling code should check if the player pressed [P]
        // and call descend() separately. This function just applies the
        // event's immediate effects.
    }

    // Mark the current room as visited
    if let Some(ref mut floor) = state.floor {
        if floor.current_room < floor.rooms.len() {
            floor.rooms[floor.current_room].visited = true;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Internal: save_score_now
// ═══════════════════════════════════════════════════════════════════════════════

/// Save the current player's score to the scoreboard.
fn save_score_now(state: &mut GameState) {
    if let Some(ref p) = state.player {
        let score_val =
            p.xp + p.gold as u64 + (p.kills * 100) as u64 + (p.floor as u64 * 500);
        let tier = p.power_tier();
        let underdog = p.underdog_multiplier();
        let misery = p.misery.misery_index;

        let entry = ScoreEntry::new(
            p.name.clone(),
            p.class.name().to_string(),
            score_val,
            p.floor,
            p.kills,
            0,
        )
        .with_tier(tier.name())
        .with_misery(misery, underdog);

        let _ = save_score(entry);

        // Build recap text
        let mode_str = match state.game_mode {
            GameMode::Story => "Story",
            GameMode::Infinite => "Infinite",
            GameMode::Daily => "Daily",
        };
        state.last_recap_text = format!(
            "CHAOS RPG | {} {} | Floor {} | Kills {} | Score {} | {} | Seed {:X}",
            p.class.name(),
            p.name,
            p.floor,
            p.kills,
            score_val,
            mode_str,
            p.seed,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Combat helpers used by the combat screen
// ═══════════════════════════════════════════════════════════════════════════════

/// Called when the player flees combat successfully.
pub fn on_player_fled(state: &mut GameState) {
    state.boss_id = None;
    state.boss_turn = 0;
    state.boss_extra = 0;
    state.boss_extra2 = 0;
    push_log(
        state,
        "You escaped into the chaos!".to_string(),
    );
    state.enemy = None;
    if let Some(ref mut p) = state.player {
        p.rooms_without_kill += 1;
    }
    descend(state);
    if state.screen != AppScreen::GameOver && state.screen != AppScreen::Victory {
        state.screen = AppScreen::FloorNav;
    }
}

/// Boss pre-turn effects — called before resolve_action.
pub fn boss_pre_turn(state: &mut GameState, bid: u8) {
    match bid {
        7 => {
            // Ouroboros: heal to full every 3 turns
            if state.boss_turn > 1 && (state.boss_turn - 1) % 3 == 0 {
                let max_hp = state.boss_extra;
                if let Some(ref mut e) = state.enemy {
                    e.hp = max_hp;
                }
                push_log(
                    state,
                    format!(
                        "OUROBOROS heals to full ({} HP)! Cycle resets.",
                        max_hp
                    ),
                );
            }
        }
        8 => {
            // Collatz Titan: transform HP
            let n = state.boss_extra;
            let new_n = if n % 2 == 0 { n / 2 } else { n * 3 + 1 };
            state.boss_extra = new_n;
            if let Some(ref mut e) = state.enemy {
                e.hp = new_n.max(1);
                if new_n > e.max_hp {
                    e.max_hp = new_n;
                }
            }
            let next = if new_n % 2 == 0 {
                new_n / 2
            } else {
                new_n * 3 + 1
            };
            push_log(
                state,
                format!(
                    "TITAN HP: {} -> {} (Collatz). Next: {}",
                    n, new_n, next
                ),
            );
            if new_n <= 4 {
                push_log(
                    state,
                    "ATTACK NOW -- Titan at Collatz minimum!".to_string(),
                );
            }
        }
        6 => {
            // Null: set enemy damage via destiny roll
            let sc = state
                .floor_seed
                .wrapping_add(state.boss_turn as u64 * 7919);
            let dr = destiny_roll(0.5, sc);
            let base = 20 + state.floor_num as i64 * 5;
            let mult = (dr.final_value + 1.5).max(0.1);
            let dmg = ((base as f64 * mult) as i64).max(1);
            if let Some(ref mut e) = state.enemy {
                e.base_damage = dmg;
            }
            push_log(
                state,
                format!(
                    "[NUL] Chaos suppressed -- flat damage only. Enemy hits for {}.",
                    dmg
                ),
            );
        }
        _ => {}
    }
}

/// Boss post-turn effects — called after resolve_action events.
pub fn boss_post_turn(
    state: &mut GameState,
    bid: u8,
    action: &chaos_rpg_core::combat::CombatAction,
    events: &[chaos_rpg_core::combat::CombatEvent],
) {
    use chaos_rpg_core::combat::{CombatAction, CombatEvent};

    match bid {
        2 => {
            // Accountant: track ledger; deliver bill after 5 turns
            let dmg_this = events
                .iter()
                .find_map(|e| {
                    if let CombatEvent::PlayerAttack { damage, .. } = e {
                        Some(*damage)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            if dmg_this > 0 {
                state.boss_extra += dmg_this;
            }
            if matches!(action, CombatAction::Defend) {
                state.boss_extra2 += 1;
            }
            let lifetime = state
                .player
                .as_ref()
                .map(|p| p.total_damage_dealt)
                .unwrap_or(0);
            let defends = state.boss_extra2;
            push_log(
                state,
                format!(
                    "[LEDGER] Fight: {}  Lifetime: {}  Defends: {} ({}% reduction)",
                    state.boss_extra,
                    lifetime,
                    defends,
                    (defends * 20).min(80)
                ),
            );
            if state.boss_turn >= 5 {
                let bill_base = lifetime + state.boss_extra;
                let reduction = (defends as f64 * 0.20).min(0.80);
                let bill =
                    ((bill_base as f64 * (1.0 - reduction)) as i64).max(1);
                push_log(
                    state,
                    format!(
                        "THE BILL! {} x {}% kept = {} damage!",
                        bill_base,
                        100 - (reduction * 100.0) as u32,
                        bill
                    ),
                );
                if let Some(ref mut p) = state.player {
                    p.take_damage(bill);
                }
                state.player_flash = 0.3;
                state.hit_shake = 0.4;
                if state
                    .player
                    .as_ref()
                    .map(|p| p.is_alive())
                    .unwrap_or(false)
                {
                    let floor = state.floor_num;
                    let xp = 600 + floor as u64 * 150;
                    let gold = 300 + floor as i64 * 30;
                    if let Some(ref mut p) = state.player {
                        p.gain_xp(xp);
                        p.kills += 1;
                        p.gold += gold;
                    }
                    if let Some(ref mut e) = state.enemy {
                        e.hp = 0;
                    }
                    push_log(
                        state,
                        format!(
                            "Survived THE BILL! +{} XP, +{} gold.",
                            xp, gold
                        ),
                    );
                    state.boss_id = None;
                    state.boss_turn = 0;
                } else {
                    push_log(
                        state,
                        "Your power was your undoing.".to_string(),
                    );
                }
            }
        }
        5 => {
            // Taxman: 1% HP attack per round
            let hp = state
                .enemy
                .as_ref()
                .map(|e| e.hp)
                .unwrap_or(0);
            if hp > 0 {
                let is_defend = matches!(action, CombatAction::Defend);
                let tax_atk = ((hp as f64 * 0.01) as i64).max(1);
                let incoming = if is_defend {
                    (tax_atk / 2).max(1)
                } else {
                    tax_atk
                };
                if let Some(ref mut p) = state.player {
                    p.take_damage(incoming);
                }
                push_log(
                    state,
                    format!(
                        "Taxman bills you {} HP (1% of {} remaining).",
                        incoming, hp
                    ),
                );
                state.player_flash = 0.15;
            }
        }
        8 => {
            // Collatz Titan: sync boss_extra with HP after player attack
            let hp = state
                .enemy
                .as_ref()
                .map(|e| e.hp)
                .unwrap_or(0);
            state.boss_extra = hp;
        }
        10 => {
            // Recursion: reflect player attack back
            let player_dmg = events
                .iter()
                .find_map(|e| {
                    if let CombatEvent::PlayerAttack { damage, .. } = e {
                        Some(*damage)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            if player_dmg > 0 {
                let vit = state
                    .player
                    .as_ref()
                    .map(|p| p.stats.vitality)
                    .unwrap_or(0);
                let is_defend = matches!(action, CombatAction::Defend);
                let reflection = if is_defend {
                    (player_dmg - vit / 2).max(1)
                } else {
                    player_dmg
                };
                if let Some(ref mut p) = state.player {
                    p.take_damage(reflection);
                }
                state.boss_extra += reflection;
                push_log(
                    state,
                    format!(
                        "RECURSION reflects {} back! (Total: {})",
                        reflection, state.boss_extra
                    ),
                );
                state.player_flash = 0.15;
            }
        }
        12 => {
            // Algorithm Reborn: phase transitions at 66% and 33%
            let (hp, max_hp) = state
                .enemy
                .as_ref()
                .map(|e| (e.hp, e.max_hp))
                .unwrap_or((1, 1));
            let pct = hp * 100 / max_hp.max(1);
            let phase = state.boss_extra;
            if phase == 1 && pct <= 66 {
                state.boss_extra = 2;
                if let Some(ref mut e) = state.enemy {
                    e.base_damage = (e.base_damage as f64 * 1.5) as i64;
                }
                push_log(
                    state,
                    "ALGORITHM REBORN Phase 2: Adapting -- damage increased!"
                        .to_string(),
                );
                state.hit_shake = 0.3;
            } else if phase == 2 && pct <= 33 {
                state.boss_extra = 3;
                if let Some(ref mut e) = state.enemy {
                    e.base_damage = (e.base_damage as f64 * 1.5) as i64;
                }
                push_log(
                    state,
                    "ALGORITHM REBORN Phase 3: FINAL PROTOCOL! Maximum power!"
                        .to_string(),
                );
                state.hit_shake = 0.4;
            }
        }
        _ => {}
    }
}

/// Fibonacci Hydra intercept: returns true if the hydra splits (combat continues).
pub fn hydra_intercept(state: &mut GameState) -> bool {
    let gen = state.boss_extra;
    let splits = state.boss_extra2;
    if gen < 7 && splits < 10 {
        state.boss_extra += 1;
        let next_gen = state.boss_extra;
        let next_hp = ((300 + state.floor_num as i64 * 40)
            * (1_i64 << (next_gen - 1).min(30)))
        .max(1);
        push_log(
            state,
            format!(
                "HYDRA: Generation {} rises with {} HP!",
                next_gen, next_hp
            ),
        );
        if let Some(ref mut e) = state.enemy {
            e.hp = next_hp;
            e.max_hp = next_hp;
        }
        if let Some(ref mut cs) = state.combat_state {
            cs.turn = 0;
        }
        return true; // combat continues
    }
    false
}

/// Boss Eigenstate (boss 4) full-override combat handler.
/// Returns true if the player died, false otherwise.
pub fn boss_eigenstate(
    state: &mut GameState,
    action: chaos_rpg_core::combat::CombatAction,
) -> bool {
    use chaos_rpg_core::chaos_pipeline::biased_chaos_roll;
    use chaos_rpg_core::combat::CombatAction;

    let floor = state.floor_num;
    let sc = state
        .floor_seed
        .wrapping_add(state.boss_turn as u64 * 131071);
    let luck_bias = state
        .player
        .as_ref()
        .map(|p| -(p.stats.luck as f64 / 200.0).clamp(-0.8, 0.8))
        .unwrap_or(0.0);
    let form_roll = biased_chaos_roll(luck_bias, luck_bias, sc);
    let is_form_a = form_roll.final_value > 0.0;
    state.last_roll = Some(form_roll);

    let tanky_hp = state.boss_extra;
    let tanky_max = state
        .enemy
        .as_ref()
        .map(|e| e.max_hp)
        .unwrap_or(500 + floor as i64 * 100);
    let oneshot_dmg = state
        .player
        .as_ref()
        .map(|p| p.max_hp + 1)
        .unwrap_or(101);
    let force = state
        .player
        .as_ref()
        .map(|p| p.stats.force)
        .unwrap_or(10);
    let vit = state
        .player
        .as_ref()
        .map(|p| p.stats.vitality)
        .unwrap_or(0);

    match action {
        CombatAction::Taunt => {
            let probe = 5 + floor as i64 / 2;
            if let Some(ref mut p) = state.player {
                p.take_damage(probe);
            }
            state.player_flash = 0.15;
            if is_form_a {
                push_log(
                    state,
                    format!(
                        "FORM A -- huge HP, no attack. Strike next! (probe: {}dmg)",
                        probe
                    ),
                );
            } else {
                push_log(
                    state,
                    format!(
                        "FORM B -- 1 HP, one-shot. DEFEND next! (probe: {}dmg)",
                        probe
                    ),
                );
            }
        }
        CombatAction::Defend => {
            if !is_form_a {
                let reduced = (oneshot_dmg - vit * 2).max(1);
                if let Some(ref mut p) = state.player {
                    p.take_damage(reduced);
                }
                state.player_flash = 0.3;
                state.hit_shake = 0.4;
                push_log(
                    state,
                    format!(
                        "Form B ATTACKS -- defended! Took {} (VIT absorbed some).",
                        reduced
                    ),
                );
            } else {
                push_log(
                    state,
                    "Form A -- you defend. No incoming attack.".to_string(),
                );
            }
        }
        CombatAction::Flee => {
            push_log(
                state,
                "The Eigenstate holds. Cannot escape.".to_string(),
            );
        }
        _ => {
            if is_form_a {
                let base = 5 + force / 5;
                let roll = chaos_roll_verbose(
                    force as f64 * 0.01,
                    sc.wrapping_add(1),
                );
                let mut dmg = (base
                    + (roll.final_value * base as f64 * 0.5) as i64)
                    .max(1);
                if roll.is_critical() {
                    dmg = (dmg as f64 * 1.5) as i64;
                }
                if roll.is_catastrophe() {
                    dmg = 0;
                }
                let new_tanky = (tanky_hp - dmg).max(0);
                state.boss_extra = new_tanky;
                if let Some(ref mut e) = state.enemy {
                    e.hp = new_tanky.max(1);
                }
                state.enemy_flash = 0.15;
                push_log(
                    state,
                    format!(
                        "Form A -- dealt {}. Tanky HP: {}/{}",
                        dmg, new_tanky, tanky_max
                    ),
                );
                if new_tanky <= 0 {
                    let xp = 700 + floor as u64 * 150;
                    let gold = 180 + floor as i64 * 30;
                    if let Some(ref mut p) = state.player {
                        p.gain_xp(xp);
                        p.kills += 1;
                        p.gold += gold;
                    }
                    if let Some(ref mut e) = state.enemy {
                        e.hp = 0;
                    }
                    push_log(
                        state,
                        "THE EIGENSTATE collapses -- defeated!".to_string(),
                    );
                    push_log(
                        state,
                        format!("+{} XP, +{} gold.", xp, gold),
                    );
                    state.boss_id = None;
                    state.boss_turn = 0;
                    on_combat_victory(state, xp, gold);
                    return false;
                }
            } else {
                if let Some(ref mut e) = state.enemy {
                    e.hp = 0;
                }
                if let Some(ref mut p) = state.player {
                    p.take_damage(oneshot_dmg);
                }
                state.player_flash = 0.3;
                state.hit_shake = 0.4;
                push_log(
                    state,
                    format!(
                        "Form B -- 1 HP! You kill it... but it fires first: {} DAMAGE!",
                        oneshot_dmg
                    ),
                );
            }
        }
    }

    // Check player death
    if !state
        .player
        .as_ref()
        .map(|p| p.is_alive())
        .unwrap_or(true)
    {
        push_log(
            state,
            "The Eigenstate collapses onto you.".to_string(),
        );
        state.boss_id = None;
        state.boss_turn = 0;
        on_player_death(state);
        return true;
    }
    false
}

/// Boss Paradox (boss 11) full-override combat handler.
/// Returns true if the player died, false otherwise.
pub fn boss_paradox(
    state: &mut GameState,
    action: chaos_rpg_core::combat::CombatAction,
) -> bool {
    use chaos_rpg_core::combat::CombatAction;

    let floor = state.floor_num;
    let sc = state
        .floor_seed
        .wrapping_add(state.boss_turn as u64 * 104729);
    let cunning = state
        .player
        .as_ref()
        .map(|p| p.stats.cunning)
        .unwrap_or(10);
    let cun_bonus = state.boss_extra2;
    let failed = state.boss_extra;

    match action {
        CombatAction::Defend => {
            state.boss_extra2 += 5;
            push_log(
                state,
                format!(
                    "You observe. Cunning bonus: +{}.",
                    state.boss_extra2
                ),
            );
            if state.boss_turn > 3 {
                let dmg = (5 + floor as i64).max(1);
                if let Some(ref mut p) = state.player {
                    p.take_damage(dmg);
                }
                state.player_flash = 0.15;
                push_log(
                    state,
                    format!(
                        "Paradox tires of stalling -- {} damage.",
                        dmg
                    ),
                );
            }
        }
        CombatAction::Taunt => {
            let bias =
                ((cunning + cun_bonus) as f64 / 200.0).clamp(-0.8, 0.8);
            let roll = chaos_roll_verbose(bias, sc);
            let needed = (40 + floor as i64 / 2 - cun_bonus).max(10);
            let score = roll.to_range(0, 100);
            state.last_roll = Some(roll);
            push_log(
                state,
                format!(
                    "CUNNING roll: {} (need > {} with +{} bonus).",
                    score, needed, cun_bonus
                ),
            );
            if score > needed {
                let xp = 800 + floor as u64 * 150;
                let gold = 150 + floor as i64 * 20;
                if let Some(ref mut p) = state.player {
                    p.gain_xp(xp);
                    p.kills += 1;
                    p.gold += gold;
                }
                if let Some(ref mut e) = state.enemy {
                    e.hp = 0;
                }
                push_log(
                    state,
                    "The Paradox acknowledges your logic. It dissolves."
                        .to_string(),
                );
                push_log(
                    state,
                    format!("+{} XP, +{} gold.", xp, gold),
                );
                state.boss_id = None;
                state.boss_turn = 0;
                on_combat_victory(state, xp, gold);
                return false;
            } else {
                state.boss_extra += 1;
                push_log(
                    state,
                    format!(
                        "Failed talk #{} -- the Paradox takes something.",
                        failed + 1
                    ),
                );
                if failed == 0 {
                    if let Some(ref mut p) = state.player {
                        if !p.known_spells.is_empty() {
                            p.known_spells.pop();
                        }
                    }
                    push_log(
                        state,
                        "A spell dissolves into paradox!".to_string(),
                    );
                } else if state
                    .player
                    .as_ref()
                    .map(|p| !p.inventory.is_empty())
                    .unwrap_or(false)
                {
                    if let Some(ref mut p) = state.player {
                        p.inventory.pop();
                    }
                    push_log(
                        state,
                        "An item winks out of existence!".to_string(),
                    );
                }
                let atk = (10 + floor as i64 / 2).max(1);
                if let Some(ref mut p) = state.player {
                    p.take_damage(atk);
                }
                state.player_flash = 0.15;
                push_log(
                    state,
                    format!(
                        "The Paradox punishes your failure: {} damage.",
                        atk
                    ),
                );
            }
        }
        CombatAction::Attack | CombatAction::HeavyAttack => {
            let force = state
                .player
                .as_ref()
                .map(|p| p.stats.force)
                .unwrap_or(10);
            let roll = chaos_roll_verbose(force as f64 * 0.01, sc);
            let base = 5 + force / 5;
            let heal = (base
                + (roll.final_value * base as f64 * 0.5) as i64)
                .max(1);
            let max_hp = state
                .enemy
                .as_ref()
                .map(|e| e.max_hp)
                .unwrap_or(999_999);
            if let Some(ref mut e) = state.enemy {
                e.hp = (e.hp + heal).min(max_hp);
            }
            let retaliation = (10 + floor as i64 / 2).max(1);
            if let Some(ref mut p) = state.player {
                p.take_damage(retaliation);
            }
            state.player_flash = 0.15;
            push_log(
                state,
                format!(
                    "Attacking HEALS the Paradox by {}! Retaliation: {}dmg.",
                    heal, retaliation
                ),
            );
            push_log(
                state,
                "Use [T] Talk or [D] Observe.".to_string(),
            );
        }
        CombatAction::Flee => {
            let atk = (5 + floor as i64 / 3).max(1);
            if let Some(ref mut p) = state.player {
                p.take_damage(atk);
            }
            state.player_flash = 0.15;
            push_log(
                state,
                format!(
                    "The Paradox is inescapable. {} damage for trying.",
                    atk
                ),
            );
        }
        _ => {
            push_log(
                state,
                "Only [T] Talk or [D] Observe work here.".to_string(),
            );
        }
    }

    // Check player death
    if !state
        .player
        .as_ref()
        .map(|p| p.is_alive())
        .unwrap_or(true)
    {
        push_log(
            state,
            "The Paradox outlasts you.".to_string(),
        );
        state.boss_id = None;
        state.boss_turn = 0;
        on_player_death(state);
        return true;
    }
    false
}

/// Level-up check after combat actions. Call after resolve_action.
/// Returns true if the player leveled up.
pub fn check_level_up(state: &mut GameState, level_before: u32) -> bool {
    let (level_after, skill_pts) = state
        .player
        .as_ref()
        .map(|p| (p.level, p.skill_points))
        .unwrap_or((0, 0));
    if level_after > level_before {
        push_log(
            state,
            format!("LEVEL UP! Now level {}!", level_after),
        );
        if skill_pts > 0 {
            push_log(
                state,
                format!("  {} skill point(s) available!", skill_pts),
            );
        }
        return true;
    }
    false
}
