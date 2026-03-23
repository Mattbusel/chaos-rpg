//! CHAOS RPG — Graphical Frontend (bracket-lib)
//!
//! This crate renders CHAOS RPG using bracket-lib, providing a proper
//! windowed graphical experience with tileset-based rendering.
//!
//! # Architecture
//!
//! bracket-lib provides a virtual console (grid of character cells) rendered
//! via OpenGL. We use multiple console layers:
//!   - Console 0 (background): Floor tiles, room environment
//!   - Console 1 (entities): Player and enemy sprites via CP437 tileset
//!   - Console 2 (UI overlay): HP bars, floating damage numbers, status badges
//!
//! All game logic runs through `chaos-rpg-core`. This crate only handles
//! display and input mapping.
//!
//! # Status
//!
//! Phase 1 complete: framework + game loop + CP437 rendering.
//! Phase 2 (custom 32×32 sprite tileset) in progress.

use bracket_lib::prelude::*;
use chaos_rpg_core::{
    character::{Character, CharacterClass, ColorTheme, Difficulty},
    chaos_pipeline::chaos_roll_verbose,
    enemy::{generate_enemy, Enemy},
    world::{generate_floor, Floor, Room, RoomType},
};

mod renderer;
mod sprites;
mod ui_overlay;

// ─── GAME STATES ─────────────────────────────────────────────────────────────

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
    theme: ColorTheme,
    selected_menu: usize,
}

impl State {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        State {
            screen: AppScreen::Title,
            player: None,
            floor: None,
            enemy: None,
            combat_log: Vec::new(),
            seed,
            frame: 0,
            theme: ColorTheme::Classic,
            selected_menu: 0,
        }
    }
}

// ─── BRACKET-LIB GAME LOOP ────────────────────────────────────────────────────

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.frame += 1;
        ctx.cls();

        match self.screen {
            AppScreen::Title => self.draw_title(ctx),
            AppScreen::CharacterCreation => self.draw_char_creation(ctx),
            AppScreen::FloorNav => self.draw_floor_nav(ctx),
            AppScreen::Combat => self.draw_combat(ctx),
            AppScreen::GameOver => self.draw_game_over(ctx),
            AppScreen::Scoreboard => self.draw_scoreboard(ctx),
        }

        self.handle_input(ctx);
    }
}

impl State {
    // ── Title Screen ──────────────────────────────────────────────────────────

    fn draw_title(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let dim = RGB::named(DARK_GRAY);
        let yellow = RGB::named(YELLOW);
        let bg = RGB::named(BLACK);

        ctx.draw_box(1, 1, 78, 22, col, bg);

        // Title art using block chars
        ctx.print_color(10, 3, yellow, bg, "  ██████╗██╗  ██╗ █████╗  ██████╗ ███████╗");
        ctx.print_color(10, 4, yellow, bg, " ██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝");
        ctx.print_color(10, 5, col, bg,    " ██║     ███████║███████║██║   ██║███████╗ ");
        ctx.print_color(10, 6, col, bg,    " ╚██████╗██║  ██║██║  ██║╚██████╔╝███████║");
        ctx.print_color(10, 7, col, bg,    "  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝");

        ctx.print_color(20, 9, dim, bg, "Where math goes to die. 10 sacred algorithms.");

        ctx.print_color(2, 12, col, bg, "┌─────────────────────────────────┐");
        ctx.print_color(2, 13, col, bg, "│  Select Mode                    │");
        ctx.print_color(2, 14, col, bg, "│                                 │");

        let options = ["Story Mode (10 floors)", "Infinite Mode", "Daily Seed Challenge", "Scoreboard", "Quit"];
        for (i, opt) in options.iter().enumerate() {
            let row = 15 + i as i32;
            let (fg, prefix) = if i == self.selected_menu {
                (RGB::named(WHITE), "► ")
            } else {
                (dim, "  ")
            };
            ctx.print_color(4, row, fg, bg, &format!("{}{}", prefix, opt));
        }

        ctx.print_color(2, 21, col, bg, "└─────────────────────────────────┘");
        ctx.print_color(4, 23, dim, bg, "↑↓ Navigate   Enter Select   Q Quit   v0.1.0");
    }

    // ── Character Creation ────────────────────────────────────────────────────

    fn draw_char_creation(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let bg = RGB::named(BLACK);
        let yellow = RGB::named(YELLOW);

        ctx.draw_box(1, 1, 78, 30, col, bg);
        ctx.print_color(3, 2, yellow, bg, "=== CHARACTER CREATION ===");
        ctx.print_color(3, 4, col, bg, "Press ENTER to generate a random character.");
        ctx.print_color(3, 6, RGB::named(GRAY), bg, "Full character creation UI coming in v0.2.");
        ctx.print_color(3, 8, col, bg, "ESC = Back to title");
    }

    // ── Floor Navigation ──────────────────────────────────────────────────────

    fn draw_floor_nav(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let yellow = RGB::named(YELLOW);
        let bg = RGB::named(BLACK);
        let red = RGB::named(RED);

        if let Some(ref player) = self.player {
            ctx.draw_box(1, 1, 78, 30, col, bg);
            ctx.print_color(3, 2, yellow, bg, &format!("Floor {}   Gold: {}   Kills: {}",
                player.floor, player.gold, player.kills));

            // HP bar
            let hp_pct = player.current_hp as f32 / player.max_hp.max(1) as f32;
            let hp_filled = (hp_pct * 30.0) as i32;
            let hp_bar: String = "█".repeat(hp_filled as usize) + &"░".repeat(30 - hp_filled as usize);
            let hp_col = if hp_pct > 0.6 { RGB::named(GREEN) } else if hp_pct > 0.3 { yellow } else { red };
            ctx.print_color(3, 4, hp_col, bg, &format!("HP [{}] {}/{}", hp_bar, player.current_hp, player.max_hp));

            if let Some(ref floor) = self.floor {
                ctx.print_color(3, 6, col, bg, "Map:");
                for (i, line) in floor.minimap().lines().enumerate() {
                    ctx.print_color(5, 7 + i as i32, RGB::named(WHITE), bg, line);
                }
            }

            ctx.print_color(3, 18, col, bg, "[E] Enter room   [C] Character   [D] Descend");
        }
    }

    // ── Combat Screen ─────────────────────────────────────────────────────────

    fn draw_combat(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let yellow = RGB::named(YELLOW);
        let red = RGB::named(RED);
        let green = RGB::named(GREEN);
        let bg = RGB::named(BLACK);

        if let (Some(ref player), Some(ref enemy)) = (&self.player, &self.enemy) {
            // Left panel: enemy
            ctx.draw_box(1, 1, 48, 22, col, bg);
            ctx.print_color(3, 2, red, bg, &format!("[{}] {}", enemy.tier.name(), enemy.name));

            // Enemy sprite (ASCII art from enemy)
            let sprite_lines: Vec<&str> = enemy.ascii_sprite.lines().collect();
            for (i, line) in sprite_lines.iter().enumerate() {
                ctx.print_color(12, 4 + i as i32, red, bg, line);
            }

            // Enemy HP bar
            let ep = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
            let ef = (ep * 24.0) as usize;
            let ebar = "█".repeat(ef) + &"░".repeat(24 - ef);
            let ecol = if ep > 0.6 { green } else if ep > 0.3 { yellow } else { red };
            ctx.print_color(3, 15, ecol, bg, &format!("HP [{}] {}/{}", ebar, enemy.hp, enemy.max_hp));

            // Player stats
            ctx.draw_box(1, 23, 48, 10, col, bg);
            ctx.print_color(3, 24, yellow, bg, &format!("{} Lv.{} {}", player.name, player.level, player.class.name()));
            let pp = player.current_hp as f32 / player.max_hp.max(1) as f32;
            let pf = (pp * 24.0) as usize;
            let pbar = "█".repeat(pf) + &"░".repeat(24 - pf);
            let pcol = if pp > 0.6 { green } else if pp > 0.3 { yellow } else { red };
            ctx.print_color(3, 25, pcol, bg, &format!("HP [{}] {}/{}", pbar, player.current_hp, player.max_hp));

            // Actions
            ctx.print_color(3, 28, col, bg, "[A]ttack  [H]eavy  [D]efend  [T]aunt  [F]lee");

            // Right panel: combat log
            ctx.draw_box(50, 1, 29, 32, col, bg);
            ctx.print_color(52, 2, yellow, bg, "Combat Log");
            for (i, entry) in self.combat_log.iter().rev().take(26).enumerate() {
                let truncated = if entry.len() > 25 { &entry[..25] } else { entry };
                ctx.print_color(52, 4 + i as i32, RGB::named(GRAY), bg, truncated);
            }
        }
    }

    // ── Game Over ─────────────────────────────────────────────────────────────

    fn draw_game_over(&mut self, ctx: &mut BTerm) {
        let red = RGB::named(RED);
        let bg = RGB::named(BLACK);

        ctx.draw_box(10, 8, 60, 16, red, bg);
        ctx.print_color(28, 11, red, bg, "GAME OVER");
        ctx.print_color(18, 14, RGB::named(GRAY), bg, "The algorithms have judged you.");
        ctx.print_color(22, 18, RGB::named(CYAN), bg, "[ENTER] Return to title");
    }

    // ── Scoreboard ────────────────────────────────────────────────────────────

    fn draw_scoreboard(&mut self, ctx: &mut BTerm) {
        let col = RGB::named(CYAN);
        let yellow = RGB::named(YELLOW);
        let bg = RGB::named(BLACK);

        ctx.draw_box(1, 1, 78, 30, col, bg);
        ctx.print_color(30, 2, yellow, bg, "★ HALL OF CHAOS ★");

        let scores = chaos_rpg_core::scoreboard::load_scores();
        ctx.print_color(3, 4, col, bg, "#   Name             Class        Floor   Score");
        ctx.print_color(3, 5, RGB::named(DARK_GRAY), bg, "─".repeat(70));

        for (i, s) in scores.iter().take(20).enumerate() {
            let row_col = match i {
                0 => RGB::named(YELLOW),
                1 => RGB::named(WHITE),
                2 => RGB::named(ORANGE),
                _ => RGB::named(GRAY),
            };
            ctx.print_color(3, 6 + i as i32, row_col, bg,
                &format!("{:<4}{:<21}{:<13}{:<8}{}", i + 1, &s.name, &s.class, s.floor_reached, s.score));
        }

        ctx.print_color(3, 28, RGB::named(DARK_GRAY), bg, "ESC = Back to title");
    }

    // ── Input Handling ────────────────────────────────────────────────────────

    fn handle_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match self.screen {
                AppScreen::Title => {
                    match key {
                        VirtualKeyCode::Up => {
                            if self.selected_menu > 0 { self.selected_menu -= 1; }
                        }
                        VirtualKeyCode::Down => {
                            if self.selected_menu < 4 { self.selected_menu += 1; }
                        }
                        VirtualKeyCode::Return => {
                            match self.selected_menu {
                                0 | 1 | 2 => {
                                    self.screen = AppScreen::CharacterCreation;
                                }
                                3 => self.screen = AppScreen::Scoreboard,
                                4 => ctx.quitting = true,
                                _ => {}
                            }
                        }
                        VirtualKeyCode::Q => ctx.quitting = true,
                        _ => {}
                    }
                }
                AppScreen::CharacterCreation => {
                    match key {
                        VirtualKeyCode::Return => {
                            // Create a default character and start
                            use chaos_rpg_core::character::{Background, CharacterClass, Difficulty};
                            let player = Character::roll_new(
                                "Hero".to_string(),
                                CharacterClass::Mage,
                                Background::Scholar,
                                self.seed,
                                Difficulty::Normal,
                            );
                            self.floor = Some(generate_floor(player.floor, self.seed));
                            self.player = Some(player);
                            self.screen = AppScreen::FloorNav;
                        }
                        VirtualKeyCode::Escape => {
                            self.screen = AppScreen::Title;
                        }
                        _ => {}
                    }
                }
                AppScreen::FloorNav => {
                    match key {
                        VirtualKeyCode::E => {
                            // Enter the current room → generate enemy and go to combat
                            if let Some(ref player) = self.player {
                                let floor_seed = self.seed.wrapping_add(player.floor as u64);
                                let enemy = generate_enemy(player.floor.max(1), floor_seed);
                                self.enemy = Some(enemy);
                                self.combat_log.clear();
                                self.combat_log.push("Combat begins!".to_string());
                                self.screen = AppScreen::Combat;
                            }
                        }
                        VirtualKeyCode::D => {
                            // Descend
                            if let Some(ref mut player) = self.player {
                                player.floor += 1;
                                let new_seed = self.seed.wrapping_add(player.floor as u64 * 9973);
                                self.floor = Some(generate_floor(player.floor, new_seed));
                            }
                        }
                        VirtualKeyCode::Q | VirtualKeyCode::Escape => {
                            self.screen = AppScreen::Title;
                            self.player = None;
                        }
                        _ => {}
                    }
                }
                AppScreen::Combat => {
                    if let (Some(ref mut player), Some(ref mut enemy)) = (&mut self.player, &mut self.enemy) {
                        match key {
                            VirtualKeyCode::A => {
                                // Basic attack
                                let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, self.seed.wrapping_add(self.frame));
                                let dmg = ((player.stats.force as f64 * (1.0 + roll.final_value)) as i64).max(1);
                                enemy.hp -= dmg;
                                self.combat_log.push(format!("You attack for {} damage!", dmg));
                                if !enemy.is_alive() {
                                    self.combat_log.push(format!("{} slain! +{} XP", enemy.name, enemy.xp_reward));
                                    player.kills += 1;
                                    player.xp += enemy.xp_reward;
                                    player.gold += enemy.gold_reward;
                                    self.screen = AppScreen::FloorNav;
                                    return;
                                }
                                // Enemy attacks back
                                let eroll = chaos_roll_verbose(enemy.chaos_level, self.seed.wrapping_add(self.frame + 1));
                                let edmg = ((enemy.base_damage as f64 * (1.0 + eroll.final_value)) as i64).max(1);
                                player.current_hp -= edmg;
                                self.combat_log.push(format!("{} attacks for {}!", enemy.name, edmg));
                                if !player.is_alive() {
                                    self.screen = AppScreen::GameOver;
                                }
                            }
                            VirtualKeyCode::F => {
                                self.combat_log.push("You fled!".to_string());
                                self.screen = AppScreen::FloorNav;
                            }
                            VirtualKeyCode::Escape => {
                                self.screen = AppScreen::FloorNav;
                            }
                            _ => {}
                        }
                    }
                }
                AppScreen::GameOver => {
                    if key == VirtualKeyCode::Return || key == VirtualKeyCode::Escape {
                        self.screen = AppScreen::Title;
                        self.player = None;
                        self.enemy = None;
                    }
                }
                AppScreen::Scoreboard => {
                    if key == VirtualKeyCode::Escape {
                        self.screen = AppScreen::Title;
                    }
                }
            }
        }
    }
}

// ─── ENTRY POINT ─────────────────────────────────────────────────────────────

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("CHAOS RPG — Where Math Goes To Die")
        .build()?;

    let gs = State::new();
    main_loop(context, gs)
}
