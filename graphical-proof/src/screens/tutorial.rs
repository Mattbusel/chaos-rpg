//! Tutorial — 5 slides explaining the chaos math.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

const SLIDES: &[(&str, &[&str])] = &[
    ("WHAT IS CHAOS RPG?", &[
        "A roguelike where every outcome is determined by",
        "chaining mathematical algorithms. No dice. Pure chaos.",
        "",
        "Your stats feed into 10 math engines. Their combined",
        "output determines damage, crits, dodges, and events.",
        "",
        "Higher stats mean better inputs. Better inputs mean",
        "better outputs. But chaos is unpredictable by definition.",
    ]),
    ("THE CHAOS PIPELINE", &[
        "Each action chains 4-8 of these 10 engines:",
        "",
        "  Linear    - proportional scaling (stable)",
        "  Lorenz    - chaotic attractor (wild swings)",
        "  Mandelbrot - fractal boundary (crit or bust)",
        "  Zeta      - Riemann zeta (oscillating)",
        "  Collatz   - 3n+1 conjecture (up then down)",
        "  Fibonacci - golden ratio (moderate, reliable)",
        "  SharpEdge - step function (all or nothing)",
        "  Orbit     - elliptical (periodic outcomes)",
        "  Recursive - self-referencing (amplifies)",
        "  Euler     - exponential growth/decay",
    ]),
    ("COMBAT", &[
        "[A] Attack - force-based melee damage",
        "[H] Heavy  - 2x damage, accuracy roll",
        "[D] Defend - reduce incoming damage next turn",
        "[T] Taunt  - provoke enemy, may stun",
        "[F] Flee   - chaos-rolled escape attempt",
        "[1-8] Cast spells (costs mana)",
        "[Q/W/E/R/Y/U/I/O] Use inventory items",
        "",
        "Crits happen when the pipeline output > 0.8",
        "Catastrophes happen when output < -0.8",
    ]),
    ("CORRUPTION & HUNGER", &[
        "Every kill corrupts the chaos engines.",
        "At 100 kills: engines begin to mutate.",
        "At 400 kills: mathematics barely resembles itself.",
        "",
        "On floor 50+, The Hunger activates:",
        "5 rooms without a kill = lose 5% max HP.",
        "The equations demand blood.",
        "",
        "Corruption is permanent. It makes the game",
        "progressively stranger. Embrace it.",
    ]),
    ("TIPS", &[
        "Press [V] in combat to see the chaos engine trace.",
        "Press [C] for your character sheet anytime.",
        "Press [T] on the title screen to change themes.",
        "Press [N] on the floor map for the passive tree.",
        "",
        "Crafting can destroy your items. Corrupt wisely.",
        "The Nemesis system remembers how you died.",
        "Misery is a resource, not just a statistic.",
        "",
        "Good luck. The mathematics are not on your side.",
    ]),
];

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let left = engine.input.just_pressed(Key::Left);
    let right = engine.input.just_pressed(Key::Right);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let esc = engine.input.just_pressed(Key::Escape);

    if (right || enter) && state.tutorial_slide < SLIDES.len() - 1 {
        state.tutorial_slide += 1;
    } else if enter && state.tutorial_slide == SLIDES.len() - 1 {
        // On last slide, Enter/Space exits
        state.tutorial_slide = 0;
        state.screen = AppScreen::Title;
    }
    if left && state.tutorial_slide > 0 {
        state.tutorial_slide -= 1;
    }
    if esc {
        state.tutorial_slide = 0;
        state.screen = AppScreen::Title;
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    ui_render::screen_backing(engine, 0.6);
    let slide_idx = state.tutorial_slide.min(SLIDES.len() - 1);
    let (slide_title, lines) = SLIDES[slide_idx];

    // Slide counter
    ui_render::heading_centered(engine, &format!("TUTORIAL {}/{}", slide_idx + 1, SLIDES.len()), 5.0, theme.heading);

    // Progress dots
    let mut dots = String::new();
    for i in 0..SLIDES.len() {
        dots.push(if i == slide_idx { '#' } else { 'o' });
        dots.push(' ');
    }
    ui_render::text_centered(engine, &dots, 4.2, theme.accent, 0.3, 0.5);

    // Title
    ui_render::body(engine, slide_title, -7.5, 3.3, theme.selected);

    // Content lines
    for (i, line) in lines.iter().enumerate() {
        let truncated: String = line.chars().take(50).collect();
        ui_render::small(engine, &truncated, -7.5, 2.4 - i as f32 * 0.42, theme.primary);
    }

    // Navigation hints
    let nav = if slide_idx == 0 {
        "[Right/Enter/Space] Next  [Esc] Back"
    } else if slide_idx == SLIDES.len() - 1 {
        "[Left] Previous  [Enter/Space/Esc] Done"
    } else {
        "[Left] Prev  [Right/Enter/Space] Next  [Esc] Back"
    };
    ui_render::small(engine, nav, -7.5, -5.2, theme.muted);
}
