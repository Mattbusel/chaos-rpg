//! CHAOS RPG — Web Frontend (macroquad)
//!
//! Renders CHAOS RPG using macroquad, targeting both native desktop and
//! WebAssembly (`cargo build --target wasm32-unknown-unknown -p chaos-rpg-web`).
//!
//! All game logic runs through `chaos-rpg-core`. This crate only handles
//! display, input, and particle effects.

use macroquad::prelude::*;
use chaos_rpg_core::{
    character::{Character, CharacterClass, Background, Difficulty},
    chaos_pipeline::chaos_roll_verbose,
    enemy::{generate_enemy, Enemy},
    scoreboard::load_scores,
    world::generate_floor,
};

// ── Screen states ─────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone)]
enum AppScreen {
    Title,
    CharacterCreation,
    FloorNav,
    Combat,
    GameOver,
    Scoreboard,
}

// ── Particle system ───────────────────────────────────────────────────────────

struct Particle {
    x: f32, y: f32,
    vx: f32, vy: f32,
    life: f32,
    color: Color,
    size: f32,
}

impl Particle {
    fn new(x: f32, y: f32, color: Color) -> Self {
        let angle = rand::gen_range(0.0f32, std::f32::consts::TAU);
        let speed = rand::gen_range(40.0f32, 120.0);
        Self {
            x, y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            life: 1.0,
            color,
            size: rand::gen_range(3.0f32, 8.0),
        }
    }

    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.vy += 60.0 * dt;
        self.life -= dt;
    }

    fn draw(&self) {
        let c = Color::new(self.color.r, self.color.g, self.color.b, self.life.max(0.0));
        draw_rectangle(self.x - self.size / 2.0, self.y - self.size / 2.0, self.size, self.size, c);
    }
}

// ── Damage float ─────────────────────────────────────────────────────────────

struct DamageFloat {
    x: f32, y: f32,
    text: String,
    color: Color,
    ttl: f32,
}

impl DamageFloat {
    fn new(x: f32, y: f32, amount: i64, is_heal: bool) -> Self {
        let color = if is_heal { GREEN } else { RED };
        let text = if is_heal { format!("+{}", amount) } else { format!("-{}", amount) };
        Self { x, y, text, color, ttl: 1.2 }
    }

    fn update(&mut self, dt: f32) {
        self.y -= 40.0 * dt;
        self.ttl -= dt;
    }

    fn draw(&self) {
        let alpha = (self.ttl / 1.2).max(0.0);
        let c = Color::new(self.color.r, self.color.g, self.color.b, alpha);
        draw_text(&self.text, self.x, self.y, 20.0, c);
    }
}

// ── App state ─────────────────────────────────────────────────────────────────

struct AppState {
    screen: AppScreen,
    player: Option<Character>,
    enemy: Option<Enemy>,
    floor_num: u32,
    particles: Vec<Particle>,
    damage_floats: Vec<DamageFloat>,
    combat_log: Vec<String>,
    selected_class: usize,
    game_over_msg: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            screen: AppScreen::Title,
            player: None,
            enemy: None,
            floor_num: 1,
            particles: Vec::new(),
            damage_floats: Vec::new(),
            combat_log: Vec::new(),
            selected_class: 0,
            game_over_msg: String::new(),
        }
    }

    fn spawn_particles(&mut self, x: f32, y: f32, count: usize, color: Color) {
        for _ in 0..count {
            self.particles.push(Particle::new(x, y, color));
        }
    }

    fn push_damage_float(&mut self, x: f32, y: f32, amount: i64, is_heal: bool) {
        self.damage_floats.push(DamageFloat::new(x, y, amount, is_heal));
    }
}

// ── UI helpers ────────────────────────────────────────────────────────────────

const FONT_SIZE: f32 = 18.0;
const TITLE_SIZE: f32 = 36.0;
const PAD: f32 = 12.0;
const PANEL_BG: Color = Color { r: 0.05, g: 0.05, b: 0.12, a: 0.92 };
const PANEL_BORDER: Color = Color { r: 0.3, g: 0.3, b: 0.6, a: 1.0 };
const HEADER_COLOR: Color = Color { r: 0.8, g: 0.7, b: 1.0, a: 1.0 };

fn draw_panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, PANEL_BG);
    draw_rectangle_lines(x, y, w, h, 2.0, PANEL_BORDER);
    if !title.is_empty() {
        draw_text(title, x + PAD, y + FONT_SIZE + PAD / 2.0, FONT_SIZE, HEADER_COLOR);
        draw_line(x, y + FONT_SIZE + PAD * 1.5, x + w, y + FONT_SIZE + PAD * 1.5, 1.0, PANEL_BORDER);
    }
}

fn draw_hp_bar(x: f32, y: f32, w: f32, h: f32, current: i64, max: i64) {
    draw_rectangle(x, y, w, h, Color::new(0.1, 0.1, 0.1, 1.0));
    let ratio = if max > 0 { (current as f32 / max as f32).clamp(0.0, 1.0) } else { 0.0 };
    let color = if ratio > 0.5 { GREEN } else if ratio > 0.25 { YELLOW } else { RED };
    draw_rectangle(x, y, w * ratio, h, color);
    draw_rectangle_lines(x, y, w, h, 1.0, DARKGRAY);
}

fn button(x: f32, y: f32, w: f32, h: f32, label: &str) -> bool {
    let (mx, my) = mouse_position();
    let hover = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let bg = if hover { Color::new(0.3, 0.2, 0.5, 1.0) } else { Color::new(0.15, 0.1, 0.3, 1.0) };
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.5, Color::new(0.6, 0.4, 0.9, 1.0));
    let tw = measure_text(label, None, FONT_SIZE as u16, 1.0).width;
    draw_text(label, x + (w - tw) / 2.0, y + h / 2.0 + FONT_SIZE / 3.0, FONT_SIZE, WHITE);
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn centered_text(text: &str, y: f32, size: f32, color: Color) {
    let tw = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, (screen_width() - tw) / 2.0, y, size, color);
}

// ── Screen renderers ──────────────────────────────────────────────────────────

fn draw_title(state: &mut AppState) {
    clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
    let (sw, sh) = (screen_width(), screen_height());

    // Animated starfield
    let t = get_time() as f32;
    for i in 0..80u32 {
        let fx = (i as f32 * 137.5 + t * (1.0 + i as f32 * 0.01)) % sw;
        let fy = (i as f32 * 97.3) % sh;
        let bright = 0.3 + 0.7 * ((t + i as f32) * 2.0).sin().abs();
        draw_circle(fx, fy, 1.0, Color::new(bright, bright, bright, 0.6));
    }

    centered_text("C H A O S   R P G", sh * 0.25, TITLE_SIZE, Color::new(0.8, 0.4, 1.0, 1.0));
    centered_text("Where Math Goes To Die", sh * 0.33, FONT_SIZE, Color::new(0.5, 0.5, 0.8, 1.0));

    let (bw, bh) = (200.0, 40.0);
    let bx = (sw - bw) / 2.0;

    if button(bx, sh * 0.5,          bw, bh, "[ NEW GAME ]")    { state.screen = AppScreen::CharacterCreation; }
    if button(bx, sh * 0.5 + 55.0,   bw, bh, "[ SCOREBOARD ]")  { state.screen = AppScreen::Scoreboard; }
    if button(bx, sh * 0.5 + 110.0,  bw, bh, "[ QUIT ]")        { std::process::exit(0); }
}

fn draw_char_creation(state: &mut AppState) {
    clear_background(Color::new(0.03, 0.03, 0.1, 1.0));
    let (sw, sh) = (screen_width(), screen_height());
    draw_panel(20.0, 20.0, sw - 40.0, sh - 40.0, "CHARACTER CREATION");

    let classes = [
        ("Mage",        CharacterClass::Mage),
        ("Berserker",   CharacterClass::Berserker),
        ("Ranger",      CharacterClass::Ranger),
        ("Thief",       CharacterClass::Thief),
        ("Necromancer", CharacterClass::Necromancer),
        ("Alchemist",   CharacterClass::Alchemist),
        ("Paladin",     CharacterClass::Paladin),
        ("VoidWalker",  CharacterClass::VoidWalker),
    ];

    draw_text("Choose Class:", 40.0, 80.0, FONT_SIZE, HEADER_COLOR);
    let (mx, my) = mouse_position();
    for (i, (name, _class)) in classes.iter().enumerate() {
        let col = (i % 4) as f32;
        let row = (i / 4) as f32;
        let bx = 40.0 + col * 160.0;
        let by = 100.0 + row * 50.0;
        let selected = i == state.selected_class;
        let bg = if selected { Color::new(0.3, 0.15, 0.5, 1.0) } else { Color::new(0.1, 0.08, 0.2, 1.0) };
        draw_rectangle(bx, by, 150.0, 36.0, bg);
        draw_rectangle_lines(bx, by, 150.0, 36.0, 1.5,
            if selected { Color::new(0.8, 0.4, 1.0, 1.0) } else { PANEL_BORDER });
        draw_text(name, bx + 8.0, by + 24.0, FONT_SIZE, WHITE);
        if mx >= bx && mx <= bx + 150.0 && my >= by && my <= by + 36.0
            && is_mouse_button_pressed(MouseButton::Left) {
            state.selected_class = i;
        }
    }

    if button((sw - 200.0) / 2.0, sh - 80.0, 200.0, 40.0, "[ START ADVENTURE ]") {
        let chosen_class = classes[state.selected_class].1.clone();
        let seed = macroquad::miniquad::date::now() as u64;
        let player = Character::roll_new(
            "Hero".to_string(),
            chosen_class,
            Background::Wanderer,
            seed,
            Difficulty::Normal,
        );
        let _floor = generate_floor(state.floor_num, seed);
        state.enemy = Some(generate_enemy(state.floor_num, seed ^ 0xDEAD_BEEF));
        state.player = Some(player);
        state.screen = AppScreen::FloorNav;
    }
    if button(20.0, sh - 80.0, 120.0, 40.0, "[ BACK ]") {
        state.screen = AppScreen::Title;
    }
}

fn draw_floor_nav(state: &mut AppState) {
    clear_background(Color::new(0.02, 0.04, 0.02, 1.0));
    let (sw, sh) = (screen_width(), screen_height());

    let (max_hp, current_hp, p_name, p_class, p_kills) = match &state.player {
        Some(p) => (p.max_hp, p.current_hp, p.name.clone(), format!("{:?}", p.class), p.kills),
        None => { state.screen = AppScreen::Title; return; }
    };

    draw_panel(10.0, 10.0, sw * 0.65, sh - 20.0, &format!("FLOOR {}", state.floor_num));
    draw_panel(sw * 0.67, 10.0, sw * 0.31, sh * 0.45, "PLAYER");

    let px = sw * 0.67 + PAD;
    draw_text(&format!("{} — {}", p_name, p_class), px, 60.0, FONT_SIZE, WHITE);
    draw_text(&format!("Floor: {}  Kills: {}", state.floor_num, p_kills), px, 82.0, 14.0, GRAY);
    draw_text("HP", px, 108.0, 14.0, Color::new(0.8, 0.3, 0.3, 1.0));
    draw_hp_bar(px + 30.0, 96.0, sw * 0.31 - PAD * 2.0 - 30.0, 14.0, current_hp, max_hp);

    let nb_y = sh * 0.6;
    if button(30.0, nb_y, 180.0, 40.0, "[ ENTER ROOM ]") {
        let seed = macroquad::miniquad::date::now() as u64;
        state.enemy = Some(generate_enemy(state.floor_num, seed));
        state.combat_log.clear();
        state.screen = AppScreen::Combat;
    }
    if button(30.0, nb_y + 55.0, 180.0, 40.0, "[ NEXT FLOOR ]") {
        state.floor_num += 1;
        if let Some(ref mut p) = state.player { p.floor = state.floor_num; }
        let seed = macroquad::miniquad::date::now() as u64;
        state.enemy = Some(generate_enemy(state.floor_num, seed));
    }
    if button(30.0, nb_y + 110.0, 180.0, 40.0, "[ MAIN MENU ]") {
        state.screen = AppScreen::Title;
    }
}

fn draw_combat(state: &mut AppState) {
    clear_background(Color::new(0.05, 0.02, 0.08, 1.0));
    let (sw, sh) = (screen_width(), screen_height());

    if state.player.is_none() || state.enemy.is_none() {
        state.screen = AppScreen::FloorNav;
        return;
    }

    // Enemy panel
    draw_panel(10.0, 10.0, sw * 0.45, sh * 0.45, "ENEMY");
    {
        let e = state.enemy.as_ref().unwrap();
        draw_text(&e.name, 20.0, 60.0, FONT_SIZE, Color::new(1.0, 0.4, 0.4, 1.0));
        draw_text(&format!("Tier: {:?}", e.tier), 20.0, 82.0, 14.0, GRAY);
        draw_text("HP", 20.0, 108.0, 14.0, Color::new(0.8, 0.3, 0.3, 1.0));
        draw_hp_bar(50.0, 96.0, sw * 0.45 - 70.0, 16.0, e.hp, e.max_hp);
    }

    // Player panel
    let py = sh * 0.47;
    draw_panel(10.0, py, sw * 0.45, sh * 0.35, "PLAYER");
    {
        let p = state.player.as_ref().unwrap();
        draw_text(&p.name, 20.0, py + 40.0, FONT_SIZE, WHITE);
        draw_text("HP", 20.0, py + 66.0, 14.0, Color::new(0.8, 0.3, 0.3, 1.0));
        draw_hp_bar(50.0, py + 54.0, sw * 0.45 - 70.0, 16.0, p.current_hp, p.max_hp);
    }

    // Combat log
    draw_panel(sw * 0.47, 10.0, sw * 0.51, sh * 0.8, "COMBAT LOG");
    let log_start = state.combat_log.len().saturating_sub(18);
    for (i, line) in state.combat_log[log_start..].iter().enumerate() {
        draw_text(line, sw * 0.47 + PAD, 60.0 + i as f32 * 20.0, 14.0, Color::new(0.75, 0.75, 0.85, 1.0));
    }

    // Action buttons
    let ab_y = sh * 0.87;
    let ab_w = 110.0;

    if button(10.0, ab_y, ab_w, 40.0, "[ ATTACK ]") {
        let roll = chaos_roll_verbose(42.0, macroquad::miniquad::date::now() as u64);
        let player_force = state.player.as_ref().unwrap().stats.force.max(1);
        let enemy_name = state.enemy.as_ref().unwrap().name.clone();
        let enemy_attack_mod = state.enemy.as_ref().unwrap().attack_modifier;

        let dmg = ((player_force as f64 * roll.final_value.abs() / 50.0) as i64).max(1);
        if let Some(ref mut e) = state.enemy { e.hp -= dmg; }
        state.combat_log.push(format!("You attack {} for {} dmg", enemy_name, dmg));
        state.spawn_particles(sw * 0.22, sh * 0.22, 12, Color::new(1.0, 0.3, 0.3, 1.0));
        state.push_damage_float(sw * 0.22, sh * 0.15, dmg, false);

        let enemy_alive = state.enemy.as_ref().map(|e| e.hp > 0).unwrap_or(false);
        if enemy_alive {
            let roll2 = chaos_roll_verbose(99.0, macroquad::miniquad::date::now() as u64 ^ 0xFFFF);
            let dmg2 = ((enemy_attack_mod as f64 * roll2.final_value.abs() / 60.0) as i64).max(1);
            if let Some(ref mut p) = state.player { p.current_hp -= dmg2; }
            state.combat_log.push(format!("{} hits you for {} dmg", enemy_name, dmg2));
            state.push_damage_float(sw * 0.22, sh * 0.5, dmg2, false);
        }

        let enemy_dead = state.enemy.as_ref().map(|e| e.hp <= 0).unwrap_or(false);
        let player_dead = state.player.as_ref().map(|p| p.current_hp <= 0).unwrap_or(false);

        if enemy_dead {
            let xp = state.floor_num as u64 * 20 + 10;
            if let Some(ref mut p) = state.player {
                p.kills += 1;
                p.gain_xp(xp);
            }
            state.combat_log.push(format!("Enemy defeated! +{} XP", xp));
            state.enemy = None;
            state.screen = AppScreen::FloorNav;
        } else if player_dead {
            state.game_over_msg = "You have been defeated.".to_string();
            state.screen = AppScreen::GameOver;
        }
    }

    if button(10.0 + 120.0, ab_y, ab_w, 40.0, "[ HEAL ]") {
        if let Some(ref mut p) = state.player {
            let heal = (p.max_hp / 5).max(10);
            p.current_hp = (p.current_hp + heal).min(p.max_hp);
            state.combat_log.push(format!("You heal for {} HP", heal));
            state.push_damage_float(sw * 0.22, sh * 0.55, heal, true);
        }
    }

    if button(10.0 + 240.0, ab_y, ab_w, 40.0, "[ FLEE ]") {
        state.combat_log.push("You flee from combat!".to_string());
        state.screen = AppScreen::FloorNav;
    }
}

fn draw_game_over(state: &mut AppState) {
    clear_background(Color::new(0.08, 0.02, 0.02, 1.0));
    let sh = screen_height();

    centered_text("GAME OVER", sh * 0.3, TITLE_SIZE, Color::new(0.9, 0.2, 0.2, 1.0));
    centered_text(&state.game_over_msg.clone(), sh * 0.45, FONT_SIZE, Color::new(0.8, 0.7, 0.7, 1.0));

    if let Some(ref p) = state.player {
        let stats = format!("Floor: {}  Kills: {}  XP: {}", state.floor_num, p.kills, p.xp);
        centered_text(&stats, sh * 0.55, FONT_SIZE, GRAY);
    }

    let sw = screen_width();
    if button((sw - 180.0) / 2.0, sh * 0.7, 180.0, 40.0, "[ MAIN MENU ]") {
        state.player = None;
        state.enemy = None;
        state.floor_num = 1;
        state.combat_log.clear();
        state.screen = AppScreen::Title;
    }
}

fn draw_scoreboard(state: &mut AppState) {
    clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
    let (sw, sh) = (screen_width(), screen_height());
    draw_panel(20.0, 20.0, sw - 40.0, sh - 40.0, "HALL OF LEGENDS");

    let scores = load_scores();
    if scores.is_empty() {
        draw_text("No scores yet. Go die heroically.", 40.0, 100.0, FONT_SIZE, GRAY);
    } else {
        let col_x = [40.0f32, 80.0, 200.0, 340.0, 460.0, 560.0];
        let headers = ["#", "Name", "Class", "Score", "Floor", "Enemies"];
        for (j, h) in headers.iter().enumerate() {
            draw_text(h, col_x[j], 80.0, 14.0, HEADER_COLOR);
        }
        draw_line(40.0, 90.0, sw - 40.0, 90.0, 1.0, PANEL_BORDER);
        for (i, entry) in scores.iter().take(20).enumerate() {
            let y = 110.0 + i as f32 * 22.0;
            let row_color = match i {
                0 => Color::new(1.0, 0.85, 0.2, 1.0),
                1 => Color::new(0.8, 0.8, 0.8, 1.0),
                2 => Color::new(0.8, 0.5, 0.2, 1.0),
                _ => Color::new(0.7, 0.7, 0.8, 1.0),
            };
            draw_text(&format!("{}", i + 1),              col_x[0], y, 14.0, row_color);
            draw_text(&entry.name,                         col_x[1], y, 14.0, row_color);
            draw_text(&format!("{:?}", entry.class),       col_x[2], y, 14.0, row_color);
            draw_text(&format!("{}", entry.score),         col_x[3], y, 14.0, row_color);
            draw_text(&format!("{}", entry.floor_reached), col_x[4], y, 14.0, row_color);
            draw_text(&format!("{}", entry.enemies_defeated), col_x[5], y, 14.0, row_color);
        }
    }

    if button(20.0, sh - 70.0, 120.0, 40.0, "[ BACK ]") {
        state.screen = AppScreen::Title;
    }
}

// ── Main loop ─────────────────────────────────────────────────────────────────

#[macroquad::main("CHAOS RPG")]
async fn main() {
    let mut state = AppState::new();
    loop {
        let dt = get_frame_time();

        for p in &mut state.particles { p.update(dt); }
        state.particles.retain(|p| p.life > 0.0);
        for f in &mut state.damage_floats { f.update(dt); }
        state.damage_floats.retain(|f| f.ttl > 0.0);

        match state.screen.clone() {
            AppScreen::Title            => draw_title(&mut state),
            AppScreen::CharacterCreation => draw_char_creation(&mut state),
            AppScreen::FloorNav         => draw_floor_nav(&mut state),
            AppScreen::Combat           => draw_combat(&mut state),
            AppScreen::GameOver         => draw_game_over(&mut state),
            AppScreen::Scoreboard       => draw_scoreboard(&mut state),
        }

        for p in &state.particles { p.draw(); }
        for f in &state.damage_floats { f.draw(); }

        next_frame().await;
    }
}
