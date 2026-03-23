//! CHAOS RPG — Graphical Frontend (bracket-lib)
//!
//! Full feature parity with the terminal version.
//! Always runs fullscreen. All room types, modes, boons, nemesis, gauntlet,
//! cursed floors, The Hunger, item volatility, crafting (all 6 ops), and
//! real chaos-engine combat via resolve_action().

use bracket_lib::prelude::*;
use chaos_rpg_core::{
    bosses::{boss_name, boss_pool_for_floor, random_unique_boss},
    character::{Background, Boon, Character, CharacterClass, Difficulty},
    chaos_pipeline::{chaos_roll_verbose, destiny_roll, ChaosRollResult},
    combat::{resolve_action, CombatAction, CombatOutcome, CombatState},
    enemy::{generate_enemy, Enemy, FloorAbility},
    items::{Item, Rarity, StatModifier},
    nemesis::{clear_nemesis, load_nemesis, save_nemesis, NemesisRecord},
    npcs::shop_npc,
    scoreboard::{load_scores, save_score, ScoreEntry},
    skill_checks::{perform_skill_check, Difficulty as SkillDiff, SkillType},
    spells::Spell,
    world::{generate_floor, room_enemy, Floor, RoomType},
};

mod renderer;
mod sprites;
mod ui_overlay;

// ─── GAME MODE ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameMode { Story, Infinite, Daily }

// ─── CRAFTING PHASE ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum CraftPhase { SelectItem, SelectOp }

// ─── SCREENS ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
    Title,
    ModeSelect,
    CharacterCreation,
    BoonSelect,
    FloorNav,
    RoomView,
    Combat,
    Shop,
    Crafting,
    GameOver,
    Victory,
    Scoreboard,
}

// ─── ROOM EVENT ───────────────────────────────────────────────────────────────

struct RoomEvent {
    title: String,
    lines: Vec<String>,
    pending_item: Option<Item>,
    pending_spell: Option<Spell>,
    gold_delta: i64,
    hp_delta: i64,
    damage_taken: i64,
    stat_bonuses: Vec<(&'static str, i64)>,
    portal_available: bool,
    resolved: bool,
}

impl RoomEvent {
    fn empty() -> Self {
        Self {
            title: String::new(), lines: Vec::new(),
            pending_item: None, pending_spell: None,
            gold_delta: 0, hp_delta: 0, damage_taken: 0,
            stat_bonuses: Vec::new(),
            portal_available: false, resolved: false,
        }
    }
}

// ─── STATE ────────────────────────────────────────────────────────────────────

struct State {
    screen: AppScreen,
    player: Option<Character>,
    floor: Option<Floor>,
    enemy: Option<Enemy>,
    combat_state: Option<CombatState>,
    last_roll: Option<ChaosRollResult>,
    combat_log: Vec<String>,
    seed: u64,
    floor_seed: u64,
    frame: u64,
    // char creation
    selected_menu: usize,
    cc_class: usize,
    cc_bg: usize,
    cc_diff: usize,
    // mode select
    mode_cursor: usize,
    game_mode: GameMode,
    // boon select
    boon_options: [Boon; 3],
    boon_cursor: usize,
    // floor state
    floor_num: u32,
    max_floor: u32,
    is_cursed_floor: bool,
    // nemesis
    nemesis_record: Option<NemesisRecord>,
    nemesis_spawned: bool,
    // combat extras
    is_boss_fight: bool,
    gauntlet_stage: u8,     // 0=off, 1/2/3=fight #
    gauntlet_enemies: Vec<Enemy>,
    loot_pending: Option<Item>,
    current_mana: i64,
    // room event
    room_event: RoomEvent,
    // shop state
    shop_items: Vec<(Item, i64)>,
    shop_heal_cost: i64,
    shop_cursor: usize,
    // crafting state
    craft_phase: CraftPhase,
    craft_item_cursor: usize,
    craft_op_cursor: usize,
    craft_message: String,
}

impl State {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        State {
            screen: AppScreen::Title,
            player: None, floor: None, enemy: None,
            combat_state: None, last_roll: None,
            combat_log: Vec::new(),
            seed, floor_seed: seed, frame: 0,
            selected_menu: 0, cc_class: 0, cc_bg: 0, cc_diff: 1,
            mode_cursor: 0, game_mode: GameMode::Infinite,
            boon_options: Boon::random_three(seed), boon_cursor: 0,
            floor_num: 1, max_floor: u32::MAX, is_cursed_floor: false,
            nemesis_record: None, nemesis_spawned: false,
            is_boss_fight: false,
            gauntlet_stage: 0, gauntlet_enemies: Vec::new(),
            loot_pending: None, current_mana: 0,
            room_event: RoomEvent::empty(),
            shop_items: Vec::new(), shop_heal_cost: 20, shop_cursor: 0,
            craft_phase: CraftPhase::SelectItem,
            craft_item_cursor: 0, craft_op_cursor: 0,
            craft_message: String::new(),
        }
    }

    fn max_mana(&self) -> i64 {
        self.player.as_ref().map(|p| (p.stats.mana + 50).max(50)).unwrap_or(50)
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.combat_log.push(msg.into());
        if self.combat_log.len() > 300 { self.combat_log.remove(0); }
    }

    fn apply_stat_modifier(&mut self, stat: &str, val: i64) {
        if let Some(ref mut p) = self.player {
            match stat {
                "vitality"  => { p.stats.vitality  += val; p.max_hp = (50 + p.stats.vitality*3 + p.stats.force).max(1); }
                "force"     => p.stats.force     += val,
                "mana"      => p.stats.mana      += val,
                "cunning"   => p.stats.cunning   += val,
                "precision" => p.stats.precision += val,
                "entropy"   => p.stats.entropy   += val,
                "luck"      => p.stats.luck      += val,
                _ => {}
            }
        }
    }

    fn daily_seed() -> u64 {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        let day = secs / 86400;
        day.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
    }

    fn start_new_game(&mut self) {
        let class = CLASSES[self.cc_class].1.clone();
        let bg    = BACKGROUNDS[self.cc_bg].1.clone();
        let diff  = DIFFICULTIES[self.cc_diff].1.clone();
        let seed  = match self.game_mode {
            GameMode::Daily => Self::daily_seed(),
            _ => self.seed,
        };
        self.seed = seed;
        self.floor_seed = seed;
        let mut player = Character::roll_new("Hero".to_string(), class, bg, seed, diff);
        player.apply_boon(self.boon_options[self.boon_cursor]);
        self.player = Some(player);
        self.floor_num = 1;
        self.max_floor = if self.game_mode == GameMode::Story { 10 } else { u32::MAX };
        self.nemesis_record = load_nemesis();
        self.nemesis_spawned = false;
        self.current_mana = self.max_mana();
        self.screen = AppScreen::FloorNav;
        self.generate_floor_for_current();
    }

    fn generate_floor_for_current(&mut self) {
        self.floor_seed = self.floor_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.floor_num as u64 * 31337);

        // Item volatility: every 20 floors, re-roll a random item
        if self.floor_num > 1 && self.floor_num % 20 == 0 {
            if let Some(ref mut p) = self.player {
                if !p.inventory.is_empty() {
                    let vol_idx = (self.floor_seed % p.inventory.len() as u64) as usize;
                    let old = p.inventory[vol_idx].name.clone();
                    p.inventory[vol_idx] = Item::generate(self.floor_seed.wrapping_add(0x766F6C));
                    let new = p.inventory[vol_idx].name.clone();
                    self.push_log(format!("⚡ ITEM VOLATILITY: {} → {}", old, new));
                }
            }
        }

        self.is_cursed_floor = self.floor_num > 0 && self.floor_num % 25 == 0;
        if self.is_cursed_floor {
            self.push_log("☠ CURSED FLOOR! All engine outputs INVERTED this floor.".to_string());
        }

        let fl = generate_floor(self.floor_num, self.floor_seed);
        self.floor = Some(fl);

        if let Some(ref mut p) = self.player {
            p.floor = self.floor_num;
        }
    }

    fn advance_floor_room(&mut self) {
        let at_end = self.floor.as_ref()
            .map(|f| f.current_room + 1 >= f.rooms.len())
            .unwrap_or(true);
        if at_end {
            // Check victory condition
            if self.floor_num >= self.max_floor {
                self.screen = AppScreen::Victory;
                self.save_score_now();
                return;
            }
            self.floor_num += 1;
            self.generate_floor_for_current();
        } else {
            self.floor.as_mut().map(|f| f.advance());
        }
        if let Some(ref mut p) = self.player { p.rooms_cleared += 1; }
        // The Hunger (floor 50+)
        let hunger_trigger = self.player.as_ref()
            .map(|p| p.floor >= 50 && p.rooms_without_kill >= 5 && self.screen != AppScreen::Combat)
            .unwrap_or(false);
        if hunger_trigger {
            let loss = self.player.as_ref().map(|p| (p.max_hp / 20).max(1)).unwrap_or(1);
            if let Some(ref mut p) = self.player {
                p.max_hp = (p.max_hp - loss).max(1);
                if p.current_hp > p.max_hp { p.current_hp = p.max_hp; }
                p.rooms_without_kill = 0;
            }
            self.push_log(format!("THE HUNGER: -{} max HP permanently!", loss));
            if self.player.as_ref().map(|p| !p.is_alive()).unwrap_or(false) {
                self.screen = AppScreen::GameOver;
                self.save_score_now();
                return;
            }
        }
    }

    fn enter_current_room(&mut self) {
        let floor_num = self.floor_num;
        let room_seed = self.floor_seed
            .wrapping_add(self.floor.as_ref().map(|f| f.current_room as u64 * 9973).unwrap_or(0));

        // BloodPact boon: take 2 HP each room
        if matches!(self.boon_options[self.boon_cursor], Boon::BloodPact) {
            if let Some(ref mut p) = self.player { p.take_damage(2); }
            self.push_log("Blood Pact: -2 HP".to_string());
            if self.player.as_ref().map(|p| !p.is_alive()).unwrap_or(false) {
                self.screen = AppScreen::GameOver;
                self.save_score_now();
                return;
            }
        }

        let room_type = self.floor.as_ref()
            .map(|f| f.current().room_type.clone())
            .unwrap_or(RoomType::Empty);
        let room_desc = self.floor.as_ref()
            .map(|f| f.current().description.clone())
            .unwrap_or_default();

        match room_type {
            RoomType::Combat | RoomType::Boss => {
                let is_boss = room_type == RoomType::Boss;

                // Nemesis spawn check
                if !self.nemesis_spawned {
                    if let Some(ref nemesis) = self.nemesis_record.clone() {
                        let spawn_roll = room_seed.wrapping_mul(0x6E656D6573697300) % 100;
                        let spawn_chance = if floor_num >= nemesis.floor_killed_at { 40 } else { 20 };
                        if floor_num >= 3 && spawn_roll < spawn_chance {
                            self.nemesis_spawned = true;
                            let base_floor = nemesis.floor_killed_at;
                            let mut nem_enemy = generate_enemy(base_floor.max(1), room_seed);
                            nem_enemy.name = format!("★ {}", nemesis.enemy_name);
                            nem_enemy.hp = (nem_enemy.hp * (100 + nemesis.hp_bonus_pct as i64) / 100).max(1);
                            nem_enemy.max_hp = nem_enemy.hp;
                            nem_enemy.base_damage = (nem_enemy.base_damage * (100 + nemesis.damage_bonus_pct as i64) / 100).max(1);
                            nem_enemy.xp_reward *= 5;
                            nem_enemy.gold_reward *= 3;
                            self.push_log(format!("☠ NEMESIS RETURNS: {}!", nem_enemy.name));
                            self.push_log(format!("HP +{}%  DMG +{}%", nemesis.hp_bonus_pct, nemesis.damage_bonus_pct));
                            self.enemy = Some(nem_enemy);
                            self.is_boss_fight = true;
                            self.gauntlet_stage = 0;
                            self.combat_state = Some(CombatState::new(room_seed));
                            if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                            self.screen = AppScreen::Combat;
                            return;
                        }
                    }
                }

                // Boss gauntlet: every 10 floors boss room = 3-fight gauntlet
                if is_boss && floor_num % 10 == 0 {
                    let mut enemies = Vec::new();
                    let mut e1 = generate_enemy(floor_num, room_seed.wrapping_add(1));
                    e1.hp = (e1.hp as f64 * 2.0) as i64; e1.max_hp = e1.hp;
                    let mut e2 = generate_enemy(floor_num, room_seed.wrapping_add(2));
                    e2.hp = (e2.hp as f64 * 3.0) as i64; e2.max_hp = e2.hp;
                    e2.base_damage = (e2.base_damage as f64 * 1.5) as i64;
                    let dr = destiny_roll(0.5, room_seed.wrapping_add(31337));
                    let pm = (dr.final_value + 1.5).max(0.5);
                    let mut e3 = generate_enemy(floor_num, room_seed.wrapping_add(3));
                    e3.hp = ((e3.hp as f64 * 4.0 * pm) as i64).max(1); e3.max_hp = e3.hp;
                    e3.base_damage = ((e3.base_damage as f64 * 2.0 * pm) as i64).max(1);
                    e3.xp_reward *= 5; e3.gold_reward *= 5;
                    enemies.push(e1); enemies.push(e2); enemies.push(e3);
                    self.gauntlet_enemies = enemies;
                    self.gauntlet_stage = 1;
                    let first = self.gauntlet_enemies.remove(0);
                    self.enemy = Some(first);
                    self.is_boss_fight = false;
                    self.push_log("★ BOSS GAUNTLET! 3 fights. No healing.".to_string());
                    self.push_log("Fight 1/3".to_string());
                    self.combat_state = Some(CombatState::new(room_seed));
                    if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                    self.screen = AppScreen::Combat;
                    return;
                }

                // Unique boss spawn (floor 50+: 20% chance; floor 100+: every 3rd room)
                let unique_roll = room_seed.wrapping_mul(0x756E697175650000) % 100;
                let spawn_unique = (floor_num >= 100 && self.floor.as_ref().map(|f| f.current_room).unwrap_or(0) % 3 == 0)
                    || (floor_num >= 50 && !is_boss && unique_roll < 20)
                    || (is_boss && floor_num % 5 == 0);
                if spawn_unique {
                    if let Some(boss_id) = random_unique_boss(floor_num, room_seed) {
                        let bname = boss_name(boss_id);
                        let mut boss_enemy = generate_enemy(floor_num + 2, room_seed);
                        boss_enemy.name = bname.to_string();
                        boss_enemy.hp = (boss_enemy.hp as f64 * 3.0) as i64;
                        boss_enemy.max_hp = boss_enemy.hp;
                        boss_enemy.base_damage = (boss_enemy.base_damage as f64 * 2.0) as i64;
                        boss_enemy.xp_reward *= 5; boss_enemy.gold_reward *= 5;
                        self.push_log(format!("★ UNIQUE BOSS: {}!", bname));
                        self.enemy = Some(boss_enemy);
                        self.is_boss_fight = true;
                        self.gauntlet_stage = 0;
                        self.combat_state = Some(CombatState::new(room_seed));
                        if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                        self.screen = AppScreen::Combat;
                        return;
                    }
                }

                // Normal enemy
                let room = self.floor.as_ref().map(|f| f.current().clone()).unwrap();
                let mut enemy = room_enemy(&room);
                // StatMirror
                if enemy.floor_ability == FloorAbility::StatMirror {
                    let (sname, sval) = self.player.as_ref().map(|p| p.highest_stat()).unwrap_or(("force", 10));
                    enemy.hp = sval.max(1); enemy.max_hp = enemy.hp;
                    self.push_log(format!("⚠ STAT MIRROR: enemy HP = your {} ({})", sname, sval));
                }
                if enemy.floor_ability == FloorAbility::NullifyAura {
                    self.push_log("⚠ NULLIFY AURA: first action returns 0.0!".to_string());
                }
                if enemy.floor_ability == FloorAbility::EngineTheft {
                    self.push_log("⚠ ENGINE THEFT: each hit steals 1 engine!".to_string());
                }
                if is_boss {
                    enemy.hp = (enemy.hp as f64 * 2.5) as i64; enemy.max_hp = enemy.hp;
                    enemy.base_damage = (enemy.base_damage as f64 * 1.8) as i64;
                    enemy.xp_reward *= 3; enemy.gold_reward *= 3;
                    self.push_log("★ BOSS BATTLE ★".to_string());
                }
                self.enemy = Some(enemy);
                self.is_boss_fight = is_boss;
                self.gauntlet_stage = 0;
                self.combat_state = Some(CombatState::new(room_seed));
                if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                self.screen = AppScreen::Combat;
            }

            RoomType::Treasure => {
                let item = Item::generate(room_seed);
                let gold_bonus = ((room_seed % 30 + 10) as i64) * floor_num as i64;
                let mut ev = RoomEvent::empty();
                ev.title = "★ TREASURE ROOM ★".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    format!("You find {} gold!", gold_bonus), String::new(),
                    format!("Item: {}", item.name),
                    format!("Rarity: {}", item.rarity.name()),
                ];
                for m in &item.stat_modifiers {
                    ev.lines.push(format!("  {:+} {}", m.value, m.stat));
                }
                ev.lines.push(String::new());
                ev.lines.push("[P] Pick up   [Enter] Leave".to_string());
                ev.gold_delta = gold_bonus;
                ev.pending_item = Some(item);
                if room_seed % 4 == 0 {
                    let spell = Spell::generate(room_seed.wrapping_add(54321));
                    ev.lines.push(String::new());
                    ev.lines.push(format!("+ SPELL SCROLL: {}", spell.name));
                    ev.lines.push(format!("  {}mp  ×{:.1} scaling", spell.mana_cost, spell.scaling_factor.abs()));
                    ev.lines.push("[L] Learn spell   [Enter] Leave scroll".to_string());
                    ev.pending_spell = Some(spell);
                }
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Shop => {
                let mut npc = shop_npc(floor_num, room_seed);
                let heal_cost = 15 + floor_num as i64 * 2;
                let cunning = self.player.as_ref().map(|p| p.stats.cunning).unwrap_or(0);
                let npc_items: Vec<Item> = npc.inventory.drain(..).collect();
                let shop: Vec<(Item, i64)> = npc_items.into_iter()
                    .map(|item| {
                        let price = npc.sale_price(item.value, cunning);
                        (item, price)
                    })
                    .collect();
                self.shop_items = shop;
                self.shop_heal_cost = heal_cost;
                self.shop_cursor = 0;
                self.screen = AppScreen::Shop;
            }

            RoomType::Shrine => {
                let entropy = self.player.as_ref().map(|p| p.stats.entropy as f64 * 0.01).unwrap_or(0.1);
                let roll = chaos_roll_verbose(entropy, room_seed);
                self.last_roll = Some(roll.clone());
                let stats: &[&'static str] = &["vitality","force","mana","cunning","precision","entropy","luck"];
                let stat_name = stats[(room_seed % stats.len() as u64) as usize];
                let buff = 3 + (roll.to_range(1, 10) as i64) + floor_num as i64 / 2;
                let hp_restore = self.player.as_ref().map(|p| p.max_hp / 5).unwrap_or(10);
                let mut ev = RoomEvent::empty();
                ev.title = "~ SHRINE ~".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    format!("Chaos value: {:.4}", roll.final_value), String::new(),
                    format!("The shrine blesses you! +{} {}", buff, stat_name),
                    format!("You feel restored. +{} HP", hp_restore),
                    String::new(), "[Enter] Accept blessing".to_string(),
                ];
                ev.stat_bonuses = vec![(stat_name, buff)];
                ev.hp_delta = hp_restore;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Trap => {
                let player_ref = self.player.as_ref().unwrap();
                let diff = match floor_num {
                    1..=3 => SkillDiff::Easy, 4..=7 => SkillDiff::Medium, _ => SkillDiff::Hard,
                };
                let check = perform_skill_check(player_ref, SkillType::Perception, diff, room_seed);
                self.last_roll = Some(check.chaos_result.clone());
                let trap_damage = if check.passed { 0 } else { 5 + floor_num as i64 * 3 + (room_seed % 10) as i64 };
                let mut ev = RoomEvent::empty();
                ev.title = "! TRAP ROOM !".to_string();
                let mut lines = vec![room_desc, String::new()];
                for line in check.display_lines() { lines.push(line); }
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
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Portal => {
                let mut ev = RoomEvent::empty();
                ev.title = "^ PORTAL ^".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    "A shimmering rift to the next floor.".to_string(),
                    String::new(),
                    "[P] Step through portal   [Enter] Resist".to_string(),
                ];
                ev.portal_available = true;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Empty => {
                let hp_gain = 5 + floor_num as i64 * 2;
                let mut ev = RoomEvent::empty();
                ev.title = "  EMPTY ROOM  ".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    format!("The stillness restores you. +{} HP", hp_gain),
                    String::new(), "[Enter] Continue".to_string(),
                ];
                ev.hp_delta = hp_gain;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::ChaosRift => {
                let entropy = self.player.as_ref().map(|p| p.stats.entropy as f64 * 0.015).unwrap_or(0.1);
                let roll = chaos_roll_verbose(entropy, room_seed);
                self.last_roll = Some(roll.clone());
                let outcome = room_seed.wrapping_mul(floor_num as u64 * 7 + 1) % 6;
                let mut ev = RoomEvent::empty();
                ev.title = "∞ CHAOS RIFT ∞".to_string();
                ev.lines = vec![
                    "REALITY ERROR. MATHEMATICAL EXCEPTION.".to_string(), String::new(),
                    format!("Chaos value: {:.4}", roll.final_value), String::new(),
                ];
                match outcome {
                    0 => {
                        let gold = ((room_seed % 100 + 50) as i64) * floor_num as i64;
                        ev.lines.push(format!("CHAOS BOUNTY: +{} gold!", gold));
                        ev.gold_delta = gold;
                    }
                    1 => {
                        let dmg = self.player.as_ref().map(|p| (p.max_hp / 4).max(1)).unwrap_or(10);
                        ev.lines.push(format!("CHAOS PUNISHMENT: -{} HP!", dmg));
                        ev.damage_taken = dmg;
                    }
                    2 => {
                        let bonus = 5 + floor_num as i64;
                        ev.lines.push(format!("CHAOS ASCENSION: +{} Entropy!", bonus));
                        ev.stat_bonuses = vec![("entropy", bonus)];
                    }
                    3 => {
                        let heal = self.player.as_ref().map(|p| p.max_hp / 3).unwrap_or(20);
                        ev.lines.push(format!("CHAOS BLESSING: +{} HP!", heal));
                        ev.hp_delta = heal;
                    }
                    4 => {
                        let gold_loss = self.player.as_ref().map(|p| p.gold / 4).unwrap_or(0);
                        let luck = 10 + floor_num as i64;
                        ev.lines.push(format!("CHAOS TRADE: -{} gold, +{} Luck!", gold_loss, luck));
                        ev.gold_delta = -gold_loss;
                        ev.stat_bonuses = vec![("luck", luck)];
                    }
                    _ => {
                        ev.lines.push("CHAOS HARMONY: All stats +1!".to_string());
                        ev.stat_bonuses = vec![
                            ("vitality",1),("force",1),("mana",1),("cunning",1),
                            ("precision",1),("entropy",1),("luck",1),
                        ];
                    }
                }
                ev.lines.push(String::new());
                ev.lines.push("[Enter] Accept fate".to_string());
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::CraftingBench => {
                self.craft_phase = CraftPhase::SelectItem;
                self.craft_item_cursor = 0;
                self.craft_op_cursor = 0;
                self.craft_message = "Choose an item to craft.".to_string();
                self.screen = AppScreen::Crafting;
            }
        }
    }

    fn resolve_combat_action(&mut self, action: CombatAction) {
        let (player, enemy, cstate) = match (&mut self.player, &mut self.enemy, &mut self.combat_state) {
            (Some(p), Some(e), Some(cs)) => (p, e, cs),
            _ => return,
        };

        let level_before = player.level;
        let (events, outcome) = resolve_action(player, enemy, action, cstate);

        if let Some(ref roll) = cstate.last_roll {
            self.last_roll = Some(roll.clone());
        }

        for ev in &events {
            self.combat_log.push(ev.to_display_string());
        }

        // Chaos engine trace
        if let Some(ref roll) = self.last_roll.clone() {
            for line in roll.display_lines().iter().take(4) {
                self.combat_log.push(format!("  {}", line));
            }
        }

        // Tick status effects (start of each new turn after action)
        if let Some(ref mut p) = self.player {
            let (_dmg, msgs) = p.tick_status_effects();
            for m in msgs { self.combat_log.push(m); }
        }

        // Level up check
        let (level_after, skill_pts) = self.player.as_ref()
            .map(|p| (p.level, p.skill_points)).unwrap_or((0, 0));
        if level_after > level_before {
            self.push_log(format!("★ LEVEL UP! Now level {}!", level_after));
            if skill_pts > 0 {
                self.push_log(format!("  {} skill point(s) available!", skill_pts));
            }
        }

        match outcome {
            CombatOutcome::PlayerWon { xp, gold } => {
                self.push_log(format!("Victory! +{} XP  +{} gold", xp, gold));
                if let Some(ref mut p) = self.player {
                    p.kills += 1;
                    let kills_before = p.kills;
                    if p.floor >= 50 {
                        p.rooms_without_kill = 0;
                    }
                    let _ = kills_before;
                }

                // Nemesis tracking: if nemesis killed, clear it and reward
                if let Some(ref nem) = self.nemesis_record.clone() {
                    if self.enemy.as_ref().map(|e| e.name.contains(&nem.enemy_name)).unwrap_or(false) {
                        clear_nemesis();
                        self.nemesis_record = None;
                        self.push_log("☆ Nemesis defeated! Grudge settled.".to_string());
                        if let Some(ref mut p) = self.player {
                            let (sname, _) = p.highest_stat();
                            match sname {
                                "Vitality"  => p.stats.vitality  += 50,
                                "Force"     => p.stats.force     += 50,
                                "Mana"      => p.stats.mana      += 50,
                                "Cunning"   => p.stats.cunning   += 50,
                                "Precision" => p.stats.precision += 50,
                                "Entropy"   => p.stats.entropy   += 50,
                                _           => p.stats.luck      += 50,
                            }
                        }
                    }
                }

                // Loot drop
                let loot_seed = self.floor_seed.wrapping_add(self.frame).wrapping_mul(6364136223846793005);
                let drop_chance = if self.is_boss_fight { 100 } else { 40 };
                if loot_seed % 100 < drop_chance {
                    let loot = Item::generate(loot_seed);
                    self.push_log(format!("★ Item dropped: {}!", loot.name));
                    self.loot_pending = Some(loot);
                }

                // Boss gauntlet: advance to next fight
                if self.gauntlet_stage > 0 && !self.gauntlet_enemies.is_empty() {
                    self.gauntlet_stage += 1;
                    let next = self.gauntlet_enemies.remove(0);
                    self.push_log(format!("GAUNTLET: Fight {}/3", self.gauntlet_stage));
                    self.enemy = Some(next);
                    let ns = self.floor_seed.wrapping_add(self.gauntlet_stage as u64 * 1337);
                    self.combat_state = Some(CombatState::new(ns));
                    if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                    return; // Stay in combat
                }

                self.gauntlet_stage = 0;
                self.enemy = None;
                // Show loot if pending, else floor nav
                if self.loot_pending.is_some() {
                    let loot = self.loot_pending.take().unwrap();
                    self.room_event = RoomEvent::empty();
                    self.room_event.title = "★ LOOT DROPPED ★".to_string();
                    self.room_event.lines = vec![
                        format!("Enemy dropped: {}", loot.name),
                        format!("Rarity: {}", loot.rarity.name()),
                    ];
                    for m in &loot.stat_modifiers {
                        self.room_event.lines.push(format!("  {:+} {}", m.value, m.stat));
                    }
                    self.room_event.lines.push(String::new());
                    self.room_event.lines.push("[P] Pick up   [Enter] Leave".to_string());
                    self.room_event.pending_item = Some(loot);
                    self.screen = AppScreen::RoomView;
                } else {
                    self.advance_floor_room();
                    if self.screen != AppScreen::GameOver && self.screen != AppScreen::Victory {
                        self.screen = AppScreen::FloorNav;
                    }
                }
            }

            CombatOutcome::PlayerDied => {
                // Save nemesis
                let enemy_name = self.enemy.as_ref().map(|e| e.name.clone()).unwrap_or_default();
                let enemy_dmg  = self.enemy.as_ref().map(|e| e.base_damage).unwrap_or(5);
                if let Some(ref p) = self.player {
                    let method = if p.spells_cast > p.kills * 2 { "spell" } else { "physical" };
                    let nem = NemesisRecord::new(
                        enemy_name.clone(), p.floor, enemy_dmg,
                        p.class.name().to_string(), method,
                    );
                    save_nemesis(&nem);
                    self.push_log(format!("☠ {} is now your Nemesis.", enemy_name));
                }
                self.save_score_now();
                self.screen = AppScreen::GameOver;
            }

            CombatOutcome::PlayerFled => {
                self.push_log("You escaped into the chaos!".to_string());
                self.enemy = None;
                if let Some(ref mut p) = self.player { p.rooms_without_kill += 1; }
                self.advance_floor_room();
                if self.screen != AppScreen::GameOver && self.screen != AppScreen::Victory {
                    self.screen = AppScreen::FloorNav;
                }
            }

            CombatOutcome::Ongoing => {} // stay in combat
        }
    }

    fn save_score_now(&mut self) {
        if let Some(ref p) = self.player {
            let score_val = p.xp + p.gold as u64 + (p.kills * 100) as u64 + (p.floor as u64 * 500);
            let entry = ScoreEntry::new(
                p.name.clone(), p.class.name().to_string(),
                score_val, p.floor, p.kills, 0,
            );
            let _ = save_score(entry);
        }
    }
}

// ─── CONST LISTS ──────────────────────────────────────────────────────────────

const CLASSES: &[(&str, CharacterClass)] = &[
    ("Mage",        CharacterClass::Mage),
    ("Berserker",   CharacterClass::Berserker),
    ("Ranger",      CharacterClass::Ranger),
    ("Thief",       CharacterClass::Thief),
    ("Necromancer", CharacterClass::Necromancer),
    ("Alchemist",   CharacterClass::Alchemist),
    ("Paladin",     CharacterClass::Paladin),
    ("VoidWalker",  CharacterClass::VoidWalker),
];

const BACKGROUNDS: &[(&str, Background)] = &[
    ("Scholar",   Background::Scholar),
    ("Wanderer",  Background::Wanderer),
    ("Gladiator", Background::Gladiator),
    ("Outcast",   Background::Outcast),
];

const DIFFICULTIES: &[(&str, Difficulty)] = &[
    ("Easy",   Difficulty::Easy),
    ("Normal", Difficulty::Normal),
    ("Brutal", Difficulty::Brutal),
    ("Chaos",  Difficulty::Chaos),
];

// ─── GAME STATE IMPL ──────────────────────────────────────────────────────────

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.frame += 1;

        match self.screen.clone() {
            AppScreen::Title            => self.draw_title(ctx),
            AppScreen::ModeSelect       => self.draw_mode_select(ctx),
            AppScreen::CharacterCreation => self.draw_char_creation(ctx),
            AppScreen::BoonSelect       => self.draw_boon_select(ctx),
            AppScreen::FloorNav         => self.draw_floor_nav(ctx),
            AppScreen::RoomView         => self.draw_room_view(ctx),
            AppScreen::Combat           => self.draw_combat(ctx),
            AppScreen::Shop             => self.draw_shop(ctx),
            AppScreen::Crafting         => self.draw_crafting(ctx),
            AppScreen::GameOver         => self.draw_game_over(ctx),
            AppScreen::Victory          => self.draw_victory(ctx),
            AppScreen::Scoreboard       => self.draw_scoreboard(ctx),
        }

        self.handle_input(ctx);
    }
}

// ─── DRAW HELPERS ─────────────────────────────────────────────────────────────

fn hbar(ctx: &mut BTerm, x: i32, y: i32, w: i32, cur: i64, max: i64, fc: (u8,u8,u8)) {
    let f = if max > 0 { ((cur * w as i64) / max.max(1)).clamp(0, w as i64) as i32 } else { 0 };
    for i in 0..w {
        let ch  = if i < f { 219u16 } else { 176u16 };
        let col = if i < f { RGB::named(fc) } else { RGB::named(DARK_GRAY) };
        ctx.set(x + i, y, col, RGB::named(BLACK), ch);
    }
}

fn hp_col(pct: f32) -> (u8,u8,u8) {
    if pct > 0.6 { GREEN } else if pct > 0.3 { YELLOW } else { RED }
}

fn room_color(rt: &RoomType) -> (u8,u8,u8) {
    match rt {
        RoomType::Combat        => RED,
        RoomType::Boss          => (200, 0, 0),
        RoomType::Treasure      => YELLOW,
        RoomType::Shop          => CYAN,
        RoomType::Shrine        => MAGENTA,
        RoomType::Trap          => ORANGE,
        RoomType::Portal        => (100, 200, 255),
        RoomType::Empty         => DARK_GRAY,
        RoomType::ChaosRift     => (180, 0, 255),
        RoomType::CraftingBench => (100, 255, 100),
    }
}

// ─── TITLE ────────────────────────────────────────────────────────────────────

impl State {
    fn draw_title(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(30, 4,  yel, bg, "  ██████╗██╗  ██╗ █████╗  ██████╗ ███████╗");
        ctx.print_color(30, 5,  yel, bg, " ██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝");
        ctx.print_color(30, 6,  col, bg, " ██║     ███████║███████║██║   ██║███████╗ ");
        ctx.print_color(30, 7,  col, bg, " ╚██████╗██║  ██║██║  ██║╚██████╔╝███████║");
        ctx.print_color(30, 8,  col, bg, "  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝");
        ctx.print_color(38, 10, dim, bg, "R P G   —   Where Math Goes To Die");
        ctx.print_color(28, 13, RGB::named(MAGENTA), bg, "Graphical Edition — Fullscreen");

        let opts = ["  New Game","  Scoreboard","  Quit"];
        let ox = 46i32; let oy = 22i32;
        ctx.draw_box(ox-2, oy-2, 32, opts.len() as i32+3, col, bg);
        for (i, opt) in opts.iter().enumerate() {
            let (fg, pfx) = if i == self.selected_menu { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(ox, oy + i as i32, fg, bg, &format!("{} {}", pfx, opt));
        }
        ctx.print_color(4, 46, dim, bg, "↑↓ Navigate   Enter Select   Q Quit");
    }

    // ─── MODE SELECT ──────────────────────────────────────────────────────────

    fn draw_mode_select(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(44, 2, yel, bg, "─── SELECT MODE ───");
        let modes = [
            ("Story Mode",    "10 floors. A complete narrative arc."),
            ("Infinite Mode", "Descend forever. Score for the leaderboard."),
            ("Daily Seed",    "Same seed for all players today. Race to top."),
        ];
        for (i, (name, desc)) in modes.iter().enumerate() {
            let y = 10 + i as i32 * 6;
            let (fg, pfx) = if i == self.mode_cursor { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(20, y,   fg, bg, &format!("{} {}", pfx, name));
            ctx.print_color(22, y+1, dim, bg, desc);
        }
        ctx.print_color(4, 46, dim, bg, "↑↓ Navigate   Enter Select   Esc Back");
    }

    // ─── CHAR CREATION ────────────────────────────────────────────────────────

    fn draw_char_creation(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(44, 2, yel, bg, "─── CHARACTER CREATION ───");

        ctx.print_color(3, 5, col, bg, "CLASS  ↑↓");
        ctx.print_color(3, 6, dim, bg, "──────────────────");
        for (i, (name, _)) in CLASSES.iter().enumerate() {
            let (fg, pfx) = if i == self.cc_class { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(3, 7 + i as i32, fg, bg, &format!("{} {}", pfx, name));
        }

        let class = &CLASSES[self.cc_class].1;
        ctx.print_color(3, 17, col, bg, "PASSIVE");
        ctx.print_color(3, 18, yel, bg, class.passive_name());
        let desc = class.passive_desc();
        let mut row = 19i32; let mut line = String::new();
        for w in desc.split_whitespace() {
            if line.len() + w.len() + 1 > 34 { ctx.print_color(3, row, dim, bg, &line); line = w.to_string(); row += 1; }
            else { if !line.is_empty() { line.push(' '); } line.push_str(w); }
        }
        if !line.is_empty() { ctx.print_color(3, row, dim, bg, &line); }

        ctx.print_color(28, 5, col, bg, "BACKGROUND  ←→");
        ctx.print_color(28, 6, dim, bg, "──────────────────");
        for (i, (name, _)) in BACKGROUNDS.iter().enumerate() {
            let (fg, pfx) = if i == self.cc_bg { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(28, 7 + i as i32, fg, bg, &format!("{} {}", pfx, name));
        }

        ctx.print_color(50, 5, col, bg, "DIFFICULTY  Tab");
        ctx.print_color(50, 6, dim, bg, "──────────────────");
        for (i, (name, _)) in DIFFICULTIES.iter().enumerate() {
            let c = match i { 0 => RGB::named(GREEN), 1 => RGB::named(YELLOW), 2 => RGB::named(ORANGE), _ => RGB::named(RED) };
            let pfx = if i == self.cc_diff { "►" } else { " " };
            ctx.print_color(50, 7 + i as i32, c, bg, &format!("{} {}", pfx, name));
        }

        let portrait = class.ascii_art();
        ctx.print_color(85, 5, col, bg, "PORTRAIT");
        for (i, l) in portrait.lines().enumerate() {
            ctx.print_color(85, 7 + i as i32, RGB::named(WHITE), bg, l);
        }

        ctx.print_color(3, 46, col, bg, "[ ENTER ] Continue to Boon Select");
        ctx.print_color(3, 47, dim, bg, "↑↓=class  ←→=background  Tab=difficulty  Esc=back");
    }

    // ─── BOON SELECT ──────────────────────────────────────────────────────────

    fn draw_boon_select(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let mag = RGB::named(MAGENTA); let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(40, 2, yel, bg, "─── CHOOSE YOUR BOON ───");
        ctx.print_color(30, 4, mag, bg, "A gift from the chaos. Choose wisely.");

        for (i, boon) in self.boon_options.iter().enumerate() {
            let y = 8 + i as i32 * 8;
            let (fg, pfx) = if i == self.boon_cursor { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(10, y,   fg,  bg, &format!("{} [{}] {}", pfx, i+1, boon.name()));
            ctx.print_color(12, y+1, dim, bg, boon.description());
        }

        ctx.print_color(4, 46, col, bg, "[ Enter / 1-3 ] Select   Esc = Back");
    }

    // ─── FLOOR NAV ────────────────────────────────────────────────────────────

    fn draw_floor_nav(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let red = RGB::named(RED); let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);

        let (pname, pclass, plv, pfloor, pkills, pgold, pxp, php, pmhp, pstatus, pcorruption,
             pkills_u, prwk) = match &self.player {
            Some(p) => (p.name.clone(), p.class.name(), p.level, p.floor,
                        p.kills, p.gold, p.xp, p.current_hp, p.max_hp,
                        p.status_badge_line(), p.corruption, p.kills, p.rooms_without_kill),
            None => { self.screen = AppScreen::Title; return; }
        };

        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(3, 2, yel, bg, &format!("FLOOR {}  —  {}  Lv.{} {}", pfloor, pname, plv, pclass));
        ctx.print_color(3, 3, dim, bg, &format!("Kills: {}  Gold: {}  XP: {}  Corruption: {}", pkills, pgold, pxp, pcorruption));

        // Cursed floor warning
        if self.is_cursed_floor {
            ctx.print_color(60, 2, red, bg, "☠ CURSED FLOOR ☠");
            ctx.print_color(60, 3, dim, bg, "Engine outputs INVERTED");
        }

        // The Hunger warning
        if pfloor >= 50 && prwk >= 3 {
            let rooms_left = 5u32.saturating_sub(prwk);
            ctx.print_color(3, 4, red, bg, &format!("THE HUNGER: {} rooms dry. {} more = -5% max HP", prwk, rooms_left));
        }

        // Nemesis warning
        if let Some(ref nem) = self.nemesis_record {
            ctx.print_color(3, 5, red, bg, &format!("☠ NEMESIS: {} lurks (floor {})", nem.enemy_name, nem.floor_killed_at));
        }

        let pp = php as f32 / pmhp.max(1) as f32;
        ctx.print_color(3, 7,  RGB::named(hp_col(pp)), bg, &format!("HP {}/{}", php, pmhp));
        hbar(ctx, 3, 8, 50, php, pmhp, hp_col(pp));
        ctx.print_color(3, 9,  RGB::named(BLUE), bg, &format!("MP {}/{}", self.current_mana, self.max_mana()));
        hbar(ctx, 3, 10, 50, self.current_mana, self.max_mana(), BLUE);
        if !pstatus.is_empty() {
            ctx.print_color(3, 11, RGB::named(MAGENTA), bg, &format!("Status: {}", pstatus));
        }

        // Minimap
        ctx.print_color(3, 13, col, bg, "FLOOR MAP");
        ctx.print_color(3, 14, dim, bg, "─────────────────────────────────────────────────────────────────────");
        if let Some(ref floor) = self.floor {
            let per_row = 20usize;
            for (i, room) in floor.rooms.iter().enumerate() {
                let rx = 3 + (i % per_row) as i32 * 5;
                let ry = 15 + (i / per_row) as i32 * 2;
                let sym = room.room_type.icon();
                let (r_col, marker) = if i == floor.current_room {
                    (RGB::named(WHITE), format!("[{}]", sym.trim_matches(|c| c == '[' || c == ']')))
                } else if i < floor.current_room {
                    (dim, "···".to_string())
                } else {
                    (RGB::named(room_color(&room.room_type)), sym.to_string())
                };
                ctx.print_color(rx, ry, r_col, bg, &marker);
            }
            let current = floor.current();
            ctx.print_color(3, 27, RGB::named(room_color(&current.room_type)), bg,
                &format!("Next: {}  —  {}", current.room_type.name(), current.description));
        }

        // Mode banner
        let mode_str = match self.game_mode {
            GameMode::Story   => format!("STORY MODE — Floor {}/{}", pfloor, 10),
            GameMode::Infinite => "INFINITE MODE".to_string(),
            GameMode::Daily   => "DAILY SEED".to_string(),
        };
        ctx.print_color(3, 29, RGB::named(MAGENTA), bg, &mode_str);

        // Recent log
        ctx.print_color(3, 31, col, bg, "LOG");
        let log_start = self.combat_log.len().saturating_sub(6);
        for (i, line) in self.combat_log[log_start..].iter().enumerate() {
            ctx.print_color(3, 32 + i as i32, dim, bg, line);
        }

        // Actions
        ctx.draw_box(1, 38, 118, 9, col, bg);
        ctx.print_color(3, 39, col, bg, "ACTIONS");
        ctx.print_color(3, 40, dim, bg, "[E] Enter room   [C] Character sheet   [S] Scoreboard   [Q] Quit");
        if self.floor.as_ref().map(|f| f.rooms_remaining() == 0).unwrap_or(false) {
            ctx.print_color(3, 41, yel, bg, "[D] Descend to next floor");
        }
        ctx.print_color(3, 42, dim, bg, "[×]=Combat  [★]=Treasure  [$]=Shop  [~]=Shrine  [!]=Trap  [B]=Boss");
        ctx.print_color(3, 43, dim, bg, "[^]=Portal  [ ]=Empty  [∞]=Rift  [⚒]=Crafting");
        let _ = pkills_u;
    }

    // ─── ROOM VIEW ────────────────────────────────────────────────────────────

    fn draw_room_view(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);

        // Apply effects once
        if !self.room_event.resolved {
            self.room_event.resolved = true;
            let gd = self.room_event.gold_delta;
            let hd = self.room_event.hp_delta;
            let dt = self.room_event.damage_taken;
            let bonuses: Vec<(&'static str, i64)> = self.room_event.stat_bonuses.clone();
            if let Some(ref mut p) = self.player {
                if gd != 0 { p.gold += gd; }
                if hd > 0  { p.heal(hd); }
                if dt > 0  { p.take_damage(dt); }
                for (stat, val) in &bonuses { self.apply_stat_modifier(stat, *val); }
            }
            // Check death from trap/chaos
            if dt > 0 {
                if let Some(ref p) = self.player {
                    if !p.is_alive() {
                        self.save_score_now();
                        self.screen = AppScreen::GameOver;
                        return;
                    }
                }
            }
        }

        let title = self.room_event.title.clone();
        ctx.print_color(40, 2, yel, bg, &title);
        for (i, line) in self.room_event.lines.iter().enumerate() {
            let fg = if line.starts_with('[') { RGB::named(WHITE) } else { dim };
            ctx.print_color(5, 5 + i as i32, fg, bg, line);
        }

        let has_item  = self.room_event.pending_item.is_some();
        let has_spell = self.room_event.pending_spell.is_some();
        let is_portal = self.room_event.portal_available;

        let y = 40;
        if has_item  { ctx.print_color(5, y,   RGB::named(WHITE), bg, "[P] Pick up item   [Enter] Leave it"); }
        if has_spell { ctx.print_color(5, y+1, RGB::named(WHITE), bg, "[L] Learn spell    [Enter] Leave scroll"); }
        if is_portal { ctx.print_color(5, y,   RGB::named(WHITE), bg, "[P] Step through portal   [Enter] Resist"); }
        if !has_item && !has_spell && !is_portal {
            ctx.print_color(5, y, RGB::named(WHITE), bg, "[Enter] Continue");
        }
    }

    // ─── COMBAT ───────────────────────────────────────────────────────────────

    fn draw_combat(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let red = RGB::named(RED); let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);

        let (pname, pclass, plv, php, pmhp, pstatus) = match &self.player {
            Some(p) => (p.name.clone(), p.class.name(), p.level, p.current_hp, p.max_hp, p.status_badge_line()),
            None => { self.screen = AppScreen::Title; return; }
        };
        let (ename, etier, ehp, emhp, esprite) = match &self.enemy {
            Some(e) => (e.name.clone(), e.tier.name().to_string(), e.hp, e.max_hp, e.ascii_sprite),
            None => { self.screen = AppScreen::FloorNav; return; }
        };

        ctx.draw_box(1, 1, 118, 48, red, bg);

        // Enemy panel
        ctx.draw_box(2, 2, 55, 22, red, bg);
        ctx.print_color(4, 3, red, bg, &format!("{} [{}]", ename, etier));
        let ep = ehp as f32 / emhp.max(1) as f32;
        ctx.print_color(4, 4, RGB::named(hp_col(ep)), bg, &format!("HP {}/{}", ehp, emhp));
        hbar(ctx, 4, 5, 50, ehp, emhp, hp_col(ep));
        if self.is_boss_fight { ctx.print_color(4, 6, red, bg, "★ BOSS"); }
        if self.gauntlet_stage > 0 {
            ctx.print_color(4, 7, red, bg, &format!("GAUNTLET {}/3", self.gauntlet_stage));
        }
        for (i, line) in esprite.lines().enumerate().take(10) {
            ctx.print_color(4, 9 + i as i32, dim, bg, line);
        }

        // Player panel
        ctx.draw_box(59, 2, 59, 22, col, bg);
        ctx.print_color(61, 3, col, bg, &format!("{} Lv.{} {}", pname, plv, pclass));
        let pp = php as f32 / pmhp.max(1) as f32;
        ctx.print_color(61, 4, RGB::named(hp_col(pp)), bg, &format!("HP {}/{}", php, pmhp));
        hbar(ctx, 61, 5, 54, php, pmhp, hp_col(pp));
        ctx.print_color(61, 6, RGB::named(BLUE), bg, &format!("MP {}/{}", self.current_mana, self.max_mana()));
        hbar(ctx, 61, 7, 54, self.current_mana, self.max_mana(), BLUE);
        if !pstatus.is_empty() {
            ctx.print_color(61, 8, RGB::named(MAGENTA), bg, &format!("Status: {}", pstatus));
        }
        if self.is_cursed_floor {
            ctx.print_color(61, 9, red, bg, "☠ CURSED FLOOR");
        }

        // Spells list
        if let Some(ref p) = self.player {
            ctx.print_color(61, 11, yel, bg, "SPELLS (1-8)");
            for (i, spell) in p.known_spells.iter().enumerate().take(8) {
                let affordable = self.current_mana >= spell.mana_cost;
                let fg = if affordable { RGB::named(CYAN) } else { dim };
                ctx.print_color(61, 12 + i as i32, fg, bg,
                    &format!("[{}] {} ({}mp)", i+1, spell.name, spell.mana_cost));
            }
        }

        // Combat actions
        ctx.draw_box(2, 25, 116, 8, col, bg);
        ctx.print_color(4, 26, col, bg, "COMBAT ACTIONS");
        ctx.print_color(4, 27, dim, bg, "[A] Attack    [H] Heavy Attack    [D] Defend    [T] Taunt    [F] Flee");
        ctx.print_color(4, 28, dim, bg, "[1-8] Cast Spell   [Q/W/E/R/Y/U/I/O] Use Item 1-8");

        // Items
        if let Some(ref p) = self.player {
            let item_keys = ["Q","W","E","R","Y","U","I","O"];
            let mut ix = 4;
            for (i, item) in p.inventory.iter().enumerate().take(8) {
                ctx.print_color(ix, 29, dim, bg, &format!("[{}]{}", item_keys[i], item.name));
                ix += item.name.len() as i32 + 5;
                if ix > 100 { break; }
            }
        }

        // Combat log
        ctx.draw_box(2, 34, 116, 13, dim, bg);
        ctx.print_color(4, 35, col, bg, "CHAOS LOG");
        let log_start = self.combat_log.len().saturating_sub(11);
        for (i, line) in self.combat_log[log_start..].iter().enumerate() {
            let fg = if line.contains("CRIT") || line.contains("BOSS") { red }
                     else if line.contains("Victory") || line.contains("+") { yel }
                     else { dim };
            ctx.print_color(4, 36 + i as i32, fg, bg, line);
        }
    }

    // ─── SHOP ─────────────────────────────────────────────────────────────────

    fn draw_shop(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(44, 2, yel, bg, "─── SHOP ───");

        let pgold = self.player.as_ref().map(|p| p.gold).unwrap_or(0);
        ctx.print_color(3, 4, yel, bg, &format!("Your gold: {}g", pgold));

        ctx.print_color(3, 6, RGB::named(WHITE), bg,
            &format!("[H] Healing Potion — {}g (+40 HP)", self.shop_heal_cost));

        for (i, (item, price)) in self.shop_items.iter().enumerate() {
            let y = 8 + i as i32 * 4;
            let fg = if i + 1 == self.shop_cursor { RGB::named(WHITE) } else { dim };
            let pfx = if i + 1 == self.shop_cursor { "►" } else { " " };
            ctx.print_color(3, y, fg, bg,
                &format!("{} [{}] {} — {}g  ({})", pfx, i+1, item.name, price, item.rarity.name()));
            for (j, m) in item.stat_modifiers.iter().enumerate().take(2) {
                ctx.print_color(7, y + 1 + j as i32, dim, bg,
                    &format!("  {:+} {}", m.value, m.stat));
            }
        }

        ctx.print_color(3, 44, dim, bg, "[1-4] Select item   [H] Buy heal   [Enter/0] Leave shop");
        ctx.print_color(3, 45, dim, bg, "↑↓ Scroll   Enter = Buy selected");
    }

    // ─── CRAFTING ─────────────────────────────────────────────────────────────

    fn draw_crafting(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(44, 2, yel, bg, "─── CRAFTING BENCH ───");

        let has_inventory = self.player.as_ref().map(|p| !p.inventory.is_empty()).unwrap_or(false);
        if !has_inventory {
            ctx.print_color(20, 20, dim, bg, "Your inventory is empty. Nothing to craft.");
            ctx.print_color(20, 22, dim, bg, "[Esc/Enter] Leave");
            return;
        }

        match self.craft_phase {
            CraftPhase::SelectItem => {
                ctx.print_color(3, 5, col, bg, "Select item to craft (↑↓ navigate, Enter confirm):");
                if let Some(ref p) = self.player {
                    for (i, item) in p.inventory.iter().enumerate() {
                        let fg = if i == self.craft_item_cursor { RGB::named(WHITE) } else { dim };
                        let pfx = if i == self.craft_item_cursor { "►" } else { " " };
                        ctx.print_color(3, 7 + i as i32, fg, bg,
                            &format!("{} [{}] {} ({})", pfx, i+1, item.name, item.rarity.name()));
                    }
                }
                ctx.print_color(3, 44, dim, bg, "↑↓ Navigate   Enter Select item   Esc Leave");
            }
            CraftPhase::SelectOp => {
                let item_name = self.player.as_ref()
                    .and_then(|p| p.inventory.get(self.craft_item_cursor))
                    .map(|i| i.name.clone())
                    .unwrap_or_default();
                ctx.print_color(3, 5, yel, bg, &format!("Crafting: {}", item_name));

                let ops = [
                    ("[1] Reforge",    "Chaos-reroll all stat modifiers"),
                    ("[2] Augment",    "Add one new chaos-rolled modifier"),
                    ("[3] Annul",      "Remove one random modifier"),
                    ("[4] Corrupt",    "Unpredictable chaos effect"),
                    ("[5] Fuse",       "Double value + upgrade rarity"),
                    ("[6] EngineLock", "Lock chaos engine into item (costs gold)"),
                ];
                for (i, (key, desc)) in ops.iter().enumerate() {
                    let fg = if i == self.craft_op_cursor { RGB::named(WHITE) } else { dim };
                    let pfx = if i == self.craft_op_cursor { "►" } else { " " };
                    ctx.print_color(3, 8 + i as i32 * 2, fg, bg, &format!("{} {}  — {}", pfx, key, desc));
                }

                if !self.craft_message.is_empty() {
                    ctx.print_color(3, 28, yel, bg, &self.craft_message);
                }
                ctx.print_color(3, 44, dim, bg, "↑↓ Navigate   Enter Apply   Esc Back to item list");
            }
        }
    }

    // ─── GAME OVER ────────────────────────────────────────────────────────────

    fn draw_game_over(&mut self, ctx: &mut BTerm) {
        let red = RGB::named(RED); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, red, bg);
        ctx.print_color(38, 5,  red, bg, "╔══════════════════════════════════════╗");
        ctx.print_color(38, 6,  red, bg, "║         Y O U   D I E D             ║");
        ctx.print_color(38, 7,  red, bg, "║   The math has claimed another soul  ║");
        ctx.print_color(38, 8,  red, bg, "╚══════════════════════════════════════╝");

        if let Some(ref p) = self.player {
            ctx.print_color(20, 12, yel, bg, &format!("{} — {} — Lv.{}", p.name, p.class.name(), p.level));
            ctx.print_color(20, 13, dim, bg, &format!("Floor {} | Kills: {} | Gold: {} | XP: {}", p.floor, p.kills, p.gold, p.xp));
            ctx.print_color(20, 14, dim, bg, &format!("Spells cast: {}  Items used: {}  Corruption: {}", p.spells_cast, p.items_used, p.corruption));
            for (i, line) in p.run_summary().iter().enumerate().take(15) {
                ctx.print_color(20, 16 + i as i32, dim, bg, line);
            }
        }

        if let Some(ref nem) = self.nemesis_record {
            ctx.print_color(20, 34, red, bg, &format!("☠ New Nemesis: {} — will appear in your next run.", nem.enemy_name));
        }

        ctx.print_color(38, 44, dim, bg, "[Enter] Return to title   [S] Scoreboard");
    }

    // ─── VICTORY ──────────────────────────────────────────────────────────────

    fn draw_victory(&mut self, ctx: &mut BTerm) {
        let yel = RGB::named(YELLOW); let col = RGB::named(CYAN);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, yel, bg);
        ctx.print_color(35, 5,  yel, bg, "╔══════════════════════════════════════════╗");
        ctx.print_color(35, 6,  yel, bg, "║   ★ V I C T O R Y ★                     ║");
        ctx.print_color(35, 7,  yel, bg, "║   You survived 10 floors of pure math.   ║");
        ctx.print_color(35, 8,  yel, bg, "╚══════════════════════════════════════════╝");
        if let Some(ref p) = self.player {
            ctx.print_color(20, 12, col, bg, &format!("{} — {} — Lv.{}", p.name, p.class.name(), p.level));
            ctx.print_color(20, 13, dim, bg, &format!("Floor {} | Kills: {} | Gold: {} | XP: {}", p.floor, p.kills, p.gold, p.xp));
            for (i, line) in p.run_summary().iter().enumerate().take(15) {
                ctx.print_color(20, 15 + i as i32, dim, bg, line);
            }
        }
        ctx.print_color(38, 44, dim, bg, "[Enter] Return to title   [S] Scoreboard");
    }

    // ─── SCOREBOARD ───────────────────────────────────────────────────────────

    fn draw_scoreboard(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(44, 2, yel, bg, "─── SCOREBOARD ───");

        let scores = load_scores();
        if scores.is_empty() {
            ctx.print_color(40, 20, dim, bg, "No scores yet. Play and die bravely.");
        } else {
            ctx.print_color(5, 5, dim, bg, "Rank  Score      Name                Class        Floor  Kills");
            ctx.print_color(5, 6, dim, bg, "─────────────────────────────────────────────────────────────");
            for (i, s) in scores.iter().enumerate().take(20) {
                let fg = if i == 0 { yel } else { dim };
                ctx.print_color(5, 7 + i as i32, fg, bg,
                    &format!("{:3}.  {:10}  {:20} {:12} {:5}  {}", i+1, s.score, s.name, s.class, s.floor_reached, s.enemies_defeated));
            }
        }
        ctx.print_color(5, 46, dim, bg, "[Esc/Q] Back to title");
    }
}

// ─── INPUT HANDLER ────────────────────────────────────────────────────────────

impl State {
    fn handle_input(&mut self, ctx: &mut BTerm) {
        let key = match ctx.key { Some(k) => k, None => return };

        match self.screen.clone() {
            AppScreen::Title => match key {
                VirtualKeyCode::Up   => self.selected_menu = self.selected_menu.saturating_sub(1),
                VirtualKeyCode::Down => self.selected_menu = (self.selected_menu + 1).min(2),
                VirtualKeyCode::Return => match self.selected_menu {
                    0 => self.screen = AppScreen::ModeSelect,
                    1 => self.screen = AppScreen::Scoreboard,
                    _ => ctx.quit(),
                },
                VirtualKeyCode::Q => ctx.quit(),
                _ => {}
            },

            AppScreen::ModeSelect => match key {
                VirtualKeyCode::Up   => self.mode_cursor = self.mode_cursor.saturating_sub(1),
                VirtualKeyCode::Down => self.mode_cursor = (self.mode_cursor + 1).min(2),
                VirtualKeyCode::Return => {
                    self.game_mode = match self.mode_cursor { 0 => GameMode::Story, 1 => GameMode::Infinite, _ => GameMode::Daily };
                    self.screen = AppScreen::CharacterCreation;
                }
                VirtualKeyCode::Escape => self.screen = AppScreen::Title,
                _ => {}
            },

            AppScreen::CharacterCreation => match key {
                VirtualKeyCode::Up    => self.cc_class = self.cc_class.saturating_sub(1),
                VirtualKeyCode::Down  => self.cc_class = (self.cc_class + 1).min(CLASSES.len() - 1),
                VirtualKeyCode::Left  => self.cc_bg = self.cc_bg.saturating_sub(1),
                VirtualKeyCode::Right => self.cc_bg = (self.cc_bg + 1).min(BACKGROUNDS.len() - 1),
                VirtualKeyCode::Tab   => self.cc_diff = (self.cc_diff + 1) % DIFFICULTIES.len(),
                VirtualKeyCode::Return => {
                    self.boon_options = Boon::random_three(self.seed.wrapping_add(self.cc_class as u64 * 777));
                    self.boon_cursor = 0;
                    self.screen = AppScreen::BoonSelect;
                }
                VirtualKeyCode::Escape => self.screen = AppScreen::ModeSelect,
                _ => {}
            },

            AppScreen::BoonSelect => match key {
                VirtualKeyCode::Up   => self.boon_cursor = self.boon_cursor.saturating_sub(1),
                VirtualKeyCode::Down => self.boon_cursor = (self.boon_cursor + 1).min(2),
                VirtualKeyCode::Key1 => { self.boon_cursor = 0; self.start_new_game(); }
                VirtualKeyCode::Key2 => { self.boon_cursor = 1; self.start_new_game(); }
                VirtualKeyCode::Key3 => { self.boon_cursor = 2; self.start_new_game(); }
                VirtualKeyCode::Return => self.start_new_game(),
                VirtualKeyCode::Escape => self.screen = AppScreen::CharacterCreation,
                _ => {}
            },

            AppScreen::FloorNav => match key {
                VirtualKeyCode::E | VirtualKeyCode::Return => {
                    self.enter_current_room();
                }
                VirtualKeyCode::D => {
                    if self.floor.as_ref().map(|f| f.rooms_remaining() == 0).unwrap_or(false) {
                        if self.floor_num >= self.max_floor {
                            self.save_score_now();
                            self.screen = AppScreen::Victory;
                        } else {
                            self.floor_num += 1;
                            self.generate_floor_for_current();
                        }
                    }
                }
                VirtualKeyCode::C => {
                    // Character sheet is shown inline via combat log - no separate screen needed
                    if let Some(ref p) = self.player {
                        for line in p.run_summary() { self.push_log(line); }
                    }
                }
                VirtualKeyCode::S => self.screen = AppScreen::Scoreboard,
                VirtualKeyCode::Q | VirtualKeyCode::Escape => {
                    self.save_score_now();
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::RoomView => {
                let has_item  = self.room_event.pending_item.is_some();
                let has_spell = self.room_event.pending_spell.is_some();
                let is_portal = self.room_event.portal_available;
                match key {
                    VirtualKeyCode::P => {
                        if is_portal {
                            self.room_event.portal_available = false;
                            if self.floor_num >= self.max_floor {
                                self.save_score_now();
                                self.screen = AppScreen::Victory;
                            } else {
                                self.floor_num += 1;
                                self.generate_floor_for_current();
                                self.screen = AppScreen::FloorNav;
                            }
                        } else if has_item {
                            if let Some(item) = self.room_event.pending_item.take() {
                                let name = item.name.clone();
                                let mods: Vec<_> = item.stat_modifiers.iter().map(|m| (m.stat.clone(), m.value)).collect();
                                for (stat, val) in &mods { self.apply_stat_modifier(stat, *val); }
                                if let Some(ref mut p) = self.player { p.add_item(item); }
                                self.push_log(format!("Picked up {}", name));
                            }
                            if !has_spell {
                                self.advance_floor_room();
                                if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                            }
                        }
                    }
                    VirtualKeyCode::L if has_spell => {
                        if let Some(spell) = self.room_event.pending_spell.take() {
                            let name = spell.name.clone();
                            if let Some(ref mut p) = self.player { p.add_spell(spell); }
                            self.push_log(format!("Learned spell: {}", name));
                        }
                        if !self.room_event.pending_item.is_some() {
                            self.advance_floor_room();
                            if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                        }
                    }
                    VirtualKeyCode::Return | VirtualKeyCode::Escape | VirtualKeyCode::X => {
                        // Skip/leave remaining pending items
                        self.room_event.pending_item = None;
                        self.room_event.pending_spell = None;
                        if is_portal {
                            self.room_event.portal_available = false;
                        }
                        self.advance_floor_room();
                        if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                    }
                    _ => {}
                }
            },

            AppScreen::Combat => {
                let action = match key {
                    VirtualKeyCode::A => Some(CombatAction::Attack),
                    VirtualKeyCode::H => Some(CombatAction::HeavyAttack),
                    VirtualKeyCode::D => Some(CombatAction::Defend),
                    VirtualKeyCode::T => Some(CombatAction::Taunt),
                    VirtualKeyCode::F => Some(CombatAction::Flee),
                    // Spells 1-8
                    VirtualKeyCode::Key1 => Some(CombatAction::UseSpell(0)),
                    VirtualKeyCode::Key2 => Some(CombatAction::UseSpell(1)),
                    VirtualKeyCode::Key3 => Some(CombatAction::UseSpell(2)),
                    VirtualKeyCode::Key4 => Some(CombatAction::UseSpell(3)),
                    VirtualKeyCode::Key5 => Some(CombatAction::UseSpell(4)),
                    VirtualKeyCode::Key6 => Some(CombatAction::UseSpell(5)),
                    VirtualKeyCode::Key7 => Some(CombatAction::UseSpell(6)),
                    VirtualKeyCode::Key8 => Some(CombatAction::UseSpell(7)),
                    // Items Q/W/E/R/Y/U/I/O = items 1-8
                    VirtualKeyCode::Q => Some(CombatAction::UseItem(0)),
                    VirtualKeyCode::W => Some(CombatAction::UseItem(1)),
                    VirtualKeyCode::E => Some(CombatAction::UseItem(2)),
                    VirtualKeyCode::R => Some(CombatAction::UseItem(3)),
                    VirtualKeyCode::Y => Some(CombatAction::UseItem(4)),
                    VirtualKeyCode::U => Some(CombatAction::UseItem(5)),
                    VirtualKeyCode::I => Some(CombatAction::UseItem(6)),
                    VirtualKeyCode::O => Some(CombatAction::UseItem(7)),
                    _ => None,
                };
                if let Some(act) = action {
                    self.resolve_combat_action(act);
                }
            },

            AppScreen::Shop => match key {
                VirtualKeyCode::H => {
                    let cost = self.shop_heal_cost;
                    let (can_afford, pgold) = self.player.as_ref()
                        .map(|p| (p.gold >= cost, p.gold)).unwrap_or((false, 0));
                    if can_afford {
                        if let Some(ref mut p) = self.player { p.gold -= cost; p.heal_scaled(40); }
                        self.push_log(format!("Bought heal potion. +40 HP (-{}g)", cost));
                    } else {
                        self.push_log(format!("Need {}g. Have {}g.", cost, pgold));
                    }
                }
                VirtualKeyCode::Key1 | VirtualKeyCode::Key2 |
                VirtualKeyCode::Key3 | VirtualKeyCode::Key4 => {
                    let idx = match key {
                        VirtualKeyCode::Key1 => 0, VirtualKeyCode::Key2 => 1,
                        VirtualKeyCode::Key3 => 2, _ => 3,
                    };
                    if idx < self.shop_items.len() {
                        let (item, price) = self.shop_items[idx].clone();
                        if let Some(ref mut p) = self.player {
                            if p.gold >= price {
                                p.gold -= price;
                                let name = item.name.clone();
                                if item.is_weapon || item.stat_modifiers.is_empty() {
                                    p.add_item(item);
                                    self.push_log(format!("Purchased {}!", name));
                                } else {
                                    for m in item.stat_modifiers.clone() {
                                        self.apply_stat_modifier(&m.stat, m.value);
                                    }
                                    self.push_log(format!("Used {}! Stats updated.", name));
                                }
                                self.shop_items.remove(idx);
                            } else {
                                self.push_log(format!("Need {}g, have {}g.", price, self.player.as_ref().map(|p| p.gold).unwrap_or(0)));
                            }
                        }
                    }
                }
                VirtualKeyCode::Return | VirtualKeyCode::Key0 | VirtualKeyCode::Escape => {
                    self.advance_floor_room();
                    if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                }
                _ => {}
            },

            AppScreen::Crafting => match self.craft_phase {
                CraftPhase::SelectItem => match key {
                    VirtualKeyCode::Up => {
                        if self.craft_item_cursor > 0 { self.craft_item_cursor -= 1; }
                    }
                    VirtualKeyCode::Down => {
                        let len = self.player.as_ref().map(|p| p.inventory.len()).unwrap_or(0);
                        if self.craft_item_cursor + 1 < len { self.craft_item_cursor += 1; }
                    }
                    VirtualKeyCode::Return => {
                        let has_item = self.player.as_ref().map(|p| !p.inventory.is_empty()).unwrap_or(false);
                        if has_item {
                            self.craft_phase = CraftPhase::SelectOp;
                            self.craft_op_cursor = 0;
                            self.craft_message = String::new();
                        }
                    }
                    VirtualKeyCode::Escape => {
                        self.advance_floor_room();
                        if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                    }
                    _ => {}
                },
                CraftPhase::SelectOp => match key {
                    VirtualKeyCode::Up => { if self.craft_op_cursor > 0 { self.craft_op_cursor -= 1; } }
                    VirtualKeyCode::Down => { if self.craft_op_cursor < 5 { self.craft_op_cursor += 1; } }
                    VirtualKeyCode::Return => {
                        self.apply_craft_op();
                    }
                    VirtualKeyCode::Key1 => { self.craft_op_cursor = 0; self.apply_craft_op(); }
                    VirtualKeyCode::Key2 => { self.craft_op_cursor = 1; self.apply_craft_op(); }
                    VirtualKeyCode::Key3 => { self.craft_op_cursor = 2; self.apply_craft_op(); }
                    VirtualKeyCode::Key4 => { self.craft_op_cursor = 3; self.apply_craft_op(); }
                    VirtualKeyCode::Key5 => { self.craft_op_cursor = 4; self.apply_craft_op(); }
                    VirtualKeyCode::Key6 => { self.craft_op_cursor = 5; self.apply_craft_op(); }
                    VirtualKeyCode::Escape => {
                        self.craft_phase = CraftPhase::SelectItem;
                        self.craft_message = String::new();
                    }
                    _ => {}
                },
            },

            AppScreen::GameOver | AppScreen::Victory => match key {
                VirtualKeyCode::Return | VirtualKeyCode::Escape => {
                    self.player = None; self.enemy = None; self.floor = None;
                    self.combat_state = None; self.combat_log.clear();
                    self.screen = AppScreen::Title;
                }
                VirtualKeyCode::S => self.screen = AppScreen::Scoreboard,
                _ => {}
            },

            AppScreen::Scoreboard => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::Q | VirtualKeyCode::Return => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },
        }
    }

    fn apply_craft_op(&mut self) {
        let idx = self.craft_item_cursor;
        let seed = self.floor_seed.wrapping_add(self.frame).wrapping_mul(6364136223846793005);

        let has_item = self.player.as_ref().map(|p| idx < p.inventory.len()).unwrap_or(false);
        if !has_item { self.craft_message = "No item at that index.".to_string(); return; }

        match self.craft_op_cursor {
            0 => { // Reforge
                if let Some(ref mut p) = self.player {
                    let n = p.inventory[idx].stat_modifiers.len().max(1);
                    p.inventory[idx].stat_modifiers.clear();
                    for j in 0..n {
                        let ms = seed.wrapping_add(j as u64 * 17777).wrapping_mul(6364136223846793005);
                        p.inventory[idx].stat_modifiers.push(StatModifier::generate_random(ms));
                    }
                    self.craft_message = format!("REFORGED! {} modifiers chaos-rolled anew.", n);
                }
            }
            1 => { // Augment
                if let Some(ref mut p) = self.player {
                    let ms = seed.wrapping_mul(0xdeadbeef).wrapping_add(p.inventory[idx].value as u64);
                    let new_mod = StatModifier::generate_random(ms);
                    let stat = new_mod.stat.clone(); let val = new_mod.value;
                    p.inventory[idx].stat_modifiers.push(new_mod);
                    p.inventory[idx].value = (p.inventory[idx].value as f64 * 1.2) as i64;
                    self.craft_message = format!("AUGMENTED! Added {:+} {}", val, stat);
                }
            }
            2 => { // Annul
                if let Some(ref mut p) = self.player {
                    if p.inventory[idx].stat_modifiers.is_empty() {
                        self.craft_message = "No modifiers to remove.".to_string();
                    } else {
                        let ri = (seed % p.inventory[idx].stat_modifiers.len() as u64) as usize;
                        let removed = p.inventory[idx].stat_modifiers.remove(ri);
                        self.craft_message = format!("ANNULLED: removed {} {:+}", removed.stat, removed.value);
                    }
                }
            }
            3 => { // Corrupt
                if let Some(ref mut p) = self.player {
                    let roll = chaos_roll_verbose(0.5, seed);
                    let outcome = roll.to_range(0, 5);
                    let item = &mut p.inventory[idx];
                    match outcome {
                        0 => {
                            if item.socket_count < 6 { item.socket_count += 1; self.craft_message = "CORRUPTED: +1 socket!".to_string(); }
                            else { self.craft_message = "CORRUPTED: item glows... nothing changes.".to_string(); }
                        }
                        1 => {
                            if !item.stat_modifiers.is_empty() {
                                let i2 = (seed.wrapping_add(99) % item.stat_modifiers.len() as u64) as usize;
                                item.stat_modifiers[i2].value *= 2;
                                self.craft_message = "CORRUPTED: a modifier was doubled!".to_string();
                            } else { self.craft_message = "CORRUPTED: sparks, nothing happens.".to_string(); }
                        }
                        2 => {
                            item.corruption = Some("Chaos-Touched".to_string());
                            item.value += (item.value as f64 * 0.5) as i64;
                            self.craft_message = "CORRUPTED: Chaos-Touched! (+50% value)".to_string();
                        }
                        3 => {
                            item.stat_modifiers.pop();
                            self.craft_message = "CORRUPTED: a modifier dissolved into void.".to_string();
                        }
                        4 => {
                            for m in &mut item.stat_modifiers { m.value = -m.value; }
                            self.craft_message = "CORRUPTED: all modifiers INVERTED!".to_string();
                        }
                        _ => {
                            item.is_weapon = !item.is_weapon;
                            self.craft_message = "CORRUPTED: item type transmogrified!".to_string();
                        }
                    }
                }
            }
            4 => { // Fuse
                if let Some(ref mut p) = self.player {
                    p.inventory[idx].value *= 2;
                    p.inventory[idx].rarity = match p.inventory[idx].rarity {
                        Rarity::Common => Rarity::Uncommon,
                        Rarity::Uncommon => Rarity::Rare,
                        Rarity::Rare => Rarity::Epic,
                        Rarity::Epic => Rarity::Legendary,
                        Rarity::Legendary => Rarity::Mythical,
                        Rarity::Mythical => Rarity::Divine,
                        Rarity::Divine => Rarity::Beyond,
                        Rarity::Beyond | Rarity::Artifact => Rarity::Artifact,
                    };
                    self.craft_message = format!("FUSED! Value doubled, rarity → {}", p.inventory[idx].rarity.name());
                }
            }
            5 => { // EngineLock
                let cost = 40 + self.floor_num as i64 * 5;
                let can_afford = self.player.as_ref().map(|p| p.gold >= cost).unwrap_or(false);
                if !can_afford {
                    self.craft_message = format!("Need {}g for EngineLock.", cost);
                    return;
                }
                let engines = ["Lorenz","Zeta","Collatz","Mandelbrot","Fibonacci","Euler","Linear","SharpEdge","Orbit","Recursive"];
                let ei = (seed % engines.len() as u64) as usize;
                let eng = engines[ei].to_string();
                if let Some(ref mut p) = self.player {
                    p.gold -= cost;
                    p.inventory[idx].engine_locks.push(eng.clone());
                    self.craft_message = format!("ENGINE LOCKED: {} embedded! (-{}g)", eng, cost);
                }
            }
            _ => {}
        }
    }
}

// ─── ENTRY POINT ─────────────────────────────────────────────────────────────

fn main() -> BError {
    let builder = BTermBuilder::simple80x50()
        .with_title("CHAOS RPG — Where Math Goes To Die")
        .with_tile_dimensions(14, 14)
        .with_dimensions(120, 50)
        .with_fps_cap(60.0)
        .with_fullscreen(true);
    main_loop(builder.build()?, State::new())
}
