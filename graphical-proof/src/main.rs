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
mod audio_bridge;
pub mod lighting;

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

        // Update music vibe based on current screen
        audio_bridge::update_music_vibe(&self.state, engine);

        // Update chaos field (always running)
        scenes::chaos_field::update(&self.state, engine, dt);

        // Screen-specific update + render
        match self.state.screen {
            AppScreen::Title => {
                screens::title::update(&mut self.state, engine, dt);
                screens::title::render(&self.state, engine);
            }
            AppScreen::ModeSelect => {
                screens::mode_select::update(&mut self.state, engine, dt);
                screens::mode_select::render(&self.state, engine);
            }
            AppScreen::CharacterCreation => {
                screens::character_creation::update(&mut self.state, engine, dt);
                screens::character_creation::render(&self.state, engine);
            }
            AppScreen::BoonSelect => {
                screens::boon_select::update(&mut self.state, engine, dt);
                screens::boon_select::render(&self.state, engine);
            }
            AppScreen::Combat => {
                screens::combat::update(&mut self.state, engine, dt);
                screens::combat::render(&self.state, engine);
            }
            AppScreen::FloorNav => {
                screens::floor_nav::update(&mut self.state, engine, dt);
                screens::floor_nav::render(&self.state, engine);
            }
            AppScreen::RoomView => {
                screens::room_view::update(&mut self.state, engine, dt);
                screens::room_view::render(&self.state, engine);
            }
            AppScreen::Shop => {
                screens::shop::update(&mut self.state, engine, dt);
                screens::shop::render(&self.state, engine);
            }
            AppScreen::CharacterSheet | AppScreen::BodyChart => {
                screens::character_sheet::update(&mut self.state, engine, dt);
                screens::character_sheet::render(&self.state, engine);
            }
            AppScreen::GameOver => {
                screens::game_over::update(&mut self.state, engine, dt);
                screens::game_over::render(&self.state, engine);
            }
            AppScreen::Victory => {
                screens::victory::update(&mut self.state, engine, dt);
                screens::victory::render(&self.state, engine);
            }
            // Meta screens use generic placeholder
            AppScreen::Crafting => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::FloorNav);
                screens::generic::render_placeholder(&self.state, engine,
                    "CRAFTING BENCH", "Select an item and operation.");
            }
            AppScreen::PassiveTree => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::FloorNav);
                screens::generic::render_placeholder(&self.state, engine,
                    "PASSIVE TREE", "820+ nodes across 8 class rings.");
            }
            AppScreen::Achievements => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "ACHIEVEMENTS", "181 achievements to unlock.");
            }
            AppScreen::RunHistory => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "RUN HISTORY", "Your past runs, newest first.");
            }
            AppScreen::DailyLeaderboard => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "DAILY LEADERBOARD", "Today's seeded challenge rankings.");
            }
            AppScreen::Bestiary => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "BESTIARY", "Enemies encountered across all runs.");
            }
            AppScreen::Codex => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "CODEX", "Lore fragments and world knowledge.");
            }
            AppScreen::Settings => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "SETTINGS", "Music vibe, theme, accessibility.");
            }
            AppScreen::Tutorial => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "TUTORIAL", "Learn the chaos math behind everything.");
            }
            AppScreen::Scoreboard => {
                screens::generic::handle_back(&mut self.state, engine, AppScreen::Title);
                screens::generic::render_placeholder(&self.state, engine,
                    "HALL OF CHAOS", "The greatest and most wretched runs.");
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
