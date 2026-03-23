//! CHAOS RPG — Graphical Frontend (bracket-lib)
//!
//! CP437-tileset OpenGL window. All game logic from chaos-rpg-core.
//!
//! Controls: arrow keys / WASD navigate, Enter/Space confirm, Esc back.
//! Launch with --fullscreen (or -f) for true fullscreen.

use bracket_lib::prelude::*;
use chaos_rpg_core::{
    character::{Background, Character, CharacterClass, ColorTheme, Difficulty},
    chaos_pipeline::chaos_roll_verbose,
    enemy::{generate_enemy, Enemy},
    scoreboard::load_scores,
    world::{generate_floor, Floor},
};

mod renderer;
mod sprites;
mod ui_overlay;

// ─── SCREEN STATES ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
    Title,
    CharacterCreation,
    FloorNav,
    Combat,
    GameOver,
    Scoreboard,
}

// ─── APP STATE ────────────────────────────────────────────────────────────────

struct State {
    screen: AppScreen,
    player: Option<Character>,
    floor: Option<Floor>,
    enemy: Option<Enemy>,
    combat_log: Vec<String>,
    seed: u64,
    frame: u64,
    selected_menu: usize,      // title menu cursor
    cc_class: usize,           // char creation: class index
    cc_bg: usize,              // char creation: background index
    cc_diff: usize,            // char creation: difficulty index
    fullscreen: bool,
    current_mana: i64,         // tracked separately since StatBlock has no current_mana
    floor_num: u32,
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
            player: None,
            floor: None,
            enemy: None,
            combat_log: Vec::new(),
            seed,
            frame: 0,
            selected_menu: 0,
            cc_class: 0,
            cc_bg: 0,
            cc_diff: 0,
            fullscreen,
            current_mana: 0,
            floor_num: 1,
        }
    }

    fn max_mana(&self) -> i64 {
        self.player.as_ref().map(|p| (p.stats.mana + 50).max(50)).unwrap_or(50)
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.combat_log.push(msg.into());
        if self.combat_log.len() > 200 { self.combat_log.remove(0); }
    }

    fn enemy_attack_player(&mut self) {
        if let (Some(ref mut player), Some(ref enemy)) = (&mut self.player, &self.enemy) {
            if enemy.hp <= 0 { return; }
            let roll = chaos_roll_verbose(enemy.chaos_level, self.seed.wrapping_add(self.frame + 7));
            let dmg = ((enemy.base_damage as f64 * (1.0 + roll.final_value.abs() * 0.3)) as i64).max(1);
            player.take_damage(dmg);
            self.combat_log.push(format!("{} attacks for {}!", enemy.name, dmg));
        }
    }

    fn check_combat_end(&mut self) -> bool {
        let enemy_dead = self.enemy.as_ref().map(|e| e.hp <= 0).unwrap_or(false);
        let player_dead = self.player.as_ref().map(|p| p.current_hp <= 0).unwrap_or(false);

        if enemy_dead {
            if let (Some(ref mut player), Some(ref enemy)) = (&mut self.player, &self.enemy) {
                player.kills += 1;
                let xp = enemy.xp_reward;
                player.gain_xp(xp);
                player.gold += enemy.gold_reward;
                self.combat_log.push(format!("{} slain! +{} XP  +{} gold", enemy.name, xp, enemy.gold_reward));
            }
            self.enemy = None;
            self.screen = AppScreen::FloorNav;
            return true;
        }
        if player_dead {
            self.screen = AppScreen::GameOver;
            return true;
        }
        false
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
    ("Scholar",  Background::Scholar),
    ("Wanderer", Background::Wanderer),
    ("Gladiator",Background::Gladiator),
    ("Outcast",  Background::Outcast),
];

const DIFFICULTIES: &[(&str, Difficulty)] = &[
    ("Easy",   Difficulty::Easy),
    ("Normal", Difficulty::Normal),
    ("Brutal", Difficulty::Brutal),
    ("Chaos",  Difficulty::Chaos),
];

// ─── GameState IMPL ───────────────────────────────────────────────────────────

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.frame += 1;

        // Tick status effects each frame (roughly every 60 frames = 1 second)
        if self.frame % 60 == 0 {
            if let Some(ref mut player) = self.player {
                let (net, msgs) = player.tick_status_effects();
                if net != 0 {
                    for m in msgs { self.combat_log.push(m); }
                }
            }
        }

        match self.screen {
            AppScreen::Title            => self.draw_title(ctx),
            AppScreen::CharacterCreation => self.draw_char_creation(ctx),
            AppScreen::FloorNav         => self.draw_floor_nav(ctx),
            AppScreen::Combat           => self.draw_combat(ctx),
            AppScreen::GameOver         => self.draw_game_over(ctx),
            AppScreen::Scoreboard       => self.draw_scoreboard(ctx),
        }

        self.handle_input(ctx);
    }
}

// ─── DRAW HELPERS ─────────────────────────────────────────────────────────────

fn hbar(ctx: &mut BTerm, x: i32, y: i32, w: i32, current: i64, max: i64, full_col: (u8,u8,u8)) {
    let filled = if max > 0 { ((current * w as i64) / max.max(1)).clamp(0, w as i64) as i32 } else { 0 };
    for i in 0..w {
        let ch = if i < filled { 219u16 } else { 176u16 };
        let col = if i < filled { RGB::named(full_col) } else { RGB::named(DARK_GRAY) };
        ctx.set(x + i, y, col, RGB::named(BLACK), ch);
    }
}

fn hp_color(pct: f32) -> (u8,u8,u8) {
    if pct > 0.6 { GREEN } else if pct > 0.3 { YELLOW } else { RED }
}

// ─── TITLE ────────────────────────────────────────────────────────────────────

impl State {
    fn draw_title(&mut self, ctx: &mut BTerm) {
        let col  = RGB::named(CYAN);
        let dim  = RGB::named(DARK_GRAY);
        let yel  = RGB::named(YELLOW);
        let bg   = RGB::named(BLACK);

        ctx.draw_box(1, 1, 118, 48, col, bg);

        ctx.print_color(30, 4,  yel, bg, "  ██████╗██╗  ██╗ █████╗  ██████╗ ███████╗");
        ctx.print_color(30, 5,  yel, bg, " ██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝");
        ctx.print_color(30, 6,  col, bg, " ██║     ███████║███████║██║   ██║███████╗ ");
        ctx.print_color(30, 7,  col, bg, " ╚██████╗██║  ██║██║  ██║╚██████╔╝███████║");
        ctx.print_color(30, 8,  col, bg, "  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝");
        ctx.print_color(42, 10, dim, bg, "R P G   —   Where Math Goes To Die");

        let options = [
            "  New Game",
            "  Infinite Mode",
            "  Daily Seed Challenge",
            "  Scoreboard",
            "  Quit",
        ];
        let ox = 50i32;
        let oy = 18i32;
        ctx.draw_box(ox - 2, oy - 2, 32, options.len() as i32 + 3, col, bg);
        for (i, opt) in options.iter().enumerate() {
            let (fg, pfx) = if i == self.selected_menu {
                (RGB::named(WHITE), "►")
            } else {
                (dim, " ")
            };
            ctx.print_color(ox, oy + i as i32, fg, bg, &format!("{} {}", pfx, opt));
        }

        ctx.print_color(4, 46, dim, bg, "↑↓ Navigate   Enter Select   Q Quit");
        let fs_hint = if self.fullscreen { "[ fullscreen ]" } else { "[ --fullscreen for fullscreen ]" };
        ctx.print_color(4, 47, dim, bg, fs_hint);
    }

    // ─── CHARACTER CREATION ───────────────────────────────────────────────────

    fn draw_char_creation(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY);
        let bg  = RGB::named(BLACK);

        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(45, 2, yel, bg, "─── CHARACTER CREATION ───");

        // Class column
        ctx.print_color(3, 5, col, bg, "CLASS");
        ctx.print_color(3, 6, dim, bg, "─────────────────");
        for (i, (name, _)) in CLASSES.iter().enumerate() {
            let (fg, pfx) = if i == self.cc_class { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(3, 7 + i as i32, fg, bg, &format!("{} {}", pfx, name));
        }

        // Class description
        let class = &CLASSES[self.cc_class].1;
        let passive_name = class.passive_name();
        let passive_desc = class.passive_desc();
        ctx.print_color(3, 18, col, bg, "PASSIVE ABILITY");
        ctx.print_color(3, 19, yel, bg, passive_name);
        // word-wrap desc across up to 3 lines of 35 chars
        let words: Vec<&str> = passive_desc.split_whitespace().collect();
        let mut line = String::new();
        let mut row = 20i32;
        for w in words {
            if line.len() + w.len() + 1 > 35 { ctx.print_color(3, row, dim, bg, &line); line = w.to_string(); row += 1; }
            else { if !line.is_empty() { line.push(' '); } line.push_str(w); }
        }
        if !line.is_empty() { ctx.print_color(3, row, dim, bg, &line); }

        // Background column
        ctx.print_color(28, 5, col, bg, "BACKGROUND");
        ctx.print_color(28, 6, dim, bg, "─────────────────");
        for (i, (name, _)) in BACKGROUNDS.iter().enumerate() {
            let (fg, pfx) = if i == self.cc_bg { (RGB::named(WHITE), "►") } else { (dim, " ") };
            ctx.print_color(28, 7 + i as i32, fg, bg, &format!("{} {}", pfx, name));
        }

        // Difficulty column
        ctx.print_color(50, 5, col, bg, "DIFFICULTY");
        ctx.print_color(50, 6, dim, bg, "─────────────────");
        for (i, (name, _)) in DIFFICULTIES.iter().enumerate() {
            let color = match i { 0 => RGB::named(GREEN), 1 => RGB::named(YELLOW), _ => RGB::named(RED) };
            let pfx = if i == self.cc_diff { "►" } else { " " };
            ctx.print_color(50, 7 + i as i32, color, bg, &format!("{} {}", pfx, name));
        }

        // ASCII portrait
        let portrait = CLASSES[self.cc_class].1.ascii_art();
        ctx.print_color(90, 5, col, bg, "PREVIEW");
        ctx.print_color(90, 6, dim, bg, "─────────────");
        for (i, line) in portrait.lines().enumerate() {
            ctx.print_color(90, 7 + i as i32, RGB::named(WHITE), bg, line);
        }

        ctx.print_color(3, 46, col, bg, "[ ENTER ] Start Adventure");
        ctx.print_color(3, 47, dim, bg, "Tab = switch column   ↑↓ = select   Esc = back");
    }

    // ─── FLOOR NAV ────────────────────────────────────────────────────────────

    fn draw_floor_nav(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY);
        let bg  = RGB::named(BLACK);

        let (name, class_name, level, floor, kills, gold, xp, current_hp, max_hp, statuses) = match &self.player {
            Some(p) => (
                p.name.clone(),
                p.class.name().to_string(),
                p.level,
                p.floor,
                p.kills,
                p.gold,
                p.xp,
                p.current_hp,
                p.max_hp,
                p.status_badge_line(),
            ),
            None => { self.screen = AppScreen::Title; return; }
        };

        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(3, 2, yel, bg, &format!("FLOOR {}  —  {}", floor, name));
        ctx.print_color(3, 3, dim, bg, &format!("Lv.{}  {}  Kills: {}  Gold: {}  XP: {}", level, class_name, kills, gold, xp));

        let hp_pct = current_hp as f32 / max_hp.max(1) as f32;
        ctx.print_color(3, 5, RGB::named(hp_color(hp_pct)), bg, &format!("HP: {}/{}", current_hp, max_hp));
        hbar(ctx, 3, 6, 40, current_hp, max_hp, hp_color(hp_pct));
        ctx.print_color(3, 7, RGB::named(BLUE), bg, &format!("MP: {}/{}", self.current_mana, self.max_mana()));
        hbar(ctx, 3, 8, 40, self.current_mana, self.max_mana(), BLUE);

        if !statuses.is_empty() {
            ctx.print_color(3, 10, RGB::named(MAGENTA), bg, &format!("Status: {}", statuses));
        }

        ctx.print_color(3, 35, col, bg, "ACTIONS");
        ctx.print_color(3, 36, dim, bg, "─────────────────────────────────────");
        ctx.print_color(3, 37, RGB::named(WHITE), bg, "[E] Enter next room    [R] Rest (+HP)");
        ctx.print_color(3, 38, RGB::named(WHITE), bg, "[>] Next floor         [I] Inventory");
        ctx.print_color(3, 39, RGB::named(WHITE), bg, "[S] Character sheet    [Q] Quit to menu");

        // Inventory preview
        if let Some(ref p) = self.player {
            ctx.print_color(60, 5, col, bg, "INVENTORY");
            ctx.print_color(60, 6, dim, bg, "─────────────────────────────────────");
            if p.inventory.is_empty() {
                ctx.print_color(60, 7, dim, bg, "(empty)");
            } else {
                for (i, item) in p.inventory.iter().take(15).enumerate() {
                    ctx.print_color(60, 7 + i as i32, RGB::named(GRAY), bg, &format!("[{}] {}", i + 1, item.name));
                }
            }
            // Known spells
            ctx.print_color(60, 24, col, bg, "SPELLS");
            ctx.print_color(60, 25, dim, bg, "─────────────────────────────────────");
            if p.known_spells.is_empty() {
                ctx.print_color(60, 26, dim, bg, "(none)");
            } else {
                for (i, spell) in p.known_spells.iter().take(8).enumerate() {
                    ctx.print_color(60, 26 + i as i32, RGB::named(MAGENTA), bg,
                        &format!("[{}] {} ({}mp)", i + 1, spell.name, spell.mana_cost));
                }
            }
        }
    }

    // ─── COMBAT ───────────────────────────────────────────────────────────────

    fn draw_combat(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let yel = RGB::named(YELLOW);
        let red = RGB::named(RED);
        let grn = RGB::named(GREEN);
        let dim = RGB::named(DARK_GRAY);
        let bg  = RGB::named(BLACK);

        if self.player.is_none() || self.enemy.is_none() {
            self.screen = AppScreen::FloorNav;
            return;
        }

        let (e_name, e_tier, e_hp, e_max_hp, e_sprite) = {
            let e = self.enemy.as_ref().unwrap();
            (e.name.clone(), e.tier.name().to_string(), e.hp, e.max_hp, e.ascii_sprite.clone())
        };
        let (p_name, p_class, p_level, p_hp, p_max_hp, p_kills, p_statuses, p_spells, p_items) = {
            let p = self.player.as_ref().unwrap();
            (
                p.name.clone(),
                p.class.name().to_string(),
                p.level,
                p.current_hp,
                p.max_hp,
                p.kills,
                p.status_badge_line(),
                p.known_spells.clone(),
                p.inventory.clone(),
            )
        };

        // ── Enemy panel (left top) ──
        ctx.draw_box(1, 1, 55, 20, red, bg);
        ctx.print_color(3, 2, red, bg, &format!("[{}]  {}", e_tier, e_name));
        let ep = e_hp as f32 / e_max_hp.max(1) as f32;
        ctx.print_color(3, 3, RGB::named(hp_color(ep)), bg, &format!("HP {}/{}", e_hp, e_max_hp));
        hbar(ctx, 3, 4, 50, e_hp, e_max_hp, hp_color(ep));
        // sprite
        for (i, line) in e_sprite.lines().enumerate() {
            ctx.print_color(22, 6 + i as i32, red, bg, line);
        }

        // ── Player panel (left middle) ──
        ctx.draw_box(1, 22, 55, 14, col, bg);
        ctx.print_color(3, 23, yel, bg, &format!("{} — Lv.{} {}  Kills:{}", p_name, p_level, p_class, p_kills));
        let pp = p_hp as f32 / p_max_hp.max(1) as f32;
        ctx.print_color(3, 24, RGB::named(hp_color(pp)), bg, &format!("HP {}/{}", p_hp, p_max_hp));
        hbar(ctx, 3, 25, 50, p_hp, p_max_hp, hp_color(pp));
        ctx.print_color(3, 26, RGB::named(BLUE), bg, &format!("MP {}/{}", self.current_mana, self.max_mana()));
        hbar(ctx, 3, 27, 50, self.current_mana, self.max_mana(), BLUE);
        if !p_statuses.is_empty() {
            ctx.print_color(3, 28, RGB::named(MAGENTA), bg, &format!("Status: {}", p_statuses));
        }

        // ── Actions panel (left bottom) ──
        ctx.draw_box(1, 37, 55, 11, col, bg);
        ctx.print_color(3, 38, col, bg, "ACTIONS");
        ctx.print_color(3, 39, dim, bg, "─────────────────────────────────────────────");
        ctx.print_color(3, 40, RGB::named(WHITE), bg, "[A]ttack  [H]eal item  [D]efend  [F]lee");

        // Spell list
        if !p_spells.is_empty() {
            ctx.print_color(3, 42, RGB::named(MAGENTA), bg, "Spells:");
            for (i, spell) in p_spells.iter().take(4).enumerate() {
                ctx.print_color(3 + i as i32 * 28, 43, RGB::named(MAGENTA), bg,
                    &format!("[{}]{} ({}mp)", i + 1, spell.name, spell.mana_cost));
            }
        }

        // Item list
        if !p_items.is_empty() {
            ctx.print_color(3, 45, RGB::named(GREEN), bg, "Items:");
            for (i, item) in p_items.iter().take(4).enumerate() {
                ctx.print_color(3 + i as i32 * 28, 46, grn, bg, &format!("[i{}]{}", i + 1, item.name));
            }
        }

        // ── Combat log (right panel) ──
        ctx.draw_box(58, 1, 60, 47, col, bg);
        ctx.print_color(60, 2, yel, bg, "COMBAT LOG");
        ctx.print_color(60, 3, dim, bg, "─────────────────────────────────────────────────");
        let start = self.combat_log.len().saturating_sub(40);
        for (i, entry) in self.combat_log[start..].iter().enumerate() {
            let truncated = if entry.len() > 56 { &entry[..56] } else { entry.as_str() };
            let row_col = if entry.contains("slain") || entry.contains("XP") { yel }
                else if entry.contains("attack") || entry.contains("deals") { RGB::named(ORANGE) }
                else if entry.contains("heal") || entry.contains("HP") { grn }
                else if entry.contains("cast") || entry.contains("spell") || entry.contains("Spell") { RGB::named(MAGENTA) }
                else { RGB::named(GRAY) };
            ctx.print_color(60, 5 + i as i32, row_col, bg, truncated);
        }
    }

    // ─── GAME OVER ────────────────────────────────────────────────────────────

    fn draw_game_over(&mut self, ctx: &mut BTerm) {
        let red = RGB::named(RED);
        let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY);
        let bg  = RGB::named(BLACK);

        ctx.draw_box(20, 8, 80, 32, red, bg);
        ctx.print_color(52, 10, red, bg, "G A M E   O V E R");
        ctx.print_color(38, 12, dim, bg, "The algorithms have judged you. You were found wanting.");

        if let Some(ref p) = self.player {
            ctx.print_color(30, 15, yel, bg, &format!("Character: {} — Lv.{} {}", p.name, p.level, p.class.name()));
            ctx.print_color(30, 16, RGB::named(GRAY), bg, &format!("Floor reached:  {}", p.floor));
            ctx.print_color(30, 17, RGB::named(GRAY), bg, &format!("Enemies slain:  {}", p.kills));
            ctx.print_color(30, 18, RGB::named(GRAY), bg, &format!("Gold earned:    {}", p.gold));
            ctx.print_color(30, 19, RGB::named(GRAY), bg, &format!("Total XP:       {}", p.xp));
            ctx.print_color(30, 20, RGB::named(GRAY), bg, &format!("Damage dealt:   {}", p.total_damage_dealt));
            ctx.print_color(30, 21, RGB::named(GRAY), bg, &format!("Damage taken:   {}", p.total_damage_taken));
            ctx.print_color(30, 22, RGB::named(GRAY), bg, &format!("Spells cast:    {}", p.spells_cast));
            ctx.print_color(30, 23, RGB::named(GRAY), bg, &format!("Items used:     {}", p.items_used));
            ctx.print_color(30, 24, RGB::named(GRAY), bg, &format!("Corruption:     {}", p.corruption));
            let score = p.score();
            ctx.print_color(30, 26, yel, bg, &format!("FINAL SCORE:  {}", score));
        }

        ctx.print_color(48, 36, RGB::named(CYAN), bg, "[ENTER] Return to title");
        ctx.print_color(45, 37, dim, bg, "[S] Save score to leaderboard");
    }

    // ─── SCOREBOARD ───────────────────────────────────────────────────────────

    fn draw_scoreboard(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let yel = RGB::named(YELLOW);
        let dim = RGB::named(DARK_GRAY);
        let bg  = RGB::named(BLACK);

        ctx.draw_box(1, 1, 118, 48, col, bg);
        ctx.print_color(50, 2, yel, bg, "★  HALL OF CHAOS  ★");

        let scores = load_scores();
        ctx.print_color(3, 4, col, bg,  &format!("{:<4} {:<20} {:<14} {:<8} {:<10} {}", "#", "Name", "Class", "Floor", "Enemies", "Score"));
        ctx.print_color(3, 5, dim, bg, &"─".repeat(100));

        for (i, s) in scores.iter().take(30).enumerate() {
            let row_col = match i { 0 => RGB::named(YELLOW), 1 => RGB::named(WHITE), 2 => RGB::named(ORANGE), _ => RGB::named(GRAY) };
            ctx.print_color(3, 6 + i as i32, row_col, bg,
                &format!("{:<4} {:<20} {:<14} {:<8} {:<10} {}", i + 1, &s.name, &s.class, s.floor_reached, s.enemies_defeated, s.score));
        }
        if scores.is_empty() {
            ctx.print_color(48, 20, dim, bg, "No scores yet. Go die heroically.");
        }

        ctx.print_color(3, 47, dim, bg, "ESC = Back to title");
    }

    // ─── INPUT ────────────────────────────────────────────────────────────────

    fn handle_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match self.screen {

                AppScreen::Title => match key {
                    VirtualKeyCode::Up    => { if self.selected_menu > 0 { self.selected_menu -= 1; } }
                    VirtualKeyCode::Down  => { if self.selected_menu < 4 { self.selected_menu += 1; } }
                    VirtualKeyCode::Return | VirtualKeyCode::Space => match self.selected_menu {
                        0 | 1 | 2 => self.screen = AppScreen::CharacterCreation,
                        3         => self.screen = AppScreen::Scoreboard,
                        _         => ctx.quitting = true,
                    },
                    VirtualKeyCode::Q => ctx.quitting = true,
                    _ => {}
                },

                AppScreen::CharacterCreation => {
                    // Tab cycles through which column we're editing
                    match key {
                        VirtualKeyCode::Escape => self.screen = AppScreen::Title,
                        VirtualKeyCode::Up => {
                            // cycle whichever column the player last changed - simplification: all arrows control class
                            if self.cc_class > 0 { self.cc_class -= 1; }
                        }
                        VirtualKeyCode::Down => {
                            if self.cc_class < CLASSES.len() - 1 { self.cc_class += 1; }
                        }
                        VirtualKeyCode::Left => {
                            if self.cc_bg > 0 { self.cc_bg -= 1; }
                        }
                        VirtualKeyCode::Right => {
                            if self.cc_bg < BACKGROUNDS.len() - 1 { self.cc_bg += 1; }
                        }
                        VirtualKeyCode::Tab => {
                            self.cc_diff = (self.cc_diff + 1) % DIFFICULTIES.len();
                        }
                        VirtualKeyCode::Numpad1 | VirtualKeyCode::Numpad2
                        | VirtualKeyCode::Numpad3 | VirtualKeyCode::Numpad4 => {
                            self.cc_diff = match key {
                                VirtualKeyCode::Numpad1 => 0, VirtualKeyCode::Numpad2 => 1,
                                VirtualKeyCode::Numpad3 => 2, _ => 3,
                            };
                        }
                        VirtualKeyCode::Return | VirtualKeyCode::Space => {
                            let class = CLASSES[self.cc_class].1.clone();
                            let bg_choice = BACKGROUNDS[self.cc_bg].1.clone();
                            let diff = DIFFICULTIES[self.cc_diff].1.clone();
                            let seed = self.seed;
                            let player = Character::roll_new("Hero".to_string(), class, bg_choice, seed, diff);
                            self.current_mana = (player.stats.mana + 50).max(50);
                            self.floor_num = 1;
                            let floor = generate_floor(self.floor_num, seed);
                            self.floor = Some(floor);
                            self.player = Some(player);
                            self.combat_log.clear();
                            self.screen = AppScreen::FloorNav;
                        }
                        _ => {}
                    }
                }

                AppScreen::FloorNav => match key {
                    VirtualKeyCode::E => {
                        // Enter a combat room
                        let seed = self.seed.wrapping_add(self.frame);
                        let floor_n = self.floor_num;
                        let enemy = generate_enemy(floor_n.max(1), seed);
                        self.enemy = Some(enemy);
                        self.combat_log.clear();
                        self.screen = AppScreen::Combat;
                    }
                    VirtualKeyCode::R => {
                        // Rest — heal 20% max HP
                        if let Some(ref mut p) = self.player {
                            let heal = (p.max_hp / 5).max(5);
                            p.heal(heal);
                            // Restore some mana too
                            let mana_restore = self.max_mana() / 4;
                            self.current_mana = (self.current_mana + mana_restore).min(self.max_mana());
                        }
                    }
                    VirtualKeyCode::Period | VirtualKeyCode::RBracket => {
                        // Next floor
                        self.floor_num += 1;
                        if let Some(ref mut p) = self.player { p.floor = self.floor_num; }
                        let floor = generate_floor(self.floor_num, self.seed.wrapping_add(self.floor_num as u64));
                        self.floor = Some(floor);
                    }
                    VirtualKeyCode::Q | VirtualKeyCode::Escape => {
                        self.screen = AppScreen::Title;
                        self.player = None;
                    }
                    _ => {}
                },

                AppScreen::Combat => {
                    if self.player.is_none() || self.enemy.is_none() { return; }

                    match key {
                        // ── Basic attack ──────────────────────────────────────
                        VirtualKeyCode::A => {
                            let force = self.player.as_ref().unwrap().stats.force;
                            let roll = chaos_roll_verbose(force as f64 * 0.01, self.seed.wrapping_add(self.frame));
                            let dmg = ((force as f64 * (1.5 + roll.final_value.abs())) as i64).max(1);
                            let e_name = self.enemy.as_ref().unwrap().name.clone();
                            if let Some(ref mut e) = self.enemy { e.hp -= dmg; }
                            if let Some(ref mut p) = self.player { p.total_damage_dealt += dmg; }
                            self.push_log(format!("You attack {} for {} damage!", e_name, dmg));
                            if !self.check_combat_end() {
                                self.enemy_attack_player();
                                self.check_combat_end();
                            }
                        }

                        // ── Heal item ─────────────────────────────────────────
                        VirtualKeyCode::H => {
                            // Look for an item with positive damage_or_defense as a proxy for healing items
                    let has_heal = self.player.as_ref()
                                .map(|p| p.inventory.iter().any(|i| i.base_type.contains("Potion") || i.name.to_lowercase().contains("potion") || i.name.to_lowercase().contains("heal")))
                                .unwrap_or(false);
                            if has_heal {
                                let idx = self.player.as_ref().unwrap()
                                    .inventory.iter().position(|i| i.base_type.contains("Potion") || i.name.to_lowercase().contains("potion") || i.name.to_lowercase().contains("heal")).unwrap();
                                if let Some(ref mut p) = self.player {
                                    if let Some(item) = p.use_item(idx) {
                                        let base_heal = (item.damage_or_defense.abs() + 20).max(20);
                                        let heal = p.item_heal_bonus(base_heal);
                                        p.heal(heal);
                                        p.items_used += 1;
                                        self.push_log(format!("Used {} — healed {} HP", item.name, heal));
                                    }
                                }
                                self.enemy_attack_player();
                                self.check_combat_end();
                            } else {
                                self.push_log("No healing items!".to_string());
                            }
                        }

                        // ── Defend ────────────────────────────────────────────
                        VirtualKeyCode::D => {
                            if let Some(ref mut p) = self.player {
                                let shield = (p.stats.vitality / 4 + 5).max(1);
                                use chaos_rpg_core::character::StatusEffect;
                                p.add_status(StatusEffect::Shielded(shield));
                                self.push_log(format!("You brace for impact! +{} shield", shield));
                            }
                            self.enemy_attack_player();
                            self.check_combat_end();
                        }

                        // ── Spell slots [1–8] ─────────────────────────────────
                        k @ (VirtualKeyCode::Key1 | VirtualKeyCode::Key2 | VirtualKeyCode::Key3
                           | VirtualKeyCode::Key4 | VirtualKeyCode::Key5 | VirtualKeyCode::Key6
                           | VirtualKeyCode::Key7 | VirtualKeyCode::Key8) => {
                            let idx = match k {
                                VirtualKeyCode::Key1 => 0, VirtualKeyCode::Key2 => 1,
                                VirtualKeyCode::Key3 => 2, VirtualKeyCode::Key4 => 3,
                                VirtualKeyCode::Key5 => 4, VirtualKeyCode::Key6 => 5,
                                VirtualKeyCode::Key7 => 6, _ => 7,
                            };
                            let (has_spell, spell_name, spell_cost, spell_scaling, spell_scaling_factor) = {
                                let p = self.player.as_ref().unwrap();
                                if idx < p.known_spells.len() {
                                    let s = &p.known_spells[idx];
                                    (true, s.name.clone(), s.mana_cost, s.scaling_stat.clone(), s.scaling_factor)
                                } else { (false, String::new(), 0i64, String::new(), 0.0f64) }
                            };
                            if has_spell {
                                if self.current_mana >= spell_cost {
                                    self.current_mana -= spell_cost;
                                    let stat_val = {
                                        let p = self.player.as_ref().unwrap();
                                        match spell_scaling.as_str() {
                                            "force"     => p.stats.force,
                                            "cunning"   => p.stats.cunning,
                                            "entropy"   => p.stats.entropy,
                                            "precision" => p.stats.precision,
                                            _           => p.stats.mana,
                                        }
                                    };
                                    let roll = chaos_roll_verbose(stat_val as f64 * 0.02, self.seed.wrapping_add(self.frame + 100));
                                    let dmg = ((stat_val as f64 * spell_scaling_factor.abs() * (1.0 + roll.final_value.abs())) as i64).max(1);
                                    let e_name = self.enemy.as_ref().unwrap().name.clone();
                                    if let Some(ref mut e) = self.enemy { e.hp -= dmg; }
                                    if let Some(ref mut p) = self.player { p.total_damage_dealt += dmg; p.spells_cast += 1; }
                                    self.push_log(format!("Cast {} on {} for {} damage!", spell_name, e_name, dmg));
                                    if !self.check_combat_end() {
                                        self.enemy_attack_player();
                                        self.check_combat_end();
                                    }
                                } else {
                                    self.push_log(format!("Not enough mana for {} ({} needed)", spell_name, spell_cost));
                                }
                            } else {
                                self.push_log("No spell in that slot.".to_string());
                            }
                        }

                        // ── Flee ──────────────────────────────────────────────
                        VirtualKeyCode::F | VirtualKeyCode::Escape => {
                            let luck = self.player.as_ref().map(|p| p.flee_luck_modifier()).unwrap_or(0);
                            let roll = chaos_roll_verbose(luck as f64 * 0.1, self.seed.wrapping_add(self.frame + 99));
                            if roll.final_value > -0.2 {
                                self.push_log("You flee from combat!".to_string());
                                self.enemy = None;
                                self.screen = AppScreen::FloorNav;
                            } else {
                                self.push_log("Flee failed! Enemy blocks your escape.".to_string());
                                self.enemy_attack_player();
                                self.check_combat_end();
                            }
                        }

                        _ => {}
                    }
                }

                AppScreen::GameOver => match key {
                    VirtualKeyCode::Return | VirtualKeyCode::Escape => {
                        self.screen = AppScreen::Title;
                        self.player = None;
                        self.enemy = None;
                    }
                    VirtualKeyCode::S => {
                        if let Some(ref p) = self.player {
                            use chaos_rpg_core::scoreboard::{save_score, ScoreEntry};
                            let entry = ScoreEntry::new(
                                p.name.clone(),
                                format!("{:?}", p.class),
                                p.score(),
                                p.floor,
                                p.kills,
                                0u32,
                            );
                            save_score(entry);
                            self.push_log("Score saved!".to_string());
                        }
                        self.screen = AppScreen::Title;
                        self.player = None;
                        self.enemy = None;
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
}

// ─── ENTRY POINT ─────────────────────────────────────────────────────────────

fn main() -> BError {
    let fullscreen = std::env::args().any(|a| a == "--fullscreen" || a == "-f");

    let mut builder = BTermBuilder::simple80x50()
        .with_title("CHAOS RPG — Where Math Goes To Die")
        .with_tile_dimensions(14, 14)
        .with_dimensions(120, 50)
        .with_fps_cap(60.0);

    if fullscreen {
        builder = builder.with_fullscreen(true);
    }

    let context = builder.build()?;
    let gs = State::new();
    main_loop(context, gs)
}
