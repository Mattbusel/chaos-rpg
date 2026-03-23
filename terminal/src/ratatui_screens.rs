//! Ratatui-powered screen rendering for CHAOS RPG.
//!
//! This module provides beautiful, fully layouted terminal screens using ratatui.
//! The combat screen, character sheet, engine trace, floor navigation, and title
//! screen are all implemented here as full-frame renders.
//!
//! # Architecture
//!
//! Each public `draw_*` function takes a ratatui `Frame` and the relevant game
//! state, and renders the complete screen. These functions are called from the
//! game loop in main.rs when a full ratatui render is wanted (combat, char sheet).
//! The rest of the game (room text, shop, etc.) stays as imperative println! output.

use chaos_rpg_core::{
    character::{Character, CharacterClass, ColorTheme, PowerTier, StatusEffect},
    chaos_pipeline::ChaosRollResult,
    enemy::Enemy,
    scoreboard::ScoreEntry,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        block::Title, Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Sparkline, Table, Row, Cell, Wrap,
    },
    Frame,
};
use std::collections::VecDeque;

// ─── UNICODE ART CONSTANTS ────────────────────────────────────────────────────

/// Class portraits — displayed on class select and character sheet.
pub const PORTRAIT_MAGE: &[&str] = &[
    "      ★  ✦  ★     ",
    "   ╭──────────╮   ",
    "   │  ╔════╗  │   ",
    "   │  ║ ◉◉ ║  │   ",
    "   │  ║  ∆ ║  │   ",
    "   │  ╚════╝  │   ",
    "   ╰──┤ ▓▓▓ ├─╯   ",
    "      │ ▒▒▒ │      ",
    "    ╔═╧═════╧═╗   ",
    "    ║ ░░░░░░░ ║   ",
    "    ║ ░∑∫∂Δπ░ ║   ",
    "    ╚══╤═══╤══╝   ",
    "       ╰───╯      ",
    "     ╱▔▔▔▔▔▔▔╲    ",
    "    ╱  ⚡ ⚡   ╲   ",
    "   ╱_____________╲ ",
];

pub const PORTRAIT_BERSERKER: &[&str] = &[
    "  ╔══╗  ╔══╗       ",
    "  ║▓▓║  ║▓▓║       ",
    "  ╚══╬══╬══╝       ",
    "   ╔═╩══╩═╗        ",
    "   ║ ■■■■ ║        ",
    "   ║ ◉  ◉ ║        ",
    "   ║  ▄▄  ║        ",
    "   ╚══════╝        ",
    " ▓▓▓╔══════╗▓▓▓   ",
    " ▓▓▓║██████║▓▓▓   ",
    " ▓▓▓║██████║▓▓▓   ",
    " ▓▓▓╚══╤╤══╝▓▓▓   ",
    "    ╔══╪╪══╗       ",
    "    ║  ││  ║       ",
    "    ╚══╧╧══╝       ",
    " ⚡ RAGE ⚡ RAGE ⚡ ",
];

pub const PORTRAIT_RANGER: &[&str] = &[
    "     ┌────────┐    ",
    "     │  ·  ·  │    ",
    "     │   ◡    │    ",
    "     └────────┘    ",
    "      ╱░░░░░╲      ",
    "    ╱░░░░░░░░░╲    ",
    "   │░░░░░░░░░░░│   ",
    "   │░╔══════╗░│   ",
    "   │░║ )) →  ║░│   ",
    "   │░╚══════╝░│   ",
    "   ╰──────────╯    ",
    "   ╱║║║║║║║║║╲    ",
    "  ╱ ║║║║║║║║║ ╲   ",
    " ╱──────────────╲  ",
    "      ↑  ↑         ",
];

pub const PORTRAIT_THIEF: &[&str] = &[
    "   ░░░░░░░░░░░░    ",
    "  ░░╔══════════╗░  ",
    "  ░░║ ●      ● ║░  ",
    "  ░░║   ┐  ┌   ║░  ",
    "  ░░╚══════════╝░  ",
    "   ░░╱░░░░░░░╲░░   ",
    "   ░╱░░░░░░░░░╲░   ",
    "  ░╱░░░░░░░░░░░╲░  ",
    "    │  ░░░░░  │    ",
    "    │░ ╔═══╗ ░│    ",
    "    │░ ║ ✦ ║ ░│    ",
    "    │░ ╚═══╝ ░│    ",
    "    │  ░░░░░  │    ",
    "  ╱  ╲     ╱  ╲    ",
    " ╱ ✦  ╲   ╱  ✦ ╲   ",
];

pub const PORTRAIT_NECROMANCER: &[&str] = &[
    "    ☠  ☠  ☠  ☠     ",
    "   ┌────────────┐   ",
    "   │ ╔════════╗ │   ",
    "   │ ║ ◯    ◯ ║ │   ",
    "   │ ║   ──   ║ │   ",
    "   │ ╚════════╝ │   ",
    "   └─┤░░░░░░░├──┘   ",
    "     │░░░░░░░│      ",
    "   ╔═╧═══════╧═╗   ",
    "   ║░░░░░░░░░░░║   ",
    "   ║░ ∂  ∇  ∂ ░║   ",
    "   ╚═══╤═══╤═══╝   ",
    "       │   │        ",
    " ☠ ─── ╧ ─ ╧ ─── ☠ ",
    "     ╱       ╲     ",
];

pub const PORTRAIT_ALCHEMIST: &[&str] = &[
    "  ╭────────────╮    ",
    "  │  ○  ●  ○   │    ",
    "  │ ╭──────╮   │    ",
    "  │ │~~~~~│   │    ",
    "  │ │ ⚗   │   │    ",
    "  ╰─┤ ╚═══╝ ├──╯   ",
    "    │  ▒▒▒  │      ",
    "  ╔═╧════════╧═╗   ",
    "  ║ ○ H₂O ⊕ ○ ║   ",
    "  ║  ∑∫ pH=7  ║   ",
    "  ╚══╤══════╤══╝   ",
    "   ┌─╧─┐  ┌─╧─┐   ",
    "   │≋≋≋│  │≋≋≋│   ",
    "   └───┘  └───┘   ",
    "    Hg  Fe  Au  Ag  ",
];

pub const PORTRAIT_PALADIN: &[&str] = &[
    "    ✦  ╔═══╗  ✦    ",
    "       ║ ✝ ║       ",
    "    ✦  ╚═══╝  ✦    ",
    "   ╔══════════╗    ",
    "   ║  ◈    ◈  ║    ",
    "   ║    ──    ║    ",
    "   ╚══════════╝    ",
    "  ╔══════════════╗  ",
    "  ║▓▓▓▓▓▓▓▓▓▓▓▓║  ",
    "  ║▓▓▓ ✦✦✦✦ ▓▓▓║  ",
    "  ║▓▓▓▓▓▓▓▓▓▓▓▓║  ",
    "  ╚══════════════╝  ",
    "     ╔══════╗      ",
    "     ║▓▓▓▓▓▓║      ",
    "     ╚══════╝      ",
    " ✦ ✦ DIVINE ✦ ✦ ✦  ",
];

pub const PORTRAIT_VOIDWALKER: &[&str] = &[
    "  · · · · · · · ·  ",
    "   ·  ╭──────╮  ·  ",
    " · ·  │ ◌  ◌ │  ·· ",
    "      │   ◡  │     ",
    "   ·  ╰──────╯  ·  ",
    "  · ·╱··········╲·· ",
    "    ╱·············╲  ",
    "  ·│···············│·",
    "   │·· ∅  ∅  ∅ ···│  ",
    "   │···············│  ",
    "  ·╲···············╱·",
    "    ╲·············╱  ",
    "  ·  ╲···········╱ · ",
    "   ·  · · · · · ·   ",
    "  · PHASE SHIFT · · ",
];

// Enemy sprite art (ASCII/Unicode block)
pub const SPRITE_FRACTAL_IMP: &[&str] = &[
    "    (◕‿◕)    ",
    "   ╱|||||╲   ",
    "   ╲|||||╱   ",
    "    ╱╱ ╲╲    ",
];

pub const SPRITE_ENTROPY_SPRITE: &[&str] = &[
    "   ~*~*~*~   ",
    "  *╔═════╗*  ",
    "   ║∿∿∿∿∿║   ",
    "  *╚═════╝*  ",
    "   ~*~*~*~   ",
];

pub const SPRITE_KNIGHT: &[&str] = &[
    "  ╔══════╗   ",
    "  ║ ■  ■ ║   ",
    "  ║   ─  ║   ",
    "  ╚══════╝   ",
    " ╔══════════╗ ",
    " ║▓▓▓▓▓▓▓▓▓║ ",
    " ╚══════════╝ ",
    "    ║    ║    ",
    "   ═╩════╩═  ",
];

pub const SPRITE_GOLEM: &[&str] = &[
    "   ╔══════╗  ",
    "   ║ ██ █ ║  ",
    "   ║  ▀▀  ║  ",
    "   ╚══════╝  ",
    "╔═══════════╗ ",
    "║███████████║ ",
    "║███████████║ ",
    "╚═══════════╝ ",
    "  ║  ║  ║   ",
    " ═╩══╩══╩═  ",
];

pub const SPRITE_BOSS_GENERIC: &[&str] = &[
    " ▄▄▄████████▄▄▄  ",
    "▐██╔════════╗██▌ ",
    "▐██║ ◎    ◎ ║██▌ ",
    "▐██║   ▄▄   ║██▌ ",
    "▐██║  ████  ║██▌ ",
    "▐██╚════════╝██▌ ",
    " ▀▀▐████████▌▀▀  ",
    "    ╔══════╗      ",
    "    ║▓▓▓▓▓▓║      ",
    "    ╚══╤╤══╝      ",
    "    ╔══╧╧══╗      ",
    "    ╚═══════╝     ",
];

pub const SPRITE_ABOMINATION: &[&str] = &[
    "╔══════════════════╗",
    "║ [[ UNDEFINED ]]  ║",
    "║ ╔══════════════╗ ║",
    "║ ║ x_INFINITY   ║ ║",
    "║ ║ ERROR 0xFFFF ║ ║",
    "║ ╚══════════════╝ ║",
    "║ STACK OVERFLOW   ║",
    "╚══════════════════╝",
];

// ─── COLOR THEMES ─────────────────────────────────────────────────────────────

pub fn theme_primary(t: ColorTheme) -> Color {
    match t {
        ColorTheme::Classic => Color::Cyan,
        ColorTheme::Neon => Color::Green,
        ColorTheme::Blood => Color::Red,
        ColorTheme::Void => Color::Magenta,
        ColorTheme::Monochrome => Color::White,
    }
}

pub fn theme_accent(t: ColorTheme) -> Color {
    match t {
        ColorTheme::Classic => Color::Yellow,
        ColorTheme::Neon => Color::LightGreen,
        ColorTheme::Blood => Color::LightRed,
        ColorTheme::Void => Color::LightMagenta,
        ColorTheme::Monochrome => Color::Gray,
    }
}

pub fn theme_danger(t: ColorTheme) -> Color {
    match t {
        ColorTheme::Classic | ColorTheme::Neon | ColorTheme::Monochrome => Color::Red,
        ColorTheme::Blood => Color::LightRed,
        ColorTheme::Void => Color::Magenta,
    }
}

fn hp_color(pct: f64) -> Color {
    if pct > 0.6 {
        Color::Green
    } else if pct > 0.3 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn engine_color(v: f64) -> Color {
    if v > 0.5 {
        Color::Green
    } else if v > 0.0 {
        Color::LightGreen
    } else if v > -0.5 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn damage_color(dmg: i64) -> Color {
    if dmg >= 5000 {
        Color::LightMagenta
    } else if dmg >= 1000 {
        Color::LightYellow
    } else if dmg >= 300 {
        Color::LightGreen
    } else {
        Color::White
    }
}

// ─── SMOOTH HP BAR ────────────────────────────────────────────────────────────

/// Renders a smooth HP/MP bar using Unicode block progression.
/// Returns a styled string for embedding in Paragraph/Line widgets.
pub fn smooth_bar(current: i64, max: i64, width: usize, color: Color) -> Line<'static> {
    let pct = (current as f64 / max.max(1) as f64).clamp(0.0, 1.0);
    let filled_f = pct * width as f64;
    let full_blocks = filled_f as usize;
    let remainder = filled_f - full_blocks as f64;

    // Block progression: ' ' → ▏ → ▎ → ▍ → ▌ → ▋ → ▊ → ▉ → █
    let partial_char = match (remainder * 8.0) as u8 {
        0 => ' ',
        1 => '▏',
        2 => '▎',
        3 => '▍',
        4 => '▌',
        5 => '▋',
        6 => '▊',
        7 => '▉',
        _ => '█',
    };

    let mut spans = vec![Span::raw("[")];
    if full_blocks > 0 {
        spans.push(Span::styled(
            "█".repeat(full_blocks),
            Style::default().fg(color),
        ));
    }
    if full_blocks < width {
        let partial = if partial_char != ' ' {
            partial_char.to_string()
        } else {
            String::new()
        };
        if !partial.is_empty() {
            spans.push(Span::styled(partial, Style::default().fg(color)));
        }
        let empty_count = width - full_blocks - if partial_char != ' ' { 1 } else { 0 };
        if empty_count > 0 {
            spans.push(Span::styled(
                "░".repeat(empty_count),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }
    spans.push(Span::raw("] "));
    spans.push(Span::styled(
        format!("{}/{}", current, max),
        Style::default().fg(color),
    ));

    Line::from(spans)
}

// ─── BIDIRECTIONAL ENGINE BAR ─────────────────────────────────────────────────

/// Renders a bidirectional bar for engine output (-1..=1).
/// Center represents 0, left is negative, right is positive.
fn engine_bar(value: f64, width: usize) -> Line<'static> {
    let half = width / 2;
    let color = engine_color(value);
    let filled = ((value.abs() * half as f64) as usize).min(half);

    let mut chars = vec!['░'; width];
    // Center marker
    chars[half] = '│';

    if value >= 0.0 {
        for i in (half + 1)..=(half + filled).min(width - 1) {
            chars[i] = '█';
        }
    } else {
        let start = (half.saturating_sub(filled)).max(0);
        for i in start..half {
            chars[i] = '█';
        }
    }

    let bar_str: String = chars.into_iter().collect();
    let (neg, center, pos) = (
        &bar_str[..half],
        &bar_str[half..half + 1],
        &bar_str[half + 1..],
    );

    Line::from(vec![
        Span::styled(
            neg.to_string(),
            if value < 0.0 {
                Style::default().fg(color)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(center.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(
            pos.to_string(),
            if value >= 0.0 {
                Style::default().fg(color)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ])
}

// ─── ENGINE TRACE WIDGET ──────────────────────────────────────────────────────

/// Draws the engine chain trace panel into the given Rect.
pub fn draw_engine_trace(
    f: &mut Frame,
    area: Rect,
    roll: &ChaosRollResult,
    theme: ColorTheme,
) {
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);
    let bar_width = (area.width as usize).saturating_sub(24).max(8);

    let mut items: Vec<ListItem> = Vec::new();

    for (i, step) in roll.chain.iter().enumerate() {
        let v = step.output;
        let sign = if v >= 0.0 { "+" } else { "" };
        let engine_col = engine_color(v);

        // Row 1: name + value
        let name_span = Span::styled(
            format!("{:2}. {:<22}", i + 1, step.engine_name),
            Style::default().fg(primary),
        );
        let val_span = Span::styled(
            format!("{}{:.4}", sign, v),
            Style::default()
                .fg(engine_col)
                .add_modifier(Modifier::BOLD),
        );
        items.push(ListItem::new(Line::from(vec![name_span, val_span])));

        // Row 2: bidirectional bar
        items.push(ListItem::new(engine_bar(v, bar_width)));
        items.push(ListItem::new(Line::raw("")));
    }

    // Final result
    let result_color = if roll.final_value > 0.5 {
        Color::LightGreen
    } else if roll.final_value > 0.0 {
        Color::Green
    } else if roll.final_value > -0.5 {
        Color::Yellow
    } else {
        Color::Red
    };

    items.push(ListItem::new(Line::from(vec![
        Span::styled("─".repeat(area.width as usize - 2), Style::default().fg(Color::DarkGray)),
    ])));

    let outcome_label = if roll.final_value > 0.8 {
        "★ CRITICAL"
    } else if roll.final_value > 0.0 {
        "✓ SUCCESS"
    } else if roll.final_value > -0.8 {
        "✗ FAILURE"
    } else {
        "☠ CATASTROPHE"
    };

    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!("{} — {:.4}  →  {}", outcome_label, roll.final_value, roll.game_value),
            Style::default()
                .fg(result_color)
                .add_modifier(Modifier::BOLD),
        ),
    ])));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary))
            .title(Span::styled(
                " ⚙ CHAOS ENGINE CHAIN ",
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(list, area);
}

// ─── COMBAT LOG WIDGET ────────────────────────────────────────────────────────

/// Draws the combat log (list of recent events) into the given Rect.
pub fn draw_combat_log(
    f: &mut Frame,
    area: Rect,
    events: &[String],
    theme: ColorTheme,
) {
    let primary = theme_primary(theme);
    let items: Vec<ListItem> = events
        .iter()
        .rev()
        .take(area.height as usize - 2)
        .map(|e| {
            let color = if e.contains("CRITICAL") || e.contains("★") {
                Color::LightYellow
            } else if e.contains("killed") || e.contains("slain") || e.contains("dead") {
                Color::LightGreen
            } else if e.contains("damage") && !e.contains("You") {
                Color::Red
            } else if e.contains("heal") || e.contains("HP") {
                Color::Green
            } else if e.contains("CHAOS") {
                Color::Magenta
            } else {
                Color::Gray
            };
            ListItem::new(Span::styled(format!("→ {}", e), Style::default().fg(color)))
        })
        .collect();

    let log = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(
                    " Combat Log ",
                    Style::default().fg(primary),
                )),
        )
        .direction(ratatui::widgets::ListDirection::BottomToTop);
    f.render_widget(log, area);
}

// ─── ENEMY DISPLAY ────────────────────────────────────────────────────────────

fn enemy_sprite_lines(enemy: &Enemy) -> &'static [&'static str] {
    use chaos_rpg_core::enemy::EnemyTier;
    match enemy.tier {
        EnemyTier::Minion => {
            if enemy.seed % 2 == 0 {
                SPRITE_FRACTAL_IMP
            } else {
                SPRITE_ENTROPY_SPRITE
            }
        }
        EnemyTier::Elite => SPRITE_KNIGHT,
        EnemyTier::Champion => SPRITE_GOLEM,
        EnemyTier::Boss => SPRITE_BOSS_GENERIC,
        EnemyTier::Abomination => SPRITE_ABOMINATION,
    }
}

/// Draws the enemy panel (art + HP bar) into the given Rect.
pub fn draw_enemy_panel(f: &mut Frame, area: Rect, enemy: &Enemy, theme: ColorTheme) {
    let primary = theme_primary(theme);
    let tier_color = match enemy.tier {
        chaos_rpg_core::enemy::EnemyTier::Minion => Color::Gray,
        chaos_rpg_core::enemy::EnemyTier::Elite => Color::Green,
        chaos_rpg_core::enemy::EnemyTier::Champion => Color::Cyan,
        chaos_rpg_core::enemy::EnemyTier::Boss => Color::Yellow,
        chaos_rpg_core::enemy::EnemyTier::Abomination => Color::Magenta,
    };

    let hp_pct = enemy.hp as f64 / enemy.max_hp.max(1) as f64;
    let hp_col = hp_color(hp_pct);

    let sprite = enemy_sprite_lines(enemy);
    let mut lines: Vec<Line> = Vec::new();

    // Enemy name + tier header
    lines.push(Line::from(vec![
        Span::styled(
            format!("[{}] ", enemy.tier.name()),
            Style::default().fg(tier_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            enemy.name.clone(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::raw(""));

    // Sprite art
    for line in sprite {
        lines.push(Line::from(Span::styled(*line, Style::default().fg(tier_color))));
    }
    lines.push(Line::raw(""));

    // HP bar
    let bar_width = (area.width as usize).saturating_sub(12).max(8);
    lines.push(smooth_bar(enemy.hp, enemy.max_hp, bar_width, hp_col));

    // Special ability
    if let Some(ability) = enemy.special_ability {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            format!("⚡ {}", ability),
            Style::default().fg(Color::Yellow),
        )));
    }

    // Floor ability
    use chaos_rpg_core::enemy::FloorAbility;
    let floor_ability_text = match enemy.floor_ability {
        FloorAbility::StatMirror => Some("◈ STAT MIRROR active"),
        FloorAbility::EngineTheft => Some("⛓ ENGINE THEFT active"),
        FloorAbility::NullifyAura => Some("∅ NULLIFY AURA active"),
        FloorAbility::None => None,
    };
    if let Some(txt) = floor_ability_text {
        lines.push(Line::from(Span::styled(
            txt,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let para = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(" Enemy ", Style::default().fg(primary))),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

// ─── PLAYER STATUS PANEL ──────────────────────────────────────────────────────

/// Draws the player status (HP/MP bars, status effects, combo) into the given Rect.
pub fn draw_player_panel(f: &mut Frame, area: Rect, player: &Character, theme: ColorTheme) {
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);
    let bar_width = (area.width as usize).saturating_sub(12).max(8);

    let mut lines: Vec<Line> = Vec::new();

    // Name / class / level
    lines.push(Line::from(vec![
        Span::styled(
            player.name.clone(),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  Lv.{}  {}", player.level, player.class.name()),
            Style::default().fg(Color::Gray),
        ),
    ]));
    lines.push(Line::raw(""));

    // HP bar
    let hp_col = hp_color(player.current_hp as f64 / player.max_hp.max(1) as f64);
    let mut hp_line = smooth_bar(player.current_hp, player.max_hp, bar_width, hp_col);
    hp_line.spans.insert(0, Span::styled("HP  ", Style::default().fg(Color::Green)));
    lines.push(hp_line);

    // Mana bar (derived from stats.mana, shown as a capacity meter)
    let mana_val = player.stats.mana.max(0);
    let mana_max = (mana_val + 50).max(50);
    let mp_col = if mana_val > mana_max / 2 { Color::Cyan } else { Color::Blue };
    let mut mp_line = smooth_bar(mana_val, mana_max, bar_width, mp_col);
    mp_line.spans.insert(0, Span::styled("MANA", Style::default().fg(Color::Cyan)));
    lines.push(mp_line);

    // Status effects
    if !player.status_effects.is_empty() {
        let badges: Vec<Span> = player
            .status_effects
            .iter()
            .take(6)
            .map(|s| {
                let n = s.name();
                let abbr = &n[..n.len().min(4)];
                Span::styled(
                    format!("[{}] ", abbr),
                    Style::default().fg(Color::Yellow),
                )
            })
            .collect();
        let mut status_line = Line::from(vec![Span::raw("Status: ")]);
        status_line.spans.extend(badges);
        lines.push(status_line);
    }

    // Shield from Shielded status effect
    for s in &player.status_effects {
        if let StatusEffect::Shielded(v) = s {
            lines.push(Line::from(Span::styled(
                format!("Shield: {}", v),
                Style::default().fg(Color::Cyan),
            )));
            break;
        }
    }

    // Corruption
    if player.corruption_stage() > 0 {
        lines.push(Line::from(Span::styled(
            format!("Corruption: {} [Stage {}/8]", player.corruption_label(), player.corruption_stage()),
            Style::default().fg(Color::Red),
        )));
    }

    let para = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(" Player ", Style::default().fg(primary))),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

// ─── ACTION BAR ──────────────────────────────────────────────────────────────

/// Draws the action menu bar at the bottom of the combat screen.
pub fn draw_action_bar(
    f: &mut Frame,
    area: Rect,
    player: &Character,
    theme: ColorTheme,
) {
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);

    let mut lines: Vec<Line> = Vec::new();

    // Basic actions
    let actions = Line::from(vec![
        Span::styled("[A] ", Style::default().fg(accent)),
        Span::raw("Attack  "),
        Span::styled("[H] ", Style::default().fg(accent)),
        Span::raw("Heavy  "),
        Span::styled("[D] ", Style::default().fg(accent)),
        Span::raw("Defend  "),
        Span::styled("[T] ", Style::default().fg(accent)),
        Span::raw("Taunt  "),
        Span::styled("[F] ", Style::default().fg(accent)),
        Span::raw("Flee"),
    ]);
    lines.push(actions);

    // Spells
    if !player.known_spells.is_empty() {
        let mut spell_spans = vec![Span::styled("Spells: ", Style::default().fg(Color::Cyan))];
        for (i, spell) in player.known_spells.iter().take(4).enumerate() {
            spell_spans.push(Span::styled(
                format!("[S{}] ", i + 1),
                Style::default().fg(Color::LightCyan),
            ));
            let truncated = if spell.name.len() > 14 {
                format!("{}…", &spell.name[..13])
            } else {
                spell.name.clone()
            };
            spell_spans.push(Span::raw(format!("{}  ", truncated)));
        }
        lines.push(Line::from(spell_spans));
    }

    // Items
    if !player.inventory.is_empty() {
        let mut item_spans = vec![Span::styled("Items:  ", Style::default().fg(Color::Yellow))];
        for (i, item) in player.inventory.iter().take(4).enumerate() {
            item_spans.push(Span::styled(
                format!("[I{}] ", i + 1),
                Style::default().fg(Color::LightYellow),
            ));
            let truncated = if item.name.len() > 14 {
                format!("{}…", &item.name[..13])
            } else {
                item.name.clone()
            };
            item_spans.push(Span::raw(format!("{}  ", truncated)));
        }
        lines.push(Line::from(item_spans));
    }

    let para = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary))
            .title(Span::styled(" Actions ", Style::default().fg(primary))),
    );
    f.render_widget(para, area);
}

// ─── FULL COMBAT SCREEN ───────────────────────────────────────────────────────

/// Data bundle for rendering the combat screen — avoids threading too many args.
pub struct CombatViewState<'a> {
    pub player: &'a Character,
    pub enemy: &'a Enemy,
    pub floor: u32,
    pub room: u32,
    pub total_rooms: usize,
    pub round: u32,
    pub last_roll: Option<&'a ChaosRollResult>,
    pub log: &'a [String],
    pub theme: ColorTheme,
    pub is_cursed: bool,
}

/// Draw the full combat screen.
///
/// Layout:
/// ```text
/// ┌─ header ─────────────────────────────────────────────────────────────┐
/// ├─ enemy panel (left col, 60%) ─┬─ engine trace (right col, 40%) ─────┤
/// │                               │                                      │
/// │  sprite + HP bar              │  Lorenz    +0.847  ░░░░│████████    │
/// │                               │  Collatz   -0.213  ███░│░░░░░░░░    │
/// ├─ player panel (left) ─────────┤  ...                                 │
/// │  HP/MP bars, status, combo    ├─ combat log ────────────────────────┤
/// │                               │  → attack for 2173                   │
/// ├─ action bar ──────────────────┴──────────────────────────────────────┤
/// │  [A] [H] [D] [T] [F]  Spells: [S1]... Items: [I1]...               │
/// └──────────────────────────────────────────────────────────────────────┘
/// ```
pub fn draw_combat_screen(f: &mut Frame, state: &CombatViewState) {
    let size = f.area();
    let theme = state.theme;
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);

    // Top-level split: header (3) | body | actions (5)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
        ])
        .split(size);

    // ── Header ───────────────────────────────────────────────────────────────
    let cursed_tag = if state.is_cursed { "  ☠ CURSED FLOOR" } else { "" };
    let header_text = format!(
        " Floor {}  Room {}/{}  Round {}{}",
        state.floor, state.room, state.total_rooms, state.round, cursed_tag
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(accent).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary)),
        )
        .alignment(Alignment::Left);
    f.render_widget(header, main_chunks[0]);

    // ── Body: left col + right col ────────────────────────────────────────────
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(main_chunks[1]);

    // Left col: enemy (top) + player (bottom), 55%/45%
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(body_chunks[0]);

    draw_enemy_panel(f, left_chunks[0], state.enemy, theme);
    draw_player_panel(f, left_chunks[1], state.player, theme);

    // Right col: engine trace (top ~65%) + combat log (bottom ~35%)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(body_chunks[1]);

    if let Some(roll) = state.last_roll {
        draw_engine_trace(f, right_chunks[0], roll, theme);
    } else {
        // Placeholder if no roll yet
        let placeholder = Paragraph::new("No roll yet.\nFight something!")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(primary))
                    .title(" ⚙ CHAOS ENGINE CHAIN "),
            )
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, right_chunks[0]);
    }

    draw_combat_log(f, right_chunks[1], state.log, theme);

    // ── Action bar ────────────────────────────────────────────────────────────
    draw_action_bar(f, main_chunks[2], state.player, theme);
}

// ─── CHARACTER SHEET ─────────────────────────────────────────────────────────

/// Draw the full character sheet screen with class portrait.
pub fn draw_character_sheet(f: &mut Frame, player: &Character, theme: ColorTheme) {
    let size = f.area();
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);

    // Three column layout: portrait | stats | equipment
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Min(30),
            Constraint::Length(32),
        ])
        .split(size);

    // ── Left: portrait ────────────────────────────────────────────────────────
    let portrait = match player.class {
        CharacterClass::Mage => PORTRAIT_MAGE,
        CharacterClass::Berserker => PORTRAIT_BERSERKER,
        CharacterClass::Ranger => PORTRAIT_RANGER,
        CharacterClass::Thief => PORTRAIT_THIEF,
        CharacterClass::Necromancer => PORTRAIT_NECROMANCER,
        CharacterClass::Alchemist => PORTRAIT_ALCHEMIST,
        CharacterClass::Paladin => PORTRAIT_PALADIN,
        CharacterClass::VoidWalker   => PORTRAIT_VOIDWALKER,
        CharacterClass::Warlord      => PORTRAIT_BERSERKER,   // reuse similar
        CharacterClass::Trickster    => PORTRAIT_THIEF,
        CharacterClass::Runesmith    => PORTRAIT_PALADIN,
        CharacterClass::Chronomancer => PORTRAIT_MAGE,
    };
    let class_color = match player.class {
        CharacterClass::Mage         => Color::Cyan,
        CharacterClass::Berserker    => Color::Red,
        CharacterClass::Ranger       => Color::Green,
        CharacterClass::Thief        => Color::DarkGray,
        CharacterClass::Necromancer  => Color::Magenta,
        CharacterClass::Alchemist    => Color::Yellow,
        CharacterClass::Paladin      => Color::LightYellow,
        CharacterClass::VoidWalker   => Color::LightMagenta,
        CharacterClass::Warlord      => Color::Red,
        CharacterClass::Trickster    => Color::LightGreen,
        CharacterClass::Runesmith    => Color::LightBlue,
        CharacterClass::Chronomancer => Color::LightCyan,
    };

    let portrait_lines: Vec<Line> = portrait
        .iter()
        .map(|l| Line::from(Span::styled(*l, Style::default().fg(class_color))))
        .collect();

    let portrait_para = Paragraph::new(Text::from(portrait_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(
                    format!(" {} ", player.class.name()),
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                )),
        );
    f.render_widget(portrait_para, chunks[0]);

    // ── Center: stats ─────────────────────────────────────────────────────────
    let tier = player.power_tier();
    let (tr, tg, tb) = tier.rgb();
    let tier_color_ratatui = Color::Rgb(tr, tg, tb);

    let bar_width = (chunks[1].width as usize).saturating_sub(20).max(8);

    let stat_names = ["Vitality", "Force", "Mana", "Cunning", "Precision", "Entropy", "Luck"];
    let stat_values = [
        player.stats.vitality,
        player.stats.force,
        player.stats.mana,
        player.stats.cunning,
        player.stats.precision,
        player.stats.entropy,
        player.stats.luck,
    ];

    let mut stat_lines: Vec<Line> = Vec::new();
    stat_lines.push(Line::from(vec![
        Span::styled(player.name.clone(), Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  Lv.{}", player.level), Style::default().fg(Color::Gray)),
        Span::styled(format!("  [{}]", player.class.name()), Style::default().fg(class_color)),
    ]));
    stat_lines.push(Line::from(Span::styled(
        format!("  {}  —  {}", tier.name(), tier.flavor()),
        Style::default().fg(tier_color_ratatui),
    )));
    stat_lines.push(Line::raw(""));

    // HP bar
    let hp_col = hp_color(player.current_hp as f64 / player.max_hp.max(1) as f64);
    let mut hp = smooth_bar(player.current_hp, player.max_hp, bar_width, hp_col);
    hp.spans.insert(0, Span::styled("HP      ", Style::default().fg(Color::Green)));
    stat_lines.push(hp);

    // Mana bar (shown as capacity from stats)
    let mana_val = player.stats.mana.max(0);
    let mana_max = (mana_val + 50).max(50);
    let mp_col = if mana_val > 40 { Color::Cyan } else { Color::Blue };
    let mut mp = smooth_bar(mana_val, mana_max, bar_width, mp_col);
    mp.spans.insert(0, Span::styled("MANA    ", Style::default().fg(Color::Cyan)));
    stat_lines.push(mp);
    stat_lines.push(Line::raw(""));

    // Stats with bars
    fn stat_color_ratatui(val: i64) -> Color {
        match val {
            i64::MIN..=-100 => Color::Red,
            -99..=-1 => Color::LightRed,
            0..=50 => Color::Yellow,
            51..=150 => Color::White,
            151..=400 => Color::Cyan,
            401..=800 => Color::LightGreen,
            801..=2000 => Color::LightYellow,
            _ => Color::LightMagenta,
        }
    }

    for (name, &val) in stat_names.iter().zip(stat_values.iter()) {
        let color = stat_color_ratatui(val);
        let bar_fill = ((val.max(0) as f64 / 200.0) * 12.0) as usize;
        let bar_fill = bar_fill.min(12);
        let mini_bar = format!(
            "[{}{}]",
            "█".repeat(bar_fill),
            "░".repeat(12 - bar_fill)
        );
        stat_lines.push(Line::from(vec![
            Span::styled(format!("{:<12}", name), Style::default().fg(Color::Gray)),
            Span::styled(format!("{:>6}  ", val), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(mini_bar, Style::default().fg(color)),
        ]));
    }

    stat_lines.push(Line::raw(""));
    stat_lines.push(Line::from(vec![
        Span::styled("Floor ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", player.floor), Style::default().fg(accent)),
        Span::styled("  Gold ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", player.gold), Style::default().fg(Color::Yellow)),
        Span::styled("  Kills ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", player.kills), Style::default().fg(Color::Red)),
    ]));

    if player.corruption_stage() > 0 {
        let kills_to_next = 50 - (player.kills % 50);
        stat_lines.push(Line::from(Span::styled(
            format!(
                "Corruption: {}  Stage {}/8  ({} kills to next)",
                player.corruption_label(),
                player.corruption_stage(),
                kills_to_next
            ),
            Style::default().fg(Color::Red),
        )));
    }

    let stats_para = Paragraph::new(Text::from(stat_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(" Statistics ", Style::default().fg(primary))),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(stats_para, chunks[1]);

    // ── Right: equipment + spells ──────────────────────────────────────────────
    let mut eq_lines: Vec<Line> = Vec::new();
    eq_lines.push(Line::from(Span::styled(
        "── Inventory ──",
        Style::default().fg(accent),
    )));

    if player.inventory.is_empty() {
        eq_lines.push(Line::from(Span::styled("  (empty)", Style::default().fg(Color::DarkGray))));
    } else {
        for item in player.inventory.iter().take(8) {
            let rarity_col = match item.rarity.name() {
                "Common" => Color::DarkGray,
                "Uncommon" => Color::White,
                "Rare" => Color::Green,
                "Epic" => Color::Blue,
                "Legendary" => Color::Magenta,
                "Mythical" => Color::Yellow,
                "Divine" => Color::Red,
                _ => Color::LightMagenta,
            };
            eq_lines.push(Line::from(vec![
                Span::styled("  ✦ ", Style::default().fg(rarity_col)),
                Span::styled(item.name.clone(), Style::default().fg(rarity_col)),
            ]));
        }
    }

    eq_lines.push(Line::raw(""));
    eq_lines.push(Line::from(Span::styled(
        "── Spells ──",
        Style::default().fg(Color::Cyan),
    )));

    if player.known_spells.is_empty() {
        eq_lines.push(Line::from(Span::styled("  (none)", Style::default().fg(Color::DarkGray))));
    } else {
        for spell in player.known_spells.iter().take(6) {
            eq_lines.push(Line::from(vec![
                Span::styled("  ⚡ ", Style::default().fg(Color::Cyan)),
                Span::styled(spell.name.clone(), Style::default().fg(Color::LightCyan)),
                Span::styled(
                    format!("  (mp:{})", spell.mana_cost),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    eq_lines.push(Line::raw(""));
    eq_lines.push(Line::from(Span::styled(
        "── Background ──",
        Style::default().fg(Color::Gray),
    )));
    eq_lines.push(Line::from(Span::styled(
        format!("  {}", player.background.name()),
        Style::default().fg(Color::White),
    )));
    eq_lines.push(Line::from(Span::styled(
        format!("  XP: {}", player.xp),
        Style::default().fg(Color::Yellow),
    )));

    let eq_para = Paragraph::new(Text::from(eq_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(" Equipment & Spells ", Style::default().fg(primary))),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(eq_para, chunks[2]);
}

// ─── FLOOR NAVIGATION SCREEN ─────────────────────────────────────────────────

/// Draw the floor navigation screen (minimap, room list, quick status).
pub fn draw_floor_nav(
    f: &mut Frame,
    player: &Character,
    minimap: &str,
    floor_rooms: usize,
    rooms_done: usize,
    theme: ColorTheme,
    is_cursed: bool,
) {
    let size = f.area();
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(size);

    // Left: map + navigation help
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(10)])
        .split(chunks[0]);

    // Minimap panel
    let mut map_lines: Vec<Line> = Vec::new();
    if is_cursed {
        map_lines.push(Line::from(Span::styled(
            "  ☠ CURSED FLOOR — ALL ENGINES INVERTED ☠",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        map_lines.push(Line::raw(""));
    }
    map_lines.push(Line::from(Span::styled(
        format!("  Floor {}  —  {}/{} rooms", player.floor, rooms_done, floor_rooms),
        Style::default().fg(accent),
    )));
    map_lines.push(Line::raw(""));
    for map_line in minimap.lines() {
        map_lines.push(Line::from(Span::styled(
            format!("  {}", map_line),
            Style::default().fg(Color::White),
        )));
    }
    map_lines.push(Line::raw(""));
    map_lines.push(Line::from(Span::styled(
        "  [x]=Combat [*]=Treasure [$]=Shop [~]=Shrine",
        Style::default().fg(Color::DarkGray),
    )));
    map_lines.push(Line::from(Span::styled(
        "  [!]=Trap [B]=Boss [^]=Portal [ ]=Empty [?]=Rift",
        Style::default().fg(Color::DarkGray),
    )));

    let map_para = Paragraph::new(Text::from(map_lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary))
            .title(Span::styled(" Floor Map ", Style::default().fg(primary))),
    );
    f.render_widget(map_para, left_chunks[0]);

    // Navigation controls
    let nav_lines = vec![
        Line::from(vec![
            Span::styled("[E]", Style::default().fg(accent)),
            Span::raw(" Enter room    "),
            Span::styled("[C]", Style::default().fg(accent)),
            Span::raw(" Character"),
        ]),
        Line::from(vec![
            Span::styled("[B]", Style::default().fg(accent)),
            Span::raw(" Body chart    "),
            Span::styled("[P]", Style::default().fg(accent)),
            Span::raw(" Skill tree"),
        ]),
        Line::from(vec![
            Span::styled("[F]", Style::default().fg(accent)),
            Span::raw(" Factions      "),
            Span::styled("[T]", Style::default().fg(accent)),
            Span::raw(" Last trace"),
        ]),
        Line::from(vec![
            Span::styled("[D]", Style::default().fg(Color::Cyan)),
            Span::raw(" Descend  "),
            Span::styled("[Q]", Style::default().fg(Color::Red)),
            Span::raw(" Quit"),
        ]),
    ];
    let nav_para = Paragraph::new(Text::from(nav_lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary))
            .title(Span::styled(" Navigate ", Style::default().fg(primary))),
    );
    f.render_widget(nav_para, left_chunks[1]);

    // Right: player quick status
    let bar_width = (chunks[1].width as usize).saturating_sub(16).max(8);
    let mut status_lines: Vec<Line> = Vec::new();

    status_lines.push(Line::from(vec![
        Span::styled(player.name.clone(), Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("  {}  Lv.{}", player.class.name(), player.level),
            Style::default().fg(Color::Gray),
        ),
    ]));
    status_lines.push(Line::raw(""));

    let hp_col = hp_color(player.current_hp as f64 / player.max_hp.max(1) as f64);
    let mut hp = smooth_bar(player.current_hp, player.max_hp, bar_width, hp_col);
    hp.spans.insert(0, Span::styled("HP  ", Style::default().fg(Color::Green)));
    status_lines.push(hp);

    let mana_v = player.stats.mana.max(0);
    let mana_mx = (mana_v + 50).max(50);
    let mp_col = if mana_v > 40 { Color::Cyan } else { Color::Blue };
    let mut mp = smooth_bar(mana_v, mana_mx, bar_width, mp_col);
    mp.spans.insert(0, Span::styled("MANA", Style::default().fg(Color::Cyan)));
    status_lines.push(mp);

    status_lines.push(Line::raw(""));
    status_lines.push(Line::from(vec![
        Span::styled("Gold  ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("{}", player.gold), Style::default().fg(Color::LightYellow)),
        Span::styled("  Kills  ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", player.kills), Style::default().fg(Color::Red)),
        Span::styled("  XP  ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", player.xp), Style::default().fg(Color::Cyan)),
    ]));

    if player.corruption_stage() > 0 {
        status_lines.push(Line::raw(""));
        status_lines.push(Line::from(Span::styled(
            format!("Corruption: {}  [Stage {}/8]", player.corruption_label(), player.corruption_stage()),
            Style::default().fg(Color::Red),
        )));
    }
    if player.floor >= 50 && player.rooms_without_kill >= 3 {
        status_lines.push(Line::from(Span::styled(
            format!("THE HUNGER: {} rooms without kill", player.rooms_without_kill),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let status_para = Paragraph::new(Text::from(status_lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(" Status ", Style::default().fg(primary))),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(status_para, chunks[1]);
}

// ─── TITLE SCREEN ─────────────────────────────────────────────────────────────

/// Draw the CHAOS RPG title screen.
pub fn draw_title_screen(f: &mut Frame, selected: usize, theme: ColorTheme) {
    let size = f.area();
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // title art
            Constraint::Min(8),     // menu
            Constraint::Length(2),  // version footer
        ])
        .split(size);

    // Title art
    let title_art = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "  ██████╗██╗  ██╗ █████╗  ██████╗ ███████╗    ██████╗ ██████╗  ██████╗ ",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " ██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝    ██╔══██╗██╔══██╗██╔════╝ ",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " ██║     ███████║███████║██║   ██║███████╗    ██████╔╝██████╔╝██║  ███╗",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " ██║     ██╔══██║██╔══██║██║   ██║╚════██║    ██╔══██╗██╔═══╝ ██║   ██║",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " ╚██████╗██║  ██║██║  ██║╚██████╔╝███████║    ██║  ██║██║     ╚██████╔╝",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝    ╚═╝  ╚═╝╚═╝      ╚═════╝ ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "          Where math goes to die. 10 sacred algorithms.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let title_para = Paragraph::new(Text::from(title_art)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary)),
    );
    f.render_widget(title_para, chunks[0]);

    // Menu
    let menu_options = ["Story Mode (10 floors)", "Infinite Mode", "Daily Seed Challenge", "Scoreboard", "Quit"];
    let items: Vec<ListItem> = menu_options
        .iter()
        .enumerate()
        .map(|(i, &opt)| {
            if i == selected {
                ListItem::new(Line::from(vec![
                    Span::styled(" ► ", Style::default().fg(accent).add_modifier(Modifier::BOLD)),
                    Span::styled(opt, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]))
            } else {
                ListItem::new(Line::from(Span::styled(
                    format!("   {}", opt),
                    Style::default().fg(Color::DarkGray),
                )))
            }
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(primary))
            .title(Span::styled(" Select Mode ", Style::default().fg(accent))),
    );
    f.render_widget(menu, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  ↑↓ Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter Select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Q Quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("v0.1.0", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

// ─── SCOREBOARD SCREEN ────────────────────────────────────────────────────────

/// Draw the scoreboard.
pub fn draw_scoreboard(f: &mut Frame, scores: &[ScoreEntry], theme: ColorTheme) {
    let size = f.area();
    let primary = theme_primary(theme);
    let accent = theme_accent(theme);

    let header_cells = ["#", "Name", "Class", "Floor", "Defeated", "Score", "Date"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(accent).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = scores
        .iter()
        .enumerate()
        .take(20)
        .map(|(i, s)| {
            let rank_color = match i {
                0 => Color::LightYellow,
                1 => Color::White,
                2 => Color::Yellow,
                _ => Color::Gray,
            };
            Row::new(vec![
                Cell::from(format!("{}", i + 1)).style(Style::default().fg(rank_color)),
                Cell::from(s.name.clone()).style(Style::default().fg(Color::White)),
                Cell::from(s.class.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("{}", s.floor_reached)).style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{}", s.enemies_defeated)).style(Style::default().fg(Color::Red)),
                Cell::from(format!("{}", s.score)).style(Style::default().fg(Color::LightYellow)),
                Cell::from(s.timestamp.clone()).style(Style::default().fg(Color::Gray)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Min(10),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary))
                .title(Span::styled(
                    " ★ HALL OF CHAOS ★ ",
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                )),
        );
    f.render_widget(table, size);
}
