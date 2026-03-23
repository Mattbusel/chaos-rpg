//! CHAOS RPG — Graphical Frontend (bracket-lib)
//!
//! Full feature parity with the terminal version.
//! Always runs fullscreen. All room types, modes, boons, nemesis, gauntlet,
//! cursed floors, The Hunger, item volatility, crafting (all 6 ops), and
//! real chaos-engine combat via resolve_action().

use bracket_lib::prelude::*;
use chaos_rpg_audio::AudioSystem;
use chaos_rpg_core::{
    audio_events::AudioEvent,

    bosses::{boss_name, boss_pool_for_floor, random_unique_boss},
    character::{Background, Boon, Character, CharacterClass, Difficulty},
    chaos_pipeline::{chaos_roll_verbose, destiny_roll, ChaosRollResult},
    combat::{resolve_action, CombatAction, CombatOutcome, CombatState},
    enemy::{generate_enemy, Enemy, FloorAbility},
    items::{Item, Rarity, StatModifier},
    nemesis::{clear_nemesis, load_nemesis, save_nemesis, NemesisRecord},
    npcs::shop_npc,
    scoreboard::{load_scores, load_misery_scores, save_score, ScoreEntry},
    skill_checks::{perform_skill_check, Difficulty as SkillDiff, SkillType},
    spells::Spell,
    world::{generate_floor, room_enemy, Floor, RoomType},
};

mod renderer;
mod sprites;
mod theme;
mod ui_overlay;

use theme::{Theme, THEMES};

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
    CharacterSheet,
    BodyChart,
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

// ─── PARTICLE SYSTEM ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct Particle {
    x: i32,
    y: f32,
    text: String,
    col: (u8, u8, u8),
    age: u32,
    lifetime: u32,
}

impl Particle {
    fn new(x: i32, y: i32, text: impl Into<String>, col: (u8,u8,u8), lifetime: u32) -> Self {
        Self { x, y: y as f32, text: text.into(), col, age: 0, lifetime }
    }
    fn alive(&self) -> bool { self.age < self.lifetime }
    fn step(&mut self) { self.age += 1; self.y -= 0.45; }
    /// Dim color in the last 40% of lifetime.
    fn render_col(&self) -> (u8, u8, u8) {
        let fade_at = self.lifetime * 3 / 5;
        if self.age <= fade_at { return self.col; }
        let pct = 1.0 - (self.age - fade_at) as f32 / (self.lifetime - fade_at).max(1) as f32;
        (
            ((self.col.0 as f32 * pct) as u8).max(12),
            ((self.col.1 as f32 * pct) as u8).max(12),
            ((self.col.2 as f32 * pct) as u8).max(12),
        )
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
    // audio
    audio: Option<AudioSystem>,
    // visual theme
    theme_idx: usize,
    // auto-play mode
    auto_mode: bool,
    auto_last_action: u64, // frame when last auto-action fired
    // ── Visual effects ──
    particles: Vec<Particle>,
    player_flash: u32,          // frames left — red border flash on player panel
    enemy_flash: u32,           // frames left — colored border flash on enemy panel
    enemy_flash_col: (u8,u8,u8),
    hit_shake: u32,             // frames of outer-border shake flash on big crits
    spell_beam: u32,            // frames of beam animation
    spell_beam_col: (u8,u8,u8),
    // ── Kill linger ──
    kill_linger: u32,                       // frames to stay on combat after kill
    post_combat_screen: Option<AppScreen>,  // screen to go to after linger
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
            audio: AudioSystem::try_new(),
            theme_idx: 0,
            auto_mode: false,
            auto_last_action: 0,
            particles: Vec::new(),
            player_flash: 0,
            enemy_flash: 0,
            enemy_flash_col: (80, 220, 80),
            hit_shake: 0,
            spell_beam: 0,
            spell_beam_col: (80, 120, 255),
            kill_linger: 0,
            post_combat_screen: None,
        }
    }

    fn theme(&self) -> &theme::Theme {
        &theme::THEMES[self.theme_idx]
    }

    fn cycle_theme(&mut self) {
        self.theme_idx = (self.theme_idx + 1) % theme::THEMES.len();
    }

    fn max_mana(&self) -> i64 {
        self.player.as_ref().map(|p| (p.stats.mana + 50).max(50)).unwrap_or(50)
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.combat_log.push(msg.into());
        if self.combat_log.len() > 300 { self.combat_log.remove(0); }
    }

    fn emit_audio(&self, event: AudioEvent) {
        if let Some(ref audio) = self.audio { audio.emit(event); }
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
        self.emit_audio(AudioEvent::FloorEntered { floor: self.floor_num, seed: self.floor_seed });
        if self.game_mode == GameMode::Daily {
            self.emit_audio(AudioEvent::DailyStart);
        }
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
                    self.emit_audio(AudioEvent::ItemVolatilityReroll);
                }
            }
        }

        self.is_cursed_floor = self.floor_num > 0 && self.floor_num % 25 == 0;
        if self.is_cursed_floor {
            self.push_log("☠ CURSED FLOOR! All engine outputs INVERTED this floor.".to_string());
            self.emit_audio(AudioEvent::CursedFloorActivated);
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
                self.emit_audio(AudioEvent::Victory);
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
            self.emit_audio(AudioEvent::HungerTriggered);
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
                            self.emit_audio(AudioEvent::NemesisSpawned);
                            self.emit_audio(AudioEvent::BossEncounterStart { boss_tier: 2 });
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
                    self.emit_audio(AudioEvent::GauntletStart);
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
                        self.emit_audio(AudioEvent::BossEncounterStart { boss_tier: 3 });
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
                    self.emit_audio(AudioEvent::BossEncounterStart { boss_tier: 1 });
                } else {
                    self.emit_audio(AudioEvent::RoomEntered { room_index: self.floor.as_ref().map(|f| f.current_room).unwrap_or(0) });
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
                self.emit_audio(AudioEvent::ShopEntered);
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

        // Track final blow for cause-of-death
        {
            use chaos_rpg_core::combat::CombatEvent;
            let last_hit = events.iter().rev().find_map(|ev| {
                if let CombatEvent::EnemyAttack { damage, is_crit } = ev {
                    Some((*damage, *is_crit))
                } else { None }
            });
            if let Some((dmg, crit)) = last_hit {
                let ename = enemy.name.clone();
                let floor = player.floor;
                let crit_tag = if crit { " [CRIT]" } else { "" };
                player.run_stats.cause_of_death =
                    format!("Floor {} — {} hit for {}{}", floor, ename, dmg, crit_tag);
                player.run_stats.final_blow_damage = dmg;
            }
        }

        for ev in &events {
            self.combat_log.push(ev.to_display_string());
        }

        // ── Spawn visual effects ──────────────────────────────────────────────
        {
            use chaos_rpg_core::combat::CombatEvent;
            for ev in &events {
                match ev {
                    // Enemy takes damage from player
                    CombatEvent::PlayerAttack { damage, is_crit } => {
                        let (text, col, lt) = if *is_crit {
                            (format!("★ {} ★", damage), (255u8, 215u8, 0u8), 22u32)
                        } else {
                            (format!("{}", damage), (80, 220, 80), 18)
                        };
                        // Jitter x slightly using frame so stacked hits don't overlap
                        let jx = 12 + (self.frame % 8) as i32;
                        self.particles.push(Particle::new(jx, 7, text, col, lt));
                        self.enemy_flash = if *is_crit { 7 } else { 3 };
                        self.enemy_flash_col = col;
                    }
                    // Player takes damage from enemy
                    CombatEvent::EnemyAttack { damage, is_crit } => {
                        let (text, col, lt) = if *is_crit {
                            (format!("☠ -{} !", damage), (255u8, 60u8, 0u8), 22u32)
                        } else {
                            (format!("-{}", damage), (220, 50, 50), 18)
                        };
                        let jx = 53 + (self.frame % 8) as i32;
                        self.particles.push(Particle::new(jx, 7, text, col, lt));
                        self.player_flash = if *is_crit { 8 } else { 4 };
                        if *is_crit { self.hit_shake = 5; }
                    }
                    // Healing
                    CombatEvent::PlayerHealed { amount } => {
                        self.particles.push(Particle::new(55, 9,
                            format!("+{}", amount), (50, 220, 100), 18));
                    }
                    // Spell cast
                    CombatEvent::SpellCast { damage, backfired, .. } => {
                        if *backfired {
                            self.spell_beam_col = (220, 50, 50);
                            self.particles.push(Particle::new(50, 5,
                                format!("BACKFIRE! -{}", damage), (255, 80, 0), 26));
                            self.hit_shake = 4;
                        } else {
                            self.spell_beam_col = (80, 140, 255);
                            self.particles.push(Particle::new(14, 6,
                                format!("✦ {}", damage), (130, 190, 255), 22));
                            self.enemy_flash = 6;
                            self.enemy_flash_col = (80, 140, 255);
                        }
                        self.spell_beam = 12;
                    }
                    // Enemy kill reward
                    CombatEvent::EnemyDied { xp, gold } => {
                        self.particles.push(Particle::new(8, 4,
                            format!("+{} XP  +{}g", xp, gold), (255, 215, 0), 28));
                    }
                    // Status applied
                    CombatEvent::StatusApplied { name } => {
                        self.particles.push(Particle::new(14, 11,
                            format!("[{}]", name), (200, 150, 60), 16));
                    }
                    // Defend
                    CombatEvent::PlayerDefend { damage_reduced } if *damage_reduced > 0 => {
                        self.particles.push(Particle::new(55, 6,
                            format!("BLOCK -{}", damage_reduced), (80, 140, 200), 18));
                    }
                    _ => {}
                }
            }
        }

        // ── Misery event wiring ───────────────────────────────────────────────
        if let Some(ref mut p) = self.player {
            use chaos_rpg_core::combat::CombatEvent;
            use chaos_rpg_core::misery_system::MiserySource;
            for ev in &events {
                match ev {
                    CombatEvent::EnemyAttack { damage, is_crit } => {
                        let new_ms = p.misery.add_misery(MiserySource::DamageTaken, *damage as f64);
                        if *is_crit { p.misery.add_misery(MiserySource::Headshot, 0.0); }
                        p.run_stats.record_damage_taken(*damage, *is_crit);
                        for ms in new_ms {
                            self.combat_log.push(format!("[MISERY] {}", ms.title()));
                        }
                        // Pity mercy check
                        let pity = chaos_rpg_core::misery_system::MiseryState::enemy_pity_chance(p.stats.total());
                        if pity > 0.0 {
                            let roll = (p.seed.wrapping_mul(p.rooms_cleared as u64 + self.frame)) % 100;
                            if roll < (pity * 100.0) as u64 {
                                p.misery.add_misery(MiserySource::EnemyPitiedYou, 0.0);
                                p.run_stats.enemies_pitied_you += 1;
                                self.combat_log.push(format!("Enemy looks at you with pity. Attack skipped."));
                            }
                        }
                    }
                    CombatEvent::PlayerAttack { damage, is_crit } => {
                        p.run_stats.record_damage_dealt(*damage, None, *is_crit);
                        let new_passives = p.misery.increment_defiance_roll();
                        for passive in new_passives {
                            self.combat_log.push(format!("[DEFIANCE] {} UNLOCKED!", passive.name()));
                        }
                    }
                    CombatEvent::PlayerFleeFailed => {
                        p.misery.add_misery(MiserySource::FleeFailed, 0.0);
                    }
                    _ => {}
                }
            }
            // Cosmic joke flavor
            if p.misery.cosmic_joke {
                if let Some(line) = chaos_rpg_core::misery_system::MiseryState::cosmic_joke_combat_line(
                    p.seed, self.frame) {
                    self.combat_log.push(format!("  {}", line));
                }
            }
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
            self.emit_audio(AudioEvent::LevelUp);
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
                // Show loot if pending, else floor nav — but linger on combat first
                let next_screen = if self.loot_pending.is_some() {
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
                    AppScreen::RoomView
                } else {
                    self.advance_floor_room();
                    if self.screen == AppScreen::GameOver || self.screen == AppScreen::Victory {
                        self.screen.clone()
                    } else {
                        AppScreen::FloorNav
                    }
                };
                self.kill_linger = 35;
                self.post_combat_screen = Some(next_screen);
            }

            CombatOutcome::PlayerDied => {
                self.particles.clear();
                self.player_flash = 0; self.enemy_flash = 0; self.hit_shake = 0;
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
                self.emit_audio(AudioEvent::EntityDied { is_player: true });
                self.emit_audio(AudioEvent::GameOver);
                self.screen = AppScreen::GameOver;
            }

            CombatOutcome::PlayerFled => {
                self.push_log("You escaped into the chaos!".to_string());
                self.emit_audio(AudioEvent::PlayerFled);
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
        use chaos_rpg_core::{
            legacy_system::{GraveyardEntry, LegacyData},
            scoreboard::{save_misery_score, MiseryEntry},
            misery_system::MiseryState,
        };
        if let Some(ref p) = self.player {
            let score_val = p.xp + p.gold as u64 + (p.kills * 100) as u64 + (p.floor as u64 * 500);
            let tier = p.power_tier();
            let underdog = p.underdog_multiplier();
            let misery = p.misery.misery_index;
            let entry = ScoreEntry::new(
                p.name.clone(), p.class.name().to_string(),
                score_val, p.floor, p.kills, 0,
            ).with_tier(tier.name()).with_misery(misery, underdog);
            let _ = save_score(entry);

            // Hall of Misery
            if misery >= 100.0 {
                let me = MiseryEntry::new(
                    &p.name, p.class.name().to_string(), misery, p.floor,
                    tier.name(), p.misery.spite_total_spent, p.misery.defiance_rolls,
                    &p.run_stats.cause_of_death, underdog,
                );
                let _ = save_misery_score(me);
            }

            // Graveyard / legacy
            let all_neg = p.stats.vitality < 0 && p.stats.force < 0 && p.stats.mana < 0;
            let epitaph = GraveyardEntry::generate_epitaph(
                p.class.name(), p.floor, p.kills, p.total_damage_dealt,
                misery, p.spells_cast, all_neg,
                p.run_stats.deaths_to_backfire > 0, tier.name(),
            );
            let ge = GraveyardEntry {
                name: p.name.clone(), class: p.class.name().to_string(),
                level: p.level, floor: p.floor, power_tier: tier.name().to_string(),
                misery_index: misery, cause_of_death: p.run_stats.cause_of_death.clone(),
                kills: p.kills, score: score_val, date: String::new(), epitaph,
            };
            let mut legacy = LegacyData::load();
            legacy.record_run(
                ge, p.total_damage_dealt, p.total_damage_taken, p.gold,
                misery, p.misery.spite_total_spent, p.run_stats.total_rolls,
                p.run_stats.deaths_to_backfire > 0, false, p.seed, tier.name(),
            );
            legacy.save();
        }
    }
}

// ─── CONST LISTS ──────────────────────────────────────────────────────────────

const CLASSES: &[(&str, CharacterClass)] = &[
    ("Mage",         CharacterClass::Mage),
    ("Berserker",    CharacterClass::Berserker),
    ("Ranger",       CharacterClass::Ranger),
    ("Thief",        CharacterClass::Thief),
    ("Necromancer",  CharacterClass::Necromancer),
    ("Alchemist",    CharacterClass::Alchemist),
    ("Paladin",      CharacterClass::Paladin),
    ("VoidWalker",   CharacterClass::VoidWalker),
    ("Warlord",      CharacterClass::Warlord),
    ("Trickster",    CharacterClass::Trickster),
    ("Runesmith",    CharacterClass::Runesmith),
    ("Chronomancer", CharacterClass::Chronomancer),
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

        if self.auto_mode {
            self.tick_auto_play(ctx);
        }

        // Kill-linger: hold combat screen after victory so effects can finish
        if self.kill_linger > 0 {
            self.kill_linger -= 1;
            self.draw_combat(ctx);
            if self.kill_linger == 0 {
                if let Some(next) = self.post_combat_screen.take() {
                    self.screen = next;
                }
            }
            return;
        }

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
            AppScreen::CharacterSheet   => self.draw_character_sheet(ctx),
            AppScreen::BodyChart        => self.draw_body_chart(ctx),
            AppScreen::GameOver         => self.draw_game_over(ctx),
            AppScreen::Victory          => self.draw_victory(ctx),
            AppScreen::Scoreboard       => self.draw_scoreboard(ctx),
        }

        self.handle_input(ctx);
    }
}

// ─── DRAW HELPERS ─────────────────────────────────────────────────────────────

use renderer::{
    draw_panel, draw_subpanel, draw_bar_gradient, draw_bar_solid,
    print_t, print_center, print_hint, draw_separator,
    print_selectable, draw_minimap_cell, stat_line,
    MinimapState, cursor_char,
};

fn room_col(rt: &RoomType, t: &theme::Theme) -> (u8,u8,u8) {
    match rt {
        RoomType::Combat        => t.danger,
        RoomType::Boss          => (min_u8(t.danger.0, 200), 0, 0),
        RoomType::Treasure      => t.gold,
        RoomType::Shop          => t.accent,
        RoomType::Shrine        => (180, 80, 220),
        RoomType::Trap          => t.warn,
        RoomType::Portal        => t.mana,
        RoomType::Empty         => t.muted,
        RoomType::ChaosRift     => t.xp,
        RoomType::CraftingBench => t.success,
    }
}

fn min_u8(a: u8, b: u8) -> u8 { if a < b { a } else { b } }

// ─── SCREENS ──────────────────────────────────────────────────────────────────

impl State {
    // ── TITLE ─────────────────────────────────────────────────────────────────

    fn draw_title(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let brd = RGB::from_u8(t.border.0,  t.border.1,  t.border.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let sel = RGB::from_u8(t.selected.0,t.selected.1,t.selected.2);

        // Fill background
        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "", &t);

        // Animated banner pulse
        let pulse = ((self.frame as f32 * 0.04).sin() * 0.15 + 0.85) as f32;
        let ph = (t.heading.0 as f32 * pulse) as u8;
        let pg = (t.heading.1 as f32 * pulse) as u8;
        let pb = (t.heading.2 as f32 * pulse) as u8;
        let pulsed = RGB::from_u8(ph, pg, pb);

        ctx.print_color(4, 4,  pulsed, bg, " ██████╗██╗  ██╗ █████╗  ██████╗ ███████╗");
        ctx.print_color(4, 5,  pulsed, bg, "██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝");
        ctx.print_color(4, 6,  hd,     bg, "██║     ███████║███████║██║   ██║███████╗");
        ctx.print_color(4, 7,  hd,     bg, "╚██████╗██║  ██║██║  ██║╚██████╔╝███████║");
        ctx.print_color(4, 8,  hd,     bg, " ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝");

        ctx.print_color(4, 9,  dim, bg, "        R P G    ─    Where Math Goes To Die");

        // Decorative separator
        draw_separator(ctx, 2, 11, 75, &t);

        ctx.print_color(5, 12, ac, bg, "Graphical Edition  ·  All Systems  ·  Fullscreen");

        // Theme name badge bottom-right
        let tname = format!(" Theme: {} [T] ", t.name);
        ctx.print_color(79 - tname.len() as i32 - 1, 47, dim, bg, &tname);

        // Menu box
        let ox = 29i32; let oy = 20i32;
        draw_subpanel(ctx, ox - 3, oy - 2, 28, 9, "MAIN MENU", &t);

        let opts = ["New Game", "Scoreboard", "Quit"];
        for (i, opt) in opts.iter().enumerate() {
            print_selectable(ctx, ox, oy + i as i32 * 2, i == self.selected_menu, opt, self.frame, &t);
        }

        // Hint bar
        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 4, 46, "↑↓", " Navigate   ", &t);
        print_hint(ctx, 22, 46, "Enter", " Select   ", &t);
        print_hint(ctx, 40, 46, "T", " Theme   ", &t);
        print_hint(ctx, 52, 46, "Q", " Quit", &t);

        // Tagline
        ctx.print_color(4, 47, dim, bg, &format!("\"{}\"", t.tagline));
    }

    // ── MODE SELECT ───────────────────────────────────────────────────────────

    fn draw_mode_select(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "SELECT MODE", &t);

        let modes = [
            ("Story Mode",    "10 floors. Narrative arc with a final boss.",    "★ Recommended for newcomers"),
            ("Infinite Mode", "Descend forever. Math gets worse every floor.",  "∞ Score for the global leaderboard"),
            ("Daily Seed",    "Same dungeon for everyone today.",               "◈ Resets at UTC midnight"),
        ];

        for (i, (name, desc, hint)) in modes.iter().enumerate() {
            let y = 10 + i as i32 * 10;
            let is_sel = i == self.mode_cursor;
            let bx = 5i32;
            if is_sel {
                draw_subpanel(ctx, bx - 2, y - 1, 72, 7, "", &t);
            }
            print_selectable(ctx, bx, y, is_sel, name, self.frame, &t);
            ctx.print_color(bx + 2, y + 2, dim, bg, desc);
            ctx.print_color(bx + 2, y + 4, if is_sel { ac } else { dim }, bg, hint);
        }

        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 4, 46, "↑↓", " Navigate   ", &t);
        print_hint(ctx, 22, 46, "Enter", " Select   ", &t);
        print_hint(ctx, 38, 46, "Esc", " Back", &t);
    }

    // ── CHAR CREATION ─────────────────────────────────────────────────────────

    fn draw_char_creation(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let sel = RGB::from_u8(t.selected.0,t.selected.1,t.selected.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let wrn = RGB::from_u8(t.warn.0,   t.warn.1,   t.warn.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "CHARACTER CREATION", &t);

        // ── Class column (scrollable — show up to 12 classes at 1 row each)
        draw_subpanel(ctx, 2, 3, 25, 32, "CLASS  ↑↓", &t);
        for (i, (name, _)) in CLASSES.iter().enumerate() {
            print_selectable(ctx, 4, 5 + i as i32 * 2, i == self.cc_class, name, self.frame, &t);
        }

        // Class passive description
        let class = &CLASSES[self.cc_class].1;
        draw_subpanel(ctx, 2, 37, 25, 7, "PASSIVE ABILITY", &t);
        ctx.print_color(4, 39, ac, bg, class.passive_name());
        let desc = class.passive_desc();
        let mut row = 40i32;
        let mut line = String::new();
        for w in desc.split_whitespace() {
            if line.len() + w.len() + 1 > 20 {
                ctx.print_color(4, row, dim, bg, &line);
                line = w.to_string(); row += 1;
            } else {
                if !line.is_empty() { line.push(' '); }
                line.push_str(w);
            }
        }
        if !line.is_empty() { ctx.print_color(4, row, dim, bg, &line); }

        // ── Background column
        draw_subpanel(ctx, 30, 3, 25, 12, "BACKGROUND  ←→", &t);
        for (i, (name, _)) in BACKGROUNDS.iter().enumerate() {
            print_selectable(ctx, 32, 5 + i as i32 * 2, i == self.cc_bg, name, self.frame, &t);
        }

        // ── Difficulty column
        draw_subpanel(ctx, 30, 18, 25, 12, "DIFFICULTY  Tab", &t);
        let diff_colors = [suc, hd, wrn, dng];
        for (i, (name, _)) in DIFFICULTIES.iter().enumerate() {
            let is_sel = i == self.cc_diff;
            let c = if is_sel { sel } else { diff_colors[i] };
            let pfx = if is_sel { format!("{} ", cursor_char(self.frame)) } else { "  ".to_string() };
            ctx.print_color(32, 20 + i as i32 * 2, c, bg, &format!("{}{}", pfx, name));
        }

        // ── Portrait column
        draw_subpanel(ctx, 57, 3, 21, 43, "PORTRAIT", &t);
        let portrait = class.ascii_art();
        for (i, l) in portrait.lines().enumerate() {
            let line: String = l.chars().take(18).collect();
            ctx.print_color(59, 5 + i as i32, ac, bg, &line);
        }
        // Class description (word-wrapped at 17 chars)
        draw_separator(ctx, 58, 9, 19, &t);
        let mut row = 10i32;
        let mut line = String::new();
        for w in class.description().split_whitespace() {
            if line.len() + w.len() + 1 > 17 {
                ctx.print_color(59, row, dim, bg, &line);
                line = w.to_string(); row += 1;
            } else {
                if !line.is_empty() { line.push(' '); }
                line.push_str(w);
            }
        }
        if !line.is_empty() { ctx.print_color(59, row, dim, bg, &line); row += 1; }
        // Passive ability
        row += 1;
        ctx.print_color(59, row, ac, bg, class.passive_name());
        row += 1;
        let mut pline = String::new();
        for w in class.passive_desc().split_whitespace() {
            if pline.len() + w.len() + 1 > 17 {
                ctx.print_color(59, row, dim, bg, &pline);
                pline = w.to_string(); row += 1;
            } else {
                if !pline.is_empty() { pline.push(' '); }
                pline.push_str(w);
            }
        }
        if !pline.is_empty() { ctx.print_color(59, row, dim, bg, &pline); }

        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 4, 46, "↑↓", " Class   ", &t);
        print_hint(ctx, 18, 46, "←→", " Background   ", &t);
        print_hint(ctx, 36, 46, "Tab", " Difficulty   ", &t);
        print_hint(ctx, 54, 46, "Enter", " Confirm   ", &t);
        print_hint(ctx, 70, 46, "Esc", " Back", &t);
    }

    // ── BOON SELECT ───────────────────────────────────────────────────────────

    fn draw_boon_select(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "CHOOSE YOUR BOON", &t);

        ctx.print_color(5, 3, dim, bg, "A gift from the chaos engine. Only one. Choose wisely.");
        draw_separator(ctx, 2, 5, 75, &t);

        for (i, boon) in self.boon_options.iter().enumerate() {
            let y = 8 + i as i32 * 12;
            let is_sel = i == self.boon_cursor;
            if is_sel {
                draw_subpanel(ctx, 2, y - 1, 75, 10, "", &t);
            }
            let key = format!("[{}] ", i + 1);
            ctx.print_color(12, y, if is_sel { ac } else { dim }, bg, &key);
            print_selectable(ctx, 16, y, is_sel, boon.name(), self.frame, &t);
            ctx.print_color(16, y + 2, dim, bg, boon.description());
        }

        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 4, 46, "↑↓ / 1-3", " Select   ", &t);
        print_hint(ctx, 28, 46, "Enter", " Confirm   ", &t);
        print_hint(ctx, 44, 46, "Esc", " Back", &t);
    }

    // ── FLOOR NAV ─────────────────────────────────────────────────────────────

    fn draw_floor_nav(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let mana= RGB::from_u8(t.mana.0,   t.mana.1,   t.mana.2);

        let (pname, pclass, plv, pfloor, pkills, pgold, pxp, php, pmhp, pstatus,
             pcorruption, prwk, ptier, pmisery, punderdog, pdefiance) = match &self.player {
            Some(p) => {
                let tier = p.power_tier();
                (p.name.clone(), p.class.name(), p.level, p.floor,
                 p.kills, p.gold, p.xp, p.current_hp, p.max_hp,
                 p.status_badge_line(), p.corruption, p.rooms_without_kill,
                 tier, p.misery.misery_index, p.underdog_multiplier(),
                 p.misery.defiance_rolls)
            }
            None => { self.screen = AppScreen::Title; return; }
        };

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "", &t);

        // ── Header bar ────────────────────────────────────────────────────────
        let floor_str = format!(" FLOOR {}  {}  Lv.{}  {} ",
            pfloor, pname, plv, pclass);
        ctx.print_color(2, 1, hd, bg, &floor_str);

        // Mode badge (right-aligned), AUTO badge if active
        if self.auto_mode {
            let pulse = (self.frame / 15) % 2 == 0;
            let auto_c = if pulse { RGB::from_u8(80, 220, 80) } else { RGB::from_u8(40, 140, 40) };
            ctx.print_color(65, 1, auto_c, bg, "◆ AUTO");
        }
        let mode_str = match self.game_mode {
            GameMode::Story    => format!("STORY {}/{}", pfloor, 10),
            GameMode::Infinite => "∞ INFINITE".to_string(),
            GameMode::Daily    => "◈ DAILY".to_string(),
        };
        ctx.print_color(79 - mode_str.len() as i32 - 1, 1, ac, bg, &mode_str);

        draw_separator(ctx, 1, 2, 77, &t);

        // Cursed floor badge (below header if active)
        if self.is_cursed_floor {
            ctx.print_color(2, 2, dng, bg, "☠ CURSED FLOOR — ALL ENGINES INVERTED ☠");
        }

        // ── Left panel: player stats ───────────────────────────────────────────
        draw_subpanel(ctx, 1, 3, 27, 20, "STATUS", &t);

        let hp_pct = php as f32 / pmhp.max(1) as f32;
        let hp_c = t.hp_color(hp_pct);
        stat_line(ctx, 3, 5, "HP  ", &format!("{}/{}", php, pmhp), hp_c, &t);
        draw_bar_gradient(ctx, 3, 6, 24, php, pmhp, hp_c, t.muted, &t);

        let mp_pct = self.current_mana as f32 / self.max_mana() as f32;
        let _ = mp_pct;
        stat_line(ctx, 3, 8, "MP  ", &format!("{}/{}", self.current_mana, self.max_mana()), t.mana, &t);
        draw_bar_solid(ctx, 3, 9, 24, self.current_mana, self.max_mana(), t.mana, &t);

        stat_line(ctx, 3, 11, "Gold  ", &format!("{}g", pgold), t.gold, &t);
        stat_line(ctx, 3, 12, "XP    ", &format!("{}", pxp), t.xp, &t);
        stat_line(ctx, 3, 13, "Kills ", &format!("{}", pkills), t.success, &t);

        // ── Power tier display with animated RGB ──────────────────────────────
        {
            let tier_rgb = ptier.rgb();
            // Animate rainbow tiers
            let tier_col = if ptier.has_effect() {
                use chaos_rpg_core::power_tier::TierEffect;
                match ptier.effect() {
                    TierEffect::Rainbow | TierEffect::RainbowFast => {
                        let speed = if matches!(ptier.effect(), TierEffect::RainbowFast) { 2 } else { 4 };
                        let pal = [(220u8,60u8,60u8),(220,180,40),(60,200,80),(80,200,220),(80,80,220),(180,60,200)];
                        pal[((self.frame / speed) as usize) % pal.len()]
                    }
                    TierEffect::Pulse => {
                        let bright = (self.frame / 15) % 2 == 0;
                        if bright { tier_rgb } else { (tier_rgb.0/2, tier_rgb.1/2, tier_rgb.2/2) }
                    }
                    TierEffect::Flash => {
                        if (self.frame / 12) % 2 == 0 { tier_rgb } else { t.bg }
                    }
                    _ => tier_rgb,
                }
            } else { tier_rgb };
            let (power_label, power_value) = match &self.player {
                Some(p) => p.power_display(),
                None => ("POWER", ptier.name().to_string()),
            };
            stat_line(ctx, 3, 14, &format!("{}: ", power_label), &power_value, tier_col, &t);
        }

        // Misery / Underdog badges
        if pmisery >= 100.0 {
            let mc = t.warn;
            stat_line(ctx, 3, 15, "Misery ", &format!("{:.0}", pmisery), mc, &t);
        }
        if punderdog > 1.01 {
            let uc = t.gold;
            stat_line(ctx, 3, 16, "Underdog ", &format!("×{:.1}", punderdog), uc, &t);
        }
        if pdefiance > 0 {
            let dc = t.accent;
            stat_line(ctx, 3, 17, "Defiance ", &format!("{} rolls", pdefiance), dc, &t);
        }

        if pcorruption > 0 {
            stat_line(ctx, 3, 18, "Corrupt ", &format!("{}", pcorruption), t.warn, &t);
        }
        if !pstatus.is_empty() {
            ctx.print_color(3, 15, RGB::from_u8(t.xp.0, t.xp.1, t.xp.2), bg,
                &format!("St: {}", &pstatus.chars().take(18).collect::<String>()));
        }

        // Hunger / Nemesis warnings
        if pfloor >= 50 && prwk >= 3 {
            let rooms_left = 5u32.saturating_sub(prwk);
            ctx.print_color(3, 17, dng, bg,
                &format!("HUNGER: {} dry ({} left)", prwk, rooms_left));
        }
        if let Some(ref nem) = self.nemesis_record {
            ctx.print_color(3, 19, dng, bg,
                &format!("☠ NEM: {} fl.{}", &nem.enemy_name.chars().take(10).collect::<String>(), nem.floor_killed_at));
        }

        // ── Minimap ───────────────────────────────────────────────────────────
        draw_subpanel(ctx, 1, 24, 77, 13, "FLOOR MAP", &t);
        if let Some(ref floor) = self.floor {
            let per_row = 15usize;
            for (i, room) in floor.rooms.iter().enumerate() {
                let rx = 3 + (i % per_row) as i32 * 5;
                let ry = 26 + (i / per_row) as i32 * 3;
                let sym = room.room_type.icon();
                let rc = room_col(&room.room_type, &t);
                let mstate = if i == floor.current_room { MinimapState::Current }
                             else if i < floor.current_room { MinimapState::Visited }
                             else { MinimapState::Ahead };
                draw_minimap_cell(ctx, rx, ry, mstate, rc, sym, &t);
            }
            let current = floor.current();
            let rc = room_col(&current.room_type, &t);
            ctx.print_color(3, 35,
                RGB::from_u8(rc.0, rc.1, rc.2), bg,
                &format!("Next: [{}]  {}  —  {}",
                    current.room_type.icon().trim_matches(|c| c == '[' || c == ']'),
                    current.room_type.name(), current.description));
        }

        // ── Current room preview panel ────────────────────────────────────────
        draw_subpanel(ctx, 30, 3, 49, 20, "CURRENT ROOM", &t);
        if let Some(ref floor) = self.floor {
            let current = floor.current();
            let rc = room_col(&current.room_type, &t);
            let room_col_rgb = RGB::from_u8(rc.0, rc.1, rc.2);

            // Icon + room type name (large)
            let icon_line = format!("{}  {}", current.room_type.icon(), current.room_type.name());
            ctx.print_color(32, 5, room_col_rgb, bg, &icon_line.chars().take(44).collect::<String>());

            draw_separator(ctx, 31, 7, 46, &t);

            // Room description
            let desc_words: Vec<&str> = current.description.split_whitespace().collect();
            let mut line = String::new();
            let mut dy = 9i32;
            for word in &desc_words {
                if line.len() + word.len() + 1 > 42 {
                    ctx.print_color(32, dy, dim, bg, &line);
                    dy += 1;
                    line = word.to_string();
                } else {
                    if !line.is_empty() { line.push(' '); }
                    line.push_str(word);
                }
            }
            if !line.is_empty() && dy <= 14 {
                ctx.print_color(32, dy, dim, bg, &line);
                dy += 1;
            }

            // Room hint based on type
            let hint = match current.room_type {
                RoomType::Combat     => "Be ready to fight. Choose your first action wisely.",
                RoomType::Boss       => "BOSS ROOM — Powerful enemy, big rewards.",
                RoomType::Treasure   => "Free item inside. May be cursed.",
                RoomType::Shop       => "Spend gold on items, spells, or healing.",
                RoomType::Shrine     => "Stat bonus + HP restore. Usually safe.",
                RoomType::Trap       => "Unavoidable. Cunning helps dodge damage.",
                RoomType::Portal     => "Skip ahead. High risk, high reward.",
                RoomType::Empty      => "Quiet room. Heals a small amount of HP.",
                RoomType::ChaosRift  => "Pure chaos. Anything can happen.",
                RoomType::CraftingBench => "Reforge, augment, corrupt your items.",
            };
            let hint_y = (dy + 1).min(16);
            draw_separator(ctx, 31, hint_y - 1, 46, &t);
            let hint_words: Vec<&str> = hint.split_whitespace().collect();
            let mut hline = String::new();
            let mut hy = hint_y;
            for word in hint_words {
                if hline.len() + word.len() + 1 > 42 {
                    ctx.print_color(32, hy, ac, bg, &hline);
                    hy += 1;
                    hline = word.to_string();
                } else {
                    if !hline.is_empty() { hline.push(' '); }
                    hline.push_str(word);
                }
            }
            if !hline.is_empty() {
                ctx.print_color(32, hy, ac, bg, &hline);
            }

            // Room number progress
            let room_prog = format!("Room {}/{}", floor.current_room + 1, floor.rooms.len());
            ctx.print_color(32, 20, dim, bg, &room_prog);
        }

        // ── Chaos / Misery alert row ──────────────────────────────────────────
        if pmisery >= 50.0 || pcorruption > 5 {
            let pulse = (self.frame / 20) % 2 == 0;
            let alert_c = if pulse { RGB::from_u8(t.warn.0, t.warn.1, t.warn.2) }
                          else { RGB::from_u8(t.warn.0/2, t.warn.1/2, t.warn.2/2) };
            let msg = if pmisery >= 200.0 { "⚠ COSMIC JOKE IMMINENT — Misery critical" }
                      else if pmisery >= 100.0 { "☠ SPITE MODE ACTIVE — enemies empowered" }
                      else if pcorruption > 20 { "✖ HIGH CORRUPTION — chaos rolls destabilizing" }
                      else { "~ Chaos levels rising — watch misery meter" };
            ctx.print_color(2, 38, alert_c, bg, msg);
        }

        // ── Systems access bar ────────────────────────────────────────────────
        draw_separator(ctx, 1, 39, 77, &t);
        let sy = 40i32;

        // Skill point alert: flash if unspent
        let (sp_col, sp_label) = if let Some(ref p) = self.player {
            if p.skill_points > 0 {
                let pulse = (self.frame / 12) % 2 == 0;
                let c = if pulse { RGB::from_u8(t.gold.0, t.gold.1, t.gold.2) }
                        else { RGB::from_u8(t.gold.0/2+20, t.gold.1/2+20, 10) };
                (c, format!("[C] Sheet  ★ {} pts", p.skill_points))
            } else {
                (RGB::from_u8(t.accent.0, t.accent.1, t.accent.2), "[C] Sheet".to_string())
            }
        } else {
            (RGB::from_u8(t.accent.0, t.accent.1, t.accent.2), "[C] Sheet".to_string())
        };
        ctx.print_color(2, sy, sp_col, bg, &sp_label);

        // Body health teaser
        let body_summary_col = if let Some(ref p) = self.player {
            let worst_pct = p.body.parts.values()
                .map(|s| s.current_hp as f32 / s.max_hp.max(1) as f32)
                .fold(1.0f32, f32::min);
            if worst_pct < 0.3 { RGB::from_u8(t.danger.0, t.danger.1, t.danger.2) }
            else if worst_pct < 0.6 { RGB::from_u8(200, 130, 40) }
            else { RGB::from_u8(t.success.0, t.success.1, t.success.2) }
        } else { RGB::from_u8(t.dim.0, t.dim.1, t.dim.2) };
        ctx.print_color(22, sy, body_summary_col, bg, "[B] Body Chart");

        print_hint(ctx, 40, sy, "[E]", " Enter Room", &t);
        print_hint(ctx, 55, sy, "[Z]", " Auto", &t);
        print_hint(ctx, 64, sy, "[S]", " Scores", &t);
        print_hint(ctx, 74, sy, "[Q]", " Quit", &t);

        let y = sy + 1;
        if self.auto_mode {
            let auto_c = (80u8, 220u8, 80u8);
            ctx.print_color(2, y, RGB::from_u8(auto_c.0, auto_c.1, auto_c.2), bg,
                "AUTO PILOT ACTIVE — pauses at item/shop/craft  [Z] to stop");
        } else if self.floor.as_ref().map(|f| f.rooms_remaining() == 0).unwrap_or(false) {
            ctx.print_color(2, y, gld, bg, "[ D ] Descend to next floor  ▼");
        }
        draw_separator(ctx, 1, 43, 77, &t);
        ctx.print_color(2, 44, dim, bg, "[×]=Fight [★]=Loot [$]=Shop [~]=Shrine [!]=Trap [^]=Portal [⚒]=Craft [∞]=Rift");
        ctx.print_color(2, 45, dim, bg, "Tip: Press [C] to manage passives — unspent points boost your stats automatically");
    }

    // ── ROOM VIEW ─────────────────────────────────────────────────────────────

    fn draw_room_view(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let sel = RGB::from_u8(t.selected.0,t.selected.1,t.selected.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);

        // Apply effects once
        if !self.room_event.resolved {
            self.room_event.resolved = true;
            let gd = self.room_event.gold_delta;
            let hd_val = self.room_event.hp_delta;
            let dt = self.room_event.damage_taken;
            let bonuses: Vec<(&'static str, i64)> = self.room_event.stat_bonuses.clone();
            if let Some(ref mut p) = self.player {
                if gd != 0 { p.gold += gd; }
                if hd_val > 0  { p.heal(hd_val); }
                if dt > 0  { p.take_damage(dt); }
                for (stat, val) in &bonuses { self.apply_stat_modifier(stat, *val); }
            }
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

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "", &t);
        draw_subpanel(ctx, 2, 2, 75, 40, "", &t);

        let title = self.room_event.title.clone();
        print_center(ctx, 2, 4, 75, t.heading, &t, &title);
        draw_separator(ctx, 3, 5, 73, &t);

        for (i, line) in self.room_event.lines.iter().enumerate() {
            let fg = if line.starts_with('[') { sel }
                     else if line.starts_with('+') || line.starts_with("You find") { hd }
                     else { dim };
            ctx.print_color(5, 7 + i as i32, fg, bg, &line.chars().take(70).collect::<String>());
        }

        let has_item  = self.room_event.pending_item.is_some();
        let has_spell = self.room_event.pending_spell.is_some();
        let is_portal = self.room_event.portal_available;

        draw_separator(ctx, 3, 40, 73, &t);
        let ay = 42i32;
        if has_item  { print_hint(ctx, 8, ay, "[P]", " Pick up item   ", &t); print_hint(ctx, 32, ay, "[Enter]", " Leave it", &t); }
        if has_spell { print_hint(ctx, 8, ay+1, "[L]", " Learn spell   ", &t); print_hint(ctx, 32, ay+1, "[Enter]", " Leave scroll", &t); }
        if is_portal { print_hint(ctx, 8, ay, "[P]", " Step through portal   ", &t); print_hint(ctx, 38, ay, "[Enter]", " Resist", &t); }
        if !has_item && !has_spell && !is_portal {
            print_hint(ctx, 8, ay, "[Enter]", " Continue", &t);
        }
    }

    // ── COMBAT ────────────────────────────────────────────────────────────────

    fn draw_combat(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let mna = RGB::from_u8(t.mana.0,   t.mana.1,   t.mana.2);
        let xp  = RGB::from_u8(t.xp.0,     t.xp.1,     t.xp.2);

        let (pname, pclass, plv, php, pmhp, pstatus) = match &self.player {
            Some(p) => (p.name.clone(), p.class.name(), p.level, p.current_hp, p.max_hp, p.status_badge_line()),
            None => { self.screen = AppScreen::Title; return; }
        };
        let (ename, etier, ehp, emhp, esprite) = match &self.enemy {
            Some(e) => (e.name.clone(), e.tier.name().to_string(), e.hp, e.max_hp, e.ascii_sprite),
            None => { self.screen = AppScreen::FloorNav; return; }
        };

        ctx.cls_bg(bg);

        // Combat border with floor/kill context in title bar
        let floor_kills = self.player.as_ref().map(|p| (p.floor, p.kills)).unwrap_or((1, 0));
        let combat_title = format!("COMBAT  ─  Floor {}  ─  Kills: {}", floor_kills.0, floor_kills.1);
        draw_panel(ctx, 0, 0, 79, 49, &combat_title, &t);

        // ── Enemy panel ───────────────────────────────────────────────────────
        draw_subpanel(ctx, 1, 2, 38, 21, "ENEMY", &t);
        let boss_lbl = if self.gauntlet_stage > 0 {
            format!(" GAUNTLET {}/3 ", self.gauntlet_stage)
        } else if self.is_boss_fight { " ★ BOSS ★ ".to_string() } else { String::new() };
        if !boss_lbl.is_empty() {
            ctx.print_color(20, 3, dng, bg, &boss_lbl);
        }
        let etier_s: String = etier.chars().take(12).collect();
        let ename_s: String = ename.chars().take(18).collect();
        ctx.print_color(3, 4, dng, bg, &format!("{} [{}]", ename_s, etier_s));
        let ep = ehp as f32 / emhp.max(1) as f32;
        let ec = t.hp_color(ep);
        stat_line(ctx, 3, 5, "HP ", &format!("{}/{}", ehp, emhp), ec, &t);
        draw_bar_gradient(ctx, 3, 6, 34, ehp, emhp, ec, t.muted, &t);

        // Sprite
        for (i, line) in esprite.lines().enumerate().take(12) {
            let s: String = line.chars().take(35).collect();
            ctx.print_color(3, 8 + i as i32, dim, bg, &s);
        }

        // ── Player panel ──────────────────────────────────────────────────────
        draw_subpanel(ctx, 41, 2, 38, 21, "PLAYER", &t);
        let pname_s: String = pname.chars().take(10).collect();
        let pclass_s: String = pclass.chars().take(12).collect();
        ctx.print_color(43, 4, hd, bg, &format!("{} Lv.{} {}", pname_s, plv, pclass_s));
        let pp = php as f32 / pmhp.max(1) as f32;
        let pc = t.hp_color(pp);
        stat_line(ctx, 43, 5, "HP ", &format!("{}/{}", php, pmhp), pc, &t);
        draw_bar_gradient(ctx, 43, 6, 34, php, pmhp, pc, t.muted, &t);
        stat_line(ctx, 43, 7, "MP ", &format!("{}/{}", self.current_mana, self.max_mana()), t.mana, &t);
        draw_bar_solid(ctx, 43, 8, 34, self.current_mana, self.max_mana(), t.mana, &t);
        // ── Status effect icons with per-effect flicker ──────────────────────
        if let Some(ref p) = self.player {
            use chaos_rpg_core::character::StatusEffect;
            let mut sx = 43i32;
            for effect in &p.status_effects {
                let (icon, base_col): (&str, (u8,u8,u8)) = match effect {
                    StatusEffect::Burning(_)          => ("🔥", (255, 100,  20)),
                    StatusEffect::Poisoned(_)         => ("☠",  ( 50, 200,  50)),
                    StatusEffect::Stunned(_)          => ("⚡", (100, 200, 255)),
                    StatusEffect::Cursed(_)           => ("✖",  (180,  50, 180)),
                    StatusEffect::Blessed(_)          => ("✦",  (255, 220,  60)),
                    StatusEffect::Shielded(_)         => ("🛡", ( 60, 100, 220)),
                    StatusEffect::Enraged(_)          => ("⚔",  (220,  30,  30)),
                    StatusEffect::Frozen(_)           => ("❄",  (100, 180, 255)),
                    StatusEffect::Regenerating(_)     => ("+",  ( 50, 240, 100)),
                    StatusEffect::Phasing(_)          => ("◈",  (200,  80, 255)),
                    StatusEffect::Empowered(_)        => ("▲",  (255, 215,   0)),
                    StatusEffect::Fracture(_)         => ("⚙",  (180, 100,  40)),
                    StatusEffect::Resonance(_)        => ("~",  (255, 200,  80)),
                    StatusEffect::PhaseLock(_)        => ("⏸",  (220, 220, 220)),
                    StatusEffect::DimensionalBleed(_) => ("∞",  (140,  40, 200)),
                    StatusEffect::Recursive(_)        => ("↻",  (255,  80,  80)),
                    StatusEffect::Nullified(_)        => ("∅",  ( 80,  80,  80)),
                };
                // Pulse: alternate brightness each ~8 frames
                let pulse = (self.frame / 8) % 2 == 0;
                let fc = if pulse { base_col } else {
                    (base_col.0 / 2, base_col.1 / 2, base_col.2 / 2)
                };
                ctx.print_color(sx, 9, RGB::from_u8(fc.0, fc.1, fc.2), bg, icon);
                sx += (icon.chars().count() as i32).max(1) + 1;
                if sx > 76 { break; }
            }
        }
        if self.is_cursed_floor {
            ctx.print_color(43, 10, dng, bg, "☠ CURSED — inverted");
        }

        // Spells
        if let Some(ref p) = self.player {
            if !p.known_spells.is_empty() {
                ctx.print_color(43, 12, ac, bg, "SPELLS  [1-8]");
                for (i, spell) in p.known_spells.iter().enumerate().take(8) {
                    let can = self.current_mana >= spell.mana_cost;
                    let fg = if can { mna } else { dim };
                    ctx.print_color(43, 13 + i as i32, fg, bg,
                        &format!("[{}] {:<12} {}mp", i+1, &spell.name.chars().take(12).collect::<String>(), spell.mana_cost));
                }
            }
        }

        // ── Actions bar ───────────────────────────────────────────────────────
        draw_subpanel(ctx, 1, 24, 77, 8, "ACTIONS", &t);
        // Row 1: action keys + labels
        let ay = 26i32;
        // Each action: key col + desc col
        let actions: &[(&str, &str, &str)] = &[
            ("[A]", "Attack",  "normal hit"),
            ("[H]", "Heavy",   "1.5x, -acc"),
            ("[D]", "Defend",  "+40 block"),
            ("[T]", "Taunt",   "lure+debuff"),
            ("[F]", "Flee",    "escape"),
        ];
        let col_w = 14i32;
        for (i, (key, label, hint)) in actions.iter().enumerate() {
            let x = 3 + i as i32 * col_w;
            ctx.print_color(x, ay,     RGB::from_u8(t.accent.0, t.accent.1, t.accent.2),  bg, key);
            ctx.print_color(x + key.len() as i32, ay, RGB::from_u8(t.selected.0, t.selected.1, t.selected.2), bg, &format!(" {}", label));
            ctx.print_color(x, ay + 1, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2),    bg, hint);
        }
        print_hint(ctx, 3 + 5 * col_w, ay, "[1-8]", " Spells", &t);
        ctx.print_color(3 + 5 * col_w, ay + 1, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg, "cast spell");

        // Items row
        if let Some(ref p) = self.player {
            if !p.inventory.is_empty() {
                let keys = ["Q","W","E","R","Y","U","I","O"];
                let mut ix = 3i32;
                ctx.print_color(ix, ay + 3, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg, "Items:");
                ix += 7;
                for (i, item) in p.inventory.iter().enumerate().take(8) {
                    if ix > 73 { break; }
                    let name_s: String = item.name.chars().take(9).collect();
                    let label = format!("[{}]{} ", keys[i], name_s);
                    ctx.print_color(ix, ay + 3, dim, bg, &label);
                    ix += label.len() as i32;
                }
            }
        }

        // ── Combat log ────────────────────────────────────────────────────────
        // Panel: y=33, h=16 → bottom border at y=48 (fits inside outer 49-row panel)
        // Inner rows: y=35 to y=47 = 13 usable lines
        draw_subpanel(ctx, 1, 33, 77, 16, "CHAOS LOG", &t);
        let log_start = self.combat_log.len().saturating_sub(13);
        for (i, line) in self.combat_log[log_start..].iter().enumerate() {
            if i >= 13 { break; }
            let fg = if line.contains("CRIT") || line.contains("BOSS") || line.contains("☠") { dng }
                     else if line.contains("Victory") || line.contains("LEVEL") { gld }
                     else if line.contains("heal") || line.contains('+') { suc }
                     else if line.starts_with("  ") { dim } // engine trace
                     else { RGB::from_u8(t.primary.0, t.primary.1, t.primary.2) };
            ctx.print_color(3, 35 + i as i32, fg, bg, &line.chars().take(74).collect::<String>());
        }

        // ── Visual effects (drawn on top of panels) ───────────────────────────

        // 1. Enemy panel hit flash — redraw border in effect color
        if self.enemy_flash > 0 {
            self.enemy_flash -= 1;
            let t_scale = self.enemy_flash as f32 / 7.0;
            let ec = self.enemy_flash_col;
            let r = (ec.0 as f32 * t_scale + 40.0 * (1.0 - t_scale)) as u8;
            let g = (ec.1 as f32 * t_scale + 40.0 * (1.0 - t_scale)) as u8;
            let b = (ec.2 as f32 * t_scale + 40.0 * (1.0 - t_scale)) as u8;
            ctx.draw_box(1, 2, 38, 21, RGB::from_u8(r, g, b), bg);
        }

        // 2. Player panel hit flash — red border
        if self.player_flash > 0 {
            self.player_flash -= 1;
            let intensity = (self.player_flash * 30 + 60) as u8;
            ctx.draw_box(41, 2, 38, 21, RGB::from_u8(intensity, 10, 10), bg);
        }

        // 3. Screen shake on big crits — outer border flash
        if self.hit_shake > 0 {
            self.hit_shake -= 1;
            let pulse = (self.hit_shake % 2 == 0) as u8;
            let intensity = 120 + pulse * 80;
            ctx.draw_box(0, 0, 79, 49, RGB::from_u8(intensity, intensity / 4, 0), bg);
        }

        // 4. Spell beam — animated line of chars across the centre gap (y=23)
        if self.spell_beam > 0 {
            self.spell_beam -= 1;
            let bc = self.spell_beam_col;
            let bc_rgb = RGB::from_u8(bc.0, bc.1, bc.2);
            let beam_chars = ["~","≈","∿","~","≋","~"];
            let beam_offset = (self.frame / 2) as usize;
            for bx in 2..77i32 {
                let c = beam_chars[(bx as usize + beam_offset) % beam_chars.len()];
                ctx.print_color(bx, 23, bc_rgb, bg, c);
            }
            // Bright flash centre dot
            ctx.print_color(39, 23, RGB::from_u8(255, 255, 200), bg, "✦");
        }

        // 5. Floating damage numbers — step and render
        for p in &mut self.particles { p.step(); }
        self.particles.retain(|p| p.alive());
        for p in &self.particles {
            let rc = p.render_col();
            let py = p.y as i32;
            if py < 2 || py > 32 { continue; } // clip to combat panels area
            ctx.print_color(p.x, py, RGB::from_u8(rc.0, rc.1, rc.2), bg, &p.text);
        }
    }

    // ── SHOP ──────────────────────────────────────────────────────────────────

    fn draw_shop(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "SHOP", &t);

        let pgold = self.player.as_ref().map(|p| p.gold).unwrap_or(0);
        stat_line(ctx, 3, 3, "Your Gold: ", &format!("{}g", pgold), t.gold, &t);
        draw_separator(ctx, 1, 4, 77, &t);

        // Heal option
        let heal_row = 5i32;
        let can_heal = self.player.as_ref().map(|p| p.gold >= self.shop_heal_cost).unwrap_or(false);
        ctx.print_color(3, heal_row, if can_heal { suc } else { dim }, bg,
            &format!("[H] Healing Potion  +40 HP  ─  {}g", self.shop_heal_cost));

        draw_separator(ctx, 1, 7, 77, &t);

        for (i, (item, price)) in self.shop_items.iter().enumerate() {
            let y = 9 + i as i32 * 5;
            let is_sel = i + 1 == self.shop_cursor;
            let can_buy = self.player.as_ref().map(|p| p.gold >= *price).unwrap_or(false);
            if is_sel { draw_subpanel(ctx, 2, y - 1, 75, 5, "", &t); }
            let name_col = if is_sel { hd } else { dim };
            let price_col = if can_buy { gld } else { dim };
            let pfx = if is_sel { format!("{} ", cursor_char(self.frame)) } else { "  ".to_string() };
            ctx.print_color(3, y, name_col, bg, &format!("{}[{}] {}", pfx, i+1, &item.name.chars().take(30).collect::<String>()));
            ctx.print_color(55, y, price_col, bg, &format!("{}g ({})", price, item.rarity.name()));
            for (j, m) in item.stat_modifiers.iter().enumerate().take(2) {
                let mc = if m.value > 0 { suc } else { dim };
                ctx.print_color(8, y + 1 + j as i32, mc, bg,
                    &format!("{:+} {}", m.value, m.stat));
            }
        }

        draw_separator(ctx, 1, 45, 77, &t);
        print_hint(ctx, 3, 46, "[1-4]", " Buy item   ", &t);
        print_hint(ctx, 22, 46, "[H]", " Heal   ", &t);
        print_hint(ctx, 31, 46, "[Enter/0/Esc]", " Leave", &t);
    }

    // ── CRAFTING ──────────────────────────────────────────────────────────────

    fn draw_crafting(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "CRAFTING BENCH", &t);

        let has_inventory = self.player.as_ref().map(|p| !p.inventory.is_empty()).unwrap_or(false);
        if !has_inventory {
            print_center(ctx, 2, 22, 75, t.dim, &t, "Your inventory is empty. Nothing to craft.");
            print_hint(ctx, 30, 25, "[Esc/Enter]", " Leave", &t);
            return;
        }

        match self.craft_phase {
            CraftPhase::SelectItem => {
                draw_subpanel(ctx, 2, 3, 75, 38, "SELECT ITEM  ↑↓ Navigate · Enter Confirm", &t);
                if let Some(ref p) = self.player {
                    for (i, item) in p.inventory.iter().enumerate() {
                        let is_sel = i == self.craft_item_cursor;
                        print_selectable(ctx, 5, 5 + i as i32 * 2, is_sel,
                            &format!("[{}] {} · {}", i+1, item.name, item.rarity.name()),
                            self.frame, &t);
                        if is_sel {
                            for (j, m) in item.stat_modifiers.iter().enumerate().take(3) {
                                let vc = if m.value > 0 { ac } else { dng };
                                ctx.print_color(10, 6 + i as i32 * 2 + j as i32, vc, bg,
                                    &format!("{:+} {}", m.value, m.stat));
                            }
                        }
                    }
                }
                draw_separator(ctx, 2, 44, 75, &t);
                print_hint(ctx, 4, 45, "↑↓", " Navigate   ", &t);
                print_hint(ctx, 20, 45, "Enter", " Select   ", &t);
                print_hint(ctx, 35, 45, "Esc", " Leave", &t);
            }
            CraftPhase::SelectOp => {
                let (item_name, item_rarity) = self.player.as_ref()
                    .and_then(|p| p.inventory.get(self.craft_item_cursor))
                    .map(|i| (i.name.clone(), i.rarity.name()))
                    .unwrap_or_default();

                ctx.print_color(3, 3, hd, bg, &format!("Crafting: {}", &item_name.chars().take(50).collect::<String>()));
                ctx.print_color(3, 4, dim, bg, &format!("Rarity: {}", item_rarity));
                draw_separator(ctx, 2, 5, 75, &t);

                let ops = [
                    ("Reforge",    "Chaos-reroll ALL stat modifiers from scratch",     t.warn),
                    ("Augment",    "ADD one new chaos-rolled modifier",                 t.success),
                    ("Annul",      "REMOVE one random modifier",                       t.danger),
                    ("Corrupt",    "Unpredictable chaos effect — anything can happen", t.xp),
                    ("Fuse",       "DOUBLE all values and upgrade rarity tier",        t.gold),
                    ("EngineLock", "Lock a chaos engine signature into the item",      t.mana),
                ];
                for (i, (name, desc, col)) in ops.iter().enumerate() {
                    let is_sel = i == self.craft_op_cursor;
                    let y = 8 + i as i32 * 5;
                    if is_sel { draw_subpanel(ctx, 2, y - 1, 75, 4, "", &t); }
                    let fc = RGB::from_u8(col.0, col.1, col.2);
                    let pfx = if is_sel { format!("{} ", cursor_char(self.frame)) } else { "  ".to_string() };
                    ctx.print_color(5, y, if is_sel { fc } else { dim }, bg,
                        &format!("{}[{}] {}", pfx, i+1, name));
                    ctx.print_color(10, y + 1, dim, bg, desc);
                }

                if !self.craft_message.is_empty() {
                    draw_separator(ctx, 2, 38, 75, &t);
                    ctx.print_color(4, 39, gld, bg, &self.craft_message.chars().take(72).collect::<String>());
                }

                draw_separator(ctx, 2, 44, 75, &t);
                print_hint(ctx, 4, 45, "↑↓ / 1-6", " Select op   ", &t);
                print_hint(ctx, 28, 45, "Enter", " Apply   ", &t);
                print_hint(ctx, 43, 45, "Esc", " Back", &t);
            }
        }
    }

    // ── CHARACTER SHEET ───────────────────────────────────────────────────────

    fn draw_character_sheet(&mut self, ctx: &mut BTerm) {
        use chaos_rpg_core::factions::{Faction, ReputationTier};

        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        let p = match &self.player { Some(p) => p.clone(), None => { self.screen = AppScreen::FloorNav; return; } };

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "", &t);

        // Header
        let header = format!(" {} — {} (Lv.{}) ", p.name, p.class.name(), p.level);
        print_center(ctx, 0, 1, 79, t.heading, &t, &header);
        draw_separator(ctx, 1, 2, 77, &t);

        // ── Left column: core stats ──
        draw_subpanel(ctx, 1, 3, 24, 20, "STATS", &t);
        let stats = [
            ("Force",     p.stats.force),
            ("Vitality",  p.stats.vitality),
            ("Mana",      p.stats.mana),
            ("Cunning",   p.stats.cunning),
            ("Precision", p.stats.precision),
            ("Entropy",   p.stats.entropy),
            ("Luck",      p.stats.luck),
        ];
        for (i, (name, val)) in stats.iter().enumerate() {
            // stat_line/draw_bar_solid take (u8,u8,u8) tuples
            let col = if *val < 0 { t.danger } else if *val >= 50 { t.gold } else { t.heading };
            stat_line(ctx, 3, 5 + i as i32 * 2, name, &format!("{:+}", val), col, &t);
            let bar_val = (*val).max(0).min(100);
            draw_bar_solid(ctx, 3, 6 + i as i32 * 2, 20, bar_val, 100, col, &t);
        }

        // Power tier
        let tier = p.power_tier();
        let tier_rgb = tier.rgb();
        let tier_col = if tier.has_effect() {
            use chaos_rpg_core::power_tier::TierEffect;
            match tier.effect() {
                TierEffect::Rainbow | TierEffect::RainbowFast => {
                    let pal = [(220u8,60u8,60u8),(220,180,40),(60,200,80),(80,200,220),(80,80,220),(180,60,200)];
                    pal[((self.frame / 4) as usize) % pal.len()]
                }
                TierEffect::Pulse => {
                    if (self.frame / 15) % 2 == 0 { tier_rgb } else { (tier_rgb.0/2, tier_rgb.1/2, tier_rgb.2/2) }
                }
                _ => tier_rgb,
            }
        } else { tier_rgb };
        let (plabel, pval) = p.power_display();
        stat_line(ctx, 3, 20, &format!("{}: ", plabel), &pval, tier_col, &t);
        // Tier flavor text (truncated to fit inner width)
        let flavor = tier.flavor();
        let flavor_short: String = flavor.chars().take(21).collect();
        let flavor_disp = if flavor.len() > 21 { format!("{}…", flavor_short) } else { flavor_short };
        ctx.print_color(3, 21, RGB::from_u8(tier_col.0/2+10, tier_col.1/2+10, tier_col.2/2+10), bg, &flavor_disp);

        // ── Middle column: run info ──
        draw_subpanel(ctx, 27, 3, 25, 20, "RUN INFO", &t);
        stat_line(ctx, 29, 5,  "Floor  ", &format!("{}", p.floor),  t.accent, &t);
        stat_line(ctx, 29, 6,  "Kills  ", &format!("{}", p.kills),  t.success, &t);
        stat_line(ctx, 29, 7,  "Gold   ", &format!("{}g", p.gold),  t.gold, &t);
        stat_line(ctx, 29, 8,  "XP     ", &format!("{}", p.xp),     t.xp, &t);
        stat_line(ctx, 29, 9,  "HP     ", &format!("{}/{}", p.current_hp, p.max_hp),
            t.hp_color(p.current_hp as f32 / p.max_hp.max(1) as f32), &t);
        stat_line(ctx, 29, 10, "MP     ", &format!("{}/{}", self.current_mana, self.max_mana()), t.mana, &t);
        stat_line(ctx, 29, 11, "Corrupt", &format!("{}", p.corruption), t.warn, &t);
        stat_line(ctx, 29, 12, "SkPts  ", &format!("{} avail", p.skill_points), t.accent, &t);
        if p.misery.misery_index >= 100.0 {
            stat_line(ctx, 29, 13, "Misery ", &format!("{:.0}", p.misery.misery_index), t.warn, &t);
        }
        if p.underdog_multiplier() > 1.01 {
            stat_line(ctx, 29, 14, "Underdg", &format!("×{:.2}", p.underdog_multiplier()), t.gold, &t);
        }
        if p.misery.defiance_rolls > 0 {
            stat_line(ctx, 29, 15, "Defianc", &format!("{}", p.misery.defiance_rolls), t.accent, &t);
        }
        if p.misery.spite > 0.0 {
            stat_line(ctx, 29, 16, "Spite  ", &format!("{:.0}", p.misery.spite), t.danger, &t);
        }
        stat_line(ctx, 29, 17, "Class  ", p.class.name(), t.heading, &t);
        stat_line(ctx, 29, 18, "BG     ", p.background.name(), t.dim, &t);

        // ── Right column: factions ──
        draw_subpanel(ctx, 54, 3, 24, 20, "FACTIONS", &t);
        let factions = [
            ("Order",    p.faction_rep.order,    Faction::OrderOfConvergence),
            ("Cult",     p.faction_rep.cult,     Faction::CultOfDivergence),
            ("Watchers", p.faction_rep.watchers, Faction::WatchersOfBoundary),
        ];
        for (i, (fname, frep, fvar)) in factions.iter().enumerate() {
            let ftier = ReputationTier::from_rep(*frep);
            let fc = match ftier {
                ReputationTier::Hostile    => dng,
                ReputationTier::Neutral    => dim,
                ReputationTier::Recognized => suc,
                ReputationTier::Trusted    => gld,
                ReputationTier::Exalted    => hd,
            };
            let fy = 5 + i as i32 * 5;
            ctx.print_color(56, fy, fc, bg, fname);
            ctx.print_color(56, fy + 1, fc, bg, &format!("  {} ({:+})", ftier.name(), frep));
            if let Some(bonus) = chaos_rpg_core::factions::FactionRep::passive_bonus(*fvar, ftier) {
                let bonus_short: String = bonus.chars().take(20).collect();
                ctx.print_color(56, fy + 2, dim, bg, &format!("  {}", bonus_short));
            }
        }

        // ── Inventory ──
        draw_subpanel(ctx, 1, 24, 38, 18, "INVENTORY", &t);
        if p.inventory.is_empty() {
            ctx.print_color(3, 26, dim, bg, "(empty)");
        }
        for (i, item) in p.inventory.iter().take(8).enumerate() {
            let iy = 26 + i as i32 * 2;
            let ic = match item.rarity {
                Rarity::Common    => dim,
                Rarity::Uncommon  => suc,
                Rarity::Rare      => ac,
                Rarity::Epic      => RGB::from_u8(160, 0, 220),
                Rarity::Legendary => gld,
                Rarity::Mythical  => RGB::from_u8(255, 50, 50),
                Rarity::Divine    => RGB::from_u8(255, 215, 0),
                _                 => RGB::from_u8(255, 255, 255),
            };
            ctx.print_color(3, iy, ic, bg, &item.name.chars().take(22).collect::<String>());
            let mods: String = item.stat_modifiers.iter()
                .map(|m| format!("{:+}{}", m.value, &m.stat[..3.min(m.stat.len())]))
                .collect::<Vec<_>>().join(" ");
            ctx.print_color(3, iy + 1, dim, bg, &format!("  {}", mods.chars().take(30).collect::<String>()));
        }

        // ── Spells ──
        draw_subpanel(ctx, 41, 24, 37, 18, "SPELLS", &t);
        if p.known_spells.is_empty() {
            ctx.print_color(43, 26, dim, bg, "(no spells known)");
        }
        for (i, spell) in p.known_spells.iter().take(8).enumerate() {
            let sy = 26 + i as i32 * 2;
            ctx.print_color(43, sy, ac, bg,
                &format!("[{}] {}", i + 1, spell.name.chars().take(18).collect::<String>()));
            ctx.print_color(43, sy + 1, dim, bg,
                &format!("    {}mp  ×{:.1}", spell.mana_cost, spell.scaling_factor.abs()));
        }

        // ── Passive Tree summary ──
        draw_subpanel(ctx, 1, 43, 77, 4, "PASSIVE TREE", &t);
        let sp = p.skill_points;
        if sp > 0 {
            let pulse = (self.frame / 12) % 2 == 0;
            let pc = if pulse { RGB::from_u8(t.gold.0, t.gold.1, t.gold.2) }
                     else { RGB::from_u8(t.gold.0/2+20, t.gold.1/2+20, 10) };
            ctx.print_color(3, 44, pc, bg, &format!("★ {} SKILL POINT{} AVAILABLE — Press [N] to auto-allocate",
                sp, if sp == 1 { "" } else { "S" }));
        } else {
            ctx.print_color(3, 44, dim, bg, "All skill points spent.");
        }
        let node_count = p.allocated_nodes.len();
        ctx.print_color(3, 45, dim, bg, &format!("{} nodes allocated  |  class: {}", node_count, p.class.passive_name()));
        print_hint(ctx, 3, 46, "[N]", " Auto-allocate  ", &t);
        print_hint(ctx, 22, 46, "[B]", " Body Chart  ", &t);
        print_hint(ctx, 38, 46, "[Esc]", " Back to floor", &t);
    }

    // ── BODY CHART ────────────────────────────────────────────────────────────

    fn draw_body_chart(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        let p = match &self.player { Some(p) => p.clone(), None => { self.screen = AppScreen::FloorNav; return; } };

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "", &t);
        print_center(ctx, 0, 1, 79, t.heading, &t, "BODY CONDITION");
        draw_separator(ctx, 1, 2, 77, &t);

        // Combat summary at top
        let summary = p.body.combat_summary();
        ctx.print_color(2, 3, if summary.contains("CRITICAL") || summary.contains("SEVERED") { dng } else { dim }, bg, &summary.chars().take(75).collect::<String>());

        // Two-column body part display with visual HP bars
        draw_subpanel(ctx, 1, 5, 77, 36, "BODY PARTS", &t);

        use chaos_rpg_core::body::BodyPart;
        let col_parts: &[&[BodyPart]] = &[
            &[BodyPart::Head, BodyPart::Torso, BodyPart::Neck,
              BodyPart::LeftArm, BodyPart::RightArm,
              BodyPart::LeftHand, BodyPart::RightHand],
            &[BodyPart::LeftLeg, BodyPart::RightLeg,
              BodyPart::LeftFoot, BodyPart::RightFoot,
              BodyPart::LeftEye, BodyPart::RightEye],
        ];
        for (col, parts) in col_parts.iter().enumerate() {
            let cx = 3 + col as i32 * 38;
            for (row, &part) in parts.iter().enumerate() {
                let ry = 7 + row as i32 * 4;
                let state = p.body.parts.get(&part);
                let (cur, max_hp, sev) = state
                    .map(|s| (s.current_hp, s.max_hp, s.injury.as_ref().map(|i| i.name())))
                    .unwrap_or((10, 10, None));
                let pct = if max_hp > 0 { cur as f32 / max_hp as f32 } else { 0.0 };
                let bar_col = t.hp_color(pct.clamp(0.0, 1.0));
                let sev_lbl = sev.unwrap_or("Healthy");
                let fg = if pct <= 0.0 { dng }
                         else if pct < 0.35 { dng }
                         else if pct < 0.65 { RGB::from_u8(200, 130, 40) }
                         else { suc };
                // Part name + HP numbers
                let name_str = part.name();
                ctx.print_color(cx, ry,     hd,  bg, &format!("{:<12}", name_str));
                ctx.print_color(cx + 12, ry, fg, bg, &format!("{}/{}", cur, max_hp));
                // Severity label
                ctx.print_color(cx + 22, ry, fg, bg, sev_lbl);
                // HP bar (width 32)
                draw_bar_gradient(ctx, cx, ry + 1, 32, cur.max(0), max_hp.max(1), bar_col, t.muted, &t);
                // Armor note if equipped
                if let Some(state) = p.body.parts.get(&part) {
                    if state.armor_defense > 0 {
                        ctx.print_color(cx + 20, ry + 2, dim, bg,
                            &format!("DEF+{}", state.armor_defense));
                    }
                }
            }
        }

        draw_separator(ctx, 1, 44, 77, &t);
        print_hint(ctx, 2, 45, "[Esc]", " Back to floor", &t);
        print_hint(ctx, 18, 45, "[C]", " Character Sheet", &t);
        // Overall warning
        let summary = p.body.combat_summary();
        if summary.contains("CRITICAL") || summary.contains("SEVERED") {
            ctx.print_color(42, 45, dng, bg, &format!("⚠ {}", &summary.chars().take(34).collect::<String>()));
        }
    }

    // ── GAME OVER ─────────────────────────────────────────────────────────────

    fn draw_game_over(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);

        ctx.cls_bg(bg);
        // Danger border
        ctx.draw_box(0, 0, 79, 49,
            RGB::from_u8(t.danger.0, t.danger.1, t.danger.2),
            bg);

        // Flashing "YOU DIED" (pulse every 30 frames)
        let pulse = if (self.frame / 30) % 2 == 0 { dng } else { hd };
        ctx.print_color(17, 5,  pulse, bg, "╔══════════════════════════════════════════╗");
        ctx.print_color(17, 6,  pulse, bg, "║         Y  O  U     D  I  E  D           ║");
        ctx.print_color(17, 7,  dng,   bg, "║     The mathematics have consumed you.   ║");
        ctx.print_color(17, 8,  pulse, bg, "╚══════════════════════════════════════════╝");

        if let Some(ref p) = self.player {
            draw_subpanel(ctx, 2, 11, 75, 30, "RUN SUMMARY", &t);

            // ── Identity + cause of death ──
            ctx.print_color(4, 13, hd, bg,
                &format!("{} · {} · Lv.{} · Floor {}", p.name, p.class.name(), p.level, p.floor));
            let cause: String = p.run_stats.cause_of_death.chars().take(60).collect();
            ctx.print_color(4, 14, dng, bg, &format!("☠  {}", cause));

            draw_separator(ctx, 3, 15, 73, &t);

            // ── Combat stats (two columns) ──
            // Left column
            stat_line(ctx, 4, 16, "Kills    ", &format!("{}", p.kills),  t.success, &t);
            stat_line(ctx, 4, 17, "Gold     ", &format!("{}g", p.gold),  t.gold, &t);
            stat_line(ctx, 4, 18, "XP       ", &format!("{}", p.xp),     t.xp, &t);
            stat_line(ctx, 4, 19, "Spells   ", &format!("{}", p.spells_cast), t.mana, &t);
            stat_line(ctx, 4, 20, "Corrupt  ", &format!("{}", p.corruption), t.danger, &t);

            // Right column — damage summary
            let dealt = p.run_stats.damage_dealt;
            let taken = p.run_stats.damage_taken;
            let ratio = if taken > 0 { dealt as f64 / taken as f64 } else { dealt as f64 };
            let ratio_col = if ratio >= 2.0 { t.success } else if ratio >= 1.0 { t.gold } else { t.danger };
            stat_line(ctx, 40, 16, "Dmg Dealt ", &format!("{}", dealt), t.success, &t);
            stat_line(ctx, 40, 17, "Dmg Taken ", &format!("{}", taken), t.danger, &t);
            stat_line(ctx, 40, 18, "D/T Ratio ", &format!("{:.2}", ratio), ratio_col, &t);
            let fbd = p.run_stats.final_blow_damage;
            if fbd > 0 {
                stat_line(ctx, 40, 19, "Final Blow", &format!("{}", fbd), t.danger, &t);
            }
            let best_hit = p.run_stats.highest_single_hit;
            if best_hit > 0 {
                stat_line(ctx, 40, 20, "Best Hit  ", &format!("{}", best_hit), t.gold, &t);
            }

            draw_separator(ctx, 3, 21, 73, &t);
            for (i, line) in p.run_summary().iter().enumerate().take(18) {
                ctx.print_color(4, 22 + i as i32, dim, bg, &line.chars().take(72).collect::<String>());
            }
        }

        if let Some(ref nem) = self.nemesis_record {
            ctx.print_color(2, 37, dng, bg,
                &format!("☠ {} is now your Nemesis — will return stronger.", &nem.enemy_name.chars().take(30).collect::<String>()));
        }

        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 10, 46, "[Enter]", " Return to title   ", &t);
        print_hint(ctx, 40, 46, "[S]", " Scoreboard", &t);
    }

    // ── VICTORY ───────────────────────────────────────────────────────────────

    fn draw_victory(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        ctx.cls_bg(bg);
        ctx.draw_box(0, 0, 79, 49, gld, bg);

        // Animated shimmer on victory banner
        let shimmer_t = (self.frame as f32 * 0.05).sin() * 0.2 + 0.8;
        let sc = Theme::lerp(t.gold, t.heading, shimmer_t);
        let shimmer = RGB::from_u8(sc.0, sc.1, sc.2);

        ctx.print_color(14, 5,  shimmer, bg, "╔═══════════════════════════════════════════════╗");
        ctx.print_color(14, 6,  shimmer, bg, "║   ★  V  I  C  T  O  R  Y  ★                  ║");
        ctx.print_color(14, 7,  gld,     bg, "║   You solved 10 floors of pure mathematical   ║");
        ctx.print_color(14, 8,  gld,     bg, "║   chaos. The algorithms bow before you.       ║");
        ctx.print_color(14, 9,  shimmer, bg, "╚═══════════════════════════════════════════════╝");

        if let Some(ref p) = self.player {
            draw_subpanel(ctx, 2, 12, 75, 26, "FINAL STATS", &t);
            ctx.print_color(4, 14, hd, bg,
                &format!("{} · {} · Lv.{}", p.name, p.class.name(), p.level));
            stat_line(ctx, 4, 15, "Floors    ", &format!("{}", p.floor), t.gold, &t);
            stat_line(ctx, 4, 16, "Kills     ", &format!("{}", p.kills), t.success, &t);
            stat_line(ctx, 4, 17, "Gold      ", &format!("{}g", p.gold), t.gold, &t);
            stat_line(ctx, 4, 18, "XP        ", &format!("{}", p.xp), t.xp, &t);
            draw_separator(ctx, 3, 19, 73, &t);
            for (i, line) in p.run_summary().iter().enumerate().take(14) {
                ctx.print_color(4, 20 + i as i32, if i == 0 { suc } else { dim }, bg, &line.chars().take(72).collect::<String>());
            }
        }

        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 10, 46, "[Enter]", " Return to title   ", &t);
        print_hint(ctx, 40, 46, "[S]", " Scoreboard", &t);
    }

    // ── SCOREBOARD ────────────────────────────────────────────────────────────

    fn draw_scoreboard(&mut self, ctx: &mut BTerm) {
        let t = self.theme().clone();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let wrn = RGB::from_u8(t.warn.0,   t.warn.1,   t.warn.2);

        ctx.cls_bg(bg);
        draw_panel(ctx, 0, 0, 79, 49, "SCOREBOARD", &t);

        // ── Hall of Chaos (regular scores) ──
        ctx.print_color(2, 2, hd, bg, "HALL OF CHAOS");
        let scores = load_scores();
        if scores.is_empty() {
            ctx.print_color(4, 4, dim, bg, "No scores yet. Play and die bravely.");
        } else {
            ctx.print_color(2, 3, dim, bg,
                &format!("{:<4} {:<10} {:<16} {:<12} {:<5} {:<5}",
                    "Rank", "Score", "Name", "Class", "Flr", "Kills"));
            draw_separator(ctx, 2, 4, 75, &t);
            for (i, s) in scores.iter().enumerate().take(10) {
                let row_col = match i { 0 => gld, 1 => hd, 2 => ac, _ => dim };
                let medal = match i { 0 => "★ ", 1 => "◆ ", 2 => "● ", _ => "  " };
                ctx.print_color(2, 5 + i as i32, row_col, bg,
                    &format!("{}{:<3}  {:<10} {:<16} {:<12} {:<5} {}",
                        medal, i+1, s.score, &s.name.chars().take(16).collect::<String>(),
                        &s.class.chars().take(12).collect::<String>(),
                        s.floor_reached, s.enemies_defeated));
            }
        }

        // ── Hall of Misery ──
        let misery_y = 17i32;
        draw_separator(ctx, 2, misery_y - 1, 75, &t);
        ctx.print_color(2, misery_y, dng, bg, "HALL OF MISERY");
        let mscores = load_misery_scores();
        if mscores.is_empty() {
            ctx.print_color(4, misery_y + 2, dim, bg, "No misery recorded. Suffer more.");
        } else {
            ctx.print_color(2, misery_y + 1, dim, bg,
                &format!("{:<4} {:<8} {:<14} {:<12} {:<5} {:<18}",
                    "Rank", "Misery", "Name", "Class", "Flr", "Cause of death"));
            draw_separator(ctx, 2, misery_y + 2, 75, &t);
            for (i, m) in mscores.iter().enumerate().take(10) {
                let row_col = match i { 0 => dng, 1 => wrn, 2 => ac, _ => dim };
                let medal = match i { 0 => "☠ ", 1 => "✦ ", 2 => "● ", _ => "  " };
                let cause: String = m.cause_of_death.chars().take(18).collect();
                ctx.print_color(2, misery_y + 3 + i as i32, row_col, bg,
                    &format!("{}{:<3}  {:<8.0} {:<14} {:<12} {:<5} {}",
                        medal, i+1, m.misery_index,
                        &m.name.chars().take(14).collect::<String>(),
                        &m.class.chars().take(12).collect::<String>(),
                        m.floor_reached, cause));
            }
        }

        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 4, 46, "[Esc/Q]", " Back to title", &t);
    }
}

// ─── INPUT HANDLER ────────────────────────────────────────────────────────────

impl State {
    // ── AUTO-PLAY TICK ─────────────────────────────────────────────────────────
    //
    // Fires once per AUTO_DELAY frames.  Handles floor navigation, combat AI,
    // and non-item room resolution automatically.  Pauses on:
    //   • Item pickup prompts (the player still decides what to keep)
    //   • Shop / Crafting screens (player manages gold/items)
    //   • GameOver / Victory (nothing to advance)
    //
    fn tick_auto_play(&mut self, _ctx: &mut BTerm) {
        const AUTO_DELAY: u64 = 60; // ~1 s at 60 fps
        if self.frame.saturating_sub(self.auto_last_action) < AUTO_DELAY {
            return;
        }

        match self.screen.clone() {
            // ── Floor navigation ─────────────────────────────────────────────
            AppScreen::FloorNav => {
                // Auto-allocate any pending skill points first
                if self.player.as_ref().map(|p| p.skill_points > 0).unwrap_or(false) {
                    let seed = self.floor_seed.wrapping_add(self.frame);
                    if let Some(ref mut p) = self.player {
                        let msgs = p.auto_allocate_passives(seed);
                        for m in msgs { self.push_log(m); }
                    }
                }

                if self.floor.as_ref().map(|f| f.rooms_remaining() == 0).unwrap_or(false) {
                    // All rooms done — descend
                    if self.floor_num >= self.max_floor {
                        self.save_score_now();
                        self.screen = AppScreen::Victory;
                    } else {
                        self.floor_num += 1;
                        self.generate_floor_for_current();
                    }
                } else {
                    self.enter_current_room();
                }
                self.auto_last_action = self.frame;
            }

            // ── Combat — player always picks manually, even in auto mode ─────
            AppScreen::Combat => {
                // Do nothing; input is handled in draw_combat key handler.
            }

            // ── Non-item room events — auto-accept ───────────────────────────
            AppScreen::RoomView => {
                let has_item = self.room_event.pending_item.is_some();
                if has_item {
                    // Pause so the player can decide
                    return;
                }
                // Portal: skip through automatically (risky but exciting)
                if self.room_event.portal_available {
                    self.room_event.portal_available = false;
                    if self.floor_num >= self.max_floor {
                        self.save_score_now();
                        self.screen = AppScreen::Victory;
                    } else {
                        self.floor_num += 1;
                        self.generate_floor_for_current();
                        self.screen = AppScreen::FloorNav;
                    }
                } else {
                    // Auto-accept all other room events (shrine bless, trap damage, etc.)
                    self.room_event.pending_spell = None;
                    self.advance_floor_room();
                    if self.screen != AppScreen::GameOver && self.screen != AppScreen::Victory {
                        self.screen = AppScreen::FloorNav;
                    }
                }
                self.auto_last_action = self.frame;
            }

            // Pause on screens where the player makes deliberate choices
            AppScreen::Shop
            | AppScreen::Crafting
            | AppScreen::GameOver
            | AppScreen::Victory => {}

            _ => {}
        }
    }

    /// Choose a combat action using a simple HP-aware strategy:
    ///   HP < 25%          → Defend
    ///   MP > 30% & spells → cast best available spell
    ///   otherwise         → Attack
    fn auto_combat_action(&self) -> CombatAction {
        let (hp, max_hp, mp, max_mp, spell_count) = match &self.player {
            Some(p) => (
                p.current_hp, p.max_hp,
                self.current_mana, self.max_mana(),
                p.known_spells.len(),
            ),
            None => return CombatAction::Attack,
        };

        let hp_pct = hp as f32 / max_hp.max(1) as f32;
        if hp_pct < 0.25 {
            return CombatAction::Defend;
        }

        // Use a spell when mana is plentiful
        let mp_pct = mp as f32 / max_mp.max(1) as f32;
        if spell_count > 0 && mp_pct > 0.30 {
            return CombatAction::UseSpell(0);
        }

        CombatAction::Attack
    }

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
                VirtualKeyCode::T => self.cycle_theme(),
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
                VirtualKeyCode::Up   => { self.boon_cursor = self.boon_cursor.saturating_sub(1); self.emit_audio(AudioEvent::MenuNavigate); }
                VirtualKeyCode::Down => { self.boon_cursor = (self.boon_cursor + 1).min(2); self.emit_audio(AudioEvent::MenuNavigate); }
                VirtualKeyCode::Key1 => { self.boon_cursor = 0; self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Key2 => { self.boon_cursor = 1; self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Key3 => { self.boon_cursor = 2; self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Return => { self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Escape => { self.emit_audio(AudioEvent::MenuCancel); self.screen = AppScreen::CharacterCreation; }
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
                    self.screen = AppScreen::CharacterSheet;
                }
                VirtualKeyCode::B => {
                    self.screen = AppScreen::BodyChart;
                }
                VirtualKeyCode::Z => {
                    self.auto_mode = !self.auto_mode;
                    self.auto_last_action = 0;
                    if self.auto_mode {
                        // Auto-alloc any pending points immediately on enabling
                        let seed = self.floor_seed.wrapping_add(self.frame);
                        if let Some(ref mut p) = self.player {
                            if p.skill_points > 0 {
                                let msgs = p.auto_allocate_passives(seed);
                                for m in msgs { self.push_log(m); }
                            }
                        }
                        self.push_log("AUTO PILOT ON — pauses for item pickups and shop/craft".to_string());
                    } else {
                        self.push_log("Auto pilot OFF.".to_string());
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
                    // Emit pre-action audio
                    match &act {
                        CombatAction::Attack => self.emit_audio(AudioEvent::PlayerAttack),
                        CombatAction::HeavyAttack => self.emit_audio(AudioEvent::PlayerHeavyAttack),
                        CombatAction::Defend => self.emit_audio(AudioEvent::PlayerDefend),
                        CombatAction::UseSpell(idx) => self.emit_audio(AudioEvent::SpellCast { spell_index: *idx }),
                        _ => {}
                    }
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

            AppScreen::CharacterSheet => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::C | VirtualKeyCode::Return => {
                    self.screen = AppScreen::FloorNav;
                }
                VirtualKeyCode::B => {
                    self.screen = AppScreen::BodyChart;
                }
                _ => {}
            },

            AppScreen::BodyChart => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::B | VirtualKeyCode::Return => {
                    self.screen = AppScreen::FloorNav;
                }
                VirtualKeyCode::C => {
                    self.screen = AppScreen::CharacterSheet;
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
        .with_tile_dimensions(12, 12)
        .with_dimensions(80, 50)
        .with_fps_cap(60.0)
        .with_fullscreen(true);
    main_loop(builder.build()?, State::new())
}
