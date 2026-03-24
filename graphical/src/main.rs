//! CHAOS RPG — Graphical Frontend (bracket-lib)
//!
//! Full feature parity with the terminal version.
//! Always runs fullscreen. All room types, modes, boons, nemesis, gauntlet,
//! cursed floors, The Hunger, item volatility, crafting (all 6 ops), and
//! real chaos-engine combat via resolve_action().

use bracket_lib::prelude::*;
use chaos_rpg_audio::AudioSystem;
use chaos_rpg_core::{
    audio_events::{AudioEvent, MusicVibe},

    bosses::{boss_name, boss_pool_for_floor, random_unique_boss},
    character::{Background, Boon, Character, CharacterClass, Difficulty},
    chaos_pipeline::{chaos_roll_verbose, destiny_roll, ChaosRollResult},
    combat::{resolve_action, CombatAction, CombatOutcome, CombatState},
    enemy::{generate_enemy, Enemy, FloorAbility},
    items::{Item, Rarity, StatModifier},
    nemesis::{clear_nemesis, load_nemesis, save_nemesis, NemesisRecord},
    npcs::shop_npc,
    scoreboard::{load_scores, load_misery_scores, save_score, ScoreEntry},
    skill_checks::{perform_skill_check, Difficulty as SkillDiff, SkillType},
    spells::Spell,
    world::{generate_floor, room_enemy, Floor, RoomType},
    achievements::{AchievementStore, RunSummary, CombatSnapshot},
    run_history::{RunHistory, RunRecord},
    chaos_config::ChaosConfig,
    daily_leaderboard::{LocalDailyStore, DailyEntry, LeaderboardRow, submit_score, fetch_scores},
};

mod achievement_banner;
mod anim_config;
mod chaos_field;
mod color_grade;
mod combat_anim;
mod death_seq;
mod nemesis_reveal;
mod renderer;
mod sprites;
mod text_effects;
mod theme;
mod tile_effects;
mod ui_overlay;
mod visual_config;
mod weather;
use visual_config as vc;
use achievement_banner::{AchievementBanner, BannerRarity, rarity_from_name};
use anim_config::AnimConfig;
use chaos_field::ChaosField;
use color_grade::ColorGrade;
use combat_anim::{
    CombatAnim, WeaponKind, SpellElement, StatusKind,
    weapon_kind_from_name, spell_element_from_name, status_kind_from_name,
};
use death_seq::DeathSeq;
use nemesis_reveal::NemesisReveal;
use tile_effects::TileEffects;
use weather::{Weather, WeatherType};

// ─── SAVE / LOAD ──────────────────────────────────────────────────────────────

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SaveState {
    player: Character,
    floor: Option<Floor>,
    floor_num: u32,
    floor_seed: u64,
    seed: u64,
    current_mana: i64,
    is_boss_fight: bool,
    game_mode: String,    // "Story" | "Infinite" | "Daily"
    nemesis_spawned: bool,
    combat_log: Vec<String>,
}

fn save_path() -> std::path::PathBuf {
    // Prefer next to the exe; fall back to current dir
    let mut p = std::env::current_exe().unwrap_or_default();
    p.pop();
    p.push("chaos_rpg_save.json");
    p
}

fn write_save(s: &SaveState) {
    if let Ok(json) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(save_path(), json);
    }
}

fn read_save() -> Option<SaveState> {
    let data = std::fs::read_to_string(save_path()).ok()?;
    serde_json::from_str(&data).ok()
}

fn delete_save() {
    let _ = std::fs::remove_file(save_path());
}

use theme::{Theme, THEMES};

// ─── GAME MODE ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameMode { Story, Infinite, Daily }

// ─── CRAFTING PHASE ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum CraftPhase { SelectItem, SelectOp }

// ─── SCREENS ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
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

// ─── ROOM EVENT ───────────────────────────────────────────────────────────────

struct RoomEvent {
    title: String,
    lines: Vec<String>,
    pending_item: Option<Item>,
    pending_spell: Option<Spell>,
    gold_delta: i64,
    hp_delta: i64,
    damage_taken: i64,
    stat_bonuses: Vec<(&'static str, i64)>,
    portal_available: bool,
    resolved: bool,
}

impl RoomEvent {
    fn empty() -> Self {
        Self {
            title: String::new(), lines: Vec::new(),
            pending_item: None, pending_spell: None,
            gold_delta: 0, hp_delta: 0, damage_taken: 0,
            stat_bonuses: Vec::new(),
            portal_available: false, resolved: false,
        }
    }
}

// ─── PARTICLE SYSTEM ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    friction: f32,  // velocity multiplier per frame (1.0 = no friction, 0.85 = high)
    gravity: f32,   // added to vy each frame
    text: String,
    col: (u8, u8, u8),
    age: u32,
    lifetime: u32,
}

impl Particle {
    /// Simple float-up particle (existing API, backwards-compatible).
    fn new(x: i32, y: i32, text: impl Into<String>, col: (u8,u8,u8), lifetime: u32) -> Self {
        Self {
            x: x as f32, y: y as f32,
            vx: 0.0, vy: -visual_config::PARTICLE_DRIFT,
            friction: 1.0, gravity: 0.0,
            text: text.into(), col, age: 0, lifetime,
        }
    }
    /// Burst/explode particle with velocity.
    fn burst(x: f32, y: f32, vx: f32, vy: f32,
             text: impl Into<String>, col: (u8,u8,u8), lifetime: u32) -> Self {
        Self { x, y, vx, vy, friction: 0.90, gravity: 0.04,
               text: text.into(), col, age: 0, lifetime }
    }
    /// Fast spark: high friction, short life.
    fn spark(x: f32, y: f32, vx: f32, vy: f32, text: &'static str, col: (u8,u8,u8)) -> Self {
        Self { x, y, vx, vy, friction: 0.82, gravity: 0.06,
               text: text.to_string(), col, age: 0, lifetime: 18 }
    }
    fn alive(&self) -> bool { self.age < self.lifetime }
    fn step(&mut self) {
        self.age += 1;
        self.x += self.vx;
        self.y += self.vy;
        self.vx *= self.friction;
        self.vy = self.vy * self.friction + self.gravity;
    }
    /// Dim color in the last 40% of lifetime.
    fn render_col(&self) -> (u8, u8, u8) {
        let fade_at = (self.lifetime as f32 * visual_config::PARTICLE_FADE_START) as u32;
        if self.age <= fade_at { return self.col; }
        let pct = 1.0 - (self.age - fade_at) as f32 / (self.lifetime - fade_at).max(1) as f32;
        (
            ((self.col.0 as f32 * pct) as u8).max(10),
            ((self.col.1 as f32 * pct) as u8).max(10),
            ((self.col.2 as f32 * pct) as u8).max(10),
        )
    }
}

// ── Particle emitter helpers ──────────────────────────────────────────────────

fn emit_death_explosion(particles: &mut Vec<Particle>, cx: f32, cy: f32, col: (u8,u8,u8)) {
    const CHARS: &[&str] = &["☠","×","+","·","*","#","!","▓","▒","░","█","▄"];
    use std::f32::consts::TAU;
    for i in 0..40usize {
        let angle = i as f32 * TAU / 40.0;
        let speed = 0.25 + (i % 5) as f32 * 0.12;
        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed * 0.6; // slight squash
        let ch = CHARS[i % CHARS.len()];
        let lt = 30 + (i % 25) as u32;
        particles.push(Particle::burst(cx, cy, vx, vy, ch, col, lt));
    }
}

fn emit_level_up_fountain(particles: &mut Vec<Particle>, cx: f32, cy: f32) {
    const CHARS: &[&str] = &["★","✦","+","·","↑","▲"];
    let col = (255u8, 215u8, 0u8);
    let white = (255u8, 255u8, 230u8);
    for i in 0..30usize {
        let spread = (i as f32 - 15.0) * 0.08;
        let vy = -(0.4 + (i % 5) as f32 * 0.09);
        let c = if i % 3 == 0 { white } else { col };
        particles.push(Particle::burst(cx + spread, cy, spread * 0.3, vy, CHARS[i % CHARS.len()], c, 50 + i as u32));
    }
}

fn emit_crit_burst(particles: &mut Vec<Particle>, cx: f32, cy: f32) {
    let col = (255u8, 215u8, 30u8);
    use std::f32::consts::TAU;
    for i in 0..16usize {
        let angle = i as f32 * TAU / 16.0;
        let speed = 0.18 + (i % 4) as f32 * 0.06;
        particles.push(Particle::spark(cx, cy, angle.cos() * speed, angle.sin() * speed, "✦", col));
    }
}

fn emit_hit_sparks(particles: &mut Vec<Particle>, cx: f32, cy: f32, col: (u8,u8,u8), count: usize) {
    use std::f32::consts::TAU;
    for i in 0..count {
        let angle = i as f32 * TAU / count as f32;
        let speed = 0.12 + (i % 3) as f32 * 0.07;
        particles.push(Particle::spark(cx, cy, angle.cos() * speed, angle.sin() * speed, "·", col));
    }
}

fn emit_loot_sparkle(particles: &mut Vec<Particle>, cx: f32, cy: f32, col: (u8,u8,u8)) {
    const CHARS: &[&str] = &["✦","·","*","+"];
    use std::f32::consts::TAU;
    for i in 0..12usize {
        let angle = i as f32 * TAU / 12.0;
        // Slow orbit: give high friction and small starting velocity
        let r = 1.5 + (i % 3) as f32 * 0.4;
        let vx = angle.cos() * 0.06 * r;
        let vy = angle.sin() * 0.04 * r;
        particles.push(Particle { x: cx + angle.cos() * r, y: cy + angle.sin() * r,
            vx, vy, friction: 1.0, gravity: 0.0,
            text: CHARS[i % CHARS.len()].to_string(), col, age: 0, lifetime: 150 });
    }
}

// ── Status-effect ambient emitters ───────────────────────────────────────────

fn emit_status_ambient(particles: &mut Vec<Particle>, cx: f32, cy: f32, frame: u64,
    effect_flags: u32)  // bitmask: 1=burn 2=freeze 4=poison 8=bleed 16=stun 32=regen
{
    if particles.len() > 1900 { return; }
    // Only emit on certain frames to cap particle rate
    if frame % 4 != 0 { return; }

    let jitter = (frame ^ (cx as u64 * 31)) % 3;

    if effect_flags & 1 != 0 {  // Burn: orange sparks float up
        let col = (255u8, 110u8, 20u8);
        particles.push(Particle::burst(
            cx + jitter as f32 - 1.0, cy + 1.0,
            (jitter as f32 - 1.0) * 0.04, -0.12, "·", col, 18));
        if frame % 8 == 0 {
            particles.push(Particle::spark(cx + jitter as f32 - 1.0, cy,
                (jitter as f32 - 1.0) * 0.06, -0.15, "▪", (255, 180, 40)));
        }
    }
    if effect_flags & 2 != 0 {  // Freeze: blue flakes drift down
        let col = (80u8, 160u8, 255u8);
        particles.push(Particle {
            x: cx + jitter as f32 * 2.0 - 2.0, y: cy - 2.0,
            vx: (jitter as f32 - 1.0) * 0.02, vy: 0.07,
            friction: 0.97, gravity: 0.001,
            text: if frame % 16 < 8 { "❄".to_string() } else { "·".to_string() },
            col, age: 0, lifetime: 20,
        });
    }
    if effect_flags & 4 != 0 {  // Poison: green bubbles float up
        let col = (40u8, 210u8, 70u8);
        particles.push(Particle::burst(
            cx + jitter as f32 - 1.0, cy + 2.0,
            (jitter as f32 - 1.0) * 0.03, -0.09, "o", col, 24));
    }
    if effect_flags & 8 != 0 {  // Bleed: red drips fall
        let col = (200u8, 20u8, 20u8);
        particles.push(Particle {
            x: cx + jitter as f32 * 2.0 - 2.0, y: cy,
            vx: 0.0, vy: 0.08, friction: 0.99, gravity: 0.004,
            text: "▪".to_string(), col, age: 0, lifetime: 16,
        });
    }
    if effect_flags & 32 != 0 {  // Regen: small green upward cross
        let col = (50u8, 240u8, 100u8);
        particles.push(Particle::burst(
            cx + jitter as f32 - 1.0, cy,
            0.0, -0.10, "+", col, 20));
    }
}

fn emit_stun_orbit(particles: &mut Vec<Particle>, cx: f32, cy: f32, frame: u64) {
    if particles.len() > 1900 { return; }
    if frame % 3 != 0 { return; }
    use std::f32::consts::TAU;
    let col = (255u8, 215u8, 0u8);
    let angle = (frame as f32 * 0.18) % TAU;
    let r = 2.5f32;
    particles.push(Particle {
        x: cx + angle.cos() * r,
        y: cy + angle.sin() * r * 0.5,
        vx: 0.0, vy: 0.0, friction: 1.0, gravity: 0.0,
        text: "★".to_string(), col, age: 0, lifetime: 6,
    });
    // second star offset by pi
    let a2 = angle + std::f32::consts::PI;
    particles.push(Particle {
        x: cx + a2.cos() * r, y: cy + a2.sin() * r * 0.5,
        vx: 0.0, vy: 0.0, friction: 1.0, gravity: 0.0,
        text: "✦".to_string(), col, age: 0, lifetime: 6,
    });
}

fn emit_room_ambient(particles: &mut Vec<Particle>, frame: u64, room_seed: u64,
    room_type_id: u8) // 1=combat 2=treasure 3=shrine 4=chaos_rift 5=boss
{
    if particles.len() > 1800 { return; }
    if frame % 8 != 0 { return; }
    let xs = (frame.wrapping_add(room_seed)) % 140 + 10;
    let x = xs as f32;
    match room_type_id {
        1 => {  // Combat: faint red haze rising
            let col = (80u8, 10u8, 10u8);
            particles.push(Particle::burst(x, 72.0, 0.0, -0.06, "·", col, 40));
        }
        2 => {  // Treasure: gold sparkles from centre
            if frame % 16 == 0 {
                let col = (255u8, 200u8, 30u8);
                use std::f32::consts::TAU;
                let angle = (frame as f32 * 0.3) % TAU;
                particles.push(Particle::spark(80.0, 35.0,
                    angle.cos() * 0.15, angle.sin() * 0.1, "✦", col));
            }
        }
        3 => {  // Shrine: blue upward particles
            let col = (60u8, 100u8, 255u8);
            let sx = 60.0 + (frame % 60) as f32;
            particles.push(Particle::burst(sx, 60.0, 0.0, -0.10, "·", col, 35));
            if frame % 24 == 0 {
                particles.push(Particle::burst(80.0, 50.0, 0.0, -0.12, "✦", (100, 150, 255), 30));
            }
        }
        4 => {  // Chaos Rift: glitching character explosions
            let col = (
                ((frame * 73) % 200 + 55) as u8,
                ((frame * 37) % 200 + 55) as u8,
                ((frame * 53) % 200 + 55) as u8,
            );
            let glitch = ["?","!","∞","∑","λ","░","▒","#","@","*"];
            let ch = glitch[frame as usize % glitch.len()];
            use std::f32::consts::TAU;
            let angle = (frame as f32 * 0.4) % TAU;
            particles.push(Particle::burst(80.0, 35.0,
                angle.cos() * 0.2, angle.sin() * 0.15, ch, col, 25));
        }
        5 => {  // Boss room: pulsing purple/red particles
            let pulse = (frame / 8) % 2 == 0;
            let col = if pulse { (200u8, 20u8, 20u8) } else { (140u8, 20u8, 180u8) };
            let bx = (frame % 140 + 10) as f32;
            particles.push(Particle::burst(bx, 70.0, 0.0, -0.08, "▪", col, 45));
        }
        _ => {}
    }
}

fn emit_boss_entrance_burst(particles: &mut Vec<Particle>, boss_id: u8, frame: u64) {
    if particles.len() > 1700 { return; }
    use std::f32::consts::TAU;
    let cx = 80.0f32; let cy = 30.0f32;
    match boss_id {
        1 => {  // Mirror: symmetric split left and right
            if frame % 3 == 0 {
                let angle = (frame as f32 * 0.2) % TAU;
                let col = (200u8, 200u8, 255u8);
                particles.push(Particle::burst(cx - 20.0, cy, -0.2, 0.0, "◈", col, 20));
                particles.push(Particle::burst(cx + 20.0, cy,  0.2, 0.0, "◈", col, 20));
            }
        }
        3 => {  // Fibonacci Hydra: golden spiral
            if frame % 2 == 0 {
                let fib_angle = frame as f32 * 2.399; // golden angle
                let r = (frame as f32 * 0.15).min(30.0);
                let col = (255u8, 200u8, 30u8);
                let ch = ["·","✦","*"][frame as usize % 3];
                particles.push(Particle::burst(
                    cx + fib_angle.cos() * r, cy + fib_angle.sin() * r * 0.5,
                    fib_angle.cos() * 0.1, fib_angle.sin() * 0.07, ch, col, 15));
            }
        }
        9 => {  // Committee: 5 clusters converging
            if frame % 4 == 0 {
                let col = (180u8, 80u8, 220u8);
                for i in 0..5usize {
                    let angle = i as f32 * TAU / 5.0;
                    let r = 30.0 - frame as f32 * 0.4;
                    if r < 2.0 { continue; }
                    particles.push(Particle::burst(
                        cx + angle.cos() * r, cy + angle.sin() * r * 0.5,
                        -angle.cos() * 0.15, -angle.sin() * 0.1, "◆", col, 12));
                }
            }
        }
        12 => {  // Algorithm Reborn: full screen ring explosion
            if frame % 2 == 0 {
                let col = (
                    ((frame * 60 + 80) % 200 + 55) as u8,
                    ((frame * 40 + 120) % 180 + 55) as u8,
                    ((frame * 80 + 60) % 200 + 55) as u8,
                );
                let angle = frame as f32 * TAU / 20.0;
                let r = frame as f32 * 0.6;
                if r < 70.0 {
                    particles.push(Particle::burst(
                        cx + angle.cos() * r, cy + angle.sin() * r * 0.5,
                        angle.cos() * 0.25, angle.sin() * 0.15,
                        ["*","#","@","!"][frame as usize % 4], col, 18));
                }
            }
        }
        _ => {  // Generic boss entrance: radial burst
            if frame % 3 == 0 {
                let col = (220u8, 40u8, 40u8);
                let angle = (frame as f32 * 0.3) % TAU;
                particles.push(Particle::burst(
                    cx + angle.cos() * 15.0, cy + angle.sin() * 10.0,
                    angle.cos() * 0.2, angle.sin() * 0.15, "☠", col, 20));
            }
        }
    }
}

fn floor_transition_flavor(floor: u32) -> &'static str {
    match floor {
        1..=5   => "The Proof begins.",
        6..=10  => "The cascade deepens.",
        11..=20 => "Mathematics grows hostile.",
        21..=30 => "Reality distorts at the edges.",
        31..=50 => "The numbers are watching.",
        51..=75 => "You are inside the recursion.",
        76..=99 => "The Proof computes faster than thought.",
        100..=149 => "Beyond mortal comprehension.",
        150..=199 => "The engines are alive.",
        _       => "There is no bottom to this proof.",
    }
}

// ─── STATE ────────────────────────────────────────────────────────────────────

struct State {
    screen: AppScreen,
    player: Option<Character>,
    floor: Option<Floor>,
    enemy: Option<Enemy>,
    combat_state: Option<CombatState>,
    last_roll: Option<ChaosRollResult>,
    combat_log: Vec<String>,
    seed: u64,
    floor_seed: u64,
    frame: u64,
    // char creation
    selected_menu: usize,
    cc_class: usize,
    cc_bg: usize,
    cc_diff: usize,
    cc_name: String,
    cc_name_active: bool,
    // mode select
    mode_cursor: usize,
    game_mode: GameMode,
    // boon select
    boon_options: [Boon; 3],
    boon_cursor: usize,
    // floor state
    floor_num: u32,
    max_floor: u32,
    is_cursed_floor: bool,
    // nemesis
    nemesis_record: Option<NemesisRecord>,
    nemesis_spawned: bool,
    // combat extras
    is_boss_fight: bool,
    gauntlet_stage: u8,     // 0=off, 1/2/3=fight #
    gauntlet_enemies: Vec<Enemy>,
    loot_pending: Option<Item>,
    current_mana: i64,
    // room event
    room_event: RoomEvent,
    // shop state
    shop_items: Vec<(Item, i64)>,
    shop_heal_cost: i64,
    shop_cursor: usize,
    // crafting state
    craft_phase: CraftPhase,
    craft_item_cursor: usize,
    craft_op_cursor: usize,
    craft_message: String,
    // audio
    audio: Option<AudioSystem>,
    music_vibe: MusicVibe,
    // visual theme
    theme_idx: usize,
    // auto-play mode
    auto_mode: bool,
    auto_last_action: u64, // frame when last auto-action fired
    // ── Visual effects ──
    particles: Vec<Particle>,
    player_flash: u32,          // frames left — red border flash on player panel
    enemy_flash: u32,           // frames left — colored border flash on enemy panel
    enemy_flash_col: (u8,u8,u8),
    hit_shake: u32,             // frames of outer-border shake flash on big crits
    spell_beam: u32,            // frames of beam animation
    spell_beam_col: (u8,u8,u8),
    // ── Chaos field ──
    chaos_field: ChaosField,
    // ── HP ghost bars ──
    ghost_player_hp: f32,       // previous HP fraction (0.0-1.0) for ghost bar
    ghost_player_timer: u32,    // frames the ghost bar lingers
    ghost_enemy_hp: f32,
    ghost_enemy_timer: u32,
    // ── Kill linger ──
    kill_linger: u32,                       // frames to stay on combat after kill
    post_combat_screen: Option<AppScreen>,  // screen to go to after linger
    // ── Passive tree browser ──
    passive_scroll: usize,
    // ── Tutorial ──
    tutorial_slide: usize,
    save_exists: bool,
    // ── Achievements ──
    achievements: AchievementStore,
    achievement_banner: Option<String>,     // currently displayed banner text
    achievement_banner_frames: u32,         // frames remaining for banner
    // ── Run history ──
    run_history: RunHistory,
    history_scroll: usize,
    // ── Bestiary / Codex ──
    bestiary_scroll: usize,
    codex_scroll: usize,
    // ── Achievements ──
    achievement_scroll: usize,
    achievement_filter: u8, // 0=All, 1=Unlocked, 2=Locked
    // ── Death recap clipboard ──
    last_recap_text: String,                // shareable text of last run
    // ── Chaos engine viz overlay ──
    chaos_viz_open: bool,
    // ── Item filter (crafting) ──
    item_filter: String,
    item_filter_active: bool,
    // ── Mod config ──
    config: ChaosConfig,
    // ── Daily leaderboard ──
    daily_store: LocalDailyStore,
    daily_rows: Vec<LeaderboardRow>,
    daily_status: String,           // "Fetching…" / "Rank #N" / error message
    daily_submitted: bool,          // already submitted this session
    // ── Unique boss state ──
    boss_id: Option<u8>,   // active unique boss (None = regular fight)
    boss_turn: u32,        // turn counter within boss fight
    boss_extra: i64,       // multi-purpose state (meaning varies per boss)
    boss_extra2: i64,      // second multi-purpose state
    // ── Smooth HP/MP display (lerped 0.0-1.0 fractions) ──
    display_player_hp: f32,
    display_enemy_hp: f32,
    display_mp: f32,
    // ── Floor transition overlay ──
    floor_transition_timer: u32,
    floor_transition_floor: u32,
    // ── Boss entrance animation ──
    boss_entrance_timer: u32,
    boss_entrance_name: String,
    // ── Craft operation animation ──
    craft_anim_timer: u32,
    craft_anim_type: u8,   // 0=none 1=reforge 2=corrupt 3=shatter 4=imbue
    // ── Title logo particle assembly ──
    title_logo_timer: u32, // counts down from 90 on first load; 0 = done
    // ── Character sheet tabs ──
    char_tab: u8,  // 0=Stats 1=Inventory 2=Effects 3=Lore 4=Log
    // ── Combat log collapse ──
    combat_log_collapsed: bool,
    // ── Bestiary / Codex selected entry ──
    bestiary_selected: usize,
    codex_selected: usize,
    // ── Visual push systems ──
    color_grade: ColorGrade,
    tile_effects: TileEffects,
    weather: Weather,
    death_seq: DeathSeq,
    // Track if death cinematic has played this death
    death_cinematic_done: bool,
    // ── Extended animation systems ──
    combat_anim: CombatAnim,
    nemesis_reveal: NemesisReveal,
    rich_banner: AchievementBanner,
    anim_config: AnimConfig,
    // Tracks the most recent CombatAction for animation selection
    last_action_type: u8,  // 0=none 1=attack 2=heavy 3=spell 4=defend 5=flee 6=item
    last_spell_name: String,
    // Room entry animation state
    room_entry_timer: u32,
    room_entry_type: u8,  // 0=none 1=combat 2=shop 3=shrine 4=rift 5=boss
}

impl State {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        let cfg = ChaosConfig::load();
        let music_vibe = MusicVibe::from_str(&cfg.audio.music_vibe);
        let audio = AudioSystem::try_new();
        if let Some(ref a) = audio { a.set_vibe(music_vibe); }
        State {
            screen: AppScreen::Title,
            player: None, floor: None, enemy: None,
            combat_state: None, last_roll: None,
            combat_log: Vec::new(),
            seed, floor_seed: seed, frame: 0,
            selected_menu: 0, cc_class: 0, cc_bg: 0, cc_diff: 1,
            cc_name: String::new(), cc_name_active: false,
            mode_cursor: 0, game_mode: GameMode::Infinite,
            boon_options: Boon::random_three(seed), boon_cursor: 0,
            floor_num: 1, max_floor: u32::MAX, is_cursed_floor: false,
            nemesis_record: None, nemesis_spawned: false,
            is_boss_fight: false,
            gauntlet_stage: 0, gauntlet_enemies: Vec::new(),
            loot_pending: None, current_mana: 0,
            room_event: RoomEvent::empty(),
            shop_items: Vec::new(), shop_heal_cost: 20, shop_cursor: 0,
            craft_phase: CraftPhase::SelectItem,
            craft_item_cursor: 0, craft_op_cursor: 0,
            craft_message: String::new(),
            audio,
            music_vibe,
            theme_idx: 0,
            auto_mode: false,
            auto_last_action: 0,
            particles: Vec::new(),
            player_flash: 0,
            enemy_flash: 0,
            enemy_flash_col: (80, 220, 80),
            hit_shake: 0,
            spell_beam: 0,
            spell_beam_col: (80, 120, 255),
            chaos_field: ChaosField::new(),
            ghost_player_hp: 1.0,
            ghost_player_timer: 0,
            ghost_enemy_hp: 1.0,
            ghost_enemy_timer: 0,
            kill_linger: 0,
            post_combat_screen: None,
            passive_scroll: 0,
            tutorial_slide: 0,
            save_exists: save_path().exists(),
            achievements: AchievementStore::load(),
            achievement_banner: None,
            achievement_banner_frames: 0,
            run_history: RunHistory::load(),
            history_scroll: 0,
            bestiary_scroll: 0,
            codex_scroll: 0,
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
            boss_id: None, boss_turn: 0, boss_extra: 0, boss_extra2: 0,
            display_player_hp: 1.0, display_enemy_hp: 1.0, display_mp: 1.0,
            floor_transition_timer: 0, floor_transition_floor: 1,
            boss_entrance_timer: 0, boss_entrance_name: String::new(),
            craft_anim_timer: 0, craft_anim_type: 0,
            title_logo_timer: 90,
            char_tab: 0,
            combat_log_collapsed: false,
            bestiary_selected: 0,
            codex_selected: 0,
            color_grade: ColorGrade::default(),
            tile_effects: TileEffects::new(),
            weather: Weather::new(),
            death_seq: DeathSeq::new(),
            death_cinematic_done: false,
            combat_anim: CombatAnim::new(),
            nemesis_reveal: NemesisReveal::new(),
            rich_banner: AchievementBanner::new(),
            anim_config: AnimConfig::load(),
            last_action_type: 0,
            last_spell_name: String::new(),
            room_entry_timer: 0,
            room_entry_type: 0,
        }
    }

    fn theme(&self) -> &theme::Theme {
        &theme::THEMES[self.theme_idx]
    }

    /// Returns a color-graded clone of the current theme.
    /// All draw functions should use this instead of self.theme_graded().
    fn theme_graded(&self) -> theme::Theme {
        let mut t = self.theme_graded();
        self.color_grade.apply_to_theme(&mut t);
        // Breathing borders via tile_effects
        let bb = self.tile_effects.border_brightness();
        t.border = (
            (t.border.0 as f32 * bb).clamp(0.0, 255.0) as u8,
            (t.border.1 as f32 * bb).clamp(0.0, 255.0) as u8,
            (t.border.2 as f32 * bb).clamp(0.0, 255.0) as u8,
        );
        t
    }

    fn cycle_theme(&mut self) {
        self.theme_idx = (self.theme_idx + 1) % theme::THEMES.len();
    }

    /// Clear the screen to theme bg and render the chaos field background.
    /// Call at the start of every draw function instead of `ctx.cls_bg(bg)`.
    fn chaos_bg(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();

        // Boss 11 (The Paradox): slightly lighter bg as inversion cue
        let is_paradox = self.boss_id == Some(11) && self.config.visuals.invert_screen_for_paradox;
        let bg_tuple = if is_paradox {
            (t.bg.0.saturating_add(18), t.bg.1.saturating_add(18), t.bg.2.saturating_add(18))
        } else {
            t.bg
        };

        let bg = RGB::from_u8(bg_tuple.0, bg_tuple.1, bg_tuple.2);
        ctx.cls_bg(bg);

        // Boss 6 (The Null): suppress chaos field — the proof stops computing
        let is_null_fight = self.boss_id == Some(6);

        if self.config.visuals.enable_chaos_field
            && !self.config.visuals.reduce_motion
            && !is_null_fight
        {
            let corruption = self.player.as_ref().map(|p| p.corruption).unwrap_or(0);
            self.chaos_field.update(self.frame, self.floor_num, corruption);
            self.chaos_field.draw(ctx, bg_tuple, t.muted, t.accent,
                                  self.floor_num, corruption, self.frame);
        }
    }

    /// Drive ColorGrade from current game state. Called once per frame from tick().
    fn update_color_grade(&mut self) {
        // Null fight: full desaturation
        if self.boss_id == Some(6) && self.screen == AppScreen::Combat {
            self.color_grade.set_null_fight();
            return;
        }
        // Paradox fight: full hue inversion
        if self.boss_id == Some(11) && self.screen == AppScreen::Combat
            && self.config.visuals.invert_screen_for_paradox {
            self.color_grade.set_paradox();
            return;
        }
        // Game over / death
        if self.screen == AppScreen::GameOver {
            self.color_grade.set_death();
            return;
        }
        // Victory
        if self.screen == AppScreen::Victory {
            self.color_grade.set_victory();
            return;
        }
        // Low HP (< 25%)
        if let Some(ref p) = self.player {
            let php = p.current_hp as f32 / p.max_hp.max(1) as f32;
            if php < 0.25 && self.screen == AppScreen::Combat {
                self.color_grade.set_low_hp();
                return;
            }
            // High corruption
            if p.corruption >= 300 {
                self.color_grade.set_high_corruption();
                return;
            }
        }
        // Boss phase 2/3
        if self.is_boss_fight && self.screen == AppScreen::Combat {
            if self.boss_turn >= 10 {
                self.color_grade.set_boss_phase3();
            } else if self.boss_turn >= 5 {
                self.color_grade.set_boss_phase2();
            } else {
                self.color_grade.set_normal();
            }
            return;
        }
        // Deep floors
        if self.floor_num >= 76 {
            self.color_grade.set_deep_floor();
            return;
        }
        self.color_grade.set_normal();
    }

    /// Trigger a pulse ring centered at (cx, cy) — called on impactful events.
    pub fn trigger_pulse_ring(&mut self, cx: f32, cy: f32, col: (f32,f32,f32), intensity: f32) {
        self.tile_effects.emit_pulse_ring(cx, cy, col, intensity, 22.0);
    }

    /// Trigger an impact ripple — called on hit/crit.
    pub fn trigger_ripple(&mut self, cx: f32, cy: f32, col: (f32,f32,f32)) {
        self.tile_effects.emit_impact_ripple(cx, cy, col);
    }

    /// Trigger screen earthquake — called on boss death, misery milestone, etc.
    pub fn trigger_earthquake(&mut self, intensity: f32, frames: u32) {
        self.tile_effects.emit_earthquake(intensity, frames);
    }

    /// Draw room entry color flash overlay — fades in from room-type color.
    fn draw_room_entry_flash(&self, ctx: &mut BTerm) {
        if self.room_entry_timer == 0 { return; }
        let t = self.room_entry_timer;
        let max_t: u32 = match self.room_entry_type {
            5 => 30, 1 => 30, 3 => 25, 4 => 30, _ => 20,
        };
        let alpha = t as f32 / max_t as f32;
        // Color by room type: 1=combat(red) 2=shop(gold) 3=shrine(blue) 4=rift(purple) 5=boss(deep red)
        let (r, g, b) = match self.room_entry_type {
            1 => (180u8, 30u8,  10u8),
            2 => (180u8, 140u8, 10u8),
            3 => (30u8,  80u8,  200u8),
            4 => (100u8, 10u8,  180u8),
            5 => (220u8, 10u8,  10u8),
            _ => return,
        };
        let v = (alpha * 40.0) as u8;
        if v == 0 { return; }
        let flash_col = RGB::from_u8(
            r.saturating_mul(v / 40 + 1).min(v),
            g.saturating_mul(v / 40 + 1).min(v / 2),
            b.saturating_mul(v / 40 + 1).min(v),
        );
        let bg_rgb = RGB::from_u8(0, 0, 0);
        // Flash the screen border (top/bottom rows)
        for x in 0..160i32 {
            ctx.print_color(x, 0, flash_col, bg_rgb, "█");
            ctx.print_color(x, 79, flash_col, bg_rgb, "█");
        }
        for y in 1..79i32 {
            ctx.print_color(0, y, flash_col, bg_rgb, "█");
            ctx.print_color(159, y, flash_col, bg_rgb, "█");
        }
    }

    /// Lerp display HP/MP fractions toward actual values (call once per frame from tick).
    fn update_display_fractions(&mut self) {
        const SPEED: f32 = 0.08;
        const SPEED_FAST: f32 = 0.14;
        if let Some(ref p) = self.player {
            let target_php = p.current_hp as f32 / p.max_hp.max(1) as f32;
            let target_mp  = self.current_mana as f32 / self.max_mana() as f32;
            let php_diff = target_php - self.display_player_hp;
            let mp_diff  = target_mp  - self.display_mp;
            // Heal faster than damage (damage shows ghost bar; heal shows smooth fill)
            let php_speed = if php_diff > 0.0 { SPEED_FAST } else { SPEED };
            let mp_speed  = if mp_diff  > 0.0 { SPEED_FAST } else { SPEED };
            self.display_player_hp = (self.display_player_hp + php_diff * php_speed).clamp(0.0, 1.0);
            self.display_mp        = (self.display_mp        + mp_diff  * mp_speed ).clamp(0.0, 1.0);
        }
        if let Some(ref e) = self.enemy {
            let target = e.hp as f32 / e.max_hp.max(1) as f32;
            let diff = target - self.display_enemy_hp;
            let speed = if diff > 0.0 { SPEED_FAST } else { SPEED };
            self.display_enemy_hp = (self.display_enemy_hp + diff * speed).clamp(0.0, 1.0);
        }
    }

    fn max_mana(&self) -> i64 {
        self.player.as_ref().map(|p| (p.stats.mana + 50).max(50)).unwrap_or(50)
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.combat_log.push(msg.into());
        if self.combat_log.len() > 300 { self.combat_log.remove(0); }
    }

    fn emit_audio(&self, event: AudioEvent) {
        if let Some(ref audio) = self.audio { audio.emit(event); }
    }

    fn apply_stat_modifier(&mut self, stat: &str, val: i64) {
        if let Some(ref mut p) = self.player {
            match stat {
                "vitality"  => { p.stats.vitality  += val; p.max_hp = (50 + p.stats.vitality*3 + p.stats.force).max(1); }
                "force"     => p.stats.force     += val,
                "mana"      => p.stats.mana      += val,
                "cunning"   => p.stats.cunning   += val,
                "precision" => p.stats.precision += val,
                "entropy"   => p.stats.entropy   += val,
                "luck"      => p.stats.luck      += val,
                _ => {}
            }
        }
    }

    fn daily_seed() -> u64 {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        let day = secs / 86400;
        day.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
    }

    fn start_new_game(&mut self) {
        let class = CLASSES[self.cc_class].1.clone();
        let bg    = BACKGROUNDS[self.cc_bg].1.clone();
        let diff  = DIFFICULTIES[self.cc_diff].1.clone();
        let seed  = match self.game_mode {
            GameMode::Daily => Self::daily_seed(),
            GameMode::Infinite if self.config.gameplay.infinite_seed_override != 0
                => self.config.gameplay.infinite_seed_override,
            _ => self.seed,
        };
        self.seed = seed;
        self.floor_seed = seed;
        let player_name = if self.cc_name.trim().is_empty() {
            "Hero".to_string()
        } else {
            self.cc_name.trim().to_string()
        };
        let mut player = Character::roll_new(player_name, class, bg, seed, diff);
        player.apply_boon(self.boon_options[self.boon_cursor]);
        // Apply config bonuses
        if self.config.gameplay.starting_gold_bonus > 0 {
            player.gold += self.config.gameplay.starting_gold_bonus;
        }
        if self.config.loaded_from_file {
            self.achievements.check_event("config_loaded", 1);
            if self.config.gameplay.starting_gold_bonus > 0 {
                self.achievements.check_event("config_gold_bonus", 1);
            }
            if self.config.gameplay.difficulty_modifier >= 2.0 {
                self.achievements.check_event("config_hard_mode", 1);
            }
            self.achievements.save();
        }
        self.player = Some(player);
        self.floor_num = 1;
        self.max_floor = if self.game_mode == GameMode::Story { 10 } else { u32::MAX };
        self.nemesis_record = load_nemesis();
        self.nemesis_spawned = false;
        self.current_mana = self.max_mana();
        self.screen = AppScreen::FloorNav;
        self.generate_floor_for_current();
        self.emit_audio(AudioEvent::FloorEntered { floor: self.floor_num, seed: self.floor_seed });
        if self.game_mode == GameMode::Daily {
            self.emit_audio(AudioEvent::DailyStart);
        }
    }

    fn generate_floor_for_current(&mut self) {
        self.floor_seed = self.floor_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.floor_num as u64 * 31337);

        // Item volatility: every 20 floors, re-roll a random item
        if self.floor_num > 1 && self.floor_num % 20 == 0 {
            if let Some(ref mut p) = self.player {
                if !p.inventory.is_empty() {
                    let vol_idx = (self.floor_seed % p.inventory.len() as u64) as usize;
                    let old = p.inventory[vol_idx].name.clone();
                    p.inventory[vol_idx] = Item::generate(self.floor_seed.wrapping_add(0x766F6C));
                    let new = p.inventory[vol_idx].name.clone();
                    self.push_log(format!("⚡ ITEM VOLATILITY: {} → {}", old, new));
                    self.emit_audio(AudioEvent::ItemVolatilityReroll);
                }
            }
        }

        self.is_cursed_floor = self.floor_num > 0 && self.floor_num % 25 == 0;
        if self.is_cursed_floor {
            self.push_log("☠ CURSED FLOOR! All engine outputs INVERTED this floor.".to_string());
            self.emit_audio(AudioEvent::CursedFloorActivated);
        }

        let fl = generate_floor(self.floor_num, self.floor_seed);
        self.floor = Some(fl);

        if let Some(ref mut p) = self.player {
            p.floor = self.floor_num;
        }
    }

    // ── SAVE / LOAD ───────────────────────────────────────────────────────────

    fn do_save(&mut self) {
        let Some(ref player) = self.player else { return; };
        let mode_str = match self.game_mode {
            GameMode::Story    => "Story",
            GameMode::Infinite => "Infinite",
            GameMode::Daily    => "Daily",
        };
        let ss = SaveState {
            player: player.clone(),
            floor: self.floor.clone(),
            floor_num: self.floor_num,
            floor_seed: self.floor_seed,
            seed: self.seed,
            current_mana: self.current_mana,
            is_boss_fight: self.is_boss_fight,
            game_mode: mode_str.to_string(),
            nemesis_spawned: self.nemesis_spawned,
            combat_log: self.combat_log.clone(),
        };
        write_save(&ss);
        self.save_exists = true;
        self.push_log("Game saved. [F5] to save · [L] on title to continue".to_string());
    }

    fn do_load(&mut self) {
        let Some(ss) = read_save() else { return; };
        self.player          = Some(ss.player);
        self.floor           = ss.floor;
        self.floor_num       = ss.floor_num;
        self.floor_seed      = ss.floor_seed;
        self.seed            = ss.seed;
        self.current_mana    = ss.current_mana;
        self.is_boss_fight   = ss.is_boss_fight;
        self.game_mode       = match ss.game_mode.as_str() {
            "Story"  => GameMode::Story,
            "Daily"  => GameMode::Daily,
            _        => GameMode::Infinite,
        };
        self.nemesis_spawned = ss.nemesis_spawned;
        self.combat_log      = ss.combat_log;
        // Always restore to floor nav — mid-combat state is not saved
        self.enemy           = None;
        self.combat_state    = None;
        self.screen          = AppScreen::FloorNav;
        self.push_log("Save loaded — welcome back.".to_string());
    }

    fn advance_floor_room(&mut self) {
        let at_end = self.floor.as_ref()
            .map(|f| f.current_room + 1 >= f.rooms.len())
            .unwrap_or(true);
        if at_end {
            // Check victory condition
            if self.floor_num >= self.max_floor {
                self.emit_audio(AudioEvent::Victory);
                self.screen = AppScreen::Victory;
                self.save_score_now();
                return;
            }
            self.floor_num += 1;
            // Trigger floor transition overlay (2.5s: 0.5s fade-in, 1.5s hold, 0.5s fade-out)
            self.floor_transition_floor = self.floor_num;
            self.floor_transition_timer = 150;
            self.generate_floor_for_current();
        } else {
            self.floor.as_mut().map(|f| f.advance());
        }
        if let Some(ref mut p) = self.player { p.rooms_cleared += 1; }
        // The Hunger (floor 50+)
        let hunger_trigger = self.player.as_ref()
            .map(|p| p.floor >= 50 && p.rooms_without_kill >= 5 && self.screen != AppScreen::Combat)
            .unwrap_or(false);
        if hunger_trigger {
            self.emit_audio(AudioEvent::HungerTriggered);
            let loss = self.player.as_ref().map(|p| (p.max_hp / 20).max(1)).unwrap_or(1);
            if let Some(ref mut p) = self.player {
                p.max_hp = (p.max_hp - loss).max(1);
                if p.current_hp > p.max_hp { p.current_hp = p.max_hp; }
                p.rooms_without_kill = 0;
            }
            self.push_log(format!("THE HUNGER: -{} max HP permanently!", loss));
            if self.player.as_ref().map(|p| !p.is_alive()).unwrap_or(false) {
                self.screen = AppScreen::GameOver;
                self.save_score_now();
                return;
            }
        }
    }

    fn enter_current_room(&mut self) {
        let floor_num = self.floor_num;
        let room_seed = self.floor_seed
            .wrapping_add(self.floor.as_ref().map(|f| f.current_room as u64 * 9973).unwrap_or(0));

        // BloodPact boon: take 2 HP each room
        if matches!(self.boon_options[self.boon_cursor], Boon::BloodPact) {
            if let Some(ref mut p) = self.player { p.take_damage(2); }
            self.push_log("Blood Pact: -2 HP".to_string());
            if self.player.as_ref().map(|p| !p.is_alive()).unwrap_or(false) {
                self.screen = AppScreen::GameOver;
                self.save_score_now();
                return;
            }
        }

        let room_type = self.floor.as_ref()
            .map(|f| f.current().room_type.clone())
            .unwrap_or(RoomType::Empty);
        let room_desc = self.floor.as_ref()
            .map(|f| f.current().description.clone())
            .unwrap_or_default();

        match room_type {
            RoomType::Combat | RoomType::Boss => {
                let is_boss = room_type == RoomType::Boss;

                // Nemesis spawn check
                if !self.nemesis_spawned {
                    if let Some(ref nemesis) = self.nemesis_record.clone() {
                        let spawn_roll = room_seed.wrapping_mul(0x6E656D6573697300) % 100;
                        let spawn_chance = if floor_num >= nemesis.floor_killed_at { 40 } else { 20 };
                        if floor_num >= 3 && spawn_roll < spawn_chance {
                            self.nemesis_spawned = true;
                            let base_floor = nemesis.floor_killed_at;
                            let mut nem_enemy = generate_enemy(base_floor.max(1), room_seed);
                            nem_enemy.name = format!("★ {}", nemesis.enemy_name);
                            nem_enemy.hp = (nem_enemy.hp * (100 + nemesis.hp_bonus_pct as i64) / 100).max(1);
                            nem_enemy.max_hp = nem_enemy.hp;
                            nem_enemy.base_damage = (nem_enemy.base_damage * (100 + nemesis.damage_bonus_pct as i64) / 100).max(1);
                            nem_enemy.xp_reward *= 5;
                            nem_enemy.gold_reward *= 3;
                            let nem_display_name = nem_enemy.name.clone();
                            self.push_log(format!("☠ NEMESIS RETURNS: {}!", nem_enemy.name));
                            self.push_log(format!("HP +{}%  DMG +{}%", nemesis.hp_bonus_pct, nemesis.damage_bonus_pct));
                            self.enemy = Some(nem_enemy);
                            self.is_boss_fight = true;
                            self.gauntlet_stage = 0;
                            self.combat_state = Some(CombatState::new(room_seed));
                            if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                            self.emit_audio(AudioEvent::NemesisSpawned);
                            self.emit_audio(AudioEvent::BossEncounterStart { boss_tier: 2 });
                            // Trigger nemesis reveal cinematic if not skipped
                            if !self.anim_config.skip_nemesis_reveal {
                                let pname = self.player.as_ref().map(|p| p.name.clone()).unwrap_or_default();
                                let ability = nemesis.enemy_name.clone();
                                self.nemesis_reveal.start(
                                    &nem_display_name, nemesis.floor_killed_at,
                                    &ability, nemesis.hp_bonus_pct as u32, nemesis.damage_bonus_pct as u32,
                                    &pname,
                                );
                                // Stay on floor nav while reveal plays; combat starts after Enter
                                return;
                            }
                            self.screen = AppScreen::Combat;
                            return;
                        }
                    }
                }

                // Boss gauntlet: every 10 floors boss room = 3-fight gauntlet
                if is_boss && floor_num % 10 == 0 {
                    let mut enemies = Vec::new();
                    let mut e1 = generate_enemy(floor_num, room_seed.wrapping_add(1));
                    e1.hp = (e1.hp as f64 * 2.0) as i64; e1.max_hp = e1.hp;
                    let mut e2 = generate_enemy(floor_num, room_seed.wrapping_add(2));
                    e2.hp = (e2.hp as f64 * 3.0) as i64; e2.max_hp = e2.hp;
                    e2.base_damage = (e2.base_damage as f64 * 1.5) as i64;
                    let dr = destiny_roll(0.5, room_seed.wrapping_add(31337));
                    let pm = (dr.final_value + 1.5).max(0.5);
                    let mut e3 = generate_enemy(floor_num, room_seed.wrapping_add(3));
                    e3.hp = ((e3.hp as f64 * 4.0 * pm) as i64).max(1); e3.max_hp = e3.hp;
                    e3.base_damage = ((e3.base_damage as f64 * 2.0 * pm) as i64).max(1);
                    e3.xp_reward *= 5; e3.gold_reward *= 5;
                    enemies.push(e1); enemies.push(e2); enemies.push(e3);
                    self.gauntlet_enemies = enemies;
                    self.gauntlet_stage = 1;
                    let first = self.gauntlet_enemies.remove(0);
                    self.enemy = Some(first);
                    self.is_boss_fight = false;
                    self.push_log("★ BOSS GAUNTLET! 3 fights. No healing.".to_string());
                    self.push_log("Fight 1/3".to_string());
                    self.combat_state = Some(CombatState::new(room_seed));
                    if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                    self.emit_audio(AudioEvent::GauntletStart);
                    self.screen = AppScreen::Combat;
                    return;
                }

                // Unique boss spawn (floor 5+: boss rooms every 5 floors; floor 50+: 20% random; floor 100+: every 3rd room)
                let unique_roll = room_seed.wrapping_mul(0x756E697175650000) % 100;
                let spawn_unique = (floor_num >= 100 && self.floor.as_ref().map(|f| f.current_room).unwrap_or(0) % 3 == 0)
                    || (floor_num >= 50 && !is_boss && unique_roll < 20)
                    || (is_boss && floor_num % 5 == 0);
                if spawn_unique {
                    if let Some(boss_id) = random_unique_boss(floor_num, room_seed) {
                        self.start_unique_boss(boss_id, floor_num, room_seed);
                        return;
                    }
                }

                // Normal enemy
                let room = self.floor.as_ref().map(|f| f.current().clone()).unwrap();
                let mut enemy = room_enemy(&room);
                // StatMirror
                if enemy.floor_ability == FloorAbility::StatMirror {
                    let (sname, sval) = self.player.as_ref().map(|p| p.highest_stat()).unwrap_or(("force", 10));
                    enemy.hp = sval.max(1); enemy.max_hp = enemy.hp;
                    self.push_log(format!("⚠ STAT MIRROR: enemy HP = your {} ({})", sname, sval));
                }
                if enemy.floor_ability == FloorAbility::NullifyAura {
                    self.push_log("⚠ NULLIFY AURA: first action returns 0.0!".to_string());
                }
                if enemy.floor_ability == FloorAbility::EngineTheft {
                    self.push_log("⚠ ENGINE THEFT: each hit steals 1 engine!".to_string());
                }
                if is_boss {
                    enemy.hp = (enemy.hp as f64 * 2.5) as i64; enemy.max_hp = enemy.hp;
                    enemy.base_damage = (enemy.base_damage as f64 * 1.8) as i64;
                    enemy.xp_reward *= 3; enemy.gold_reward *= 3;
                    self.push_log("★ BOSS BATTLE ★".to_string());
                    self.emit_audio(AudioEvent::BossEncounterStart { boss_tier: 1 });
                } else {
                    self.emit_audio(AudioEvent::RoomEntered { room_index: self.floor.as_ref().map(|f| f.current_room).unwrap_or(0) });
                }
                self.enemy = Some(enemy);
                self.is_boss_fight = is_boss;
                self.gauntlet_stage = 0;
                self.combat_state = Some(CombatState::new(room_seed));
                if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                self.room_entry_type = if is_boss { 5 } else { 1 };
                self.room_entry_timer = 30;
                self.screen = AppScreen::Combat;
            }

            RoomType::Treasure => {
                let item = Item::generate(room_seed);
                let gold_bonus = ((room_seed % 30 + 10) as i64) * floor_num as i64;
                let mut ev = RoomEvent::empty();
                ev.title = "★ TREASURE ROOM ★".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    format!("You find {} gold!", gold_bonus), String::new(),
                    format!("Item: {}", item.name),
                    format!("Rarity: {}", item.rarity.name()),
                ];
                for m in &item.stat_modifiers {
                    ev.lines.push(format!("  {:+} {}", m.value, m.stat));
                }
                ev.lines.push(String::new());
                ev.lines.push("[P] Pick up   [Enter] Leave".to_string());
                ev.gold_delta = gold_bonus;
                ev.pending_item = Some(item);
                if room_seed % 4 == 0 {
                    let spell = Spell::generate(room_seed.wrapping_add(54321));
                    ev.lines.push(String::new());
                    ev.lines.push(format!("+ SPELL SCROLL: {}", spell.name));
                    ev.lines.push(format!("  {}mp  ×{:.1} scaling", spell.mana_cost, spell.scaling_factor.abs()));
                    ev.lines.push("[L] Learn spell   [Enter] Leave scroll".to_string());
                    ev.pending_spell = Some(spell);
                }
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Shop => {
                let mut npc = shop_npc(floor_num, room_seed);
                let heal_cost = 15 + floor_num as i64 * 2;
                let cunning = self.player.as_ref().map(|p| p.stats.cunning).unwrap_or(0);
                let npc_items: Vec<Item> = npc.inventory.drain(..).collect();
                let shop: Vec<(Item, i64)> = npc_items.into_iter()
                    .map(|item| {
                        let price = npc.sale_price(item.value, cunning);
                        (item, price)
                    })
                    .collect();
                self.shop_items = shop;
                self.shop_heal_cost = heal_cost;
                self.shop_cursor = 0;
                self.emit_audio(AudioEvent::ShopEntered);
                self.room_entry_type = 2;
                self.room_entry_timer = 20;
                self.screen = AppScreen::Shop;
            }

            RoomType::Shrine => {
                let entropy = self.player.as_ref().map(|p| p.stats.entropy as f64 * 0.01).unwrap_or(0.1);
                let roll = chaos_roll_verbose(entropy, room_seed);
                self.last_roll = Some(roll.clone());
                let stats: &[&'static str] = &["vitality","force","mana","cunning","precision","entropy","luck"];
                let stat_name = stats[(room_seed % stats.len() as u64) as usize];
                let buff = 3 + (roll.to_range(1, 10) as i64) + floor_num as i64 / 2;
                let hp_restore = self.player.as_ref().map(|p| p.max_hp / 5).unwrap_or(10);
                let mut ev = RoomEvent::empty();
                ev.title = "~ SHRINE ~".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    format!("Chaos value: {:.4}", roll.final_value), String::new(),
                    format!("The shrine blesses you! +{} {}", buff, stat_name),
                    format!("You feel restored. +{} HP", hp_restore),
                    String::new(), "[Enter] Accept blessing".to_string(),
                ];
                ev.stat_bonuses = vec![(stat_name, buff)];
                ev.hp_delta = hp_restore;
                self.room_event = ev;
                self.room_entry_type = 3;
                self.room_entry_timer = 25;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Trap => {
                let player_ref = self.player.as_ref().unwrap();
                let diff = match floor_num {
                    1..=3 => SkillDiff::Easy, 4..=7 => SkillDiff::Medium, _ => SkillDiff::Hard,
                };
                let check = perform_skill_check(player_ref, SkillType::Perception, diff, room_seed);
                self.last_roll = Some(check.chaos_result.clone());
                let trap_damage = if check.passed { 0 } else { 5 + floor_num as i64 * 3 + (room_seed % 10) as i64 };
                let mut ev = RoomEvent::empty();
                ev.title = "! TRAP ROOM !".to_string();
                let mut lines = vec![room_desc, String::new()];
                for line in check.display_lines() { lines.push(line); }
                lines.push(String::new());
                if check.passed {
                    lines.push("You spot and dodge the trap!".to_string());
                } else {
                    lines.push(format!("TRAP TRIGGERED! -{} HP!", trap_damage));
                }
                lines.push(String::new());
                lines.push("[Enter] Continue".to_string());
                ev.lines = lines;
                ev.damage_taken = trap_damage;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Portal => {
                let mut ev = RoomEvent::empty();
                ev.title = "^ PORTAL ^".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    "A shimmering rift to the next floor.".to_string(),
                    String::new(),
                    "[P] Step through portal   [Enter] Resist".to_string(),
                ];
                ev.portal_available = true;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::Empty => {
                let hp_gain = 5 + floor_num as i64 * 2;
                let mut ev = RoomEvent::empty();
                ev.title = "  EMPTY ROOM  ".to_string();
                ev.lines = vec![
                    room_desc, String::new(),
                    format!("The stillness restores you. +{} HP", hp_gain),
                    String::new(), "[Enter] Continue".to_string(),
                ];
                ev.hp_delta = hp_gain;
                self.room_event = ev;
                self.screen = AppScreen::RoomView;
            }

            RoomType::ChaosRift => {
                let entropy = self.player.as_ref().map(|p| p.stats.entropy as f64 * 0.015).unwrap_or(0.1);
                let roll = chaos_roll_verbose(entropy, room_seed);
                self.last_roll = Some(roll.clone());
                let outcome = room_seed.wrapping_mul(floor_num as u64 * 7 + 1) % 6;
                let mut ev = RoomEvent::empty();
                ev.title = "∞ CHAOS RIFT ∞".to_string();
                ev.lines = vec![
                    "REALITY ERROR. MATHEMATICAL EXCEPTION.".to_string(), String::new(),
                    format!("Chaos value: {:.4}", roll.final_value), String::new(),
                ];
                match outcome {
                    0 => {
                        let gold = ((room_seed % 100 + 50) as i64) * floor_num as i64;
                        ev.lines.push(format!("CHAOS BOUNTY: +{} gold!", gold));
                        ev.gold_delta = gold;
                    }
                    1 => {
                        let dmg = self.player.as_ref().map(|p| (p.max_hp / 4).max(1)).unwrap_or(10);
                        ev.lines.push(format!("CHAOS PUNISHMENT: -{} HP!", dmg));
                        ev.damage_taken = dmg;
                    }
                    2 => {
                        let bonus = 5 + floor_num as i64;
                        ev.lines.push(format!("CHAOS ASCENSION: +{} Entropy!", bonus));
                        ev.stat_bonuses = vec![("entropy", bonus)];
                    }
                    3 => {
                        let heal = self.player.as_ref().map(|p| p.max_hp / 3).unwrap_or(20);
                        ev.lines.push(format!("CHAOS BLESSING: +{} HP!", heal));
                        ev.hp_delta = heal;
                    }
                    4 => {
                        let gold_loss = self.player.as_ref().map(|p| p.gold / 4).unwrap_or(0);
                        let luck = 10 + floor_num as i64;
                        ev.lines.push(format!("CHAOS TRADE: -{} gold, +{} Luck!", gold_loss, luck));
                        ev.gold_delta = -gold_loss;
                        ev.stat_bonuses = vec![("luck", luck)];
                    }
                    _ => {
                        ev.lines.push("CHAOS HARMONY: All stats +1!".to_string());
                        ev.stat_bonuses = vec![
                            ("vitality",1),("force",1),("mana",1),("cunning",1),
                            ("precision",1),("entropy",1),("luck",1),
                        ];
                    }
                }
                ev.lines.push(String::new());
                ev.lines.push("[Enter] Accept fate".to_string());
                self.room_event = ev;
                self.room_entry_type = 4;
                self.room_entry_timer = 30;
                self.screen = AppScreen::RoomView;
            }

            RoomType::CraftingBench => {
                self.craft_phase = CraftPhase::SelectItem;
                self.craft_item_cursor = 0;
                self.craft_op_cursor = 0;
                self.craft_message = "Choose an item to craft.".to_string();
                self.screen = AppScreen::Crafting;
            }
        }
    }

    // ─── UNIQUE BOSS WIRING ──────────────────────────────────────────────────

    fn start_unique_boss(&mut self, boss_id: u8, floor_num: u32, room_seed: u64) {
        let bname = boss_name(boss_id);
        let mut enemy = generate_enemy(floor_num + 2, room_seed);
        enemy.name = bname.to_string();
        enemy.xp_reward *= 5;
        enemy.gold_reward *= 5;
        self.boss_id = Some(boss_id);
        self.boss_turn = 0;
        match boss_id {
            1 => {
                let (max_hp, force, prec) = self.player.as_ref()
                    .map(|p| (p.max_hp, p.stats.force, p.stats.precision))
                    .unwrap_or((100, 10, 10));
                enemy.hp = max_hp;
                enemy.max_hp = max_hp;
                enemy.base_damage = 5 + force / 5 + prec / 10;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.push_log("★ THE MIRROR: Your exact reflection — same HP, same force.".to_string());
                self.push_log("Your class passive still applies. Find the asymmetry.".to_string());
            }
            2 => {
                let lifetime = self.player.as_ref().map(|p| p.total_damage_dealt).unwrap_or(0);
                enemy.hp = 999_999;
                enemy.max_hp = 999_999;
                enemy.base_damage = 0;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.push_log(format!("★ THE ACCOUNTANT: Lifetime damage on record: {}.", lifetime));
                self.push_log("5 rounds, then THE BILL. [D] Defend reduces it 20%/round.".to_string());
            }
            3 => {
                let hp = 200 + floor_num as i64 * 30;
                enemy.hp = hp;
                enemy.max_hp = hp;
                enemy.base_damage = 8 + floor_num as i64 * 2;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.push_log("★ FIBONACCI HYDRA: Kill it — it splits. 10 splits = victory.".to_string());
                self.push_log("Splits: 1,1,2,3,5,8,13. Burst damage wins.".to_string());
            }
            4 => {
                let oneshot = self.player.as_ref().map(|p| p.max_hp + 1).unwrap_or(101);
                let tanky_max = 500 + floor_num as i64 * 100;
                enemy.hp = tanky_max;
                enemy.max_hp = tanky_max;
                enemy.base_damage = oneshot;
                self.boss_extra = tanky_max;
                self.boss_extra2 = 0;
                self.push_log("★ THE EIGENSTATE: Form A = huge HP no attack; Form B = 1 HP one-shot.".to_string());
                self.push_log("[T] Taunt reveals form safely. [D] Defend survives Form B.".to_string());
            }
            5 => {
                let stolen = self.player.as_ref().map(|p| p.gold).unwrap_or(0);
                if let Some(ref mut p) = self.player {
                    p.gold = 0;
                }
                let hp = stolen.max(1);
                enemy.hp = hp;
                enemy.max_hp = hp;
                enemy.base_damage = 1;
                self.boss_extra = stolen;
                self.boss_extra2 = 0;
                self.push_log(format!("★ THE TAXMAN: Your {} gold SEIZED! HP = gold owed.", stolen));
                self.push_log("Damage = gold recovered. He bills you 1% HP/round.".to_string());
            }
            6 => {
                let hp = 300 + floor_num as i64 * 80;
                enemy.hp = hp;
                enemy.max_hp = hp;
                enemy.base_damage = 20 + floor_num as i64 * 5;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.push_log("★ THE NULL: Chaos suppressed. Your damage is flat. No crits.".to_string());
                self.push_log("Enemy uses full 10-engine destiny rolls.".to_string());
            }
            7 => {
                let (total_dmg, kills) = self.player.as_ref()
                    .map(|p| (p.total_damage_dealt, p.kills.max(1) as i64))
                    .unwrap_or((0, 1));
                let avg = total_dmg / kills;
                let hp = (avg * 3).max(500 + floor_num as i64 * 60);
                enemy.hp = hp;
                enemy.max_hp = hp;
                enemy.base_damage = 15 + floor_num as i64 * 4;
                self.boss_extra = hp;
                self.boss_extra2 = 0;
                self.push_log(format!("★ THE OUROBOROS: Heals to full every 3 turns! HP: {}.", hp));
                self.push_log("Kill it within 3 turns. Heavy attacks.".to_string());
            }
            8 => {
                let start_hp = chaos_roll_verbose(0.5, room_seed).to_range(1000, 9999).max(1000);
                enemy.hp = start_hp;
                enemy.max_hp = start_hp;
                enemy.base_damage = 10 + floor_num as i64 * 3;
                self.boss_extra = start_hp;
                self.boss_extra2 = 0;
                self.push_log(format!("★ THE COLLATZ TITAN: HP follows Collatz. Start: {}.", start_hp));
                self.push_log("Each turn: even→HP/2, odd→HP×3+1. Attack when HP is low!".to_string());
            }
            9 => {
                let hp_each = 200 + floor_num as i64 * 40;
                enemy.hp = hp_each * 5;
                enemy.max_hp = hp_each * 5;
                enemy.base_damage = 8 + floor_num as i64;
                self.boss_extra = 0b11111;
                self.boss_extra2 = hp_each;
                self.push_log("★ THE COMMITTEE: 5 members, each immune to a different attack type.".to_string());
                self.push_log("[T] Taunt bypasses all immunities. Vary attacks.".to_string());
            }
            10 => {
                let hp = self.player.as_ref().map(|p| p.max_hp).unwrap_or(100);
                enemy.hp = hp;
                enemy.max_hp = hp;
                enemy.base_damage = 5;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.push_log(format!("★ THE RECURSION: HP = your max HP ({}). Every hit reflects!", hp));
                self.push_log("[D] Defend reduces reflection by VIT/2.".to_string());
            }
            11 => {
                enemy.hp = 999_999;
                enemy.max_hp = 999_999;
                enemy.base_damage = 10 + floor_num as i64 / 2;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.push_log("★ THE PARADOX: Immune to damage. Cannot flee.".to_string());
                self.push_log("[T] Taunt = Talk (CUNNING roll). [D] Defend = +5 CUN bonus.".to_string());
            }
            12 => {
                let hp = 2000 + floor_num as i64 * 200;
                enemy.hp = hp;
                enemy.max_hp = hp;
                enemy.base_damage = 25 + floor_num as i64 * 5;
                self.boss_extra = 1;
                self.boss_extra2 = 0;
                self.push_log("★ THE ALGORITHM REBORN: 3 phases. Adapts at 66% and 33% HP.".to_string());
                self.push_log("Vary attack types — it learns patterns.".to_string());
            }
            _ => {
                enemy.hp = (enemy.hp as f64 * 3.0) as i64;
                enemy.max_hp = enemy.hp;
                enemy.base_damage = (enemy.base_damage as f64 * 2.0) as i64;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
            }
        }
        self.enemy = Some(enemy);
        self.is_boss_fight = true;
        self.gauntlet_stage = 0;
        self.combat_state = Some(CombatState::new(room_seed));
        if let Some(ref mut cs) = self.combat_state {
            cs.is_cursed = self.is_cursed_floor;
        }
        self.emit_audio(AudioEvent::BossEncounterStart { boss_tier: 3 });
        // Trigger boss entrance animation overlay (3s)
        self.boss_entrance_timer = 180;
        self.boss_entrance_name = boss_name(boss_id).to_string();
        self.screen = AppScreen::Combat;
    }

    /// Pre-turn boss effects called before resolve_action.
    fn boss_pre_turn(&mut self, bid: u8) {
        match bid {
            7 => {
                // Ouroboros: heal to full every 3 turns
                if self.boss_turn > 1 && (self.boss_turn - 1) % 3 == 0 {
                    let max_hp = self.boss_extra;
                    if let Some(ref mut e) = self.enemy {
                        e.hp = max_hp;
                    }
                    self.push_log(format!("⟳ OUROBOROS heals to full ({} HP)! Cycle resets.", max_hp));
                }
            }
            8 => {
                // Collatz Titan: transform HP before player acts
                let n = self.boss_extra;
                let new_n = if n % 2 == 0 { n / 2 } else { n * 3 + 1 };
                self.boss_extra = new_n;
                if let Some(ref mut e) = self.enemy {
                    e.hp = new_n.max(1);
                    if new_n > e.max_hp {
                        e.max_hp = new_n;
                    }
                }
                let next = if new_n % 2 == 0 { new_n / 2 } else { new_n * 3 + 1 };
                self.push_log(format!("TITAN HP: {} → {} (Collatz). Next: {}", n, new_n, next));
                if new_n <= 4 {
                    self.push_log("★ ATTACK NOW — Titan at Collatz minimum!".to_string());
                }
            }
            6 => {
                // Null: set enemy damage via destiny roll
                let sc = self.floor_seed.wrapping_add(self.boss_turn as u64 * 7919);
                let dr = destiny_roll(0.5, sc);
                let base = 20 + self.floor_num as i64 * 5;
                let mult = (dr.final_value + 1.5).max(0.1);
                let dmg = ((base as f64 * mult) as i64).max(1);
                if let Some(ref mut e) = self.enemy {
                    e.base_damage = dmg;
                }
                self.push_log(format!("[NUL] Chaos suppressed — flat damage only. Enemy hits for {}.", dmg));
            }
            _ => {}
        }
    }

    /// Post-turn boss effects called after resolve_action events are processed.
    fn boss_post_turn(
        &mut self,
        bid: u8,
        action: &CombatAction,
        events: &[chaos_rpg_core::combat::CombatEvent],
    ) {
        use chaos_rpg_core::combat::CombatEvent;
        match bid {
            2 => {
                // Accountant: track ledger; deliver bill after 5 turns
                let dmg_this = events.iter().find_map(|e| {
                    if let CombatEvent::PlayerAttack { damage, .. } = e {
                        Some(*damage)
                    } else {
                        None
                    }
                }).unwrap_or(0);
                if dmg_this > 0 {
                    self.boss_extra += dmg_this;
                }
                if matches!(action, CombatAction::Defend) {
                    self.boss_extra2 += 1;
                }
                let lifetime = self.player.as_ref().map(|p| p.total_damage_dealt).unwrap_or(0);
                let defends = self.boss_extra2;
                self.push_log(format!(
                    "[LEDGER] Fight: {}  Lifetime: {}  Defends: {} ({}% reduction)",
                    self.boss_extra,
                    lifetime,
                    defends,
                    (defends * 20).min(80)
                ));
                if self.boss_turn >= 5 {
                    let bill_base = lifetime + self.boss_extra;
                    let reduction = (defends as f64 * 0.20).min(0.80);
                    let bill = ((bill_base as f64 * (1.0 - reduction)) as i64).max(1);
                    self.push_log(format!(
                        "★ THE BILL! {} × {}% kept = {} damage!",
                        bill_base,
                        100 - (reduction * 100.0) as u32,
                        bill
                    ));
                    if let Some(ref mut p) = self.player {
                        p.take_damage(bill);
                    }
                    self.player_flash = vc::flash_crit();
                    self.hit_shake = vc::shake_heavy();
                    if self.player.as_ref().map(|p| p.is_alive()).unwrap_or(false) {
                        let floor = self.floor_num;
                        let xp = 600 + floor as u64 * 150;
                        let gold = 300 + floor as i64 * 30;
                        if let Some(ref mut p) = self.player {
                            p.gain_xp(xp);
                            p.kills += 1;
                            p.gold += gold;
                        }
                        if let Some(ref mut e) = self.enemy {
                            e.hp = 0;
                        }
                        self.push_log(format!("Survived THE BILL! +{} XP, +{} gold.", xp, gold));
                        self.boss_id = None;
                        self.boss_turn = 0;
                    } else {
                        self.push_log("Your power was your undoing.".to_string());
                    }
                }
            }
            5 => {
                // Taxman: 1% HP attack per round
                let hp = self.enemy.as_ref().map(|e| e.hp).unwrap_or(0);
                if hp > 0 {
                    let is_defend = matches!(action, CombatAction::Defend);
                    let tax_atk = ((hp as f64 * 0.01) as i64).max(1);
                    let incoming = if is_defend { (tax_atk / 2).max(1) } else { tax_atk };
                    if let Some(ref mut p) = self.player {
                        p.take_damage(incoming);
                    }
                    self.push_log(format!("Taxman bills you {} HP (1% of {} remaining).", incoming, hp));
                    self.player_flash = vc::flash_normal();
                }
            }
            8 => {
                // Collatz Titan: sync boss_extra with HP after player attack
                let hp = self.enemy.as_ref().map(|e| e.hp).unwrap_or(0);
                self.boss_extra = hp;
            }
            10 => {
                // Recursion: reflect player attack back
                let player_dmg = events.iter().find_map(|e| {
                    if let CombatEvent::PlayerAttack { damage, .. } = e {
                        Some(*damage)
                    } else {
                        None
                    }
                }).unwrap_or(0);
                if player_dmg > 0 {
                    let vit = self.player.as_ref().map(|p| p.stats.vitality).unwrap_or(0);
                    let is_defend = matches!(action, CombatAction::Defend);
                    let reflection = if is_defend { (player_dmg - vit / 2).max(1) } else { player_dmg };
                    if let Some(ref mut p) = self.player {
                        p.take_damage(reflection);
                    }
                    self.boss_extra += reflection;
                    self.push_log(format!(
                        "RECURSION reflects {} back! (Total: {})",
                        reflection, self.boss_extra
                    ));
                    self.player_flash = vc::flash_normal();
                }
            }
            12 => {
                // Algorithm Reborn: phase transitions at 66% and 33%
                let (hp, max_hp) = self.enemy.as_ref().map(|e| (e.hp, e.max_hp)).unwrap_or((1, 1));
                let pct = hp * 100 / max_hp.max(1);
                let phase = self.boss_extra;
                if phase == 1 && pct <= 66 {
                    self.boss_extra = 2;
                    if let Some(ref mut e) = self.enemy {
                        e.base_damage = (e.base_damage as f64 * 1.5) as i64;
                    }
                    self.push_log("★ ALGORITHM REBORN Phase 2: Adapting — damage increased!".to_string());
                    self.hit_shake = vc::shake_boss();
                } else if phase == 2 && pct <= 33 {
                    self.boss_extra = 3;
                    if let Some(ref mut e) = self.enemy {
                        e.base_damage = (e.base_damage as f64 * 1.5) as i64;
                    }
                    self.push_log("★ ALGORITHM REBORN Phase 3: FINAL PROTOCOL! Maximum power!".to_string());
                    self.hit_shake = vc::shake_heavy();
                }
            }
            _ => {}
        }
    }

    /// Apply boss-specific win bonuses (Taxman gold return, etc.).
    fn boss_win_bonus(&mut self, bid: u8, xp: u64, gold: i64) -> (u64, i64) {
        match bid {
            5 => {
                let stolen = self.boss_extra;
                let returned = stolen + stolen / 5;
                if let Some(ref mut p) = self.player {
                    p.gold += returned;
                }
                self.push_log(format!("Gold returned: {} + 20% interest = {}!", stolen, returned));
                (xp, gold + returned)
            }
            7 if self.boss_turn <= 3 => {
                self.push_log("BURST KILL — Ouroboros down before its first reset!".to_string());
                (xp + 200, gold + 50)
            }
            _ => (xp, gold),
        }
    }

    /// Full-override: The Eigenstate (boss 4).
    fn boss_eigenstate(&mut self, action: CombatAction) {
        use chaos_rpg_core::chaos_pipeline::biased_chaos_roll;
        let floor = self.floor_num;
        let sc = self.floor_seed.wrapping_add(self.boss_turn as u64 * 131071);
        let luck_bias = self.player.as_ref()
            .map(|p| -(p.stats.luck as f64 / 200.0).clamp(-0.8, 0.8))
            .unwrap_or(0.0);
        let form_roll = biased_chaos_roll(luck_bias, luck_bias, sc);
        let is_form_a = form_roll.final_value > 0.0;
        self.last_roll = Some(form_roll);

        let tanky_hp = self.boss_extra;
        let tanky_max = self.enemy.as_ref().map(|e| e.max_hp).unwrap_or(500 + floor as i64 * 100);
        let oneshot_dmg = self.player.as_ref().map(|p| p.max_hp + 1).unwrap_or(101);
        let force = self.player.as_ref().map(|p| p.stats.force).unwrap_or(10);
        let vit = self.player.as_ref().map(|p| p.stats.vitality).unwrap_or(0);

        match action {
            CombatAction::Taunt => {
                let probe = 5 + floor as i64 / 2;
                if let Some(ref mut p) = self.player {
                    p.take_damage(probe);
                }
                self.player_flash = vc::flash_normal();
                if is_form_a {
                    self.push_log(format!("FORM A — huge HP, no attack. Strike next! (probe: {}dmg)", probe));
                } else {
                    self.push_log(format!("FORM B — 1 HP, one-shot. DEFEND next! (probe: {}dmg)", probe));
                }
            }
            CombatAction::Defend => {
                if !is_form_a {
                    let reduced = (oneshot_dmg - vit * 2).max(1);
                    if let Some(ref mut p) = self.player {
                        p.take_damage(reduced);
                    }
                    self.player_flash = vc::flash_crit();
                    self.hit_shake = vc::shake_heavy();
                    self.push_log(format!("Form B ATTACKS — defended! Took {} (VIT absorbed some).", reduced));
                } else {
                    self.push_log("Form A — you defend. No incoming attack.".to_string());
                }
            }
            CombatAction::Flee => {
                self.push_log("The Eigenstate holds. Cannot escape.".to_string());
            }
            _ => {
                if is_form_a {
                    let base = 5 + force / 5;
                    let roll = chaos_roll_verbose(force as f64 * 0.01, sc.wrapping_add(1));
                    let mut dmg = (base + (roll.final_value * base as f64 * 0.5) as i64).max(1);
                    if roll.is_critical() {
                        dmg = (dmg as f64 * 1.5) as i64;
                    }
                    if roll.is_catastrophe() {
                        dmg = 0;
                    }
                    let new_tanky = (tanky_hp - dmg).max(0);
                    self.boss_extra = new_tanky;
                    if let Some(ref mut e) = self.enemy {
                        e.hp = new_tanky.max(1);
                    }
                    self.enemy_flash = vc::flash_normal();
                    self.enemy_flash_col = (80, 200, 80);
                    self.push_log(format!("Form A — dealt {}. Tanky HP: {}/{}", dmg, new_tanky, tanky_max));
                    if new_tanky <= 0 {
                        let xp = 700 + floor as u64 * 150;
                        let gold = 180 + floor as i64 * 30;
                        if let Some(ref mut p) = self.player {
                            p.gain_xp(xp);
                            p.kills += 1;
                            p.gold += gold;
                        }
                        if let Some(ref mut e) = self.enemy {
                            e.hp = 0;
                        }
                        self.push_log("THE EIGENSTATE collapses — defeated!".to_string());
                        self.push_log(format!("+{} XP, +{} gold.", xp, gold));
                        self.boss_id = None;
                        self.boss_turn = 0;
                        self.complete_combat_win(xp, gold, true);
                        return;
                    }
                } else {
                    if let Some(ref mut e) = self.enemy {
                        e.hp = 0;
                    }
                    if let Some(ref mut p) = self.player {
                        p.take_damage(oneshot_dmg);
                    }
                    self.player_flash = vc::flash_crit();
                    self.hit_shake = vc::shake_heavy();
                    self.push_log(format!(
                        "Form B — 1 HP! You kill it... but it fires first: {} DAMAGE!",
                        oneshot_dmg
                    ));
                }
            }
        }

        if !self.player.as_ref().map(|p| p.is_alive()).unwrap_or(true) {
            self.push_log("The Eigenstate collapses onto you.".to_string());
            self.boss_id = None;
            self.boss_turn = 0;
            self.complete_combat_death();
        }
    }

    /// Full-override: The Paradox (boss 11).
    fn boss_paradox(&mut self, action: CombatAction) {
        let floor = self.floor_num;
        let sc = self.floor_seed.wrapping_add(self.boss_turn as u64 * 104729);
        let cunning = self.player.as_ref().map(|p| p.stats.cunning).unwrap_or(10);
        let cun_bonus = self.boss_extra2;
        let failed = self.boss_extra;

        match action {
            CombatAction::Defend => {
                self.boss_extra2 += 5;
                self.push_log(format!("You observe. Cunning bonus: +{}.", self.boss_extra2));
                if self.boss_turn > 3 {
                    let dmg = (5 + floor as i64).max(1);
                    if let Some(ref mut p) = self.player {
                        p.take_damage(dmg);
                    }
                    self.player_flash = vc::flash_normal();
                    self.push_log(format!("Paradox tires of stalling — {} damage.", dmg));
                }
            }
            CombatAction::Taunt => {
                let bias = ((cunning + cun_bonus) as f64 / 200.0).clamp(-0.8, 0.8);
                let roll = chaos_roll_verbose(bias, sc);
                let needed = (40 + floor as i64 / 2 - cun_bonus).max(10);
                let score = roll.to_range(0, 100);
                self.last_roll = Some(roll);
                self.push_log(format!(
                    "CUNNING roll: {} (need > {} with +{} bonus).",
                    score, needed, cun_bonus
                ));
                if score > needed {
                    let xp = 800 + floor as u64 * 150;
                    let gold = 150 + floor as i64 * 20;
                    if let Some(ref mut p) = self.player {
                        p.gain_xp(xp);
                        p.kills += 1;
                        p.gold += gold;
                    }
                    if let Some(ref mut e) = self.enemy {
                        e.hp = 0;
                    }
                    self.push_log("The Paradox acknowledges your logic. It dissolves.".to_string());
                    self.push_log(format!("+{} XP, +{} gold.", xp, gold));
                    self.boss_id = None;
                    self.boss_turn = 0;
                    self.complete_combat_win(xp, gold, true);
                    return;
                } else {
                    self.boss_extra += 1;
                    self.push_log(format!("Failed talk #{} — the Paradox takes something.", failed + 1));
                    if failed == 0 {
                        if let Some(ref mut p) = self.player {
                            if !p.known_spells.is_empty() {
                                p.known_spells.pop();
                            }
                        }
                        self.push_log("A spell dissolves into paradox!".to_string());
                    } else if self.player.as_ref().map(|p| !p.inventory.is_empty()).unwrap_or(false) {
                        if let Some(ref mut p) = self.player {
                            p.inventory.pop();
                        }
                        self.push_log("An item winks out of existence!".to_string());
                    }
                    let atk = (10 + floor as i64 / 2).max(1);
                    if let Some(ref mut p) = self.player {
                        p.take_damage(atk);
                    }
                    self.player_flash = vc::flash_normal();
                    self.push_log(format!("The Paradox punishes your failure: {} damage.", atk));
                }
            }
            CombatAction::Attack | CombatAction::HeavyAttack => {
                let force = self.player.as_ref().map(|p| p.stats.force).unwrap_or(10);
                let roll = chaos_roll_verbose(force as f64 * 0.01, sc);
                let base = 5 + force / 5;
                let heal = (base + (roll.final_value * base as f64 * 0.5) as i64).max(1);
                let max_hp = self.enemy.as_ref().map(|e| e.max_hp).unwrap_or(999_999);
                if let Some(ref mut e) = self.enemy {
                    e.hp = (e.hp + heal).min(max_hp);
                }
                let retaliation = (10 + floor as i64 / 2).max(1);
                if let Some(ref mut p) = self.player {
                    p.take_damage(retaliation);
                }
                self.player_flash = vc::flash_normal();
                self.push_log(format!(
                    "Attacking HEALS the Paradox by {}! Retaliation: {}dmg.",
                    heal, retaliation
                ));
                self.push_log("Use [T] Talk or [D] Observe.".to_string());
            }
            CombatAction::Flee => {
                let atk = (5 + floor as i64 / 3).max(1);
                if let Some(ref mut p) = self.player {
                    p.take_damage(atk);
                }
                self.player_flash = vc::flash_normal();
                self.push_log(format!("The Paradox is inescapable. {} damage for trying.", atk));
            }
            _ => {
                self.push_log("Only [T] Talk or [D] Observe work here.".to_string());
            }
        }

        if !self.player.as_ref().map(|p| p.is_alive()).unwrap_or(true) {
            self.push_log("The Paradox outlasts you.".to_string());
            self.boss_id = None;
            self.boss_turn = 0;
            self.complete_combat_death();
        }
    }

    /// Shared win completion used by full-override boss handlers.
    fn complete_combat_win(&mut self, xp: u64, gold: i64, loot_guaranteed: bool) {
        self.push_log(format!("Victory! +{} XP  +{} gold", xp, gold));
        if let Some(ref mut p) = self.player {
            if p.floor >= 50 {
                p.rooms_without_kill = 0;
            }
        }
        let loot_seed = self.floor_seed.wrapping_add(self.frame).wrapping_mul(6364136223846793005);
        let drop_chance = if loot_guaranteed { 100u64 } else { 60 };
        if loot_seed % 100 < drop_chance {
            let loot = Item::generate(loot_seed);
            self.push_log(format!("★ Item dropped: {}!", loot.name));
            self.loot_pending = Some(loot);
        }
        let next_screen = if self.loot_pending.is_some() {
            let loot = self.loot_pending.take().unwrap();
            self.room_event = RoomEvent::empty();
            self.room_event.title = "★ LOOT DROPPED ★".to_string();
            self.room_event.lines = vec![
                format!("Enemy dropped: {}", loot.name),
                format!("Rarity: {}", loot.rarity.name()),
            ];
            for m in &loot.stat_modifiers {
                self.room_event.lines.push(format!("  {:+} {}", m.value, m.stat));
            }
            self.room_event.lines.push(String::new());
            self.room_event.lines.push("[P] Pick up   [Enter] Leave".to_string());
            self.room_event.pending_item = Some(loot);
            AppScreen::RoomView
        } else {
            self.advance_floor_room();
            if self.screen == AppScreen::GameOver || self.screen == AppScreen::Victory {
                self.screen.clone()
            } else {
                AppScreen::FloorNav
            }
        };
        self.kill_linger = vc::kill_linger();
        self.post_combat_screen = Some(next_screen);
        self.emit_audio(AudioEvent::EntityDied { is_player: false });
    }

    /// Shared death completion used by full-override boss handlers.
    fn complete_combat_death(&mut self) {
        self.particles.clear();
        self.player_flash = 0;
        self.enemy_flash = 0;
        self.hit_shake = 0;
        let enemy_name = self.enemy.as_ref().map(|e| e.name.clone()).unwrap_or_default();
        let enemy_dmg = self.enemy.as_ref().map(|e| e.base_damage).unwrap_or(5);
        if let Some(ref p) = self.player {
            let method = if p.spells_cast > p.kills * 2 { "spell" } else { "physical" };
            let nem = NemesisRecord::new(
                enemy_name.clone(),
                p.floor,
                enemy_dmg,
                p.class.name().to_string(),
                method,
            );
            save_nemesis(&nem);
            self.push_log(format!("☠ {} is now your Nemesis.", enemy_name));
        }
        self.save_score_now();
        self.emit_audio(AudioEvent::EntityDied { is_player: true });
        self.emit_audio(AudioEvent::GameOver);
        // Start death cinematic
        let final_dmg = self.player.as_ref()
            .map(|p| p.run_stats.final_blow_damage).unwrap_or(enemy_dmg);
        let epitaph = self.player.as_ref().map(|p|
            format!("A {} who reached floor {}. The mathematics consumed them.",
                p.class.name(), p.floor)
        ).unwrap_or_else(|| "The chaos consumed them.".to_string());
        self.death_seq.start(final_dmg, &enemy_name, &epitaph);
        self.death_cinematic_done = false;
        self.trigger_earthquake(0.9, 50);
        self.screen = AppScreen::GameOver;
    }

    fn resolve_combat_action(&mut self, action: CombatAction) {
        // ── Boss full-override handlers ──
        if let Some(bid) = self.boss_id {
            self.boss_turn += 1;
            match bid {
                4 => {
                    self.boss_eigenstate(action);
                    return;
                }
                11 => {
                    self.boss_paradox(action);
                    return;
                }
                _ => {
                    self.boss_pre_turn(bid);
                }
            }
        }

        let (player, enemy, cstate) = match (&mut self.player, &mut self.enemy, &mut self.combat_state) {
            (Some(p), Some(e), Some(cs)) => (p, e, cs),
            _ => return,
        };

        let level_before = player.level;
        let action_clone = action.clone();
        let (events, outcome) = resolve_action(player, enemy, action, cstate);

        if let Some(ref roll) = cstate.last_roll {
            self.last_roll = Some(roll.clone());
        }

        // Track final blow for cause-of-death
        {
            use chaos_rpg_core::combat::CombatEvent;
            let last_hit = events.iter().rev().find_map(|ev| {
                if let CombatEvent::EnemyAttack { damage, is_crit } = ev {
                    Some((*damage, *is_crit))
                } else { None }
            });
            if let Some((dmg, crit)) = last_hit {
                let ename = enemy.name.clone();
                let floor = player.floor;
                let crit_tag = if crit { " [CRIT]" } else { "" };
                player.run_stats.cause_of_death =
                    format!("Floor {} — {} hit for {}{}", floor, ename, dmg, crit_tag);
                player.run_stats.final_blow_damage = dmg;
            }
        }

        for ev in &events {
            self.combat_log.push(ev.to_display_string());
        }

        // ── Record HP before events for ghost bars ────────────────────────────
        let php_before = self.player.as_ref().map(|p| p.current_hp as f32 / p.max_hp.max(1) as f32).unwrap_or(1.0);
        let ehp_before = self.enemy.as_ref().map(|e| e.hp as f32 / e.max_hp.max(1) as f32).unwrap_or(1.0);

        // ── Start combat animations from events ───────────────────────────────
        {
            use chaos_rpg_core::combat::CombatEvent;
            let anim_speed = self.anim_config.effective_combat();
            let wname = self.player.as_ref()
                .and_then(|p| p.equipped.weapon.as_ref())
                .map(|w| w.name.as_str())
                .unwrap_or("");
            let weapon = weapon_kind_from_name(wname);
            let spell_nm = self.last_spell_name.clone();
            let element = spell_element_from_name(&spell_nm);
            for ev in &events {
                match ev {
                    CombatEvent::PlayerAttack { damage, is_crit } => {
                        if self.last_action_type == 2 {
                            self.combat_anim.start_player_heavy(*damage, *is_crit, weapon, anim_speed);
                        } else {
                            self.combat_anim.start_player_melee(*damage, *is_crit, weapon, anim_speed);
                        }
                    }
                    CombatEvent::SpellCast { damage, backfired, .. } => {
                        self.combat_anim.start_player_spell(*damage, false, element, self.anim_config.effective_spell());
                    }
                    CombatEvent::EnemyAttack { damage, is_crit } => {
                        self.combat_anim.start_enemy_attack(*damage, *is_crit, anim_speed);
                    }
                    CombatEvent::PlayerDefend { .. } => {
                        self.combat_anim.start_defend(anim_speed);
                    }
                    CombatEvent::StatusApplied { name } => {
                        let sk = status_kind_from_name(name);
                        // Fire status ring on enemy (assume enemy applied it to player or vice versa)
                        self.combat_anim.start_status_apply(sk, false, anim_speed);
                    }
                    _ => {}
                }
            }
        }

        // ── Spawn visual effects ──────────────────────────────────────────────
        {
            use chaos_rpg_core::combat::CombatEvent;
            for ev in &events {
                match ev {
                    // Enemy takes damage from player
                    CombatEvent::PlayerAttack { damage, is_crit } => {
                        if *is_crit {
                            let jx = 10 + (self.frame % 10) as i32;
                            self.particles.push(Particle::new(jx,     6, format!("★ CRIT ★"), (255, 215, 0), vc::particle_lifetime_crit()));
                            self.particles.push(Particle::new(jx + 2, 8, format!("{}", damage), (255, 240, 80), vc::particle_lifetime_crit()));
                            self.particles.push(Particle::new(jx + 4, 10, "✦✦✦".to_string(), (255, 180, 0), vc::particle_lifetime_crit()));
                            emit_crit_burst(&mut self.particles, jx as f32 + 4.0, 8.0);
                            self.enemy_flash = vc::flash_crit();
                            self.enemy_flash_col = (255, 215, 0);
                            self.hit_shake = vc::shake_crit();
                            self.trigger_ripple(38.0, 18.0, (1.0, 0.8, 0.0));
                            // Update enemy ghost bar
                            self.ghost_enemy_hp = ehp_before;
                            self.ghost_enemy_timer = 60;
                        } else {
                            let jx = 12 + (self.frame % 8) as i32;
                            self.particles.push(Particle::new(jx, 7, format!("-{}", damage), (80, 220, 80), vc::particle_lifetime_normal()));
                            emit_hit_sparks(&mut self.particles, jx as f32, 7.0, (80, 220, 80), 6);
                            self.enemy_flash = vc::flash_normal();
                            self.enemy_flash_col = (80, 220, 80);
                            self.ghost_enemy_hp = ehp_before;
                            self.ghost_enemy_timer = 50;
                        }
                    }
                    // Player takes damage from enemy
                    CombatEvent::EnemyAttack { damage, is_crit } => {
                        if *is_crit {
                            let jx = 95 + (self.frame % 10) as i32;
                            self.particles.push(Particle::new(jx,     5, format!("☠ CRIT ☠"), (255, 40, 0), vc::particle_lifetime_crit()));
                            self.particles.push(Particle::new(jx + 2, 8, format!("-{} !", damage), (255, 80, 30), vc::particle_lifetime_crit()));
                            self.particles.push(Particle::new(jx + 4, 11, "!!!".to_string(), (200, 20, 20), vc::particle_lifetime_crit()));
                            emit_hit_sparks(&mut self.particles, jx as f32 + 4.0, 8.0, (220, 50, 20), 10);
                            self.player_flash = vc::flash_crit();
                            self.hit_shake = vc::shake_crit();
                            self.trigger_ripple(118.0, 20.0, (1.0, 0.15, 0.0));
                            self.trigger_earthquake(0.4, 15);
                        } else {
                            let jx = 95 + (self.frame % 8) as i32;
                            self.particles.push(Particle::new(jx, 7, format!("-{}", damage), (220, 50, 50), vc::particle_lifetime_normal()));
                            emit_hit_sparks(&mut self.particles, jx as f32, 7.0, (200, 60, 60), 5);
                            self.player_flash = vc::flash_normal();
                            if self.is_boss_fight { self.hit_shake = vc::shake_boss(); }
                        }
                        // Update player ghost bar
                        self.ghost_player_hp = php_before;
                        self.ghost_player_timer = 60;
                    }
                    // Healing
                    CombatEvent::PlayerHealed { amount } => {
                        self.particles.push(Particle::new(100, 8,  format!("+{} HP", amount), (50, 220, 100), vc::particle_lifetime_heal()));
                        self.particles.push(Particle::new(102, 11, "♥ ♥ ♥".to_string(), (80, 255, 130), vc::particle_lifetime_heal()));
                    }
                    // Spell cast
                    CombatEvent::SpellCast { damage, backfired, .. } => {
                        if *backfired {
                            self.spell_beam_col = (220, 50, 50);
                            self.particles.push(Particle::new(95, 5,  format!("BACKFIRE!"), (255, 60, 0), vc::particle_lifetime_backfire()));
                            self.particles.push(Particle::new(97, 8,  format!("-{}", damage), (255, 120, 40), vc::particle_lifetime_backfire()));
                            self.particles.push(Particle::new(99, 11, "⚡⚡⚡".to_string(), (255, 80, 0), vc::particle_lifetime_backfire()));
                            self.hit_shake = vc::shake_heavy();
                            self.player_flash = vc::flash_crit();
                            self.ghost_player_hp = php_before;
                            self.ghost_player_timer = 60;
                        } else {
                            self.spell_beam_col = (80, 140, 255);
                            self.particles.push(Particle::new(10, 5,  format!("✦ SPELL ✦"), (150, 200, 255), vc::particle_lifetime_spell()));
                            self.particles.push(Particle::new(12, 8,  format!("-{}", damage), (130, 190, 255), vc::particle_lifetime_spell()));
                            self.enemy_flash = vc::flash_crit();
                            self.enemy_flash_col = (80, 140, 255);
                            self.ghost_enemy_hp = ehp_before;
                            self.ghost_enemy_timer = 50;
                        }
                        self.spell_beam = vc::beam_charge() + vc::beam_hold();
                    }
                    // Enemy kill reward — big celebration + death explosion
                    CombatEvent::EnemyDied { xp, gold } => {
                        self.particles.push(Particle::new(5,  4, format!("+{} XP", xp),    (255, 215, 0), vc::particle_lifetime_reward()));
                        self.particles.push(Particle::new(5,  7, format!("+{}g", gold),     (255, 180, 60), vc::particle_lifetime_reward()));
                        self.particles.push(Particle::new(12, 5, "★ DEFEATED ★".to_string(), (255, 240, 100), vc::particle_lifetime_reward()));
                        self.particles.push(Particle::new(14, 9, "✦✦✦✦✦".to_string(),       (200, 160, 40), vc::particle_lifetime_reward()));
                        let t_danger = self.theme().danger;
                        emit_death_explosion(&mut self.particles, 38.0, 18.0, t_danger);
                        self.enemy_flash = vc::kill_flash();
                        self.enemy_flash_col = (255, 255, 255);
                        self.hit_shake = vc::shake_crit();
                        self.trigger_earthquake(0.55, 30);
                        self.trigger_ripple(38.0, 18.0, (0.9, 0.7, 0.1));
                    }
                    // Status applied
                    CombatEvent::StatusApplied { name } => {
                        self.particles.push(Particle::new(10, 10,
                            format!("⚡ {}", name), (200, 150, 60), vc::particle_lifetime_status()));
                    }
                    // Defend
                    CombatEvent::PlayerDefend { damage_reduced } if *damage_reduced > 0 => {
                        self.particles.push(Particle::new(95, 6,  format!("🛡 BLOCK"), (80, 140, 200), vc::particle_lifetime_normal()));
                        self.particles.push(Particle::new(97, 9,  format!("-{}", damage_reduced), (120, 180, 255), vc::particle_lifetime_normal()));
                    }
                    // Item equipped during combat
                    CombatEvent::ItemEquipped { name, slot } => {
                        let s: String = name.chars().take(14).collect();
                        self.particles.push(Particle::new(95, 10,
                            format!("⚙ {} → {}", s, slot), (60, 200, 220), vc::particle_lifetime_status()));
                    }
                    // Item durability warning — orange flash when under 25%
                    CombatEvent::ItemDurabilityLost { name, durability, max_durability } => {
                        if *durability > 0 && *durability <= max_durability / 4 {
                            let s: String = name.chars().take(10).collect();
                            self.particles.push(Particle::new(95, 13,
                                format!("⚠ {} cracking!", s), (255, 130, 30), vc::particle_lifetime_status()));
                            self.player_flash = vc::flash_normal();
                        }
                    }
                    // Item destroyed — big red explosion
                    CombatEvent::ItemDestroyed { name } => {
                        let s: String = name.chars().take(12).collect();
                        self.particles.push(Particle::new(95, 8,  format!("💥 SHATTERED!"), (255, 30, 30), vc::particle_lifetime_crit()));
                        self.particles.push(Particle::new(97, 11, format!("{}", s),          (200, 40, 40), vc::particle_lifetime_crit()));
                        self.particles.push(Particle::new(99, 14, "▓▒░ DESTROYED ░▒▓".to_string(), (180, 20, 20), vc::particle_lifetime_crit()));
                        self.player_flash = vc::flash_crit();
                        self.hit_shake = vc::shake_heavy();
                    }
                    _ => {}
                }
            }
        }

        // ── Audio: emit per-event SFX ────────────────────────────────────────
        {
            use chaos_rpg_core::combat::CombatEvent;
            use chaos_rpg_core::audio_events::AudioEvent as AE;
            for ev in &events {
                match ev {
                    CombatEvent::PlayerAttack { damage, is_crit } => {
                        self.emit_audio(AE::DamageDealt { amount: *damage as i32, is_crit: *is_crit });
                        if *is_crit { self.emit_audio(AE::EngineCritical); }
                    }
                    CombatEvent::EnemyAttack { damage, is_crit } => {
                        self.emit_audio(AE::EnemyAttack);
                        self.emit_audio(AE::DamageDealt { amount: *damage as i32, is_crit: *is_crit });
                        if *is_crit { self.emit_audio(AE::EngineCritical); }
                    }
                    CombatEvent::PlayerHealed { amount } => {
                        self.emit_audio(AE::HealApplied { amount: *amount as i32 });
                    }
                    CombatEvent::EnemyDied { .. } => {
                        self.emit_audio(AE::EntityDied { is_player: false });
                    }
                    CombatEvent::StatusApplied { .. } => {
                        self.emit_audio(AE::StatusApplied);
                    }
                    _ => {}
                }
            }
            // Chaos engine audio from the roll chain
            if let Some(ref roll) = self.last_roll.clone() {
                for (i, step) in roll.chain.iter().enumerate() {
                    self.emit_audio(AE::ChaosEngineRoll { engine_id: (i % 10) as u8 });
                }
                if roll.chain.len() > 3 {
                    self.emit_audio(AE::ChaosCascade { depth: roll.chain.len() as u8 });
                }
                if roll.is_critical() {
                    self.emit_audio(AE::EngineCritical);
                }
            }
        }

        // ── Misery event wiring ───────────────────────────────────────────────
        if let Some(ref mut p) = self.player {
            use chaos_rpg_core::combat::CombatEvent;
            use chaos_rpg_core::misery_system::MiserySource;
            for ev in &events {
                match ev {
                    CombatEvent::EnemyAttack { damage, is_crit } => {
                        let new_ms = p.misery.add_misery(MiserySource::DamageTaken, *damage as f64);
                        if *is_crit { p.misery.add_misery(MiserySource::Headshot, 0.0); }
                        p.run_stats.record_damage_taken(*damage, *is_crit);
                        for ms in new_ms {
                            self.combat_log.push(format!("[MISERY] {}", ms.title()));
                        }
                        // Pity mercy check
                        let pity = chaos_rpg_core::misery_system::MiseryState::enemy_pity_chance(p.stats.total());
                        if pity > 0.0 {
                            let roll = (p.seed.wrapping_mul(p.rooms_cleared as u64 + self.frame)) % 100;
                            if roll < (pity * 100.0) as u64 {
                                p.misery.add_misery(MiserySource::EnemyPitiedYou, 0.0);
                                p.run_stats.enemies_pitied_you += 1;
                                self.combat_log.push(format!("Enemy looks at you with pity. Attack skipped."));
                            }
                        }
                    }
                    CombatEvent::PlayerAttack { damage, is_crit } => {
                        p.run_stats.record_damage_dealt(*damage, None, *is_crit);
                        let new_passives = p.misery.increment_defiance_roll();
                        for passive in new_passives {
                            self.combat_log.push(format!("[DEFIANCE] {} UNLOCKED!", passive.name()));
                        }
                    }
                    CombatEvent::PlayerFleeFailed => {
                        p.misery.add_misery(MiserySource::FleeFailed, 0.0);
                    }
                    _ => {}
                }
            }
            // Cosmic joke flavor
            if p.misery.cosmic_joke {
                if let Some(line) = chaos_rpg_core::misery_system::MiseryState::cosmic_joke_combat_line(
                    p.seed, self.frame) {
                    self.combat_log.push(format!("  {}", line));
                }
            }
        }

        // last_roll stored for on-screen display — not pushed to log

        // Tick status effects (start of each new turn after action)
        if let Some(ref mut p) = self.player {
            let (_dmg, msgs) = p.tick_status_effects();
            for m in msgs { self.combat_log.push(m); }
        }

        // Level up check
        let (level_after, skill_pts) = self.player.as_ref()
            .map(|p| (p.level, p.skill_points)).unwrap_or((0, 0));
        if level_after > level_before {
            self.push_log(format!("★ LEVEL UP! Now level {}!", level_after));
            if skill_pts > 0 {
                self.push_log(format!("  {} skill point(s) available!", skill_pts));
            }
            self.emit_audio(AudioEvent::LevelUp);
            // Level-up fountain particles on player panel
            emit_level_up_fountain(&mut self.particles, 118.0, 20.0);
            self.trigger_pulse_ring(80.0, 40.0, (1.0, 0.85, 0.0), 1.2);
            let lvl_speed = self.anim_config.effective_combat();
            self.combat_anim.start_level_up_pillar(lvl_speed);
        }

        // ── Boss post-turn hooks ──────────────────────────────────────────────
        let bid_opt = self.boss_id;
        if let Some(bid) = bid_opt {
            self.boss_post_turn(bid, &action_clone, &events);
        }

        // Fibonacci Hydra (boss 3): intercept PlayerWon to spawn next generation
        let outcome = if let (Some(3), CombatOutcome::PlayerWon { .. }) = (bid_opt, &outcome) {
            let gen = self.boss_extra;
            let splits = self.boss_extra2;
            if gen < 7 && splits < 10 {
                self.boss_extra += 1;
                let next_gen = self.boss_extra;
                let next_hp = ((300 + self.floor_num as i64 * 40) * (1_i64 << (next_gen - 1).min(30))).max(1);
                self.push_log(format!(
                    "★ HYDRA: Generation {} rises with {} HP!",
                    next_gen, next_hp
                ));
                if let Some(ref mut e) = self.enemy {
                    e.hp = next_hp;
                    e.max_hp = next_hp;
                }
                if let Some(ref mut cs) = self.combat_state {
                    cs.turn = 0;
                }
                CombatOutcome::Ongoing
            } else {
                outcome
            }
        } else {
            outcome
        };

        match outcome {
            CombatOutcome::PlayerWon { xp, gold } => {
                let (xp, gold) = if let Some(bid) = self.boss_id.take() {
                    self.boss_turn = 0;
                    self.boss_extra = 0;
                    self.boss_extra2 = 0;
                    self.boss_win_bonus(bid, xp, gold)
                } else {
                    (xp, gold)
                };
                self.push_log(format!("Victory! +{} XP  +{} gold", xp, gold));
                if let Some(ref mut p) = self.player {
                    p.kills += 1;
                    let kills_before = p.kills;
                    if p.floor >= 50 {
                        p.rooms_without_kill = 0;
                    }
                    let _ = kills_before;
                }

                // Nemesis tracking: if nemesis killed, clear it and reward
                if let Some(ref nem) = self.nemesis_record.clone() {
                    if self.enemy.as_ref().map(|e| e.name.contains(&nem.enemy_name)).unwrap_or(false) {
                        clear_nemesis();
                        self.nemesis_record = None;
                        self.push_log("☆ Nemesis defeated! Grudge settled.".to_string());
                        if let Some(ref mut p) = self.player {
                            let (sname, _) = p.highest_stat();
                            match sname {
                                "Vitality"  => p.stats.vitality  += 50,
                                "Force"     => p.stats.force     += 50,
                                "Mana"      => p.stats.mana      += 50,
                                "Cunning"   => p.stats.cunning   += 50,
                                "Precision" => p.stats.precision += 50,
                                "Entropy"   => p.stats.entropy   += 50,
                                _           => p.stats.luck      += 50,
                            }
                        }
                    }
                }

                // Loot drop
                let loot_seed = self.floor_seed.wrapping_add(self.frame).wrapping_mul(6364136223846793005);
                let drop_chance = if self.is_boss_fight { 100 } else { 40 };
                if loot_seed % 100 < drop_chance {
                    let loot = Item::generate(loot_seed);
                    self.push_log(format!("★ Item dropped: {}!", loot.name));
                    self.loot_pending = Some(loot);
                }

                // Boss gauntlet: advance to next fight
                if self.gauntlet_stage > 0 && !self.gauntlet_enemies.is_empty() {
                    self.gauntlet_stage += 1;
                    let next = self.gauntlet_enemies.remove(0);
                    self.push_log(format!("GAUNTLET: Fight {}/3", self.gauntlet_stage));
                    self.enemy = Some(next);
                    let ns = self.floor_seed.wrapping_add(self.gauntlet_stage as u64 * 1337);
                    self.combat_state = Some(CombatState::new(ns));
                    if let Some(ref mut cs) = self.combat_state { cs.is_cursed = self.is_cursed_floor; }
                    return; // Stay in combat
                }

                self.gauntlet_stage = 0;
                // NOTE: self.enemy is intentionally NOT nulled here.
                // draw_combat reads it during the kill_linger frames. It's cleared when linger ends.
                // Show loot if pending, else floor nav — but linger on combat first
                let next_screen = if self.loot_pending.is_some() {
                    let loot = self.loot_pending.take().unwrap();
                    self.room_event = RoomEvent::empty();
                    self.room_event.title = "★ LOOT DROPPED ★".to_string();
                    self.room_event.lines = vec![
                        format!("Enemy dropped: {}", loot.name),
                        format!("Rarity: {}", loot.rarity.name()),
                    ];
                    for m in &loot.stat_modifiers {
                        self.room_event.lines.push(format!("  {:+} {}", m.value, m.stat));
                    }
                    self.room_event.lines.push(String::new());
                    self.room_event.lines.push("[P] Pick up   [Enter] Leave".to_string());
                    self.room_event.pending_item = Some(loot);
                    AppScreen::RoomView
                } else {
                    self.advance_floor_room();
                    if self.screen == AppScreen::GameOver || self.screen == AppScreen::Victory {
                        self.screen.clone()
                    } else {
                        AppScreen::FloorNav
                    }
                };
                self.kill_linger = vc::kill_linger();
                self.post_combat_screen = Some(next_screen);
            }

            CombatOutcome::PlayerDied => {
                self.boss_id = None;
                self.boss_turn = 0;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.particles.clear();
                self.player_flash = 0; self.enemy_flash = 0; self.hit_shake = 0;
                // Save nemesis
                let enemy_name = self.enemy.as_ref().map(|e| e.name.clone()).unwrap_or_default();
                let enemy_dmg  = self.enemy.as_ref().map(|e| e.base_damage).unwrap_or(5);
                if let Some(ref p) = self.player {
                    let method = if p.spells_cast > p.kills * 2 { "spell" } else { "physical" };
                    let nem = NemesisRecord::new(
                        enemy_name.clone(), p.floor, enemy_dmg,
                        p.class.name().to_string(), method,
                    );
                    save_nemesis(&nem);
                    self.push_log(format!("☠ {} is now your Nemesis.", enemy_name));
                }
                self.save_score_now();
                self.emit_audio(AudioEvent::EntityDied { is_player: true });
                self.emit_audio(AudioEvent::GameOver);
                // Start death cinematic
                let final_dmg = self.player.as_ref()
                    .map(|p| p.run_stats.final_blow_damage).unwrap_or(enemy_dmg);
                let epitaph = self.player.as_ref().map(|p|
                    format!("A {} who reached floor {}. The mathematics consumed them.",
                        p.class.name(), p.floor)
                ).unwrap_or_else(|| "The chaos consumed them.".to_string());
                self.death_seq.start(final_dmg, &enemy_name, &epitaph);
                self.death_cinematic_done = false;
                self.trigger_earthquake(0.9, 50);
                self.screen = AppScreen::GameOver;
            }

            CombatOutcome::PlayerFled => {
                self.boss_id = None;
                self.boss_turn = 0;
                self.boss_extra = 0;
                self.boss_extra2 = 0;
                self.push_log("You escaped into the chaos!".to_string());
                self.emit_audio(AudioEvent::PlayerFled);
                let flee_speed = self.anim_config.effective_combat();
                self.combat_anim.start_flee(true, flee_speed);
                self.enemy = None;
                if let Some(ref mut p) = self.player { p.rooms_without_kill += 1; }
                self.advance_floor_room();
                if self.screen != AppScreen::GameOver && self.screen != AppScreen::Victory {
                    self.screen = AppScreen::FloorNav;
                }
            }

            CombatOutcome::Ongoing => {} // stay in combat
        }
    }

    fn save_score_now(&mut self) {
        use chaos_rpg_core::{
            legacy_system::{GraveyardEntry, LegacyData},
            scoreboard::{save_misery_score, MiseryEntry},
            misery_system::MiseryState,
        };
        if let Some(ref p) = self.player {
            let score_val = p.xp + p.gold as u64 + (p.kills * 100) as u64 + (p.floor as u64 * 500);
            let tier = p.power_tier();
            let underdog = p.underdog_multiplier();
            let misery = p.misery.misery_index;
            let entry = ScoreEntry::new(
                p.name.clone(), p.class.name().to_string(),
                score_val, p.floor, p.kills, 0,
            ).with_tier(tier.name()).with_misery(misery, underdog);
            let _ = save_score(entry);

            // Hall of Misery
            if misery >= 100.0 {
                let me = MiseryEntry::new(
                    &p.name, p.class.name().to_string(), misery, p.floor,
                    tier.name(), p.misery.spite_total_spent, p.misery.defiance_rolls,
                    &p.run_stats.cause_of_death, underdog,
                );
                let _ = save_misery_score(me);
            }

            // Graveyard / legacy
            let all_neg = p.stats.vitality < 0 && p.stats.force < 0 && p.stats.mana < 0;
            let epitaph = GraveyardEntry::generate_epitaph(
                p.class.name(), p.floor, p.kills, p.total_damage_dealt,
                misery, p.spells_cast, all_neg,
                p.run_stats.deaths_to_backfire > 0, tier.name(),
            );
            let ge = GraveyardEntry {
                name: p.name.clone(), class: p.class.name().to_string(),
                level: p.level, floor: p.floor, power_tier: tier.name().to_string(),
                misery_index: misery, cause_of_death: p.run_stats.cause_of_death.clone(),
                kills: p.kills, score: score_val, date: String::new(), epitaph: epitaph.clone(),
            };
            let mut legacy = LegacyData::load();
            legacy.record_run(
                ge, p.total_damage_dealt, p.total_damage_taken, p.gold,
                misery, p.misery.spite_total_spent, p.run_stats.total_rolls,
                p.run_stats.deaths_to_backfire > 0, false, p.seed, tier.name(),
            );
            legacy.save();

            // Run history
            let mode_str = match self.game_mode {
                GameMode::Story    => "Story",
                GameMode::Infinite => "Infinite",
                GameMode::Daily    => "Daily",
            };
            let won = self.screen == AppScreen::Victory;

            // Build auto-narrative
            let pos_stats: Vec<(String, i64)> = [
                ("Vitality", p.stats.vitality), ("Force", p.stats.force),
                ("Mana", p.stats.mana), ("Cunning", p.stats.cunning),
                ("Precision", p.stats.precision), ("Entropy", p.stats.entropy),
                ("Luck", p.stats.luck),
            ].iter().filter(|(_, v)| *v > 0).map(|(n, v)| (n.to_string(), *v)).collect();
            let neg_stats: Vec<(String, i64)> = [
                ("Vitality", p.stats.vitality), ("Force", p.stats.force),
                ("Mana", p.stats.mana), ("Cunning", p.stats.cunning),
                ("Precision", p.stats.precision), ("Entropy", p.stats.entropy),
                ("Luck", p.stats.luck),
            ].iter().filter(|(_, v)| *v < 0).map(|(n, v)| (n.to_string(), *v)).collect();
            let narrative = chaos_rpg_core::lore::narrative::RunNarrative {
                character_name:       p.name.clone(),
                character_class:      p.class.name().to_string(),
                character_background: p.background.name().to_string(),
                difficulty:           p.difficulty.name().to_string(),
                game_mode:            mode_str.to_string(),
                destiny_roll_value:   0.0,
                positive_stats:       pos_stats,
                negative_stats:       neg_stats,
                boon_name:            p.boon.map(|b| b.name().to_string()),
                final_floor:          p.floor,
                final_tier:           tier.name().to_string(),
                total_kills:          p.kills as u64,
                total_damage:         p.total_damage_dealt,
                events:               p.narrative_events.clone(),
                custom_origin:        if p.character_lore.origin.is_empty() { None }
                                      else { Some(p.character_lore.origin.clone()) },
                epitaph:              epitaph.clone(),
                won,
            };
            let auto_narrative = narrative.generate();
            let character_lore_opt = if p.character_lore.origin.is_empty()
                && p.character_lore.motivation.is_empty()
            {
                None
            } else {
                Some(p.character_lore.clone())
            };

            let record = RunRecord {
                date:           chrono_date_simple(),
                name:           p.name.clone(),
                class:          p.class.name().to_string(),
                difficulty:     p.difficulty.name().to_string(),
                game_mode:      mode_str.to_string(),
                floor:          p.floor,
                level:          p.level,
                kills:          p.kills as u64,
                score:          score_val,
                damage_dealt:   p.run_stats.damage_dealt,
                damage_taken:   p.run_stats.damage_taken,
                highest_hit:    p.run_stats.highest_single_hit,
                spells_cast:    p.run_stats.spells_cast,
                items_used:     p.items_used,
                gold:           p.gold,
                misery_index:   misery,
                corruption:     p.corruption,
                power_tier:     tier.name().to_string(),
                cause_of_death: p.run_stats.cause_of_death.clone(),
                seed:           p.seed,
                won,
                epitaph:        epitaph.clone(),
                auto_narrative,
                character_lore: character_lore_opt,
            };
            self.run_history.push(record);

            // Achievement check
            let run_summary = RunSummary {
                floor:           p.floor,
                kills:           p.kills as u64,
                level:           p.level,
                class:           p.class.name().to_string(),
                difficulty:      p.difficulty.name().to_string(),
                damage_dealt:    p.run_stats.damage_dealt,
                damage_taken:    p.run_stats.damage_taken,
                highest_hit:     p.run_stats.highest_single_hit,
                spells_cast:     p.run_stats.spells_cast,
                items_used:      p.items_used,
                gold:            p.gold,
                misery_index:    misery,
                corruption:      p.corruption,
                power_tier:      tier.name().to_string(),
                total_stats:     p.stats.total(),
                cause_of_death:  p.run_stats.cause_of_death.clone(),
                rooms_cleared:   p.rooms_cleared,
                deaths_in_run:   0,
                fled_count:      0,
                all_stats_negative: all_neg,
                total_runs:      self.run_history.runs.len() as u32,
                total_deaths:    0,
                won:             self.screen == AppScreen::Victory,
                seed:            p.seed,
            };
            self.achievements.check_run(&run_summary);
            self.achievements.save();
            if let Some((banner_text, rarity_str)) = self.achievements.pop_banner_with_rarity() {
                // Legacy simple banner (for terminal/other consumers)
                self.achievement_banner = Some(banner_text.clone());
                self.achievement_banner_frames = 180;
                // Rich rarity banner
                let rarity = rarity_from_name(&rarity_str);
                let speed = self.anim_config.effective_achievement();
                self.rich_banner.start(&banner_text, rarity, speed);
            }

            // Build shareable recap text
            self.last_recap_text = build_recap_text(p, score_val, misery, tier.name(), &epitaph, mode_str, p.seed);

            // Daily leaderboard auto-submit
            if self.game_mode == GameMode::Daily && !self.daily_submitted {
                let entry = DailyEntry {
                    date:  chrono_date_simple(),
                    name:  if !self.config.meta.player_name.is_empty() {
                        self.config.meta.player_name.clone()
                    } else { p.name.clone() },
                    class: p.class.name().to_string(),
                    floor: p.floor,
                    score: score_val,
                    kills: p.kills as u64,
                    seed:  p.seed,
                    won:   self.screen == AppScreen::Victory,
                };
                let improved = self.daily_store.record(entry.clone());
                if improved && self.config.leaderboard.submit_daily {
                    let url = self.config.leaderboard.url.clone();
                    match submit_score(&url, &entry) {
                        Ok(rank) => {
                            self.daily_status = format!("Submitted! Rank #{}", rank);
                            self.achievements.check_event("daily_submitted", rank as i64);
                            if rank == 1 { self.achievements.check_event("daily_rank1", 1); }
                            if rank <= 3 { self.achievements.check_event("daily_top3", 1); }
                        }
                        Err(e) => self.daily_status = format!("Submit failed: {}", &e.chars().take(40).collect::<String>()),
                    }
                }
                self.daily_submitted = true;
                self.achievements.check_event("daily_first", 1);
            }
        }
    }
}

// ─── CONST LISTS ──────────────────────────────────────────────────────────────

const CLASSES: &[(&str, CharacterClass)] = &[
    ("Mage",         CharacterClass::Mage),
    ("Berserker",    CharacterClass::Berserker),
    ("Ranger",       CharacterClass::Ranger),
    ("Thief",        CharacterClass::Thief),
    ("Necromancer",  CharacterClass::Necromancer),
    ("Alchemist",    CharacterClass::Alchemist),
    ("Paladin",      CharacterClass::Paladin),
    ("VoidWalker",   CharacterClass::VoidWalker),
    ("Warlord",      CharacterClass::Warlord),
    ("Trickster",    CharacterClass::Trickster),
    ("Runesmith",    CharacterClass::Runesmith),
    ("Chronomancer", CharacterClass::Chronomancer),
];

const BACKGROUNDS: &[(&str, Background)] = &[
    ("Scholar",   Background::Scholar),
    ("Wanderer",  Background::Wanderer),
    ("Gladiator", Background::Gladiator),
    ("Outcast",   Background::Outcast),
    ("Merchant",  Background::Merchant),
    ("Cultist",   Background::Cultist),
    ("Exile",     Background::Exile),
    ("Oracle",    Background::Oracle),
];

const DIFFICULTIES: &[(&str, Difficulty)] = &[
    ("Easy",   Difficulty::Easy),
    ("Normal", Difficulty::Normal),
    ("Brutal", Difficulty::Brutal),
    ("Chaos",  Difficulty::Chaos),
];

// ─── GAME STATE IMPL ──────────────────────────────────────────────────────────

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        self.frame += 1;
        self.update_display_fractions();

        // ── Visual push: update all effect systems ────────────────────────────
        self.color_grade.update();
        self.tile_effects.update(self.frame);
        self.death_seq.update();

        // Combat animation sequencer
        let frame_now = self.frame;
        self.combat_anim.update(frame_now);
        for ep in std::mem::take(&mut self.combat_anim.pending_particles) {
            self.particles.push(Particle::burst(ep.x, ep.y, ep.vx, ep.vy, ep.ch, ep.col, ep.lifetime));
        }

        // Nemesis reveal update
        self.nemesis_reveal.update();

        // Room entry animation countdown
        if self.room_entry_timer > 0 { self.room_entry_timer -= 1; }

        // Drive ColorGrade from game state
        self.update_color_grade();

        // Weather: pick type from current floor/boss state
        let wt = WeatherType::for_floor(self.floor_num, self.is_boss_fight);
        self.weather.set_type(wt);
        self.weather.update(self.frame);

        // Screen-space lights: clear every frame, re-register from game state
        self.tile_effects.clear_lights();
        if let Some(ref p) = self.player {
            let php = p.current_hp as f32 / p.max_hp.max(1) as f32;
            // Low HP edge effect
            let lhp_target = if php < 0.25 {
                ((0.25 - php) / 0.25) * 0.85
            } else { 0.0 };
            self.tile_effects.set_low_hp(lhp_target);
            // Vignette on deep floors
            let vig_target = if self.floor_num >= 50 {
                ((self.floor_num - 50) as f32 / 50.0).min(0.4)
            } else { 0.0 };
            self.tile_effects.set_vignette(vig_target, (0.0, 0.0, 0.0));
        }

        if self.auto_mode {
            self.tick_auto_play(ctx);
        }

        // Kill-linger: hold combat screen after victory so effects can finish
        if self.kill_linger > 0 {
            self.kill_linger -= 1;
            self.draw_combat(ctx);
            if self.kill_linger == 0 {
                if let Some(next) = self.post_combat_screen.take() {
                    self.enemy = None; // safe to clear now that linger frames are done
                    self.screen = next;
                }
            }
            // Draw overlay after kill-linger combat frame
            let t = self.theme_graded();
            self.tile_effects.draw_overlay(ctx, t.bg);
            return;
        }

        // Death cinematic — intercept GameOver screen until cinematic finishes
        if self.screen == AppScreen::GameOver && !self.death_cinematic_done {
            if self.death_seq.active {
                // Draw chaos bg first
                self.chaos_bg(ctx);
                let t = self.theme_graded();
                self.death_seq.draw(ctx, t.bg);
                if self.death_seq.is_done() {
                    self.death_cinematic_done = true;
                    self.death_seq.active = false;
                }
                // Handle Enter to skip cinematic
                if let Some(key) = ctx.key {
                    if key == VirtualKeyCode::Return || key == VirtualKeyCode::Space {
                        self.death_cinematic_done = true;
                        self.death_seq.active = false;
                    }
                }
                self.tile_effects.draw_overlay(ctx, t.bg);
                return;
            }
        }

        match self.screen.clone() {
            AppScreen::Title            => self.draw_title(ctx),
            AppScreen::Tutorial         => self.draw_tutorial(ctx),
            AppScreen::ModeSelect       => self.draw_mode_select(ctx),
            AppScreen::CharacterCreation => self.draw_char_creation(ctx),
            AppScreen::BoonSelect       => self.draw_boon_select(ctx),
            AppScreen::FloorNav         => self.draw_floor_nav(ctx),
            AppScreen::RoomView         => self.draw_room_view(ctx),
            AppScreen::Combat           => self.draw_combat(ctx),
            AppScreen::Shop             => self.draw_shop(ctx),
            AppScreen::Crafting         => self.draw_crafting(ctx),
            AppScreen::CharacterSheet   => self.draw_character_sheet(ctx),
            AppScreen::BodyChart        => self.draw_body_chart(ctx),
            AppScreen::PassiveTree      => self.draw_passive_tree(ctx),
            AppScreen::GameOver         => self.draw_game_over(ctx),
            AppScreen::Victory          => self.draw_victory(ctx),
            AppScreen::Achievements     => self.draw_achievements(ctx),
            AppScreen::RunHistory       => self.draw_run_history(ctx),
            AppScreen::DailyLeaderboard => self.draw_daily_leaderboard(ctx),
            AppScreen::Scoreboard       => self.draw_scoreboard(ctx),
            AppScreen::Bestiary         => self.draw_bestiary(ctx),
            AppScreen::Codex            => self.draw_codex(ctx),
            AppScreen::Settings         => self.draw_settings(ctx),
        }

        // ── Visual push overlays (drawn over all UI, under banner) ──────────────
        {
            let t = self.theme_graded();
            // Weather (drawn after chaos bg but before UI — only in chaos_bg context;
            // here we just draw any frame-based weather overlay chars)
            self.weather.draw(ctx, t.bg);
            // TileEffects overlay: pulse rings, vignette, low HP edge
            self.tile_effects.draw_overlay(ctx, t.bg);
            // Storm flash
            if self.weather.storm_flash > 0 {
                let bright = (self.weather.storm_flash as f32 / 3.0 * 60.0) as u8;
                let fl = RGB::from_u8(bright, bright, bright);
                for y in [0i32, 79] {
                    for x in 0..160i32 {
                        ctx.print_color(x, y, fl, fl, " ");
                    }
                }
            }
        }

        // Achievement banner overlay — rich rarity-tiered system
        {
            let t = self.theme_graded();
            let frame = self.frame;
            self.rich_banner.update(frame);
            // Drain any unlock particles into the main particle system
            for bp in std::mem::take(&mut self.rich_banner.pending_particles) {
                self.particles.push(Particle::burst(bp.x, bp.y, bp.vx, bp.vy, bp.ch, bp.col, bp.lifetime));
            }
            self.rich_banner.draw(ctx, t.bg, frame);
        }
        // Legacy simple banner (still drives the old achievement_banner_frames countdown)
        if self.achievement_banner_frames > 0 {
            self.achievement_banner_frames -= 1;
            if self.achievement_banner_frames == 0 {
                self.achievement_banner = None;
            }
        }

        // ── Floor transition overlay ──────────────────────────────────────────
        if self.floor_transition_timer > 0 {
            self.floor_transition_timer -= 1;
            let t = self.theme_graded();
            let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
            // Fade: 0..30 = fade-in, 30..120 = hold, 120..150 = fade-out
            let elapsed = 150 - self.floor_transition_timer;
            let alpha = if elapsed < 30 {
                elapsed as f32 / 30.0
            } else if self.floor_transition_timer < 30 {
                self.floor_transition_timer as f32 / 30.0
            } else {
                1.0f32
            };
            let floor_str = format!("  FLOOR {}  ", self.floor_transition_floor);
            let flavor = floor_transition_flavor(self.floor_transition_floor);
            let bw = (floor_str.len().max(flavor.len() + 4) + 2) as i32;
            let bx = (160 - bw) / 2;
            let by = 32i32;
            let hd_col = RGB::from_u8(
                (t.heading.0 as f32 * alpha) as u8,
                (t.heading.1 as f32 * alpha) as u8,
                (t.heading.2 as f32 * alpha) as u8,
            );
            let ac_col = RGB::from_u8(
                (t.accent.0 as f32 * alpha) as u8,
                (t.accent.1 as f32 * alpha) as u8,
                (t.accent.2 as f32 * alpha) as u8,
            );
            let dim_col = RGB::from_u8(
                (t.dim.0 as f32 * alpha) as u8,
                (t.dim.1 as f32 * alpha) as u8,
                (t.dim.2 as f32 * alpha) as u8,
            );
            ctx.draw_box(bx, by, bw, 7, hd_col, bg);
            let fx = (160 - floor_str.len() as i32) / 2;
            ctx.print_color(fx, by + 2, hd_col, bg, &floor_str);
            let ffx = (160 - flavor.len() as i32) / 2;
            ctx.print_color(ffx, by + 4, ac_col, bg, flavor);
            // Particle burst at box corners during fade-in
            if elapsed < 30 && elapsed % 5 == 0 {
                let ex = bx as f32; let ey = by as f32;
                let bw2 = bw as f32;
                for &(px, py, vx, vy) in &[
                    (ex, ey, -0.15f32, -0.1f32), (ex + bw2, ey, 0.15, -0.1),
                    (ex, ey + 7.0, -0.15, 0.1), (ex + bw2, ey + 7.0, 0.15, 0.1),
                ] {
                    self.particles.push(Particle::burst(px, py, vx, vy, "✦",
                        (t.accent.0, t.accent.1, t.accent.2), 25));
                }
            }
        }

        // ── Boss entrance animation overlay ───────────────────────────────────
        if self.boss_entrance_timer > 0 {
            self.boss_entrance_timer -= 1;
            let t = self.theme_graded();
            let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
            let total = 180u32;
            let elapsed = total - self.boss_entrance_timer;
            // Phase 1 (0..60): blackout grows, name materializes char by char
            // Phase 2 (60..120): hold with particle burst
            // Phase 3 (120..180): fade out
            let alpha = if elapsed < 30 {
                elapsed as f32 / 30.0
            } else if self.boss_entrance_timer < 60 {
                (self.boss_entrance_timer as f32 / 60.0).powi(2)
            } else {
                1.0f32
            };
            let dng_col = RGB::from_u8(
                (t.danger.0 as f32 * alpha) as u8,
                (t.danger.1 as f32 * alpha) as u8,
                (t.danger.2 as f32 * alpha) as u8,
            );
            let hd_col = RGB::from_u8(
                (t.heading.0 as f32 * alpha) as u8,
                (t.heading.1 as f32 * alpha) as u8,
                (t.heading.2 as f32 * alpha) as u8,
            );
            // Dark vignette: overlay on top two rows and bottom two rows
            for x in 0..160i32 {
                ctx.set(x, 0, dng_col, bg, 219u16);
                ctx.set(x, 79, dng_col, bg, 219u16);
            }

            // Central boss announcement box
            let bname = &self.boss_entrance_name.clone();
            let star_str = format!("  ★  {}  ★  ", bname);
            let bw = (star_str.len() + 4) as i32;
            let bx = (160 - bw) / 2;
            let by = 29i32;
            ctx.draw_box(bx, by, bw, 5, dng_col, bg);
            ctx.print_color(bx + 2, by, hd_col, bg, "  BOSS ENCOUNTER  ");

            // Name materializes char-by-char during phase 1
            let chars_to_show = if elapsed < 60 {
                (elapsed as usize * star_str.len() / 60).min(star_str.len())
            } else {
                star_str.len()
            };
            let partial: String = star_str.chars().take(chars_to_show).collect();
            let nx = (160 - star_str.len() as i32) / 2;
            ctx.print_color(nx, by + 2, dng_col, bg, &partial);

            // Particle burst during hold phase
            let bid = self.boss_id.unwrap_or(0);
            emit_boss_entrance_burst(&mut self.particles, bid, elapsed as u64);

            // Chaos chars around the announcement during phase 2
            if elapsed >= 60 && elapsed < 120 {
                let chaos_chars = ["∞","∑","λ","∂","Ω","π"];
                for i in 0..8i32 {
                    let cx2 = bx - 2 + i * (bw / 7 + 1);
                    let cy_top = by - 1;
                    let cy_bot = by + 6;
                    let ch = chaos_chars[(elapsed as usize / 4 + i as usize) % chaos_chars.len()];
                    ctx.print_color(cx2.max(0).min(158), cy_top, dng_col, bg, ch);
                    ctx.print_color(cx2.max(0).min(158), cy_bot, dng_col, bg, ch);
                }
            }
        }

        self.handle_input(ctx);
    }
}

// ─── DRAW HELPERS ─────────────────────────────────────────────────────────────

use renderer::{
    draw_panel, draw_subpanel, draw_bar_gradient, draw_bar_solid,
    print_t, print_center, print_hint, draw_separator,
    print_selectable, draw_minimap_cell, stat_line,
    MinimapState, cursor_char,
};

fn room_col(rt: &RoomType, t: &theme::Theme) -> (u8,u8,u8) {
    match rt {
        RoomType::Combat        => t.danger,
        RoomType::Boss          => (min_u8(t.danger.0, 200), 0, 0),
        RoomType::Treasure      => t.gold,
        RoomType::Shop          => t.accent,
        RoomType::Shrine        => (180, 80, 220),
        RoomType::Trap          => t.warn,
        RoomType::Portal        => t.mana,
        RoomType::Empty         => t.muted,
        RoomType::ChaosRift     => t.xp,
        RoomType::CraftingBench => t.success,
    }
}

fn min_u8(a: u8, b: u8) -> u8 { if a < b { a } else { b } }

// ─── SCREENS ──────────────────────────────────────────────────────────────────

impl State {
    // ── TITLE ─────────────────────────────────────────────────────────────────

    fn draw_title(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let muted = RGB::from_u8(t.muted.0, t.muted.1,   t.muted.2);
        let danger = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "", &t);

        // ── Math symbol rain (background flavor) — full 160×80 coverage ──
        let math_chars = ["∫","∂","∑","∏","∇","λ","Ω","ε","δ","π","μ","ζ","⊕","∞","√","≈","≠","±","∧","∨"];
        for col_i in 0..40usize {
            let col_seed = col_i as u64 * 2654435761;
            let x = 2 + (col_seed % 154) as i32;
            let speed = 1 + (col_seed % 3) as u64;
            let offset = col_seed % 78;
            let y = ((self.frame / speed.max(1) + offset) % 76) as i32;
            if y < 2 || y > 77 { continue; }
            let sym_i = ((col_seed.wrapping_add(self.frame / 8)) % math_chars.len() as u64) as usize;
            let fade = (y as f32 / 76.0).clamp(0.1, 0.9);
            let rc = (t.muted.0 as f32 * (1.0 - fade) + t.dim.0 as f32 * fade * 0.4) as u8;
            let gc = (t.muted.1 as f32 * (1.0 - fade) + t.dim.1 as f32 * fade * 0.4) as u8;
            let bc = (t.muted.2 as f32 * (1.0 - fade) + t.dim.2 as f32 * fade * 0.4) as u8;
            ctx.print_color(x, y, RGB::from_u8(rc, gc, bc), bg, math_chars[sym_i]);
        }

        // ── First-load title logo particle convergence ────────────────────
        // On very first frames, particles converge from edges to form the logo
        if self.title_logo_timer > 0 {
            self.title_logo_timer -= 1;
            let elapsed = 90 - self.title_logo_timer;
            // Emit convergence particles streaming toward logo center (x=80,y=7)
            if elapsed < 60 {
                use std::f32::consts::TAU;
                let num = (elapsed as usize / 3 + 1).min(8);
                for i in 0..num {
                    let angle = (i as f32 * TAU / num as f32) + elapsed as f32 * 0.1;
                    let r = 50.0 - elapsed as f32 * 0.6;
                    let r = r.max(2.0);
                    let spd = 0.5 + i as f32 * 0.05;
                    let chaos_str = ["C","H","A","O","S","R","P","G"];
                    let col_pal = [
                        (t.heading.0, t.heading.1, t.heading.2),
                        (t.accent.0, t.accent.1, t.accent.2),
                        (t.gold.0, t.gold.1, t.gold.2),
                    ];
                    let col = col_pal[i % col_pal.len()];
                    self.particles.push(Particle::burst(
                        80.0 + angle.cos() * r, 7.0 + angle.sin() * r * 0.4,
                        -angle.cos() * spd * 0.08, -angle.sin() * spd * 0.05,
                        chaos_str[i % chaos_str.len()], col, 30));
                }
            }
            // Flash explosion burst when particles arrive
            if elapsed == 60 {
                use std::f32::consts::TAU;
                for i in 0..32usize {
                    let angle = i as f32 * TAU / 32.0;
                    let col = if i % 3 == 0 {
                        (t.heading.0, t.heading.1, t.heading.2)
                    } else {
                        (t.accent.0, t.accent.1, t.accent.2)
                    };
                    self.particles.push(Particle::spark(80.0, 7.0,
                        angle.cos() * 0.3, angle.sin() * 0.15,
                        ["✦","★","·","*"][i % 4], col));
                }
            }
        }

        // ── Animated banner pulse — centered in 160-col screen ────────────
        let pulse = ((self.frame as f32 * 0.04).sin() * 0.15 + 0.85) as f32;
        let ph = (t.heading.0 as f32 * pulse) as u8;
        let pg = (t.heading.1 as f32 * pulse) as u8;
        let pb = (t.heading.2 as f32 * pulse) as u8;
        let pulsed = RGB::from_u8(ph, pg, pb);

        // ASCII art is 43 chars wide — center at x=58 on 160-col screen
        let bx = 58i32;
        ctx.print_color(bx, 5,  pulsed, bg, " ██████╗██╗  ██╗ █████╗  ██████╗ ███████╗");
        ctx.print_color(bx, 6,  pulsed, bg, "██╔════╝██║  ██║██╔══██╗██╔═══██╗██╔════╝");
        ctx.print_color(bx, 7,  hd,     bg, "██║     ███████║███████║██║   ██║███████╗");
        ctx.print_color(bx, 8,  hd,     bg, "╚██████╗██║  ██║██║  ██║╚██████╔╝███████║");
        ctx.print_color(bx, 9,  hd,     bg, " ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝");

        ctx.print_color(bx + 4, 11, dim, bg, "R P G    ─    Where Math Goes To Die");

        draw_separator(ctx, 2, 13, 155, &t);

        // ── Chaos engine one-liner ─────────────────────────────────────────
        ctx.print_color(bx, 15, ac, bg,
            "Every action rolls a recursive chain of chaotic math modifiers.");
        ctx.print_color(bx, 16, dim, bg,
            "The deeper the chain, the wilder the output. Embrace the cascade.");

        // ── Continue notice (if save exists) ──────────────────────────────
        if self.save_exists {
            let flash = if (self.frame / 20) % 2 == 0 { ac } else { hd };
            ctx.print_color(bx, 18, flash, bg, "► SAVE DETECTED — press [L] to Continue");
        }

        // ── Grouped menu — three columns ──────────────────────────────────
        // Group 0: PLAY    (Continue, New Game)
        // Group 1: PROGRESS (Achievements, Bestiary, Codex, History, Daily, Scoreboard)
        // Group 2: SETTINGS (Options, Tutorial, Quit)

        // Build flat menu index: Continue (if save), New Game, Achievements,
        // Bestiary, Codex, History, Daily, Scoreboard, Options, Tutorial, Quit
        let mut opts: Vec<(&str, u8)> = Vec::new(); // (label, group)
        if self.save_exists { opts.push(("Continue", 0)); }
        opts.push(("New Game", 0));
        opts.push(("Achievements", 1));
        opts.push(("Bestiary", 1));
        opts.push(("Codex", 1));
        opts.push(("History", 1));
        opts.push(("Daily Seed", 1));
        opts.push(("Scoreboard", 1));
        opts.push(("Options", 2));
        opts.push(("Tutorial", 2));
        opts.push(("Quit", 2));

        let ox = 56i32; let oy = 26i32;
        // Group headers + items
        let groups: [(&str, i32); 3] = [
            ("── PLAY ──", ox),
            ("── PROGRESS ──", ox + 22),
            ("── SETTINGS ──", ox + 50),
        ];
        for (gi, (gname, gx)) in groups.iter().enumerate() {
            let fg = RGB::from_u8(t.muted.0, t.muted.1, t.muted.2);
            ctx.print_color(*gx, oy - 1, fg, bg, gname);
            let mut row = 0i32;
            for (flat_i, (label, group)) in opts.iter().enumerate() {
                if *group as usize != gi { continue; }
                print_selectable(ctx, *gx, oy + row, flat_i == self.selected_menu, label, self.frame, &t);
                row += 2;
            }
        }

        // ── Hint bar — spread across full width ───────────────────────────
        draw_separator(ctx, 2, 73, 155, &t);
        print_hint(ctx, 2,  74, "↑↓",   " Nav  ",    &t);
        print_hint(ctx, 14, 74, "Enter"," Select  ", &t);
        print_hint(ctx, 32, 74, "[T]",  " Theme  ",  &t);
        print_hint(ctx, 46, 74, "[?]",  " Tutorial",&t);
        print_hint(ctx, 60, 74, "[L]",  " Load save",&t);

        // ── Theme badge & tagline ──────────────────────────────────────────
        let tname = format!(" {} [T] ", t.name);
        ctx.print_color(158 - tname.len() as i32, 76, muted, bg, &tname);
        ctx.print_color(4, 76, muted, bg, &format!("\"{}\"", t.tagline));

        // ── Title screen particle render ───────────────────────────────────
        for p in &mut self.particles { p.step(); }
        self.particles.retain(|p| p.alive());
        if self.config.visuals.enable_particles {
            for p in &self.particles {
                let rc = p.render_col();
                let px = p.x as i32; let py = p.y as i32;
                if py < 2 || py > 78 || px < 1 || px > 158 { continue; }
                ctx.print_color(px, py, RGB::from_u8(rc.0, rc.1, rc.2), bg, &p.text);
            }
        }
    }

    // ── MODE SELECT ───────────────────────────────────────────────────────────

    fn draw_mode_select(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "SELECT MODE", &t);

        let modes = [
            ("Story Mode",    "10 floors. Narrative arc with a final boss.",    "★ Recommended for newcomers"),
            ("Infinite Mode", "Descend forever. Math gets worse every floor.",  "∞ Score for the global leaderboard"),
            ("Daily Seed",    "Same dungeon for everyone today.",               "◈ Resets at UTC midnight"),
        ];

        for (i, (name, desc, hint)) in modes.iter().enumerate() {
            let y = 10 + i as i32 * 10;
            let is_sel = i == self.mode_cursor;
            let bx = 5i32;
            if is_sel {
                draw_subpanel(ctx, bx - 2, y - 1, 72, 7, "", &t);
            }
            print_selectable(ctx, bx, y, is_sel, name, self.frame, &t);
            ctx.print_color(bx + 2, y + 2, dim, bg, desc);
            ctx.print_color(bx + 2, y + 4, if is_sel { ac } else { dim }, bg, hint);
        }

        draw_separator(ctx, 2, 45, 75, &t);
        print_hint(ctx, 4, 46, "↑↓", " Navigate   ", &t);
        print_hint(ctx, 22, 46, "Enter", " Select   ", &t);
        print_hint(ctx, 38, 46, "Esc", " Back", &t);
    }

    // ── CHAR CREATION ─────────────────────────────────────────────────────────

    fn draw_char_creation(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let sel = RGB::from_u8(t.selected.0,t.selected.1,t.selected.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let wrn = RGB::from_u8(t.warn.0,   t.warn.1,   t.warn.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "CHARACTER CREATION", &t);

        // ── Name entry ──
        let name_label = if self.cc_name_active {
            format!("NAME (Enter to confirm): {}▌", &self.cc_name)
        } else {
            let n = if self.cc_name.is_empty() { "Anonymous" } else { &self.cc_name };
            format!("NAME [N to edit]: {}", n)
        };
        let name_col = if self.cc_name_active { sel } else { hd };
        draw_subpanel(ctx, 2, 3, 75, 3, "", &t);
        ctx.print_color(4, 4, name_col, bg, &name_label.chars().take(72).collect::<String>());

        // ── Class column (scrollable — show up to 12 classes at 1 row each)
        draw_subpanel(ctx, 2, 7, 25, 30, "CLASS  ↑↓", &t);
        for (i, (name, _)) in CLASSES.iter().enumerate() {
            print_selectable(ctx, 4, 9 + i as i32 * 2, i == self.cc_class, name, self.frame, &t);
        }

        // Class passive description
        let class = &CLASSES[self.cc_class].1;
        draw_subpanel(ctx, 2, 40, 25, 7, "PASSIVE ABILITY", &t);
        ctx.print_color(4, 42, ac, bg, class.passive_name());
        let desc = class.passive_desc();
        let mut row = 43i32;
        let mut line = String::new();
        for w in desc.split_whitespace() {
            if line.len() + w.len() + 1 > 20 {
                ctx.print_color(4, row, dim, bg, &line);
                line = w.to_string(); row += 1;
            } else {
                if !line.is_empty() { line.push(' '); }
                line.push_str(w);
            }
        }
        if !line.is_empty() { ctx.print_color(4, row, dim, bg, &line); }

        // ── Background column
        draw_subpanel(ctx, 30, 7, 25, 18, "BACKGROUND  ←→", &t);
        for (i, (name, _)) in BACKGROUNDS.iter().enumerate() {
            print_selectable(ctx, 32, 9 + i as i32 * 2, i == self.cc_bg, name, self.frame, &t);
        }

        // ── Difficulty column
        draw_subpanel(ctx, 30, 27, 25, 12, "DIFFICULTY  Tab", &t);
        let diff_colors = [suc, hd, wrn, dng];
        for (i, (name, _)) in DIFFICULTIES.iter().enumerate() {
            let is_sel = i == self.cc_diff;
            let c = if is_sel { sel } else { diff_colors[i] };
            let pfx = if is_sel { format!("{} ", cursor_char(self.frame)) } else { "  ".to_string() };
            ctx.print_color(32, 29 + i as i32 * 2, c, bg, &format!("{}{}", pfx, name));
        }

        // ── Portrait column
        draw_subpanel(ctx, 57, 7, 21, 43, "PORTRAIT", &t);
        let portrait = class.ascii_art();
        for (i, l) in portrait.lines().enumerate() {
            let line: String = l.chars().take(18).collect();
            ctx.print_color(59, 9 + i as i32, ac, bg, &line);
        }
        // Class description (word-wrapped at 17 chars)
        draw_separator(ctx, 58, 13, 19, &t);
        let mut row = 14i32;
        let mut line = String::new();
        for w in class.description().split_whitespace() {
            if line.len() + w.len() + 1 > 17 {
                ctx.print_color(59, row, dim, bg, &line);
                line = w.to_string(); row += 1;
            } else {
                if !line.is_empty() { line.push(' '); }
                line.push_str(w);
            }
        }
        if !line.is_empty() { ctx.print_color(59, row, dim, bg, &line); row += 1; }
        // Passive ability
        row += 1;
        ctx.print_color(59, row, ac, bg, class.passive_name());
        row += 1;
        let mut pline = String::new();
        for w in class.passive_desc().split_whitespace() {
            if pline.len() + w.len() + 1 > 17 {
                ctx.print_color(59, row, dim, bg, &pline);
                pline = w.to_string(); row += 1;
            } else {
                if !pline.is_empty() { pline.push(' '); }
                pline.push_str(w);
            }
        }
        if !pline.is_empty() { ctx.print_color(59, row, dim, bg, &pline); }

        // ── Right panel: full descriptions ──
        draw_subpanel(ctx, 80, 3, 77, 68, "PREVIEW", &t);

        // Class description (word-wrapped at 72 chars)
        ctx.print_color(82, 5, hd, bg, &format!("CLASS: {}", class.name()));
        draw_separator(ctx, 81, 6, 75, &t);
        let mut rrow = 7i32;
        let mut rline = String::new();
        for w in class.description().split_whitespace() {
            if rline.len() + w.len() + 1 > 72 {
                ctx.print_color(82, rrow, dim, bg, &rline);
                rline = w.to_string(); rrow += 1;
            } else {
                if !rline.is_empty() { rline.push(' '); }
                rline.push_str(w);
            }
        }
        if !rline.is_empty() { ctx.print_color(82, rrow, dim, bg, &rline); rrow += 1; }

        // Passive ability
        rrow += 1;
        ctx.print_color(82, rrow, ac, bg, &format!("Passive: {}", class.passive_name()));
        rrow += 1;
        let mut pline2 = String::new();
        for w in class.passive_desc().split_whitespace() {
            if pline2.len() + w.len() + 1 > 72 {
                ctx.print_color(82, rrow, dim, bg, &pline2);
                pline2 = w.to_string(); rrow += 1;
            } else {
                if !pline2.is_empty() { pline2.push(' '); }
                pline2.push_str(w);
            }
        }
        if !pline2.is_empty() { ctx.print_color(82, rrow, dim, bg, &pline2); rrow += 2; }

        // Background description
        let bg_data = &BACKGROUNDS[self.cc_bg].1;
        ctx.print_color(82, rrow, hd, bg, &format!("BACKGROUND: {}", bg_data.name()));
        draw_separator(ctx, 81, rrow + 1, 75, &t);
        rrow += 2;
        let mut bline = String::new();
        for w in bg_data.description().split_whitespace() {
            if bline.len() + w.len() + 1 > 72 {
                ctx.print_color(82, rrow, dim, bg, &bline);
                bline = w.to_string(); rrow += 1;
            } else {
                if !bline.is_empty() { bline.push(' '); }
                bline.push_str(w);
            }
        }
        if !bline.is_empty() { ctx.print_color(82, rrow, dim, bg, &bline); rrow += 2; }

        // Difficulty description
        let diff_data = &DIFFICULTIES[self.cc_diff].1;
        let diff_col = diff_colors[self.cc_diff];
        ctx.print_color(82, rrow, diff_col, bg, &format!("DIFFICULTY: {}", diff_data.name()));
        draw_separator(ctx, 81, rrow + 1, 75, &t);
        rrow += 2;
        let mut dline = String::new();
        for w in diff_data.description().split_whitespace() {
            if dline.len() + w.len() + 1 > 72 {
                ctx.print_color(82, rrow, dim, bg, &dline);
                dline = w.to_string(); rrow += 1;
            } else {
                if !dline.is_empty() { dline.push(' '); }
                dline.push_str(w);
            }
        }
        if !dline.is_empty() { ctx.print_color(82, rrow, dim, bg, &dline); }

        draw_separator(ctx, 2, 74, 155, &t);
        print_hint(ctx, 4, 75, "↑↓", " Class   ", &t);
        print_hint(ctx, 18, 75, "←→", " Background   ", &t);
        print_hint(ctx, 36, 75, "Tab", " Difficulty   ", &t);
        print_hint(ctx, 54, 75, "Enter", " Confirm   ", &t);
        print_hint(ctx, 70, 75, "Esc", " Back", &t);
    }

    // ── BOON SELECT ───────────────────────────────────────────────────────────

    fn draw_boon_select(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "CHOOSE YOUR BOON", &t);

        ctx.print_color(5, 3, dim, bg, "A gift from the chaos engine. Only one. Choose wisely.");
        draw_separator(ctx, 2, 5, 155, &t);

        // Left: boon selection (cols 2-77)
        for (i, boon) in self.boon_options.iter().enumerate() {
            let y = 8 + i as i32 * 14;
            let is_sel = i == self.boon_cursor;
            if is_sel {
                draw_subpanel(ctx, 2, y - 1, 75, 12, "", &t);
            }
            let key = format!("[{}] ", i + 1);
            ctx.print_color(12, y, if is_sel { ac } else { dim }, bg, &key);
            print_selectable(ctx, 16, y, is_sel, boon.name(), self.frame, &t);
            // Word-wrap description at 55 chars
            let mut brow = y + 2;
            let mut bline = String::new();
            for w in boon.description().split_whitespace() {
                if bline.len() + w.len() + 1 > 55 {
                    ctx.print_color(16, brow, dim, bg, &bline);
                    bline = w.to_string(); brow += 1;
                } else {
                    if !bline.is_empty() { bline.push(' '); }
                    bline.push_str(w);
                }
            }
            if !bline.is_empty() { ctx.print_color(16, brow, dim, bg, &bline); }
        }

        // Right: selected boon detail panel (cols 80-157)
        draw_subpanel(ctx, 80, 6, 77, 60, "BOON DETAILS", &t);
        if let Some(boon) = self.boon_options.get(self.boon_cursor) {
            ctx.print_color(82, 8, ac, bg, boon.name());
            draw_separator(ctx, 81, 9, 75, &t);
            let mut drow = 10i32;
            let mut dline = String::new();
            for w in boon.description().split_whitespace() {
                if dline.len() + w.len() + 1 > 72 {
                    ctx.print_color(82, drow, hd, bg, &dline);
                    dline = w.to_string(); drow += 1;
                } else {
                    if !dline.is_empty() { dline.push(' '); }
                    dline.push_str(w);
                }
            }
            if !dline.is_empty() { ctx.print_color(82, drow, hd, bg, &dline); }
        }

        draw_separator(ctx, 2, 74, 155, &t);
        print_hint(ctx, 4, 75, "↑↓ / 1-3", " Select   ", &t);
        print_hint(ctx, 28, 75, "Enter", " Confirm   ", &t);
        print_hint(ctx, 44, 75, "Esc", " Back", &t);
    }

    // ── FLOOR NAV ─────────────────────────────────────────────────────────────
    // Map-first layout: map is the hero, status is a compact sidebar.
    // Three zones: header (row 0-2), map center (rows 3-60), footer (rows 61-79).

    fn draw_floor_nav(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let muted = RGB::from_u8(t.muted.0, t.muted.1, t.muted.2);

        let (pname, pclass, plv, pfloor, pkills, pgold, pxp, php, pmhp, pstatus,
             pcorruption, prwk, ptier, pmisery, punderdog, pdefiance) = match &self.player {
            Some(p) => {
                let tier = p.power_tier();
                (p.name.clone(), p.class.name(), p.level, p.floor,
                 p.kills, p.gold, p.xp, p.current_hp, p.max_hp,
                 p.status_badges_plain(), p.corruption, p.rooms_without_kill,
                 tier, p.misery.misery_index, p.underdog_multiplier(),
                 p.misery.defiance_rolls)
            }
            None => { self.screen = AppScreen::Title; return; }
        };

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "", &t);

        // ── Zone 1: Universal header (rows 0-2) ───────────────────────────────
        let floor_str = format!(" FLOOR {}  {}  Lv.{}  {} ", pfloor, pname, plv, pclass);
        ctx.print_color(2, 1, hd, bg, &floor_str);

        // Mode badge right-aligned (T4)
        let mode_str = match self.game_mode {
            GameMode::Story    => format!("STORY {}/{}", pfloor, 10),
            GameMode::Infinite => "∞ INFINITE".to_string(),
            GameMode::Daily    => "◈ DAILY".to_string(),
        };
        ctx.print_color(159 - mode_str.len() as i32 - 1, 1, ac, bg, &mode_str);

        // Auto-pilot badge (T1 — critical state)
        if self.auto_mode {
            let pulse = (self.frame / 15) % 2 == 0;
            let auto_c = if pulse { RGB::from_u8(80, 220, 80) } else { RGB::from_u8(40, 140, 40) };
            ctx.print_color(80, 1, auto_c, bg, "◆ AUTO [Z]stop");
        }
        draw_separator(ctx, 1, 2, 157, &t);

        // Cursed floor warning (T1 danger)
        if self.is_cursed_floor {
            let pulse = (self.frame / 20) % 2 == 0;
            let cc = if pulse { dng } else { RGB::from_u8(t.danger.0/2, 0, 0) };
            ctx.print_color(2, 2, cc, bg, "☠ CURSED FLOOR — ALL ENGINES INVERTED ☠");
        }

        // ── Zone 2a: Quick-status sidebar (left, cols 0-35, rows 3-55) ─────────
        draw_subpanel(ctx, 1, 3, 34, 52, "STATUS", &t);

        // Vitals — T2 active values
        let hp_pct = php as f32 / pmhp.max(1) as f32;
        let hp_c = t.hp_color(hp_pct);
        stat_line(ctx, 3, 5, "HP ", &format!("{}/{}", php, pmhp), hp_c, &t);
        draw_bar_gradient(ctx, 3, 6, 30, php, pmhp, hp_c, t.muted, &t);
        stat_line(ctx, 3, 8, "MP ", &format!("{}/{}", self.current_mana, self.max_mana()), t.mana, &t);
        draw_bar_solid(ctx, 3, 9, 30, self.current_mana, self.max_mana(), t.mana, &t);

        // T4 labels with T2 values
        stat_line(ctx, 3, 11, "Gold  ", &format!("{}g", pgold), t.gold, &t);
        stat_line(ctx, 3, 12, "Kills ", &format!("{}", pkills), t.success, &t);

        // Power tier (animated T1 for high tiers)
        {
            let tier_rgb = ptier.rgb();
            let tier_col = if ptier.has_effect() {
                use chaos_rpg_core::power_tier::TierEffect;
                match ptier.effect() {
                    TierEffect::Rainbow | TierEffect::RainbowFast => {
                        let speed = if matches!(ptier.effect(), TierEffect::RainbowFast) { 2 } else { 4 };
                        let pal = [(220u8,60u8,60u8),(220,180,40),(60,200,80),(80,200,220),(80,80,220),(180,60,200)];
                        pal[((self.frame / speed) as usize) % pal.len()]
                    }
                    TierEffect::Pulse => {
                        if (self.frame / 15) % 2 == 0 { tier_rgb } else { (tier_rgb.0/2, tier_rgb.1/2, tier_rgb.2/2) }
                    }
                    TierEffect::Flash => {
                        if (self.frame / 12) % 2 == 0 { tier_rgb } else { t.bg }
                    }
                    _ => tier_rgb,
                }
            } else { tier_rgb };
            let (power_label, power_value) = match &self.player {
                Some(p) => p.power_display(),
                None => ("POWER", ptier.name().to_string()),
            };
            stat_line(ctx, 3, 13, &format!("{}: ", power_label), &power_value, tier_col, &t);
        }

        draw_separator(ctx, 2, 14, 31, &t);

        // Equipped items (T3 secondary)
        if let Some(ref p) = self.player {
            use chaos_rpg_core::character::EquipSlot;
            ctx.print_color(3, 15, muted, bg, "EQUIPPED");
            let eq_list = [
                (EquipSlot::Weapon, "WPN"),
                (EquipSlot::Body,   "BOD"),
                (EquipSlot::Ring1,  "RNG"),
                (EquipSlot::Ring2,  "RN2"),
                (EquipSlot::Amulet, "AMU"),
            ];
            for (j, (slot, label)) in eq_list.iter().enumerate() {
                let (name_s, dur_pct) = if let Some(item) = p.equipped.get(*slot) {
                    let n: String = item.name.chars().take(20).collect();
                    let pct = item.durability as f32 / item.max_durability.max(1) as f32;
                    (n, Some(pct))
                } else {
                    ("empty".to_string(), None)
                };
                let col = if dur_pct.map(|p| p < 0.25).unwrap_or(false) { dng }
                else if dur_pct.map(|p| p < 0.50).unwrap_or(false) { RGB::from_u8(220, 160, 40) }
                else { dim };
                ctx.print_color(3, 16 + j as i32, col, bg,
                    &format!("[{}] {}", label, &name_s.chars().take(22).collect::<String>()));
            }
        }

        draw_separator(ctx, 2, 22, 31, &t);

        // Alerts (T1 danger)
        let mut alert_y = 23i32;
        if pmisery >= 100.0 {
            let pulse = (self.frame / 20) % 2 == 0;
            let mc = if pulse { RGB::from_u8(t.warn.0, t.warn.1, t.warn.2) }
                     else { RGB::from_u8(t.warn.0/2, t.warn.1/2, t.warn.2/2) };
            let msg = if pmisery >= 200.0 { "⚠ MISERY CRITICAL!" }
                      else { "☠ SPITE MODE ON" };
            ctx.print_color(3, alert_y, mc, bg, msg);
            alert_y += 1;
        }
        if pcorruption > 20 {
            ctx.print_color(3, alert_y, RGB::from_u8(t.warn.0, t.warn.1, t.warn.2), bg,
                &format!("✖ CORRUPT {}", pcorruption));
            alert_y += 1;
        }
        if pfloor >= 50 && prwk >= 3 {
            let rooms_left = 5u32.saturating_sub(prwk);
            ctx.print_color(3, alert_y, dng, bg,
                &format!("⚠ HUNGER {}/5", prwk));
            alert_y += 1;
        }
        if let Some(ref nem) = self.nemesis_record {
            ctx.print_color(3, alert_y, dng, bg,
                &format!("☠ NEM fl.{}", nem.floor_killed_at));
            alert_y += 1;
        }
        if punderdog > 1.01 {
            ctx.print_color(3, alert_y, gld, bg,
                &format!("↑ UNDERDG ×{:.1}", punderdog));
            alert_y += 1;
        }
        if !pstatus.is_empty() && alert_y < 53 {
            ctx.print_color(3, alert_y, RGB::from_u8(t.xp.0, t.xp.1, t.xp.2), bg,
                &pstatus.chars().take(30).collect::<String>());
        }

        // ── Zone 2b: FLOOR MAP — hero panel (center+right, cols 36-158) ───────
        draw_subpanel(ctx, 37, 3, 120, 52, "FLOOR MAP", &t);

        if let Some(ref floor) = self.floor {
            // Map cells: 20 rooms per row, each cell 5 chars wide
            let per_row = 20usize;
            for (i, room) in floor.rooms.iter().enumerate() {
                let rx = 39 + (i % per_row) as i32 * 6;
                let ry = 5 + (i / per_row) as i32 * 3;
                if ry > 52 { break; }
                let sym = room.room_type.icon();
                let rc = room_col(&room.room_type, &t);
                let mstate = if i == floor.current_room { MinimapState::Current }
                             else if i < floor.current_room { MinimapState::Visited }
                             else { MinimapState::Ahead };
                draw_minimap_cell(ctx, rx, ry, mstate, rc, sym, &t);
                // Current room: pulsing ▶ prefix (T1 selected)
                if i == floor.current_room {
                    let pulse = ((self.frame as f32 * 0.06).sin() * 0.4 + 0.6) as f32;
                    let pr = (rc.0 as f32 * pulse).min(255.0) as u8;
                    let pg = (rc.1 as f32 * pulse).min(255.0) as u8;
                    let pb = (rc.2 as f32 * pulse).min(255.0) as u8;
                    ctx.print_color(rx - 1, ry, RGB::from_u8(pr, pg, pb), bg, "▶");
                }
                // Next room: subtle dot (T4)
                if i == floor.current_room + 1 && i < floor.rooms.len() {
                    let path_col = RGB::from_u8(
                        t.dim.0.saturating_add(30), t.dim.1.saturating_add(30), t.dim.2.saturating_add(30));
                    ctx.print_color(rx - 1, ry, path_col, bg, "·");
                }
            }

            // Current room callout (T2 active)
            let current = floor.current();
            let rc = room_col(&current.room_type, &t);
            let room_prog = format!("Room {}/{}", floor.current_room + 1, floor.rooms.len());
            draw_separator(ctx, 38, 44, 118, &t);
            ctx.print_color(39, 45, RGB::from_u8(rc.0, rc.1, rc.2), bg,
                &format!("▶ {}  {}  — {}", current.room_type.icon(),
                    current.room_type.name(),
                    &current.description.chars().take(90).collect::<String>()));
            ctx.print_color(142, 45, dim, bg, &room_prog);

            // Room hint (T3 secondary)
            let hint = match current.room_type {
                RoomType::Combat     => "Prepare for battle.",
                RoomType::Boss       => "★ BOSS ROOM ★ — Great rewards await.",
                RoomType::Treasure   => "Free item — may be cursed.",
                RoomType::Shop       => "Spend gold on items, spells, or HP.",
                RoomType::Shrine     => "Stat boost + HP restore.",
                RoomType::Trap       => "Unavoidable hazard. Cunning reduces damage.",
                RoomType::Portal     => "Skip ahead floors. High risk.",
                RoomType::Empty      => "Silence. Restores a little HP.",
                RoomType::ChaosRift  => "Pure chaos — anything can happen.",
                RoomType::CraftingBench => "Reforge, augment, corrupt, repair items.",
            };
            ctx.print_color(39, 47, dim, bg, hint);

            // Room-type ambient particles
            let rt_id: u8 = match current.room_type {
                RoomType::Shrine        => 3,
                RoomType::ChaosRift     => 4,
                RoomType::Treasure      => 2,
                RoomType::Boss          => 5,
                RoomType::Combat        => 1,
                _                       => 0,
            };
            if rt_id > 0 {
                emit_room_ambient(&mut self.particles, self.frame,
                    self.floor_seed.wrapping_add(self.floor_num as u64 * 31), rt_id);
            }

            // All-clear / descend prompt (T1)
            if floor.rooms_remaining() == 0 {
                let pulse = (self.frame / 15) % 2 == 0;
                let dc = if pulse { gld } else { RGB::from_u8(t.gold.0/2+20, t.gold.1/2+20, 0) };
                ctx.print_color(39, 49, dc, bg, "▼  All rooms cleared — [D] Descend  ▼");
            }
        }

        // ── Particle render pass ──────────────────────────────────────────────
        for p in &mut self.particles { p.step(); }
        self.particles.retain(|p| p.alive());
        if self.config.visuals.enable_particles {
            for p in &self.particles {
                let rc = p.render_col();
                let px = p.x as i32; let py = p.y as i32;
                if py < 3 || py > 60 || px < 1 || px > 158 { continue; }
                ctx.print_color(px, py, RGB::from_u8(rc.0, rc.1, rc.2), bg, &p.text);
            }
        }

        // ── Zone 3: Universal footer (rows 61-79) ─────────────────────────────
        draw_separator(ctx, 1, 61, 157, &t);

        // Primary actions (T2 active)
        let (sp_col, sp_label) = if let Some(ref p) = self.player {
            if p.skill_points > 0 {
                let pulse = (self.frame / 12) % 2 == 0;
                let c = if pulse { gld } else { RGB::from_u8(t.gold.0/2+20, t.gold.1/2+20, 10) };
                (c, format!("[C] Sheet ★{}", p.skill_points))
            } else {
                (ac, "[C] Sheet".to_string())
            }
        } else { (ac, "[C] Sheet".to_string()) };

        let body_col = if let Some(ref p) = self.player {
            let worst = p.body.parts.values()
                .map(|s| s.current_hp as f32 / s.max_hp.max(1) as f32)
                .fold(1.0f32, f32::min);
            if worst < 0.3 { dng } else if worst < 0.6 { RGB::from_u8(200, 130, 40) } else { suc }
        } else { dim };

        // Row 1: primary nav (T2)
        print_hint(ctx, 2,  62, "[E]", " Enter  ", &t);
        print_hint(ctx, 16, 62, "[D]", " Descend  ", &t);
        ctx.print_color(34, 62, sp_col, bg, &sp_label);
        ctx.print_color(50, 62, body_col, bg, "[B] Body");
        print_hint(ctx, 64, 62, "[N]", " Passives  ", &t);
        print_hint(ctx, 80, 62, "[I]", " Inventory  ", &t);
        print_hint(ctx, 98, 62, "[Z]", " Auto  ", &t);
        print_hint(ctx, 110, 62, "[S]", " Scores  ", &t);
        print_hint(ctx, 124, 62, "[Q]", " Quit", &t);

        // Row 2: room type legend (T4 muted)
        ctx.print_color(2, 63, muted, bg,
            "[×]=Fight [★]=Loot [$]=Shop [~]=Shrine [!]=Trap [^]=Portal [⚒]=Craft");

        // Row 3: state alerts (T1)
        if self.auto_mode {
            ctx.print_color(2, 64, RGB::from_u8(80, 220, 80), bg,
                "◆ AUTO PILOT ACTIVE — pauses at item/shop/craft. [Z] to stop.");
        }

        // Nemesis reveal overlay — draws over floor nav when active
        if self.nemesis_reveal.active {
            let frame = self.frame;
            self.nemesis_reveal.draw(ctx, t.bg, frame);
        }
    }

    // ── ROOM VIEW ─────────────────────────────────────────────────────────────

    fn draw_room_view(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let sel = RGB::from_u8(t.selected.0,t.selected.1,t.selected.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);

        // Apply effects once
        if !self.room_event.resolved {
            self.room_event.resolved = true;
            let gd = self.room_event.gold_delta;
            let hd_val = self.room_event.hp_delta;
            let dt = self.room_event.damage_taken;
            let bonuses: Vec<(&'static str, i64)> = self.room_event.stat_bonuses.clone();
            // Shrine blessing pulse
            if !bonuses.is_empty() || hd_val > 0 {
                self.trigger_pulse_ring(80.0, 40.0, (0.4, 0.6, 1.0), 0.9);
            }
            if let Some(ref mut p) = self.player {
                if gd != 0 { p.gold += gd; }
                if hd_val > 0  { p.heal(hd_val); }
                if dt > 0  { p.take_damage(dt); }
                for (stat, val) in &bonuses { self.apply_stat_modifier(stat, *val); }
            }
            if dt > 0 {
                if let Some(ref p) = self.player {
                    if !p.is_alive() {
                        self.save_score_now();
                        self.screen = AppScreen::GameOver;
                        return;
                    }
                }
            }
        }

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "", &t);
        draw_subpanel(ctx, 2, 2, 155, 70, "", &t);

        let title = self.room_event.title.clone();
        print_center(ctx, 2, 4, 155, t.heading, &t, &title);
        draw_separator(ctx, 3, 5, 153, &t);

        for (i, line) in self.room_event.lines.iter().enumerate() {
            let fg = if line.starts_with('[') { sel }
                     else if line.starts_with('+') || line.starts_with("You find") { hd }
                     else { dim };
            ctx.print_color(5, 7 + i as i32, fg, bg, &line.chars().take(70).collect::<String>());
        }

        let has_item  = self.room_event.pending_item.is_some();
        let has_spell = self.room_event.pending_spell.is_some();
        let is_portal = self.room_event.portal_available;

        draw_separator(ctx, 3, 68, 153, &t);
        let ay = 70i32;
        if has_item  { print_hint(ctx, 8, ay, "[P]", " Pick up item   ", &t); print_hint(ctx, 40, ay, "[Enter]", " Leave it", &t); }
        if has_spell { print_hint(ctx, 8, ay+1, "[L]", " Learn spell   ", &t); print_hint(ctx, 40, ay+1, "[Enter]", " Leave scroll", &t); }
        if is_portal { print_hint(ctx, 8, ay, "[P]", " Step through portal   ", &t); print_hint(ctx, 50, ay, "[Enter]", " Resist", &t); }
        if !has_item && !has_spell && !is_portal {
            print_hint(ctx, 8, ay, "[Enter]", " Continue", &t);
        }

        // Room-type ambient particle effects
        let rt_id: u8 = {
            let title = &self.room_event.title;
            if title.contains("Shrine")   { 3 }
            else if title.contains("Rift") || title.contains("Chaos") { 4 }
            else if title.contains("Treasure") || title.contains("Chest") || title.contains("LOOT") { 2 }
            else if title.contains("Portal") { 3 }
            else { 0 }
        };
        if rt_id > 0 {
            emit_room_ambient(&mut self.particles, self.frame,
                self.floor_seed.wrapping_add(self.floor_num as u64 * 137), rt_id);
        }
        // Render particles (room view shares same particle system)
        let bg_rgb = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
        for p in &mut self.particles { p.step(); }
        self.particles.retain(|p| p.alive());
        if self.config.visuals.enable_particles {
            for p in &self.particles {
                let rc = p.render_col();
                let px = p.x as i32; let py = p.y as i32;
                if py < 3 || py > 67 || px < 3 || px > 156 { continue; }
                ctx.print_color(px, py, RGB::from_u8(rc.0, rc.1, rc.2), bg_rgb, &p.text);
            }
        }

        // Room entry flash overlay
        self.draw_room_entry_flash(ctx);
    }

    // ── COMBAT ────────────────────────────────────────────────────────────────

    fn draw_combat(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let mna = RGB::from_u8(t.mana.0,   t.mana.1,   t.mana.2);
        let xp  = RGB::from_u8(t.xp.0,     t.xp.1,     t.xp.2);

        let (pname, pclass, plv, php, pmhp, pstatus) = match &self.player {
            Some(p) => (p.name.clone(), p.class.name(), p.level, p.current_hp, p.max_hp, p.status_badges_plain()),
            None => { self.screen = AppScreen::Title; return; }
        };
        let (ename, etier, ehp, emhp, esprite) = match &self.enemy {
            Some(e) => (e.name.clone(), e.tier.name().to_string(), e.hp, e.max_hp, e.ascii_sprite),
            None => { self.screen = AppScreen::FloorNav; return; }
        };
        let floor_kills = self.player.as_ref().map(|p| (p.floor, p.kills)).unwrap_or((1, 0));

        self.chaos_bg(ctx);

        // ── Zone 1: Thin header row (row 0) — floor context label ──────────────
        let floor_label = if self.is_boss_fight {
            if self.gauntlet_stage > 0 {
                format!("COMBAT — GAUNTLET {}/3", self.gauntlet_stage)
            } else {
                format!("COMBAT — Floor {} — ★ BOSS", floor_kills.0)
            }
        } else {
            format!("COMBAT — Floor {}", floor_kills.0)
        };
        draw_panel(ctx, 0, 0, 159, 79, &floor_label, &t);

        // Inline vitals: HP and MP on header line (T2 — active)
        let pp_hdr = php as f32 / pmhp.max(1) as f32;
        let hp_bar_w = 24i32;
        let hp_filled = (pp_hdr * hp_bar_w as f32) as i32;
        let hp_col = t.hp_color(pp_hdr);
        let hp_bar: String = "█".repeat(hp_filled.max(0) as usize)
            + &"░".repeat((hp_bar_w - hp_filled).max(0) as usize);
        ctx.print_color(100, 0, RGB::from_u8(hp_col.0, hp_col.1, hp_col.2), bg,
            &format!("HP[{}]{}/{}", hp_bar, php, pmhp));
        let mp_pct_hdr = if self.max_mana() > 0 { self.current_mana as f32 / self.max_mana() as f32 } else { 1.0 };
        let mp_filled = (mp_pct_hdr * 12.0) as i32;
        let mp_bar: String = "█".repeat(mp_filled.max(0) as usize)
            + &"░".repeat((12 - mp_filled).max(0) as usize);
        ctx.print_color(100, 1, RGB::from_u8(t.mana.0, t.mana.1, t.mana.2), bg,
            &format!("MP[{}]{}/{}", mp_bar, self.current_mana, self.max_mana()));

        // ── Enemy panel (left half) ────────────────────────────────────────────
        draw_subpanel(ctx, 1, 2, 78, 36, "ENEMY", &t);
        let boss_lbl = if self.gauntlet_stage > 0 {
            format!(" GAUNTLET {}/3 ", self.gauntlet_stage)
        } else if self.is_boss_fight { " ★ BOSS ★ ".to_string() } else { String::new() };
        if !boss_lbl.is_empty() {
            ctx.print_color(40, 3, dng, bg, &boss_lbl);
        }
        let etier_s: String = etier.chars().take(20).collect();
        let ename_s: String = ename.chars().take(30).collect();
        ctx.print_color(3, 4, dng, bg, &format!("{} [{}]", ename_s, etier_s));
        let ep = ehp as f32 / emhp.max(1) as f32;
        let ec = t.hp_color(ep);
        stat_line(ctx, 3, 5, "HP ", &format!("{}/{}", ehp, emhp), ec, &t);
        // Ghost bar (shows previous HP level as a dim overhang)
        if self.config.visuals.enable_hp_ghost && self.ghost_enemy_timer > 0 {
            self.ghost_enemy_timer = self.ghost_enemy_timer.saturating_sub(1);
            let ghost_fill = (self.ghost_enemy_hp * 74.0) as i32;
            let disp_fill = (self.display_enemy_hp * 74.0) as i32;
            let ghost_col = RGB::from_u8(t.danger.0 / 3, t.danger.1 / 3, t.danger.2 / 3);
            for gx in disp_fill..ghost_fill {
                ctx.set(3 + gx, 6, ghost_col, RGB::from_u8(t.bg.0, t.bg.1, t.bg.2), 177u16);
            }
        }
        // Use smooth display fraction for bar (lerped toward actual HP)
        let disp_ehp = (self.display_enemy_hp * emhp as f32) as i64;
        draw_bar_gradient(ctx, 3, 6, 74, disp_ehp, emhp, ec, t.muted, &t);

        // Sprite — now 20 lines tall
        for (i, line) in esprite.lines().enumerate().take(20) {
            let s: String = line.chars().take(72).collect();
            ctx.print_color(3, 8 + i as i32, dim, bg, &s);
        }

        // ── Player panel (right half) ──────────────────────────────────────────
        draw_subpanel(ctx, 81, 2, 77, 36, "PLAYER", &t);
        let pname_s: String = pname.chars().take(14).collect();
        let pclass_s: String = pclass.chars().take(16).collect();
        ctx.print_color(83, 4, hd, bg, &format!("{} Lv.{}  {}", pname_s, plv, pclass_s));
        let pp = php as f32 / pmhp.max(1) as f32;
        let pc = t.hp_color(pp);
        stat_line(ctx, 83, 5, "HP ", &format!("{}/{}", php, pmhp), pc, &t);
        // Player ghost bar (overhang showing recent damage taken)
        if self.config.visuals.enable_hp_ghost && self.ghost_player_timer > 0 {
            self.ghost_player_timer = self.ghost_player_timer.saturating_sub(1);
            let ghost_fill = (self.ghost_player_hp * 73.0) as i32;
            let disp_fill = (self.display_player_hp * 73.0) as i32;
            let ghost_col = RGB::from_u8(t.danger.0 / 3, t.danger.1 / 3, t.danger.2 / 3);
            for gx in disp_fill..ghost_fill {
                ctx.set(83 + gx, 6, ghost_col, RGB::from_u8(t.bg.0, t.bg.1, t.bg.2), 177u16);
            }
        }
        // Smooth display HP bar
        let disp_php = (self.display_player_hp * pmhp as f32) as i64;
        draw_bar_gradient(ctx, 83, 6, 73, disp_php, pmhp, pc, t.muted, &t);
        // Smooth display MP bar
        let max_mp = self.max_mana();
        let disp_mp = (self.display_mp * max_mp as f32) as i64;
        stat_line(ctx, 83, 7, "MP ", &format!("{}/{}", self.current_mana, max_mp), t.mana, &t);
        draw_bar_solid(ctx, 83, 8, 73, disp_mp, max_mp, t.mana, &t);
        // ── Status effect icons with per-effect flicker ──────────────────────
        if let Some(ref p) = self.player {
            use chaos_rpg_core::character::StatusEffect;
            let mut sx = 83i32;
            for effect in &p.status_effects {
                let (icon, base_col): (&str, (u8,u8,u8)) = match effect {
                    StatusEffect::Burning(_)          => ("🔥", (255, 100,  20)),
                    StatusEffect::Poisoned(_)         => ("☠",  ( 50, 200,  50)),
                    StatusEffect::Stunned(_)          => ("⚡", (100, 200, 255)),
                    StatusEffect::Cursed(_)           => ("✖",  (180,  50, 180)),
                    StatusEffect::Blessed(_)          => ("✦",  (255, 220,  60)),
                    StatusEffect::Shielded(_)         => ("🛡", ( 60, 100, 220)),
                    StatusEffect::Enraged(_)          => ("⚔",  (220,  30,  30)),
                    StatusEffect::Frozen(_)           => ("❄",  (100, 180, 255)),
                    StatusEffect::Regenerating(_)     => ("+",  ( 50, 240, 100)),
                    StatusEffect::Phasing(_)          => ("◈",  (200,  80, 255)),
                    StatusEffect::Empowered(_)        => ("▲",  (255, 215,   0)),
                    StatusEffect::Fracture(_)         => ("⚙",  (180, 100,  40)),
                    StatusEffect::Resonance(_)        => ("~",  (255, 200,  80)),
                    StatusEffect::PhaseLock(_)        => ("⏸",  (220, 220, 220)),
                    StatusEffect::DimensionalBleed(_) => ("∞",  (140,  40, 200)),
                    StatusEffect::Recursive(_)        => ("↻",  (255,  80,  80)),
                    StatusEffect::Nullified(_)        => ("∅",  ( 80,  80,  80)),
                };
                let pulse = (self.frame / 8) % 2 == 0;
                let fc = if pulse { base_col } else {
                    (base_col.0 / 2, base_col.1 / 2, base_col.2 / 2)
                };
                ctx.print_color(sx, 10, RGB::from_u8(fc.0, fc.1, fc.2), bg, icon);
                sx += (icon.chars().count() as i32).max(1) + 1;
                if sx > 155 { break; }
            }
        }
        if self.is_cursed_floor {
            ctx.print_color(83, 11, dng, bg, "☠ CURSED FLOOR — all engines inverted");
        }

        // ── Status-effect ambient particles ───────────────────────────────────
        // Player status effects → right panel (x≈120, y≈18)
        if let Some(ref p) = self.player.clone() {
            use chaos_rpg_core::character::StatusEffect;
            let mut flags = 0u32;
            let mut has_stun = false;
            for effect in &p.status_effects {
                match effect {
                    StatusEffect::Burning(_)       => flags |= 1,
                    StatusEffect::Frozen(_)        => flags |= 2,
                    StatusEffect::Poisoned(_)      => flags |= 4,
                    StatusEffect::DimensionalBleed(_) => flags |= 8,
                    StatusEffect::Stunned(_)       => has_stun = true,
                    StatusEffect::Regenerating(_)  => flags |= 32,
                    _ => {}
                }
            }
            if flags != 0 {
                emit_status_ambient(&mut self.particles, 120.0, 18.0, self.frame, flags);
            }
            if has_stun {
                emit_stun_orbit(&mut self.particles, 120.0, 18.0, self.frame);
            }
        }
        // Enemy ambient: emit floor_ability-based particles in enemy panel
        if let Some(ref e) = self.enemy {
            use chaos_rpg_core::enemy::FloorAbility;
            let eff_col = match e.floor_ability {
                FloorAbility::EngineTheft  => Some((180u8, 60u8, 255u8)),
                FloorAbility::NullifyAura  => Some((60u8, 60u8, 60u8)),
                FloorAbility::StatMirror   => Some((160u8, 160u8, 255u8)),
                FloorAbility::None         => None,
            };
            if let Some(col) = eff_col {
                let frame = self.frame;
                if frame % 6 == 0 {
                    self.particles.push(Particle::burst(
                        40.0 + (frame % 5) as f32 - 2.0, 12.0,
                        0.0, -0.07, "·", col, 20));
                }
            }
        }
        // Room-type ambient particles
        {
            let rt_id: u8 = if self.is_boss_fight { 5 }
                else if self.gauntlet_stage > 0 { 5 }
                else { 1 }; // normal combat
            emit_room_ambient(&mut self.particles, self.frame,
                self.floor_seed.wrapping_add(self.floor_num as u64), rt_id);
        }

        // Spells — now shows all 8 with full names
        if let Some(ref p) = self.player {
            if !p.known_spells.is_empty() {
                ctx.print_color(83, 13, ac, bg, "SPELLS  [1-8]");
                for (i, spell) in p.known_spells.iter().enumerate().take(8) {
                    let can = self.current_mana >= spell.mana_cost;
                    let fg = if can { mna } else { dim };
                    ctx.print_color(83, 14 + i as i32, fg, bg,
                        &format!("[{}] {:<20} {:>3}mp  ×{:.1}",
                            i+1,
                            &spell.name.chars().take(20).collect::<String>(),
                            spell.mana_cost,
                            spell.scaling_factor.abs()));
                }
            }
        }

        // ── Actions bar (full width) ───────────────────────────────────────────
        draw_subpanel(ctx, 1, 40, 157, 11, "ACTIONS", &t);
        let ay = 42i32;
        let actions: &[(&str, &str, &str)] = &[
            ("[A]", "Attack",      "normal hit"),
            ("[H]", "Heavy",       "1.5x damage, -accuracy"),
            ("[D]", "Defend",      "+40 block this round"),
            ("[T]", "Taunt",       "lure + debuff"),
            ("[F]", "Flee",        "attempt escape"),
        ];
        let col_w = 26i32;
        for (i, (key, label, hint)) in actions.iter().enumerate() {
            let x = 3 + i as i32 * col_w;
            ctx.print_color(x, ay,     RGB::from_u8(t.accent.0, t.accent.1, t.accent.2),  bg, key);
            ctx.print_color(x + key.len() as i32, ay, RGB::from_u8(t.selected.0, t.selected.1, t.selected.2), bg, &format!(" {}", label));
            ctx.print_color(x, ay + 1, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2),    bg, hint);
        }
        print_hint(ctx, 3 + 5 * col_w, ay, "[1-8]", " Cast Spell", &t);
        ctx.print_color(3 + 5 * col_w, ay + 1, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg, "uses mana");

        // Items row
        if let Some(ref p) = self.player {
            if !p.inventory.is_empty() {
                let keys = ["Q","W","E","R","Y","U","I","O"];
                let mut ix = 3i32;
                ctx.print_color(ix, ay + 3, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg, "Items:");
                ix += 8;
                for (i, item) in p.inventory.iter().enumerate().take(8) {
                    if ix > 153 { break; }
                    let is_eq = item.equip_slot().is_some();
                    let name_s: String = item.name.chars().take(12).collect();
                    let label = format!("[{}]{}{} ", keys[i], if is_eq {"⚙"} else {" "}, name_s);
                    let item_col = if is_eq { RGB::from_u8(60, 200, 60) } else { dim };
                    ctx.print_color(ix, ay + 3, item_col, bg, &label);
                    ix += label.chars().count() as i32;
                }
            }
        }

        // ── Equipment strip (ay+5) — full width, wider name + longer bars ──────
        if let Some(ref p) = self.player {
            use chaos_rpg_core::character::EquipSlot;
            let eq_slots = [
                (EquipSlot::Weapon, "WPN"),
                (EquipSlot::Body,   "BOD"),
                (EquipSlot::Ring1,  "RNG"),
                (EquipSlot::Ring2,  "RN2"),
                (EquipSlot::Amulet, "AMU"),
            ];
            let mut ex = 3i32;
            for (slot, label) in &eq_slots {
                let (name_s, dur_pct) = if let Some(item) = p.equipped.get(*slot) {
                    let n: String = item.name.chars().take(14).collect();
                    let pct = item.durability as f32 / item.max_durability.max(1) as f32;
                    (n, pct)
                } else {
                    ("--------------".to_string(), 1.0)
                };
                let dur_col = if dur_pct > 0.75 { RGB::from_u8(60, 220, 60) }
                    else if dur_pct > 0.50 { RGB::from_u8(220, 200, 40) }
                    else if dur_pct > 0.25 { RGB::from_u8(255, 130, 30) }
                    else { RGB::from_u8(220, 30, 30) };
                let filled = (dur_pct * 8.0) as usize;
                let bar_s = format!("[{}{}]", "█".repeat(filled), "░".repeat(8 - filled));
                ctx.print_color(ex, ay + 5, dim, bg, &format!("[{}]", label));
                ex += 6;
                ctx.print_color(ex, ay + 5, dur_col, bg, &bar_s);
                ex += 11;
                ctx.print_color(ex, ay + 5, dim, bg, &format!("{:<16}", name_s));
                ex += 17;
                if ex > 153 { break; }
            }
        }

        // ── Zone 3: Chaos log — collapsible with [Tab] ───────────────────────
        let log_title = if self.combat_log_collapsed {
            "CHAOS LOG  [Tab] expand ▸"
        } else {
            "CHAOS LOG  [Tab] collapse ▾"
        };
        draw_subpanel(ctx, 1, 53, 157, 25, log_title, &t);

        // Single-line action result summary at zone 2/3 boundary (T1 if crit/catastrophe, T2 otherwise)
        if let Some(ref roll) = self.last_roll {
            let result_label = if roll.is_critical()    { "✦ CRITICAL" }
                               else if roll.final_value > 0.0 { "✓ SUCCESS" }
                               else if roll.is_catastrophe()  { "☠ CATASTROPHE" }
                               else { "✗ FAILURE" };
            let result_col = if roll.is_critical()      { gld }
                             else if roll.final_value > 0.0  { suc }
                             else if roll.is_catastrophe()   { RGB::from_u8(255, 0, 100) }
                             else { dng };
            // Chain string — compressed to single line
            let chain_str: String = roll.chain.iter()
                .map(|s| format!("{}({:+.1})", &s.engine_name.chars().take(6).collect::<String>(), s.output))
                .collect::<Vec<_>>().join("→");
            let bar_filled = ((roll.final_value + 1.0) / 2.0 * 40.0).round() as usize;
            let bar: String = "█".repeat(bar_filled.min(40)) + &"░".repeat(40usize.saturating_sub(bar_filled));
            ctx.print_color(3, 54, dim, bg,
                &format!("[{}] {:+.3}", bar, roll.final_value));
            ctx.print_color(50, 54, result_col, bg, result_label);
            if !self.combat_log_collapsed {
                ctx.print_color(80, 54, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg,
                    &format!("chain:{}", chain_str.chars().take(74).collect::<String>()));
            }
        } else {
            ctx.print_color(3, 54, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg,
                "Awaiting first action…");
        }
        draw_separator(ctx, 2, 55, 155, &t);

        if self.combat_log_collapsed {
            // Collapsed: show only the last 2 log lines (T3 color)
            let log_start = self.combat_log.len().saturating_sub(2);
            for (i, line) in self.combat_log[log_start..].iter().enumerate() {
                ctx.print_color(3, 56 + i as i32,
                    RGB::from_u8(t.dim.0, t.dim.1, t.dim.2), bg,
                    &line.chars().take(154).collect::<String>());
            }
        } else {
            // Expanded: 20 log lines (y=56..75)
            let log_start = self.combat_log.len().saturating_sub(20);
            for (i, line) in self.combat_log[log_start..].iter().enumerate() {
                if i >= 20 { break; }
                // T1 for critical events, T2 for positive, T3 for normal
                let fg = if line.contains("SHATTERED") || line.contains("lost forever") || line.contains("CATASTROPHE") {
                             RGB::from_u8(255, 30, 30)
                         } else if line.contains("Equipped") || line.contains("durability") {
                             RGB::from_u8(60, 200, 220)
                         } else if line.contains("CRIT") || line.contains("BOSS") || line.contains("☠") { dng }
                         else if line.contains("Victory") || line.contains("LEVEL") { gld }
                         else if line.contains("heal") || line.contains('+') { suc }
                         else { RGB::from_u8(t.primary.0, t.primary.1, t.primary.2) };
                ctx.print_color(3, 56 + i as i32, fg, bg, &line.chars().take(154).collect::<String>());
            }
        }

        // ── Visual effects (drawn on top of panels) ───────────────────────────

        // 1. Enemy panel hit flash — redraw border
        if self.enemy_flash > 0 {
            self.enemy_flash -= 1;
            let t_scale = self.enemy_flash as f32 / vc::flash_crit() as f32;
            let ec = self.enemy_flash_col;
            let r = (ec.0 as f32 * t_scale + 40.0 * (1.0 - t_scale)) as u8;
            let g = (ec.1 as f32 * t_scale + 40.0 * (1.0 - t_scale)) as u8;
            let b = (ec.2 as f32 * t_scale + 40.0 * (1.0 - t_scale)) as u8;
            ctx.draw_box(1, 2, 78, 36, RGB::from_u8(r, g, b), bg);
        }

        // 2. Player panel hit flash — red border
        if self.player_flash > 0 {
            self.player_flash -= 1;
            let intensity = (self.player_flash * 30 + 60) as u8;
            ctx.draw_box(81, 2, 77, 36, RGB::from_u8(intensity, 10, 10), bg);
        }

        // 3. Screen shake on big crits — outer border flash
        if self.hit_shake > 0 {
            self.hit_shake -= 1;
            let pulse = (self.hit_shake % 2 == 0) as u8;
            let intensity = 120 + pulse * 80;
            ctx.draw_box(0, 0, 159, 79, RGB::from_u8(intensity, intensity / 4, 0), bg);
        }

        // 4. Spell beam — charge then fire across the centre gap (y=40)
        if self.spell_beam > 0 {
            self.spell_beam -= 1;
            let bc = self.spell_beam_col;
            let total = vc::beam_charge() + vc::beam_hold();
            let elapsed = total - self.spell_beam;
            if elapsed < vc::beam_charge() {
                let filled = (elapsed as i32 * 155 / vc::beam_charge() as i32).min(155);
                let charge_col = RGB::from_u8(
                    (bc.0 as u32 * elapsed as u32 / vc::beam_charge() as u32) as u8,
                    (bc.1 as u32 * elapsed as u32 / vc::beam_charge() as u32) as u8,
                    (bc.2 as u32 * elapsed as u32 / vc::beam_charge() as u32) as u8,
                );
                for bx in 2..(2 + filled) {
                    ctx.print_color(bx, 40, charge_col, bg, "·");
                }
            } else {
                let bc_rgb = RGB::from_u8(bc.0, bc.1, bc.2);
                let beam_chars = ["~","≈","∿","~","≋","~"];
                let beam_offset = (self.frame / 2) as usize;
                for bx in 2..157i32 {
                    let c = beam_chars[(bx as usize + beam_offset) % beam_chars.len()];
                    ctx.print_color(bx, 40, bc_rgb, bg, c);
                }
                ctx.print_color(79, 40, RGB::from_u8(255, 255, 200), bg, "✦");
            }
        }

        // 5. Floating particles — step, cull, render
        for p in &mut self.particles { p.step(); }
        self.particles.retain(|p| p.alive());
        let max_p = self.config.visuals.max_particles as usize;
        if self.particles.len() > max_p {
            self.particles.drain(0..self.particles.len() - max_p);
        }
        if self.config.visuals.enable_particles {
            for p in &self.particles {
                let rc = p.render_col();
                let px = p.x as i32;
                let py = p.y as i32;
                if py < 1 || py > 78 || px < 1 || px > 158 { continue; }
                ctx.print_color(px, py, RGB::from_u8(rc.0, rc.1, rc.2), bg, &p.text);
            }
        }

        // 6. Combat animation overlay (slash trails, telegraph, spell paths, etc.)
        {
            let frame = self.frame;
            let bg_rgb = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
            self.combat_anim.draw(ctx, bg_rgb, frame);
        }

        // 6b. Boss-specific visual overlay
        if self.boss_id.is_some() {
            self.draw_boss_visual_overlay(ctx, bg);
        }

        // 7. Chaos Engine Visualization overlay ([V])
        if self.chaos_viz_open {
            self.draw_chaos_viz_overlay(ctx);
        }

        // Footer hints (T4 muted — labels)
        ctx.print_color(3, 78, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg,
            if self.chaos_viz_open { "[V] Close Engine Viz" } else { "[V] Engine Viz" });
        ctx.print_color(30, 78, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg,
            if self.combat_log_collapsed { "[Tab] Expand Log" } else { "[Tab] Collapse Log" });
    }

    fn draw_boss_visual_overlay(&mut self, ctx: &mut BTerm, bg: RGB) {
        let Some(bid) = self.boss_id else { return; };
        let t = self.theme_graded();
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let dng = RGB::from_u8(t.danger.0,  t.danger.1,  t.danger.2);
        let suc = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let gld = RGB::from_u8(t.gold.0,    t.gold.1,    t.gold.2);
        let muted = RGB::from_u8(t.muted.0, t.muted.1,   t.muted.2);

        match bid {
            // Boss 1 — THE MIRROR: symmetry indicator + reflected stat bars
            1 => {
                // Draw a vertical split-line at center with "mirror" label
                let pulse = (self.frame / 10) % 2 == 0;
                let mirror_col = if pulse { RGB::from_u8(200, 200, 255) } else { RGB::from_u8(100, 100, 180) };
                ctx.print_color(78, 3, mirror_col, bg, "◈ MIRROR ◈");
                ctx.print_color(78, 4, muted, bg, "Reflect");
                for y in 5..37i32 {
                    let ch = if y % 3 == 0 { "║" } else { "│" };
                    ctx.print_color(79, y, mirror_col, bg, ch);
                }
                // "Same HP as you" — warn bar
                let flip = (self.frame / 20) % 2 == 0;
                if flip {
                    ctx.print_color(3, 37, RGB::from_u8(160, 160, 255), bg,
                        "[ MIRROR: reflects your own power — find the asymmetry ]");
                }
            }

            // Boss 2 — THE ACCOUNTANT: live ledger HUD
            2 => {
                let lifetime_dmg = self.player.as_ref().map(|p| p.total_damage_dealt).unwrap_or(0);
                let fight_dmg = self.boss_extra;
                let defends = self.boss_extra2;
                let reduction = (defends * 20).min(80);
                let bill_est = ((lifetime_dmg + fight_dmg) as f64 * (1.0 - reduction as f64 / 100.0)) as i64;
                ctx.print_color(3, 37, gld, bg, &format!(
                    "LEDGER: fight={} lifetime={} defends={}×20%={reduction}% off → BILL≈{}",
                    fight_dmg, lifetime_dmg, defends, bill_est).chars().take(155).collect::<String>());
                // Turns remaining bar
                let turns_left = 5i32 - self.boss_turn as i32;
                let tl = turns_left.max(0);
                for i in 0..5i32 {
                    let col = if i < tl { gld } else { dng };
                    ctx.print_color(3 + i * 15, 38, col, bg,
                        if i < tl { "[TURN]" } else { "[BILL!]" });
                }
            }

            // Boss 3 — FIBONACCI HYDRA: split counter + sequence display
            3 => {
                let splits = self.boss_extra as usize;
                let fib_seq = [1u64, 1, 2, 3, 5, 8, 13];
                let current_hp_mult = fib_seq.get(splits).copied().unwrap_or(1);
                ctx.print_color(3, 37, gld, bg, &format!(
                    "HYDRA SPLITS: {}/10  Next split adds {} heads  (Fib: 1,1,2,3,5,8,13…)",
                    splits, current_hp_mult));
                // Growing sequence bar
                for i in 0..splits.min(10) {
                    let col = if i < 5 { gld } else { dng };
                    ctx.print_color(3 + i as i32 * 7, 38, col, bg, &format!("[×{}]",
                        fib_seq.get(i).copied().unwrap_or(1)));
                }
                // Flash on split (every other frame when splits > 0)
                if splits > 0 && (self.frame / 8) % 2 == 0 {
                    ctx.print_color(58, 37, RGB::from_u8(255, 200, 30), bg, "⟶ SPLIT ⟶");
                }
            }

            // Boss 5 — THE TAXMAN: tax bracket + HP drain indicator
            5 => {
                let ehp = self.enemy.as_ref().map(|e| e.hp).unwrap_or(0);
                let tax = ((ehp as f64 * 0.01) as i64).max(1);
                let pulse = (self.frame / 6) % 2 == 0;
                let tax_col = if pulse { gld } else { RGB::from_u8(200, 180, 20) };
                ctx.print_color(3, 37, tax_col, bg, &format!(
                    "TAXMAN: 1% HP drain/turn = {} dmg  (Turn {}) — [D] Defend halves it",
                    tax, self.boss_turn));
                // Gold drain particle: stream left-to-right on row 38
                let stream_x = ((self.frame * 3) % 150) as i32 + 3;
                let stream_col = RGB::from_u8(220, 180, 30);
                ctx.print_color(stream_x, 38, stream_col, bg, "¢");
                ctx.print_color((stream_x + 20).min(155), 38, stream_col, bg, "¢");
                ctx.print_color((stream_x + 40).min(155), 38, stream_col, bg, "¢");
            }

            // Boss 7 — OUROBOROS: circular ring phase indicator
            7 => {
                let boss_hp = self.enemy.as_ref().map(|e| e.hp).unwrap_or(0);
                let max_hp  = self.boss_extra;
                let cycle_turn = self.boss_turn % 3;
                let turns_to_reset = 3 - cycle_turn;
                ctx.print_color(3, 37, RGB::from_u8(100, 220, 100), bg, &format!(
                    "OUROBOROS: Heals to full every 3 turns — {}/3 until reset  HP: {}",
                    cycle_turn, boss_hp));
                // Serpent ring visual: circular arc fills as turn approaches
                use std::f32::consts::TAU;
                let cx = 72i32; let cy = 38i32;
                let r = 4;
                let filled_angle = cycle_turn as f32 / 3.0 * TAU;
                for seg in 0..24usize {
                    let angle = seg as f32 * TAU / 24.0;
                    let on = angle <= filled_angle;
                    let col = if on { RGB::from_u8(80, 200, 80) } else { muted };
                    let sx = cx + (angle.cos() * r as f32 * 2.0) as i32;
                    let sy = cy + (angle.sin() * r as f32) as i32;
                    if sx >= 0 && sx < 160 && sy >= 0 && sy < 79 {
                        ctx.print_color(sx, sy, col, bg, if on { "●" } else { "○" });
                    }
                }
            }

            // Boss 8 — COLLATZ TITAN: sequence display + next value
            8 => {
                let n = self.boss_extra;
                let next = if n % 2 == 0 { n / 2 } else { n * 3 + 1 };
                let next2 = if next % 2 == 0 { next / 2 } else { next * 3 + 1 };
                let at_min = n <= 4;
                let seq_col = if at_min { dng } else if n < 20 { gld } else { muted };
                ctx.print_color(3, 37, seq_col, bg, &format!(
                    "COLLATZ: HP={n}  → {next}  → {next2}   {}",
                    if at_min { "★ ATTACK NOW — at minimum!" }
                    else if n % 2 == 0 { "(even: halving next)" }
                    else { "(odd: tripling next!)" }));
                // Countdown bar: flashes at 1/2/4
                let bar_n = (n as f32 / 100.0).clamp(0.0, 1.0);
                let bar_filled = (bar_n * 74.0) as i32;
                for x in 0..74i32 {
                    let c = if x < bar_filled { seq_col } else { muted };
                    ctx.set(3 + x, 38, c, bg, if x < bar_filled { 219u16 } else { 176u16 });
                }
                // Flash warning on odd turns (about to triple)
                if n % 2 != 0 && (self.frame / 8) % 2 == 0 {
                    ctx.print_color(78, 37, dng, bg, "  ▲ TRIPLE ▲");
                }
            }

            // Boss 4 — THE EIGENSTATE: flicker between 1HP and 10000HP visual
            4 if self.config.visuals.enable_eigenstate_flicker => {
                let flicker = (self.frame / 3) % 2 == 0;
                let label = if flicker { " [1 HP] " } else { " [10,000 HP] " };
                let col = if flicker { suc } else { dng };
                ctx.print_color(20, 3, col, bg, label);
                // Static-noise flicker around enemy name
                if (self.frame / 2) % 3 != 0 {
                    let noise_chars = ["▒","░","▓","?","!"];
                    for i in 0..5i32 {
                        let nc = noise_chars[(self.frame as usize / 2 + i as usize) % noise_chars.len()];
                        ctx.print_color(3 + i * 2, 4, muted, bg, nc);
                    }
                }
            }

            // Boss 6 — THE NULL: announce pipeline nullified + mono bar indicator
            6 => {
                let null_col = RGB::from_u8(60, 60, 60);
                ctx.print_color(3, 37, null_col, bg, "[ CHAOS PIPELINE: NULLIFIED — BASE STATS ONLY ]");
                // Draining bar that shrinks each turn
                let turns = self.boss_turn.min(40) as i32;
                let drain_width = 74 - turns;
                for x in 0..drain_width {
                    ctx.set(3 + x, 38, null_col, bg, 176u16);
                }
            }

            // Boss 9 — THE COMMITTEE: vote indicators above enemy panel
            9 => {
                let vote_mask = self.boss_extra as u8;  // bitmask of secured votes
                let vote_labels = ["HP%", "ROLL", "GOLD", "KILLS", "TAUNT"];
                let vote_x = [3i32, 16, 30, 43, 57];
                ctx.print_color(3, 3, ac, bg, "COMMITTEE VOTES:");
                for (i, (label, &vx)) in vote_labels.iter().zip(vote_x.iter()).enumerate() {
                    let secured = (vote_mask >> i) & 1 == 1;
                    let col = if secured { suc } else { muted };
                    let sym = if secured { "[Y]" } else { "[ ]" };
                    ctx.print_color(vx, 4, col, bg, &format!("{} {}", sym, label));
                }
                let secured_count = vote_mask.count_ones();
                let tally_col = if secured_count >= 3 { suc } else { dng };
                ctx.print_color(72, 4, tally_col, bg, &format!("{}/5", secured_count));
            }

            // Boss 10 — THE RECURSION: stack bar showing accumulated damage
            10 => {
                let stack_dmg = self.boss_extra;
                let bar_label = format!("STACK: {} dmg (will reflect on attack)", stack_dmg);
                let bar_frac = (stack_dmg as f32 / 10000.0).clamp(0.0, 1.0);
                let bar_filled = (bar_frac * 72.0) as i32;
                let stack_col = if bar_frac > 0.75 { dng }
                    else if bar_frac > 0.4 { RGB::from_u8(255, 150, 50) }
                    else { gld };
                ctx.print_color(3, 37, stack_col, bg, &bar_label.chars().take(70).collect::<String>());
                for x in 0..bar_filled {
                    ctx.set(3 + x, 38, stack_col, bg, 219u16);
                }
                for x in bar_filled..72i32 {
                    ctx.set(3 + x, 38, muted, bg, 176u16);
                }
            }

            // Boss 11 — THE PARADOX: visual inversion cue
            11 if self.config.visuals.invert_screen_for_paradox => {
                // Draw inverted-HP note on player panel
                let note_col = RGB::from_u8(200, 60, 200);
                ctx.print_color(83, 38, note_col, bg, "[ PARADOX: lower defense = less damage ]");
            }

            // Boss 12 — THE ALGORITHM REBORN: phase annotations
            12 => {
                let phase = if self.boss_extra == 0 { 1 }
                    else if self.boss_extra == 1 { 2 }
                    else { 3 };
                let phase_str = match phase {
                    1 => "PHASE 1: EVALUATION",
                    2 => "PHASE 2: ADAPTATION",
                    _ => "PHASE 3: COUNTER-SPECIALIZATION",
                };
                let phase_col = match phase {
                    1 => ac,
                    2 => gld,
                    _ => dng,
                };
                ctx.print_color(40, 3, phase_col, bg, phase_str);

                // Phase 1: chaos field "spells" the player's name letter by letter
                if phase == 1 {
                    let pname = self.player.as_ref().map(|p| p.name.clone()).unwrap_or_default();
                    let name_chars: Vec<char> = pname.chars().collect();
                    let reveal = ((self.frame / 15) as usize).min(name_chars.len());
                    if !name_chars.is_empty() {
                        let partial: String = name_chars[..reveal].iter().collect();
                        let nx = 80 - (name_chars.len() as i32) / 2;
                        ctx.print_color(nx, 37, RGB::from_u8(60, 60, 80), bg,
                            &format!("{}_", partial));
                        // After full reveal: flash it
                        if reveal == name_chars.len() && (self.frame / 20) % 2 == 0 {
                            ctx.print_color(nx - 2, 37, ac, bg,
                                &format!("[ {} ]", pname));
                        }
                    }
                }

                // Phase 2: pulsing border adaptation overlay
                if phase == 2 {
                    let adapt_chars = ["∇","∂","∑","∫","∏","λ"];
                    for i in 0..8i32 {
                        let ch = adapt_chars[((self.frame / 4 + i as u64) as usize) % adapt_chars.len()];
                        let ax = 2 + i * 19;
                        ctx.print_color(ax, 37, gld, bg, ch);
                    }
                    ctx.print_color(3, 38, muted, bg, "The Algorithm is adapting to your strategy…");
                }

                // Phase 3: "I SEE YOU" escalating overlay — particle formation + border pulse
                if phase == 3 {
                    // Multi-layer "I SEE YOU" at different brightness/positions
                    let isy = [
                        (3i32,   37i32, (t.danger.0,     t.danger.1,     t.danger.2)),
                        (55,     25,    (t.danger.0/2,   t.danger.1/2,   t.danger.2/2)),
                        (100,    45,    (t.danger.0/3,   t.danger.1/3,   t.danger.2/3)),
                        (20,     55,    (t.danger.0/4,   t.danger.1/4,   t.danger.2/4)),
                    ];
                    for &(x, y, col) in &isy {
                        let show = (self.frame / 8 + x as u64) % 4 < 3;
                        if show {
                            ctx.print_color(x, y, RGB::from_u8(col.0, col.1, col.2), bg,
                                "I  S E E  Y O U");
                        }
                    }
                    // Borders pulse in danger color cycle
                    let border_phase = (self.frame / 4) % 3;
                    let bpulse_col = match border_phase {
                        0 => RGB::from_u8(t.danger.0, 0, 0),
                        1 => RGB::from_u8(t.danger.0/2, 0, t.danger.2/2),
                        _ => RGB::from_u8(0, 0, t.danger.2),
                    };
                    ctx.draw_box(1, 2, 78, 36, bpulse_col, bg);
                    ctx.draw_box(81, 2, 77, 36, bpulse_col, bg);
                    // Glitch text on enemy name row
                    let glitch_chars = ["#","@","!","?","∞","*","λ","∂"];
                    for gx in 3..35i32 {
                        if ((gx as u64 + self.frame * 7) % 23) < 3 {
                            let gc = glitch_chars[(gx as u64 + self.frame) as usize % glitch_chars.len()];
                            ctx.print_color(gx, 4, dng, bg, gc);
                        }
                    }
                    // Particle formation: particles converge toward "I SEE YOU" positions
                    if self.frame % 6 == 0 && self.particles.len() < 1800 {
                        use std::f32::consts::TAU;
                        let angle = (self.frame as f32 * 0.5) % TAU;
                        let r = 40.0f32;
                        let col = (t.danger.0, t.danger.1, t.danger.2);
                        let targets = [(11.0f32, 37.0f32), (55.0, 25.0), (100.0, 45.0)];
                        let target = targets[((self.frame / 30) as usize) % 3];
                        let px = target.0 + angle.cos() * r;
                        let py = target.1 + angle.sin() * r * 0.5;
                        let vx = (target.0 - px) * 0.04;
                        let vy = (target.1 - py) * 0.04;
                        self.particles.push(Particle::burst(px, py, vx, vy, "·", col, 25));
                    }
                }
            }

            _ => {}
        }
    }

    fn draw_chaos_viz_overlay(&self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let gld = RGB::from_u8(t.gold.0,    t.gold.1,    t.gold.2);
        let suc = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let dng = RGB::from_u8(t.danger.0,  t.danger.1,  t.danger.2);
        let muted = RGB::from_u8(t.muted.0, t.muted.1,   t.muted.2);

        // Overlay box covers the bottom 3/4 of the screen
        let ox = 2i32; let oy = 8i32;
        let ow = 155i32; let oh = 68i32;
        ctx.draw_box(ox, oy, ow, oh,
            RGB::from_u8(t.accent.0 / 2, t.accent.1 / 2, t.accent.2 / 2), bg);
        ctx.print_color(ox + 2, oy, hd, bg, " CHAOS ENGINE VISUALIZER ");
        ctx.print_color(ox + ow - 10, oy, dim, bg, " [V] Close ");

        match &self.last_roll {
            None => {
                ctx.print_color(ox + 3, oy + 4, muted, bg,
                    "No chaos roll yet. Make a combat move to see the engine chain.");
            }
            Some(roll) => {
                // Header: final verdict
                let (verdict, verdict_col) = if roll.is_critical()       { ("CRITICAL HIT",  gld) }
                    else if roll.final_value > 0.5                        { ("CLEAN HIT",     suc) }
                    else if roll.final_value > 0.0                        { ("WEAK HIT",      ac) }
                    else if roll.is_catastrophe()                         { ("CATASTROPHE",   RGB::from_u8(255, 20, 80)) }
                    else                                                   { ("MISS / FAIL",   dng) };
                ctx.print_color(ox + 3, oy + 2, verdict_col, bg,
                    &format!("Final: {:+.4}   {}", roll.final_value, verdict));

                // Progress bar
                let bar_w = 60i32;
                let filled = ((roll.final_value.clamp(-1.0, 1.0) + 1.0) / 2.0 * bar_w as f64) as i32;
                for i in 0..bar_w {
                    let c = if i < filled { verdict_col } else { muted };
                    ctx.print_color(ox + 3 + i, oy + 3, c, bg, if i < filled { "█" } else { "░" });
                }
                ctx.print_color(ox + 3, oy + 3, muted, bg, "-1.0");
                ctx.print_color(ox + 30, oy + 3, muted, bg, "0.0");
                ctx.print_color(ox + ow - 8, oy + 3, muted, bg, "+1.0");

                // Chain steps
                ctx.print_color(ox + 3, oy + 5, hd, bg, "Engine Chain:");
                ctx.print_color(ox + 3, oy + 6, dim, bg,
                    "  #  Engine          Input      Output     Delta");
                ctx.print_color(ox + 3, oy + 7, muted, bg,
                    "  ─  ──────────────  ─────────  ─────────  ─────────");

                for (i, step) in roll.chain.iter().enumerate() {
                    let y = oy + 8 + i as i32 * 2;
                    if y >= oy + oh - 4 { break; }

                    let delta = step.output - step.input;
                    let (out_col, delta_str) = if delta > 0.1      { (suc,   format!("{:+.3}", delta)) }
                        else if delta < -0.1                        { (dng,   format!("{:+.3}", delta)) }
                        else                                        { (muted, format!("{:+.3}", delta)) };

                    // Animated "active" highlight on last step
                    let is_last = i == roll.chain.len() - 1;
                    let row_col = if is_last { ac } else { dim };

                    let eng: String = step.engine_name.chars().take(14).collect();
                    ctx.print_color(ox + 3,  y, row_col, bg, &format!("{:>3}", i + 1));
                    ctx.print_color(ox + 7,  y, row_col, bg, &format!("{:<16}", eng));
                    ctx.print_color(ox + 23, y, muted,   bg, &format!("{:>+9.4}", step.input));
                    ctx.print_color(ox + 34, y, out_col, bg, &format!("{:>+9.4}", step.output));
                    ctx.print_color(ox + 45, y, out_col, bg, &delta_str);

                    // Tiny bar for this step's magnitude
                    let mag = step.output.abs().min(1.0);
                    let bar_len = (mag * 15.0) as usize;
                    let bar_col = if step.output > 0.0 { suc } else { dng };
                    let bar: String = "▪".repeat(bar_len);
                    ctx.print_color(ox + 52, y, bar_col, bg, &bar);
                }

                // Footer: chain stats
                let chain_len = roll.chain.len();
                let pos_count = roll.chain.iter().filter(|s| s.output > s.input).count();
                let neg_count = chain_len - pos_count;
                ctx.print_color(ox + 3, oy + oh - 3, muted, bg,
                    &format!("Chain depth: {}   Positive steps: {}   Negative steps: {}",
                        chain_len, pos_count, neg_count));
            }
        }

        // Room entry flash overlay
        self.draw_room_entry_flash(ctx);
    }

    // ── SHOP ──────────────────────────────────────────────────────────────────

    fn draw_shop(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "SHOP", &t);

        let pgold = self.player.as_ref().map(|p| p.gold).unwrap_or(0);
        stat_line(ctx, 3, 3, "Your Gold: ", &format!("{}g", pgold), t.gold, &t);
        draw_separator(ctx, 1, 4, 155, &t);

        // Heal option
        let heal_row = 5i32;
        let can_heal = self.player.as_ref().map(|p| p.gold >= self.shop_heal_cost).unwrap_or(false);
        ctx.print_color(3, heal_row, if can_heal { suc } else { dim }, bg,
            &format!("[H] Healing Potion  +40 HP  ─  {}g", self.shop_heal_cost));

        draw_separator(ctx, 1, 7, 77, &t);

        for (i, (item, price)) in self.shop_items.iter().enumerate() {
            let y = 9 + i as i32 * 6;
            let is_sel = i + 1 == self.shop_cursor;
            let can_buy = self.player.as_ref().map(|p| p.gold >= *price).unwrap_or(false);
            if is_sel { draw_subpanel(ctx, 2, y - 1, 75, 6, "", &t); }
            let name_col = if is_sel { hd } else { dim };
            let price_col = if can_buy { gld } else { dim };
            let pfx = if is_sel { format!("{} ", cursor_char(self.frame)) } else { "  ".to_string() };
            ctx.print_color(3, y, name_col, bg, &format!("{}[{}] {}", pfx, i+1, &item.name.chars().take(30).collect::<String>()));
            ctx.print_color(55, y, price_col, bg, &format!("{}g ({})", price, item.rarity.name()));
            for (j, m) in item.stat_modifiers.iter().enumerate().take(3) {
                let mc = if m.value > 0 { suc } else { dim };
                ctx.print_color(8, y + 1 + j as i32, mc, bg,
                    &format!("{:+} {}", m.value, m.stat));
            }
        }

        // ── Right panel: player inventory ──
        draw_subpanel(ctx, 80, 3, 77, 68, "YOUR INVENTORY", &t);
        if let Some(ref p) = self.player {
            let php = p.current_hp;
            let pmhp = p.max_hp;
            let hp_pct = php as f32 / pmhp.max(1) as f32;
            let hp_c = t.hp_color(hp_pct);
            stat_line(ctx, 82, 5, "HP  ", &format!("{}/{}", php, pmhp), hp_c, &t);
            draw_bar_gradient(ctx, 82, 6, 73, php, pmhp, hp_c, t.muted, &t);
            stat_line(ctx, 82, 8, "Gold", &format!("{}g", p.gold), t.gold, &t);
            draw_separator(ctx, 81, 9, 75, &t);
            if p.inventory.is_empty() {
                ctx.print_color(82, 11, dim, bg, "(empty inventory)");
            } else {
                for (i, item) in p.inventory.iter().enumerate() {
                    let iy = 11 + i as i32 * 4;
                    if iy > 68 { break; }
                    let ic = match item.rarity {
                        Rarity::Common    => dim,
                        Rarity::Uncommon  => suc,
                        Rarity::Rare      => ac,
                        Rarity::Epic      => RGB::from_u8(160, 0, 220),
                        Rarity::Legendary => gld,
                        Rarity::Mythical  => RGB::from_u8(255, 50, 50),
                        Rarity::Divine    => RGB::from_u8(255, 215, 0),
                        _                 => hd,
                    };
                    ctx.print_color(82, iy, ic, bg, &item.name.chars().take(35).collect::<String>());
                    ctx.print_color(82, iy + 1, dim, bg, &format!("  ({})", item.rarity.name()));
                    let mods: String = item.stat_modifiers.iter()
                        .map(|m| format!("{:+}{}", m.value, &m.stat[..3.min(m.stat.len())]))
                        .collect::<Vec<_>>().join(" ");
                    ctx.print_color(82, iy + 2, dim, bg, &format!("  {}", mods.chars().take(70).collect::<String>()));
                }
            }
        }

        draw_separator(ctx, 1, 74, 155, &t);
        print_hint(ctx, 3, 75, "[1-4]", " Buy item   ", &t);
        print_hint(ctx, 22, 75, "[H]", " Heal   ", &t);
        print_hint(ctx, 31, 75, "[Enter/0/Esc]", " Leave", &t);
    }

    // ── CRAFTING ──────────────────────────────────────────────────────────────

    fn draw_crafting(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "CRAFTING BENCH", &t);

        let has_inventory = self.player.as_ref().map(|p| !p.inventory.is_empty()).unwrap_or(false);
        if !has_inventory {
            print_center(ctx, 2, 22, 75, t.dim, &t, "Your inventory is empty. Nothing to craft.");
            print_hint(ctx, 30, 25, "[Esc/Enter]", " Leave", &t);
            return;
        }

        match self.craft_phase {
            CraftPhase::SelectItem => {
                // Filter bar
                let filter_lc = self.item_filter.to_lowercase();
                let filter_label = if self.item_filter_active {
                    format!("/ {}_ (Enter/Esc to finish)", &self.item_filter)
                } else if !self.item_filter.is_empty() {
                    format!("filter: \"{}\"  [/] to change · [Esc] clear", &self.item_filter)
                } else {
                    "[/] Filter items  ↑↓ Navigate  Enter Confirm".to_string()
                };
                draw_subpanel(ctx, 2, 3, 75, 68, &filter_label, &t);

                if let Some(ref p) = self.player {
                    let mut row = 0i32;
                    for (i, item) in p.inventory.iter().enumerate() {
                        if !filter_lc.is_empty() {
                            if !item.name.to_lowercase().contains(&filter_lc)
                                && !item.rarity.name().to_lowercase().contains(&filter_lc) {
                                continue;
                            }
                        }
                        let is_sel = i == self.craft_item_cursor;
                        let y = 5 + row * 2;
                        if y > 68 { break; }
                        let charge_tag = if item.charges > 0 { format!(" [{}c]", item.charges) } else { String::new() };
                        print_selectable(ctx, 5, y, is_sel,
                            &format!("[{}] {}{} · {}", i+1, &item.name.chars().take(25).collect::<String>(), charge_tag, item.rarity.name()),
                            self.frame, &t);
                        if is_sel {
                            for (j, m) in item.stat_modifiers.iter().enumerate().take(3) {
                                let vc = if m.value > 0 { ac } else { dng };
                                ctx.print_color(10, y + 1 + j as i32, vc, bg,
                                    &format!("{:+} {}", m.value, m.stat));
                            }
                        }
                        row += 1;
                    }
                    if row == 0 && !filter_lc.is_empty() {
                        ctx.print_color(5, 7, dim, bg, "No items match filter.");
                    }
                }

                // Right panel: selected item detail
                draw_subpanel(ctx, 80, 3, 77, 68, "ITEM DETAIL", &t);
                if let Some(ref p) = self.player {
                    if let Some(item) = p.inventory.get(self.craft_item_cursor) {
                        let ic = match item.rarity {
                            Rarity::Common    => dim,
                            Rarity::Uncommon  => suc,
                            Rarity::Rare      => ac,
                            Rarity::Epic      => RGB::from_u8(160, 0, 220),
                            Rarity::Legendary => gld,
                            Rarity::Mythical  => RGB::from_u8(255, 50, 50),
                            Rarity::Divine    => RGB::from_u8(255, 215, 0),
                            _                 => hd,
                        };
                        ctx.print_color(82, 5, ic, bg, &item.name.chars().take(50).collect::<String>());
                        ctx.print_color(82, 6, dim, bg, &format!("Rarity: {}", item.rarity.name()));
                        if item.charges > 0 {
                            ctx.print_color(82, 7, ac, bg, &format!("Charges: {}", item.charges));
                        }
                        draw_separator(ctx, 81, 8, 75, &t);
                        ctx.print_color(82, 9, hd, bg, "Stat Modifiers:");
                        for (j, m) in item.stat_modifiers.iter().enumerate() {
                            let mc = if m.value > 0 { suc } else { dng };
                            ctx.print_color(84, 11 + j as i32, mc, bg,
                                &format!("{:+} {}", m.value, m.stat));
                        }
                        draw_separator(ctx, 81, 20, 75, &t);
                        ctx.print_color(82, 21, hd, bg, "Craft operations available:");
                        let ops = ["Reforge", "Augment", "Annul", "Corrupt", "Fuse", "EngineLock", "Shatter", "Imbue", "Repair"];
                        for (k, op) in ops.iter().enumerate() {
                            ctx.print_color(84, 23 + k as i32, dim, bg, &format!("[{}] {}", k+1, op));
                        }
                    }
                }

                draw_separator(ctx, 2, 74, 155, &t);
                print_hint(ctx, 4, 75, "↑↓", " Navigate   ", &t);
                print_hint(ctx, 20, 75, "Enter", " Select   ", &t);
                print_hint(ctx, 35, 75, "/", " Filter   ", &t);
                print_hint(ctx, 48, 75, "Esc", " Leave", &t);
            }
            CraftPhase::SelectOp => {
                let (item_name, item_rarity, item_mods) = self.player.as_ref()
                    .and_then(|p| p.inventory.get(self.craft_item_cursor))
                    .map(|i| (i.name.clone(), i.rarity.name(), i.stat_modifiers.clone()))
                    .unwrap_or_default();

                ctx.print_color(3, 3, hd, bg, &format!("Crafting: {}", &item_name.chars().take(50).collect::<String>()));
                ctx.print_color(3, 4, dim, bg, &format!("Rarity: {}", item_rarity));
                draw_separator(ctx, 2, 5, 75, &t);

                let ops = [
                    ("Reforge",    "Chaos-reroll ALL stat modifiers from scratch",           t.warn),
                    ("Augment",    "ADD one new chaos-rolled modifier",                       t.success),
                    ("Annul",      "REMOVE one random modifier",                             t.danger),
                    ("Corrupt",    "Choose risk tier — Safe/Risky/Reckless — to roll chaos", t.xp),
                    ("Fuse",       "DOUBLE all values and upgrade rarity tier",              t.gold),
                    ("EngineLock", "Lock a chaos engine signature into the item",            t.mana),
                    ("Shatter",    "DESTROY item — scatter its mods to other items",         t.danger),
                    ("Imbue",      "Grant item 3 CHARGES (bonus effect on use)",             t.mana),
                    ("Repair",     "RESTORE item durability to maximum (costs gold)",        t.success),
                ];
                for (i, (name, desc, col)) in ops.iter().enumerate() {
                    let is_sel = i == self.craft_op_cursor;
                    let y = 8 + i as i32 * 5;
                    if is_sel { draw_subpanel(ctx, 2, y - 1, 75, 4, "", &t); }
                    let fc = RGB::from_u8(col.0, col.1, col.2);
                    let pfx = if is_sel { format!("{} ", cursor_char(self.frame)) } else { "  ".to_string() };
                    ctx.print_color(5, y, if is_sel { fc } else { dim }, bg,
                        &format!("{}[{}] {}", pfx, i+1, name));
                    ctx.print_color(10, y + 1, dim, bg, desc);
                }

                if !self.craft_message.is_empty() {
                    draw_separator(ctx, 2, 58, 75, &t);
                    ctx.print_color(4, 59, gld, bg, &self.craft_message.chars().take(72).collect::<String>());
                }

                // Right panel: current item mods
                draw_subpanel(ctx, 80, 3, 77, 68, "CURRENT MODIFIERS", &t);
                ctx.print_color(82, 5, hd, bg, &item_name.chars().take(50).collect::<String>());
                ctx.print_color(82, 6, dim, bg, &format!("Rarity: {}", item_rarity));
                draw_separator(ctx, 81, 7, 75, &t);
                if item_mods.is_empty() {
                    ctx.print_color(82, 9, dim, bg, "(no modifiers)");
                } else {
                    for (j, m) in item_mods.iter().enumerate() {
                        let mc = if m.value > 0 { suc } else { dng };
                        ctx.print_color(82, 9 + j as i32 * 2, mc, bg,
                            &format!("{:+} {}", m.value, m.stat));
                    }
                }

                draw_separator(ctx, 2, 74, 155, &t);
                print_hint(ctx, 4, 75, "↑↓ / 1-8", " Select op   ", &t);
                print_hint(ctx, 28, 75, "Enter", " Apply   ", &t);
                print_hint(ctx, 43, 75, "Esc", " Back", &t);
            }
        }

        // ── Craft animation overlay ────────────────────────────────────────
        if self.craft_anim_timer > 0 {
            self.craft_anim_timer -= 1;
            let elapsed = 40 - self.craft_anim_timer;
            let alpha = if elapsed < 10 {
                elapsed as f32 / 10.0
            } else if self.craft_anim_timer < 10 {
                self.craft_anim_timer as f32 / 10.0
            } else { 1.0 };
            let bg_rgb = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
            match self.craft_anim_type {
                1 => {  // Reforge: item text dissolves into particles then reassembles
                    let pulse = (elapsed % 4) < 2;
                    let col = RGB::from_u8(
                        (t.accent.0 as f32 * alpha) as u8,
                        (t.accent.1 as f32 * alpha) as u8,
                        (t.accent.2 as f32 * alpha) as u8,
                    );
                    let dissolve_chars = ["░","▒","▓","█","▓","▒","░"];
                    for i in 0..8i32 {
                        let dc = dissolve_chars[(elapsed as usize + i as usize) % dissolve_chars.len()];
                        ctx.print_color(82 + i * 2, 5, col, bg_rgb, dc);
                    }
                    if elapsed >= 20 {
                        let reassemble_chars = ["*","+","·","✦","★"];
                        for i in 0..5i32 {
                            if (elapsed as usize - 20) >= i as usize * 4 {
                                ctx.print_color(82 + i * 3, 5, col, bg_rgb,
                                    reassemble_chars[i as usize % reassemble_chars.len()]);
                            }
                        }
                    }
                    // Spray particles during middle of animation
                    if elapsed > 5 && elapsed < 30 && self.frame % 3 == 0 {
                        let col_t = (t.accent.0, t.accent.1, t.accent.2);
                        for i in 0..4usize {
                            use std::f32::consts::TAU;
                            let angle = (i as f32 * TAU / 4.0) + elapsed as f32 * 0.3;
                            self.particles.push(Particle::spark(
                                90.0, 5.0, angle.cos() * 0.2, angle.sin() * 0.12,
                                ["·","*","+"][i % 3], col_t));
                        }
                    }
                }
                2 => {  // Corrupt: screen shake + glitch text overlay
                    let glitch_chars = ["?","!","#","@","∞","∑","λ"];
                    for i in 0..12i32 {
                        let gx = 82 + (i * 13 + elapsed as i32 * 7) % 70;
                        let gy = 5 + (i * 7 + elapsed as i32 * 3) % 8;
                        let gc = glitch_chars[(elapsed as usize + i as usize * 3) % glitch_chars.len()];
                        let gc_col = RGB::from_u8(
                            ((t.danger.0 as f32) * alpha) as u8,
                            ((t.danger.1 as f32) * alpha * 0.3) as u8,
                            ((t.danger.2 as f32) * alpha * 0.8) as u8,
                        );
                        ctx.print_color(gx.clamp(82, 155), gy.clamp(3, 68), gc_col, bg_rgb, gc);
                    }
                }
                3 => {  // Shatter: characters explode outward
                    let col_t = (t.danger.0, t.danger.1, t.danger.2);
                    if elapsed < 5 && self.frame % 2 == 0 {
                        emit_death_explosion(&mut self.particles, 90.0, 35.0, col_t);
                    }
                    let shard_col = RGB::from_u8(
                        (t.danger.0 as f32 * alpha) as u8, 0, 0);
                    ctx.print_color(82, 5, shard_col, bg_rgb, "[ SHATTERED ]");
                }
                _ => {}
            }
        }
    }

    // ── CHARACTER SHEET ───────────────────────────────────────────────────────

    fn draw_character_sheet(&mut self, ctx: &mut BTerm) {
        use chaos_rpg_core::factions::{Faction, ReputationTier};

        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let muted = RGB::from_u8(t.muted.0, t.muted.1, t.muted.2);

        let p = match &self.player { Some(p) => p.clone(), None => { self.screen = AppScreen::FloorNav; return; } };

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "", &t);

        // ── Universal header ──────────────────────────────────────────────────
        let header = format!(" {} — {} Lv.{} ", p.name, p.class.name(), p.level);
        print_center(ctx, 0, 1, 159, t.heading, &t, &header);

        // ── Tab bar (row 2) ───────────────────────────────────────────────────
        // Tabs: 0=Stats  1=Inventory  2=Effects  3=Lore  4=Log
        let tabs = ["Stats","Inventory","Effects","Lore","Log"];
        let tab_x: [i32; 5] = [2, 20, 42, 60, 72];
        for (i, (label, &x)) in tabs.iter().zip(tab_x.iter()).enumerate() {
            let is_sel = i == self.char_tab as usize;
            let fg = if is_sel {
                RGB::from_u8(t.selected.0, t.selected.1, t.selected.2)
            } else {
                RGB::from_u8(t.dim.0, t.dim.1, t.dim.2)
            };
            let text = if is_sel { format!("[{}]", label) } else { format!(" {}  ", label) };
            ctx.print_color(x, 2, fg, bg, &text);
        }
        ctx.print_color(90, 2, muted, bg, "← → Tab to switch");
        draw_separator(ctx, 1, 3, 157, &t);

        // ── Tab content (rows 4-71) ───────────────────────────────────────────
        match self.char_tab {
            // ── Tab 0: Stats ──────────────────────────────────────────────────
            0 => {
                // Left: core stats with animated bars
                draw_subpanel(ctx, 1, 4, 38, 65, "CORE STATS", &t);
                let stats = [
                    ("Force",     p.stats.force),
                    ("Vitality",  p.stats.vitality),
                    ("Mana",      p.stats.mana),
                    ("Cunning",   p.stats.cunning),
                    ("Precision", p.stats.precision),
                    ("Entropy",   p.stats.entropy),
                    ("Luck",      p.stats.luck),
                ];
                for (i, (name, val)) in stats.iter().enumerate() {
                    let base_col = if *val < 0 { t.danger } else if *val >= 50 { t.gold } else { t.heading };
                    let col = if *val >= 80 {
                        let pal = [(220u8,180u8,40u8),(60,220,80),(80,200,220),(80,80,220),(180,60,200),(220,60,60)];
                        pal[((self.frame / 8 + i as u64) as usize) % pal.len()]
                    } else if *val >= 50 {
                        let bright = (self.frame / 12) % 2 == 0;
                        if bright { t.gold } else { (t.gold.0/2 + 30, t.gold.1/2 + 20, 10) }
                    } else if *val < 0 {
                        let jitter = (self.frame / 5 + i as u64) % 4 < 2;
                        if jitter { t.danger } else { (t.danger.0/2, 0, 0) }
                    } else { base_col };
                    stat_line(ctx, 3, 6 + i as i32 * 4, name, &format!("{:+}", val), col, &t);
                    let bar_val = (*val).max(0).min(100);
                    draw_bar_solid(ctx, 3, 7 + i as i32 * 4, 34, bar_val, 100, col, &t);
                    if *val >= 70 && self.frame % 20 == (i as u64 * 4) % 20 {
                        self.particles.push(Particle::spark(
                            37.0, (6 + i as i32 * 4) as f32, 0.05, -0.08, "✦", col));
                    }
                }

                // Power tier
                let tier = p.power_tier();
                let tier_rgb = tier.rgb();
                let tier_col = if tier.has_effect() {
                    use chaos_rpg_core::power_tier::TierEffect;
                    match tier.effect() {
                        TierEffect::Rainbow | TierEffect::RainbowFast => {
                            let pal = [(220u8,60u8,60u8),(220,180,40),(60,200,80),(80,200,220),(80,80,220),(180,60,200)];
                            pal[((self.frame / 4) as usize) % pal.len()]
                        }
                        TierEffect::Pulse => {
                            if (self.frame / 15) % 2 == 0 { tier_rgb } else { (tier_rgb.0/2, tier_rgb.1/2, tier_rgb.2/2) }
                        }
                        _ => tier_rgb,
                    }
                } else { tier_rgb };
                let (plabel, pval) = p.power_display();
                use chaos_rpg_core::power_tier::TierEffect as TierFx;
                let display_val = if tier.has_effect() && matches!(tier.effect(), TierFx::Flash) {
                    let glitch_chars = ["?","#","@","!","∞","∑","λ","░","▒"];
                    pval.chars().map(|c| {
                        let r = (c as u64 * 31 + self.frame * 7) % 100;
                        if r < 20 { glitch_chars[(c as u64 + self.frame) as usize % glitch_chars.len()].to_string() }
                        else { c.to_string() }
                    }).collect::<String>()
                } else { pval };
                stat_line(ctx, 3, 34, &format!("{}: ", plabel), &display_val, tier_col, &t);
                let flavor: String = tier.flavor().chars().take(32).collect();
                ctx.print_color(3, 35, RGB::from_u8(tier_col.0/2+10, tier_col.1/2+10, tier_col.2/2+10), bg, &flavor);

                // Middle: run info + class
                draw_subpanel(ctx, 41, 4, 38, 30, "RUN INFO", &t);
                stat_line(ctx, 43, 6,  "Floor  ", &format!("{}", p.floor),  t.accent, &t);
                stat_line(ctx, 43, 7,  "Kills  ", &format!("{}", p.kills),  t.success, &t);
                stat_line(ctx, 43, 8,  "Gold   ", &format!("{}g", p.gold),  t.gold, &t);
                stat_line(ctx, 43, 9,  "XP     ", &format!("{}", p.xp),     t.xp, &t);
                stat_line(ctx, 43, 10, "HP     ", &format!("{}/{}", p.current_hp, p.max_hp),
                    t.hp_color(p.current_hp as f32 / p.max_hp.max(1) as f32), &t);
                stat_line(ctx, 43, 11, "MP     ", &format!("{}/{}", self.current_mana, self.max_mana()), t.mana, &t);
                stat_line(ctx, 43, 12, "Corrupt", &format!("{}", p.corruption), t.warn, &t);
                stat_line(ctx, 43, 13, "Class  ", p.class.name(), t.heading, &t);
                stat_line(ctx, 43, 14, "BG     ", p.background.name(), t.dim, &t);
                if p.skill_points > 0 {
                    let pulse = (self.frame / 12) % 2 == 0;
                    let pc_col = if pulse { t.gold } else { (t.gold.0/2+20, t.gold.1/2+20, 10) };
                    stat_line(ctx, 43, 15, "SkPts  ", &format!("{} avail", p.skill_points), pc_col, &t);
                }
                if p.misery.misery_index >= 100.0 {
                    stat_line(ctx, 43, 16, "Misery ", &format!("{:.0}", p.misery.misery_index), t.warn, &t);
                }
                if p.underdog_multiplier() > 1.01 {
                    stat_line(ctx, 43, 17, "Underdg", &format!("×{:.2}", p.underdog_multiplier()), t.gold, &t);
                }

                // Middle lower: factions
                draw_subpanel(ctx, 41, 36, 38, 33, "FACTIONS", &t);
                let factions = [
                    ("Order",    p.faction_rep.order,    Faction::OrderOfConvergence),
                    ("Cult",     p.faction_rep.cult,     Faction::CultOfDivergence),
                    ("Watchers", p.faction_rep.watchers, Faction::WatchersOfBoundary),
                ];
                for (i, (fname, frep, fvar)) in factions.iter().enumerate() {
                    let ftier = ReputationTier::from_rep(*frep);
                    let fc = match ftier {
                        ReputationTier::Hostile    => dng,
                        ReputationTier::Neutral    => dim,
                        ReputationTier::Recognized => suc,
                        ReputationTier::Trusted    => gld,
                        ReputationTier::Exalted    => hd,
                    };
                    let fy = 38 + i as i32 * 4;
                    ctx.print_color(43, fy, fc, bg, &format!("{} — {} ({:+})", fname, ftier.name(), frep));
                    if let Some(bonus) = chaos_rpg_core::factions::FactionRep::passive_bonus(*fvar, ftier) {
                        ctx.print_color(45, fy + 1, dim, bg, &bonus.chars().take(32).collect::<String>());
                    }
                }

                // Right: class info + boon
                draw_subpanel(ctx, 81, 4, 76, 30, "CLASS & BOON", &t);
                ctx.print_color(83, 6, hd, bg, p.class.name());
                ctx.print_color(83, 7, dim, bg, &format!("BG: {}", p.background.name()));
                ctx.print_color(83, 8, ac, bg, p.class.passive_name());
                let mut pr = 9i32;
                let mut pline = String::new();
                for w in p.class.passive_desc().split_whitespace() {
                    if pline.len() + w.len() + 1 > 72 {
                        ctx.print_color(83, pr, dim, bg, &pline);
                        pline = w.to_string(); pr += 1;
                    } else {
                        if !pline.is_empty() { pline.push(' '); }
                        pline.push_str(w);
                    }
                }
                if !pline.is_empty() { ctx.print_color(83, pr, dim, bg, &pline); }
                draw_separator(ctx, 82, 17, 74, &t);
                ctx.print_color(83, 18, hd, bg, "Active Boon:");
                if let Some(ref boon) = p.boon {
                    ctx.print_color(83, 19, ac, bg, boon.name());
                    ctx.print_color(83, 20, dim, bg, &boon.description().chars().take(72).collect::<String>());
                } else {
                    ctx.print_color(83, 19, dim, bg, "No boon active.");
                }
                draw_separator(ctx, 82, 22, 74, &t);
                let sp = p.skill_points;
                ctx.print_color(83, 23, dim, bg, &format!("{} passive nodes  |  {} skill pts", p.allocated_nodes.len(), sp));

                // Right lower: passive tree summary
                draw_subpanel(ctx, 81, 36, 76, 33, "PASSIVE TREE", &t);
                let node_count = p.allocated_nodes.len();
                ctx.print_color(83, 38, dim, bg, &format!("{} nodes allocated", node_count));
                if sp > 0 {
                    let pulse = (self.frame / 12) % 2 == 0;
                    let pc = if pulse { gld } else { RGB::from_u8(t.gold.0/2+20, t.gold.1/2+20, 10) };
                    ctx.print_color(83, 39, pc, bg, &format!("★ {} SKILL POINT{} AVAILABLE — [P] to allocate",
                        sp, if sp == 1 { "" } else { "S" }));
                }
            }

            // ── Tab 1: Inventory ──────────────────────────────────────────────
            1 => {
                // Left: items list
                draw_subpanel(ctx, 1, 4, 76, 65, "INVENTORY", &t);
                if p.inventory.is_empty() {
                    ctx.print_color(3, 8, dim, bg, "(inventory empty — find items in Treasure, Shop, or Boss rooms)");
                } else {
                    for (i, item) in p.inventory.iter().enumerate() {
                        let iy = 6 + i as i32 * 4;
                        if iy > 67 { break; }
                        let ic = match item.rarity {
                            Rarity::Common    => dim,
                            Rarity::Uncommon  => suc,
                            Rarity::Rare      => ac,
                            Rarity::Epic      => RGB::from_u8(160, 0, 220),
                            Rarity::Legendary => gld,
                            Rarity::Mythical  => RGB::from_u8(255, 50, 50),
                            Rarity::Divine    => RGB::from_u8(255, 215, 0),
                            _                 => hd,
                        };
                        let equip_marker = if item.equip_slot().is_some() { "⚙" } else { " " };
                        ctx.print_color(3, iy, ic, bg,
                            &format!("{}{:<3} {}", equip_marker, i+1, &item.name.chars().take(40).collect::<String>()));
                        ctx.print_color(3, iy + 1, dim, bg, &format!("    Rarity: {}  Dur:{}/{}",
                            item.rarity.name(), item.durability, item.max_durability));
                        let mods: String = item.stat_modifiers.iter()
                            .map(|m| format!("{:+}{}", m.value, &m.stat[..4.min(m.stat.len())]))
                            .collect::<Vec<_>>().join("  ");
                        ctx.print_color(3, iy + 2, dim, bg,
                            &format!("    {}", mods.chars().take(68).collect::<String>()));
                    }
                    if p.inventory.len() > 16 {
                        ctx.print_color(3, 68, muted, bg,
                            &format!("… and {} more  (go to Crafting Bench to see all)", p.inventory.len() - 16));
                    }
                }

                // Right: equipped items
                draw_subpanel(ctx, 79, 4, 78, 65, "EQUIPPED", &t);
                if let Some(ref pfull) = self.player {
                    use chaos_rpg_core::character::EquipSlot;
                    let eq_slots = [
                        (EquipSlot::Weapon, "Weapon"),
                        (EquipSlot::Body,   "Body"),
                        (EquipSlot::Ring1,  "Ring 1"),
                        (EquipSlot::Ring2,  "Ring 2"),
                        (EquipSlot::Amulet, "Amulet"),
                    ];
                    for (j, (slot, slabel)) in eq_slots.iter().enumerate() {
                        let ey = 6 + j as i32 * 12;
                        if ey > 67 { break; }
                        ctx.print_color(81, ey, muted, bg, slabel);
                        if let Some(item) = pfull.equipped.get(*slot) {
                            let ic = match item.rarity {
                                Rarity::Legendary => gld,
                                Rarity::Epic      => RGB::from_u8(160, 0, 220),
                                Rarity::Rare      => ac,
                                Rarity::Uncommon  => suc,
                                _                 => dim,
                            };
                            let dur_pct = item.durability as f32 / item.max_durability.max(1) as f32;
                            let dur_col = if dur_pct < 0.25 { dng }
                                else if dur_pct < 0.5 { RGB::from_u8(220,160,40) } else { dim };
                            ctx.print_color(81, ey + 1, ic, bg,
                                &item.name.chars().take(40).collect::<String>());
                            ctx.print_color(81, ey + 2, dur_col, bg,
                                &format!("  Dur: {}/{}", item.durability, item.max_durability));
                            let mods: String = item.stat_modifiers.iter()
                                .map(|m| format!("{:+}{}", m.value, &m.stat[..4.min(m.stat.len())]))
                                .collect::<Vec<_>>().join("  ");
                            ctx.print_color(81, ey + 3, dim, bg,
                                &format!("  {}", mods.chars().take(70).collect::<String>()));
                        } else {
                            ctx.print_color(81, ey + 1, muted, bg, "  (empty)");
                        }
                    }
                }
            }

            // ── Tab 2: Effects ────────────────────────────────────────────────
            2 => {
                // Left: status effects
                draw_subpanel(ctx, 1, 4, 76, 65, "STATUS EFFECTS", &t);
                let badges = p.status_badges_plain();
                if badges.is_empty() {
                    ctx.print_color(3, 8, dim, bg, "No active status effects.");
                    ctx.print_color(3, 10, muted, bg, "Effects are applied during combat.");
                } else {
                    for (i, badge) in badges.split('|').enumerate() {
                        let badge = badge.trim();
                        if badge.is_empty() { continue; }
                        let y = 6 + i as i32 * 2;
                        if y > 67 { break; }
                        ctx.print_color(3, y, ac, bg, badge);
                    }
                }

                // Right: corruption + misery info
                draw_subpanel(ctx, 79, 4, 78, 30, "CORRUPTION & MISERY", &t);
                stat_line(ctx, 81, 6, "Corruption ", &format!("{}", p.corruption), t.warn, &t);
                if p.corruption > 0 {
                    let desc = if p.corruption <= 5 { "Minor chaos distortion." }
                               else if p.corruption <= 20 { "Rolls occasionally invert." }
                               else { "The dungeon lies to you." };
                    ctx.print_color(81, 7, dim, bg, desc);
                }
                stat_line(ctx, 81, 9, "Misery     ", &format!("{:.0}", p.misery.misery_index), t.warn, &t);
                if p.misery.misery_index >= 100.0 {
                    ctx.print_color(81, 10, dng, bg, "☠ SPITE MODE — enemies empowered");
                }
                stat_line(ctx, 81, 12, "Defiance   ", &format!("{} rolls", p.misery.defiance_rolls), t.accent, &t);
                if p.misery.defiance_rolls > 0 {
                    ctx.print_color(81, 13, dim, bg, "Near-death negation charges.");
                }
                if p.misery.spite > 0.0 {
                    stat_line(ctx, 81, 15, "Spite      ", &format!("{:.0}", p.misery.spite), t.danger, &t);
                }
                stat_line(ctx, 81, 17, "Underdog   ", &format!("×{:.2}", p.underdog_multiplier()), t.gold, &t);
                if p.underdog_multiplier() > 1.01 {
                    ctx.print_color(81, 18, dim, bg, "Bonus dmg vs stronger enemies.");
                }

                // Right lower: floor ability info
                draw_subpanel(ctx, 79, 36, 78, 33, "FLOOR ABILITIES (ENEMIES)", &t);
                ctx.print_color(81, 38, dim, bg, "On floor 20+  enemies have StatMirror:");
                ctx.print_color(81, 39, muted, bg, "  HP = your highest stat value at encounter.");
                ctx.print_color(81, 41, dim, bg, "On floor 40+  enemies have EngineTheft:");
                ctx.print_color(81, 42, muted, bg, "  Successful hits steal 1 engine from your next roll.");
                ctx.print_color(81, 44, dim, bg, "On floor 60+  enemies have NullifyAura:");
                ctx.print_color(81, 45, muted, bg, "  Your first action each combat returns base stats only.");
            }

            // ── Tab 3: Lore ───────────────────────────────────────────────────
            3 => {
                draw_subpanel(ctx, 1, 4, 157, 65, "LORE — CLASS & BACKGROUND", &t);
                ctx.print_color(3, 6, hd, bg, &format!("Class: {}", p.class.name()));
                ctx.print_color(3, 7, ac, bg, p.class.passive_name());
                let mut ly = 9i32;
                let mut lline = String::new();
                for w in p.class.passive_desc().split_whitespace() {
                    if lline.len() + w.len() + 1 > 152 {
                        ctx.print_color(3, ly, dim, bg, &lline);
                        lline = w.to_string(); ly += 1;
                    } else {
                        if !lline.is_empty() { lline.push(' '); }
                        lline.push_str(w);
                    }
                }
                if !lline.is_empty() { ctx.print_color(3, ly, dim, bg, &lline); ly += 1; }

                ly += 1;
                draw_separator(ctx, 2, ly, 154, &t);
                ly += 1;
                ctx.print_color(3, ly, hd, bg, &format!("Background: {}", p.background.name()));
                ly += 1;
                ctx.print_color(3, ly, dim, bg, p.background.description());
                ly += 2;

                draw_separator(ctx, 2, ly, 154, &t);
                ly += 1;
                ctx.print_color(3, ly, hd, bg, "Factions");
                ly += 1;
                let factions = [
                    ("Order of Convergence", p.faction_rep.order,    Faction::OrderOfConvergence),
                    ("Cult of Divergence",   p.faction_rep.cult,     Faction::CultOfDivergence),
                    ("Watchers of Boundary", p.faction_rep.watchers, Faction::WatchersOfBoundary),
                ];
                for (fname, frep, fvar) in &factions {
                    if ly > 67 { break; }
                    let ftier = ReputationTier::from_rep(*frep);
                    let fc = match ftier {
                        ReputationTier::Hostile    => dng,
                        ReputationTier::Neutral    => dim,
                        ReputationTier::Recognized => suc,
                        ReputationTier::Trusted    => gld,
                        ReputationTier::Exalted    => hd,
                    };
                    ctx.print_color(3, ly, fc, bg,
                        &format!("{}: {} ({:+})", fname, ftier.name(), frep));
                    if let Some(bonus) = chaos_rpg_core::factions::FactionRep::passive_bonus(*fvar, ftier) {
                        ctx.print_color(5, ly + 1, dim, bg, &bonus.chars().take(150).collect::<String>());
                        ly += 2;
                    } else {
                        ly += 1;
                    }
                }
            }

            // ── Tab 4: Log ────────────────────────────────────────────────────
            _ => {
                draw_subpanel(ctx, 1, 4, 157, 65, "COMBAT LOG", &t);
                if self.combat_log.is_empty() {
                    ctx.print_color(3, 8, dim, bg, "(no log entries yet — enter a combat room first)");
                } else {
                    let log_start = self.combat_log.len().saturating_sub(60);
                    for (i, line) in self.combat_log[log_start..].iter().enumerate() {
                        let y = 6 + i as i32;
                        if y > 67 { break; }
                        let fg = if line.contains("SHATTERED") || line.contains("CATASTROPHE") {
                                     dng
                                 } else if line.contains("CRIT") || line.contains("BOSS") { dng }
                                 else if line.contains("Victory") || line.contains("LEVEL") { gld }
                                 else if line.contains("heal") || line.contains('+') { suc }
                                 else { dim };
                        ctx.print_color(3, y, fg, bg, &line.chars().take(154).collect::<String>());
                    }
                }
            }
        }

        // ── Universal footer ──────────────────────────────────────────────────
        draw_separator(ctx, 1, 72, 157, &t);
        print_hint(ctx, 3,  73, "← →", " Tab  ", &t);
        print_hint(ctx, 16, 73, "[N]", " Auto-alloc  ", &t);
        print_hint(ctx, 34, 73, "[P]", " Full Tree  ", &t);
        print_hint(ctx, 50, 73, "[B]", " Body  ", &t);
        print_hint(ctx, 61, 73, "[Esc]", " Back", &t);

        // Render stat sparkle particles
        for p in &mut self.particles { p.step(); }
        self.particles.retain(|p| p.alive());
        if self.config.visuals.enable_particles {
            for p in &self.particles {
                let rc = p.render_col();
                let px = p.x as i32; let py = p.y as i32;
                if py < 4 || py > 72 || px < 2 || px > 157 { continue; }
                ctx.print_color(px, py, RGB::from_u8(rc.0, rc.1, rc.2), bg, &p.text);
            }
        }
    }

    // ── TUTORIAL ─────────────────────────────────────────────────────────────

    fn draw_tutorial(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg    = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd    = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac    = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim   = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let sel   = RGB::from_u8(t.selected.0,t.selected.1,t.selected.2);
        let warn  = RGB::from_u8(t.warn.0,    t.warn.1,    t.warn.2);
        let succ  = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let danger= RGB::from_u8(t.danger.0,  t.danger.1,  t.danger.2);

        self.chaos_bg(ctx);
        let slide = self.tutorial_slide.max(1);

        // Outer panel
        draw_panel(ctx, 0, 0, 159, 79,
            &format!("CHAOS ENGINE — HOW TO PLAY  [{}/5]", slide), &t);

        // Progress dots
        for i in 1..=5usize {
            let dot = if i == slide { "◆" } else { "◇" };
            let col = if i == slide { ac } else { dim };
            ctx.print_color(35 + (i as i32 - 1) * 3, 2, col, bg, dot);
        }

        const TOTAL_SLIDES: usize = 5;

        match slide {
            // ── Slide 1: What is the Chaos Engine? ────────────────────────
            1 => {
                ctx.print_color(4, 5,  hd,   bg, "SLIDE 1 — THE CHAOS ENGINE");
                ctx.print_color(4, 7,  sel,  bg, "Every action in this game runs through the Chaos Engine.");
                ctx.print_color(4, 9,  dim,  bg, "When you attack, cast a spell, or get hit, the engine fires:");
                ctx.print_color(4, 11, ac,   bg, "  1.  A base roll is computed from your stats.");
                ctx.print_color(4, 12, ac,   bg, "  2.  A chain of sub-engines each modify the result.");
                ctx.print_color(4, 13, ac,   bg, "  3.  The final value determines damage / healing / effect.");
                ctx.print_color(4, 15, warn, bg, "The chain length grows with depth. Floor 1 = 2 links.");
                ctx.print_color(4, 16, warn, bg, "Floor 50 = 8+ links. Floor 100+ = fully recursive chaos.");
                ctx.print_color(4, 18, dim,  bg, "Example chain:  base(42) → sigmoid(38) → entropy(61) → CRIT(97)");
                ctx.print_color(4, 19, succ, bg, "                                                  ^^^  97 damage!");
                ctx.print_color(4, 21, dim,  bg, "The same attack can deal 12 damage or 340 damage.");
                ctx.print_color(4, 22, dim,  bg, "That is not a bug. That IS the game.");

                // Mini ASCII diagram
                ctx.print_color(12, 25, ac,  bg,  "[ YOU ]──►[ engine A ]──►[ engine B ]──►[ RESULT ]");
                ctx.print_color(12, 26, dim, bg,  "   ↑            ↑               ↑");
                ctx.print_color(12, 27, dim, bg,  " stats      sigmoid          entropy");
            }

            // ── Slide 2: Body System ──────────────────────────────────────
            2 => {
                ctx.print_color(4, 5,  hd,   bg, "SLIDE 2 — THE BODY SYSTEM");
                ctx.print_color(4, 7,  sel,  bg, "Your character has 13 body parts, each with independent HP.");
                ctx.print_color(4, 9,  dim,  bg, "Damage is distributed to body parts based on hit location.");
                ctx.print_color(4, 11, ac,   bg, "  Head   — controls   Focus / Chaos resistance");
                ctx.print_color(4, 12, ac,   bg, "  Torso  — core HP pool; death if it reaches 0");
                ctx.print_color(4, 13, ac,   bg, "  Arms   — attack damage & carry weight");
                ctx.print_color(4, 14, ac,   bg, "  Legs   — dodge & movement");
                ctx.print_color(4, 15, ac,   bg, "  Spine  — links all systems; injury = cascading debuffs");
                ctx.print_color(4, 17, warn, bg, "Injury Severities:  Bruised → Fractured → Shattered");
                ctx.print_color(4, 18, warn, bg, "                    → Severed → MATH.ABSENT");
                ctx.print_color(4, 20, dim,  bg, "MATH.ABSENT = the part is gone. The engine notices.");
                ctx.print_color(4, 21, danger,bg,"A severed Head triggers immediate Death Math.");
                ctx.print_color(4, 23, dim,  bg, "Press [B] from the floor map to view your Body Chart.");
            }

            // ── Slide 3: Passive Tree ─────────────────────────────────────
            3 => {
                ctx.print_color(4, 5,  hd,   bg, "SLIDE 3 — PASSIVE SKILL TREE");
                ctx.print_color(4, 7,  sel,  bg, "Every kill grants skill points. Spend them in the Passive Tree.");
                ctx.print_color(4, 9,  dim,  bg, "Nodes are organized by type:");
                ctx.print_color(4, 11, ac,   bg, "  Stat       — +STR, +DEX, +HP etc.  Small bonuses.");
                ctx.print_color(4, 12, warn, bg, "  Notable    — Named abilities. Larger effects.");
                ctx.print_color(4, 13, danger,bg, "  Keystone   — Game-changing rules. Read carefully.");
                ctx.print_color(4, 14, RGB::from_u8(160,80,255), bg,
                                            "  Engine     — Modify the chaos chain itself.");
                ctx.print_color(4, 15, succ, bg, "  Synergy    — Unlock combos between other nodes.");
                ctx.print_color(4, 17, dim,  bg, "Some nodes require other nodes as prerequisites.");
                ctx.print_color(4, 18, dim,  bg, "Press [P] on the Character Sheet to open the full tree.");
                ctx.print_color(4, 19, dim,  bg, "Press [N] anywhere to auto-allocate pending points.");
                ctx.print_color(4, 21, warn, bg, "Keystones often have drawbacks. READ THEM.");
            }

            // ── Slide 4: Misery + Corruption ─────────────────────────────
            4 => {
                ctx.print_color(4, 5,  hd,    bg, "SLIDE 4 — MISERY & CORRUPTION");
                ctx.print_color(4, 7,  sel,   bg, "Misery is a global modifier that makes everything worse.");
                ctx.print_color(4, 9,  dim,   bg, "It accumulates when you:");
                ctx.print_color(4, 11, danger,bg, "  × Take heavy damage        + Misery");
                ctx.print_color(4, 12, danger,bg, "  × Use cursed items          + Misery");
                ctx.print_color(4, 13, danger,bg, "  × Die and continue (Story)  + Misery");
                ctx.print_color(4, 14, danger,bg, "  × Fail skill checks         + Misery");
                ctx.print_color(4, 16, succ,  bg, "High Misery unlocks the Hall of Misery leaderboard.");
                ctx.print_color(4, 17, dim,   bg, "Defiance builds when you survive near-death moments.");
                ctx.print_color(4, 18, dim,   bg, "At 100 Defiance: one death is negated — Spite activates.");
                ctx.print_color(4, 20, warn,  bg, "Corruption > 5 means your chaos rolls can invert.");
                ctx.print_color(4, 21, warn,  bg, "Corruption > 20: the dungeon itself starts lying to you.");
                ctx.print_color(4, 23, dim,   bg, "Watch the amber alert on the floor map. It means something.");
            }

            // ── Slide 5: Tips & Keys ──────────────────────────────────────
            _ => {
                ctx.print_color(4, 5,  hd,  bg, "SLIDE 5 — QUICK REFERENCE");
                ctx.print_color(4, 7,  sel, bg, "Floor navigation:");
                ctx.print_color(4, 8,  ac,  bg, "  Enter/E  Enter room        D  Descend to next floor");
                ctx.print_color(4, 9,  ac,  bg, "  C        Character sheet   B  Body chart");
                ctx.print_color(4, 10, ac,  bg, "  P        Passive tree      N  Auto-allocate points");
                ctx.print_color(4, 11, ac,  bg, "  F5       Save game         Z  Auto-pilot");
                ctx.print_color(4, 13, sel, bg, "Combat:");
                ctx.print_color(4, 14, ac,  bg, "  A  Attack   S  Spell   D  Defend   H  Heal");
                ctx.print_color(4, 15, ac,  bg, "  I  Use item  F  Flee   Q  Quit to title");
                ctx.print_color(4, 17, sel, bg, "The chaos engine CHAIN is shown at the top of the combat log.");
                ctx.print_color(4, 18, dim, bg, "Each link is: EngineName(output_value)");
                ctx.print_color(4, 19, dim, bg, "Watch for CRIT bursts — they chain-multiply across all links.");
                ctx.print_color(4, 21, sel, bg, "Pro tips:");
                ctx.print_color(4, 22, dim, bg, "  ● Engine nodes amplify chain variance — high risk / reward.");
                ctx.print_color(4, 23, dim, bg, "  ● Keystone 'Pure Chaos' removes all caps. Yes, all of them.");
                ctx.print_color(4, 24, dim, bg, "  ● If a body part says MATH.ABSENT — you are doing great.");
            }
        }

        // ── Navigation footer ─────────────────────────────────────────────
        draw_separator(ctx, 2, 74, 155, &t);
        print_hint(ctx, 4,  75, "←→/Space", " Navigate slides  ", &t);
        print_hint(ctx, 36, 75, "Esc",      " Back to title",     &t);

        if slide < TOTAL_SLIDES {
            let next_flash = if (self.frame / 20) % 2 == 0 { ac } else { hd };
            ctx.print_color(56, 75, next_flash, bg, "► Next slide");
        } else {
            ctx.print_color(56, 75, succ, bg, "► Press Enter to play!");
        }

        ctx.print_color(4, 77, dim, bg, &format!("Slide {}/{} — press ? on title to reopen", slide, TOTAL_SLIDES));
    }

    // ── PASSIVE TREE ──────────────────────────────────────────────────────────

    fn draw_passive_tree(&mut self, ctx: &mut BTerm) {
        use chaos_rpg_core::passive_tree::{nodes, NodeType};

        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);
        let mna = RGB::from_u8(t.mana.0,   t.mana.1,   t.mana.2);

        let p = match &self.player { Some(p) => p.clone(), None => { self.screen = AppScreen::FloorNav; return; } };
        let allocated_set: std::collections::HashSet<u32> = p.allocated_nodes.iter().cloned().collect();

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "PASSIVE TREE", &t);

        // Header bar
        let sp = p.skill_points;
        let sp_col = if sp > 0 { gld } else { dim };
        let header = format!("{} — {} pts available — {} nodes allocated",
            p.class.name(), sp, allocated_set.len());
        ctx.print_color(2, 1, hd, bg, &header.chars().take(76).collect::<String>());
        if sp > 0 {
            let pulse = (self.frame / 12) % 2 == 0;
            let pc = if pulse { gld } else { RGB::from_u8(t.gold.0/2+20, t.gold.1/2+10, 10) };
            ctx.print_color(60, 1, pc, bg, &format!("[N] Spend {} pts", sp));
        }
        draw_separator(ctx, 1, 2, 77, &t);

        // Build node list: allocated first, then available, then locked
        let all_nodes = nodes();
        // Filter to nodes that belong to player's class or are universal
        let class_nodes: Vec<&chaos_rpg_core::passive_tree::TreeNode> = all_nodes.iter()
            .filter(|n| n.class_start == Some(p.class) || n.class_start.is_none())
            .collect();

        // Categorize
        let mut allocated_nodes: Vec<&chaos_rpg_core::passive_tree::TreeNode> = Vec::new();
        let mut available_nodes: Vec<&chaos_rpg_core::passive_tree::TreeNode> = Vec::new();
        let mut locked_nodes:    Vec<&chaos_rpg_core::passive_tree::TreeNode> = Vec::new();

        // Build temporary PlayerPassives to check can_allocate
        use chaos_rpg_core::passive_tree::PlayerPassives;
        let tmp_passives = PlayerPassives {
            allocated: p.allocated_nodes.iter().map(|&id| id as u16).collect(),
            points: p.skill_points,
            ..Default::default()
        };

        for node in &class_nodes {
            if allocated_set.contains(&(node.id as u32)) {
                allocated_nodes.push(node);
            } else if tmp_passives.can_allocate(node.id) {
                available_nodes.push(node);
            } else {
                locked_nodes.push(node);
            }
        }

        // Combine: available first (actionable), then allocated, then locked
        let mut display: Vec<(u8, &chaos_rpg_core::passive_tree::TreeNode)> = Vec::new(); // 0=avail,1=alloc,2=locked
        for n in &available_nodes { display.push((0, n)); }
        for n in &allocated_nodes { display.push((1, n)); }
        for n in &locked_nodes    { display.push((2, n)); }

        // Clamp scroll
        let rows_per_page = 66usize;
        let max_scroll = display.len().saturating_sub(rows_per_page);
        if self.passive_scroll > max_scroll { self.passive_scroll = max_scroll; }

        // Section separators
        let avail_count = available_nodes.len();
        let alloc_count = allocated_nodes.len();

        // Column headers
        ctx.print_color(2,  3, ac,  bg, "Status");
        ctx.print_color(11, 3, ac,  bg, "Node Name");
        ctx.print_color(36, 3, ac,  bg, "Type");
        ctx.print_color(51, 3, ac,  bg, "Effect");
        draw_separator(ctx, 1, 4, 77, &t);

        let visible = &display[self.passive_scroll..display.len().min(self.passive_scroll + rows_per_page)];
        for (i, (cat, node)) in visible.iter().enumerate() {
            let y = 5 + i as i32;

            // Section header rows
            let global_idx = self.passive_scroll + i;
            if global_idx == 0 && avail_count > 0 {
                ctx.print_color(2, y, suc, bg, &format!("-- AVAILABLE TO ALLOCATE ({}) --", avail_count));
                continue;
            }
            if global_idx == avail_count && alloc_count > 0 {
                ctx.print_color(2, y, mna, bg, &format!("-- ALLOCATED ({}) --", alloc_count));
                continue;
            }
            if global_idx == avail_count + alloc_count {
                ctx.print_color(2, y, dim, bg, &format!("-- LOCKED ({}) — allocate prerequisites first --", locked_nodes.len()));
                continue;
            }

            let (status_str, status_col) = match cat {
                0 => ("  [open]", suc),
                1 => ("  [done]", mna),
                _ => ("  [lock]", dim),
            };
            ctx.print_color(2, y, status_col, bg, status_str);

            // Node name (truncated to 22 chars)
            let name_col = match cat { 0 => hd, 1 => mna, _ => dim };
            ctx.print_color(11, y, name_col, bg, &node.name.chars().take(22).collect::<String>());

            // Type tag + color
            let (type_tag, type_col) = match &node.node_type {
                NodeType::Stat { stat, min, max } => (format!("Stat/{}", stat.chars().take(5).collect::<String>()), ac),
                NodeType::Notable { stat, bonus, .. } => (format!("Notable {:+}", bonus), gld),
                NodeType::Keystone { .. } => ("KEYSTONE".to_string(), dng),
                NodeType::Engine { engine, .. } => (format!("Engine/{}", engine.chars().take(6).collect::<String>()), RGB::from_u8(180, 80, 255)),
                NodeType::Synergy { cluster, .. } => (format!("Syn #{}", cluster), RGB::from_u8(100, 200, 180)),
            };
            ctx.print_color(36, y, type_col, bg, &type_tag.chars().take(13).collect::<String>());

            // Short desc
            ctx.print_color(51, y, dim, bg, &node.short_desc.chars().take(26).collect::<String>());
        }

        // Scroll indicator
        if display.len() > rows_per_page {
            let scroll_pct = self.passive_scroll * 100 / display.len().max(1);
            ctx.print_color(2, 73, dim, bg, &format!("Showing {}-{} of {}  ({}% scrolled)",
                self.passive_scroll + 1,
                (self.passive_scroll + rows_per_page).min(display.len()),
                display.len(), scroll_pct));
        }

        draw_separator(ctx, 1, 74, 157, &t);
        print_hint(ctx, 2,  75, "[Up/Dn]", " Scroll  ", &t);
        print_hint(ctx, 22, 75, "[PgUp/PgDn]", " Page  ", &t);
        if sp > 0 { print_hint(ctx, 46, 75, "[N]", " Auto-allocate all points  ", &t); }
        print_hint(ctx, 2,  76, "[Esc/C]", " Back to Sheet", &t);

        // Render allocation burst particles
        for p in &mut self.particles { p.step(); }
        self.particles.retain(|p| p.alive());
        if self.config.visuals.enable_particles {
            for p in &self.particles {
                let rc = p.render_col();
                let px = p.x as i32; let py = p.y as i32;
                if py < 3 || py > 73 || px < 2 || px > 157 { continue; }
                ctx.print_color(px, py, RGB::from_u8(rc.0, rc.1, rc.2), bg, &p.text);
            }
        }
    }

    // ── BODY CHART ────────────────────────────────────────────────────────────

    fn draw_body_chart(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        let p = match &self.player { Some(p) => p.clone(), None => { self.screen = AppScreen::FloorNav; return; } };

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "", &t);
        print_center(ctx, 0, 1, 159, t.heading, &t, "BODY CONDITION");
        draw_separator(ctx, 1, 2, 157, &t);

        // Combat summary at top
        let summary = p.body.combat_summary();
        ctx.print_color(2, 3, if summary.contains("CRITICAL") || summary.contains("SEVERED") { dng } else { dim }, bg, &summary.chars().take(155).collect::<String>());

        // Two-column body part display with visual HP bars
        draw_subpanel(ctx, 1, 5, 77, 66, "BODY PARTS", &t);

        use chaos_rpg_core::body::BodyPart;
        let col_parts: &[&[BodyPart]] = &[
            &[BodyPart::Head, BodyPart::Torso, BodyPart::Neck,
              BodyPart::LeftArm, BodyPart::RightArm,
              BodyPart::LeftHand, BodyPart::RightHand],
            &[BodyPart::LeftLeg, BodyPart::RightLeg,
              BodyPart::LeftFoot, BodyPart::RightFoot,
              BodyPart::LeftEye, BodyPart::RightEye],
        ];
        for (col, parts) in col_parts.iter().enumerate() {
            let cx = 3 + col as i32 * 38;
            for (row, &part) in parts.iter().enumerate() {
                let ry = 7 + row as i32 * 4;
                let state = p.body.parts.get(&part);
                let (cur, max_hp, sev) = state
                    .map(|s| (s.current_hp, s.max_hp, s.injury.as_ref().map(|i| i.name())))
                    .unwrap_or((10, 10, None));
                let pct = if max_hp > 0 { cur as f32 / max_hp as f32 } else { 0.0 };
                let bar_col = t.hp_color(pct.clamp(0.0, 1.0));
                let sev_lbl = sev.unwrap_or("Healthy");
                let fg = if pct <= 0.0 { dng }
                         else if pct < 0.35 { dng }
                         else if pct < 0.65 { RGB::from_u8(200, 130, 40) }
                         else { suc };
                // Part name + HP numbers
                let name_str = part.name();
                ctx.print_color(cx, ry,     hd,  bg, &format!("{:<12}", name_str));
                ctx.print_color(cx + 12, ry, fg, bg, &format!("{}/{}", cur, max_hp));
                // Severity label
                ctx.print_color(cx + 22, ry, fg, bg, sev_lbl);
                // HP bar (width 32)
                draw_bar_gradient(ctx, cx, ry + 1, 32, cur.max(0), max_hp.max(1), bar_col, t.muted, &t);
                // Armor note if equipped
                if let Some(state) = p.body.parts.get(&part) {
                    if state.armor_defense > 0 {
                        ctx.print_color(cx + 20, ry + 2, dim, bg,
                            &format!("DEF+{}", state.armor_defense));
                    }
                }
            }
        }

        // Right panel: overall summary + stats affected
        draw_subpanel(ctx, 80, 5, 77, 66, "BODY SUMMARY", &t);
        let total_parts = p.body.parts.len();
        let injured: usize = p.body.parts.values().filter(|s| s.injury.is_some()).count();
        let severed: usize = p.body.parts.values()
            .filter(|s| s.injury.as_ref().map(|i| i.name() == "MATH.ABSENT").unwrap_or(false))
            .count();
        ctx.print_color(82, 7, hd, bg, &format!("Parts: {}/{} healthy", total_parts - injured, total_parts));
        if injured > 0 {
            ctx.print_color(82, 8, if severed > 0 { dng } else { RGB::from_u8(200,130,40) }, bg,
                &format!("{} injured{}",
                    injured,
                    if severed > 0 { format!(", {} SEVERED", severed) } else { String::new() }));
        }
        draw_separator(ctx, 81, 9, 75, &t);
        ctx.print_color(82, 10, hd, bg, "Worst injuries:");
        let mut wr = 11i32;
        for (part, state) in &p.body.parts {
            if let Some(ref inj) = state.injury {
                let pct = if state.max_hp > 0 { state.current_hp as f32 / state.max_hp as f32 } else { 0.0 };
                let fc = if pct <= 0.0 { dng } else { RGB::from_u8(200,130,40) };
                ctx.print_color(82, wr, fc, bg,
                    &format!("{:<12} — {}", part.name().chars().take(12).collect::<String>(), inj.name()));
                wr += 1;
                if wr > 26 { break; }
            }
        }
        if wr == 11 {
            ctx.print_color(82, 11, suc, bg, "No injuries — impressive.");
        }
        draw_separator(ctx, 81, 28, 75, &t);
        ctx.print_color(82, 29, hd, bg, "Armor defense by part:");
        let mut ar = 30i32;
        for (part, state) in &p.body.parts {
            if state.armor_defense > 0 {
                ctx.print_color(82, ar, dim, bg,
                    &format!("{:<12} DEF+{}", part.name().chars().take(12).collect::<String>(), state.armor_defense));
                ar += 1;
                if ar > 50 { break; }
            }
        }
        if ar == 30 {
            ctx.print_color(82, 30, dim, bg, "No armor equipped.");
        }

        draw_separator(ctx, 1, 74, 157, &t);
        print_hint(ctx, 2, 75, "[Esc]", " Back to floor", &t);
        print_hint(ctx, 18, 75, "[C]", " Character Sheet", &t);
        let summary2 = p.body.combat_summary();
        if summary2.contains("CRITICAL") || summary2.contains("SEVERED") {
            ctx.print_color(42, 75, dng, bg, &format!("⚠ {}", &summary2.chars().take(70).collect::<String>()));
        }
    }

    // ── GAME OVER ─────────────────────────────────────────────────────────────

    fn draw_game_over(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "", &t);

        // ── Flashing "YOU DIED" banner — centered in full 160-col screen ──
        let pulse = if (self.frame / 30) % 2 == 0 { dng } else { hd };
        ctx.print_color(57, 2, pulse, bg, "╔══════════════════════════════════════════╗");
        ctx.print_color(57, 3, pulse, bg, "║         Y  O  U     D  I  E  D           ║");
        ctx.print_color(57, 4, dng,   bg, "║     The mathematics have consumed you.   ║");
        ctx.print_color(57, 5, pulse, bg, "╚══════════════════════════════════════════╝");

        draw_separator(ctx, 1, 7, 157, &t);

        if let Some(ref p) = self.player {
            // ── LEFT PANEL: identity + stats ──────────────────────────────
            draw_subpanel(ctx, 1, 8, 77, 58, "RUN SUMMARY", &t);

            ctx.print_color(3, 10, hd, bg,
                &format!("{} · {} · Lv.{} · Floor {}", p.name, p.class.name(), p.level, p.floor));
            let cause: String = p.run_stats.cause_of_death.chars().take(70).collect();
            ctx.print_color(3, 11, dng, bg, &format!("☠  {}", cause));

            draw_separator(ctx, 2, 12, 75, &t);

            // Stats grid — two sub-columns
            stat_line(ctx, 3, 13, "Kills    ", &format!("{}", p.kills),       t.success, &t);
            stat_line(ctx, 3, 14, "Gold     ", &format!("{}g", p.gold),       t.gold,    &t);
            stat_line(ctx, 3, 15, "XP       ", &format!("{}", p.xp),          t.xp,      &t);
            stat_line(ctx, 3, 16, "Spells   ", &format!("{}", p.spells_cast), t.mana,    &t);
            stat_line(ctx, 3, 17, "Corrupt  ", &format!("{}", p.corruption),  t.danger,  &t);

            let dealt = p.run_stats.damage_dealt;
            let taken = p.run_stats.damage_taken;
            let ratio = if taken > 0 { dealt as f64 / taken as f64 } else { dealt as f64 };
            let ratio_col = if ratio >= 2.0 { t.success } else if ratio >= 1.0 { t.gold } else { t.danger };
            stat_line(ctx, 40, 13, "Dmg Dealt ", &format!("{}", dealt),         t.success,  &t);
            stat_line(ctx, 40, 14, "Dmg Taken ", &format!("{}", taken),         t.danger,   &t);
            stat_line(ctx, 40, 15, "D/T Ratio ", &format!("{:.2}", ratio),      ratio_col,  &t);
            if p.run_stats.final_blow_damage > 0 {
                stat_line(ctx, 40, 16, "Final Blow", &format!("{}", p.run_stats.final_blow_damage), t.danger, &t);
            }
            if p.run_stats.highest_single_hit > 0 {
                stat_line(ctx, 40, 17, "Best Hit  ", &format!("{}", p.run_stats.highest_single_hit), t.gold, &t);
            }
            if p.misery.misery_index >= 100.0 {
                stat_line(ctx, 40, 18, "Misery    ", &format!("{:.0}", p.misery.misery_index), t.warn, &t);
            }

            draw_separator(ctx, 2, 19, 75, &t);
            // Run summary log — up to 44 lines
            for (i, line) in p.run_summary().iter().enumerate().take(44) {
                ctx.print_color(3, 20 + i as i32, dim, bg, &line.chars().take(72).collect::<String>());
            }

            // ── RIGHT PANEL: run narrative / final events ──────────────────
            draw_subpanel(ctx, 80, 8, 78, 58, "FINAL EVENTS", &t);

            // Cause of death headline
            ctx.print_color(82, 10, dng, bg, "Cause of death:");
            ctx.print_color(82, 11, hd,  bg, &cause.chars().take(74).collect::<String>());

            draw_separator(ctx, 81, 13, 76, &t);

            // Recent combat log (last events before death)
            ctx.print_color(82, 14, ac, bg, "Last combat events:");
            let log_start = self.combat_log.len().saturating_sub(48);
            for (i, line) in self.combat_log[log_start..].iter().enumerate().take(48) {
                let y = 15 + i as i32;
                if y > 63 { break; }
                let line_s: String = line.chars().take(74).collect();
                ctx.print_color(82, y, dim, bg, &line_s);
            }
        }

        if let Some(ref nem) = self.nemesis_record {
            ctx.print_color(2, 68, dng, bg,
                &format!("☠ {} is now your Nemesis — will return stronger.", &nem.enemy_name.chars().take(50).collect::<String>()));
        }

        draw_separator(ctx, 1, 72, 157, &t);
        print_hint(ctx, 4,  73, "[Enter]", " Return to title   ", &t);
        print_hint(ctx, 28, 73, "[S]",     " Scoreboard",         &t);
        print_hint(ctx, 42, 73, "[H]",     " Run History",        &t);
    }

    // ── VICTORY ───────────────────────────────────────────────────────────────

    fn draw_victory(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let suc = RGB::from_u8(t.success.0,t.success.1,t.success.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "", &t);

        // ── Animated victory banner — centered in 160-col screen ──
        let shimmer_t = (self.frame as f32 * 0.05).sin() * 0.2 + 0.8;
        let sc = Theme::lerp(t.gold, t.heading, shimmer_t);
        let shimmer = RGB::from_u8(sc.0, sc.1, sc.2);

        ctx.print_color(55, 2, shimmer, bg, "╔═════════════════════════════════════════════════╗");
        ctx.print_color(55, 3, shimmer, bg, "║   ★  ★  V  I  C  T  O  R  Y  ★  ★             ║");
        ctx.print_color(55, 4, gld,     bg, "║   You solved 10 floors of pure mathematical     ║");
        ctx.print_color(55, 5, gld,     bg, "║   chaos. The algorithms bow before you.         ║");
        ctx.print_color(55, 6, shimmer, bg, "╚═════════════════════════════════════════════════╝");

        draw_separator(ctx, 1, 8, 157, &t);

        if let Some(ref p) = self.player {
            // ── LEFT PANEL: final stats ────────────────────────────────────
            draw_subpanel(ctx, 1, 9, 77, 58, "FINAL STATS", &t);

            ctx.print_color(3, 11, hd, bg,
                &format!("{} · {} · Lv.{}", p.name, p.class.name(), p.level));

            draw_separator(ctx, 2, 12, 75, &t);

            stat_line(ctx, 3, 13, "Floors    ", &format!("{}", p.floor),        t.gold,    &t);
            stat_line(ctx, 3, 14, "Kills     ", &format!("{}", p.kills),        t.success, &t);
            stat_line(ctx, 3, 15, "Gold      ", &format!("{}g", p.gold),        t.gold,    &t);
            stat_line(ctx, 3, 16, "XP        ", &format!("{}", p.xp),           t.xp,      &t);
            stat_line(ctx, 3, 17, "Spells    ", &format!("{}", p.spells_cast),  t.mana,    &t);
            stat_line(ctx, 3, 18, "Corrupt   ", &format!("{}", p.corruption),   t.danger,  &t);

            let dealt = p.run_stats.damage_dealt;
            let taken = p.run_stats.damage_taken;
            let ratio = if taken > 0 { dealt as f64 / taken as f64 } else { dealt as f64 };
            let ratio_col = if ratio >= 2.0 { t.success } else if ratio >= 1.0 { t.gold } else { t.danger };
            stat_line(ctx, 40, 13, "Dmg Dealt ", &format!("{}", dealt),       t.success, &t);
            stat_line(ctx, 40, 14, "Dmg Taken ", &format!("{}", taken),       t.danger,  &t);
            stat_line(ctx, 40, 15, "D/T Ratio ", &format!("{:.2}", ratio),    ratio_col, &t);
            if p.run_stats.highest_single_hit > 0 {
                stat_line(ctx, 40, 16, "Best Hit  ", &format!("{}", p.run_stats.highest_single_hit), t.gold, &t);
            }

            draw_separator(ctx, 2, 19, 75, &t);

            // Run summary log — up to 44 lines
            for (i, line) in p.run_summary().iter().enumerate().take(44) {
                let c = if i == 0 { suc } else { dim };
                ctx.print_color(3, 20 + i as i32, c, bg, &line.chars().take(72).collect::<String>());
            }

            // ── RIGHT PANEL: narrative / combat log ────────────────────────
            draw_subpanel(ctx, 80, 9, 78, 58, "COMBAT LOG", &t);

            ctx.print_color(82, 11, ac, bg, "Final battle events:");
            draw_separator(ctx, 81, 12, 76, &t);

            let log_start = self.combat_log.len().saturating_sub(54);
            for (i, line) in self.combat_log[log_start..].iter().enumerate().take(54) {
                let y = 13 + i as i32;
                if y > 64 { break; }
                ctx.print_color(82, y, dim, bg, &line.chars().take(74).collect::<String>());
            }
        }

        draw_separator(ctx, 1, 72, 157, &t);
        print_hint(ctx, 4,  73, "[Enter]", " Return to title   ", &t);
        print_hint(ctx, 28, 73, "[S]",     " Scoreboard",         &t);
        print_hint(ctx, 42, 73, "[H]",     " Run History",        &t);
    }

    // ── SCOREBOARD ────────────────────────────────────────────────────────────

    fn draw_scoreboard(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,     t.bg.1,     t.bg.2);
        let hd  = RGB::from_u8(t.heading.0,t.heading.1,t.heading.2);
        let ac  = RGB::from_u8(t.accent.0, t.accent.1, t.accent.2);
        let dim = RGB::from_u8(t.dim.0,    t.dim.1,    t.dim.2);
        let gld = RGB::from_u8(t.gold.0,   t.gold.1,   t.gold.2);
        let dng = RGB::from_u8(t.danger.0, t.danger.1, t.danger.2);
        let wrn = RGB::from_u8(t.warn.0,   t.warn.1,   t.warn.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "SCOREBOARD", &t);

        // ── Hall of Chaos (regular scores) — left side ──
        ctx.print_color(2, 2, hd, bg, "HALL OF CHAOS");
        let scores = load_scores();
        if scores.is_empty() {
            ctx.print_color(4, 4, dim, bg, "No scores yet. Play and die bravely.");
        } else {
            ctx.print_color(2, 3, dim, bg,
                &format!("{:<4} {:<10} {:<16} {:<12} {:<5} {:<5}",
                    "Rank", "Score", "Name", "Class", "Flr", "Kills"));
            draw_separator(ctx, 2, 4, 75, &t);
            for (i, s) in scores.iter().enumerate().take(30) {
                let row_col = match i { 0 => gld, 1 => hd, 2 => ac, _ => dim };
                let medal = match i { 0 => "★ ", 1 => "◆ ", 2 => "● ", _ => "  " };
                ctx.print_color(2, 5 + i as i32, row_col, bg,
                    &format!("{}{:<3}  {:<10} {:<16} {:<12} {:<5} {}",
                        medal, i+1, s.score, &s.name.chars().take(16).collect::<String>(),
                        &s.class.chars().take(12).collect::<String>(),
                        s.floor_reached, s.enemies_defeated));
            }
        }

        // ── Hall of Misery ──
        let misery_y = 37i32;
        draw_separator(ctx, 2, misery_y - 1, 75, &t);
        ctx.print_color(2, misery_y, dng, bg, "HALL OF MISERY");
        let mscores = load_misery_scores();
        if mscores.is_empty() {
            ctx.print_color(4, misery_y + 2, dim, bg, "No misery recorded. Suffer more.");
        } else {
            ctx.print_color(2, misery_y + 1, dim, bg,
                &format!("{:<4} {:<8} {:<14} {:<12} {:<5} {:<18}",
                    "Rank", "Misery", "Name", "Class", "Flr", "Cause of death"));
            draw_separator(ctx, 2, misery_y + 2, 75, &t);
            for (i, m) in mscores.iter().enumerate().take(30) {
                let row_col = match i { 0 => dng, 1 => wrn, 2 => ac, _ => dim };
                let medal = match i { 0 => "☠ ", 1 => "✦ ", 2 => "● ", _ => "  " };
                let cause: String = m.cause_of_death.chars().take(18).collect();
                ctx.print_color(2, misery_y + 3 + i as i32, row_col, bg,
                    &format!("{}{:<3}  {:<8.0} {:<14} {:<12} {:<5} {}",
                        medal, i+1, m.misery_index,
                        &m.name.chars().take(14).collect::<String>(),
                        &m.class.chars().take(12).collect::<String>(),
                        m.floor_reached, cause));
            }
        }

        // ── Right panel: extended chaos scores ──
        draw_subpanel(ctx, 80, 2, 77, 70, "EXTENDED LEADERBOARD", &t);
        ctx.print_color(82, 3, hd, bg, "All-time best scores:");
        draw_separator(ctx, 81, 4, 75, &t);
        if scores.is_empty() {
            ctx.print_color(82, 6, dim, bg, "No scores recorded yet.");
        } else {
            ctx.print_color(82, 5, dim, bg,
                &format!("{:<4} {:<10} {:<16} {:<12} {:<5} {:<5} Mode",
                    "Rank", "Score", "Name", "Class", "Flr", "Kills"));
            for (i, s) in scores.iter().enumerate() {
                let row_col = match i { 0 => gld, 1 => hd, 2 => ac, _ => dim };
                let medal = match i { 0 => "★ ", 1 => "◆ ", 2 => "● ", _ => "  " };
                ctx.print_color(82, 6 + i as i32, row_col, bg,
                    &format!("{}{:<3}  {:<10} {:<16} {:<12} {:<5} {}",
                        medal, i+1, s.score,
                        &s.name.chars().take(16).collect::<String>(),
                        &s.class.chars().take(12).collect::<String>(),
                        s.floor_reached, s.enemies_defeated));
                if 6 + i as i32 > 70 { break; }
            }
        }

        draw_separator(ctx, 2, 74, 155, &t);
        print_hint(ctx, 4, 75, "[Esc/Q]", " Back to title", &t);
    }
}

// ─── INPUT HANDLER ────────────────────────────────────────────────────────────

impl State {
    // ── AUTO-PLAY TICK ─────────────────────────────────────────────────────────
    //
    // Fires once per AUTO_DELAY frames.  Handles floor navigation, combat AI,
    // and non-item room resolution automatically.  Pauses on:
    //   • Item pickup prompts (the player still decides what to keep)
    //   • Shop / Crafting screens (player manages gold/items)
    //   • GameOver / Victory (nothing to advance)
    //
    fn tick_auto_play(&mut self, _ctx: &mut BTerm) {
        const AUTO_DELAY: u64 = 60; // ~1 s at 60 fps
        if self.frame.saturating_sub(self.auto_last_action) < AUTO_DELAY {
            return;
        }

        match self.screen.clone() {
            // ── Floor navigation ─────────────────────────────────────────────
            AppScreen::FloorNav => {
                // Auto-allocate any pending skill points first
                if self.player.as_ref().map(|p| p.skill_points > 0).unwrap_or(false) {
                    let seed = self.floor_seed.wrapping_add(self.frame);
                    if let Some(ref mut p) = self.player {
                        let msgs = p.auto_allocate_passives(seed);
                        for m in msgs { self.push_log(m); }
                    }
                }

                if self.floor.as_ref().map(|f| f.rooms_remaining() == 0).unwrap_or(false) {
                    // All rooms done — descend
                    if self.floor_num >= self.max_floor {
                        self.save_score_now();
                        self.screen = AppScreen::Victory;
                    } else {
                        self.floor_num += 1;
                        self.generate_floor_for_current();
                    }
                } else {
                    self.enter_current_room();
                }
                self.auto_last_action = self.frame;
            }

            // ── Combat — player always picks manually, even in auto mode ─────
            AppScreen::Combat => {
                // Do nothing; input is handled in draw_combat key handler.
            }

            // ── Non-item room events — auto-accept ───────────────────────────
            AppScreen::RoomView => {
                let has_item = self.room_event.pending_item.is_some();
                if has_item {
                    // Pause so the player can decide
                    return;
                }
                // Portal: skip through automatically (risky but exciting)
                if self.room_event.portal_available {
                    self.room_event.portal_available = false;
                    if self.floor_num >= self.max_floor {
                        self.save_score_now();
                        self.screen = AppScreen::Victory;
                    } else {
                        self.floor_num += 1;
                        self.generate_floor_for_current();
                        self.screen = AppScreen::FloorNav;
                    }
                } else {
                    // Auto-accept all other room events (shrine bless, trap damage, etc.)
                    self.room_event.pending_spell = None;
                    self.advance_floor_room();
                    if self.screen != AppScreen::GameOver && self.screen != AppScreen::Victory {
                        self.screen = AppScreen::FloorNav;
                    }
                }
                self.auto_last_action = self.frame;
            }

            // Pause on screens where the player makes deliberate choices
            AppScreen::Shop
            | AppScreen::Crafting
            | AppScreen::GameOver
            | AppScreen::Victory => {}

            _ => {}
        }
    }

    /// Choose a combat action using a simple HP-aware strategy:
    ///   HP < 25%          → Defend
    ///   MP > 30% & spells → cast best available spell
    ///   otherwise         → Attack
    fn auto_combat_action(&self) -> CombatAction {
        let (hp, max_hp, mp, max_mp, spell_count) = match &self.player {
            Some(p) => (
                p.current_hp, p.max_hp,
                self.current_mana, self.max_mana(),
                p.known_spells.len(),
            ),
            None => return CombatAction::Attack,
        };

        let hp_pct = hp as f32 / max_hp.max(1) as f32;
        if hp_pct < 0.25 {
            return CombatAction::Defend;
        }

        // Use a spell when mana is plentiful
        let mp_pct = mp as f32 / max_mp.max(1) as f32;
        if spell_count > 0 && mp_pct > 0.30 {
            return CombatAction::UseSpell(0);
        }

        CombatAction::Attack
    }

    fn handle_input(&mut self, ctx: &mut BTerm) {
        let key = match ctx.key { Some(k) => k, None => return };

        match self.screen.clone() {
            AppScreen::Title => match key {
                VirtualKeyCode::Up   => self.selected_menu = self.selected_menu.saturating_sub(1),
                VirtualKeyCode::Down => {
                    // Total menu items: (Continue if save) + New Game + 6 progress + 3 settings = 10 or 11
                    let max = if self.save_exists { 10 } else { 9 };
                    self.selected_menu = (self.selected_menu + 1).min(max);
                }
                VirtualKeyCode::Return => {
                    // Build same option list as draw_title
                    let mut opts: Vec<u8> = Vec::new(); // group ids for ordered matching
                    if self.save_exists { opts.push(0); } // Continue
                    opts.push(1); // New Game
                    opts.push(2); // Achievements
                    opts.push(3); // Bestiary
                    opts.push(4); // Codex
                    opts.push(5); // History
                    opts.push(6); // Daily
                    opts.push(7); // Scoreboard
                    opts.push(8); // Options
                    opts.push(9); // Tutorial
                    opts.push(10); // Quit
                    let sel = self.selected_menu.min(opts.len() - 1);
                    match opts[sel] {
                        0 => { if self.save_exists { self.do_load(); } }
                        1 => self.screen = AppScreen::ModeSelect,
                        2 => { self.achievement_scroll = 0; self.achievement_filter = 0; self.screen = AppScreen::Achievements; }
                        3 => { self.bestiary_scroll = 0; self.bestiary_selected = 0; self.screen = AppScreen::Bestiary; }
                        4 => { self.codex_scroll = 0; self.codex_selected = 0; self.screen = AppScreen::Codex; }
                        5 => { self.history_scroll = 0; self.screen = AppScreen::RunHistory; }
                        6 => {
                            self.daily_rows.clear();
                            self.daily_status = "Fetching leaderboard...".to_string();
                            self.screen = AppScreen::DailyLeaderboard;
                            let url = self.config.leaderboard.url.clone();
                            let date = chrono_date_simple();
                            if self.config.leaderboard.fetch_on_open {
                                match fetch_scores(&url, &date) {
                                    Ok(rows) => { self.daily_status = format!("Updated — {} entries", rows.len()); self.daily_rows = rows; }
                                    Err(e) => self.daily_status = format!("Fetch error: {}", &e.chars().take(40).collect::<String>()),
                                }
                            }
                        }
                        7 => self.screen = AppScreen::Scoreboard,
                        8 => self.screen = AppScreen::Settings,
                        9 => { self.tutorial_slide = 1; self.screen = AppScreen::Tutorial; }
                        _ => ctx.quit(),
                    }
                }
                VirtualKeyCode::L => { if self.save_exists { self.do_load(); } }
                VirtualKeyCode::T => self.cycle_theme(),
                VirtualKeyCode::O => self.screen = AppScreen::Settings,
                VirtualKeyCode::Q => ctx.quit(),
                VirtualKeyCode::Slash | VirtualKeyCode::F1 => {
                    self.tutorial_slide = 1;
                    self.screen = AppScreen::Tutorial;
                }
                VirtualKeyCode::J => { self.achievement_scroll = 0; self.achievement_filter = 0; self.screen = AppScreen::Achievements; }
                VirtualKeyCode::H => { self.history_scroll = 0; self.screen = AppScreen::RunHistory; }
                VirtualKeyCode::B => { self.bestiary_scroll = 0; self.bestiary_selected = 0; self.screen = AppScreen::Bestiary; }
                VirtualKeyCode::X => { self.codex_scroll = 0; self.codex_selected = 0; self.screen = AppScreen::Codex; }
                VirtualKeyCode::D => {
                    self.daily_rows.clear();
                    self.daily_status = "Fetching leaderboard...".to_string();
                    self.screen = AppScreen::DailyLeaderboard;
                    let url  = self.config.leaderboard.url.clone();
                    let date = chrono_date_simple();
                    if self.config.leaderboard.fetch_on_open {
                        match fetch_scores(&url, &date) {
                            Ok(rows) => {
                                self.daily_status = format!("Updated — {} entries", rows.len());
                                self.daily_rows = rows;
                            }
                            Err(e) => self.daily_status = format!("Fetch error: {}", &e.chars().take(40).collect::<String>()),
                        }
                    }
                }
                _ => {}
            },

            AppScreen::ModeSelect => match key {
                VirtualKeyCode::Up   => self.mode_cursor = self.mode_cursor.saturating_sub(1),
                VirtualKeyCode::Down => self.mode_cursor = (self.mode_cursor + 1).min(2),
                VirtualKeyCode::Return => {
                    self.game_mode = match self.mode_cursor { 0 => GameMode::Story, 1 => GameMode::Infinite, _ => GameMode::Daily };
                    self.screen = AppScreen::CharacterCreation;
                }
                VirtualKeyCode::Escape => self.screen = AppScreen::Title,
                _ => {}
            },

            AppScreen::CharacterCreation => {
                if self.cc_name_active {
                    match key {
                        VirtualKeyCode::Return | VirtualKeyCode::Escape => {
                            self.cc_name_active = false;
                        }
                        VirtualKeyCode::Back => { self.cc_name.pop(); }
                        _ => {
                            // Capture typed characters
                            if let Some(c) = ctx.key.and_then(|k| key_to_char(k, ctx.shift)) {
                                if self.cc_name.len() < 20 && (c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '\'') {
                                    self.cc_name.push(c);
                                }
                            }
                        }
                    }
                } else { match key {
                VirtualKeyCode::Up    => self.cc_class = self.cc_class.saturating_sub(1),
                VirtualKeyCode::Down  => self.cc_class = (self.cc_class + 1).min(CLASSES.len() - 1),
                VirtualKeyCode::Left  => self.cc_bg = self.cc_bg.saturating_sub(1),
                VirtualKeyCode::Right => self.cc_bg = (self.cc_bg + 1).min(BACKGROUNDS.len() - 1),
                VirtualKeyCode::Tab   => self.cc_diff = (self.cc_diff + 1) % DIFFICULTIES.len(),
                VirtualKeyCode::N     => { self.cc_name_active = true; }
                VirtualKeyCode::Return => {
                    self.boon_options = Boon::random_three(self.seed.wrapping_add(self.cc_class as u64 * 777));
                    self.boon_cursor = 0;
                    self.screen = AppScreen::BoonSelect;
                }
                VirtualKeyCode::Escape => self.screen = AppScreen::ModeSelect,
                _ => {}
            } }  // close inner match + else block
            }    // close AppScreen::CharacterCreation block

            AppScreen::BoonSelect => match key {
                VirtualKeyCode::Up   => { self.boon_cursor = self.boon_cursor.saturating_sub(1); self.emit_audio(AudioEvent::MenuNavigate); }
                VirtualKeyCode::Down => { self.boon_cursor = (self.boon_cursor + 1).min(2); self.emit_audio(AudioEvent::MenuNavigate); }
                VirtualKeyCode::Key1 => { self.boon_cursor = 0; self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Key2 => { self.boon_cursor = 1; self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Key3 => { self.boon_cursor = 2; self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Return => { self.emit_audio(AudioEvent::BoonSelected); self.start_new_game(); }
                VirtualKeyCode::Escape => { self.emit_audio(AudioEvent::MenuCancel); self.screen = AppScreen::CharacterCreation; }
                _ => {}
            },

            AppScreen::FloorNav => {
                // If nemesis reveal is active, Enter/Escape advances to combat
                if self.nemesis_reveal.active {
                    if matches!(key, VirtualKeyCode::Return | VirtualKeyCode::Escape | VirtualKeyCode::Space) {
                        self.nemesis_reveal.active = false;
                        self.screen = AppScreen::Combat;
                    }
                    return;
                }
            match key {
                VirtualKeyCode::E | VirtualKeyCode::Return => {
                    self.enter_current_room();
                }
                VirtualKeyCode::D => {
                    if self.floor.as_ref().map(|f| f.rooms_remaining() == 0).unwrap_or(false) {
                        if self.floor_num >= self.max_floor {
                            self.save_score_now();
                            self.screen = AppScreen::Victory;
                        } else {
                            self.floor_num += 1;
                            self.generate_floor_for_current();
                        }
                    }
                }
                VirtualKeyCode::C => {
                    self.screen = AppScreen::CharacterSheet;
                }
                VirtualKeyCode::B => {
                    self.screen = AppScreen::BodyChart;
                }
                VirtualKeyCode::Z => {
                    self.auto_mode = !self.auto_mode;
                    self.auto_last_action = 0;
                    if self.auto_mode {
                        // Auto-alloc any pending points immediately on enabling
                        let seed = self.floor_seed.wrapping_add(self.frame);
                        if let Some(ref mut p) = self.player {
                            if p.skill_points > 0 {
                                let msgs = p.auto_allocate_passives(seed);
                                for m in msgs { self.push_log(m); }
                            }
                        }
                        self.push_log("AUTO PILOT ON — pauses for item pickups and shop/craft".to_string());
                    } else {
                        self.push_log("Auto pilot OFF.".to_string());
                    }
                }
                VirtualKeyCode::F5 => { self.do_save(); }
                VirtualKeyCode::S => self.screen = AppScreen::Scoreboard,
                VirtualKeyCode::Q | VirtualKeyCode::Escape => {
                    self.save_score_now();
                    self.screen = AppScreen::Title;
                }
                _ => {}
            }}, // end FloorNav match + nemesis_reveal block

            AppScreen::RoomView => {
                let has_item  = self.room_event.pending_item.is_some();
                let has_spell = self.room_event.pending_spell.is_some();
                let is_portal = self.room_event.portal_available;
                match key {
                    VirtualKeyCode::P => {
                        if is_portal {
                            self.room_event.portal_available = false;
                            if self.floor_num >= self.max_floor {
                                self.save_score_now();
                                self.screen = AppScreen::Victory;
                            } else {
                                self.floor_num += 1;
                                self.generate_floor_for_current();
                                self.screen = AppScreen::FloorNav;
                            }
                        } else if has_item {
                            if let Some(item) = self.room_event.pending_item.take() {
                                let name = item.name.clone();
                                let mods: Vec<_> = item.stat_modifiers.iter().map(|m| (m.stat.clone(), m.value)).collect();
                                for (stat, val) in &mods { self.apply_stat_modifier(stat, *val); }
                                if let Some(ref mut p) = self.player { p.add_item(item); }
                                self.push_log(format!("Picked up {}", name));
                            }
                            if !has_spell {
                                self.advance_floor_room();
                                if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                            }
                        }
                    }
                    VirtualKeyCode::L if has_spell => {
                        if let Some(spell) = self.room_event.pending_spell.take() {
                            let name = spell.name.clone();
                            if let Some(ref mut p) = self.player { p.add_spell(spell); }
                            self.push_log(format!("Learned spell: {}", name));
                        }
                        if !self.room_event.pending_item.is_some() {
                            self.advance_floor_room();
                            if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                        }
                    }
                    VirtualKeyCode::Return | VirtualKeyCode::Escape | VirtualKeyCode::X => {
                        // Skip/leave remaining pending items
                        self.room_event.pending_item = None;
                        self.room_event.pending_spell = None;
                        if is_portal {
                            self.room_event.portal_available = false;
                        }
                        self.advance_floor_room();
                        if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                    }
                    _ => {}
                }
            },

            AppScreen::Combat => {
                let action = match key {
                    VirtualKeyCode::A => Some(CombatAction::Attack),
                    VirtualKeyCode::H => Some(CombatAction::HeavyAttack),
                    VirtualKeyCode::D => Some(CombatAction::Defend),
                    VirtualKeyCode::T => Some(CombatAction::Taunt),
                    VirtualKeyCode::F => Some(CombatAction::Flee),
                    // Spells 1-8
                    VirtualKeyCode::Key1 => Some(CombatAction::UseSpell(0)),
                    VirtualKeyCode::Key2 => Some(CombatAction::UseSpell(1)),
                    VirtualKeyCode::Key3 => Some(CombatAction::UseSpell(2)),
                    VirtualKeyCode::Key4 => Some(CombatAction::UseSpell(3)),
                    VirtualKeyCode::Key5 => Some(CombatAction::UseSpell(4)),
                    VirtualKeyCode::Key6 => Some(CombatAction::UseSpell(5)),
                    VirtualKeyCode::Key7 => Some(CombatAction::UseSpell(6)),
                    VirtualKeyCode::Key8 => Some(CombatAction::UseSpell(7)),
                    // Items Q/W/E/R/Y/U/I/O = items 1-8
                    VirtualKeyCode::Q => Some(CombatAction::UseItem(0)),
                    VirtualKeyCode::W => Some(CombatAction::UseItem(1)),
                    VirtualKeyCode::E => Some(CombatAction::UseItem(2)),
                    VirtualKeyCode::R => Some(CombatAction::UseItem(3)),
                    VirtualKeyCode::Y => Some(CombatAction::UseItem(4)),
                    VirtualKeyCode::U => Some(CombatAction::UseItem(5)),
                    VirtualKeyCode::I => Some(CombatAction::UseItem(6)),
                    VirtualKeyCode::O => Some(CombatAction::UseItem(7)),
                    VirtualKeyCode::V => {
                        self.chaos_viz_open = !self.chaos_viz_open;
                        if self.chaos_viz_open {
                            self.achievements.check_event("chaos_engine_viz", 1);
                            self.achievements.save();
                        }
                        None
                    }
                    VirtualKeyCode::Tab => {
                        self.combat_log_collapsed = !self.combat_log_collapsed;
                        None
                    }
                    _ => None,
                };
                if let Some(act) = action {
                    // Track action type for combat animation selection
                    match &act {
                        CombatAction::Attack      => { self.last_action_type = 1; }
                        CombatAction::HeavyAttack => { self.last_action_type = 2; }
                        CombatAction::UseSpell(idx) => {
                            self.last_action_type = 3;
                            // Capture spell name before resolve
                            self.last_spell_name = self.player.as_ref()
                                .and_then(|p| p.known_spells.get(*idx))
                                .map(|s| s.name.clone())
                                .unwrap_or_default();
                        }
                        CombatAction::Defend  => { self.last_action_type = 4; }
                        CombatAction::Flee    => { self.last_action_type = 5; }
                        _                     => { self.last_action_type = 0; }
                    }
                    // Emit pre-action audio
                    match &act {
                        CombatAction::Attack => self.emit_audio(AudioEvent::PlayerAttack),
                        CombatAction::HeavyAttack => self.emit_audio(AudioEvent::PlayerHeavyAttack),
                        CombatAction::Defend => self.emit_audio(AudioEvent::PlayerDefend),
                        CombatAction::UseSpell(idx) => self.emit_audio(AudioEvent::SpellCast { spell_index: *idx }),
                        _ => {}
                    }
                    self.resolve_combat_action(act);
                }
            },

            AppScreen::Shop => match key {
                VirtualKeyCode::H => {
                    let cost = self.shop_heal_cost;
                    let (can_afford, pgold) = self.player.as_ref()
                        .map(|p| (p.gold >= cost, p.gold)).unwrap_or((false, 0));
                    if can_afford {
                        if let Some(ref mut p) = self.player { p.gold -= cost; p.heal_scaled(40); }
                        self.push_log(format!("Bought heal potion. +40 HP (-{}g)", cost));
                    } else {
                        self.push_log(format!("Need {}g. Have {}g.", cost, pgold));
                    }
                }
                VirtualKeyCode::Key1 | VirtualKeyCode::Key2 |
                VirtualKeyCode::Key3 | VirtualKeyCode::Key4 => {
                    let idx = match key {
                        VirtualKeyCode::Key1 => 0, VirtualKeyCode::Key2 => 1,
                        VirtualKeyCode::Key3 => 2, _ => 3,
                    };
                    if idx < self.shop_items.len() {
                        let (item, price) = self.shop_items[idx].clone();
                        if let Some(ref mut p) = self.player {
                            if p.gold >= price {
                                p.gold -= price;
                                let name = item.name.clone();
                                if item.is_weapon || item.stat_modifiers.is_empty() {
                                    p.add_item(item);
                                    self.push_log(format!("Purchased {}!", name));
                                } else {
                                    for m in item.stat_modifiers.clone() {
                                        self.apply_stat_modifier(&m.stat, m.value);
                                    }
                                    self.push_log(format!("Used {}! Stats updated.", name));
                                }
                                self.shop_items.remove(idx);
                            } else {
                                self.push_log(format!("Need {}g, have {}g.", price, self.player.as_ref().map(|p| p.gold).unwrap_or(0)));
                            }
                        }
                    }
                }
                VirtualKeyCode::Return | VirtualKeyCode::Key0 | VirtualKeyCode::Escape => {
                    self.advance_floor_room();
                    if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                }
                _ => {}
            },

            AppScreen::Crafting => match self.craft_phase {
                CraftPhase::SelectItem => {
                    if self.item_filter_active {
                        // Filter typing mode
                        match key {
                            VirtualKeyCode::Escape | VirtualKeyCode::Return => {
                                self.item_filter_active = false;
                                self.craft_item_cursor = 0;
                            }
                            VirtualKeyCode::Back => { self.item_filter.pop(); }
                            k => {
                                // Map VirtualKeyCode to char (basic a-z, 0-9, space)
                                let ch: Option<char> = match k {
                                    VirtualKeyCode::Space => Some(' '),
                                    VirtualKeyCode::Key0 => Some('0'), VirtualKeyCode::Key1 => Some('1'),
                                    VirtualKeyCode::Key2 => Some('2'), VirtualKeyCode::Key3 => Some('3'),
                                    VirtualKeyCode::Key4 => Some('4'), VirtualKeyCode::Key5 => Some('5'),
                                    VirtualKeyCode::Key6 => Some('6'), VirtualKeyCode::Key7 => Some('7'),
                                    VirtualKeyCode::Key8 => Some('8'), VirtualKeyCode::Key9 => Some('9'),
                                    VirtualKeyCode::A => Some('a'), VirtualKeyCode::B => Some('b'),
                                    VirtualKeyCode::C => Some('c'), VirtualKeyCode::D => Some('d'),
                                    VirtualKeyCode::E => Some('e'), VirtualKeyCode::F => Some('f'),
                                    VirtualKeyCode::G => Some('g'), VirtualKeyCode::H => Some('h'),
                                    VirtualKeyCode::I => Some('i'), VirtualKeyCode::J => Some('j'),
                                    VirtualKeyCode::K => Some('k'), VirtualKeyCode::L => Some('l'),
                                    VirtualKeyCode::M => Some('m'), VirtualKeyCode::N => Some('n'),
                                    VirtualKeyCode::O => Some('o'), VirtualKeyCode::P => Some('p'),
                                    VirtualKeyCode::Q => Some('q'), VirtualKeyCode::R => Some('r'),
                                    VirtualKeyCode::S => Some('s'), VirtualKeyCode::T => Some('t'),
                                    VirtualKeyCode::U => Some('u'), VirtualKeyCode::V => Some('v'),
                                    VirtualKeyCode::W => Some('w'), VirtualKeyCode::X => Some('x'),
                                    VirtualKeyCode::Y => Some('y'), VirtualKeyCode::Z => Some('z'),
                                    _ => None,
                                };
                                if let Some(c) = ch {
                                    if self.item_filter.len() < 20 { self.item_filter.push(c); }
                                }
                            }
                        }
                    } else {
                        match key {
                            VirtualKeyCode::Up => {
                                if self.craft_item_cursor > 0 { self.craft_item_cursor -= 1; }
                            }
                            VirtualKeyCode::Down => {
                                let len = self.player.as_ref().map(|p| p.inventory.len()).unwrap_or(0);
                                if self.craft_item_cursor + 1 < len { self.craft_item_cursor += 1; }
                            }
                            VirtualKeyCode::Return => {
                                let has_item = self.player.as_ref().map(|p| !p.inventory.is_empty()).unwrap_or(false);
                                if has_item {
                                    self.craft_phase = CraftPhase::SelectOp;
                                    self.craft_op_cursor = 0;
                                    self.craft_message = String::new();
                                }
                            }
                            VirtualKeyCode::Slash => {
                                self.item_filter_active = true;
                                self.item_filter.clear();
                                self.achievements.check_event("item_filter_used", 1);
                                self.achievements.save();
                            }
                            VirtualKeyCode::Escape => {
                                if !self.item_filter.is_empty() {
                                    self.item_filter.clear();
                                } else {
                                    self.advance_floor_room();
                                    if self.screen != AppScreen::GameOver { self.screen = AppScreen::FloorNav; }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                CraftPhase::SelectOp => match key {
                    VirtualKeyCode::Up => { if self.craft_op_cursor > 0 { self.craft_op_cursor -= 1; } }
                    VirtualKeyCode::Down => { if self.craft_op_cursor < 8 { self.craft_op_cursor += 1; } }
                    VirtualKeyCode::Return => { self.apply_craft_op(); }
                    VirtualKeyCode::Key1 => { self.craft_op_cursor = 0; self.apply_craft_op(); }
                    VirtualKeyCode::Key2 => { self.craft_op_cursor = 1; self.apply_craft_op(); }
                    VirtualKeyCode::Key3 => { self.craft_op_cursor = 2; self.apply_craft_op(); }
                    VirtualKeyCode::Key4 => { self.craft_op_cursor = 3; self.apply_craft_op(); }
                    VirtualKeyCode::Key5 => { self.craft_op_cursor = 4; self.apply_craft_op(); }
                    VirtualKeyCode::Key6 => { self.craft_op_cursor = 5; self.apply_craft_op(); }
                    VirtualKeyCode::Key7 => { self.craft_op_cursor = 6; self.apply_craft_op(); }
                    VirtualKeyCode::Key8 => { self.craft_op_cursor = 7; self.apply_craft_op(); }
                    VirtualKeyCode::Key9 => { self.craft_op_cursor = 8; self.apply_craft_op(); }
                    VirtualKeyCode::Escape => {
                        self.craft_phase = CraftPhase::SelectItem;
                        self.craft_message = String::new();
                    }
                    _ => {}
                },
            },

            AppScreen::GameOver | AppScreen::Victory => match key {
                VirtualKeyCode::Return | VirtualKeyCode::Escape => {
                    self.player = None; self.enemy = None; self.floor = None;
                    self.combat_state = None; self.combat_log.clear();
                    self.screen = AppScreen::Title;
                }
                VirtualKeyCode::S => self.screen = AppScreen::Scoreboard,
                _ => {}
            },

            AppScreen::Scoreboard => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::Q | VirtualKeyCode::Return => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::CharacterSheet => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::C | VirtualKeyCode::Return => {
                    self.screen = AppScreen::FloorNav;
                }
                VirtualKeyCode::Right | VirtualKeyCode::Tab => {
                    self.char_tab = (self.char_tab + 1) % 5;
                }
                VirtualKeyCode::Left => {
                    if self.char_tab > 0 { self.char_tab -= 1; } else { self.char_tab = 4; }
                }
                VirtualKeyCode::Key1 => self.char_tab = 0,
                VirtualKeyCode::Key2 => self.char_tab = 1,
                VirtualKeyCode::Key3 => self.char_tab = 2,
                VirtualKeyCode::Key4 => self.char_tab = 3,
                VirtualKeyCode::Key5 => self.char_tab = 4,
                VirtualKeyCode::B => { self.screen = AppScreen::BodyChart; }
                VirtualKeyCode::P => { self.screen = AppScreen::PassiveTree; }
                VirtualKeyCode::N => {
                    let seed = self.floor_seed.wrapping_add(self.frame);
                    if let Some(ref mut p) = self.player {
                        if p.skill_points > 0 {
                            let msgs = p.auto_allocate_passives(seed);
                            for m in msgs { self.combat_log.push(m); }
                        }
                    }
                }
                _ => {}
            },

            AppScreen::PassiveTree => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::C => { self.screen = AppScreen::CharacterSheet; }
                VirtualKeyCode::Up   => { self.passive_scroll = self.passive_scroll.saturating_sub(1); }
                VirtualKeyCode::Down => { self.passive_scroll += 1; }
                VirtualKeyCode::PageUp   => { self.passive_scroll = self.passive_scroll.saturating_sub(10); }
                VirtualKeyCode::PageDown => { self.passive_scroll += 10; }
                VirtualKeyCode::N => {
                    let seed = self.floor_seed.wrapping_add(self.frame);
                    if let Some(ref mut p) = self.player {
                        if p.skill_points > 0 {
                            let msgs = p.auto_allocate_passives(seed);
                            for m in msgs { self.combat_log.push(m); }
                            self.passive_scroll = 0;
                            // Allocation burst particles
                            emit_level_up_fountain(&mut self.particles, 80.0, 40.0);
                            self.trigger_pulse_ring(80.0, 40.0, (0.8, 1.0, 0.6), 1.0);
                        }
                    }
                }
                _ => {}
            },

            AppScreen::BodyChart => match key {
                VirtualKeyCode::Escape | VirtualKeyCode::B | VirtualKeyCode::Return => {
                    self.screen = AppScreen::FloorNav;
                }
                VirtualKeyCode::C => {
                    self.screen = AppScreen::CharacterSheet;
                }
                _ => {}
            },

            AppScreen::Tutorial => match key {
                VirtualKeyCode::Right | VirtualKeyCode::Return | VirtualKeyCode::Space => {
                    const TOTAL_SLIDES: usize = 5;
                    if self.tutorial_slide >= TOTAL_SLIDES {
                        self.tutorial_slide = 0;
                        self.screen = AppScreen::Title;
                    } else {
                        self.tutorial_slide += 1;
                    }
                }
                VirtualKeyCode::Left => {
                    if self.tutorial_slide > 1 { self.tutorial_slide -= 1; }
                }
                VirtualKeyCode::Escape | VirtualKeyCode::Q => {
                    self.tutorial_slide = 0;
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::Achievements => match key {
                VirtualKeyCode::Up   => { self.achievement_scroll = self.achievement_scroll.saturating_sub(1); }
                VirtualKeyCode::Down => { self.achievement_scroll += 1; } // clamped in draw
                VirtualKeyCode::PageUp   => { self.achievement_scroll = self.achievement_scroll.saturating_sub(15); }
                VirtualKeyCode::PageDown => { self.achievement_scroll += 15; } // clamped in draw
                VirtualKeyCode::Home => { self.achievement_scroll = 0; }
                VirtualKeyCode::F | VirtualKeyCode::Tab => {
                    self.achievement_filter = (self.achievement_filter + 1) % 3;
                    self.achievement_scroll = 0;
                }
                VirtualKeyCode::Escape | VirtualKeyCode::Q | VirtualKeyCode::Return => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::RunHistory => match key {
                VirtualKeyCode::Up   => { if self.history_scroll > 0 { self.history_scroll -= 1; } }
                VirtualKeyCode::Down => { self.history_scroll = (self.history_scroll + 1).min(self.run_history.runs.len().saturating_sub(1)); }
                VirtualKeyCode::Escape | VirtualKeyCode::Q | VirtualKeyCode::Return => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::DailyLeaderboard => match key {
                VirtualKeyCode::R => {
                    // Manual refresh
                    self.daily_status = "Fetching...".to_string();
                    let url  = self.config.leaderboard.url.clone();
                    let date = chrono_date_simple();
                    match fetch_scores(&url, &date) {
                        Ok(rows) => {
                            self.daily_status = format!("Updated — {} entries", rows.len());
                            self.daily_rows = rows;
                        }
                        Err(e) => self.daily_status = format!("Error: {}", &e.chars().take(40).collect::<String>()),
                    }
                }
                VirtualKeyCode::Escape | VirtualKeyCode::Q | VirtualKeyCode::Return => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::Bestiary => match key {
                VirtualKeyCode::Up => {
                    if self.bestiary_selected > 0 { self.bestiary_selected -= 1; }
                }
                VirtualKeyCode::Down => {
                    // Draw function clamps to actual list size; bump freely here
                    self.bestiary_selected = self.bestiary_selected.saturating_add(1);
                }
                VirtualKeyCode::Escape | VirtualKeyCode::Q | VirtualKeyCode::Return => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::Codex => match key {
                VirtualKeyCode::Up => {
                    if self.codex_selected > 0 { self.codex_selected -= 1; }
                }
                VirtualKeyCode::Down => {
                    // Draw function clamps to actual list size; bump freely here
                    self.codex_selected = self.codex_selected.saturating_add(1);
                }
                VirtualKeyCode::Escape | VirtualKeyCode::Q | VirtualKeyCode::Return => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },

            AppScreen::Settings => match key {
                VirtualKeyCode::Left | VirtualKeyCode::Right => {
                    // Cycle vibe
                    self.music_vibe = if matches!(key, VirtualKeyCode::Left) {
                        // cycle backwards: Off->Minimal->Classic->Chill
                        match self.music_vibe {
                            MusicVibe::Chill   => MusicVibe::Off,
                            MusicVibe::Classic => MusicVibe::Chill,
                            MusicVibe::Minimal => MusicVibe::Classic,
                            MusicVibe::Off     => MusicVibe::Minimal,
                        }
                    } else {
                        self.music_vibe.cycle()
                    };
                    if let Some(ref audio) = self.audio {
                        audio.set_vibe(self.music_vibe);
                    }
                }
                VirtualKeyCode::T => self.cycle_theme(),
                VirtualKeyCode::Escape | VirtualKeyCode::Q => {
                    self.screen = AppScreen::Title;
                }
                _ => {}
            },
        }
    }

    fn apply_craft_op(&mut self) {
        let idx = self.craft_item_cursor;
        let seed = self.floor_seed.wrapping_add(self.frame).wrapping_mul(6364136223846793005);

        let has_item = self.player.as_ref().map(|p| idx < p.inventory.len()).unwrap_or(false);
        if !has_item { self.craft_message = "No item at that index.".to_string(); return; }

        match self.craft_op_cursor {
            0 => { // Reforge
                if let Some(ref mut p) = self.player {
                    let n = p.inventory[idx].stat_modifiers.len().max(1);
                    p.inventory[idx].stat_modifiers.clear();
                    for j in 0..n {
                        let ms = seed.wrapping_add(j as u64 * 17777).wrapping_mul(6364136223846793005);
                        p.inventory[idx].stat_modifiers.push(StatModifier::generate_random(ms));
                    }
                    self.craft_message = format!("REFORGED! {} modifiers chaos-rolled anew.", n);
                    self.craft_anim_timer = 40; self.craft_anim_type = 1;
                }
            }
            1 => { // Augment
                if let Some(ref mut p) = self.player {
                    let ms = seed.wrapping_mul(0xdeadbeef).wrapping_add(p.inventory[idx].value as u64);
                    let new_mod = StatModifier::generate_random(ms);
                    let stat = new_mod.stat.clone(); let val = new_mod.value;
                    p.inventory[idx].stat_modifiers.push(new_mod);
                    p.inventory[idx].value = (p.inventory[idx].value as f64 * 1.2) as i64;
                    self.craft_message = format!("AUGMENTED! Added {:+} {}", val, stat);
                }
            }
            2 => { // Annul
                if let Some(ref mut p) = self.player {
                    if p.inventory[idx].stat_modifiers.is_empty() {
                        self.craft_message = "No modifiers to remove.".to_string();
                    } else {
                        let ri = (seed % p.inventory[idx].stat_modifiers.len() as u64) as usize;
                        let removed = p.inventory[idx].stat_modifiers.remove(ri);
                        self.craft_message = format!("ANNULLED: removed {} {:+}", removed.stat, removed.value);
                    }
                }
            }
            3 => { // Corrupt — risk-tiered
                // Risk tier cycles with the cursor sub-key (frame parity used as proxy)
                // Each time the player presses Enter the tier advances: Safe → Risky → Reckless
                let risk = (self.frame / 60) % 3; // 0=Safe 1=Risky 2=Reckless
                if let Some(ref mut p) = self.player {
                    let roll = chaos_roll_verbose(0.5, seed);
                    // Safe:  5 buckets (0-4), positive-weighted  — multiplier *1.2
                    // Risky: 7 buckets, neutral                   — multiplier *1.5
                    // Reckless: 9 buckets, destruction possible   — multiplier *2
                    let (buckets, tag) = match risk {
                        0 => (5u64, "SAFE"),
                        1 => (7u64, "RISKY"),
                        _ => (9u64, "RECKLESS"),
                    };
                    let outcome = roll.to_range(0, (buckets - 1) as i64) as u64;
                    let item = &mut p.inventory[idx];
                    let result = match outcome {
                        0 => {
                            if item.socket_count < 6 { item.socket_count += 1; format!("[{}] +1 socket!", tag) }
                            else { format!("[{}] glows... nothing changes.", tag) }
                        }
                        1 => {
                            if !item.stat_modifiers.is_empty() {
                                let i2 = (seed.wrapping_add(99) % item.stat_modifiers.len() as u64) as usize;
                                let mult = if risk == 2 { 3 } else if risk == 1 { 2 } else { 2 };
                                item.stat_modifiers[i2].value *= mult;
                                format!("[{}] a modifier was ×{}!", tag, mult)
                            } else { format!("[{}] sparks, nothing happens.", tag) }
                        }
                        2 => {
                            item.corruption = Some("Chaos-Touched".to_string());
                            let bonus = if risk == 2 { 1.0 } else if risk == 1 { 0.75 } else { 0.5 };
                            item.value += (item.value as f64 * bonus) as i64;
                            format!("[{}] Chaos-Touched! (+{:.0}% value)", tag, bonus * 100.0)
                        }
                        3 => {
                            if item.stat_modifiers.is_empty() { format!("[{}] Nothing to remove.", tag) }
                            else { item.stat_modifiers.pop(); format!("[{}] a modifier dissolved.", tag) }
                        }
                        4 => {
                            for m in &mut item.stat_modifiers { m.value = -m.value; }
                            format!("[{}] all modifiers INVERTED!", tag)
                        }
                        5 | 6 if risk >= 1 => {
                            // Risky-exclusive: double or halve all mods
                            let coin = seed % 2 == 0;
                            for m in &mut item.stat_modifiers {
                                if coin { m.value *= 2; } else { m.value /= 2; }
                            }
                            format!("[{}] mods {}!", tag, if coin { "DOUBLED" } else { "halved" })
                        }
                        7 | 8 if risk == 2 => {
                            // Reckless-exclusive: item destroyed 5% or type flip
                            if outcome == 8 {
                                p.inventory.remove(idx);
                                self.craft_message = "[RECKLESS] Item DESTROYED by chaos!".to_string();
                                self.achievements.check_event("corrupt_used", -1);
                                self.achievements.save();
                                self.craft_phase = CraftPhase::SelectItem;
                                return;
                            } else {
                                item.is_weapon = !item.is_weapon;
                                format!("[RECKLESS] item type transmogrified!")
                            }
                        }
                        _ => { item.is_weapon = !item.is_weapon; format!("[{}] transmogrified!", tag) }
                    };
                    self.craft_message = result;
                    self.craft_anim_timer = 40; self.craft_anim_type = 2;
                    self.achievements.check_event("corrupt_used", 1);
                    if risk == 2 { self.achievements.check_event("corrupt_used", 5); }
                    self.achievements.save();
                }
            }
            4 => { // Fuse
                if let Some(ref mut p) = self.player {
                    p.inventory[idx].value *= 2;
                    p.inventory[idx].rarity = match p.inventory[idx].rarity {
                        Rarity::Common => Rarity::Uncommon,
                        Rarity::Uncommon => Rarity::Rare,
                        Rarity::Rare => Rarity::Epic,
                        Rarity::Epic => Rarity::Legendary,
                        Rarity::Legendary => Rarity::Mythical,
                        Rarity::Mythical => Rarity::Divine,
                        Rarity::Divine => Rarity::Beyond,
                        Rarity::Beyond | Rarity::Artifact => Rarity::Artifact,
                    };
                    self.craft_message = format!("FUSED! Value doubled, rarity → {}", p.inventory[idx].rarity.name());
                }
            }
            5 => { // EngineLock
                let cost = 40 + self.floor_num as i64 * 5;
                let can_afford = self.player.as_ref().map(|p| p.gold >= cost).unwrap_or(false);
                if !can_afford {
                    self.craft_message = format!("Need {}g for EngineLock.", cost);
                    return;
                }
                let engines = ["Lorenz","Zeta","Collatz","Mandelbrot","Fibonacci","Euler","Linear","SharpEdge","Orbit","Recursive"];
                let ei = (seed % engines.len() as u64) as usize;
                let eng = engines[ei].to_string();
                if let Some(ref mut p) = self.player {
                    p.gold -= cost;
                    p.inventory[idx].engine_locks.push(eng.clone());
                    self.craft_message = format!("ENGINE LOCKED: {} embedded! (-{}g)", eng, cost);
                }
            }
            6 => { // Shatter — destroy item, scatter mods to other items
                if let Some(ref mut p) = self.player {
                    if p.inventory.len() < 2 {
                        self.craft_message = "Need at least 2 items to Shatter.".to_string();
                        return;
                    }
                    let mods: Vec<_> = p.inventory[idx].stat_modifiers.clone();
                    let name = p.inventory[idx].name.clone();
                    p.inventory.remove(idx);
                    let n = p.inventory.len();
                    if n > 0 && !mods.is_empty() {
                        for (j, m) in mods.into_iter().enumerate() {
                            let target = (seed.wrapping_add(j as u64 * 31337)) % n as u64;
                            p.inventory[target as usize].stat_modifiers.push(m);
                        }
                        self.craft_message = format!("SHATTERED {}! Mods scattered to other items.", name);
                    } else {
                        self.craft_message = format!("SHATTERED {}! (No mods to scatter.)", name);
                    }
                    self.craft_anim_timer = 40; self.craft_anim_type = 3;
                    self.craft_phase = CraftPhase::SelectItem;
                }
            }
            7 => { // Imbue — add charges
                if let Some(ref mut p) = self.player {
                    let item = &mut p.inventory[idx];
                    let charges = if item.charges > 0 { item.charges + 3 } else { 3 };
                    item.charges = charges.min(9);
                    self.craft_message = format!("IMBUED! Item now has {} charges (+use for bonus effect).", item.charges);
                }
            }
            8 => { // Repair — restore durability
                let cost = self.player.as_ref().map(|p| {
                    chaos_rpg_core::crafting::repair_cost(&p.inventory[idx], self.floor_num as u32)
                }).unwrap_or(0);
                let can_afford = self.player.as_ref().map(|p| p.gold >= cost).unwrap_or(false);
                if !can_afford {
                    self.craft_message = format!("Need {}g to repair.", cost);
                    return;
                }
                if let Some(ref mut p) = self.player {
                    let item = &p.inventory[idx];
                    if item.durability == item.max_durability {
                        self.craft_message = format!("{} is already at full durability.", item.name);
                        return;
                    }
                    use chaos_rpg_core::crafting::{repair, CraftResult};
                    match repair(&p.inventory[idx]) {
                        CraftResult::Success { description, item: repaired } => {
                            p.gold -= cost;
                            p.inventory[idx] = repaired;
                            self.craft_message = format!("{} (-{}g)", description, cost);
                        }
                        CraftResult::Failure { reason } | CraftResult::Bricked { description: reason, .. } => {
                            self.craft_message = reason;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// ─── HELPER FUNCTIONS ────────────────────────────────────────────────────────

/// Convert a VirtualKeyCode + shift state to a displayable character for text input.
fn key_to_char(key: bracket_lib::prelude::VirtualKeyCode, shift: bool) -> Option<char> {
    use bracket_lib::prelude::VirtualKeyCode::*;
    match key {
        A => Some(if shift { 'A' } else { 'a' }),
        B => Some(if shift { 'B' } else { 'b' }),
        C => Some(if shift { 'C' } else { 'c' }),
        D => Some(if shift { 'D' } else { 'd' }),
        E => Some(if shift { 'E' } else { 'e' }),
        F => Some(if shift { 'F' } else { 'f' }),
        G => Some(if shift { 'G' } else { 'g' }),
        H => Some(if shift { 'H' } else { 'h' }),
        I => Some(if shift { 'I' } else { 'i' }),
        J => Some(if shift { 'J' } else { 'j' }),
        K => Some(if shift { 'K' } else { 'k' }),
        L => Some(if shift { 'L' } else { 'l' }),
        M => Some(if shift { 'M' } else { 'm' }),
        N => Some(if shift { 'N' } else { 'n' }),
        O => Some(if shift { 'O' } else { 'o' }),
        P => Some(if shift { 'P' } else { 'p' }),
        Q => Some(if shift { 'Q' } else { 'q' }),
        R => Some(if shift { 'R' } else { 'r' }),
        S => Some(if shift { 'S' } else { 's' }),
        T => Some(if shift { 'T' } else { 't' }),
        U => Some(if shift { 'U' } else { 'u' }),
        V => Some(if shift { 'V' } else { 'v' }),
        W => Some(if shift { 'W' } else { 'w' }),
        X => Some(if shift { 'X' } else { 'x' }),
        Y => Some(if shift { 'Y' } else { 'y' }),
        Z => Some(if shift { 'Z' } else { 'z' }),
        Key0 => Some(if shift { ')' } else { '0' }),
        Key1 => Some(if shift { '!' } else { '1' }),
        Key2 => Some(if shift { '@' } else { '2' }),
        Key3 => Some(if shift { '#' } else { '3' }),
        Key4 => Some(if shift { '$' } else { '4' }),
        Key5 => Some(if shift { '%' } else { '5' }),
        Key6 => Some(if shift { '^' } else { '6' }),
        Key7 => Some(if shift { '&' } else { '7' }),
        Key8 => Some(if shift { '*' } else { '8' }),
        Key9 => Some(if shift { '(' } else { '9' }),
        Space => Some(' '),
        Minus => Some(if shift { '_' } else { '-' }),
        Apostrophe => Some(if shift { '"' } else { '\'' }),
        _ => None,
    }
}

/// Returns current date as "YYYY-MM-DD" using only std.
fn chrono_date_simple() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Days since epoch
    let days = secs / 86400;
    // Gregorian calendar calculation
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Build the shareable plain-text run recap.
fn build_recap_text(
    p: &chaos_rpg_core::character::Character,
    score: u64,
    misery: f64,
    tier: &str,
    epitaph: &str,
    mode: &str,
    seed: u64,
) -> String {
    let ratio = if p.run_stats.damage_taken > 0 {
        p.run_stats.damage_dealt as f64 / p.run_stats.damage_taken as f64
    } else {
        p.run_stats.damage_dealt as f64
    };
    format!(
        "=== CHAOS RPG — Run Recap ===\n\
         Date      : {date}\n\
         Seed      : {seed}\n\
         Mode      : {mode}\n\
         Character : {name} ({class} / {diff}) Lv.{level}\n\
         Tier      : {tier}\n\
         \n\
         Floor     : {floor}\n\
         Score     : {score}\n\
         Kills     : {kills}\n\
         Gold      : {gold}g\n\
         \n\
         Dmg Dealt : {dealt}\n\
         Dmg Taken : {taken}\n\
         D/T Ratio : {ratio:.2}\n\
         Best Hit  : {best}\n\
         Spells    : {spells}\n\
         \n\
         Misery    : {misery:.1}\n\
         Corruption: {corr}\n\
         Cause     : {cod}\n\
         \n\
         \"{epitaph}\"\n\
         \n\
         Play free at https://mfletcherdev.itch.io/chaos-rpg\n\
         ==============================",
        date    = chrono_date_simple(),
        seed    = seed,
        mode    = mode,
        name    = p.name,
        class   = p.class.name(),
        diff    = p.difficulty.name(),
        level   = p.level,
        tier    = tier,
        floor   = p.floor,
        score   = score,
        kills   = p.kills,
        gold    = p.gold,
        dealt   = p.run_stats.damage_dealt,
        taken   = p.run_stats.damage_taken,
        ratio   = ratio,
        best    = p.run_stats.highest_single_hit,
        spells  = p.run_stats.spells_cast,
        misery  = misery,
        corr    = p.corruption,
        cod     = p.run_stats.cause_of_death,
        epitaph = epitaph,
    )
}

// ─── ACHIEVEMENTS SCREEN ─────────────────────────────────────────────────────

impl State {
    fn draw_achievements(&mut self, ctx: &mut BTerm) {
        use chaos_rpg_core::achievements::AchievementRarity;

        let t = self.theme_graded();
        let bg    = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd    = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac    = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let gld   = RGB::from_u8(t.gold.0,    t.gold.1,    t.gold.2);
        let dim   = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let suc   = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let muted = RGB::from_u8(t.muted.0,   t.muted.1,   t.muted.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "ACHIEVEMENTS", &t);

        // Helper: rarity -> color
        let rarity_rgb = |r: &AchievementRarity| -> (u8, u8, u8) {
            match r {
                AchievementRarity::Common    => (170, 170, 170),
                AchievementRarity::Uncommon  => (80,  200,  80),
                AchievementRarity::Rare      => (60,  120, 240),
                AchievementRarity::Epic      => (160,  60, 210),
                AchievementRarity::Legendary => (240, 150,  20),
                AchievementRarity::Mythic    => (240,  40,  40),
                AchievementRarity::Omega     => (240,  10, 200),
            }
        };

        // Build list: sort unlocked first (by rarity desc), then locked (by rarity desc)
        let all = chaos_rpg_core::achievements::all_achievements();
        let total = all.len();
        let unlocked_count = all.iter().filter(|a| self.achievements.is_unlocked(&a.id)).count();
        let locked_count = total - unlocked_count;

        let rarity_order = |r: &AchievementRarity| -> u8 {
            match r {
                AchievementRarity::Omega     => 6,
                AchievementRarity::Mythic    => 5,
                AchievementRarity::Legendary => 4,
                AchievementRarity::Epic      => 3,
                AchievementRarity::Rare      => 2,
                AchievementRarity::Uncommon  => 1,
                AchievementRarity::Common    => 0,
            }
        };

        // Apply filter
        let filter = self.achievement_filter;
        let mut filtered: Vec<_> = all.iter()
            .filter(|a| {
                let u = self.achievements.is_unlocked(&a.id);
                match filter {
                    1 => u,
                    2 => !u,
                    _ => true,
                }
            })
            .collect();

        // Sort: unlocked → locked, within each group by rarity desc
        filtered.sort_by(|a, b| {
            let ua = self.achievements.is_unlocked(&a.id);
            let ub = self.achievements.is_unlocked(&b.id);
            match (ua, ub) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => rarity_order(&b.rarity).cmp(&rarity_order(&a.rarity)),
            }
        });

        let filtered_total = filtered.len();

        // ── LEFT PANEL: scrollable list (cols 1-78) ──────────────────────────
        let list_y0 = 8i32;
        let list_h  = 62i32; // rows 8-69
        let entry_h = 2i32;  // 2 lines per entry
        let visible  = (list_h / entry_h) as usize; // 31 entries visible

        // Clamp scroll
        let max_scroll = filtered_total.saturating_sub(visible);
        self.achievement_scroll = self.achievement_scroll.min(max_scroll);
        let scroll = self.achievement_scroll;

        for (vi, ach) in filtered.iter().skip(scroll).take(visible).enumerate() {
            let unlocked = self.achievements.is_unlocked(&ach.id);
            let rc_tuple = rarity_rgb(&ach.rarity);
            let rc = RGB::from_u8(rc_tuple.0, rc_tuple.1, rc_tuple.2);
            let y = list_y0 + vi as i32 * entry_h;

            if unlocked {
                // ★ [RARITY] Name
                let rarity_short = match ach.rarity {
                    AchievementRarity::Common    => "CMN",
                    AchievementRarity::Uncommon  => "UNC",
                    AchievementRarity::Rare      => "RAR",
                    AchievementRarity::Epic      => "EPC",
                    AchievementRarity::Legendary => "LEG",
                    AchievementRarity::Mythic    => "MYT",
                    AchievementRarity::Omega     => "Ω  ",
                };
                ctx.print_color(3,  y, suc, bg, "★");
                ctx.print_color(5,  y, rc,  bg, rarity_short);
                let name: String = ach.name.chars().take(38).collect();
                ctx.print_color(9,  y, hd,  bg, &name);
                // Unlock date (right-aligned in left panel)
                if !ach.unlock_date.is_empty() {
                    let date: String = ach.unlock_date.chars().take(10).collect();
                    ctx.print_color(62, y, dim, bg, &date);
                }
                // Description line
                let desc: String = ach.description.chars().take(72).collect();
                ctx.print_color(5, y + 1, dim, bg, &desc);
            } else {
                // ○ [???] Name (dimmed, no description)
                ctx.print_color(3,  y, muted, bg, "○");
                ctx.print_color(5,  y, muted, bg, "???");
                let name: String = ach.name.chars().take(38).collect();
                ctx.print_color(9,  y, muted, bg, &name);
                ctx.print_color(5,  y + 1, muted, bg, "Locked — complete the challenge to reveal the description.");
            }
        }

        // Scroll indicator
        if filtered_total > visible {
            let pct = if max_scroll > 0 { scroll * 100 / max_scroll } else { 100 };
            ctx.print_color(3, 71, dim, bg, &format!("↑↓ {}/{} ({:3}%)", scroll + 1, filtered_total, pct));
        }

        // ── DIVIDER ──────────────────────────────────────────────────────────
        for y in 1..=77i32 {
            ctx.print_color(79, y, muted, bg, "│");
        }

        // ── RIGHT PANEL: stats & category breakdown (cols 81-157) ────────────
        let rx = 82i32;

        // Overall progress
        ctx.print_color(rx, 2, hd, bg, &format!("{}/{} Unlocked", unlocked_count, total));
        let bar_w = 72i32;
        let filled = if total > 0 { (unlocked_count as i32 * bar_w) / total as i32 } else { 0 };
        for i in 0..bar_w {
            ctx.print_color(rx + i, 3, if i < filled { suc } else { muted }, bg,
                if i < filled { "█" } else { "░" });
        }

        // Filter tabs
        let filter_labels = ["[A]ll", "[U]nlocked", "[L]ocked"];
        let filter_counts  = [total, unlocked_count, locked_count];
        let mut fx = rx;
        for (i, (lbl, cnt)) in filter_labels.iter().zip(filter_counts.iter()).enumerate() {
            let col = if self.achievement_filter == i as u8 { ac } else { dim };
            ctx.print_color(fx, 5, col, bg, &format!("{} {}", lbl, cnt));
            fx += lbl.len() as i32 + cnt.to_string().len() as i32 + 2;
        }

        draw_separator(ctx, 80, 6, 78, &t);

        // Category breakdown
        let categories: &[(&str, &[&str])] = &[
            ("Getting Started", &["first_blood","baby_steps","window_shopper","identity_crisis",
                "graveyard_shift","try_try_again","loot_goblin","bench_warmer","read_the_manual","first_clear"]),
            ("Combat",          &["overkill","paper_cut","untouchable","glass_cannon","speedrunner",
                "first_retreat","tactical_retreat","404_damage","negative_damage","kill_spree_5",
                "kill_spree_20","combo_crits","survive_1hp","one_hp_win","taunt_master","heavy_carry",
                "defend_100","max_kills_run","no_flee","pacifist"]),
            ("Chaos Engine",    &["pipeline_speaks","the_number","golden_ratio","eulers_number",
                "perfectly_balanced","killed_by_math","chain_10","all_positive_chain",
                "all_negative_chain","perfect_zero","max_value","min_value","pi_roll"]),
            ("Death & Misery",  &["humbling","misery_5k","defiant","cosmic_punchline","transcendent",
                "published_failure","accountants_bill","democratic_execution","ouroboros_loop",
                "recursion_overflow","thousand_deaths"]),
            ("Progression",     &["double_digits","quarter_century","the_deep","are_you_okay",
                "the_algorithm_awaits","beyond_the_algorithm","infinity_and_beyond","centurion",
                "mass_extinction","genocide_route","millionaire","level_cap","floor_15","floor_30",
                "floor_150","floor_250","floor_300","level_50","level_75"]),
            ("Class Mastery",   &["mage_first","berserker_first","ranger_first","thief_first",
                "necro_first","alchemist_first","paladin_first","voidwalker_first","warlord_first",
                "trickster_first","runesmith_first","chrono_first","all_classes","mage_chaos","berserker_floor100"]),
            ("Bosses",          &["boss_the_mirror","boss_the_accountant","boss_the_fibonacci_hydra",
                "boss_the_eigenstate","boss_the_taxman","boss_the_null","boss_the_paradox",
                "boss_the_recursion","boss_the_committee","boss_the_collatz_titan",
                "boss_the_ouroboros","boss_the_algorithm_reborn"]),
            ("Spells & Items",  &["first_spell","spells_100","spells_1000","backfire_survivor",
                "backfire_10","full_mana_always","mana_zero_kill","spell_overkill","items_50_run",
                "sell_all","gold_zero","gold_50k","charged_item_use","shatter_epic",
                "full_sockets","divine_item","artifact_item"]),
            ("Crafting",        &["bench_warmer","craft_10","craft_100","craft_500","shatter_first",
                "imbue_first","engine_lock_3","reforge_legendary","augment_to_max",
                "transcendent_corruption","gone_reduced"]),
            ("Exploration",     &["portal_junkie","shrine_hopper","trap_magnet","all_room_types",
                "chaos_rift_3","clear_100_rooms","clear_1000_rooms","floor_25_no_gold",
                "no_items_floor50","gauntlet_flawless","portal_chain"]),
            ("Daily & Meta",    &["daily_driver","streak","daily_first","daily_win","daily_top3",
                "daily_rank1","daily_30","veteran","modder","runs_25","runs_50","runs_200","runs_500",
                "config_loaded","config_gold_bonus","config_hard_mode"]),
            ("OMEGA",           &["omega_tier","omega_long_run","omega_boss_rush","omega_the_algorithm",
                "chaos_all_classes"]),
        ];

        let mut cy = 7i32;
        for (cat_name, ids) in categories {
            if cy > 70 { break; }
            let cat_total = ids.len();
            let cat_done  = ids.iter().filter(|id| self.achievements.is_unlocked(id)).count();
            let cat_bar_w = 20i32;
            let cat_fill  = if cat_total > 0 { (cat_done as i32 * cat_bar_w) / cat_total as i32 } else { 0 };
            let col = if cat_done == cat_total { gld } else if cat_done > 0 { suc } else { dim };
            ctx.print_color(rx, cy, col, bg, &format!("{:.<18} {:2}/{}", cat_name, cat_done, cat_total));
            for i in 0..cat_bar_w {
                ctx.print_color(rx + 24 + i, cy, if i < cat_fill { suc } else { muted }, bg,
                    if i < cat_fill { "▪" } else { "·" });
            }
            cy += 1;
        }

        draw_separator(ctx, 80, cy + 1, 78, &t);

        // Rarity breakdown
        cy += 2;
        ctx.print_color(rx, cy, hd, bg, "By rarity:");
        cy += 1;
        let rarities = [
            ("Common",    AchievementRarity::Common),
            ("Uncommon",  AchievementRarity::Uncommon),
            ("Rare",      AchievementRarity::Rare),
            ("Epic",      AchievementRarity::Epic),
            ("Legendary", AchievementRarity::Legendary),
            ("Mythic",    AchievementRarity::Mythic),
            ("Omega",     AchievementRarity::Omega),
        ];
        for (name, rarity) in &rarities {
            if cy > 75 { break; }
            let rtup = rarity_rgb(rarity);
            let rc = RGB::from_u8(rtup.0, rtup.1, rtup.2);
            let r_total = all.iter().filter(|a| std::mem::discriminant(&a.rarity) == std::mem::discriminant(rarity)).count();
            let r_done  = all.iter().filter(|a| std::mem::discriminant(&a.rarity) == std::mem::discriminant(rarity)
                && self.achievements.is_unlocked(&a.id)).count();
            ctx.print_color(rx, cy, rc, bg, &format!("■ {:9} {:2}/{}", name, r_done, r_total));
            cy += 1;
        }

        // ── FOOTER ───────────────────────────────────────────────────────────
        draw_separator(ctx, 2, 72, 155, &t);
        print_hint(ctx, 2,  73, "↑↓",       " Scroll",           &t);
        print_hint(ctx, 16, 73, "PgUp/Dn",  " Page",             &t);
        print_hint(ctx, 38, 73, "Home",      " Top",              &t);
        print_hint(ctx, 50, 73, "[F]/Tab",   " Filter",           &t);
        print_hint(ctx, 70, 73, "Esc",       " Back",             &t);
    }

    fn draw_run_history(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg   = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd   = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac   = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let gld  = RGB::from_u8(t.gold.0,    t.gold.1,    t.gold.2);
        let dim  = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let suc  = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let dng  = RGB::from_u8(t.danger.0,  t.danger.1,  t.danger.2);
        let muted = RGB::from_u8(t.muted.0,  t.muted.1,   t.muted.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "RUN HISTORY", &t);

        let runs = self.run_history.runs.clone();
        let total = runs.len();

        ctx.print_color(4, 2, hd, bg,
            &format!("Last {} runs  (newest first) — ↑↓ to scroll", total.min(100)));

        draw_separator(ctx, 2, 3, 155, &t);

        // Column headers
        ctx.print_color(4,   4, ac, bg, "Date");
        ctx.print_color(16,  4, ac, bg, "Name");
        ctx.print_color(28,  4, ac, bg, "Class");
        ctx.print_color(42,  4, ac, bg, "Flr");
        ctx.print_color(47,  4, ac, bg, "Score");
        ctx.print_color(58,  4, ac, bg, "Kills");
        ctx.print_color(65,  4, ac, bg, "Mode");
        ctx.print_color(74,  4, ac, bg, "Diff");
        ctx.print_color(82,  4, ac, bg, "Gold");
        ctx.print_color(90,  4, ac, bg, "Tier");
        ctx.print_color(101, 4, ac, bg, "Result");
        ctx.print_color(108, 4, ac, bg, "Cause of death");

        draw_separator(ctx, 2, 5, 155, &t);

        let visible_rows = 62usize;
        let start = self.history_scroll.min(total.saturating_sub(1));
        let end   = (start + visible_rows).min(total);

        for (row, rec) in runs[start..end].iter().enumerate() {
            let y = 6 + row as i32;
            let result_col = if rec.won { suc } else { dng };
            let result_str = if rec.won { "WON " } else { "died" };

            let date_str: String = rec.date.chars().take(10).collect();
            ctx.print_color(4,   y, muted, bg, &date_str);
            ctx.print_color(16,  y, hd,    bg, &rec.name.chars().take(10).collect::<String>());
            ctx.print_color(28,  y, dim,   bg, &rec.class.chars().take(12).collect::<String>());
            ctx.print_color(42,  y, gld,   bg, &format!("{}", rec.floor));
            ctx.print_color(47,  y, ac,    bg, &format!("{}", rec.score));
            ctx.print_color(58,  y, suc,   bg, &format!("{}", rec.kills));
            ctx.print_color(65,  y, dim,   bg, &rec.game_mode.chars().take(8).collect::<String>());
            ctx.print_color(74,  y, dim,   bg, &rec.difficulty.chars().take(6).collect::<String>());
            ctx.print_color(82,  y, gld,   bg, &format!("{}", rec.gold));
            ctx.print_color(90,  y, dim,   bg, &rec.power_tier.chars().take(10).collect::<String>());
            ctx.print_color(101, y, result_col, bg, result_str);
            if !rec.won {
                ctx.print_color(108, y, dim, bg, &rec.cause_of_death.chars().take(47).collect::<String>());
            }
        }

        // Scroll indicator
        if total > visible_rows {
            let pct = if total > 1 { start * 100 / (total - 1) } else { 0 };
            ctx.print_color(150, 6, dim, bg, &format!("{}%", pct));
        }

        draw_separator(ctx, 2, 74, 155, &t);
        print_hint(ctx, 4,  75, "↑↓",    " Scroll   ",    &t);
        print_hint(ctx, 18, 75, "Esc",   " Back to title", &t);
    }
}

// ─── DAILY LEADERBOARD SCREEN ────────────────────────────────────────────────

impl State {
    fn draw_daily_leaderboard(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg   = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd   = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac   = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let gld  = RGB::from_u8(t.gold.0,    t.gold.1,    t.gold.2);
        let dim  = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let suc  = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let dng  = RGB::from_u8(t.danger.0,  t.danger.1,  t.danger.2);
        let muted = RGB::from_u8(t.muted.0,  t.muted.1,   t.muted.2);

        self.chaos_bg(ctx);
        let today = chrono_date_simple();
        draw_panel(ctx, 0, 0, 159, 79, &format!("DAILY LEADERBOARD — {}", today), &t);

        // Today's seed
        let daily_seed = State::daily_seed();
        ctx.print_color(4, 2, muted, bg, &format!("Today's seed: {}   ", daily_seed));
        ctx.print_color(4, 3, dim, bg,
            "Same dungeon for everyone today. Rankings by score.");

        // Status line
        let status_col = if self.daily_status.starts_with("Error") || self.daily_status.starts_with("Fetch error") {
            dng
        } else if self.daily_status.starts_with("Submit") || self.daily_status.starts_with("Updated") {
            suc
        } else { dim };
        ctx.print_color(4, 4, status_col, bg, &self.daily_status.chars().take(70).collect::<String>());

        draw_separator(ctx, 2, 5, 155, &t);

        // My best today
        if let Some(best) = self.daily_store.best_for_today(&today) {
            ctx.print_color(4, 6, hd, bg, "Your best today:");
            ctx.print_color(4, 7, gld, bg, &format!(
                "  {}/{} — Floor {}  Score {}  Kills {}  {}",
                best.name, best.class, best.floor, best.score, best.kills,
                if best.won { "[WON]" } else { "" }
            ));
            draw_separator(ctx, 2, 8, 155, &t);
        }

        // Remote rows
        let rows_y_start = 9i32;
        if self.daily_rows.is_empty() {
            ctx.print_color(4, rows_y_start + 2, muted, bg,
                "No scores loaded. Press [R] to refresh, or play a Daily run to appear here.");
        } else {
            // Headers
            ctx.print_color(4,  rows_y_start, ac, bg, "Rank");
            ctx.print_color(11, rows_y_start, ac, bg, "Name");
            ctx.print_color(24, rows_y_start, ac, bg, "Class");
            ctx.print_color(38, rows_y_start, ac, bg, "Floor");
            ctx.print_color(45, rows_y_start, ac, bg, "Score");
            ctx.print_color(57, rows_y_start, ac, bg, "Kills");
            ctx.print_color(64, rows_y_start, ac, bg, "Result");
            draw_separator(ctx, 2, rows_y_start + 1, 155, &t);

            for (i, row) in self.daily_rows.iter().enumerate().take(62) {
                let y = rows_y_start + 2 + i as i32;
                if y > 72 { break; }
                let rank_col = match row.rank {
                    1 => gld,
                    2 => RGB::from_u8(192, 192, 192),
                    3 => RGB::from_u8(205, 127, 50),
                    _ => muted,
                };
                let result_col = if row.won { suc } else { dng };
                ctx.print_color(4,  y, rank_col, bg, &format!("#{:<4}", row.rank));
                ctx.print_color(11, y, hd,       bg, &row.name.chars().take(11).collect::<String>());
                ctx.print_color(24, y, dim,      bg, &row.class.chars().take(12).collect::<String>());
                ctx.print_color(38, y, gld,      bg, &format!("{}", row.floor));
                ctx.print_color(45, y, ac,       bg, &format!("{}", row.score));
                ctx.print_color(57, y, suc,      bg, &format!("{}", row.kills));
                ctx.print_color(64, y, result_col, bg, if row.won { "WON" } else { "died" });
            }
        }

        draw_separator(ctx, 2, 74, 155, &t);
        print_hint(ctx, 4,  75, "[R]",   " Refresh   ", &t);
        print_hint(ctx, 18, 75, "[Esc]", " Back to title", &t);
        ctx.print_color(40, 75, muted, bg,
            &format!("Endpoint: {}", &self.config.leaderboard.url.chars().take(70).collect::<String>()));
    }
}

// ─── BESTIARY SCREEN ─────────────────────────────────────────────────────────

impl State {
    fn draw_bestiary(&mut self, ctx: &mut BTerm) {
        use chaos_rpg_core::player_bestiary::PlayerBestiary;
        let t = self.theme_graded();
        let bg   = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd   = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac   = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let gld  = RGB::from_u8(t.gold.0,    t.gold.1,    t.gold.2);
        let dim  = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let dng  = RGB::from_u8(t.danger.0,  t.danger.1,  t.danger.2);
        let suc  = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let muted = RGB::from_u8(t.muted.0,  t.muted.1,   t.muted.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "BESTIARY", &t);

        let bestiary = PlayerBestiary::load();
        let entries = bestiary.sorted_for_display();
        let total_entries = entries.len();

        // ── Split-panel layout ────────────────────────────────────────────────
        // Left panel (cols 0-52): scrollable enemy list
        // Right panel (cols 54-159): selected enemy detail

        // Header
        ctx.print_color(2, 1, hd, bg, &format!("{} enemies encountered", total_entries));
        ctx.print_color(90, 1, muted, bg, "↑↓ Navigate  Enter/→ Select  ← Back  Esc Return");
        draw_separator(ctx, 1, 2, 157, &t);

        // ── Left: list panel ─────────────────────────────────────────────────
        draw_subpanel(ctx, 1, 3, 51, 70, "ENCOUNTERED", &t);

        let visible = 62usize;
        let max_scroll = total_entries.saturating_sub(visible);
        self.bestiary_scroll = self.bestiary_scroll.min(max_scroll);
        self.bestiary_selected = self.bestiary_selected.min(total_entries.saturating_sub(1));
        // Keep selected in view
        if self.bestiary_selected >= self.bestiary_scroll + visible {
            self.bestiary_scroll = self.bestiary_selected.saturating_sub(visible - 1);
        } else if self.bestiary_selected < self.bestiary_scroll {
            self.bestiary_scroll = self.bestiary_selected;
        }
        let start = self.bestiary_scroll;
        let end = (start + visible).min(total_entries);

        for (row, rec) in entries[start..end].iter().enumerate() {
            let abs_i = start + row;
            let rec = *rec;
            let y = 5 + row as i32;
            let is_sel = abs_i == self.bestiary_selected;
            let has_lore = chaos_rpg_core::lore::enemies::enemy_lore(&rec.name).is_some();
            let lore_marker = if has_lore { "★" } else { " " };
            let kill_col = if rec.times_killed > 0 { suc } else { muted };
            if is_sel {
                let bar_bg = RGB::from_u8(
                    (t.accent.0 as u16 * 15 / 100) as u8,
                    (t.accent.1 as u16 * 15 / 100) as u8,
                    (t.accent.2 as u16 * 15 / 100) as u8);
                for xi in 2..52 {
                    ctx.set(xi, y, bg, bar_bg, 32u16);
                }
                ctx.print_color(2, y, RGB::from_u8(t.selected.0, t.selected.1, t.selected.2), bg,
                    &format!("{}{} {}  ×{} killed",
                        cursor_char(self.frame), lore_marker,
                        &rec.name.chars().take(24).collect::<String>(),
                        rec.times_killed));
            } else {
                ctx.print_color(2, y, hd, bg,
                    &format!(" {} {:<24} {}", lore_marker,
                        &rec.name.chars().take(24).collect::<String>(),
                        rec.times_fought));
                ctx.print_color(42, y, kill_col, bg, &format!("×{}", rec.times_killed));
            }
        }

        // Scroll indicator
        if total_entries > visible {
            let pct = if total_entries > 1 { start * 100 / (total_entries - 1) } else { 0 };
            ctx.print_color(2, 68, muted, bg, &format!("{}/{} ({}%)", start + 1, total_entries, pct));
        }

        // ── Right: detail panel ───────────────────────────────────────────────
        draw_subpanel(ctx, 53, 3, 104, 70, "DETAIL", &t);

        if let Some(&rec) = entries.get(self.bestiary_selected) {
            ctx.print_color(55, 5, hd, bg, &rec.name.chars().take(60).collect::<String>());
            draw_separator(ctx, 54, 6, 102, &t);

            // Stats
            stat_line(ctx, 55, 7, "Encountered: ", &format!("{}", rec.times_fought), t.dim, &t);
            let kill_col = if rec.times_killed > 0 { t.success } else { t.muted };
            stat_line(ctx, 55, 8, "Kills:       ", &format!("{}", rec.times_killed), kill_col, &t);
            let death_col = if rec.times_killed_player > 0 { t.danger } else { t.dim };
            stat_line(ctx, 55, 9, "Deaths:      ", &format!("{}", rec.times_killed_player), death_col, &t);
            stat_line(ctx, 55, 10, "Max Damage:  ", &format!("{}", rec.max_damage_seen), t.danger, &t);

            draw_separator(ctx, 54, 11, 102, &t);

            // Lore entry
            if let Some(lore) = chaos_rpg_core::lore::enemies::enemy_lore(&rec.name) {
                ctx.print_color(55, 12, ac, bg, &format!("★ {}", lore.name));
                let mut ly = 14i32;
                let mut lline = String::new();
                for w in lore.description.split_whitespace() {
                    if lline.len() + w.len() + 1 > 100 {
                        ctx.print_color(55, ly, dim, bg, &lline);
                        lline = w.to_string(); ly += 1;
                    } else {
                        if !lline.is_empty() { lline.push(' '); }
                        lline.push_str(w);
                    }
                }
                if !lline.is_empty() { ctx.print_color(55, ly, dim, bg, &lline); }
            } else {
                ctx.print_color(55, 13, muted, bg, "(no lore entry — encounter more of this enemy to unlock)");
            }
        } else {
            ctx.print_color(55, 20, muted, bg, "No enemies recorded yet.");
            ctx.print_color(55, 21, dim, bg, "Enter combat rooms to populate the Bestiary.");
        }

        draw_separator(ctx, 1, 74, 157, &t);
        print_hint(ctx, 2,  75, "↑↓",  " Navigate  ", &t);
        print_hint(ctx, 18, 75, "Esc", " Back to title", &t);
    }

    fn draw_codex(&mut self, ctx: &mut BTerm) {
        use chaos_rpg_core::codex_progress::CodexProgress;
        use chaos_rpg_core::lore::codex::CODEX_ENTRIES;
        let t = self.theme_graded();
        let bg   = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd   = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac   = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let gld  = RGB::from_u8(t.gold.0,    t.gold.1,    t.gold.2);
        let dim  = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);
        let dng  = RGB::from_u8(t.danger.0,  t.danger.1,  t.danger.2);
        let suc  = RGB::from_u8(t.success.0, t.success.1, t.success.2);
        let muted = RGB::from_u8(t.muted.0,  t.muted.1,   t.muted.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "CODEX — THE PROOF", &t);

        let progress = CodexProgress::load();
        let total = CODEX_ENTRIES.len();
        let unlocked_count = progress.unlocked_entries.len();

        // Header
        ctx.print_color(2, 1, hd, bg, &format!("{}/{} entries unlocked", unlocked_count, total));
        ctx.print_color(90, 1, muted, bg, "↑↓ Navigate  Enter/→ Select  ← Back  Esc Return");
        draw_separator(ctx, 1, 2, 157, &t);

        // ── Left: list panel ─────────────────────────────────────────────────
        draw_subpanel(ctx, 1, 3, 51, 70, "ENTRIES", &t);

        let visible = 62usize;
        let max_scroll = total.saturating_sub(visible);
        self.codex_scroll = self.codex_scroll.min(max_scroll);
        self.codex_selected = self.codex_selected.min(total.saturating_sub(1));
        if self.codex_selected >= self.codex_scroll + visible {
            self.codex_scroll = self.codex_selected.saturating_sub(visible - 1);
        } else if self.codex_selected < self.codex_scroll {
            self.codex_scroll = self.codex_selected;
        }
        let start = self.codex_scroll;
        let end = (start + visible).min(total);

        for (row, entry) in CODEX_ENTRIES[start..end].iter().enumerate() {
            let abs_i = start + row;
            let y = 5 + row as i32;
            let is_sel = abs_i == self.codex_selected;
            let is_unlocked = progress.unlocked_entries.contains(entry.id);
            let lock_marker = if is_unlocked { "★" } else { "·" };
            let (name_col, lock_col) = if is_unlocked { (hd, suc) } else { (muted, muted) };
            let cat_str = format!("{:?}", entry.category);

            if is_sel {
                let bar_bg = RGB::from_u8(
                    (t.accent.0 as u16 * 15 / 100) as u8,
                    (t.accent.1 as u16 * 15 / 100) as u8,
                    (t.accent.2 as u16 * 15 / 100) as u8);
                for xi in 2..52 { ctx.set(xi, y, bg, bar_bg, 32u16); }
                ctx.print_color(2, y, RGB::from_u8(t.selected.0, t.selected.1, t.selected.2), bg,
                    &format!("{}{} {:<26} {}", cursor_char(self.frame), lock_marker,
                        &entry.title.chars().take(26).collect::<String>(),
                        &cat_str.chars().take(10).collect::<String>()));
            } else {
                ctx.print_color(2, y, lock_col, bg, lock_marker);
                ctx.print_color(4, y, name_col, bg,
                    &format!("{:<28} {}", &entry.title.chars().take(28).collect::<String>(),
                        &cat_str.chars().take(10).collect::<String>()));
            }
        }

        if total > visible {
            let pct = if total > 1 { start * 100 / (total - 1) } else { 0 };
            ctx.print_color(2, 68, muted, bg, &format!("{}/{} ({}%)", start + 1, total, pct));
        }

        // ── Right: detail panel ───────────────────────────────────────────────
        draw_subpanel(ctx, 53, 3, 104, 70, "ENTRY DETAIL", &t);

        if let Some(entry) = CODEX_ENTRIES.get(self.codex_selected) {
            let is_unlocked = progress.unlocked_entries.contains(entry.id);
            let cat_str = format!("{:?}", entry.category);
            ctx.print_color(55, 5, if is_unlocked { hd } else { muted }, bg,
                &entry.title.chars().take(60).collect::<String>());
            ctx.print_color(55, 6, gld, bg, &cat_str);
            ctx.print_color(120, 6, if is_unlocked { suc } else { dng }, bg,
                if is_unlocked { "★ UNLOCKED" } else { "· locked" });
            draw_separator(ctx, 54, 7, 102, &t);

            if is_unlocked {
                let mut ly = 9i32;
                let mut lline = String::new();
                for w in entry.body.split_whitespace() {
                    if lline.len() + w.len() + 1 > 100 {
                        ctx.print_color(55, ly, dim, bg, &lline);
                        lline = w.to_string(); ly += 1;
                        if ly > 70 { break; }
                    } else {
                        if !lline.is_empty() { lline.push(' '); }
                        lline.push_str(w);
                    }
                }
                if !lline.is_empty() && ly <= 70 { ctx.print_color(55, ly, dim, bg, &lline); }
            } else {
                ctx.print_color(55, 9, muted, bg, "[ LOCKED ]");
                ctx.print_color(55, 11, dim, bg, "Unlock this entry by progressing through the game.");
            }
        } else {
            ctx.print_color(55, 20, muted, bg, "Select an entry to view it.");
        }

        draw_separator(ctx, 1, 74, 157, &t);
        print_hint(ctx, 2,  75, "↑↓",  " Navigate  ", &t);
        print_hint(ctx, 18, 75, "Esc", " Back to title", &t);
    }

    // ── SETTINGS ─────────────────────────────────────────────────────────────

    fn draw_settings(&mut self, ctx: &mut BTerm) {
        let t = self.theme_graded();
        let bg  = RGB::from_u8(t.bg.0,      t.bg.1,      t.bg.2);
        let hd  = RGB::from_u8(t.heading.0, t.heading.1, t.heading.2);
        let ac  = RGB::from_u8(t.accent.0,  t.accent.1,  t.accent.2);
        let dim = RGB::from_u8(t.dim.0,     t.dim.1,     t.dim.2);

        self.chaos_bg(ctx);
        draw_panel(ctx, 0, 0, 159, 79, "SETTINGS", &t);

        let ox = 30i32; let oy = 12i32;
        draw_subpanel(ctx, ox - 2, oy - 2, 100, 24, "AUDIO", &t);

        // Music Vibe row
        ctx.print_color(ox, oy,     hd,  bg, "Music Vibe");
        ctx.print_color(ox, oy + 1, dim, bg, "Changes the mood and volume of the music.");
        let vibe_label = self.music_vibe.display_name();
        let arrow_col = ac;
        ctx.print_color(ox,     oy + 3, arrow_col, bg, "◄");
        ctx.print_color(ox + 2, oy + 3, hd,        bg, vibe_label);
        ctx.print_color(ox + 2 + vibe_label.len() as i32 + 1, oy + 3, arrow_col, bg, "►");
        ctx.print_color(ox, oy + 4, dim, bg, "Left/Right to cycle. Changes take effect immediately.");

        // Volume note
        ctx.print_color(ox, oy + 7, hd,  bg, "Tip");
        ctx.print_color(ox, oy + 8, dim, bg, "For fine-grained volume control, add a chaos_config.toml");
        ctx.print_color(ox, oy + 9, dim, bg, "next to the exe: [audio] music_volume = 0.8");

        // Theme row (already exists via T key, just document it)
        draw_separator(ctx, ox - 2, oy + 12, 98, &t);
        ctx.print_color(ox, oy + 13, hd,  bg, "Visual Theme");
        ctx.print_color(ox, oy + 14, dim, bg, "Press [T] anywhere on the title screen to cycle themes.");

        draw_separator(ctx, 2, 73, 155, &t);
        print_hint(ctx, 2,  74, "◄►",  " Change Vibe", &t);
        print_hint(ctx, 20, 74, "T",   " Theme",       &t);
        print_hint(ctx, 32, 74, "Esc", " Back",        &t);
    }
}

// ─── ENTRY POINT ─────────────────────────────────────────────────────────────

fn main() -> BError {
    let builder = BTermBuilder::simple(160, 80)?
        .with_title("CHAOS RPG — Where Math Goes To Die")
        .with_tile_dimensions(12, 12)
        .with_dimensions(160, 80)
        .with_fps_cap(60.0)
        .with_fullscreen(true);
    main_loop(builder.build()?, State::new())
}
