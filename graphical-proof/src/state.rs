//! Game state for the proof-engine frontend.
//!
//! Ports the `State` struct from `graphical/src/main.rs` — every field the
//! renderer needs lives here. Game logic stays in `chaos-rpg-core`.

use proof_engine::prelude::{Vec4};

use chaos_rpg_core::{
    achievements::AchievementStore,
    character::{Boon, Character, CharacterClass, Difficulty},
    chaos_config::ChaosConfig,
    chaos_pipeline::ChaosRollResult,
    combat::{CombatAction, CombatState},
    daily_leaderboard::{LocalDailyStore, LeaderboardRow},
    enemy::Enemy,
    items::Item,
    nemesis::NemesisRecord,
    run_history::RunHistory,
    spells::Spell,
    world::{Floor, RoomType},
};

// ── Screen enum ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Title,
    Tutorial,
    ModeSelect,
    CharacterCreation,
    BoonSelect,
    FloorNav,
    RoomView,
    Combat,
    Shop,
    Crafting,
    CharacterSheet,
    BodyChart,
    PassiveTree,
    GameOver,
    Victory,
    Scoreboard,
    Achievements,
    RunHistory,
    DailyLeaderboard,
    Bestiary,
    Codex,
    Settings,
}

// ── Game mode ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    Story,
    Infinite,
    Daily,
}

// ── Crafting phase ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CraftPhase {
    SelectItem,
    SelectOp,
}

// ── Room event ───────────────────────────────────────────────────────────────

pub struct RoomEvent {
    pub title: String,
    pub lines: Vec<String>,
    pub pending_item: Option<Item>,
    pub pending_spell: Option<Spell>,
    pub gold_delta: i64,
    pub hp_delta: i64,
    pub damage_taken: i64,
    pub stat_bonuses: Vec<(&'static str, i64)>,
    pub portal_available: bool,
    pub resolved: bool,
}

impl RoomEvent {
    pub fn empty() -> Self {
        Self {
            title: String::new(),
            lines: Vec::new(),
            pending_item: None,
            pending_spell: None,
            gold_delta: 0,
            hp_delta: 0,
            damage_taken: 0,
            stat_bonuses: Vec::new(),
            portal_available: false,
            resolved: false,
        }
    }
}

// ── Save / Load ──────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub player: Character,
    pub floor: Option<Floor>,
    pub floor_num: u32,
    pub floor_seed: u64,
    pub seed: u64,
    pub current_mana: i64,
    pub is_boss_fight: bool,
    pub game_mode: String,
    pub nemesis_spawned: bool,
    pub combat_log: Vec<String>,
}

pub fn save_path() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap_or_default();
    p.pop();
    p.push("chaos_rpg_save.json");
    p
}

pub fn write_save(s: &SaveState) {
    if let Ok(json) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(save_path(), json);
    }
}

pub fn read_save() -> Option<SaveState> {
    let data = std::fs::read_to_string(save_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn delete_save() {
    let _ = std::fs::remove_file(save_path());
}

// ── Main game state ──────────────────────────────────────────────────────────

pub struct GameState {
    // ── Current screen ──
    pub screen: AppScreen,

    // ── Core game objects ──
    pub player: Option<Character>,
    pub floor: Option<Floor>,
    pub enemy: Option<Enemy>,
    pub combat_state: Option<CombatState>,
    pub last_roll: Option<ChaosRollResult>,
    pub combat_log: Vec<String>,
    pub seed: u64,
    pub floor_seed: u64,
    pub frame: u64,

    // ── Character creation ──
    pub selected_menu: usize,
    pub cc_class: usize,
    pub cc_bg: usize,
    pub cc_diff: usize,
    pub cc_name: String,
    pub cc_name_active: bool,

    // ── Mode select ──
    pub mode_cursor: usize,
    pub game_mode: GameMode,

    // ── Boon select ──
    pub boon_options: [Boon; 3],
    pub boon_cursor: usize,

    // ── Floor state ──
    pub floor_num: u32,
    pub max_floor: u32,
    pub is_cursed_floor: bool,

    // ── Nemesis ──
    pub nemesis_record: Option<NemesisRecord>,
    pub nemesis_spawned: bool,

    // ── Combat extras ──
    pub is_boss_fight: bool,
    pub gauntlet_stage: u8,
    pub gauntlet_enemies: Vec<Enemy>,
    pub loot_pending: Option<Item>,
    pub current_mana: i64,

    // ── Room event ──
    pub room_event: RoomEvent,

    // ── Shop state ──
    pub shop_items: Vec<(Item, i64)>,
    pub shop_heal_cost: i64,
    pub shop_cursor: usize,

    // ── Crafting state ──
    pub craft_phase: CraftPhase,
    pub craft_item_cursor: usize,
    pub craft_op_cursor: usize,
    pub craft_message: String,

    // ── Visual theme ──
    pub theme_idx: usize,

    // ── Auto-play mode ──
    pub auto_mode: bool,
    pub auto_last_action: u64,

    // ── Visual effects (proof-engine driven) ──
    pub player_flash: f32,
    pub enemy_flash: f32,
    pub enemy_flash_color: Vec4,
    pub hit_shake: f32,
    pub spell_beam_timer: f32,
    pub spell_beam_color: Vec4,

    // ── HP ghost bars ──
    pub ghost_player_hp: f32,
    pub ghost_player_timer: f32,
    pub ghost_enemy_hp: f32,
    pub ghost_enemy_timer: f32,

    // ── Kill linger ──
    pub kill_linger: f32,
    pub post_combat_screen: Option<AppScreen>,

    // ── Smooth HP/MP display ──
    pub display_player_hp: f32,
    pub display_enemy_hp: f32,
    pub display_mp: f32,

    // ── Passive tree browser ──
    pub passive_scroll: usize,

    // ── Tutorial ──
    pub tutorial_slide: usize,
    pub save_exists: bool,

    // ── Achievements ──
    pub achievements: AchievementStore,
    pub achievement_banner: Option<String>,
    pub achievement_banner_timer: f32,

    // ── Run history ──
    pub run_history: RunHistory,
    pub history_scroll: usize,

    // ── Bestiary / Codex ──
    pub bestiary_scroll: usize,
    pub bestiary_selected: usize,
    pub codex_scroll: usize,
    pub codex_selected: usize,

    // ── Achievement scroll/filter ──
    pub achievement_scroll: usize,
    pub achievement_filter: u8,

    // ── Death recap ──
    pub last_recap_text: String,

    // ── Chaos engine viz overlay ──
    pub chaos_viz_open: bool,

    // ── Item filter (crafting) ──
    pub item_filter: String,
    pub item_filter_active: bool,

    // ── Config ──
    pub config: ChaosConfig,

    // ── Daily leaderboard ──
    pub daily_store: LocalDailyStore,
    pub daily_rows: Vec<LeaderboardRow>,
    pub daily_status: String,
    pub daily_submitted: bool,

    // ── Boss state ──
    pub boss_id: Option<u8>,
    pub boss_turn: u32,
    pub boss_extra: i64,
    pub boss_extra2: i64,

    // ── Floor transition ──
    pub floor_transition_timer: f32,
    pub floor_transition_floor: u32,

    // ── Boss entrance ──
    pub boss_entrance_timer: f32,
    pub boss_entrance_name: String,

    // ── Craft animation ──
    pub craft_anim_timer: f32,
    pub craft_anim_type: u8,

    // ── Title logo ──
    pub title_logo_timer: f32,

    // ── Character sheet tabs ──
    pub char_tab: u8,

    // ── Combat log collapse ──
    pub combat_log_collapsed: bool,

    // ── Death cinematic ──
    pub death_cinematic_done: bool,

    // ── Last combat action (for animation selection) ──
    pub last_action_type: u8,
    pub last_spell_name: String,

    // ── Room entry animation ──
    pub room_entry_timer: f32,
    pub room_entry_type: u8,

    // ── Screen dimensions ──
    pub screen_width: u32,
    pub screen_height: u32,
}

impl GameState {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        let cfg = ChaosConfig::load();

        GameState {
            screen: AppScreen::Title,
            player: None,
            floor: None,
            enemy: None,
            combat_state: None,
            last_roll: None,
            combat_log: Vec::new(),
            seed,
            floor_seed: seed,
            frame: 0,

            selected_menu: 0,
            cc_class: 0,
            cc_bg: 0,
            cc_diff: 1,
            cc_name: String::new(),
            cc_name_active: false,

            mode_cursor: 0,
            game_mode: GameMode::Infinite,

            boon_options: Boon::random_three(seed),
            boon_cursor: 0,

            floor_num: 1,
            max_floor: u32::MAX,
            is_cursed_floor: false,

            nemesis_record: None,
            nemesis_spawned: false,

            is_boss_fight: false,
            gauntlet_stage: 0,
            gauntlet_enemies: Vec::new(),
            loot_pending: None,
            current_mana: 0,

            room_event: RoomEvent::empty(),

            shop_items: Vec::new(),
            shop_heal_cost: 20,
            shop_cursor: 0,

            craft_phase: CraftPhase::SelectItem,
            craft_item_cursor: 0,
            craft_op_cursor: 0,
            craft_message: String::new(),

            theme_idx: 0,

            auto_mode: false,
            auto_last_action: 0,

            player_flash: 0.0,
            enemy_flash: 0.0,
            enemy_flash_color: Vec4::new(0.3, 0.86, 0.3, 1.0),
            hit_shake: 0.0,
            spell_beam_timer: 0.0,
            spell_beam_color: Vec4::new(0.31, 0.47, 1.0, 1.0),

            ghost_player_hp: 1.0,
            ghost_player_timer: 0.0,
            ghost_enemy_hp: 1.0,
            ghost_enemy_timer: 0.0,

            kill_linger: 0.0,
            post_combat_screen: None,

            display_player_hp: 1.0,
            display_enemy_hp: 1.0,
            display_mp: 0.0,

            passive_scroll: 0,

            tutorial_slide: 0,
            save_exists: read_save().is_some(),

            achievements: AchievementStore::load(),
            achievement_banner: None,
            achievement_banner_timer: 0.0,

            run_history: RunHistory::load(),
            history_scroll: 0,

            bestiary_scroll: 0,
            bestiary_selected: 0,
            codex_scroll: 0,
            codex_selected: 0,

            achievement_scroll: 0,
            achievement_filter: 0,

            last_recap_text: String::new(),

            chaos_viz_open: false,

            item_filter: String::new(),
            item_filter_active: false,

            config: cfg,

            daily_store: LocalDailyStore::load(),
            daily_rows: Vec::new(),
            daily_status: String::new(),
            daily_submitted: false,

            boss_id: None,
            boss_turn: 0,
            boss_extra: 0,
            boss_extra2: 0,

            floor_transition_timer: 0.0,
            floor_transition_floor: 0,

            boss_entrance_timer: 0.0,
            boss_entrance_name: String::new(),

            craft_anim_timer: 0.0,
            craft_anim_type: 0,

            title_logo_timer: 1.5,

            char_tab: 0,

            combat_log_collapsed: false,

            death_cinematic_done: false,

            last_action_type: 0,
            last_spell_name: String::new(),

            room_entry_timer: 0.0,
            room_entry_type: 0,

            screen_width: 1280,
            screen_height: 800,
        }
    }

    /// Tick visual timers (seconds-based instead of frame-based).
    pub fn tick_timers(&mut self, dt: f32) {
        self.frame += 1;

        if self.player_flash > 0.0 {
            self.player_flash = (self.player_flash - dt).max(0.0);
        }
        if self.enemy_flash > 0.0 {
            self.enemy_flash = (self.enemy_flash - dt).max(0.0);
        }
        if self.hit_shake > 0.0 {
            self.hit_shake = (self.hit_shake - dt).max(0.0);
        }
        if self.spell_beam_timer > 0.0 {
            self.spell_beam_timer = (self.spell_beam_timer - dt).max(0.0);
        }
        if self.kill_linger > 0.0 {
            self.kill_linger = (self.kill_linger - dt).max(0.0);
            if self.kill_linger <= 0.0 {
                if let Some(next) = self.post_combat_screen.take() {
                    self.screen = next;
                }
            }
        }
        if self.ghost_player_timer > 0.0 {
            self.ghost_player_timer = (self.ghost_player_timer - dt).max(0.0);
        }
        if self.ghost_enemy_timer > 0.0 {
            self.ghost_enemy_timer = (self.ghost_enemy_timer - dt).max(0.0);
        }
        if self.achievement_banner_timer > 0.0 {
            self.achievement_banner_timer = (self.achievement_banner_timer - dt).max(0.0);
            if self.achievement_banner_timer <= 0.0 {
                self.achievement_banner = None;
            }
        }
        if self.floor_transition_timer > 0.0 {
            self.floor_transition_timer = (self.floor_transition_timer - dt).max(0.0);
        }
        if self.boss_entrance_timer > 0.0 {
            self.boss_entrance_timer = (self.boss_entrance_timer - dt).max(0.0);
        }
        if self.craft_anim_timer > 0.0 {
            self.craft_anim_timer = (self.craft_anim_timer - dt).max(0.0);
        }
        if self.title_logo_timer > 0.0 {
            self.title_logo_timer = (self.title_logo_timer - dt).max(0.0);
        }
        if self.room_entry_timer > 0.0 {
            self.room_entry_timer = (self.room_entry_timer - dt).max(0.0);
        }

        // Smooth HP/MP interpolation (8% per frame at 60fps ≈ lerp factor)
        let lerp_speed = 1.0 - (1.0 - 0.08_f32).powf(dt * 60.0);
        if let Some(ref player) = self.player {
            let target_hp = player.current_hp as f32 / player.max_hp.max(1) as f32;
            self.display_player_hp += (target_hp - self.display_player_hp) * lerp_speed;
            let max_mp = self.max_mana();
            let target_mp = if max_mp > 0 {
                self.current_mana as f32 / max_mp as f32
            } else {
                0.0
            };
            self.display_mp += (target_mp - self.display_mp) * lerp_speed;
        }
        if let Some(ref enemy) = self.enemy {
            let target = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
            self.display_enemy_hp += (target - self.display_enemy_hp) * lerp_speed;
        }
    }

    /// Max mana for the current player (same formula as graphical frontend).
    pub fn max_mana(&self) -> i64 {
        self.player
            .as_ref()
            .map(|p| (p.stats.mana + 50).max(50))
            .unwrap_or(50)
    }

    /// Current corruption percentage (0.0 - 1.0).
    pub fn corruption_frac(&self) -> f32 {
        self.player
            .as_ref()
            .map(|p| (p.kills as f32 / 400.0).clamp(0.0, 1.0))
            .unwrap_or(0.0)
    }
}
