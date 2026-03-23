//! Terminal UI — all rendering, menus, and input for CHAOS RPG.
//!
//! Includes: ANSI color themes, box-drawing borders, class selector with ASCII
//! art, animated chaos roll display, difficulty/theme customization menus.

use crossterm::{
    cursor, execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::sync::OnceLock;

use crate::character::{
    display_stat, Boon, Character, CharacterClass, Background, ColorTheme, Difficulty,
};

// ─── GLOBAL THEME ─────────────────────────────────────────────────────────────

static THEME: OnceLock<ColorTheme> = OnceLock::new();

pub fn set_theme(theme: ColorTheme) {
    let _ = THEME.set(theme);
}

fn theme() -> ColorTheme {
    *THEME.get().unwrap_or(&ColorTheme::Classic)
}

// ─── ANSI COLOR CONSTANTS (for main.rs format! usage) ─────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[97m";
pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";

// ─── THEME-AWARE ACCESSORS ────────────────────────────────────────────────────

pub fn t_primary() -> &'static str {
    theme().primary()
}
pub fn t_danger() -> &'static str {
    theme().danger()
}
pub fn t_success() -> &'static str {
    theme().success()
}
pub fn t_warning() -> &'static str {
    theme().warning()
}
pub fn t_magic() -> &'static str {
    theme().magic()
}
pub fn t_title() -> &'static str {
    theme().title()
}

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

// ─── HELPERS ──────────────────────────────────────────────────────────────────

pub fn clear_screen() {
    let mut out = io::stdout();
    let _ = execute!(out, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0));
}

pub fn println_color(color: &str, msg: &str) {
    println!("{}{}{}", color, msg, RESET);
}

pub fn prompt(msg: &str) -> String {
    print!("{}{}{} ", t_primary(), msg, RESET);
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

// ─── BOX DRAWING UTILITIES ────────────────────────────────────────────────────

/// Draw a full-width labeled box header
pub fn box_header(label: &str, color: &str, width: usize) {
    let inner = width.saturating_sub(4);
    let padded = format!("{:^width$}", label, width = inner);
    println!(
        "{}╔{}╗{}",
        color,
        "═".repeat(width - 2),
        RESET
    );
    println!("{}║ {}{}{} ║{}", color, BOLD, padded, RESET, "");
    println!("{}╚{}╝{}", color, "═".repeat(width - 2), RESET);
    // fix trailing RESET
    print!("{}", RESET);
}

pub fn box_section(lines: &[String], color: &str, width: usize) {
    let inner = width.saturating_sub(4);
    println!("{}┌{}┐{}", color, "─".repeat(width - 2), RESET);
    for line in lines {
        let display = if line.len() > inner {
            format!("{}…", &line[..inner - 1])
        } else {
            format!("{:<width$}", line, width = inner)
        };
        println!("{}│ {} │{}", color, display, RESET);
    }
    println!("{}└{}┘{}", color, "─".repeat(width - 2), RESET);
}

pub fn separator(color: &str, width: usize) {
    println!("{}{}{}",  color, "─".repeat(width), RESET);
}

// ─── TITLE SCREEN ─────────────────────────────────────────────────────────────

pub fn show_title() {
    clear_screen();
    let c = t_title();
    let y = t_warning();
    let m = t_magic();
    let d = DIM;
    println!();
    println!("{}╔══════════════════════════════════════════════════════╗{}", c, RESET);
    println!("{}║{}                                                      {}║{}", c, RESET, c, RESET);
    println!("{}║{}  {}  ___  _   _    _    ___  ____   ____  ____  ____  {}║{}", c, RESET, y, RESET, c);
    println!("{}║{} {} / __|  | | |  / \\  / _ \\/ ___| |  _ \\|  _ \\/ ___| {}║{}", c, RESET, y, RESET, c);
    println!("{}║{} {}| |     | |_| | / _ \\| | | \\___ \\ | |_) | |_) \\___  {}║{}", c, RESET, y, RESET, c);
    println!("{}║{} {}| |___  |  _  |/ ___ \\ |_| |___) ||  _ <|  __/ ___) {}║{}", c, RESET, m, RESET, c);
    println!("{}║{}  {}\\____|_| |_/_/   \\_\\____/|____/ |_| \\_\\_|   |____/ {}║{}", c, RESET, m, RESET, c);
    println!("{}║{}                                                      {}║{}", c, RESET, c, RESET);
    println!("{}║{}     {}Where math goes to die. 10 sacred algorithms.{}     {}║{}", c, RESET, d, RESET, c, RESET);
    println!("{}║{}                                                      {}║{}", c, RESET, c, RESET);
    println!("{}╠══════════════════════════════════════════════════════╣{}", c, RESET);
    println!("{}║{}  {}[N]{} Story Mode      — 10 floors of pure chaos          {}║{}", c, RESET, GREEN, RESET, c, RESET);
    println!("{}║{}  {}[I]{} Infinite Mode   — descend forever                  {}║{}", c, RESET, GREEN, RESET, c, RESET);
    println!("{}║{}  {}[S]{} Scoreboard      — hall of the mathematically gifted {}║{}", c, RESET, YELLOW, RESET, c, RESET);
    println!("{}║{}  {}[H]{} Help / Tutorial — the 10 algorithms explained       {}║{}", c, RESET, YELLOW, RESET, c, RESET);
    println!("{}║{}  {}[X]{} Exit            — the chaos subsides                {}║{}", c, RESET, RED, RESET, c, RESET);
    println!("{}╚══════════════════════════════════════════════════════╝{}", c, RESET);
    println!();
}

pub fn select_mode() -> GameMode {
    loop {
        let input = prompt("  MODE >");
        match input.to_uppercase().as_str() {
            "N" | "1" => return GameMode::Story,
            "I" | "2" => return GameMode::Infinite,
            "S" | "3" => return GameMode::Scoreboard,
            "H" | "?" => {
                show_help();
                show_title();
            }
            "X" | "Q" | "EXIT" | "QUIT" => return GameMode::Quit,
            _ => println!("  {}Unknown — type N, I, S, H, or X{}", DIM, RESET),
        }
    }
}

// ─── DIFFICULTY SELECTION ─────────────────────────────────────────────────────

pub fn select_difficulty() -> Difficulty {
    clear_screen();
    let c = t_primary();
    println!();
    println!("  {}╔══════════════════════════════════╗{}", c, RESET);
    println!("  {}║       SELECT DIFFICULTY          ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════╝{}", c, RESET);
    println!();
    println!("  {}[1]{} Easy       — {}{}",        GREEN,   RESET, Difficulty::Easy.description(),    RESET);
    println!("  {}[2]{} Normal     — {}{}",        CYAN,    RESET, Difficulty::Normal.description(),  RESET);
    println!("  {}[3]{} Brutal     — {}{}",        YELLOW,  RESET, Difficulty::Brutal.description(),  RESET);
    println!("  {}[4]{} CHAOS      — {}{}",        RED,     RESET, Difficulty::Chaos.description(),   RESET);
    println!();
    loop {
        match prompt("  DIFFICULTY >").as_str() {
            "1" => return Difficulty::Easy,
            "2" | "" => return Difficulty::Normal,
            "3" => return Difficulty::Brutal,
            "4" => return Difficulty::Chaos,
            _ => println!("  {}Enter 1-4{}", DIM, RESET),
        }
    }
}

// ─── THEME SELECTION ──────────────────────────────────────────────────────────

pub fn select_color_theme() -> ColorTheme {
    clear_screen();
    println!();
    println!("  {}╔══════════════════════════════════╗{}", CYAN, RESET);
    println!("  {}║        SELECT COLOR THEME        ║{}", CYAN, RESET);
    println!("  {}╚══════════════════════════════════╝{}", CYAN, RESET);
    println!();
    println!("  {}[1]{} Classic    — Standard ANSI terminal colors", CYAN, RESET);
    println!("  {}[2]{} Neon       — Bright electric cyberpunk", BRIGHT_CYAN, RESET);
    println!("  {}[3]{} Blood      — Deep reds and dark tones", BRIGHT_RED, RESET);
    println!("  {}[4]{} Void       — Purple and shadow", BRIGHT_MAGENTA, RESET);
    println!("  {}[5]{} Monochrome — Grayscale only", WHITE, RESET);
    println!();
    loop {
        match prompt("  THEME >").as_str() {
            "1" | "" => return ColorTheme::Classic,
            "2" => return ColorTheme::Neon,
            "3" => return ColorTheme::Blood,
            "4" => return ColorTheme::Void,
            "5" => return ColorTheme::Monochrome,
            _ => println!("  {}Enter 1-5{}", DIM, RESET),
        }
    }
}

// ─── CLASS SELECTION UI ───────────────────────────────────────────────────────

pub fn create_character_ui() -> (String, CharacterClass, Background, Difficulty) {
    let difficulty = select_difficulty();
    let theme = select_color_theme();
    set_theme(theme);

    clear_screen();
    let c = t_primary();
    println!();
    println!("  {}╔══════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║           CHARACTER CREATION             ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════════════╝{}", c, RESET);
    println!();

    let name = loop {
        let n = prompt("  Your name >");
        if n.is_empty() {
            println!("  {}A name is required.{}", RED, RESET);
        } else if n.len() > 20 {
            println!("  {}Max 20 characters.{}", RED, RESET);
        } else {
            break n;
        }
    };

    let class = select_class_ui();
    let background = select_background_ui();

    (name, class, background, difficulty)
}

/// Show 3 random boons and let the player pick one.
pub fn show_boon_select(seed: u64) -> Boon {
    let boons = Boon::random_three(seed);
    let c = t_primary();
    let m = t_magic();
    clear_screen();
    println!();
    println!("  {}╔══════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║           ✦  CHOOSE YOUR BOON  ✦            ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════════════════╝{}", c, RESET);
    println!();
    println!("  {}A gift from the chaos — choose wisely.{}", m, RESET);
    println!();

    for (i, boon) in boons.iter().enumerate() {
        let bc = boon.color_code();
        println!("  {}[{}] {}{}{}", c, i + 1, bc, boon.name(), RESET);
        println!("      {}{}{}", DIM, boon.description(), RESET);
        println!();
    }

    loop {
        let input = prompt("  Choose boon [1/2/3] >");
        match input.trim() {
            "1" => return boons[0],
            "2" => return boons[1],
            "3" => return boons[2],
            _ => println!("  {}Enter 1, 2, or 3.{}", RED, RESET),
        }
    }
}

fn select_class_ui() -> CharacterClass {
    let classes = [
        (CharacterClass::Mage, "1"),
        (CharacterClass::Berserker, "2"),
        (CharacterClass::Ranger, "3"),
        (CharacterClass::Thief, "4"),
        (CharacterClass::Necromancer, "5"),
        (CharacterClass::Alchemist, "6"),
        (CharacterClass::Paladin, "7"),
        (CharacterClass::VoidWalker, "8"),
    ];

    clear_screen();
    let c = t_primary();
    println!();
    println!("  {}╔══════════════════════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║                     CHOOSE YOUR CLASS                        ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════════════════════════════════╝{}", c, RESET);
    println!();

    for (i, (class, num)) in classes.iter().enumerate() {
        let col = match i {
            0 => BRIGHT_CYAN,
            1 => BRIGHT_RED,
            2 => BRIGHT_GREEN,
            3 => YELLOW,
            4 => MAGENTA,
            5 => GREEN,
            6 => WHITE,
            _ => BRIGHT_MAGENTA,
        };
        println!(
            "  {}[{}]{} {:12} — {}",
            col, num, RESET, class.name(), class.description()
        );
        println!(
            "       {}Passive: {} — {}{}",
            DIM, class.passive_name(), class.passive_desc(), RESET
        );
        println!();
    }

    loop {
        match prompt("  CLASS >").as_str() {
            "1" => return CharacterClass::Mage,
            "2" => return CharacterClass::Berserker,
            "3" => return CharacterClass::Ranger,
            "4" => return CharacterClass::Thief,
            "5" => return CharacterClass::Necromancer,
            "6" => return CharacterClass::Alchemist,
            "7" => return CharacterClass::Paladin,
            "8" => return CharacterClass::VoidWalker,
            _ => println!("  {}Enter 1-8{}", DIM, RESET),
        }
    }
}

fn select_background_ui() -> Background {
    let backgrounds = [
        Background::Scholar,
        Background::Wanderer,
        Background::Gladiator,
        Background::Outcast,
        Background::Merchant,
        Background::Cultist,
        Background::Exile,
        Background::Oracle,
    ];

    clear_screen();
    let c = t_primary();
    println!();
    println!("  {}╔════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║          CHOOSE YOUR BACKGROUND        ║{}", c, RESET);
    println!("  {}╚════════════════════════════════════════╝{}", c, RESET);
    println!();

    for (i, bg) in backgrounds.iter().enumerate() {
        println!(
            "  {}[{}]{} {:12} — {}",
            t_primary(), i + 1, RESET, bg.name(), bg.description()
        );
    }
    println!();

    loop {
        let input = prompt("  BACKGROUND >");
        if let Ok(n) = input.parse::<usize>() {
            if n >= 1 && n <= backgrounds.len() {
                return backgrounds[n - 1];
            }
        }
        println!("  {}Enter 1-8{}", DIM, RESET);
    }
}

// ─── CHARACTER SHEET ──────────────────────────────────────────────────────────

pub fn show_character_sheet(c: &Character) {
    let col = t_primary();
    let tier = c.power_tier();
    let name_col = t_warning();
    println!();
    println!("  {}╔══════════════════════════════════════════════════╗{}", col, RESET);
    println!(
        "  {}║  {}{} {}{}  — Lv.{} {} ({}){}{}║{}",
        col, name_col, c.name, RESET, col,
        c.level, c.class.name(), c.background.name(),
        RESET, col, RESET
    );
    println!(
        "  {}║  {}{}  {}{}{}║{}",
        col, tier.color_code(), tier.name(),
        DIM, tier.flavor(), RESET,
        RESET
    );
    println!("  {}║  {}Passive: {}{} — {}{}║{}",
        col, t_magic(), BOLD, c.class.passive_name(), RESET, col, RESET);
    println!("  {}╠══════════════════════════════════════════════════╣{}", col, RESET);
    println!("  {}║  HP:  {}  {}║{}",
        col, c.hp_bar(24), col, RESET);
    println!("  {}║  Floor {}  Gold {}  Kills {}  XP {}{}  {}║{}",
        col, c.floor, c.gold, c.kills, c.xp, DIM, col, RESET);
    if !c.status_effects.is_empty() {
        println!("  {}║  Status: {}  {}║{}", col, c.status_badge_line(), col, RESET);
    }
    println!("  {}╠══════════════════════════════════════════════════╣{}", col, RESET);

    let stats = [
        ("Vitality",  c.stats.vitality),
        ("Force",     c.stats.force),
        ("Mana",      c.stats.mana),
        ("Cunning",   c.stats.cunning),
        ("Precision", c.stats.precision),
        ("Entropy",   c.stats.entropy),
        ("Luck",      c.stats.luck),
    ];
    for (name, val) in &stats {
        println!("  {}║ {}{}", col, display_stat(name, *val), RESET);
    }

    println!("  {}╠══════════════════════════════════════════════════╣{}", col, RESET);
    println!("  {}║  Spells: {}  Items: {}  Difficulty: {}{}║{}",
        col, c.known_spells.len(), c.inventory.len(),
        c.difficulty.name(), col, RESET);
    println!("  {}╚══════════════════════════════════════════════════╝{}", col, RESET);
}

// ─── ENEMY DISPLAY ────────────────────────────────────────────────────────────

pub fn show_enemy(enemy: &crate::enemy::Enemy) {
    let tier_col = enemy.tier_color();
    println!();
    for line in enemy.ascii_sprite.lines() {
        println!("  {}{}{}", tier_col, line, RESET);
    }
    println!();
    println!(
        "  {}[ {} ] {}  HP: {}/{}{}",
        tier_col,
        enemy.tier.name(),
        enemy.name,
        enemy.hp,
        enemy.max_hp,
        RESET
    );
    let hp_pct = enemy.hp as f64 / enemy.max_hp as f64;
    let hp_col = if hp_pct > 0.6 { GREEN } else if hp_pct > 0.3 { YELLOW } else { RED };
    let bar_len = 24usize;
    let filled = ((hp_pct * bar_len as f64) as usize).min(bar_len);
    println!(
        "  {}[{}{}{}]{}",
        hp_col,
        "█".repeat(filled),
        "░".repeat(bar_len - filled),
        RESET,
        ""
    );
    if let Some(ability) = enemy.special_ability {
        println!("  {}[ABILITY] {}{}", YELLOW, ability, RESET);
    }
}

// ─── COMBAT MENU ──────────────────────────────────────────────────────────────

pub fn show_combat_menu(
    player: &Character,
    enemy: &crate::enemy::Enemy,
    round: u32,
) {
    let c = t_primary();
    let tier_col = enemy.tier_color();
    let w = 56usize;
    let bar = "═".repeat(w - 2);

    println!();
    println!("  {}╔{}╗{}", c, bar, RESET);
    println!(
        "  {}║  {} Round {:<3} {}  Floor {}  {}{}{}║{}",
        c, BOLD, round, RESET, player.floor, DIM,
        player.difficulty.name(), RESET, RESET
    );
    println!("  {}╠{}╣{}", c, bar, RESET);

    // Enemy info block
    println!(
        "  {}║  {}[{}]{} {:<36}{}║{}",
        c, tier_col, enemy.tier.name(), RESET,
        &enemy.name[..enemy.name.len().min(36)],
        c, RESET
    );
    let hp_pct = enemy.hp as f64 / enemy.max_hp as f64;
    let e_col = if hp_pct > 0.6 { GREEN } else if hp_pct > 0.3 { YELLOW } else { RED };
    let efill = ((hp_pct * 30.0) as usize).min(30);
    println!(
        "  {}║  {}HP [{}{}{}{}]{} {}/{:<12}{}║{}",
        c, e_col,
        "█".repeat(efill), "░".repeat(30 - efill),
        RESET, e_col, RESET,
        enemy.hp, enemy.max_hp,
        c, RESET
    );

    println!("  {}╠{}╣{}", c, bar, RESET);

    // Player info block
    println!(
        "  {}║  {}{}{} Lv.{} {} {}{}║{}",
        c, BOLD, player.name, RESET,
        player.level, player.class.name(),
        c, "", RESET
    );
    println!(
        "  {}║  {}  Gold: {}  Kills: {}{}║{}",
        c, DIM, player.gold, player.kills, c, RESET
    );

    let pfill = ((player.hp_percent() * 30.0) as usize).min(30);
    let p_col = if player.hp_percent() > 0.6 { GREEN } else if player.hp_percent() > 0.3 { YELLOW } else { RED };
    println!(
        "  {}║  {}HP [{}{}{}{}]{} {}/{}{}{}║{}",
        c, p_col,
        "█".repeat(pfill), "░".repeat(30 - pfill),
        RESET, p_col, RESET,
        player.current_hp, player.max_hp,
        c, "", RESET
    );

    // Status badges
    let badges = player.status_badge_line();
    if !badges.is_empty() {
        println!("  {}║  Status: {}{:<30}{}║{}", c, badges, "", c, RESET);
    }

    println!("  {}╠{}╣{}", c, bar, RESET);

    // Action menu
    println!(
        "  {}║  {}[A]{} Attack    {}[H]{} Heavy    {}[D]{} Defend    {}[F]{} Flee    {}║{}",
        c, GREEN, RESET, YELLOW, RESET, CYAN, RESET, RED, RESET, c, RESET
    );
    println!(
        "  {}║  {}[T]{} Taunt     {}[S]{} Spell    {}[I]{} Item     {}[?]{} Trace   {}║{}",
        c, MAGENTA, RESET, BRIGHT_CYAN, RESET, GREEN, RESET, DIM, RESET, c, RESET
    );

    if !player.known_spells.is_empty() {
        println!("  {}║  {}Spells:{}", c, DIM, RESET);
        for (i, spell) in player.known_spells.iter().enumerate().take(4) {
            let name = &spell.name[..spell.name.len().min(28)];
            println!(
                "  {}║    [S{}] {}{:<32}{}║{}",
                c, i + 1, t_magic(), name, c, RESET
            );
        }
    }

    if !player.inventory.is_empty() {
        println!("  {}║  {}Items:{}", c, DIM, RESET);
        for (i, item) in player.inventory.iter().enumerate().take(4) {
            let name = &item.name[..item.name.len().min(28)];
            println!(
                "  {}║    [I{}] {}{:<32}{}║{}",
                c, i + 1, item.rarity.color_code(), name, c, RESET
            );
        }
    }

    println!("  {}╚{}╝{}", c, bar, RESET);
    println!();
}

pub fn read_combat_action() -> crate::combat::CombatAction {
    use crate::combat::CombatAction;
    loop {
        let s = prompt("  ACTION >");
        let lower = s.to_lowercase();
        let trimmed = lower.trim();
        match trimmed {
            "a" | "attack" => return CombatAction::Attack,
            "h" | "heavy" => return CombatAction::HeavyAttack,
            "d" | "defend" => return CombatAction::Defend,
            "t" | "taunt" => return CombatAction::Taunt,
            "f" | "flee" => return CombatAction::Flee,
            s if s.starts_with('s') => {
                let idx = s[1..].parse::<usize>().unwrap_or(1).saturating_sub(1);
                return CombatAction::UseSpell(idx);
            }
            s if s.starts_with('i') => {
                let idx = s[1..].parse::<usize>().unwrap_or(1).saturating_sub(1);
                return CombatAction::UseItem(idx);
            }
            "?" => {
                println!("  {}Use 't' after combat to review the last chaos trace.{}", DIM, RESET);
            }
            _ => println!("  {}a/h/d/t/f/s#/i#{}", DIM, RESET),
        }
    }
}

pub fn display_combat_events(events: &[crate::combat::CombatEvent]) {
    for event in events {
        let line = event.to_display_string();
        let color = if line.contains("CRITICAL") || line.contains("CRIT") {
            YELLOW
        } else if line.contains("CHAOS") || line.contains("chaos") {
            BRIGHT_MAGENTA
        } else if line.contains("slain") || line.contains("Victory") {
            GREEN
        } else if line.contains("damage") && line.contains("Enemy") {
            RED
        } else if line.contains("BACKFIRE") || line.contains("backfir") {
            BRIGHT_RED
        } else if line.contains("blasts") || line.contains("spell") || line.contains("Spell") {
            BRIGHT_CYAN
        } else if line.contains("recover") || line.contains("healed") || line.contains("heal") {
            BRIGHT_GREEN
        } else {
            WHITE
        };
        println!("  {}{}{}", color, line, RESET);
    }
}

// ─── FLOOR / ROOM DISPLAY ─────────────────────────────────────────────────────

pub fn show_floor_header(floor: u32, mode: &GameMode) {
    let mode_str = if *mode == GameMode::Story {
        "Story"
    } else {
        "Infinite"
    };
    let c = t_primary();
    println!();
    println!(
        "  {}╔══════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║  Floor {:>3}  [{:<8}]       ║{}",
        c, floor, mode_str, RESET
    );
    println!(
        "  {}╚══════════════════════════════╝{}",
        c, RESET
    );
    println!();
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
        "Euler's identity carved in a skull: e^(i*pi)+1=0",
        "Collatz echoes: 27, 82, 41, 124, 62, 31...",
        "The golden ratio hums in dust: 1.6180339887...",
        "Binary — the first abstraction. The floor counts in base 2.",
        "A necromancer's ghost: 'Death is just a local minimum.'",
        "Alchemical symbols bleed through the stone walls.",
        "A paladin's prayer carved in bedrock: '3.14159...'",
        "Void signatures pulse across every surface.",
    ];
    if roll.final_value > 0.3 {
        let idx = (seed % events.len() as u64) as usize;
        Some(format!("  {}>> {}{}", t_magic(), events[idx], RESET))
    } else {
        None
    }
}

// ─── LEVEL UP / VICTORY / GAME OVER ──────────────────────────────────────────

pub fn show_level_up(level: u32, msg: &str) {
    let c = t_warning();
    println!();
    println!("  {}╔═══════════════════════════════════╗{}", c, RESET);
    println!("  {}║   *** LEVEL UP! Now Level {:>3} ***  ║{}", c, level, RESET);
    println!("  {}║   {}{}{}{}║{}", c, DIM, msg, RESET, c, RESET);
    println!("  {}╚═══════════════════════════════════╝{}", c, RESET);
    println!();
}

pub fn show_victory(player: &Character) {
    let c = t_warning();
    println!();
    println!("  {}╔══════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║                                              ║{}", c, RESET);
    println!("  {}║     *** VICTORY — THE MATH YIELDS ***        ║{}", c, RESET);
    println!("  {}║                                              ║{}", c, RESET);
    println!("  {}║  {}{} has transcended the abyss!{}{}          ║{}", c, BOLD, player.name, RESET, c, RESET);
    println!("  {}║  Score: {}{}{:<8}{}{}                         ║{}", c, t_magic(), BOLD, player.score(), RESET, c, RESET);
    println!("  {}║                                              ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════════════════╝{}", c, RESET);
    println!();
}

pub fn show_game_over(player: &Character) {
    let c = t_danger();
    println!();
    println!("  {}╔══════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║                                              ║{}", c, RESET);
    println!("  {}║          *** GAME OVER ***                   ║{}", c, RESET);
    println!("  {}║                                              ║{}", c, RESET);
    println!(
        "  {}║  {}{}{} fell on Floor {}  Level {}{}{}          ║{}",
        c, BOLD, player.name, RESET, player.floor, player.level, c, RESET, RESET
    );
    println!(
        "  {}║  Final Score: {}{}{:<10}{}{}                  ║{}",
        c, t_warning(), BOLD, player.score(), RESET, c, RESET
    );
    println!("  {}║                                              ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════════════════╝{}", c, RESET);
    println!();
}

// ─── SCOREBOARD ───────────────────────────────────────────────────────────────

pub fn show_scoreboard(scores: &[crate::scoreboard::ScoreEntry]) {
    clear_screen();
    let c = t_warning();
    println!();
    println!("  {}╔══════════════════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║                  HALL OF CHAOS — TOP SCORES              ║{}", c, RESET);
    println!("  {}╠══════╦════════════════╦════════════╦═══════╦═══════╦═════╣{}", c, RESET);
    println!("  {}║ {:>4} ║ {:<14} ║ {:<10} ║ {:>5} ║ {:>5} ║ Date  ║{}", c, "#", "Name", "Class", "Score", "Floor", RESET);
    println!("  {}╠══════╬════════════════╬════════════╬═══════╬═══════╬═════╣{}", c, RESET);
    if scores.is_empty() {
        println!("  {}║  No scores yet. The void awaits your sacrifice.          ║{}", DIM, RESET);
    } else {
        for (i, s) in scores.iter().enumerate().take(15) {
            let row_col = if i == 0 { YELLOW } else if i < 3 { CYAN } else { WHITE };
            println!(
                "  {}║ {:>4} ║ {:<14} ║ {:<10} ║ {:>5} ║ {:>5} ║ {} ║{}",
                row_col,
                i + 1,
                &s.name[..s.name.len().min(14)],
                &s.class[..s.class.len().min(10)],
                s.score,
                s.floor_reached,
                &s.timestamp[..s.timestamp.len().min(5)],
                RESET
            );
        }
    }
    println!("  {}╚══════╩════════════════╩════════════╩═══════╩═══════╩═════╝{}", c, RESET);
    println!();
    press_enter(&format!("  {}[ENTER] to return...{}", DIM, RESET));
}

// ─── HELP ─────────────────────────────────────────────────────────────────────

pub fn show_help() {
    clear_screen();
    let c = t_primary();
    println!();
    println!("  {}╔══════════════════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║              CHAOS RPG — HOW TO PLAY                     ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════════════════════════════╝{}", c, RESET);
    println!();
    println!("  {}THE 10 SACRED ALGORITHMS:{}", t_warning(), RESET);
    println!("  {}Lorenz{}     · Butterfly effect chaos attractor", CYAN, RESET);
    println!("  {}Fourier{}    · Harmonic decomposition of fate", CYAN, RESET);
    println!("  {}Primes{}     · Density sieve of fortune", CYAN, RESET);
    println!("  {}Riemann{}    · Zeta function partial sums", CYAN, RESET);
    println!("  {}Fibonacci{}  · Golden spiral trajectory", CYAN, RESET);
    println!("  {}Mandelbrot{} · Escape velocity (inside = cursed)", CYAN, RESET);
    println!("  {}Logistic{}   · r=3.9 chaos regime bifurcation", CYAN, RESET);
    println!("  {}Euler{}      · Totient ratio irregularity", CYAN, RESET);
    println!("  {}Collatz{}    · 3n+1 stopping time", CYAN, RESET);
    println!("  {}ModExp{}     · Modular exponentiation hash", CYAN, RESET);
    println!();
    println!("  {}STATS (all unbounded — can go negative):{}", t_warning(), RESET);
    println!("  VIT=HP  FOR=Damage  MAN=Magic  CUN=Crit");
    println!("  PRC=Accuracy  ENT=Chaos bonus  LCK=Fortune");
    println!();
    println!("  {}COMBAT:{}", t_warning(), RESET);
    println!("  [A] Attack   [H] Heavy   [D] Defend   [T] Taunt");
    println!("  [F] Flee     [S#] Spell  [I#] Item    [?] Show last trace");
    println!();
    println!("  {}CLASSES:{}", t_warning(), RESET);
    for class in &[
        CharacterClass::Mage, CharacterClass::Berserker,
        CharacterClass::Ranger, CharacterClass::Thief,
        CharacterClass::Necromancer, CharacterClass::Alchemist,
        CharacterClass::Paladin, CharacterClass::VoidWalker,
    ] {
        println!("  {}{:<12}{} {}Passive:{} {}", t_primary(), class.name(), RESET, DIM, RESET, class.passive_desc());
    }
    println!();
    press_enter(&format!("  {}[ENTER] to return...{}", DIM, RESET));
}
