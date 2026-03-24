//! Character creation screen — class, background, difficulty, name.

use proof_engine::prelude::*;
use chaos_rpg_core::character::{Character, CharacterClass, Background, Difficulty};
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

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

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    if state.cc_name_active {
        // Name editor: capture letter keys
        let letters = [
            (Key::A,'A'),(Key::B,'B'),(Key::C,'C'),(Key::D,'D'),(Key::E,'E'),
            (Key::F,'F'),(Key::G,'G'),(Key::H,'H'),(Key::I,'I'),(Key::J,'J'),
            (Key::K,'K'),(Key::L,'L'),(Key::M,'M'),(Key::N,'N'),(Key::O,'O'),
            (Key::P,'P'),(Key::Q,'Q'),(Key::R,'R'),(Key::S,'S'),(Key::T,'T'),
            (Key::U,'U'),(Key::V,'V'),(Key::W,'W'),(Key::X,'X'),(Key::Y,'Y'),(Key::Z,'Z'),
        ];
        for &(key, ch) in &letters {
            if engine.input.just_pressed(key) && state.cc_name.len() < 16 {
                state.cc_name.push(ch);
            }
        }
        if engine.input.just_pressed(Key::Space) && state.cc_name.len() < 16 {
            state.cc_name.push(' ');
        }
        if engine.input.just_pressed(Key::Backspace) { state.cc_name.pop(); }
        if engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Escape) {
            state.cc_name_active = false;
        }
        return;
    }

    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let left = engine.input.just_pressed(Key::Left);
    let right = engine.input.just_pressed(Key::Right);
    let tab = engine.input.just_pressed(Key::Tab);
    let n_key = engine.input.just_pressed(Key::N);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let esc = engine.input.just_pressed(Key::Escape);

    if up { state.cc_class = (state.cc_class + CLASSES.len() - 1) % CLASSES.len(); }
    if down { state.cc_class = (state.cc_class + 1) % CLASSES.len(); }
    if left { state.cc_bg = (state.cc_bg + BACKGROUNDS.len() - 1) % BACKGROUNDS.len(); }
    if right { state.cc_bg = (state.cc_bg + 1) % BACKGROUNDS.len(); }
    if tab { state.cc_diff = (state.cc_diff + 1) % DIFFICULTIES.len(); }
    if n_key { state.cc_name_active = true; }
    if esc { state.screen = AppScreen::ModeSelect; }

    if enter {
        let name = if state.cc_name.is_empty() { "Hero".to_string() } else { state.cc_name.clone() };
        let player = Character::roll_new(name, CLASSES[state.cc_class], BACKGROUNDS[state.cc_bg], state.seed, DIFFICULTIES[state.cc_diff]);
        state.player = Some(player);
        state.screen = AppScreen::BoonSelect;
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    ui_render::heading_centered(engine, "CHARACTER CREATION", 5.0, theme.heading);

    // Name field
    let name_display = if state.cc_name.is_empty() { "<press N to edit>" } else { &state.cc_name };
    let name_color = if state.cc_name_active { theme.accent } else { theme.primary };
    ui_render::text(engine, &format!("Name: {}|", name_display), -6.0, 3.8, name_color, 0.4, 0.6);

    // Three columns: Class (left), Background (center), Difficulty (right)
    let col1 = -8.0;
    let col2 = -2.0;
    let col3 = 3.5;

    // Class column
    ui_render::text(engine, "CLASS [Up/Down]", col1, 2.8, theme.heading, 0.3, 0.6);
    for (i, cls) in CLASSES.iter().enumerate() {
        let sel = i == state.cc_class;
        let marker = if sel { ">" } else { " " };
        let color = if sel { theme.selected } else { theme.dim };
        ui_render::text(engine, &format!("{} {}", marker, cls.name()), col1, 2.0 - i as f32 * 0.45, color, 0.3, if sel { 0.7 } else { 0.25 });
    }

    // Background column
    ui_render::text(engine, "BG [Left/Right]", col2, 2.8, theme.heading, 0.3, 0.6);
    for (i, bg) in BACKGROUNDS.iter().enumerate() {
        let sel = i == state.cc_bg;
        let marker = if sel { ">" } else { " " };
        let color = if sel { theme.selected } else { theme.dim };
        ui_render::text(engine, &format!("{} {}", marker, bg.name()), col2, 2.0 - i as f32 * 0.45, color, 0.3, if sel { 0.7 } else { 0.25 });
    }

    // Difficulty column
    ui_render::text(engine, "DIFF [Tab]", col3, 2.8, theme.heading, 0.3, 0.6);
    for (i, diff) in DIFFICULTIES.iter().enumerate() {
        let sel = i == state.cc_diff;
        let marker = if sel { ">" } else { " " };
        let color = if sel { theme.selected } else { theme.dim };
        ui_render::text(engine, &format!("{} {}", marker, diff.name()), col3, 2.0 - i as f32 * 0.45, color, 0.3, if sel { 0.7 } else { 0.25 });
    }

    // Preview panel
    let cls = CLASSES[state.cc_class];
    let bg = BACKGROUNDS[state.cc_bg];
    let diff = DIFFICULTIES[state.cc_diff];

    ui_render::text(engine, &format!("-- {} --", cls.name()), -8.0, -3.5, theme.accent, 0.35, 0.6);
    // Truncate description to fit
    let desc: String = cls.description().chars().take(60).collect();
    ui_render::small(engine, &desc, -8.0, -4.0, theme.primary);
    ui_render::small(engine, &format!("Passive: {}", cls.passive_name()), -8.0, -4.5, theme.primary);

    ui_render::text(engine, &format!("-- {} --", bg.name()), 0.0, -3.5, theme.accent, 0.35, 0.6);
    let bg_desc: String = bg.description().chars().take(40).collect();
    ui_render::small(engine, &bg_desc, 0.0, -4.0, theme.primary);

    ui_render::small(engine, "[N] Edit name | Enter/Space to confirm | Esc back", -7.0, -5.2, theme.muted);
}
