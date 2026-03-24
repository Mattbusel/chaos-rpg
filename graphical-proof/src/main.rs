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
pub mod magic;
mod scenes;
mod audio_bridge;
mod music_bridge;
mod anim_bridge;
pub mod lighting;
pub mod shader_presets;
pub mod cinematics;
pub mod physics_bridge;
pub mod dungeon_bridge;
pub mod boss_bridge;
pub mod enemy_ai;
pub mod weather_system;
pub mod terrain_map;
pub mod game_economy;
pub mod dialogue_system;
pub mod mod_system;
pub mod replay_system;
pub mod save_upgrade;
pub mod debug_tools;
pub mod ui_render;
pub mod gpu_chaos;
pub mod combat_visuals;
pub mod combat_hud;
pub mod exploration;

use state::{AppScreen, GameState};
use theme::THEMES;
use debug_tools::DebugToolsManager;
use gpu_chaos::ChaosComputeManager;

// ── ProofGame implementation ─────────────────────────────────────────────────

struct ChaosRpgGame {
    state: GameState,
    music_bridge: music_bridge::MusicBridge,
    anim_bridge: anim_bridge::AnimBridge,
    debug_tools: DebugToolsManager,
    chaos_compute: ChaosComputeManager,
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
        // X-mirror fix is in the engine pipeline.rs (flip_x matrix).
        // Camera stays at default (0,0,10) looking at origin.

        // Initialize the chaos field background
        scenes::chaos_field::init(&self.state, engine);
    }

    fn update(&mut self, engine: &mut ProofEngine, dt: f32) {
        // CRITICAL: Clear all glyphs from previous frame.
        // We do immediate-mode rendering — every glyph is spawned fresh each frame.
        // Without this, glyphs accumulate and FPS drops to zero within seconds.
        engine.scene.glyphs = proof_engine::glyph::GlyphPool::new(8192);

        // Handle debug tool input BEFORE game input (F-keys, console, etc.)
        let debug_consumed = self.debug_tools.handle_input(engine);

        // Tick visual timers
        self.state.tick_timers(dt);

        // Update debug tools (execute pending commands, tick overlays)
        self.debug_tools.update(dt, &mut self.state, engine);

        // Update music vibe based on current screen
        audio_bridge::update_music_vibe(&self.state, engine);

        // Update music bridge (procedural music director)
        let music_effects = self.music_bridge.update(dt, &self.state);
        music_bridge::apply_effects_to_engine(&music_effects, engine);

        // Update animation bridge (entity animation state machines)
        let anim_transforms = self.anim_bridge.update(dt, &self.state);
        let _ = &anim_transforms; // transforms are applied by screen-specific renderers

        // Update GPU chaos field (replaces old CPU chaos field)
        self.chaos_compute.set_floor_theme(
            self.state.floor_num,
            self.state.corruption_frac() * 500.0,
        );
        self.chaos_compute.update(dt);
        self.chaos_compute.render(engine);

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
            AppScreen::Crafting => {
                screens::crafting::update(&mut self.state, engine, dt);
                screens::crafting::render(&self.state, engine);
            }
            AppScreen::PassiveTree => {
                screens::passive_tree::update(&mut self.state, engine, dt);
                screens::passive_tree::render(&self.state, engine);
            }
            AppScreen::Achievements => {
                screens::achievements::update(&mut self.state, engine, dt);
                screens::achievements::render(&self.state, engine);
            }
            AppScreen::RunHistory => {
                screens::run_history::update(&mut self.state, engine, dt);
                screens::run_history::render(&self.state, engine);
            }
            AppScreen::DailyLeaderboard => {
                screens::daily_leaderboard::update(&mut self.state, engine, dt);
                screens::daily_leaderboard::render(&self.state, engine);
            }
            AppScreen::Bestiary => {
                screens::bestiary::update(&mut self.state, engine, dt);
                screens::bestiary::render(&self.state, engine);
            }
            AppScreen::Codex => {
                screens::codex::update(&mut self.state, engine, dt);
                screens::codex::render(&self.state, engine);
            }
            AppScreen::Settings => {
                screens::settings::update(&mut self.state, engine, dt);
                screens::settings::render(&self.state, engine);
            }
            AppScreen::Tutorial => {
                screens::tutorial::update(&mut self.state, engine, dt);
                screens::tutorial::render(&self.state, engine);
            }
            AppScreen::Scoreboard => {
                screens::scoreboard::update(&mut self.state, engine, dt);
                screens::scoreboard::render(&self.state, engine);
            }
        }

        // Apply screen shake from combat hits
        if self.state.hit_shake > 0.0 {
            engine.add_trauma(self.state.hit_shake * dt * 2.0);
        }

        // Render debug overlay on top of everything
        self.debug_tools.submit_to_engine(engine);
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
        music_bridge: music_bridge::MusicBridge::init(),
        anim_bridge: anim_bridge::AnimBridge::init(),
        debug_tools: DebugToolsManager::new(),
        chaos_compute: ChaosComputeManager::init_auto(),
    };
    ProofEngine::run_game(game);
}
