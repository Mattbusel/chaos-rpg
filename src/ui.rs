//! Terminal UI helpers — box drawing, color coding, input, menus.
//!
//! Uses ANSI escape codes directly for maximum compatibility.
//! No raw-mode required — all input is line-buffered.

use crate::character::{Character, CharacterClass, Background, display_stat, stat_color};
use crate::enemy::Enemy;
use crate::scoreboard::ScoreEntry;
use std::io::{self, Write};

// ─── COLORS ──────────────────────────────────────────────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[97m";
pub const DIM: &str = "\x1b[2m";
pub const BG_RED: &str = "\x1b[41m";
pub const BG_BLUE: &str = "\x1b[44m";

// ─── PRIMITIVES ──────────────────────────────────────────────────────────────

pub fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    let _ = io::stdout().flush();
}

pub fn print_separator(ch: char, width: usize) {
    println!("{}", ch.to_string().repeat(width));
}

pub fn box_line(content: &str, width: usize) -> String {
    let pad = width.saturating_sub(content.len() + 4);
    format!("║ {}{} ║", content, " ".repeat(pad))
}

pub fn print_box(title: &str, lines: &[&str], width: usize) {
    let border = "═".repeat(width - 2);
    println!("╔{}╗", border);
    let title_pad = width.saturating_sub(title.len() + 4);
    let left_pad = title_pad / 2;
    let right_pad = title_pad - left_pad;
    println!("║{}{}{}║", " ".repeat(left_pad), title, " ".repeat(right_pad));
    println!("╠{}╣", border);
    for line in lines {
        println!("{}", box_line(line, width));
    }
    println!("╚{}╝", border);
}

pub fn prompt(msg: &str) -> String {
    print!("{}{}{} ", CYAN, msg, RESET);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}

pub fn press_enter(msg: &str) {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}

pub fn println_color(color: &str, msg: &str) {
    println!("{}{}{}", color, msg, RESET);
}

// ─── TITLE SCREEN ────────────────────────────────────────────────────────────

pub fn show_title() {
    clear_screen();
    println!("{}{}", MAGENTA, BOLD);
    println!(r"  ██████╗██╗  ██╗ █████╗  ██████╗ ███████╗");
    println!(r"  ██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝");
    println!(r"  ██║     ███████║███████║██║   ██║███████╗");
    println!(r"  ██║     ██╔══██║██╔══██║██║   ██║╚════██║");
    println!(r"  ╚██████╗██║  ██║██║  ██║╚██████╔╝███████║");
    println!(r"   ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝");
    println!();
    println!("  {}██████╗ ██████╗  ██████╗{}", CYAN, MAGENTA);
    println!("  {}██╔══██╗██╔══██╗██╔════╝{}", CYAN, MAGENTA);
    println!("  {}██████╔╝██████╔╝██║  ███╗{}", CYAN, MAGENTA);
    println!("  {}██╔══██╗██╔═══╝ ██║   ██║{}", CYAN, MAGENTA);
    println!("  {}██║  ██║██║     ╚██████╔╝{}", CYAN, MAGENTA);
    println!("  {}╚═╝  ╚═╝╚═╝      ╚═════╝{}", CYAN, MAGENTA);
    println!("{}", RESET);
    println!("  {}A mathematically cursed terminal roguelike.{}", DIM, RESET);
    println!("  {}Every outcome is determined by chaining 10 sacred algorithms.{}", DIM, RESET);
    println!();
    println!("  {}[ Lorenz · Fourier · Mandelbrot · Riemann · Fibonacci ]{}", YELLOW, RESET);
    println!("  {}[ Logistic · Euler · Collatz · Prime · Modular Exp    ]{}", YELLOW, RESET);
    println!();
}

// ─── MODE SELECTION ──────────────────────────────────────────────────────────

pub fn select_mode() -> GameMode {
    println!("  {}Select mode:{}", BOLD, RESET);
    println!("  {}[1]{} Story Mode    — 10 floors, narrative events, final boss", YELLOW, RESET);
    println!("  {}[2]{} Infinite Mode — endless floors, score attack", YELLOW, RESET);
    println!("  {}[3]{} View Scoreboard", YELLOW, RESET);
    println!("  {}[q]{} Quit", YELLOW, RESET);
    println!();

    loop {
        let input = prompt("Choose >");
        match input.as_str() {
            "1" => return GameMode::Story,
            "2" => return GameMode::Infinite,
            "3" => return GameMode::Scoreboard,
            "q" | "Q" | "quit" => return GameMode::Quit,
            _ => println!("  {}Invalid choice. Enter 1, 2, 3, or q.{}", RED, RESET),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    Story,
    Infinite,
    Scoreboard,
    Quit,
}

// ─── CHARACTER CREATION ──────────────────────────────────────────────────────

pub fn create_character_ui() -> (String, CharacterClass, Background) {
    clear_screen();
    println!("  {}=== CHARACTER CREATION ==={}", BOLD, RESET);
    println!();

    // Name
    let name = loop {
        let n = prompt("Enter your name >");
        if n.is_empty() {
            println!("  {}Name cannot be empty.{}", RED, RESET);
        } else if n.len() > 20 {
            println!("  {}Name too long (max 20 chars).{}", RED, RESET);
        } else {
            break n;
        }
    };

    println!();
    println!("  {}Choose your class:{}", BOLD, RESET);
    println!("  {}[1]{} Mage       — MANA & ENTROPY master. Volatile but devastating.", CYAN, RESET);
    println!("  {}[2]{} Berserker  — VITALITY & FORCE. Pain becomes power at low HP.", CYAN, RESET);
    println!("  {}[3]{} Ranger     — PRECISION & LUCK. Consistent, deadly at range.", CYAN, RESET);
    println!("  {}[4]{} Thief      — CUNNING & LUCK. Chaos favors the devious.", CYAN, RESET);
    println!();

    let class = loop {
        match prompt("Class >").as_str() {
            "1" => break CharacterClass::Mage,
            "2" => break CharacterClass::Berserker,
            "3" => break CharacterClass::Ranger,
            "4" => break CharacterClass::Thief,
            _ => println!("  {}Enter 1–4.{}", RED, RESET),
        }
    };

    println!();
    println!("  {}Choose your background:{}", BOLD, RESET);
    println!("  {}[1]{} Scholar   — +15 MANA, +10 ENTROPY", YELLOW, RESET);
    println!("  {}[2]{} Wanderer  — +15 LUCK, +10 PRECISION", YELLOW, RESET);
    println!("  {}[3]{} Gladiator — +15 FORCE, +10 VITALITY", YELLOW, RESET);
    println!("  {}[4]{} Outcast   — +15 CUNNING, +10 ENTROPY", YELLOW, RESET);
    println!("  {}[5]{} Merchant  — +10 LUCK, +15 CUNNING", YELLOW, RESET);
    println!("  {}[6]{} Cultist   — +20 MANA, +20 ENTROPY, -10 VITALITY", YELLOW, RESET);
    println!();

    let background = loop {
        match prompt("Background >").as_str() {
            "1" => break Background::Scholar,
            "2" => break Background::Wanderer,
            "3" => break Background::Gladiator,
            "4" => break Background::Outcast,
            "5" => break Background::Merchant,
            "6" => break Background::Cultist,
            _ => println!("  {}Enter 1–6.{}", RED, RESET),
        }
    };

    (name, class, background)
}

// ─── CHARACTER SHEET ─────────────────────────────────────────────────────────

pub fn show_character_sheet(c: &Character) {
    println!();
    println!("  ╔═══════════════════════════════════════════════╗");
    println!("  ║  {}{}  — {}  [Lv.{}]  Floor: {}{}", BOLD, c.name, c.class, c.level, c.floor, RESET);
    println!("  ║  Background: {}  |  Power: {}{}{}", c.background.name(),
        c.power_tier().color_code(), c.power_tier().name(), RESET);
    println!("  ╠═══════════════════════════════════════════════╣");
    println!("  ║  {}HP:{} {}", BOLD, RESET, c.hp_bar(24));
    println!("  ║  XP: {}  Gold: {}  Kills: {}", c.xp, c.gold, c.kills);
    println!("  ╠═══════════════════════════════════════════════╣");
    println!("  {}", display_stat("VITALITY", c.stats.vitality));
    println!("  {}", display_stat("FORCE",    c.stats.force));
    println!("  {}", display_stat("MANA",     c.stats.mana));
    println!("  {}", display_stat("CUNNING",  c.stats.cunning));
    println!("  {}", display_stat("PRECISION",c.stats.precision));
    println!("  {}", display_stat("ENTROPY",  c.stats.entropy));
    println!("  {}", display_stat("LUCK",     c.stats.luck));
    println!("  ╚═══════════════════════════════════════════════╝");
}

// ─── ENEMY DISPLAY ───────────────────────────────────────────────────────────

pub fn show_enemy(enemy: &Enemy) {
    println!();
    let color = enemy.tier_color();
    println!("  {}┌─ {} [{}] ─┐{}", color, enemy.name, enemy.tier.name(), RESET);
    for line in enemy.ascii_sprite.lines() {
        println!("  {}  {}{}", color, line, RESET);
    }
    println!("  HP: {}", enemy.hp_bar(24));
    if let Some(ability) = enemy.special_ability {
        println!("  {}⚡ {}{}", YELLOW, ability, RESET);
    }
}

// ─── COMBAT MENU ─────────────────────────────────────────────────────────────

pub fn show_combat_menu(player: &Character, enemy: &Enemy, round: u32) {
    println!();
    println!("  {}─── Round {} ────────────────────────────{}", DIM, round, RESET);
    println!("  {}  {} HP: {}{}", GREEN, player.name, RESET, player.hp_bar(18));
    println!("  {}  {} HP: {}{}", RED, enemy.name, RESET, enemy.hp_bar(18));
    println!();
    println!("  Actions:");
    println!("  {}[a]{} Attack          {}[h]{} Heavy Attack", CYAN, RESET, CYAN, RESET);
    println!("  {}[d]{} Defend          {}[s]{} Cast Spell (Mage)", CYAN, RESET, CYAN, RESET);
    println!("  {}[t]{} Taunt           {}[f]{} Flee", CYAN, RESET, CYAN, RESET);
    println!("  {}[?]{} Show last chaos roll", DIM, RESET);
}

pub fn read_combat_action() -> crate::combat::CombatAction {
    loop {
        let input = prompt("Action >");
        match input.to_lowercase().as_str() {
            "a" | "attack" => return crate::combat::CombatAction::Attack,
            "h" | "heavy" => return crate::combat::CombatAction::HeavyAttack,
            "d" | "defend" => return crate::combat::CombatAction::Defend,
            "t" | "taunt" => return crate::combat::CombatAction::Taunt,
            "f" | "flee" => return crate::combat::CombatAction::Flee,
            "s" | "spell" => return crate::combat::CombatAction::UseSpell(0),
            _ => println!("  {}Unknown action. Use a/h/d/t/f/s{}", RED, RESET),
        }
    }
}

// ─── COMBAT EVENTS ───────────────────────────────────────────────────────────

pub fn display_combat_events(events: &[crate::combat::CombatEvent]) {
    println!();
    for event in events {
        let line = event.to_display_string();
        let color = if line.contains("CRITICAL") || line.contains("CRIT") {
            YELLOW
        } else if line.contains("CHAOS") || line.contains("chaos") {
            MAGENTA
        } else if line.contains("slain") || line.contains("Victory") {
            GREEN
        } else if line.contains("damage") && line.starts_with("Enemy") {
            RED
        } else {
            WHITE
        };
        println!("  {}{}{}", color, line, RESET);
    }
}

// ─── FLOOR TRANSITION ────────────────────────────────────────────────────────

pub fn show_floor_header(floor: u32, mode: &GameMode) {
    println!();
    println!("  {}╔══════════════════════════════════╗{}", CYAN, RESET);
    let mode_str = if *mode == GameMode::Story { "Story" } else { "Infinite" };
    println!("  {}║  Floor {:3}  [{} Mode]             ║{}", CYAN, floor, mode_str, RESET);
    println!("  {}╚══════════════════════════════════╝{}", CYAN, RESET);
    println!();
}

pub fn floor_choices() -> FloorChoice {
    println!("  You stand at a crossroads.");
    println!("  {}[1]{} Explore (encounter enemy)", CYAN, RESET);
    println!("  {}[2]{} Descend to next floor", CYAN, RESET);
    println!("  {}[3]{} Rest (recover {}HP, cost: 10 gold)", CYAN, RESET, 20);
    println!("  {}[4]{} View character sheet", CYAN, RESET);
    println!("  {}[5]{} View last chaos roll trace", CYAN, RESET);
    println!();
    loop {
        match prompt("Choice >").as_str() {
            "1" => return FloorChoice::Explore,
            "2" => return FloorChoice::Descend,
            "3" => return FloorChoice::Rest,
            "4" => return FloorChoice::ViewSheet,
            "5" => return FloorChoice::ViewTrace,
            _ => println!("  {}Enter 1–5.{}", RED, RESET),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FloorChoice {
    Explore,
    Descend,
    Rest,
    ViewSheet,
    ViewTrace,
}

// ─── LEVEL UP DISPLAY ────────────────────────────────────────────────────────

pub fn show_level_up(new_level: u32, stat_gains: &str) {
    println!();
    println!("  {}★★★ LEVEL UP! You are now level {} ★★★{}", YELLOW, new_level, RESET);
    println!("  {}", stat_gains);
}

// ─── GAME OVER ───────────────────────────────────────────────────────────────

pub fn show_game_over(player: &Character) {
    println!();
    println!("  {}{}╔══════════════════════════════════╗{}", RED, BOLD, RESET);
    println!("  {}{}║          GAME  OVER              ║{}", RED, BOLD, RESET);
    println!("  {}{}╚══════════════════════════════════╝{}", RED, BOLD, RESET);
    println!();
    println!("  {} fell to the mathematical abyss.", player.name);
    println!("  Floor reached: {}", player.floor);
    println!("  Enemies slain: {}", player.kills);
    println!("  Final score:   {}{}{}", YELLOW, player.score(), RESET);
    println!();
}

pub fn show_victory(player: &Character) {
    println!();
    println!("  {}{}╔══════════════════════════════════════╗{}", YELLOW, BOLD, RESET);
    println!("  {}{}║    YOU CONQUERED THE CHAOS RPG!      ║{}", YELLOW, BOLD, RESET);
    println!("  {}{}╚══════════════════════════════════════╝{}", YELLOW, BOLD, RESET);
    println!();
    println!("  {} has transcended mathematical reality.", player.name);
    println!("  10 floors cleared. All algorithms mastered.");
    println!("  Final score: {}{}{}", MAGENTA, player.score(), RESET);
    println!();
}

// ─── SCOREBOARD ──────────────────────────────────────────────────────────────

pub fn show_scoreboard(scores: &[ScoreEntry]) {
    clear_screen();
    println!("  {}=== TOP SCORES ====================================={}", BOLD, RESET);
    println!();
    if scores.is_empty() {
        println!("  {}No scores yet. Be the first to conquer CHAOS RPG!{}", DIM, RESET);
    } else {
        println!("  {:>3}  {:<18} {:<12} {:>8}  {:>6}  {}",
            "#", "Name", "Class", "Score", "Floor", "Date");
        println!("  {}", "─".repeat(62));
        for (i, s) in scores.iter().enumerate() {
            let color = match i {
                0 => YELLOW,
                1 => WHITE,
                2 => "\x1b[33m",
                _ => DIM,
            };
            println!("  {}{:>3}  {:<18} {:<12} {:>8}  {:>6}  {}{}",
                color, i + 1, s.name, s.class, s.score, s.floor_reached, s.timestamp, RESET);
        }
    }
    println!();
    press_enter(&format!("  {}Press ENTER to return...{}", DIM, RESET));
}

// ─── HELP / TUTORIAL ─────────────────────────────────────────────────────────

pub fn show_help() {
    clear_screen();
    println!("  {}=== HOW TO PLAY CHAOS RPG ===={}", BOLD, RESET);
    println!();
    println!("  {}THE CHAOS ENGINE:{}", CYAN, RESET);
    println!("  Every roll chains 4-10 mathematical algorithms in sequence.");
    println!("  Each algorithm's output feeds as input to the next.");
    println!("  Same seed = same fate. There is no true randomness here.");
    println!();
    println!("  {}STATS:{}", CYAN, RESET);
    println!("  VITALITY  — Max HP and physical resistance");
    println!("  FORCE     — Physical attack power");
    println!("  MANA      — Spell power and magic resistance");
    println!("  CUNNING   — Crit chance, flee success, trap detection");
    println!("  PRECISION — Accuracy, ranged damage bonus");
    println!("  ENTROPY   — Chaos bonus on all rolls (amplifies extremes)");
    println!("  LUCK      — General fortune, damage reduction");
    println!();
    println!("  {}Stats have no cap. They can exceed 99999.{}", YELLOW, RESET);
    println!();
    println!("  {}COMBAT:{}", CYAN, RESET);
    println!("  [a] Attack      — Uses FORCE. Berserker rage bonus at low HP.");
    println!("  [h] Heavy Attack — More damage, can catastrophically miss.");
    println!("  [d] Defend      — Halves incoming damage this round.");
    println!("  [t] Taunt       — Stun enemy on crit. Enrage on catastrophe.");
    println!("  [f] Flee        — Uses LUCK+CUNNING to escape.");
    println!("  [s] Spell       — Mage-class spell using MANA.");
    println!();
    println!("  {}ROLLS:{}", CYAN, RESET);
    println!("  CRITICAL (>80%)  — Double damage / special effect");
    println!("  SUCCESS  (>50%)  — Normal success");
    println!("  FAILURE  (<50%)  — Reduced or no effect");
    println!("  CATASTROPHE (<-80%) — Total miss, possible self-damage");
    println!();
    press_enter(&format!("  {}Press ENTER...{}", DIM, RESET));
}

// ─── STORY NARRATIVE EVENTS ──────────────────────────────────────────────────

pub fn story_event(floor: u32, seed: u64) -> Option<String> {
    use crate::chaos_pipeline::chaos_roll_verbose;
    let roll = chaos_roll_verbose(floor as f64 * 0.1, seed);

    let events = [
        "The walls pulse with Fibonacci spirals. You feel... watched.",
        "A voice whispers: 'The zeta function has zeros you can't see.'",
        "Strange attractors orbit your footsteps. The Lorenz butterfly flaps.",
        "Prime numbers glow on the floor — 2, 3, 5, 7, 11...",
        "The Mandelbrot boundary bleeds through the ceiling.",
        "A logistic map cascades across the wall: r=3.9, x=chaos.",
        "Euler's totient carved into a skull: φ(1000003) = 1000002.",
        "Collatz sequence echoes: 27 → 82 → 41 → 124 → 62 → 31...",
        "The golden ratio hums in the dust: 1.6180339887...",
        "Binary — the first abstraction. The floor counts upward in base-2.",
    ];

    if roll.final_value > 0.3 {
        let idx = (seed % events.len() as u64) as usize;
        Some(format!("  {}▶ {}{}", MAGENTA, events[idx], RESET))
    } else {
        None
    }
}
