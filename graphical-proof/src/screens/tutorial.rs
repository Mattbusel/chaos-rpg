//! Tutorial — 5 slides explaining the chaos math.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

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
        "  Linear    — proportional scaling (boring but stable)",
        "  Lorenz    — chaotic attractor (wild swings)",
        "  Mandelbrot — fractal boundary (crit or catastrophe)",
        "  Zeta      — Riemann zeta (oscillating extremes)",
        "  Collatz   — 3n+1 conjecture (up then down)",
        "  Fibonacci — golden ratio (moderate, reliable)",
        "  SharpEdge — step function (all or nothing)",
        "  Orbit     — elliptical (periodic outcomes)",
        "  Recursive — self-referencing (amplifies trends)",
        "  Euler     — exponential growth/decay",
    ]),
    ("COMBAT", &[
        "[A] Attack — force-based melee damage",
        "[H] Heavy  — 2x damage, accuracy roll",
        "[D] Defend — reduce incoming damage next turn",
        "[T] Taunt  — provoke enemy, may stun",
        "[F] Flee   — chaos-rolled escape attempt",
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
    let right = engine.input.just_pressed(Key::Right) || engine.input.just_pressed(Key::Space) || engine.input.just_pressed(Key::Enter);
    let esc = engine.input.just_pressed(Key::Escape);

    if right && state.tutorial_slide < SLIDES.len() - 1 { state.tutorial_slide += 1; }
    if left && state.tutorial_slide > 0 { state.tutorial_slide -= 1; }
    if esc { state.tutorial_slide = 0; state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let slide_idx = state.tutorial_slide.min(SLIDES.len() - 1);
    let (title, lines) = SLIDES[slide_idx];

    // Slide counter
    render_text(engine, &format!("TUTORIAL — {}/{}", slide_idx + 1, SLIDES.len()),
        -18.0, 9.0, theme.heading, 0.8);

    // Progress dots
    let mut dots = String::new();
    for i in 0..SLIDES.len() {
        dots.push(if i == slide_idx { '●' } else { '○' });
        dots.push(' ');
    }
    render_text(engine, &dots, -4.0, 7.5, theme.accent, 0.5);

    // Title
    render_text(engine, title, -12.0, 6.0, theme.selected, 0.8);

    // Content lines
    for (i, line) in lines.iter().enumerate() {
        render_text(engine, line, -14.0, 4.0 - i as f32 * 1.0, theme.primary, 0.4);
    }

    // Navigation hints
    let nav = if slide_idx == 0 {
        "[Right/Space/Enter] Next  [Esc] Back"
    } else if slide_idx == SLIDES.len() - 1 {
        "[Left] Previous  [Esc] Done"
    } else {
        "[Left] Previous  [Right/Space/Enter] Next  [Esc] Back"
    };
    render_text(engine, nav, -14.0, -12.0, theme.muted, 0.2);
}

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color, emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
