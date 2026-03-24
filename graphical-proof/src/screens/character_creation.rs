//! Character creation screen — class, background, difficulty, name.

use proof_engine::prelude::*;
use chaos_rpg_core::character::{Character, CharacterClass, Background, Difficulty};
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

const CLASSES: &[CharacterClass] = &[
    CharacterClass::Mage, CharacterClass::Berserker, CharacterClass::Ranger,
    CharacterClass::Thief, CharacterClass::Necromancer, CharacterClass::Alchemist,
    CharacterClass::Paladin, CharacterClass::VoidWalker, CharacterClass::Warlord,
    CharacterClass::Trickster, CharacterClass::Runesmith, CharacterClass::Chronomancer,
];

const BACKGROUNDS: &[Background] = &[
    Background::Scholar, Background::Wanderer, Background::Gladiator, Background::Outcast,
    Background::Merchant, Background::Cultist, Background::Exile, Background::Oracle,
];

const DIFFICULTIES: &[Difficulty] = &[
    Difficulty::Easy, Difficulty::Normal, Difficulty::Brutal, Difficulty::Chaos,
];

const LETTER_KEYS: &[(Key, char)] = &[
    (Key::A, 'A'), (Key::B, 'B'), (Key::C, 'C'), (Key::D, 'D'),
    (Key::E, 'E'), (Key::F, 'F'), (Key::G, 'G'), (Key::H, 'H'),
    (Key::I, 'I'), (Key::J, 'J'), (Key::K, 'K'), (Key::L, 'L'),
    (Key::M, 'M'), (Key::N, 'N'), (Key::O, 'O'), (Key::P, 'P'),
    (Key::Q, 'Q'), (Key::R, 'R'), (Key::S, 'S'), (Key::T, 'T'),
    (Key::U, 'U'), (Key::V, 'V'), (Key::W, 'W'), (Key::X, 'X'),
    (Key::Y, 'Y'), (Key::Z, 'Z'),
];

const DIGIT_KEYS: &[(Key, char)] = &[
    (Key::Num0, '0'), (Key::Num1, '1'), (Key::Num2, '2'), (Key::Num3, '3'),
    (Key::Num4, '4'), (Key::Num5, '5'), (Key::Num6, '6'), (Key::Num7, '7'),
    (Key::Num8, '8'), (Key::Num9, '9'),
];

// ── Input ───────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    if state.cc_name_active {
        // Name editor mode
        for &(key, ch) in LETTER_KEYS {
            if engine.input.just_pressed(key) && state.cc_name.len() < 16 {
                state.cc_name.push(ch);
            }
        }
        for &(key, ch) in DIGIT_KEYS {
            if engine.input.just_pressed(key) && state.cc_name.len() < 16 {
                state.cc_name.push(ch);
            }
        }
        if engine.input.just_pressed(Key::Space) && state.cc_name.len() < 16 {
            state.cc_name.push(' ');
        }
        if engine.input.just_pressed(Key::Backspace) {
            state.cc_name.pop();
        }
        if engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape) {
            state.cc_name_active = false;
        }
        return;
    }

    // Navigation
    if engine.input.just_pressed(Key::Up) {
        state.cc_class = (state.cc_class + CLASSES.len() - 1) % CLASSES.len();
    }
    if engine.input.just_pressed(Key::Down) {
        state.cc_class = (state.cc_class + 1) % CLASSES.len();
    }
    if engine.input.just_pressed(Key::Left) {
        state.cc_bg = (state.cc_bg + BACKGROUNDS.len() - 1) % BACKGROUNDS.len();
    }
    if engine.input.just_pressed(Key::Right) {
        state.cc_bg = (state.cc_bg + 1) % BACKGROUNDS.len();
    }
    if engine.input.just_pressed(Key::Tab) {
        state.cc_diff = (state.cc_diff + 1) % DIFFICULTIES.len();
    }
    if engine.input.just_pressed(Key::N) {
        state.cc_name_active = true;
    }
    if engine.input.just_pressed(Key::Escape) {
        state.screen = AppScreen::ModeSelect;
    }
    if engine.input.just_pressed(Key::Enter) {
        let name = if state.cc_name.is_empty() {
            "Unnamed".to_string()
        } else {
            state.cc_name.clone()
        };
        let class = CLASSES[state.cc_class];
        let background = BACKGROUNDS[state.cc_bg];
        let difficulty = DIFFICULTIES[state.cc_diff];
        let seed = state.seed;
        let player = Character::roll_new(name, class, background, seed, difficulty);
        state.player = Some(player);
        state.screen = AppScreen::BoonSelect;
    }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let sel = theme.selected;
    let dim = theme.dim;
    let head = theme.heading;
    let acc = theme.accent;
    let pri = theme.primary;

    // Title
    render_text(engine, "=== CHARACTER CREATION ===", -5.5, 8.0, head, 0.9);

    // ── Name field (top center) ──
    let name_label = if state.cc_name_active { "> NAME: " } else { "  NAME: " };
    let name_color = if state.cc_name_active { acc } else { pri };
    let display_name = if state.cc_name.is_empty() { "<press N to edit>" } else { &state.cc_name };
    let name_line = format!("{}{}_", name_label, display_name);
    render_text(engine, &name_line, -10.0, 6.5, name_color, 0.7);

    // ── Class list (left column) ──
    render_text(engine, "CLASS [Up/Down]", -18.0, 5.0, head, 0.7);
    for (i, cls) in CLASSES.iter().enumerate() {
        let y = 4.0 - i as f32 * 0.7;
        let selected = i == state.cc_class;
        let marker = if selected { "> " } else { "  " };
        let color = if selected { sel } else { dim };
        let em = if selected { 0.8 } else { 0.3 };
        let label = format!("{}{}", marker, cls.name());
        render_text(engine, &label, -18.0, y, color, em);
    }

    // ── Background list (middle column) ──
    render_text(engine, "BACKGROUND [L/R]", -5.0, 5.0, head, 0.7);
    for (i, bg) in BACKGROUNDS.iter().enumerate() {
        let y = 4.0 - i as f32 * 0.7;
        let selected = i == state.cc_bg;
        let marker = if selected { "> " } else { "  " };
        let color = if selected { sel } else { dim };
        let em = if selected { 0.8 } else { 0.3 };
        let label = format!("{}{}", marker, bg.name());
        render_text(engine, &label, -5.0, y, color, em);
    }

    // ── Difficulty (right column) ──
    render_text(engine, "DIFFICULTY [Tab]", 8.0, 5.0, head, 0.7);
    for (i, diff) in DIFFICULTIES.iter().enumerate() {
        let y = 4.0 - i as f32 * 0.7;
        let selected = i == state.cc_diff;
        let marker = if selected { "> " } else { "  " };
        let color = if selected { sel } else { dim };
        let em = if selected { 0.8 } else { 0.3 };
        let label = format!("{}{}", marker, diff.name());
        render_text(engine, &label, 8.0, y, color, em);
    }

    // ── Preview panel (bottom area) ──
    let cls = CLASSES[state.cc_class];
    let bg = BACKGROUNDS[state.cc_bg];
    let diff = DIFFICULTIES[state.cc_diff];

    // Class description + passive + ASCII art
    render_text(engine, &format!("-- {} --", cls.name()), -18.0, -5.0, acc, 0.8);
    render_text(engine, cls.description(), -18.0, -5.8, pri, 0.5);

    let passive_line = format!("Passive: {} - {}", cls.passive_name(), cls.passive_desc());
    render_text(engine, &passive_line, -18.0, -6.6, pri, 0.5);

    // ASCII art for class (render each line)
    let art = cls.ascii_art();
    for (li, line) in art.lines().enumerate() {
        render_text(engine, line, -18.0, -7.6 - li as f32 * 0.6, dim, 0.4);
    }

    // Background description
    render_text(engine, &format!("-- {} --", bg.name()), -2.0, -5.0, acc, 0.8);
    render_text(engine, bg.description(), -2.0, -5.8, pri, 0.5);

    // Difficulty description
    render_text(engine, &format!("-- {} --", diff.name()), 8.0, -5.0, acc, 0.8);
    render_text(engine, diff.description(), 8.0, -5.8, pri, 0.5);

    // Controls footer
    render_text(engine, "[N] Name  [Enter] Confirm  [Esc] Back", -10.0, -9.0, dim, 0.4);
}

// ── Helper ──────────────────────────────────────────────────────────────────

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color,
            emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
