//! CHAOS RPG — Graphical Frontend (bracket-lib)
//!
//! Full game parity with the terminal version. All room types, shops,
//! treasure, shrines, traps, chaos rifts, portals, and crafting bench
//! are implemented. Run with --fullscreen / -f for fullscreen.

use bracket_lib::prelude::*;
use chaos_rpg_core::{
    character::{Background, Character, CharacterClass, Difficulty, StatusEffect},
    chaos_pipeline::chaos_roll_verbose,
    enemy::{generate_enemy, Enemy},
    items::Item,
    scoreboard::{load_scores, save_score, ScoreEntry},
    spells::Spell,
    world::{generate_floor, Floor, RoomType},
};

mod renderer;
mod sprites;
mod ui_overlay;

// ─── SCREENS ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
    Title,
    CharacterCreation,
    FloorNav,       // floor minimap + advance / enter room
    RoomView,       // non-combat room event (treasure, shrine, trap, etc.)
    Combat,
    Shop,
    Crafting,
    GameOver,
    Scoreboard,
}

// ─── PENDING ROOM DATA ────────────────────────────────────────────────────────

/// Data carried from room resolution into the RoomView screen.
struct RoomEvent {
    title: String,
    lines: Vec<String>,             // descriptive lines rendered in the panel
    pending_item: Option<Item>,     // item awaiting [P]ick-up
    pending_spell: Option<Spell>,   // spell awaiting [L]earn
    gold_delta: i64,
    hp_delta: i64,
    mana_delta: i64,
    stat_bonuses: Vec<(&'static str, i64)>,
    damage_taken: i64,
    portal_available: bool,
    resolved: bool,                 // stat/gold effects already applied
}

impl RoomEvent {
    fn empty() -> Self {
        Self {
            title: String::new(),
            lines: Vec::new(),
            pending_item: None,
            pending_spell: None,
            gold_delta: 0,
            hp_delta: 0,
            mana_delta: 0,
            stat_bonuses: Vec::new(),
            damage_taken: 0,
            portal_available: false,
            resolved: false,
        }
    }
}

// ─── STATE ────────────────────────────────────────────────────────────────────

struct State {
    screen: AppScreen,
    player: Option<Character>,
    floor: Option<Floor>,
    enemy: Option<Enemy>,
    combat_log: Vec<String>,
    seed: u64,
    frame: u64,
    selected_menu: usize,
    cc_class: usize,
    cc_bg: usize,
    cc_diff: usize,
    fullscreen: bool,
    current_mana: i64,
    floor_num: u32,
    room_event: RoomEvent,
    // shop state
    shop_items: Vec<(Item, i64)>,   // (item, price)
    shop_heal_cost: i64,
    shop_cursor: usize,
    // crafting state
    craft_cursor: usize,
    craft_message: String,
}

impl State {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        let fullscreen = std::env::args().any(|a| a == "--fullscreen" || a == "-f");
        State {
            screen: AppScreen::Title,
            player: None, floor: None, enemy: None,
            combat_log: Vec::new(),
            seed, frame: 0,
            selected_menu: 0, cc_class: 0, cc_bg: 0, cc_diff: 1,
            fullscreen,
            current_mana: 0,
            floor_num: 1,
            room_event: RoomEvent::empty(),
            shop_items: Vec::new(), shop_heal_cost: 20, shop_cursor: 0,
            craft_cursor: 0, craft_message: String::new(),
        }
    }

    fn max_mana(&self) -> i64 {
        self.player.as_ref().map(|p| (p.stats.mana + 50).max(50)).unwrap_or(50)
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.combat_log.push(msg.into());
        if self.combat_log.len() > 200 { self.combat_log.remove(0); }
    }

    fn enemy_counter_attack(&mut self) {
        if let (Some(ref mut player), Some(ref enemy)) = (&mut self.player, &self.enemy) {
            if enemy.hp <= 0 { return; }
            let roll = chaos_roll_verbose(enemy.chaos_level, self.seed.wrapping_add(self.frame + 7));
            let dmg = ((enemy.base_damage as f64 * (1.0 + roll.final_value.abs() * 0.3)) as i64).max(1);
            player.take_damage(dmg);
            self.combat_log.push(format!("{} attacks for {}!", enemy.name, dmg));
        }
    }

    fn check_combat_end(&mut self) -> bool {
        let enemy_dead  = self.enemy.as_ref().map(|e| e.hp <= 0).unwrap_or(false);
        let player_dead = self.player.as_ref().map(|p| p.current_hp <= 0).unwrap_or(false);
        if enemy_dead {
            if let (Some(ref mut p), Some(ref e)) = (&mut self.player, &self.enemy) {
                p.kills += 1;
                p.gain_xp(e.xp_reward);
                p.gold += e.gold_reward;
                self.combat_log.push(format!("{} slain! +{} XP  +{} gold", e.name, e.xp_reward, e.gold_reward));
                // loot drop
                let loot_seed = self.seed.wrapping_add(self.frame).wrapping_mul(6364136223846793005);
                if loot_seed % 100 < 40 {
                    let loot = Item::generate(loot_seed);
                    self.combat_log.push(format!("Item dropped: {}", loot.name));
                    // Queue it in room_event for [P] pick-up prompt after combat
                    self.room_event = RoomEvent::empty();
                    self.room_event.title = "LOOT DROPPED".to_string();
                    self.room_event.lines = vec![
                        format!("Enemy dropped: {}", loot.name),
                        format!("Rarity: {:?}", loot.rarity),
                        String::new(),
                        "[P] Pick up   [any] Leave".to_string(),
                    ];
                    self.room_event.pending_item = Some(loot);
                }
            }
            self.enemy = None;
            if self.room_event.pending_item.is_some() {
                self.screen = AppScreen::RoomView;
            } else {
                self.screen = AppScreen::FloorNav;
            }
            return true;
        }
        if player_dead {
            self.screen = AppScreen::GameOver;
            return true;
        }
        false
    }

    /// Build a RoomEvent for the current floor room and advance to RoomView or Combat.
    fn enter_current_room(&mut self) {
        let floor_num = self.floor_num;
        let seed = self.seed.wrapping_add(
            self.floor.as_ref().map(|f| f.current_room as u64 * 1337).unwrap_or(0)
        );

        let room_type = self.floor.as_ref().map(|f| f.current().room_type.clone()).unwrap_or(RoomType::Empty);
        let room_desc = self.floor.as_ref().map(|f| f.current().description.clone()).unwrap_or_default();

        match room_type {
            RoomType::Combat | RoomType::Boss => {
                let is_boss = room_type == RoomType::Boss;
                let enemy_floor = if is_boss { floor_num + 2 } else { floor_num };
                let enemy = generate_enemy(enemy_floor.max(1), seed);
                self.enemy = Some(enemy);
                self.combat_log.clear();
                if is_boss { self.push_log("★ BOSS BATTLE ★".to_string()); }
                self.screen = AppScreen::Combat;
            }

            RoomType::Treasure => {
                let item = Item::generate(seed);
                let gold_bonus = ((seed % 30 + 10) as i64) * floor_num as i64;

                let mut ev = RoomEvent::empty();
                ev.title = "★ TREASURE ROOM ★".to_string();
                ev.lines = vec![
                    room_desc,
                    String::new(),
                    format!("You find {} gold!", gold_bonus),
                    String::new(),
                    format!("Item: {}", item.name),
                    format!("Rarity: {:?}", item.rarity),
                    String::new(),
                ];
                for m in &item.stat_modifiers {
                    ev.lines.push(format!("  {:+} {}", m.value, m.stat));
                }
                ev.lines.push(String::new());
                ev.lines.push("[P] Pick up item   [any] Leave it".to_string());
                ev.gold_delta = gold_bonus;
                ev.pending_item = Some(item);

                // 25% chance of spell scroll
                if seed % 4 == 0 {
                    let spell = Spell::generate(seed.wrapping_add(54321));
                    ev.lines.push(String::new());
                    ev.lines.push(format!("+ SPELL SCROLL: {}", spell.name));
                    ev.lines.push(format!("  {}mp  dmg×{:.1}", spell.mana_cost, spell.scaling_factor.abs()));
                    ev.lines.push("[L] Learn spell   [skip] Leave scroll".to_string());
                    ev.pending_spell = Some(spell);
                }

                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Shop => {
                let heal_cost = 15 + floor_num as i64 * 2;
                let mut shop = Vec::new();
                for i in 0..4u64 {
                    let item = Item::generate(seed.wrapping_add(i * 9999));
                    let cunning = self.player.as_ref().map(|p| p.stats.cunning).unwrap_or(0);
                    let price = (item.value as i64 + floor_num as i64 * 5 - cunning / 10).max(5);
                    shop.push((item, price));
                }
                self.shop_items = shop;
                self.shop_heal_cost = heal_cost;
                self.shop_cursor = 0;
                self.screen = AppScreen::Shop;
            }

            RoomType::Shrine => {
                let roll = chaos_roll_verbose(
                    self.player.as_ref().map(|p| p.stats.entropy as f64 * 0.01).unwrap_or(0.1), seed
                );
                let stats: &[&str] = &["vitality","force","mana","cunning","precision","entropy","luck"];
                let stat_name = stats[(seed % stats.len() as u64) as usize];
                let buff = 3 + (roll.to_range(1, 10) as i64) + floor_num as i64 / 2;
                let hp_restore = self.player.as_ref().map(|p| p.max_hp / 5).unwrap_or(10);

                let mut ev = RoomEvent::empty();
                ev.title = "~ SHRINE ~".to_string();
                ev.lines = vec![
                    room_desc,
                    String::new(),
                    format!("The shrine blesses you! +{} {}", buff, stat_name),
                    format!("You feel restored. +{} HP", hp_restore),
                    String::new(),
                    "[ENTER] Accept blessing".to_string(),
                ];
                ev.stat_bonuses = vec![(stat_name, buff)];
                ev.hp_delta = hp_restore;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Trap => {
                let cunning = self.player.as_ref().map(|p| p.stats.cunning).unwrap_or(0);
                let roll = chaos_roll_verbose(cunning as f64 * 0.01, seed);
                let evaded = roll.final_value > 0.0;
                let trap_damage = if evaded { 0 } else { 5 + floor_num as i64 * 3 + (seed % 10) as i64 };

                let mut ev = RoomEvent::empty();
                ev.title = "! TRAP ROOM !".to_string();
                ev.lines = vec![
                    room_desc,
                    String::new(),
                    if evaded { "You spot and dodge the trap!".to_string() }
                    else { format!("TRAP TRIGGERED! -{} HP!", trap_damage) },
                    String::new(),
                    "[ENTER] Continue".to_string(),
                ];
                ev.damage_taken = trap_damage;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Portal => {
                let mut ev = RoomEvent::empty();
                ev.title = "^ PORTAL ^".to_string();
                ev.lines = vec![
                    room_desc,
                    String::new(),
                    "A shimmering rift to the next floor.".to_string(),
                    String::new(),
                    "[P] Step through portal".to_string(),
                    "[any] Resist the pull".to_string(),
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
                    room_desc,
                    String::new(),
                    format!("The stillness restores you. +{} HP", hp_gain),
                    String::new(),
                    "[ENTER] Continue".to_string(),
                ];
                ev.hp_delta = hp_gain;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::ChaosRift => {
                let roll = chaos_roll_verbose(
                    self.player.as_ref().map(|p| p.stats.entropy as f64 * 0.015).unwrap_or(0.1), seed
                );
                let outcome = seed.wrapping_mul(floor_num as u64 * 7 + 1) % 6;
                let mut ev = RoomEvent::empty();
                ev.title = "∞ CHAOS RIFT ∞".to_string();
                ev.lines = vec![
                    "REALITY ERROR. MATHEMATICAL EXCEPTION.".to_string(),
                    String::new(),
                    format!("Chaos value: {:.4}", roll.final_value),
                    String::new(),
                ];
                match outcome {
                    0 => {
                        let gold = ((seed % 100 + 50) as i64) * floor_num as i64;
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
                        let luck_gain = 10 + floor_num as i64;
                        ev.lines.push(format!("CHAOS TRADE: -{} gold, +{} Luck!", gold_loss, luck_gain));
                        ev.gold_delta = -gold_loss;
                        ev.stat_bonuses = vec![("luck", luck_gain)];
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
                ev.lines.push("[ENTER] Accept fate".to_string());
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::CraftingBench => {
                self.craft_cursor = 0;
                self.craft_message = String::new();
                self.screen = AppScreen::Crafting;
            }
        }
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

        // Status effect tick every ~60 frames
        if self.frame % 60 == 0 {
            if let Some(ref mut p) = self.player {
                let (_net, msgs) = p.tick_status_effects();
                for m in msgs { self.combat_log.push(m); }
            }
        }

        // Paladin regen every 90 frames (~1.5s)
        if self.frame % 90 == 0 {
            if let Some(ref mut p) = self.player {
                if let CharacterClass::Paladin = p.class {
                    let regen = 3 + p.stats.vitality / 20;
                    if regen > 0 && self.screen == AppScreen::Combat { p.heal(regen); }
                }
            }
        }

        match self.screen {
            AppScreen::Title            => self.draw_title(ctx),
            AppScreen::CharacterCreation => self.draw_char_creation(ctx),
            AppScreen::FloorNav         => self.draw_floor_nav(ctx),
            AppScreen::RoomView         => self.draw_room_view(ctx),
            AppScreen::Combat           => self.draw_combat(ctx),
            AppScreen::Shop             => self.draw_shop(ctx),
            AppScreen::Crafting         => self.draw_crafting(ctx),
            AppScreen::GameOver         => self.draw_game_over(ctx),
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
        ctx.print_color(40, 10, dim, bg, "R P G   —   Where Math Goes To Die");

        let opts = ["  New Game","  Scoreboard","  Quit"];
        let ox = 48i32; let oy = 20i32;
        ctx.draw_box(ox-2, oy-2, 30, opts.len() as i32+3, col, bg);
        for (i, opt) in opts.iter().enumerate() {
            let (fg, pfx) = if i == self.selected_menu { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(ox, oy + i as i32, fg, bg, &format!("{} {}", pfx, opt));
        }

        ctx.print_color(4, 46, dim, bg, "↑↓ Navigate   Enter Select   Q Quit");
        let fs = if self.fullscreen { "[ fullscreen active ]" } else { "[ --fullscreen for fullscreen ]" };
        ctx.print_color(4, 47, dim, bg, fs);
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

        // class passive
        let class = &CLASSES[self.cc_class].1;
        ctx.print_color(3, 17, col, bg, "PASSIVE");
        ctx.print_color(3, 18, yel, bg, class.passive_name());
        let desc = class.passive_desc();
        let mut row = 19i32;
        let mut line = String::new();
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

        // portrait
        let portrait = class.ascii_art();
        ctx.print_color(85, 5, col, bg, "PORTRAIT");
        for (i, l) in portrait.lines().enumerate() {
            ctx.print_color(85, 7 + i as i32, RGB::named(WHITE), bg, l);
        }

        ctx.print_color(3, 46, col, bg, "[ ENTER ] Start Adventure");
        ctx.print_color(3, 47, dim, bg, "↑↓=class  ←→=background  Tab=difficulty  Esc=back");
    }

    // ─── FLOOR NAV ────────────────────────────────────────────────────────────

    fn draw_floor_nav(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);

        let (p_name, p_class, p_level, p_floor, p_kills, p_gold, p_xp,
             p_hp, p_max_hp, p_status, p_corruption) = match &self.player {
            Some(p) => (p.name.clone(), p.class.name(), p.level, p.floor,
                        p.kills, p.gold, p.xp, p.current_hp, p.max_hp,
                        p.status_badge_line(), p.corruption),
            None => { self.screen = AppScreen::Title; return; }
        };

        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(3, 2, yel, bg, &format!("FLOOR {}  —  {}  Lv.{} {}", p_floor, p_name, p_level, p_class));
        ctx.print_color(3, 3, dim, bg, &format!("Kills: {}  Gold: {}  XP: {}  Corruption: {}", p_kills, p_gold, p_xp, p_corruption));

        let pp = p_hp as f32 / p_max_hp.max(1) as f32;
        ctx.print_color(3, 5, RGB::named(hp_col(pp)), bg, &format!("HP {}/{}", p_hp, p_max_hp));
        hbar(ctx, 3, 6, 50, p_hp, p_max_hp, hp_col(pp));
        ctx.print_color(3, 7, RGB::named(BLUE), bg, &format!("MP {}/{}", self.current_mana, self.max_mana()));
        hbar(ctx, 3, 8, 50, self.current_mana, self.max_mana(), BLUE);
        if !p_status.is_empty() {
            ctx.print_color(3, 10, RGB::named(MAGENTA), bg, &format!("Status: {}", p_status));
        }

        // Floor minimap
        ctx.print_color(3, 12, col, bg, "FLOOR MAP");
        ctx.print_color(3, 13, dim, bg, "─────────────────────────────────────────────────────────────────────");
        if let Some(ref floor) = self.floor {
            let per_row = 20usize;
            for (i, room) in floor.rooms.iter().enumerate() {
                let col_idx = (i % per_row) as i32;
                let row_idx = (i / per_row) as i32;
                let rx = 3 + col_idx * 5;
                let ry = 14 + row_idx * 2;
                let symbol = room.room_type.icon();
                let (r_col, marker) = if i == floor.current_room {
                    (RGB::named(WHITE), format!("[{}]", symbol.trim_matches(|c| c == '[' || c == ']')))
                } else if i < floor.current_room {
                    (dim, "···".to_string())
                } else {
                    (RGB::named(room_color(&room.room_type)), symbol.to_string())
                };
                ctx.print_color(rx, ry, r_col, bg, &marker);
            }

            // Current room info
            let current = floor.current();
            let crt = &current.room_type;
            ctx.print_color(3, 28, RGB::named(room_color(crt)), bg,
                &format!("Current: {}  —  {}", crt.name(), current.description));
        }

        // Actions
        ctx.draw_box(1, 34, 118, 14, col, bg);
        ctx.print_color(3, 35, col, bg, "ACTIONS");
        ctx.print_color(3, 36, dim, bg, "─────────────────────────────────────────────────────────────────────");
        ctx.print_color(3, 37, RGB::named(WHITE), bg, "[E] Enter current room   [N] Advance to next room   [R] Rest (+HP/MP)");
        ctx.print_color(3, 38, RGB::named(WHITE), bg, "[>] Next floor           [I] Inventory               [Q] Quit to menu");
        ctx.print_color(3, 40, dim, bg, "Room symbols:  [×]=Combat  [★]=Treasure  [$]=Shop  [☯]=Shrine");
        ctx.print_color(3, 41, dim, bg, "               [!]=Trap    [☠]=Boss      [↑]=Portal [∞]=Chaos Rift");

        // Inventory + spells sidebar
        ctx.print_color(70, 12, col, bg, "INVENTORY");
        ctx.print_color(70, 13, dim, bg, "────────────────────────────────────────");
        if let Some(ref p) = self.player {
            if p.inventory.is_empty() {
                ctx.print_color(70, 14, dim, bg, "(empty)");
            } else {
                for (i, item) in p.inventory.iter().take(8).enumerate() {
                    ctx.print_color(70, 14 + i as i32, RGB::named(GRAY), bg,
                        &format!("[{}] {}", i+1, item.name));
                }
            }
            ctx.print_color(70, 23, col, bg, "SPELLS");
            ctx.print_color(70, 24, dim, bg, "────────────────────────────────────────");
            if p.known_spells.is_empty() {
                ctx.print_color(70, 25, dim, bg, "(none)");
            } else {
                for (i, s) in p.known_spells.iter().take(8).enumerate() {
                    ctx.print_color(70, 25 + i as i32, RGB::named(MAGENTA), bg,
                        &format!("[{}] {} ({}mp)", i+1, s.name, s.mana_cost));
                }
            }
        }
    }

    // ─── ROOM VIEW ────────────────────────────────────────────────────────────

    fn draw_room_view(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        let title_col = RGB::named(
            if self.room_event.title.contains("TRAP") || self.room_event.title.contains("CHAOS") { ORANGE }
            else if self.room_event.title.contains("TREASURE") { YELLOW }
            else if self.room_event.title.contains("SHRINE") { MAGENTA }
            else if self.room_event.title.contains("PORTAL") { (100u8,200,255) }
            else { (100u8,255,100) }
        );

        ctx.draw_box(10, 3, 100, 42, col, bg);
        ctx.print_color(12, 4, title_col, bg, &self.room_event.title);
        ctx.print_color(12, 5, dim, bg, &"─".repeat(96));

        for (i, line) in self.room_event.lines.iter().enumerate() {
            let lc = if line.starts_with('[') { yel } else if line.starts_with("  +") || line.starts_with("  -") { col } else { RGB::named(GRAY) };
            ctx.print_color(12, 7 + i as i32, lc, bg, line);
        }

        ctx.print_color(12, 43, dim, bg, "Enter = continue/accept   P = pick up item   L = learn spell   X = skip/leave");
    }

    // ─── COMBAT ───────────────────────────────────────────────────────────────

    fn draw_combat(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let red = RGB::named(RED); let grn = RGB::named(GREEN);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        let ora = RGB::named(ORANGE);

        if self.player.is_none() || self.enemy.is_none() {
            self.screen = AppScreen::FloorNav; return;
        }

        let (en, et, ehp, emhp, esp) = {
            let e = self.enemy.as_ref().unwrap();
            (e.name.clone(), e.tier.name().to_string(), e.hp, e.max_hp, e.ascii_sprite.clone())
        };
        let (pn, pc, plv, php, pmhp, pst, psp, pit) = {
            let p = self.player.as_ref().unwrap();
            (p.name.clone(), p.class.name().to_string(), p.level, p.current_hp, p.max_hp,
             p.status_badge_line(), p.known_spells.clone(), p.inventory.clone())
        };

        // Enemy panel
        ctx.draw_box(1, 1, 57, 21, red, bg);
        ctx.print_color(3, 2, red, bg, &format!("[{}]  {}", et, en));
        let ep = ehp as f32 / emhp.max(1) as f32;
        ctx.print_color(3, 3, RGB::named(hp_col(ep)), bg, &format!("HP {}/{}", ehp, emhp));
        hbar(ctx, 3, 4, 52, ehp, emhp, hp_col(ep));
        for (i, l) in esp.lines().enumerate() { ctx.print_color(22, 6 + i as i32, red, bg, l); }

        // Player panel
        ctx.draw_box(1, 23, 57, 16, col, bg);
        ctx.print_color(3, 24, yel, bg, &format!("{} Lv.{} {}",  pn, plv, pc));
        let pp = php as f32 / pmhp.max(1) as f32;
        ctx.print_color(3, 25, RGB::named(hp_col(pp)), bg, &format!("HP {}/{}", php, pmhp));
        hbar(ctx, 3, 26, 52, php, pmhp, hp_col(pp));
        ctx.print_color(3, 27, RGB::named(BLUE), bg, &format!("MP {}/{}", self.current_mana, self.max_mana()));
        hbar(ctx, 3, 28, 52, self.current_mana, self.max_mana(), BLUE);
        if !pst.is_empty() { ctx.print_color(3, 29, RGB::named(MAGENTA), bg, &format!("Status: {}", pst)); }

        // Actions
        ctx.draw_box(1, 40, 57, 9, col, bg);
        ctx.print_color(3, 41, col, bg, "ACTIONS");
        ctx.print_color(3, 42, dim, bg, "──────────────────────────────────────────────────");
        ctx.print_color(3, 43, RGB::named(WHITE), bg, "[A]ttack  [H]eal item  [D]efend  [F]lee");
        if !psp.is_empty() {
            ctx.print_color(3, 45, RGB::named(MAGENTA), bg, "Spells:");
            for (i, s) in psp.iter().take(4).enumerate() {
                ctx.print_color(3 + i as i32 * 28, 46, RGB::named(MAGENTA), bg,
                    &format!("[{}]{} {}mp", i+1, &s.name[..s.name.len().min(12)], s.mana_cost));
            }
        }

        // Combat log
        ctx.draw_box(60, 1, 58, 47, col, bg);
        ctx.print_color(62, 2, yel, bg, "COMBAT LOG");
        ctx.print_color(62, 3, dim, bg, &"─".repeat(54));
        let start = self.combat_log.len().saturating_sub(40);
        for (i, entry) in self.combat_log[start..].iter().enumerate() {
            let trunc = if entry.len() > 54 { &entry[..54] } else { entry.as_str() };
            let rc = if entry.contains("slain") || entry.contains("XP") { yel }
                else if entry.contains("BOSS")  { red }
                else if entry.contains("attack") || entry.contains("deals") { ora }
                else if entry.contains("heal") { grn }
                else if entry.contains("cast") || entry.contains("Spell") { RGB::named(MAGENTA) }
                else { RGB::named(GRAY) };
            ctx.print_color(62, 5 + i as i32, rc, bg, trunc);
        }
    }

    // ─── SHOP ─────────────────────────────────────────────────────────────────

    fn draw_shop(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let grn = RGB::named(GREEN); let dim = RGB::named(DARK_GRAY);
        let bg = RGB::named(BLACK);
        ctx.draw_box(5, 1, 110, 46, col, bg);
        ctx.print_color(45, 2, yel, bg, "$ S H O P $");

        let gold = self.player.as_ref().map(|p| p.gold).unwrap_or(0);
        ctx.print_color(5, 4, yel, bg, &format!("Your gold: {}", gold));

        ctx.print_color(5, 6, col, bg, "[H] Healing Potion");
        ctx.print_color(5, 7, dim, bg, &format!("    Restores 40 HP — {}g", self.shop_heal_cost));

        ctx.print_color(5, 9, col, bg, "ITEMS FOR SALE");
        ctx.print_color(5, 10, dim, bg, &"─".repeat(100));
        for (i, (item, price)) in self.shop_items.iter().enumerate() {
            let row = 11 + i as i32 * 4;
            let (fg, pfx) = if i == self.shop_cursor { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(5, row, fg, bg, &format!("{} [{}] {} — {}g", pfx, i+1, item.name, price));
            ctx.print_color(7, row+1, dim, bg, &format!("    Rarity: {:?}", item.rarity));
            for (j, m) in item.stat_modifiers.iter().take(3).enumerate() {
                ctx.print_color(7, row + 2 + j as i32, col, bg, &format!("    {:+} {}", m.value, m.stat));
            }
            let can_afford = gold >= *price;
            let ac = if can_afford { grn } else { dim };
            ctx.print_color(80, row, ac, bg, if can_afford { "[ENTER to buy]" } else { "[not enough gold]" });
        }

        ctx.print_color(5, 43, dim, bg, "↑↓ = select   Enter = buy   H = heal potion   Esc = leave");
    }

    // ─── CRAFTING ─────────────────────────────────────────────────────────────

    fn draw_crafting(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(5, 1, 110, 46, col, bg);
        ctx.print_color(45, 2, yel, bg, "⚒  CRAFTING BENCH  ⚒");

        let inv_len = self.player.as_ref().map(|p| p.inventory.len()).unwrap_or(0);
        if inv_len == 0 {
            ctx.print_color(10, 10, dim, bg, "Your inventory is empty — nothing to craft with.");
            ctx.print_color(10, 12, dim, bg, "Esc = leave");
            return;
        }

        ctx.print_color(5, 5, col, bg, "INVENTORY  (select an item to re-roll its stats)");
        ctx.print_color(5, 6, dim, bg, &"─".repeat(100));

        if let Some(ref p) = self.player {
            for (i, item) in p.inventory.iter().enumerate() {
                let row = 7 + i as i32 * 2;
                let (fg, pfx) = if i == self.craft_cursor { (RGB::named(WHITE), "►") } else { (dim, " ") };
                ctx.print_color(5, row, fg, bg, &format!("{} [{}] {}", pfx, i+1, item.name));
                let mods: Vec<String> = item.stat_modifiers.iter().map(|m| format!("{:+}{}", m.value, &m.stat[..3])).collect();
                ctx.print_color(7, row+1, dim, bg, &format!("    {:?}  {}", item.rarity, mods.join("  ")));
            }
        }

        if !self.craft_message.is_empty() {
            ctx.print_color(5, 42, RGB::named(GREEN), bg, &self.craft_message);
        }

        ctx.print_color(5, 44, dim, bg, "↑↓ = select item   Enter = re-roll stats   Esc = leave");
    }

    // ─── GAME OVER ────────────────────────────────────────────────────────────

    fn draw_game_over(&mut self, ctx: &mut BTerm) {
        let red = RGB::named(RED); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(15, 5, 90, 38, red, bg);
        ctx.print_color(52, 7, red, bg, "G A M E   O V E R");
        ctx.print_color(35, 9, dim, bg, "The algorithms have judged you. You were found wanting.");

        if let Some(ref p) = self.player {
            let rows: &[(&str, String)] = &[
                ("Character", format!("{} — Lv.{} {}", p.name, p.level, p.class.name())),
                ("Floor reached", p.floor.to_string()),
                ("Enemies slain", p.kills.to_string()),
                ("Gold earned", p.gold.to_string()),
                ("Total XP", p.xp.to_string()),
                ("Damage dealt", p.total_damage_dealt.to_string()),
                ("Damage taken", p.total_damage_taken.to_string()),
                ("Spells cast", p.spells_cast.to_string()),
                ("Items used", p.items_used.to_string()),
                ("Corruption", p.corruption.to_string()),
            ];
            for (i, (label, val)) in rows.iter().enumerate() {
                ctx.print_color(25, 12 + i as i32, RGB::named(GRAY), bg, &format!("{:<20} {}", label, val));
            }
            ctx.print_color(25, 24, yel, bg, &format!("FINAL SCORE:  {}", p.score()));
        }

        ctx.print_color(42, 32, RGB::named(CYAN), bg, "[ENTER] Return to title");
        ctx.print_color(40, 33, dim, bg, "[S] Save score to leaderboard first");
    }

    // ─── SCOREBOARD ───────────────────────────────────────────────────────────

    fn draw_scoreboard(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN); let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY); let bg = RGB::named(BLACK);
        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(50, 2, yel, bg, "★  HALL OF CHAOS  ★");
        ctx.print_color(3, 4, col, bg,
            &format!("{:<4} {:<20} {:<14} {:<8} {:<12} {}", "#", "Name", "Class", "Floor", "Enemies", "Score"));
        ctx.print_color(3, 5, dim, bg, &"─".repeat(100));
        let scores = load_scores();
        if scores.is_empty() {
            ctx.print_color(46, 20, dim, bg, "No scores yet. Go die heroically.");
        }
        for (i, s) in scores.iter().take(30).enumerate() {
            let rc = match i { 0 => RGB::named(YELLOW), 1 => RGB::named(WHITE), 2 => RGB::named(ORANGE), _ => RGB::named(GRAY) };
            ctx.print_color(3, 6 + i as i32, rc, bg,
                &format!("{:<4} {:<20} {:<14} {:<8} {:<12} {}", i+1, &s.name, &s.class, s.floor_reached, s.enemies_defeated, s.score));
        }
        ctx.print_color(3, 47, dim, bg, "Esc = back");
    }

    // ─── INPUT ────────────────────────────────────────────────────────────────

    fn handle_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match self.screen {

                AppScreen::Title => match key {
                    VirtualKeyCode::Up    => { if self.selected_menu > 0 { self.selected_menu -= 1; } }
                    VirtualKeyCode::Down  => { if self.selected_menu < 2 { self.selected_menu += 1; } }
                    VirtualKeyCode::Return | VirtualKeyCode::Space => match self.selected_menu {
                        0 => self.screen = AppScreen::CharacterCreation,
                        1 => self.screen = AppScreen::Scoreboard,
                        _ => ctx.quitting = true,
                    },
                    VirtualKeyCode::Q => ctx.quitting = true,
                    _ => {}
                },

                AppScreen::CharacterCreation => match key {
                    VirtualKeyCode::Escape => self.screen = AppScreen::Title,
                    VirtualKeyCode::Up     => { if self.cc_class > 0 { self.cc_class -= 1; } }
                    VirtualKeyCode::Down   => { if self.cc_class < CLASSES.len()-1 { self.cc_class += 1; } }
                    VirtualKeyCode::Left   => { if self.cc_bg > 0 { self.cc_bg -= 1; } }
                    VirtualKeyCode::Right  => { if self.cc_bg < BACKGROUNDS.len()-1 { self.cc_bg += 1; } }
                    VirtualKeyCode::Tab    => { self.cc_diff = (self.cc_diff + 1) % DIFFICULTIES.len(); }
                    VirtualKeyCode::Return | VirtualKeyCode::Space => {
                        let class  = CLASSES[self.cc_class].1.clone();
                        let bg_c   = BACKGROUNDS[self.cc_bg].1.clone();
                        let diff   = DIFFICULTIES[self.cc_diff].1.clone();
                        let player = Character::roll_new("Hero".to_string(), class, bg_c, self.seed, diff);
                        self.current_mana = (player.stats.mana + 50).max(50);
                        self.floor_num = 1;
                        let floor = generate_floor(self.floor_num, self.seed);
                        self.floor = Some(floor);
                        self.player = Some(player);
                        self.combat_log.clear();
                        self.screen = AppScreen::FloorNav;
                    }
                    _ => {}
                },

                AppScreen::FloorNav => match key {
                    VirtualKeyCode::E => {
                        self.enter_current_room();
                    }
                    VirtualKeyCode::N => {
                        // Advance to next room on this floor
                        let advanced = self.floor.as_mut().map(|f| f.advance()).unwrap_or(false);
                        if !advanced {
                            // Floor complete — move to next floor
                            self.floor_num += 1;
                            if let Some(ref mut p) = self.player { p.floor = self.floor_num; }
                            let fl = generate_floor(self.floor_num, self.seed.wrapping_add(self.floor_num as u64));
                            self.floor = Some(fl);
                        }
                    }
                    VirtualKeyCode::R => {
                        if let Some(ref mut p) = self.player {
                            let heal = (p.max_hp / 5).max(5);
                            p.heal(heal);
                            let mr = self.max_mana() / 4;
                            self.current_mana = (self.current_mana + mr).min(self.max_mana());
                        }
                    }
                    VirtualKeyCode::Period | VirtualKeyCode::RBracket => {
                        self.floor_num += 1;
                        if let Some(ref mut p) = self.player { p.floor = self.floor_num; }
                        let fl = generate_floor(self.floor_num, self.seed.wrapping_add(self.floor_num as u64));
                        self.floor = Some(fl);
                    }
                    VirtualKeyCode::Q | VirtualKeyCode::Escape => {
                        self.screen = AppScreen::Title;
                        self.player = None;
                    }
                    _ => {}
                },

                AppScreen::RoomView => {
                    // Apply immediate effects once
                    if !self.room_event.resolved {
                        self.room_event.resolved = true;
                        let gd = self.room_event.gold_delta;
                        let hd = self.room_event.hp_delta;
                        let dt = self.room_event.damage_taken;
                        let md = self.room_event.mana_delta;
                        let portal = self.room_event.portal_available;
                        let bons: Vec<(&'static str, i64)> = self.room_event.stat_bonuses.clone();
                        if let Some(ref mut p) = self.player {
                            p.gold += gd;
                            if hd > 0 { p.heal(hd); }
                            if dt > 0 { p.take_damage(dt); }
                            let _ = md; // mana handled below
                        }
                        self.current_mana = (self.current_mana + md).min(self.max_mana());
                        for (stat, val) in bons { self.apply_stat_modifier(stat, val); }
                        let _ = portal;
                        // Check death from trap
                        if self.player.as_ref().map(|p| p.current_hp <= 0).unwrap_or(false) {
                            self.screen = AppScreen::GameOver;
                            return;
                        }
                    }

                    match key {
                        VirtualKeyCode::P => {
                            if let Some(item) = self.room_event.pending_item.take() {
                                let name = item.name.clone();
                                let mods: Vec<_> = item.stat_modifiers.iter().map(|m| (m.stat.clone(), m.value)).collect();
                                for (stat, val) in &mods { self.apply_stat_modifier(stat, *val); }
                                if let Some(ref mut p) = self.player {
                                    p.add_item(item);
                                }
                                self.push_log(format!("Picked up {}", name));
                            }
                            if self.room_event.pending_spell.is_none() {
                                self.advance_floor_room();
                                self.screen = AppScreen::FloorNav;
                            }
                        }
                        VirtualKeyCode::L => {
                            if let Some(spell) = self.room_event.pending_spell.take() {
                                let name = spell.name.clone();
                                if let Some(ref mut p) = self.player { p.add_spell(spell); }
                                self.push_log(format!("Learned spell: {}", name));
                            }
                            if self.room_event.pending_item.is_none() {
                                self.advance_floor_room();
                                self.screen = AppScreen::FloorNav;
                            }
                        }
                        VirtualKeyCode::Return | VirtualKeyCode::Space | VirtualKeyCode::X => {
                            // Skip/leave or accept (if no pending items)
                            if self.room_event.portal_available {
                                // Portal: advance floor
                                self.floor_num += 1;
                                if let Some(ref mut p) = self.player { p.floor = self.floor_num; }
                                let fl = generate_floor(self.floor_num, self.seed.wrapping_add(self.floor_num as u64));
                                self.floor = Some(fl);
                            } else {
                                self.advance_floor_room();
                            }
                            self.room_event = RoomEvent::empty();
                            self.screen = AppScreen::FloorNav;
                        }
                        _ => {}
                    }
                }

                AppScreen::Combat => {
                    if self.player.is_none() || self.enemy.is_none() { return; }

                    match key {
                        VirtualKeyCode::A => {
                            let force = self.player.as_ref().unwrap().stats.force;
                            let roll = chaos_roll_verbose(force as f64 * 0.01, self.seed.wrapping_add(self.frame));
                            let dmg = ((force as f64 * (1.5 + roll.final_value.abs())) as i64).max(1);
                            let en = self.enemy.as_ref().unwrap().name.clone();
                            if let Some(ref mut e) = self.enemy { e.hp -= dmg; }
                            if let Some(ref mut p) = self.player { p.total_damage_dealt += dmg; }
                            self.push_log(format!("You attack {} for {} damage!", en, dmg));
                            if !self.check_combat_end() { self.enemy_counter_attack(); self.check_combat_end(); }
                        }
                        VirtualKeyCode::H => {
                            let idx = self.player.as_ref().and_then(|p| p.inventory.iter().position(|i| {
                                i.base_type.contains("Potion") || i.name.to_lowercase().contains("potion") || i.name.to_lowercase().contains("heal")
                            }));
                            if let Some(idx) = idx {
                                if let Some(ref mut p) = self.player {
                                    if let Some(item) = p.use_item(idx) {
                                        let base = (item.damage_or_defense.abs() + 20).max(20);
                                        let heal = p.item_heal_bonus(base);
                                        p.heal(heal); p.items_used += 1;
                                        self.push_log(format!("Used {} — healed {} HP", item.name, heal));
                                    }
                                }
                                self.enemy_counter_attack(); self.check_combat_end();
                            } else { self.push_log("No healing items!".to_string()); }
                        }
                        VirtualKeyCode::D => {
                            if let Some(ref mut p) = self.player {
                                let shield = (p.stats.vitality / 4 + 5).max(1);
                                p.add_status(StatusEffect::Shielded(shield));
                                self.push_log(format!("You brace! +{} shield", shield));
                            }
                            self.enemy_counter_attack(); self.check_combat_end();
                        }
                        k @ (VirtualKeyCode::Key1|VirtualKeyCode::Key2|VirtualKeyCode::Key3
                           |VirtualKeyCode::Key4|VirtualKeyCode::Key5|VirtualKeyCode::Key6
                           |VirtualKeyCode::Key7|VirtualKeyCode::Key8) => {
                            let idx = match k {
                                VirtualKeyCode::Key1=>0,VirtualKeyCode::Key2=>1,VirtualKeyCode::Key3=>2,
                                VirtualKeyCode::Key4=>3,VirtualKeyCode::Key5=>4,VirtualKeyCode::Key6=>5,
                                VirtualKeyCode::Key7=>6,_=>7,
                            };
                            let info = self.player.as_ref().and_then(|p| p.known_spells.get(idx).map(|s|
                                (s.name.clone(), s.mana_cost, s.scaling_stat.clone(), s.scaling_factor)
                            ));
                            if let Some((sname, cost, scaling, factor)) = info {
                                if self.current_mana >= cost {
                                    self.current_mana -= cost;
                                    let sv = self.player.as_ref().map(|p| match scaling.as_str() {
                                        "force"=>"p.stats.force", "cunning"=>"p.stats.cunning",
                                        "entropy"=>"p.stats.entropy", "precision"=>"p.stats.precision",
                                        _=>"p.stats.mana",
                                    }).unwrap_or("p.stats.mana");
                                    let stat_val = self.player.as_ref().map(|p| match sv {
                                        "p.stats.force"     => p.stats.force,
                                        "p.stats.cunning"   => p.stats.cunning,
                                        "p.stats.entropy"   => p.stats.entropy,
                                        "p.stats.precision" => p.stats.precision,
                                        _                   => p.stats.mana,
                                    }).unwrap_or(10);
                                    let roll = chaos_roll_verbose(stat_val as f64 * 0.02, self.seed.wrapping_add(self.frame+100));
                                    let dmg = ((stat_val as f64 * factor.abs() * (1.0 + roll.final_value.abs())) as i64).max(1);
                                    let en = self.enemy.as_ref().unwrap().name.clone();
                                    if let Some(ref mut e) = self.enemy { e.hp -= dmg; }
                                    if let Some(ref mut p) = self.player { p.total_damage_dealt += dmg; p.spells_cast += 1; }
                                    self.push_log(format!("Cast {} on {} for {} damage!", sname, en, dmg));
                                    if !self.check_combat_end() { self.enemy_counter_attack(); self.check_combat_end(); }
                                } else { self.push_log(format!("Not enough mana! ({} needed)", cost)); }
                            } else { self.push_log("No spell in that slot.".to_string()); }
                        }
                        VirtualKeyCode::F | VirtualKeyCode::Escape => {
                            let luck = self.player.as_ref().map(|p| p.flee_luck_modifier()).unwrap_or(0);
                            let roll = chaos_roll_verbose(luck as f64 * 0.1, self.seed.wrapping_add(self.frame+99));
                            if roll.final_value > -0.2 {
                                self.push_log("You flee!".to_string());
                                self.enemy = None;
                                self.advance_floor_room();
                                self.screen = AppScreen::FloorNav;
                            } else {
                                self.push_log("Flee failed! Enemy blocks your escape.".to_string());
                                self.enemy_counter_attack(); self.check_combat_end();
                            }
                        }
                        _ => {}
                    }
                }

                AppScreen::Shop => match key {
                    VirtualKeyCode::Escape => { self.advance_floor_room(); self.screen = AppScreen::FloorNav; }
                    VirtualKeyCode::Up     => { if self.shop_cursor > 0 { self.shop_cursor -= 1; } }
                    VirtualKeyCode::Down   => { if self.shop_cursor < self.shop_items.len().saturating_sub(1) { self.shop_cursor += 1; } }
                    VirtualKeyCode::H => {
                        let cost = self.shop_heal_cost;
                        if let Some(ref mut p) = self.player {
                            if p.gold >= cost { p.gold -= cost; p.heal_scaled(40); }
                        }
                    }
                    VirtualKeyCode::Return | VirtualKeyCode::Space => {
                        if self.shop_cursor < self.shop_items.len() {
                            let price = self.shop_items[self.shop_cursor].1;
                            if self.player.as_ref().map(|p| p.gold >= price).unwrap_or(false) {
                                if let Some(ref mut p) = self.player { p.gold -= price; }
                                let (item, _) = self.shop_items.remove(self.shop_cursor);
                                let name = item.name.clone();
                                if self.shop_cursor >= self.shop_items.len() && self.shop_cursor > 0 {
                                    self.shop_cursor -= 1;
                                }
                                if let Some(ref mut p) = self.player { p.add_item(item); }
                                self.push_log(format!("Bought {}", name));
                            }
                        }
                    }
                    _ => {}
                },

                AppScreen::Crafting => match key {
                    VirtualKeyCode::Escape => { self.advance_floor_room(); self.screen = AppScreen::FloorNav; }
                    VirtualKeyCode::Up     => { if self.craft_cursor > 0 { self.craft_cursor -= 1; } }
                    VirtualKeyCode::Down   => {
                        let len = self.player.as_ref().map(|p| p.inventory.len()).unwrap_or(0);
                        if self.craft_cursor + 1 < len { self.craft_cursor += 1; }
                    }
                    VirtualKeyCode::Return | VirtualKeyCode::Space => {
                        // Re-roll the selected item's stat modifiers via chaos pipeline
                        let idx = self.craft_cursor;
                        let len = self.player.as_ref().map(|p| p.inventory.len()).unwrap_or(0);
                        if idx < len {
                            let new_seed = self.seed.wrapping_add(self.frame).wrapping_mul(6364136223846793005);
                            let new_item = Item::generate(new_seed);
                            let name = new_item.name.clone();
                            if let Some(ref mut p) = self.player {
                                p.inventory[idx] = new_item;
                            }
                            self.craft_message = format!("Re-rolled! New item: {}", name);
                        }
                    }
                    _ => {}
                },

                AppScreen::GameOver => match key {
                    VirtualKeyCode::Return | VirtualKeyCode::Escape => {
                        self.screen = AppScreen::Title;
                        self.player = None; self.enemy = None;
                    }
                    VirtualKeyCode::S => {
                        if let Some(ref p) = self.player {
                            let entry = ScoreEntry::new(p.name.clone(), format!("{:?}", p.class),
                                p.score(), p.floor, p.kills, 0u32);
                            save_score(entry);
                        }
                        self.screen = AppScreen::Title;
                        self.player = None; self.enemy = None;
                    }
                    _ => {}
                },

                AppScreen::Scoreboard => match key {
                    VirtualKeyCode::Escape | VirtualKeyCode::Q => self.screen = AppScreen::Title,
                    _ => {}
                },
            }
        }
    }

    fn advance_floor_room(&mut self) {
        let at_end = self.floor.as_ref().map(|f| f.current_room + 1 >= f.rooms.len()).unwrap_or(true);
        if at_end {
            self.floor_num += 1;
            if let Some(ref mut p) = self.player { p.floor = self.floor_num; }
            let fl = generate_floor(self.floor_num, self.seed.wrapping_add(self.floor_num as u64));
            self.floor = Some(fl);
        } else {
            self.floor.as_mut().map(|f| f.advance());
        }
    }
}

// ─── ENTRY POINT ─────────────────────────────────────────────────────────────

fn main() -> BError {
    let fullscreen = std::env::args().any(|a| a == "--fullscreen" || a == "-f");
    let mut builder = BTermBuilder::simple80x50()
        .with_title("CHAOS RPG — Where Math Goes To Die")
        .with_tile_dimensions(14, 14)
        .with_dimensions(120, 50)
        .with_fps_cap(60.0);
    if fullscreen { builder = builder.with_fullscreen(true); }
    main_loop(builder.build()?, State::new())
}
