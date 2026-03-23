//! Terminal display using crossterm.

use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

pub fn stat_color(value: i64) -> Color {
    match value {
        i64::MIN..=-100 => Color::DarkRed,
        -99..=-1 => Color::Red,
        0..=20 => Color::White,
        21..=100 => Color::Cyan,
        101..=1000 => Color::Green,
        1001..=9999 => Color::Yellow,
        _ => Color::Magenta,
    }
}

pub fn hp_color(pct: f64) -> Color {
    if pct > 0.6 {
        Color::Green
    } else if pct > 0.3 {
        Color::Yellow
    } else {
        Color::Red
    }
}

pub fn clear_screen() {
    let mut out = io::stdout();
    let _ = execute!(out, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0));
}

pub fn print_colored(text: &str, color: Color) {
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(color),
        Print(text),
        ResetColor
    );
}

pub fn println_colored(text: &str, color: Color) {
    print_colored(text, color);
    println!();
}

pub fn hp_bar(current: i64, max: i64, width: usize) -> String {
    let pct = if max > 0 {
        (current as f64 / max as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let filled = (pct * width as f64) as usize;
    format!(
        "[{}{}] {}/{}",
        "█".repeat(filled),
        "░".repeat(width - filled),
        current,
        max
    )
}

pub fn draw_title_screen() {
    clear_screen();
    let lines = [
        (
            "╔════════════════════════════════════════════════════╗",
            Color::Red,
        ),
        (
            "║   ____ _   _    _    ___  ____   ____  ____  ____  ║",
            Color::Red,
        ),
        (
            "║  / ___| | | |  /   / _ / ___| |  _ |  _ / ___| ║",
            Color::Red,
        ),
        (
            "║ | |   | |_| | / _ | | | ___  | |_) | |_) ___  ║",
            Color::Yellow,
        ),
        (
            "║ | |___|  _  |/ ___  |_| |___) ||  _ <|  __/ ___) |║",
            Color::Yellow,
        ),
        (
            "║  ____|_| |_/_/   ____/|____/ |_| __|   |____/ ║",
            Color::Magenta,
        ),
        (
            "║                                                      ║",
            Color::White,
        ),
        (
            "║          Where Math Goes To Die                      ║",
            Color::Cyan,
        ),
        (
            "║   Every outcome: 4-10 chained algorithms             ║",
            Color::Cyan,
        ),
        (
            "║                                                      ║",
            Color::White,
        ),
        (
            "║   [N] New Game (Story Mode)                          ║",
            Color::Green,
        ),
        (
            "║   [I] Infinite Mode                                  ║",
            Color::Green,
        ),
        (
            "║   [Q] Quick Roll (just character creation)           ║",
            Color::Green,
        ),
        (
            "║   [S] Scoreboard                                     ║",
            Color::Yellow,
        ),
        (
            "║   [H] Help / The 10 Algorithms                      ║",
            Color::Yellow,
        ),
        (
            "║   [X] Exit                                           ║",
            Color::DarkRed,
        ),
        (
            "╚════════════════════════════════════════════════════╝",
            Color::Red,
        ),
    ];
    for (line, color) in &lines {
        println_colored(line, *color);
    }
    println!();
    print!("> ");
    let _ = io::stdout().flush();
}

pub fn draw_character_sheet(character: &crate::character::Character) {
    clear_screen();
    println_colored(
        &format!("=== {} ({}) ===", character.name, character.class.name()),
        Color::Yellow,
    );
    println!();
    let stats = [
        ("Vitality", character.stats.vitality),
        ("Force", character.stats.force),
        ("Mana", character.stats.mana),
        ("Cunning", character.stats.cunning),
        ("Precision", character.stats.precision),
        ("Entropy", character.stats.entropy),
        ("Luck", character.stats.luck),
    ];
    for (name, val) in &stats {
        let color = stat_color(*val);
        print!("  {:12}", name);
        print_colored(&format!("{:>8}", val), color);
        let bar_len = ((*val).clamp(0, 100) as usize / 5).min(20);
        println!(" [{}{}]", "▓".repeat(bar_len), "░".repeat(20 - bar_len));
    }
    println!();
    let hp_col = hp_color(character.hp_percent());
    print!("  HP: ");
    println_colored(&hp_bar(character.current_hp, character.max_hp, 20), hp_col);
    println!(
        "  Lv.{}  Floor {}  XP: {}  Gold: {}  Kills: {}",
        character.level, character.floor, character.xp, character.gold, character.kills
    );
    println!();
    let tier = character.power_tier();
    let tc = match tier {
        crate::character::PowerTier::Mortal => Color::White,
        crate::character::PowerTier::Awakened => Color::Green,
        crate::character::PowerTier::Champion => Color::Cyan,
        crate::character::PowerTier::Legendary => Color::Yellow,
        crate::character::PowerTier::Transcendent => Color::Magenta,
        crate::character::PowerTier::Godlike => Color::Red,
    };
    print!("  POWER: ");
    println_colored(tier.name(), tc);
}

pub fn draw_combat_screen(
    player: &crate::character::Character,
    enemy: &crate::enemy::Enemy,
    log: &[String],
    floor: u32,
) {
    clear_screen();
    let w = 54usize;
    let bar = "=".repeat(w);
    println_colored(&format!("+{}+", bar), Color::DarkYellow);
    println_colored(
        &format!("| FLOOR {:3} {:^46}|", floor, ""),
        Color::DarkYellow,
    );
    println_colored(&format!("+{}+", bar), Color::DarkYellow);
    let tier_color = match enemy.tier {
        crate::enemy::EnemyTier::Minion => Color::Green,
        crate::enemy::EnemyTier::Elite => Color::Yellow,
        crate::enemy::EnemyTier::Champion => Color::Red,
        crate::enemy::EnemyTier::Boss | crate::enemy::EnemyTier::Abomination => Color::Magenta,
    };
    for line in enemy.ascii_sprite.lines() {
        println_colored(&format!("| {:<w$}|", line, w = w), tier_color);
    }
    println_colored(&format!("+{}+", bar), Color::DarkYellow);
    let ename = if enemy.name.len() > 40 {
        &enemy.name[..40]
    } else {
        &enemy.name
    };
    println_colored(
        &format!("| [{:^10}] {:<40}|", enemy.tier.name(), ename),
        tier_color,
    );
    let ebar = hp_bar(enemy.hp, enemy.max_hp, 24);
    print!("| HP: ");
    print_colored(&ebar, hp_color(enemy.hp_percent()));
    println!("{:>w$}|", "", w = (w - 5 - ebar.len()).max(0));
    println_colored(&format!("+{}+", bar), Color::DarkYellow);
    let show = 4usize.min(log.len());
    let start = log.len() - show;
    for line in &log[start..] {
        let t = if line.len() > w {
            &line[..w]
        } else {
            line.as_str()
        };
        println!("| {:<w$}|", t, w = w);
    }
    while log.len() - start < 4 {
        println!("| {:<w$}|", "", w = w);
        break;
    }
    println_colored(&format!("+{}+", bar), Color::DarkYellow);
    let pbar = hp_bar(player.current_hp, player.max_hp, 16);
    print!("| YOU HP: ");
    print_colored(&pbar, hp_color(player.hp_percent()));
    println!("{:>w$}|", "", w = (w - 9 - pbar.len()).max(0));
    println!(
        "| {:52}|",
        format!(
            "{} Lv.{} Gold:{}",
            player.class.name(),
            player.level,
            player.gold
        )
    );
    println_colored(&format!("+{}+", bar), Color::DarkYellow);
    println_colored(
        "| [A]ttack [H]eavy [D]efend [F]lee [T]alk [C]har    |",
        Color::Green,
    );
    println_colored(&format!("+{}+", bar), Color::DarkYellow);
    print!("> ");
    let _ = io::stdout().flush();
}

pub fn draw_scoreboard(scores: &[crate::scoreboard::ScoreEntry]) {
    clear_screen();
    println_colored("TOP SCORES", Color::Yellow);
    println!("{}", "-".repeat(60));
    if scores.is_empty() {
        println!("No scores yet. Go get cursed.");
    } else {
        println!(
            "{:<4} {:<16} {:<12} {:<10} {:<6} {}",
            "#", "Name", "Class", "Score", "Floor", "Date"
        );
        println!("{}", "-".repeat(60));
        for (i, s) in scores.iter().enumerate() {
            println!(
                "{:<4} {:<16} {:<12} {:<10} {:<6} {}",
                i + 1,
                &s.name[..s.name.len().min(15)],
                &s.class[..s.class.len().min(11)],
                s.score,
                s.floor_reached,
                s.timestamp
            );
        }
    }
    println!();
    print!("Press Enter to return...");
    let _ = io::stdout().flush();
}

// ─── ANSI COLOR CONSTANTS (for main.rs format! usage) ────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[97m";

// ─── GAME MODE / FLOOR CHOICE ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    Story,
    Infinite,
    Scoreboard,
    Quit,
}

#[derive(Debug, Clone)]
pub enum FloorChoice {
    Explore,
    Descend,
    Rest,
    ViewSheet,
    ViewTrace,
}

// ─── LINE-BUFFERED I/O HELPERS ────────────────────────────────────────────────

pub fn prompt(msg: &str) -> String {
    print!("{}{}{} ", CYAN, msg, RESET);
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    s.trim().to_string()
}

pub fn press_enter(msg: &str) {
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
}

pub fn println_color(color: &str, msg: &str) {
    println!("{}{}{}", color, msg, RESET);
}

// ─── HIGH-LEVEL SCREENS ───────────────────────────────────────────────────────

pub fn show_title() {
    draw_title_screen();
}

pub fn select_mode() -> GameMode {
    println!();
    println!(
        "  {}[N]{} Story Mode  {}[I]{} Infinite  {}[S]{} Scoreboard  {}[H]{} Help  {}[X]{} Exit",
        GREEN, RESET, GREEN, RESET, YELLOW, RESET, YELLOW, RESET, RED, RESET
    );
    println!();
    loop {
        let input = prompt(">");
        match input.to_uppercase().as_str() {
            "N" | "1" => return GameMode::Story,
            "I" | "2" => return GameMode::Infinite,
            "S" | "3" => return GameMode::Scoreboard,
            "H" | "?" => {
                show_help();
                draw_title_screen();
            }
            "X" | "Q" | "EXIT" | "QUIT" => return GameMode::Quit,
            _ => println!("  {}Unknown — N/I/S/H/X{}", DIM, RESET),
        }
    }
}

pub fn show_scoreboard(scores: &[crate::scoreboard::ScoreEntry]) {
    draw_scoreboard(scores);
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
}

pub fn show_character_sheet(c: &crate::character::Character) {
    draw_character_sheet(c);
}

pub fn show_enemy(enemy: &crate::enemy::Enemy) {
    println!();
    for line in enemy.ascii_sprite.lines() {
        println!("  {}", line);
    }
    println!(
        "  {}[{}]{} {}  HP: {}",
        CYAN,
        enemy.tier.name(),
        RESET,
        enemy.name,
        hp_bar(enemy.hp, enemy.max_hp, 20)
    );
    if let Some(ability) = enemy.special_ability {
        println!("  {}⚡ {}{}", YELLOW, ability, RESET);
    }
}

pub fn show_game_over(player: &crate::character::Character) {
    println!();
    println!("  {}{}☠  GAME OVER  ☠{}", RED, BOLD, RESET);
    println!("  {} fell at floor {}.", player.name, player.floor);
    println!(
        "  Enemies slain: {}  Gold: {}  Score: {}{}{}",
        player.kills,
        player.gold,
        YELLOW,
        player.score(),
        RESET
    );
    println!();
}

pub fn show_victory(player: &crate::character::Character) {
    println!();
    println!("  {}{}★  VICTORY  ★{}", YELLOW, BOLD, RESET);
    println!("  {} has transcended the mathematical abyss!", player.name);
    println!("  Score: {}{}{}", MAGENTA, player.score(), RESET);
    println!();
}

pub fn show_level_up(level: u32, msg: &str) {
    println!(
        "  {}{}▲ LEVEL UP! Now level {}. {}{}",
        YELLOW, BOLD, level, msg, RESET
    );
}

pub fn show_floor_header(floor: u32, mode: &GameMode) {
    let mode_str = if *mode == GameMode::Story {
        "Story"
    } else {
        "Infinite"
    };
    println_color(
        CYAN,
        &format!("  ═══ Floor {}  [{} Mode] ═══", floor, mode_str),
    );
    println!();
}

pub fn show_help() {
    clear_screen();
    println!("  {}=== CHAOS RPG: HOW TO PLAY ==={}", BOLD, RESET);
    println!();
    println!("  {}THE CHAOS ENGINE:{}", CYAN, RESET);
    println!("  Every roll chains 4-10 mathematical algorithms.");
    println!("  Lorenz · Fourier · Mandelbrot · Riemann · Fibonacci");
    println!("  Logistic · Euler · Collatz · Prime · Modular Exp");
    println!();
    println!("  {}COMBAT ACTIONS:{}", CYAN, RESET);
    println!("  [A] Attack  [H] Heavy Attack  [D] Defend  [T] Taunt  [F] Flee");
    println!();
    println!("  {}STATS (all unbounded):{}", CYAN, RESET);
    println!("  VIT=HP  FOR=Damage  MAN=Magic  CUN=Crit  PRC=Accuracy  ENT=Chaos  LCK=Fortune");
    println!();
    press_enter(&format!("  {}[ENTER] to return...{}", DIM, RESET));
}

pub fn story_event(floor: u32, seed: u64) -> Option<String> {
    use crate::chaos_pipeline::chaos_roll_verbose;
    let roll = chaos_roll_verbose(floor as f64 * 0.1, seed);
    let events = [
        "The walls pulse with Fibonacci spirals. You feel watched.",
        "A voice: 'The Riemann hypothesis holds here. Barely.'",
        "Strange attractors orbit your footsteps. The Lorenz butterfly flaps.",
        "Prime numbers glow on the floor: 2, 3, 5, 7, 11...",
        "The Mandelbrot boundary bleeds through the ceiling.",
        "Logistic map on the wall: r=3.9, x=chaos.",
        "Euler's identity carved in a skull: e^(iπ)+1=0",
        "Collatz echoes: 27→82→41→124→62→31→94→47→142...",
        "The golden ratio hums in dust: 1.6180339887...",
        "Binary — the first abstraction. The floor counts in base 2.",
    ];
    if roll.final_value > 0.3 {
        let idx = (seed % events.len() as u64) as usize;
        Some(format!("  {}▶ {}{}", MAGENTA, events[idx], RESET))
    } else {
        None
    }
}

pub fn floor_choices() -> FloorChoice {
    println!(
        "  {}[1]{} Explore  {}[2]{} Descend  {}[3]{} Rest(10g)  {}[4]{} Sheet  {}[5]{} Last Roll",
        CYAN, RESET, CYAN, RESET, CYAN, RESET, CYAN, RESET, CYAN, RESET
    );
    loop {
        match prompt(">").as_str() {
            "1" => return FloorChoice::Explore,
            "2" => return FloorChoice::Descend,
            "3" => return FloorChoice::Rest,
            "4" => return FloorChoice::ViewSheet,
            "5" => return FloorChoice::ViewTrace,
            _ => println!("  {}1-5{}", DIM, RESET),
        }
    }
}

pub fn create_character_ui() -> (
    String,
    crate::character::CharacterClass,
    crate::character::Background,
) {
    use crate::character::{Background, CharacterClass};
    clear_screen();
    println!("  {}CHARACTER CREATION{}", BOLD, RESET);
    println!();

    let name = loop {
        let n = prompt("Name >");
        if n.is_empty() {
            println!("  {}Name required.{}", RED, RESET);
        } else if n.len() > 20 {
            println!("  {}Max 20 chars.{}", RED, RESET);
        } else {
            break n;
        }
    };

    println!();
    println!(
        "  Class: {}[1]{} Mage  {}[2]{} Berserker  {}[3]{} Ranger  {}[4]{} Thief",
        CYAN, RESET, CYAN, RESET, CYAN, RESET, CYAN, RESET
    );
    let class = loop {
        match prompt("Class >").as_str() {
            "1" => break CharacterClass::Mage,
            "2" => break CharacterClass::Berserker,
            "3" => break CharacterClass::Ranger,
            "4" => break CharacterClass::Thief,
            _ => println!("  {}1-4{}", RED, RESET),
        }
    };

    println!();
    println!(
        "  Background: {}[1]{} Scholar  {}[2]{} Wanderer  {}[3]{} Gladiator",
        CYAN, RESET, CYAN, RESET, CYAN, RESET
    );
    println!(
        "              {}[4]{} Outcast  {}[5]{} Merchant  {}[6]{} Cultist",
        CYAN, RESET, CYAN, RESET, CYAN, RESET
    );
    let background = loop {
        match prompt("Background >").as_str() {
            "1" => break Background::Scholar,
            "2" => break Background::Wanderer,
            "3" => break Background::Gladiator,
            "4" => break Background::Outcast,
            "5" => break Background::Merchant,
            "6" => break Background::Cultist,
            _ => println!("  {}1-6{}", RED, RESET),
        }
    };

    (name, class, background)
}

pub fn show_combat_menu(
    player: &crate::character::Character,
    enemy: &crate::enemy::Enemy,
    round: u32,
) {
    let log: Vec<String> = Vec::new();
    draw_combat_screen(player, enemy, &log, player.floor);
    println!("  Round {}", round);
}

pub fn read_combat_action() -> crate::combat::CombatAction {
    use crate::combat::CombatAction;
    loop {
        let s = prompt("Action >");
        match s.to_lowercase().trim() {
            "a" | "attack" => return CombatAction::Attack,
            "h" | "heavy" => return CombatAction::HeavyAttack,
            "d" | "defend" => return CombatAction::Defend,
            "t" | "taunt" => return CombatAction::Taunt,
            "f" | "flee" => return CombatAction::Flee,
            "s" | "spell" => return CombatAction::UseSpell(0),
            _ => println!("  {}a/h/d/t/f/s{}", DIM, RESET),
        }
    }
}

pub fn display_combat_events(events: &[crate::combat::CombatEvent]) {
    for event in events {
        let line = event.to_display_string();
        let color = if line.contains("CRITICAL") || line.contains("CRIT") {
            YELLOW
        } else if line.contains("CHAOS") {
            MAGENTA
        } else if line.contains("slain") {
            GREEN
        } else if line.contains("damage") && line.contains("Enemy") {
            RED
        } else {
            WHITE
        };
        println!("  {}{}{}", color, line, RESET);
    }
}
