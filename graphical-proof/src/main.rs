//! CHAOS RPG — Proof Engine Frontend
//!
//! Full graphical frontend powered by the mathematical rendering engine.
//! Every visual is the output of a mathematical function.

use proof_engine::prelude::*;
use proof_engine::integration::ProofGame;

mod state;
mod theme;
mod screens;
mod entities;
mod effects;
mod scenes;

use state::{AppScreen, GameState};
use theme::THEMES;

// ── ProofGame implementation ─────────────────────────────────────────────────

struct ChaosRpgGame {
    state: GameState,
}

impl ProofGame for ChaosRpgGame {
    fn title(&self) -> &str {
        "CHAOS RPG \u{2014} Where Math Goes To Die"
    }

    fn config(&self) -> EngineConfig {
        EngineConfig {
            window_title: self.title().to_string(),
            window_width: 1280,
            window_height: 800,
            target_fps: 60,
            ..EngineConfig::default()
        }
    }

    fn on_start(&mut self, engine: &mut ProofEngine) {
        // Initialize the chaos field background
        scenes::chaos_field::init(&self.state, engine);
    }

    fn update(&mut self, engine: &mut ProofEngine, dt: f32) {
        // Tick visual timers
        self.state.tick_timers(dt);

        // Update chaos field (always running)
        scenes::chaos_field::update(&self.state, engine, dt);

        // Screen-specific update + render
        match self.state.screen {
            AppScreen::Title => {
                screens::title::update(&mut self.state, engine, dt);
                screens::title::render(&self.state, engine);
            }
            AppScreen::CharacterCreation => {
                screens::character_creation::update(&mut self.state, engine, dt);
                screens::character_creation::render(&self.state, engine);
            }
            AppScreen::Combat => {
                screens::combat::update(&mut self.state, engine, dt);
                screens::combat::render(&self.state, engine);
            }
            AppScreen::FloorNav => {
                screens::floor_nav::update(&mut self.state, engine, dt);
                screens::floor_nav::render(&self.state, engine);
            }
            AppScreen::CharacterSheet | AppScreen::BodyChart => {
                screens::character_sheet::update(&mut self.state, engine, dt);
                screens::character_sheet::render(&self.state, engine);
            }
            // All other screens get a minimal fallback for now
            _ => {
                render_fallback_screen(&self.state, engine);
            }
        }

        // Apply screen shake from combat hits
        if self.state.hit_shake > 0.0 {
            engine.add_trauma(self.state.hit_shake * dt * 2.0);
        }
    }

    fn on_resize(&mut self, _engine: &mut ProofEngine, width: u32, height: u32) {
        self.state.screen_width = width;
        self.state.screen_height = height;
    }

    fn on_stop(&mut self, _engine: &mut ProofEngine) {
        // Auto-save on clean exit if in a run
        if self.state.player.is_some() && self.state.floor.is_some() {
            if let Some(ref player) = self.state.player {
                let save = state::SaveState {
                    player: player.clone(),
                    floor: self.state.floor.clone(),
                    floor_num: self.state.floor_num,
                    floor_seed: self.state.floor_seed,
                    seed: self.state.seed,
                    current_mana: self.state.current_mana,
                    is_boss_fight: self.state.is_boss_fight,
                    game_mode: match self.state.game_mode {
                        state::GameMode::Story => "Story".to_string(),
                        state::GameMode::Infinite => "Infinite".to_string(),
                        state::GameMode::Daily => "Daily".to_string(),
                    },
                    nemesis_spawned: self.state.nemesis_spawned,
                    combat_log: self.state.combat_log.clone(),
                };
                state::write_save(&save);
            }
        }
    }
}

/// Minimal fallback for screens not yet ported — just show the screen name.
fn render_fallback_screen(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let label = format!("{:?} — Press [Esc] to return", state.screen);
    // Spawn a temporary label glyph at screen center
    for (i, ch) in label.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(-12.0 + i as f32 * 0.6, 0.0, 0.0),
            color: theme.heading,
            emission: 0.6,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // Handle Escape to go back to title
    if engine.input.just_pressed(Key::Escape) {
        // This is a simplification — each screen has its own back behavior
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() {
    let game = ChaosRpgGame {
        state: GameState::new(),
    };
    ProofEngine::run_game(game);
}
