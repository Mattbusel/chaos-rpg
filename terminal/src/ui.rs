//! Terminal UI — all rendering, menus, and input for CHAOS RPG.
//!
//! Includes: ANSI color themes, box-drawing borders, class selector with ASCII
//! art, animated chaos roll display, difficulty/theme customization menus.

use crossterm::{
    cursor, execute,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::OnceLock;

// ─── AUTO-PLAY MODE ───────────────────────────────────────────────────────────

thread_local! {
    static AUTO_MODE: RefCell<bool> = RefCell::new(false);
}

pub fn set_auto_mode(v: bool) {
    AUTO_MODE.with(|m| *m.borrow_mut() = v);
}

pub fn is_auto_mode() -> bool {
    AUTO_MODE.with(|m| *m.borrow())
}

/// Choose a combat action automatically based on character state.
pub fn auto_combat_action(player: &chaos_rpg_core::character::Character) -> chaos_rpg_core::combat::CombatAction {
    use chaos_rpg_core::combat::CombatAction;
    let hp_pct = player.current_hp as f32 / player.max_hp.max(1) as f32;
    if hp_pct < 0.25 {
        println!("  {}[AUTO] HP critical — Defending.{}", GREEN, RESET);
        return CombatAction::Defend;
    }
    if !player.known_spells.is_empty() && player.stats.mana > 10 {
        println!("  {}[AUTO] Casting spell 1.{}", GREEN, RESET);
        return CombatAction::UseSpell(0);
    }
    println!("  {}[AUTO] Attacking.{}", GREEN, RESET);
    CombatAction::Attack
}

use chaos_rpg_core::character::{
    display_stat, Background, Boon, Character, CharacterClass, ColorTheme, Difficulty,
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
    DailySeed,
    Scoreboard,
    Bestiary,
    Codex,
    Achievements,
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

// ─── FULLSCREEN / ALTERNATE SCREEN ────────────────────────────────────────────

/// Enters the alternate screen buffer and hides the cursor.
/// Drop the returned guard to restore the terminal.
pub struct FullscreenGuard;

impl FullscreenGuard {
    pub fn enter() -> Self {
        let mut out = io::stdout();
        let _ = execute!(out, EnterAlternateScreen, cursor::Hide);
        // Install a panic hook so the terminal is always restored
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let mut out = io::stdout();
            let _ = execute!(out, LeaveAlternateScreen, cursor::Show);
            default_hook(info);
        }));
        FullscreenGuard
    }
}

impl Drop for FullscreenGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = execute!(out, LeaveAlternateScreen, cursor::Show);
    }
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
    if is_auto_mode() {
        // Brief pause so the player can read what happened before auto-advancing.
        std::thread::sleep(std::time::Duration::from_millis(600));
        return;
    }
    print!("{}", msg);
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
}

// ─── BOX DRAWING UTILITIES ────────────────────────────────────────────────────

/// Count visible display columns in a string, skipping ANSI escape sequences.
pub fn visible_len(s: &str) -> usize {
    let mut len = 0usize;
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_esc = true;
        } else if in_esc {
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

/// Right-pad `s` to `width` visible columns with spaces (ANSI-aware).
pub fn pad_visible(s: &str, width: usize) -> String {
    let v = visible_len(s);
    if v < width {
        format!("{}{}", s, " ".repeat(width - v))
    } else {
        s.to_string()
    }
}

/// Truncate `s` to at most `max_cols` visible columns (ANSI-aware).
/// Appends '…' if truncated.
pub fn truncate_visible(s: &str, max_cols: usize) -> String {
    if visible_len(s) <= max_cols {
        return s.to_string();
    }
    // Walk chars, keeping track of visible count, stop before ANSI sequences.
    let mut out = String::new();
    let mut vis = 0usize;
    let mut in_esc = false;
    let target = max_cols.saturating_sub(1); // 1 col for the ellipsis
    for c in s.chars() {
        if c == '\x1b' {
            in_esc = true;
            out.push(c);
        } else if in_esc {
            out.push(c);
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
        } else {
            if vis >= target {
                break;
            }
            out.push(c);
            vis += 1;
        }
    }
    out.push('…');
    out
}

/// Draw a full-width labeled box header
pub fn box_header(label: &str, color: &str, width: usize) {
    let inner = width.saturating_sub(4);
    let padded = format!("{:^width$}", label, width = inner);
    println!("{}╔{}╗{}", color, "═".repeat(width - 2), RESET);
    println!("{}║ {}{}{} ║", color, BOLD, padded, RESET);
    println!("{}╚{}╝{}", color, "═".repeat(width - 2), RESET);
    print!("{}", RESET);
}

pub fn box_section(lines: &[String], color: &str, width: usize) {
    let inner = width.saturating_sub(4);
    println!("{}┌{}┐{}", color, "─".repeat(width - 2), RESET);
    for line in lines {
        let truncated = truncate_visible(line, inner);
        let padded = pad_visible(&truncated, inner);
        println!("{}│ {} │{}", color, padded, RESET);
    }
    println!("{}└{}┘{}", color, "─".repeat(width - 2), RESET);
}

pub fn separator(color: &str, width: usize) {
    println!("{}{}{}", color, "─".repeat(width), RESET);
}

// ─── TITLE SCREEN ─────────────────────────────────────────────────────────────

pub fn show_title() {
    clear_screen();
    let c = t_title();
    let y = t_warning();
    let m = t_magic();
    let d = DIM;
    println!();
    println!(
        "{}╔══════════════════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "{}║{}                                                      {}║{}",
        c, RESET, c, RESET
    );
    println!(
        "{}║{}  {}  ___  _   _    _    ___  ____   ____  ____  ____  {}║{}",
        c, RESET, y, RESET, c
    );
    println!(
        "{}║{} {} / __|  | | |  / \\  / _ \\/ ___| |  _ \\|  _ \\/ ___| {}║{}",
        c, RESET, y, RESET, c
    );
    println!(
        "{}║{} {}| |     | |_| | / _ \\| | | \\___ \\ | |_) | |_) \\___  {}║{}",
        c, RESET, y, RESET, c
    );
    println!(
        "{}║{} {}| |___  |  _  |/ ___ \\ |_| |___) ||  _ <|  __/ ___) {}║{}",
        c, RESET, m, RESET, c
    );
    println!(
        "{}║{}  {}\\____|_| |_/_/   \\_\\____/|____/ |_| \\_\\_|   |____/ {}║{}",
        c, RESET, m, RESET, c
    );
    println!(
        "{}║{}                                                      {}║{}",
        c, RESET, c, RESET
    );
    println!(
        "{}║{}     {}Where math goes to die. 10 sacred algorithms.{}     {}║{}",
        c, RESET, d, RESET, c, RESET
    );
    println!(
        "{}║{}                                                      {}║{}",
        c, RESET, c, RESET
    );
    println!(
        "{}╠══════════════════════════════════════════════════════╣{}",
        c, RESET
    );
    println!(
        "{}║{}  {}[N]{} Story Mode      — 10 floors of pure chaos          {}║{}",
        c, RESET, GREEN, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[I]{} Infinite Mode   — descend forever                  {}║{}",
        c, RESET, GREEN, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[D]{} Daily Seed      — shared seed, everyone races today {}║{}",
        c, RESET, CYAN, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[S]{} Scoreboard      — hall of the mathematically gifted {}║{}",
        c, RESET, YELLOW, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[B]{} Bestiary        — entities encountered in The Proof  {}║{}",
        c, RESET, MAGENTA, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[X]{} Codex           — lore of The Proof (unlockable)     {}║{}",
        c, RESET, CYAN, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[A]{} Achievements    — see what you've unlocked           {}║{}",
        c, RESET, BRIGHT_CYAN, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[H]{} Help / Tutorial — the 10 algorithms explained       {}║{}",
        c, RESET, YELLOW, RESET, c, RESET
    );
    println!(
        "{}║{}  {}[Q]{} Exit            — the chaos subsides                {}║{}",
        c, RESET, RED, RESET, c, RESET
    );
    println!(
        "{}╚══════════════════════════════════════════════════════╝{}",
        c, RESET
    );
    println!();
}

pub fn select_mode() -> GameMode {
    loop {
        let input = prompt("  MODE >");
        match input.to_uppercase().as_str() {
            "N" | "1" => return GameMode::Story,
            "I" | "2" => return GameMode::Infinite,
            "D" | "3" => return GameMode::DailySeed,
            "S" | "4" => return GameMode::Scoreboard,
            "B" => return GameMode::Bestiary,
            "X" | "5" => return GameMode::Codex,
            "A" => return GameMode::Achievements,
            "H" | "?" => {
                show_help();
                show_title();
            }
            "Q" | "EXIT" | "QUIT" => return GameMode::Quit,
            _ => println!("  {}Unknown — type N, I, D, S, B, X, A, H, or Q{}", DIM, RESET),
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
    println!(
        "  {}[1]{} Easy       — {}{}",
        GREEN,
        RESET,
        Difficulty::Easy.description(),
        RESET
    );
    println!(
        "  {}[2]{} Normal     — {}{}",
        CYAN,
        RESET,
        Difficulty::Normal.description(),
        RESET
    );
    println!(
        "  {}[3]{} Brutal     — {}{}",
        YELLOW,
        RESET,
        Difficulty::Brutal.description(),
        RESET
    );
    println!(
        "  {}[4]{} CHAOS      — {}{}",
        RED,
        RESET,
        Difficulty::Chaos.description(),
        RESET
    );
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
    println!(
        "  {}[1]{} Classic    — Standard ANSI terminal colors",
        CYAN, RESET
    );
    println!(
        "  {}[2]{} Neon       — Bright electric cyberpunk",
        BRIGHT_CYAN, RESET
    );
    println!(
        "  {}[3]{} Blood      — Deep reds and dark tones",
        BRIGHT_RED, RESET
    );
    println!(
        "  {}[4]{} Void       — Purple and shadow",
        BRIGHT_MAGENTA, RESET
    );
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
    println!(
        "  {}╔══════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║           CHARACTER CREATION             ║{}",
        c, RESET
    );
    println!(
        "  {}╚══════════════════════════════════════════╝{}",
        c, RESET
    );
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
    println!(
        "  {}╔══════════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║           ✦  CHOOSE YOUR BOON  ✦            ║{}",
        c, RESET
    );
    println!(
        "  {}╚══════════════════════════════════════════════╝{}",
        c, RESET
    );
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
        (CharacterClass::Mage,         "1"),
        (CharacterClass::Berserker,    "2"),
        (CharacterClass::Ranger,       "3"),
        (CharacterClass::Thief,        "4"),
        (CharacterClass::Necromancer,  "5"),
        (CharacterClass::Alchemist,    "6"),
        (CharacterClass::Paladin,      "7"),
        (CharacterClass::VoidWalker,   "8"),
        (CharacterClass::Warlord,      "9"),
        (CharacterClass::Trickster,    "a"),
        (CharacterClass::Runesmith,    "b"),
        (CharacterClass::Chronomancer, "c"),
    ];

    clear_screen();
    let c = t_primary();
    println!();
    println!(
        "  {}╔══════════════════════════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║                     CHOOSE YOUR CLASS                        ║{}",
        c, RESET
    );
    println!(
        "  {}╚══════════════════════════════════════════════════════════════╝{}",
        c, RESET
    );
    println!();

    for (i, (class, num)) in classes.iter().enumerate() {
        let col = match i {
            0  => BRIGHT_CYAN,
            1  => BRIGHT_RED,
            2  => BRIGHT_GREEN,
            3  => YELLOW,
            4  => MAGENTA,
            5  => GREEN,
            6  => WHITE,
            7  => BRIGHT_MAGENTA,
            8  => RED,
            9  => CYAN,
            10 => BRIGHT_GREEN,
            _  => BRIGHT_RED,
        };
        println!(
            "  {}[{}]{} {:12} — {}",
            col,
            num,
            RESET,
            class.name(),
            class.description()
        );
        println!(
            "       {}Passive: {} — {}{}",
            DIM,
            class.passive_name(),
            class.passive_desc(),
            RESET
        );
        println!();
    }

    loop {
        match prompt("  CLASS [1-9/a-c] >").as_str() {
            "1" => return CharacterClass::Mage,
            "2" => return CharacterClass::Berserker,
            "3" => return CharacterClass::Ranger,
            "4" => return CharacterClass::Thief,
            "5" => return CharacterClass::Necromancer,
            "6" => return CharacterClass::Alchemist,
            "7" => return CharacterClass::Paladin,
            "8" => return CharacterClass::VoidWalker,
            "9" => return CharacterClass::Warlord,
            "a" => return CharacterClass::Trickster,
            "b" => return CharacterClass::Runesmith,
            "c" => return CharacterClass::Chronomancer,
            _ => println!("  {}Enter 1-9 or a/b/c{}", DIM, RESET),
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
            t_primary(),
            i + 1,
            RESET,
            bg.name(),
            bg.description()
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
    println!(
        "  {}╔══════════════════════════════════════════════════╗{}",
        col, RESET
    );
    println!(
        "  {}║  {}{} {}{}  — Lv.{} {} ({}){}{}║{}",
        col,
        name_col,
        c.name,
        RESET,
        col,
        c.level,
        c.class.name(),
        c.background.name(),
        RESET,
        col,
        RESET
    );
    let (power_label, power_value) = c.power_display();
    let underdog = c.underdog_multiplier();
    let underdog_str = if underdog > 1.01 {
        format!("  {}[UNDERDOG ×{:.1}]{}", "\x1b[33m", underdog, RESET)
    } else { String::new() };
    let misery = c.misery.misery_index;
    let misery_str = if misery >= 100.0 {
        format!("  {}MISERY: {:.0}{}", "\x1b[35m", misery, RESET)
    } else { String::new() };
    println!(
        "  {}║  {}{}: {}{}{}  {}{}{}{}║{}",
        col,
        tier.ansi_color(),
        power_label,
        power_value,
        RESET,
        underdog_str,
        DIM,
        tier.flavor(),
        misery_str,
        RESET,
        RESET
    );
    {
        // Passive: "  ║  [PASSIVE] Name — desc  ║" — box inner is 48 visible cols
        let pname = c.class.passive_name();
        let pdesc = c.class.passive_desc();
        // prefix "  [PASSIVE] " = 12 visible, name, " -- " = 4, desc, trailing up to border
        let prefix_vis = 12 + pname.len() + 4;
        let desc_budget = 48usize.saturating_sub(prefix_vis);
        let desc_show = if pdesc.len() > desc_budget && desc_budget > 1 {
            format!("{}…", &pdesc[..desc_budget.saturating_sub(1)])
        } else {
            pdesc.to_string()
        };
        let full_line = format!(
            "{}[PASSIVE]{} {} -- {}",
            t_magic(), RESET, pname, desc_show
        );
        let padded = pad_visible(&full_line, 48);
        println!("  {}║  {}{}║{}", col, padded, col, RESET);
    }
    println!(
        "  {}╠══════════════════════════════════════════════════╣{}",
        col, RESET
    );
    println!("  {}║  HP:  {}  {}║{}", col, c.hp_bar(24), col, RESET);
    println!(
        "  {}║  Floor {}  Gold {}  Kills {}  XP {}{}  {}║{}",
        col, c.floor, c.gold, c.kills, c.xp, DIM, col, RESET
    );
    if c.corruption_stage() > 0 {
        let next_milestone = ((c.kills / 50) + 1) * 50;
        println!(
            "  {}║  Corruption: {} — Stage {}/8  (next: {} kills){}  {}║{}",
            col,
            c.corruption_label(),
            c.corruption_stage(),
            next_milestone,
            DIM,
            col,
            RESET
        );
    }
    if !c.status_effects.is_empty() {
        println!(
            "  {}║  Status: {}  {}║{}",
            col,
            c.status_badge_line(),
            col,
            RESET
        );
    }
    println!(
        "  {}╠══════════════════════════════════════════════════╣{}",
        col, RESET
    );

    let stats = [
        ("Vitality", c.stats.vitality),
        ("Force", c.stats.force),
        ("Mana", c.stats.mana),
        ("Cunning", c.stats.cunning),
        ("Precision", c.stats.precision),
        ("Entropy", c.stats.entropy),
        ("Luck", c.stats.luck),
    ];
    for (name, val) in &stats {
        println!("  {}║ {}{}", col, display_stat(name, *val), RESET);
    }

    println!(
        "  {}╠══════════════════════════════════════════════════╣{}",
        col, RESET
    );
    // Inventory / spells / skill points row
    {
        let row = format!(
            "Spells: {}  Items: {}  Difficulty: {}  SP: {}",
            c.known_spells.len(), c.inventory.len(), c.difficulty.name(), c.skill_points
        );
        println!("  {}║  {}{}║{}", col, pad_visible(&row, 48), col, RESET);
    }
    // Faction reputation row
    {
        use chaos_rpg_core::factions::Faction;
        let fr = &c.faction_rep;
        let row = format!(
            "{}ORDER:{} {:>4}  {}CULT:{} {:>4}  {}WATCH:{} {:>4}",
            "\x1b[34m", RESET, fr.get(Faction::OrderOfConvergence),
            "\x1b[31m", RESET, fr.get(Faction::CultOfDivergence),
            "\x1b[35m", RESET, fr.get(Faction::WatchersOfBoundary),
        );
        println!("  {}║  {}{}║{}", col, pad_visible(&row, 48), col, RESET);
    }
    println!(
        "  {}╚══════════════════════════════════════════════════╝{}",
        col, RESET
    );
    // Body summary — always shown
    println!();
    println!("  {}Body:{}", DIM, RESET);
    println!("  {}", c.body.combat_summary());
    // Full body chart if any injuries
    let has_injuries = c.body.parts.values().any(|s| s.injury.is_some());
    if has_injuries {
        println!();
        for line in c.body.display_lines() {
            println!("  {}", line);
        }
    }
}

// ─── ENEMY DISPLAY ────────────────────────────────────────────────────────────

pub fn show_enemy(enemy: &chaos_rpg_core::enemy::Enemy) {
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
    let hp_col = if hp_pct > 0.6 {
        GREEN
    } else if hp_pct > 0.3 {
        YELLOW
    } else {
        RED
    };
    let bar_len = 24usize;
    let filled = ((hp_pct * bar_len as f64) as usize).min(bar_len);
    println!(
        "  {}[{}{}{}]",
        hp_col,
        "█".repeat(filled),
        "░".repeat(bar_len - filled),
        RESET,
    );
    if let Some(ability) = enemy.special_ability {
        println!("  {}[ABILITY] {}{}", YELLOW, ability, RESET);
    }
}

// ─── COMBAT MENU ──────────────────────────────────────────────────────────────

pub fn show_combat_menu(player: &Character, enemy: &chaos_rpg_core::enemy::Enemy, round: u32) {
    let c = t_primary();
    let tier_col = enemy.tier_color();
    let w = 56usize;
    let bar = "═".repeat(w - 2);

    println!();
    println!("  {}╔{}╗{}", c, bar, RESET);
    println!(
        "  {}║  {} Round {:<3} {}  Floor {}  {}{}{}║{}",
        c,
        BOLD,
        round,
        RESET,
        player.floor,
        DIM,
        player.difficulty.name(),
        RESET,
        RESET
    );
    println!("  {}╠{}╣{}", c, bar, RESET);

    // Enemy info block
    println!(
        "  {}║  {}[{}]{} {:<36}{}║{}",
        c,
        tier_col,
        enemy.tier.name(),
        RESET,
        &enemy.name[..enemy.name.len().min(36)],
        c,
        RESET
    );
    let hp_pct = enemy.hp as f64 / enemy.max_hp as f64;
    let e_col = if hp_pct > 0.6 {
        GREEN
    } else if hp_pct > 0.3 {
        YELLOW
    } else {
        RED
    };
    let efill = ((hp_pct * 20.0) as usize).min(20);
    println!(
        "  {}║  {}HP [{}{}{}{}]{} {}/{}{}║{}",
        c,
        e_col,
        "█".repeat(efill),
        "░".repeat(20 - efill),
        RESET,
        e_col,
        RESET,
        enemy.hp,
        enemy.max_hp,
        c,
        RESET
    );

    println!("  {}╠{}╣{}", c, bar, RESET);

    // Player info block
    println!(
        "  {}║  {}{}{} Lv.{} {} {}║{}",
        c,
        BOLD,
        player.name,
        RESET,
        player.level,
        player.class.name(),
        c,
        RESET
    );
    println!(
        "  {}║  {}  Gold: {}  Kills: {}{}║{}",
        c, DIM, player.gold, player.kills, c, RESET
    );

    let pfill = ((player.hp_percent() * 30.0) as usize).min(30);
    let p_col = if player.hp_percent() > 0.6 {
        GREEN
    } else if player.hp_percent() > 0.3 {
        YELLOW
    } else {
        RED
    };
    println!(
        "  {}║  {}HP [{}{}{}{}]{} {}/{}{}║{}",
        c,
        p_col,
        "█".repeat(pfill),
        "░".repeat(30 - pfill),
        RESET,
        p_col,
        RESET,
        player.current_hp,
        player.max_hp,
        c,
        RESET
    );

    // Status badges — use visible_len padding so ANSI codes don't break alignment
    let badges = player.status_badge_line();
    if !badges.is_empty() {
        // Box inner = w-4 = 52; "Status: " = 8; leaves 44 for badges
        let status_label = format!("Status: {}", badges);
        println!("  {}║  {}{}║{}", c, pad_visible(&status_label, 52), c, RESET);
    }
    // Body injury indicator in combat
    let body_summary = player.body.combat_summary();
    if !body_summary.is_empty() {
        let brow = format!("{}Body:{} {}", DIM, RESET, body_summary);
        println!("  {}║  {}{}║{}", c, pad_visible(&brow, 52), c, RESET);
    }

    println!("  {}╠{}╣{}", c, bar, RESET);

    // Equipment panel
    {
        use chaos_rpg_core::character::EquipSlot;
        let slots = [
            (EquipSlot::Weapon, "WPN"),
            (EquipSlot::Body,   "BOD"),
            (EquipSlot::Ring1,  "RNG"),
            (EquipSlot::Ring2,  "RN2"),
            (EquipSlot::Amulet, "AMU"),
        ];
        println!("  {}║  {}Equipped:{}", c, DIM, RESET);
        for (slot, label) in &slots {
            if let Some(item) = player.equipped.get(*slot) {
                // Durability bar (compact: 6 chars)
                let pct = item.durability as f64 / item.max_durability.max(1) as f64;
                let filled = (pct * 6.0) as usize;
                let dur_col = if pct > 0.75 { GREEN }
                    else if pct > 0.50 { YELLOW }
                    else if pct > 0.25 { "\x1b[91m" }  // orange
                    else { RED };
                let dur_bar = format!("{}[{}{}{}]{}",
                    dur_col,
                    "█".repeat(filled),
                    "░".repeat(6 - filled),
                    dur_col,
                    RESET
                );
                let name = &item.name[..item.name.len().min(22)];
                println!(
                    "  {}║    {}{}{} {} {}{:<22}{}{}║{}",
                    c, DIM, label, RESET, dur_bar,
                    item.rarity.color_code(), name, RESET, c, RESET
                );
            } else {
                println!(
                    "  {}║    {}{}{} {}[empty]{:<30}{}║{}",
                    c, DIM, label, RESET, DIM, "", RESET, RESET
                );
            }
        }
    }

    println!("  {}╠{}╣{}", c, bar, RESET);

    // Action menu
    println!(
        "  {}║  {}[A]{} Attack    {}[H]{} Heavy    {}[D]{} Defend    {}[F]{} Flee    {}║{}",
        c, GREEN, RESET, YELLOW, RESET, CYAN, RESET, RED, RESET, c, RESET
    );
    println!(
        "  {}║  {}[T]{} Taunt     {}[S]{} Spell    {}[I]{} Equip/Use {}[?]{} Trace  {}║{}",
        c, MAGENTA, RESET, BRIGHT_CYAN, RESET, GREEN, RESET, DIM, RESET, c, RESET
    );

    if !player.known_spells.is_empty() {
        println!("  {}║  {}Spells:{}", c, DIM, RESET);
        for (i, spell) in player.known_spells.iter().enumerate().take(4) {
            let name = &spell.name[..spell.name.len().min(28)];
            println!(
                "  {}║    [S{}] {}{:<32}{}║{}",
                c,
                i + 1,
                t_magic(),
                name,
                c,
                RESET
            );
        }
    }

    if !player.inventory.is_empty() {
        println!("  {}║  {}Inventory (I# to equip/use):{}", c, DIM, RESET);
        for (i, item) in player.inventory.iter().enumerate().take(4) {
            let name = &item.name[..item.name.len().min(22)];
            let tag = if item.equip_slot().is_some() { "[EQP]" } else { "[USE]" };
            println!(
                "  {}║    [I{}] {}{} {}{:<22}{}{}║{}",
                c,
                i + 1,
                DIM, tag, RESET,
                name,
                c,
                RESET,
                RESET
            );
        }
    }

    println!("  {}╚{}╝{}", c, bar, RESET);
    println!();
}

pub fn read_combat_action() -> chaos_rpg_core::combat::CombatAction {
    use chaos_rpg_core::combat::CombatAction;
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
            "?" | "help" => {
                println!();
                println!("  {}--- COMBAT ACTIONS ---{}", CYAN, RESET);
                println!("  {}[A]{} Attack      — standard attack, builds combo streak", GREEN, RESET);
                println!("  {}[H]{} Heavy       — consumes combo for big hit", YELLOW, RESET);
                println!("  {}[D]{} Defend      — reduce next hit by VIT/3 + FOR/5", CYAN, RESET);
                println!("  {}[T]{} Taunt       — draw enemy attack, boosts next strike", MAGENTA, RESET);
                println!("  {}[F]{} Flee        — attempt escape (LCK-based, leg injuries penalize)", RED, RESET);
                println!("  {}[S1-S9]{} Spell   — cast a known spell by number", BRIGHT_CYAN, RESET);
                println!("  {}[I1-I9]{} Item    — equip [EQP] items or consume [USE] items", GREEN, RESET);
                println!("  {}Note:{} equipped items wear down with use; repair at crafting bench", DIM, RESET);
                println!("  {}Note:{} engine trace shows automatically after each action.", DIM, RESET);
                println!();
            }
            _ => println!("  {}Unknown: a/h/d/t/f/s#/i#  (? for help){}", DIM, RESET),
        }
    }
}

pub fn display_combat_events(events: &[chaos_rpg_core::combat::CombatEvent]) {
    for event in events {
        use chaos_rpg_core::combat::CombatEvent;
        // Special rendering for new durability events
        match event {
            CombatEvent::ItemEquipped { name, slot } => {
                println!("  {}⚙  Equipped: {} → {} slot{}", GREEN, name, slot, RESET);
                continue;
            }
            CombatEvent::ItemDurabilityLost { name, durability, max_durability } => {
                let pct = *durability as f64 / (*max_durability).max(1) as f64;
                let col = if pct > 0.5 { YELLOW } else { RED };
                println!("  {}◈  {} durability {}/{}{}", col, name, durability, max_durability, RESET);
                continue;
            }
            CombatEvent::ItemDestroyed { name } => {
                println!("  {}💥 {} SHATTERED! Item lost forever.{}", RED, name, RESET);
                continue;
            }
            _ => {}
        }

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

// ─── FACTION REP DISPLAY ──────────────────────────────────────────────────────

/// Full equipment management screen (out-of-combat).
/// Shows 5 equip slots + inventory. Player can equip (e#) or unequip (u<slot>).
pub fn show_equipment_screen(player: &mut chaos_rpg_core::character::Character) {
    use chaos_rpg_core::character::EquipSlot;
    let c = t_primary();
    let w = 54usize;
    let bar = "═".repeat(w);
    loop {
        clear_screen();
        println!();
        println!("  {}╔{}╗{}", c, bar, RESET);
        println!("  {}║  ⚙  EQUIPMENT MANAGER{:<32}║{}", c, "", RESET);
        println!("  {}╠{}╣{}", c, bar, RESET);

        // Equipped slots
        let slots = [
            (EquipSlot::Weapon, "Weapon"),
            (EquipSlot::Body,   "Body  "),
            (EquipSlot::Ring1,  "Ring 1"),
            (EquipSlot::Ring2,  "Ring 2"),
            (EquipSlot::Amulet, "Amulet"),
        ];
        println!("  {}║  {}Equipped Items:{}{:<38}{}║{}", c, DIM, RESET, "", c, RESET);
        for (slot, label) in &slots {
            let slot_key = match slot {
                EquipSlot::Weapon => "w",
                EquipSlot::Body   => "b",
                EquipSlot::Ring1  => "r",
                EquipSlot::Ring2  => "n",
                EquipSlot::Amulet => "a",
            };
            if let Some(item) = player.equipped.get(*slot) {
                let pct = item.durability as f64 / item.max_durability.max(1) as f64;
                let filled = (pct * 8.0) as usize;
                let dur_col = if pct > 0.75 { GREEN }
                    else if pct > 0.50 { YELLOW }
                    else if pct > 0.25 { "\x1b[91m" }
                    else { RED };
                let bar_str = format!("{}[{}{}{}]{}",
                    dur_col, "█".repeat(filled), "░".repeat(8 - filled), dur_col, RESET);
                let name = &item.name[..item.name.len().min(20)];
                println!("  {}║  [u{}] {}{}{} {} {}{:<20}{}{}║{}",
                    c, slot_key, DIM, label, RESET, bar_str,
                    item.rarity.color_code(), name, RESET, c, RESET);
            } else {
                println!("  {}║  [--] {}{}{} {}[empty]{:<33}{}║{}",
                    c, DIM, label, RESET, DIM, "", RESET, RESET);
            }
        }
        println!("  {}╠{}╣{}", c, bar, RESET);

        // Inventory
        println!("  {}║  {}Inventory (e# to equip):{}{:<29}{}║{}", c, DIM, RESET, "", c, RESET);
        if player.inventory.is_empty() {
            println!("  {}║    {}(empty){:<44}{}║{}", c, DIM, "", RESET, RESET);
        }
        for (i, item) in player.inventory.iter().enumerate().take(8) {
            let tag = if item.equip_slot().is_some() { GREEN } else { DIM };
            let kind = if item.equip_slot().is_some() { "[EQP]" } else { "[USE]" };
            let pct = item.durability as f64 / item.max_durability.max(1) as f64;
            let dur_col = if pct > 0.75 { GREEN }
                else if pct > 0.50 { YELLOW }
                else if pct > 0.25 { "\x1b[91m" }
                else { RED };
            let dur_str = format!("{}{}/{}{}",
                dur_col, item.durability, item.max_durability, RESET);
            let name = &item.name[..item.name.len().min(18)];
            println!("  {}║  [e{}] {}{}{} {}{:<18}{} DUR:{}{}║{}",
                c, i + 1,
                tag, kind, RESET,
                item.rarity.color_code(), name, RESET,
                dur_str, c, RESET);
        }
        println!("  {}╠{}╣{}", c, bar, RESET);
        println!("  {}║  {}e# = equip item   u<w/b/r/n/a> = unequip slot{:<5}{}║{}",
            c, DIM, "", RESET, RESET);
        println!("  {}║  {}[Enter] to exit equipment screen{:<21}{}║{}",
            c, DIM, "", RESET, RESET);
        println!("  {}╚{}╝{}", c, bar, RESET);

        let input = prompt("  Equipment > ").to_lowercase();
        let input = input.trim();

        if input.is_empty() || input == "q" || input == "exit" {
            break;
        }

        if let Some(rest) = input.strip_prefix('e') {
            if let Ok(idx) = rest.parse::<usize>() {
                let real_idx = idx.saturating_sub(1);
                match player.equip_from_inventory(real_idx) {
                    Some(name) => println!("  {}Equipped: {}{}", GREEN, name, RESET),
                    None => println!("  {}Nothing to equip at slot {} (item may have no equip slot){}", DIM, idx, RESET),
                }
                press_enter(&format!("  {}[ENTER]...{}", DIM, RESET));
            }
        } else if let Some(rest) = input.strip_prefix('u') {
            let slot = match rest.trim() {
                "w" => Some(EquipSlot::Weapon),
                "b" => Some(EquipSlot::Body),
                "r" => Some(EquipSlot::Ring1),
                "n" => Some(EquipSlot::Ring2),
                "a" => Some(EquipSlot::Amulet),
                _   => None,
            };
            if let Some(s) = slot {
                if player.unequip_slot(s) {
                    println!("  {}Unequipped.{}", GREEN, RESET);
                } else {
                    println!("  {}Slot was already empty.{}", DIM, RESET);
                }
                press_enter(&format!("  {}[ENTER]...{}", DIM, RESET));
            }
        }
    }
}

pub fn show_faction_rep(player: &chaos_rpg_core::character::Character) {
    use chaos_rpg_core::factions::{Faction, vendor_greeting};
    let col = t_primary();
    let fr = &player.faction_rep;
    println!();
    println!("  {}╔══════════════════════════════════════════════════╗{}", col, RESET);
    println!("  {}║           FACTION REPUTATION                     ║{}", col, RESET);
    println!("  {}╠══════════════════════════════════════════════════╣{}", col, RESET);
    for faction in Faction::all() {
        let rep = fr.get(faction);
        let tier = fr.tier(faction);
        let fc = faction.color();
        let tc = tier.color();
        // Bar: -500 to +500 mapped to 20 cols
        let bar_pct = ((rep + 500) as f64 / 1000.0).clamp(0.0, 1.0);
        let filled = (bar_pct * 20.0) as usize;
        let bar = format!("[{}{}{}]",
            "\x1b[32m".to_owned() + &"█".repeat(filled),
            "\x1b[90m".to_owned() + &"░".repeat(20 - filled),
            RESET);
        println!("  {}║  {}{:<24}{} {}{}  {:>5}{} {}{}",
            col, fc, faction.name(), RESET,
            tc, tier.name(), RESET,
            rep, RESET, bar);
        println!("  {}║    {}\"{}\"{}",
            col, DIM, vendor_greeting(faction, tier), RESET);
        println!("  {}║{}", col, RESET);
    }
    println!("  {}╠══════════════════════════════════════════════════╣{}", col, RESET);
    println!("  {}║  {}Gaining rep with Order/Cult lowers the other.   {}║{}", col, DIM, col, RESET);
    println!("  {}║  {}Watchers are neutral -- no cross-penalty.       {}║{}", col, DIM, col, RESET);
    println!("  {}╚══════════════════════════════════════════════════╝{}", col, RESET);
}

// ─── FLOOR / ROOM DISPLAY ─────────────────────────────────────────────────────

pub fn show_floor_header(floor: u32, mode: &GameMode) {
    let mode_str = match mode {
        GameMode::Story => "Story",
        GameMode::DailySeed => "Daily",
        _ => "Infinite",
    };
    let c = t_primary();
    println!();
    println!("  {}╔══════════════════════════════╗{}", c, RESET);
    println!(
        "  {}║  Floor {:>3}  [{:<8}]       ║{}",
        c, floor, mode_str, RESET
    );
    println!("  {}╚══════════════════════════════╝{}", c, RESET);
    println!();
}

pub fn story_event(floor: u32, seed: u64) -> Option<String> {
    use chaos_rpg_core::chaos_pipeline::chaos_roll_verbose;
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
    println!(
        "  {}║   *** LEVEL UP! Now Level {:>3} ***  ║{}",
        c, level, RESET
    );
    println!("  {}║   {}{}{}{}║{}", c, DIM, msg, RESET, c, RESET);
    println!("  {}╚═══════════════════════════════════╝{}", c, RESET);
    println!();
}

pub fn show_victory(player: &Character) {
    let c = t_warning();
    println!();
    println!(
        "  {}╔══════════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║                                              ║{}",
        c, RESET
    );
    println!(
        "  {}║     *** VICTORY — THE MATH YIELDS ***        ║{}",
        c, RESET
    );
    println!(
        "  {}║                                              ║{}",
        c, RESET
    );
    println!(
        "  {}║  {}{} has transcended the abyss!{}{}          ║{}",
        c, BOLD, player.name, RESET, c, RESET
    );
    println!(
        "  {}║  Score: {}{}{:<8}{}{}                         ║{}",
        c,
        t_magic(),
        BOLD,
        player.score(),
        RESET,
        c,
        RESET
    );
    println!(
        "  {}║                                              ║{}",
        c, RESET
    );
    println!(
        "  {}╚══════════════════════════════════════════════╝{}",
        c, RESET
    );
    println!();
}

pub fn show_game_over(player: &Character) {
    let c = t_danger();
    println!();
    println!(
        "  {}╔══════════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║                                              ║{}",
        c, RESET
    );
    println!(
        "  {}║          *** GAME OVER ***                   ║{}",
        c, RESET
    );
    println!(
        "  {}║                                              ║{}",
        c, RESET
    );
    println!(
        "  {}║  {}{}{} fell on Floor {}  Level {}{}{}          ║{}",
        c, BOLD, player.name, RESET, player.floor, player.level, c, RESET, RESET
    );
    println!(
        "  {}║  Final Score: {}{}{:<10}{}{}                  ║{}",
        c,
        t_warning(),
        BOLD,
        player.score(),
        RESET,
        c,
        RESET
    );
    println!(
        "  {}║                                              ║{}",
        c, RESET
    );
    println!(
        "  {}╚══════════════════════════════════════════════╝{}",
        c, RESET
    );
    println!();
}

// ─── SCOREBOARD ───────────────────────────────────────────────────────────────

pub fn show_scoreboard(scores: &[chaos_rpg_core::scoreboard::ScoreEntry]) {
    use chaos_rpg_core::scoreboard::load_misery_scores;
    clear_screen();
    let c = t_warning();
    println!();
    println!(
        "  {}╔══════════════════════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║                  HALL OF CHAOS — TOP SCORES              ║{}",
        c, RESET
    );
    println!(
        "  {}╠══════╦════════════════╦════════════╦═══════╦═══════╦═════╣{}",
        c, RESET
    );
    println!(
        "  {}║ {:>4} ║ {:<14} ║ {:<10} ║ {:>5} ║ {:>5} ║ Date  ║{}",
        c, "#", "Name", "Class", "Score", "Floor", RESET
    );
    println!(
        "  {}╠══════╬════════════════╬════════════╬═══════╬═══════╬═════╣{}",
        c, RESET
    );
    if scores.is_empty() {
        println!(
            "  {}║  No scores yet. The void awaits your sacrifice.          ║{}",
            DIM, RESET
        );
    } else {
        for (i, s) in scores.iter().enumerate().take(15) {
            let row_col = if i == 0 {
                YELLOW
            } else if i < 3 {
                CYAN
            } else {
                WHITE
            };
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
    println!(
        "  {}╚══════╩════════════════╩════════════╩═══════╩═══════╩═════╝{}",
        c, RESET
    );

    // ── Hall of Misery ──
    println!();
    let m = t_magic();
    println!(
        "  {}╔══════════════════════════════════════════════════════════╗{}",
        RED, RESET
    );
    println!(
        "  {}║                   HALL OF MISERY                         ║{}",
        RED, RESET
    );
    println!(
        "  {}╠══════╦════════╦════════════════╦════════════╦═════╦══════╣{}",
        RED, RESET
    );
    println!(
        "  {}║ {:>4} ║ {:>6} ║ {:<14} ║ {:<10} ║ {:>3} ║ {:<4} ║{}",
        RED, "#", "Misery", "Name", "Class", "Flr", "Tier", RESET
    );
    println!(
        "  {}╠══════╬════════╬════════════════╬════════════╬═════╬══════╣{}",
        RED, RESET
    );
    let mscores = load_misery_scores();
    if mscores.is_empty() {
        println!(
            "  {}║  No misery recorded. Suffer more.                        ║{}",
            DIM, RESET
        );
    } else {
        for (i, ms) in mscores.iter().enumerate().take(10) {
            let row_col = if i == 0 { RED } else if i < 3 { MAGENTA } else { DIM };
            let tier_short: String = ms.power_tier.chars().take(4).collect();
            println!(
                "  {}║ {:>4} ║ {:>6.0} ║ {:<14} ║ {:<10} ║ {:>3} ║ {:<4} ║{}",
                row_col,
                i + 1,
                ms.misery_index,
                &ms.name[..ms.name.len().min(14)],
                &ms.class[..ms.class.len().min(10)],
                ms.floor_reached,
                tier_short,
                RESET
            );
        }
    }
    println!(
        "  {}╚══════╩════════╩════════════════╩════════════╩═════╩══════╝{}",
        RED, RESET
    );
    let _ = m;
    println!();
    press_enter(&format!("  {}[ENTER] to return...{}", DIM, RESET));
}

// ─── HELP ─────────────────────────────────────────────────────────────────────

pub fn show_help() {
    clear_screen();
    let c = t_primary();
    println!();
    println!(
        "  {}╔══════════════════════════════════════════════════════════╗{}",
        c, RESET
    );
    println!(
        "  {}║              CHAOS RPG — HOW TO PLAY                     ║{}",
        c, RESET
    );
    println!(
        "  {}╚══════════════════════════════════════════════════════════╝{}",
        c, RESET
    );
    println!();
    println!("  {}THE 10 SACRED ALGORITHMS:{}", t_warning(), RESET);
    println!(
        "  {}Lorenz{}     · Butterfly effect chaos attractor",
        CYAN, RESET
    );
    println!(
        "  {}Fourier{}    · Harmonic decomposition of fate",
        CYAN, RESET
    );
    println!("  {}Primes{}     · Density sieve of fortune", CYAN, RESET);
    println!("  {}Riemann{}    · Zeta function partial sums", CYAN, RESET);
    println!("  {}Fibonacci{}  · Golden spiral trajectory", CYAN, RESET);
    println!(
        "  {}Mandelbrot{} · Escape velocity (inside = cursed)",
        CYAN, RESET
    );
    println!(
        "  {}Logistic{}   · r=3.9 chaos regime bifurcation",
        CYAN, RESET
    );
    println!("  {}Euler{}      · Totient ratio irregularity", CYAN, RESET);
    println!("  {}Collatz{}    · 3n+1 stopping time", CYAN, RESET);
    println!(
        "  {}ModExp{}     · Modular exponentiation hash",
        CYAN, RESET
    );
    println!();
    println!(
        "  {}STATS (all unbounded — can go negative):{}",
        t_warning(),
        RESET
    );
    println!("  VIT=HP  FOR=Damage  MAN=Magic  CUN=Crit");
    println!("  PRC=Accuracy  ENT=Chaos bonus  LCK=Fortune");
    println!();
    println!("  {}COMBAT:{}", t_warning(), RESET);
    println!("  [A] Attack   [H] Heavy   [D] Defend   [T] Taunt");
    println!("  [F] Flee     [S#] Spell  [I#] Item    [?] Show last trace");
    println!();
    println!("  {}CLASSES:{}", t_warning(), RESET);
    for class in &[
        CharacterClass::Mage,
        CharacterClass::Berserker,
        CharacterClass::Ranger,
        CharacterClass::Thief,
        CharacterClass::Necromancer,
        CharacterClass::Alchemist,
        CharacterClass::Paladin,
        CharacterClass::VoidWalker,
    ] {
        println!(
            "  {}{:<12}{} {}Passive:{} {}",
            t_primary(),
            class.name(),
            RESET,
            DIM,
            RESET,
            class.passive_desc()
        );
    }
    println!();
    press_enter(&format!("  {}[ENTER] to return...{}", DIM, RESET));
}

// ─── PASSIVE TREE UI ──────────────────────────────────────────────────────────

/// Interactive passive skill tree allocation. Shows available nodes and lets
/// the player type a node ID to allocate it. Applies stat bonuses immediately.
pub fn show_passive_tree_ui(player: &mut Character, seed: u64) {
    use chaos_rpg_core::passive_tree::{nodes, NodeType, PlayerPassives};

    // Build PlayerPassives from character state.
    let mut passives = PlayerPassives {
        allocated: player.allocated_nodes.iter().map(|&id| id as u16).collect(),
        stat_bonuses: std::collections::HashMap::new(),
        points: player.skill_points,
        keystones: std::collections::HashSet::new(),
        completed_synergies: std::collections::HashSet::new(),
        cursor: player
            .allocated_nodes
            .first()
            .map(|&id| id as u16)
            .unwrap_or(0),
    };

    loop {
        clear_screen();
        for line in passives.display_map(player.class) {
            println!("{}", line);
        }
        println!();
        println!(
            "  {}Skill Points remaining: {}{}{}",
            DIM,
            t_warning(),
            passives.points,
            RESET
        );
        println!();

        // Show reachable unallocated nodes.
        let reachable: Vec<_> = nodes()
            .iter()
            .filter(|n| !passives.allocated.contains(&n.id) && passives.can_allocate(n.id))
            .collect();

        if reachable.is_empty() || passives.points == 0 {
            if passives.points == 0 {
                println!("  {}No skill points remaining.{}", DIM, RESET);
            } else {
                println!(
                    "  {}No reachable nodes from current allocation.{}",
                    DIM, RESET
                );
            }
            press_enter(&format!("  {}[ENTER] to continue...{}", DIM, RESET));
            break;
        }

        println!("  {}Available nodes:{}", t_primary(), RESET);
        for node in &reachable {
            let type_tag = match &node.node_type {
                NodeType::Stat { stat, min, max } => {
                    format!("[STAT] {} ({}-{})", stat.to_uppercase(), min, max)
                }
                NodeType::Engine { engine, .. } => format!("[ENGINE] {}", engine),
                NodeType::Keystone { id } => format!("[KEYSTONE] {}", id),
                NodeType::Synergy { cluster, .. } => format!("[SYNERGY] cluster {}", cluster),
                NodeType::Notable { stat, bonus, .. } => {
                    format!("[NOTABLE] +{} {}", bonus, stat.to_uppercase())
                }
            };
            println!(
                "  {}{:>3}{} {} — {} {}{}{}",
                t_warning(),
                node.id,
                RESET,
                node.name,
                node.short_desc,
                DIM,
                type_tag,
                RESET
            );
        }
        println!();
        println!("  {}[0] Done  [A] Auto-allocate all{}", DIM, RESET);
        println!();

        let input = prompt("  Node ID > ");
        let trimmed = input.trim();
        if trimmed == "0" || trimmed.eq_ignore_ascii_case("q") {
            break;
        }

        // Auto-allocate: spend all points using the class priority heuristic
        if trimmed.eq_ignore_ascii_case("a") {
            let prev_bonuses = passives.stat_bonuses.clone();
            let msgs = passives.auto_allocate_all(player.class, seed);
            if msgs.is_empty() {
                println!("  {}Nothing to allocate.{}", DIM, RESET);
            } else {
                for msg in &msgs {
                    println!("  {}{}{}", t_success(), msg, RESET);
                }
                // Apply all newly gained stat bonuses
                for (&nid, &value) in &passives.stat_bonuses {
                    if !prev_bonuses.contains_key(&nid) {
                        if let Some(node) = nodes().iter().find(|n| n.id == nid) {
                            if let NodeType::Stat { stat, .. } | NodeType::Notable { stat, .. } = &node.node_type {
                                passive_apply_stat(player, stat, value);
                            }
                        }
                    }
                }
            }
            press_enter(&format!("  {}[ENTER]...{}", DIM, RESET));
            continue;
        }

        if let Ok(node_id) = trimmed.parse::<u16>() {
            let prev_bonuses = passives.stat_bonuses.clone();
            if let Some(msg) = passives.allocate(node_id, seed.wrapping_add(node_id as u64 * 777)) {
                println!("  {}{}{}", t_success(), msg, RESET);
                // Apply any newly locked-in stat bonuses to the character.
                for (&nid, &value) in &passives.stat_bonuses {
                    if !prev_bonuses.contains_key(&nid) {
                        if let Some(node) = nodes().iter().find(|n| n.id == nid) {
                            if let NodeType::Stat { stat, .. } = &node.node_type {
                                passive_apply_stat(player, stat, value);
                            }
                        }
                    }
                }
                press_enter(&format!("  {}[ENTER]...{}", DIM, RESET));
            } else {
                println!(
                    "  {}Cannot allocate node {} — check requirements.{}",
                    t_danger(),
                    node_id,
                    RESET
                );
                press_enter(&format!("  {}[ENTER]...{}", DIM, RESET));
            }
        }
    }

    // Write back updated state.
    player.allocated_nodes = passives.allocated.into_iter().map(|id| id as u32).collect();
    player.skill_points = passives.points;
}

fn passive_apply_stat(player: &mut Character, stat: &str, value: i64) {
    match stat {
        "vitality" => {
            player.stats.vitality += value;
            player.max_hp = (50 + player.stats.vitality * 3 + player.stats.force).max(1);
        }
        "force" => {
            player.stats.force += value;
            player.max_hp = (50 + player.stats.vitality * 3 + player.stats.force).max(1);
        }
        "mana" => player.stats.mana += value,
        "cunning" => player.stats.cunning += value,
        "precision" => player.stats.precision += value,
        "entropy" => player.stats.entropy += value,
        "luck" => player.stats.luck += value,
        _ => {}
    }
}

// ─── BESTIARY SCREEN ──────────────────────────────────────────────────────────

pub fn show_bestiary() {
    use chaos_rpg_core::player_bestiary::PlayerBestiary;
    use chaos_rpg_core::lore::enemies::{enemy_lore, generic_enemy_lore};
    use chaos_rpg_core::lore::bosses::boss_lore;

    clear_screen();
    let c = t_primary();
    let bestiary = PlayerBestiary::load();
    let records = bestiary.sorted_for_display();

    println!();
    println!("  {}╔══════════════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║  BESTIARY — ENTITIES OF THE PROOF                    ║{}", c, RESET);
    println!("  {}╠══════════════════════════════════════════════════════╣{}", c, RESET);
    println!(
        "  {}║{}  Encountered: {:<5}  Killed: {:<5}                        {}║{}",
        c, RESET,
        bestiary.total_encountered(),
        bestiary.total_killed(),
        c, RESET
    );
    println!("  {}╚══════════════════════════════════════════════════════╝{}", c, RESET);
    println!();

    if records.is_empty() {
        println!("  {}No entities encountered yet. Enter The Proof.{}", DIM, RESET);
        println!();
        press_enter(&format!("  {}[ENTER] Return{}", DIM, RESET));
        return;
    }

    // Paginate through entries
    let per_page = 8usize;
    let mut page = 0usize;
    let total_pages = (records.len() + per_page - 1) / per_page;

    loop {
        clear_screen();
        println!();
        println!("  {}BESTIARY{}  — Page {}/{}", c, RESET, page + 1, total_pages);
        println!();

        let start = page * per_page;
        let end = (start + per_page).min(records.len());

        for rec in &records[start..end] {
            let boss_marker = if rec.is_boss { format!("{}[BOSS]{} ", MAGENTA, RESET) } else { String::new() };
            let lore_marker = if rec.lore_unlocked { "" } else { " [LOCKED]" };
            println!(
                "  {}{}{}{} — fought: {}  killed: {}  died to: {}{}",
                boss_marker,
                YELLOW,
                rec.name,
                RESET,
                rec.times_fought,
                rec.times_killed,
                rec.times_killed_player,
                lore_marker,
            );
            println!(
                "      HP range: {}  Damage range: {}",
                rec.hp_range_display(),
                rec.damage_range_display(),
            );

            if rec.lore_unlocked {
                let lore_text = if rec.is_boss {
                    boss_lore(&rec.name)
                        .map(|b| b.full_entry)
                        .unwrap_or("A boss that defies classification.")
                } else {
                    enemy_lore(&rec.name)
                        .map(|e| e.description)
                        .unwrap_or_else(|| {
                            // Use a seed derived from the name for consistent generic lore
                            let seed: u64 = rec.name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                            generic_enemy_lore(seed)
                        })
                };
                println!("      {}{}{}",DIM, lore_text, RESET);

                if rec.is_boss && rec.strategy_unlocked {
                    if let Some(boss) = boss_lore(&rec.name) {
                        println!("      {}STRATEGY: {}{}", CYAN, boss.strategy_hint, RESET);
                    }
                }
            }
            println!();
        }

        println!("  {}[N]ext  [P]rev  [Q]uit{}", DIM, RESET);
        let input = prompt("  >");
        match input.to_uppercase().as_str() {
            "N" if page + 1 < total_pages => page += 1,
            "P" if page > 0 => page -= 1,
            "Q" | "" => break,
            _ => {}
        }
    }
}

// ─── CODEX SCREEN ─────────────────────────────────────────────────────────────

pub fn show_codex() {
    use chaos_rpg_core::codex_progress::CodexProgress;
    use chaos_rpg_core::lore::codex::{CodexCategory, CODEX_ENTRIES};
    use chaos_rpg_core::lore::fragments::FRAGMENTS;

    clear_screen();
    let c = t_primary();
    let progress = CodexProgress::load();

    println!();
    println!("  {}╔══════════════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║  CODEX — KNOWLEDGE OF THE PROOF                      ║{}", c, RESET);
    println!("  {}╠══════════════════════════════════════════════════════╣{}", c, RESET);
    println!(
        "  {}║{}  Entries: {}/{}  Fragments: {}/{}                          {}║{}",
        c, RESET,
        progress.unlocked_count(),
        progress.total_count(),
        progress.fragment_unlocked_count(),
        progress.total_fragment_count(),
        c, RESET
    );
    println!("  {}╚══════════════════════════════════════════════════════╝{}", c, RESET);
    println!();

    let categories = [
        CodexCategory::TheProof,
        CodexCategory::TheEpochs,
        CodexCategory::TheEngines,
        CodexCategory::TheFactions,
        CodexCategory::TheMathematician,
        CodexCategory::Materials,
        CodexCategory::Phenomena,
        CodexCategory::Theories,
    ];

    // Category picker
    loop {
        clear_screen();
        println!();
        println!("  {}CODEX — SELECT CATEGORY{}", c, RESET);
        println!();
        for (i, cat) in categories.iter().enumerate() {
            let unlocked = progress.unlocked_in_category(*cat).len();
            let total = CODEX_ENTRIES.iter().filter(|e| e.category == *cat).count();
            println!(
                "  {}[{}]{} {} — {}/{}",
                YELLOW, i + 1, RESET,
                cat.display_name(),
                unlocked,
                total
            );
        }
        let frag_count = progress.fragment_unlocked_count();
        println!(
            "  {}[F]{} Mathematician's Fragments — {}/{}",
            MAGENTA, RESET,
            frag_count,
            FRAGMENTS.len()
        );
        println!();
        println!("  {}[Q] Back{}", DIM, RESET);

        let input = prompt("  >");
        match input.to_uppercase().as_str() {
            "Q" | "" => return,
            "F" => show_codex_fragments(&progress),
            s => {
                if let Ok(n) = s.parse::<usize>() {
                    if n >= 1 && n <= categories.len() {
                        show_codex_category(&progress, categories[n - 1]);
                    }
                }
            }
        }
    }
}

fn show_codex_category(
    progress: &chaos_rpg_core::codex_progress::CodexProgress,
    cat: chaos_rpg_core::lore::codex::CodexCategory,
) {
    let c = t_primary();
    let unlocked = progress.unlocked_in_category(cat);
    let locked = progress.locked_in_category(cat);

    let per_page = 3usize;
    let all_entries: Vec<_> = unlocked.iter().map(|e| (true, *e))
        .chain(locked.iter().map(|e| (false, *e)))
        .collect();
    let total_pages = (all_entries.len() + per_page - 1).max(1) / per_page;
    let mut page = 0usize;

    loop {
        clear_screen();
        println!();
        println!("  {}CODEX — {}{}  (Page {}/{})", c, cat.display_name(), RESET, page + 1, total_pages);
        println!();

        if all_entries.is_empty() {
            println!("  {}No entries in this category yet.{}", DIM, RESET);
        } else {
            let start = page * per_page;
            let end = (start + per_page).min(all_entries.len());
            for (is_unlocked, entry) in &all_entries[start..end] {
                if *is_unlocked {
                    println!("  {}{}{}",YELLOW, entry.title, RESET);
                    println!();
                    // Word-wrap the body at ~70 chars
                    for line in word_wrap(entry.body, 68) {
                        println!("  {}", line);
                    }
                } else {
                    println!("  {}{}  [???]{}", DIM, entry.title, RESET);
                    println!("  {}Unlock: {}{}", DIM, entry.unlock_hint, RESET);
                }
                println!();
            }
        }

        println!("  {}[N]ext  [P]rev  [Q]uit{}", DIM, RESET);
        let input = prompt("  >");
        match input.to_uppercase().as_str() {
            "N" if page + 1 < total_pages => page += 1,
            "P" if page > 0 => page -= 1,
            "Q" | "" => break,
            _ => {}
        }
    }
}

fn show_codex_fragments(progress: &chaos_rpg_core::codex_progress::CodexProgress) {
    let c = t_primary();
    let unlocked = progress.unlocked_fragments_sorted();
    let total = chaos_rpg_core::lore::fragments::FRAGMENTS.len();

    clear_screen();
    println!();
    println!("  {}THE MATHEMATICIAN'S FRAGMENTS — {}/{}{}",MAGENTA, unlocked.len(), total, RESET);
    println!();

    if unlocked.is_empty() {
        println!("  {}No fragments found. They are waiting at the edges of The Proof.{}", DIM, RESET);
    } else {
        for frag in &unlocked {
            println!("  {}{}{}",YELLOW, frag.title, RESET);
            println!();
            for line in word_wrap(frag.text, 68) {
                println!("  {}", line);
            }
            println!("  {}[Unlocked by: {}]{}", DIM, frag.unlock_condition, RESET);
            println!();
        }
        // Show locked hints
        for frag in chaos_rpg_core::lore::fragments::FRAGMENTS {
            if !progress.fragment_unlocked(frag.id) {
                println!("  {}Fragment {} — [???]  Unlock: {}{}", DIM, frag.id, frag.unlock_condition, RESET);
            }
        }
    }
    println!();
    press_enter(&format!("  {}[ENTER] Back{}", DIM, RESET));
}

// ─── ACHIEVEMENTS SCREEN ──────────────────────────────────────────────────────

pub fn show_achievements() {
    use chaos_rpg_core::achievements::{AchievementRarity, AchievementStore};

    let store = AchievementStore::load();
    let all = &store.achievements;

    // 0 = all, 1 = unlocked, 2 = locked
    let mut filter: u8 = 0;
    let mut page: usize = 0;
    let per_page = 12usize;

    loop {
        let filtered: Vec<&chaos_rpg_core::achievements::Achievement> = all
            .iter()
            .filter(|a| match filter {
                1 => a.unlocked,
                2 => !a.unlocked,
                _ => true,
            })
            .collect();

        // Sort: unlocked first (by rarity desc), then locked (by rarity desc)
        let rarity_order = |r: &AchievementRarity| match r {
            AchievementRarity::Omega     => 6,
            AchievementRarity::Mythic    => 5,
            AchievementRarity::Legendary => 4,
            AchievementRarity::Epic      => 3,
            AchievementRarity::Rare      => 2,
            AchievementRarity::Uncommon  => 1,
            AchievementRarity::Common    => 0,
        };
        let mut sorted: Vec<&chaos_rpg_core::achievements::Achievement> = filtered.clone();
        sorted.sort_by(|a, b| {
            match (a.unlocked, b.unlocked) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => rarity_order(&b.rarity).cmp(&rarity_order(&a.rarity)),
            }
        });

        let total = sorted.len();
        let total_pages = ((total + per_page - 1) / per_page).max(1);
        if page >= total_pages { page = total_pages - 1; }

        let unlocked_count = all.iter().filter(|a| a.unlocked).count();
        let total_count = all.len();

        clear_screen();
        let c = t_primary();
        println!();
        println!("  {}╔══════════════════════════════════════════════════════╗{}", c, RESET);
        println!("  {}║  ACHIEVEMENTS                                        ║{}", c, RESET);
        println!("  {}╠══════════════════════════════════════════════════════╣{}", c, RESET);

        // Progress bar (20 chars wide)
        let filled = if total_count > 0 { unlocked_count * 20 / total_count } else { 0 };
        let bar: String = (0..20).map(|i| if i < filled { '█' } else { '░' }).collect();
        println!(
            "  {}║{}  Progress: {}{}{}  {}/{} unlocked{}  Page {}/{}{}  {}║{}",
            c, RESET,
            BRIGHT_GREEN, bar, RESET,
            unlocked_count, total_count,
            RESET,
            page + 1, total_pages,
            RESET,
            c, RESET
        );

        // Filter tabs
        let (fa, fu, fl) = match filter {
            1 => (DIM, BRIGHT_CYAN, DIM),
            2 => (DIM, DIM, BRIGHT_CYAN),
            _ => (BRIGHT_CYAN, DIM, DIM),
        };
        println!(
            "  {}║{}  Filter: {}[A]ll{}  {}[U]nlocked{}  {}[L]ocked{}                      {}║{}",
            c, RESET, fa, RESET, fu, RESET, fl, RESET, c, RESET
        );
        println!("  {}╚══════════════════════════════════════════════════════╝{}", c, RESET);
        println!();

        if sorted.is_empty() {
            println!("  {}Nothing to show with this filter.{}", DIM, RESET);
        } else {
            let start = page * per_page;
            let end = (start + per_page).min(sorted.len());
            for ach in &sorted[start..end] {
                if ach.unlocked {
                    let (rarity_color, badge) = match ach.rarity {
                        AchievementRarity::Common    => (WHITE,          "CMN"),
                        AchievementRarity::Uncommon  => (GREEN,          "UNC"),
                        AchievementRarity::Rare      => (CYAN,           "RAR"),
                        AchievementRarity::Epic      => (MAGENTA,        "EPC"),
                        AchievementRarity::Legendary => (YELLOW,         "LEG"),
                        AchievementRarity::Mythic    => (BRIGHT_MAGENTA, "MYT"),
                        AchievementRarity::Omega     => (BRIGHT_RED,     " Ω "),
                    };
                    println!(
                        "  {}★{} {}[{}]{} {}{}{}",
                        YELLOW, RESET,
                        rarity_color, badge, RESET,
                        BOLD, ach.name, RESET
                    );
                    println!("      {}{}{}", DIM, ach.description, RESET);
                    if !ach.unlock_date.is_empty() {
                        println!("      {}Unlocked: {}{}", DIM, ach.unlock_date, RESET);
                    }
                } else {
                    println!(
                        "  {}○{} {}[???]{} {}{}{}",
                        DIM, RESET,
                        DIM, RESET,
                        DIM, ach.name, RESET
                    );
                    println!("      {}Locked — complete the challenge to reveal...{}", DIM, RESET);
                }
                println!();
            }
        }

        println!(
            "  {}[N]ext  [P]rev  [A]ll/[U]nlocked/[L]ocked  [Q]uit{}",
            DIM, RESET
        );
        let input = prompt("  >");
        match input.to_uppercase().as_str() {
            "N" if page + 1 < total_pages => page += 1,
            "P" if page > 0 => page -= 1,
            "A" => { filter = 0; page = 0; }
            "U" => { filter = 1; page = 0; }
            "L" => { filter = 2; page = 0; }
            "Q" | "" => break,
            _ => {}
        }
    }
}

// ─── LORE EDITOR SCREEN ───────────────────────────────────────────────────────

/// Opens the lore editor for the given character. Returns the updated CharacterLore.
pub fn show_lore_editor(player: &Character) -> chaos_rpg_core::character_lore::CharacterLore {
    use chaos_rpg_core::character_lore::{CharacterLore, LoreEditorState, LoreField};
    use crossterm::event::{self, Event, KeyCode, KeyModifiers};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

    let mut state = LoreEditorState::new(player.character_lore.clone());

    enable_raw_mode().ok();

    loop {
        // Render
        let _ = execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
        let c = t_primary();
        println!();
        println!("  {}╔══════════════════════════════════════════════════════╗{}", c, RESET);
        println!("  {}║  LORE EDITOR — {:<38}║{}", c, format!("{} the {}", player.name, player.class.name()), RESET);
        println!("  {}╠══════════════════════════════════════════════════════╣{}", c, RESET);
        println!("  {}║  Tab/Shift-Tab: switch field  Esc: save & exit       ║{}", c, RESET);
        println!("  {}╚══════════════════════════════════════════════════════╝{}", c, RESET);
        println!();

        for field in LoreField::ALL {
            let is_active = *field == state.active_field;
            let label_color = if is_active { YELLOW } else { DIM };
            let text = field.get(&state.lore);
            let char_count = text.chars().count();
            let max = field.max_len();
            println!(
                "  {}{} ({}/{}){}",
                label_color, field.label(), char_count, max, RESET
            );
            if is_active {
                println!("  {}{}{}▌{}", CYAN, text, BRIGHT_CYAN, RESET);
            } else {
                let display = if text.is_empty() {
                    format!("{}(empty){}", DIM, RESET)
                } else {
                    let truncated: String = text.chars().take(70).collect();
                    let ellipsis = if text.chars().count() > 70 { "…" } else { "" };
                    format!("{}{}{}", DIM, truncated + ellipsis, RESET)
                };
                println!("  {}", display);
            }
            println!("  {}{}{}",DIM, field.hint(), RESET);
            println!();
        }

        let _ = io::stdout().flush();

        // Input handling
        if let Ok(Event::Key(key)) = event::read() {
            match (key.code, key.modifiers) {
                (KeyCode::Esc, _) => break,
                (KeyCode::Tab, KeyModifiers::SHIFT) => state.prev_field(),
                (KeyCode::Tab, _) => state.next_field(),
                (KeyCode::BackTab, _) => state.prev_field(),
                (KeyCode::Backspace, _) => state.pop_char(),
                (KeyCode::Char(ch), _) => {
                    if ch == '\t' {
                        state.next_field();
                    } else {
                        state.push_char(ch);
                    }
                }
                (KeyCode::Enter, _) => state.push_char('\n'),
                _ => {}
            }
        }
    }

    disable_raw_mode().ok();
    state.lore.clamp_lengths();
    state.lore
}

/// Display player-authored lore below the character sheet.
pub fn show_character_lore_section(lore: &chaos_rpg_core::character_lore::CharacterLore) {
    if lore.is_empty() {
        return;
    }
    let c = t_primary();
    println!("  {}── Character Lore ──────────────────────────────────{}", DIM, RESET);
    if !lore.origin.is_empty() {
        println!("  {}Origin:{} {}", c, RESET, lore.origin);
    }
    if !lore.motivation.is_empty() {
        println!("  {}Motivation:{} {}", c, RESET, lore.motivation);
    }
    if !lore.personality.is_empty() {
        println!("  {}Personality:{} {}", c, RESET, lore.personality);
    }
    if !lore.notes.is_empty() {
        println!("  {}Notes:{} {}", c, RESET, lore.notes);
    }
    println!();
}

/// Show auto-generated run narrative.
pub fn show_run_narrative(narrative: &str) {
    if narrative.is_empty() {
        return;
    }
    let c = t_primary();
    println!();
    println!("  {}╔══════════════════════════════════════════════════════╗{}", c, RESET);
    println!("  {}║  RUN NARRATIVE                                        ║{}", c, RESET);
    println!("  {}╚══════════════════════════════════════════════════════╝{}", c, RESET);
    println!();
    for para in narrative.split("\n\n") {
        for line in word_wrap(para, 70) {
            println!("  {}", line);
        }
        println!();
    }
}

/// Word-wrap helper — splits text at max_width columns.
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let mut current = String::new();
        for word in words {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= max_width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current.clone());
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
        if paragraph.is_empty() {
            lines.push(String::new());
        }
    }
    lines
}
