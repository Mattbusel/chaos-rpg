mod ui;
mod ratatui_screens;

use chaos_rpg_audio::AudioSystem;
use chaos_rpg_core::{
    audio_events::{AudioEvent, MusicVibe},
    bosses::{boss_name, random_unique_boss, run_unique_boss, BossOutcome},
    chaos_pipeline::chaos_roll_verbose,
    character::{Character, CharacterClass, Difficulty as GameDifficulty},
    combat::{resolve_action, CombatAction, CombatOutcome, CombatState},
    enemy::{generate_enemy, Enemy, FloorAbility},
    items::Item,
    legacy_system::{GraveyardEntry, LegacyData},
    misery_system::{MiserySource, SpiteAction},
    nemesis::{load_nemesis, save_nemesis, NemesisRecord},
    npcs::shop_npc,
    scoreboard::{load_misery_scores, save_misery_score, save_score, MiseryEntry, ScoreEntry},
    skill_checks::{perform_skill_check, Difficulty, SkillType},
    world::{generate_floor, room_enemy, Room, RoomType},
};
use ui::GameMode;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_seed() -> u64 {
    if let Ok(seed_str) = std::env::var("CHAOS_SEED") {
        if let Ok(seed) = seed_str.trim().parse::<u64>() {
            return seed;
        }
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

/// Derive a seed that is identical for everyone on the same calendar day (UTC).
/// Format: days since Unix epoch × a prime. Stable for a full 24h UTC window.
fn daily_seed() -> u64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let day = secs / 86400; // calendar day (UTC)
    day.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

fn main() {
    let _fullscreen = ui::FullscreenGuard::enter();

    loop {
        ui::show_title();
        let mode = ui::select_mode();

        match mode {
            GameMode::Quit => {
                println!("\n  {}The chaos subsides. For now.{}", ui::DIM, ui::RESET);
                return;
            }
            GameMode::Scoreboard => {
                let scores = chaos_rpg_core::scoreboard::load_scores();
                ui::show_scoreboard(&scores);
            }
            GameMode::Bestiary => {
                ui::show_bestiary();
            }
            GameMode::Codex => {
                ui::show_codex();
            }
            GameMode::Achievements => {
                ui::show_achievements();
            }
            GameMode::Story | GameMode::Infinite | GameMode::DailySeed => {
                run_game(mode);
            }
        }
    }
}

use std::cell::RefCell;

thread_local! {
    static AUDIO: RefCell<Option<AudioSystem>> = RefCell::new(None);
}

fn emit_audio(ev: AudioEvent) {
    AUDIO.with(|a| { if let Some(ref s) = *a.borrow() { s.emit(ev); } });
}

fn run_game(mode: GameMode) {
    let cfg = chaos_rpg_core::chaos_config::ChaosConfig::load();
    let vibe = MusicVibe::from_str(&cfg.audio.music_vibe);
    AUDIO.with(|a| {
        let sys = AudioSystem::try_new();
        if let Some(ref s) = sys { s.set_vibe(vibe); }
        *a.borrow_mut() = sys;
    });

    let help = ui::prompt("  Show tutorial? [y/N] >");
    if help.eq_ignore_ascii_case("y") {
        ui::show_help();
    }

    let auto_ans = ui::prompt("  Auto-play mode? (AI handles combat/navigation, pauses for items/shop) [y/N] >");
    if auto_ans.trim().eq_ignore_ascii_case("y") {
        ui::set_auto_mode(true);
        println!("  {}[AUTO MODE ON] — type 'z' on the floor to toggle off.{}", ui::GREEN, ui::RESET);
        println!();
    }

    let (name, class, background, game_difficulty): (_, _, _, GameDifficulty) =
        ui::create_character_ui();
    let seed = if mode == GameMode::DailySeed {
        let s = daily_seed();
        println!(
            "  {}Daily Seed: {}{}{} — everyone shares this seed today.{}",
            ui::DIM,
            ui::CYAN,
            s,
            ui::DIM,
            ui::RESET
        );
        println!();
        s
    } else {
        current_seed()
    };
    let mut player = Character::roll_new(name, class, background, seed, game_difficulty);

    // Boon selection
    let boon = ui::show_boon_select(seed);
    player.apply_boon(boon);

    ui::clear_screen();
    println!("\n  {}Destiny roll complete.{}", ui::YELLOW, ui::RESET);
    println!("  The 10 sacred algorithms have determined your fate.");
    println!();
    ui::show_character_sheet(&player);
    println!();

    let intro = match player.class {
        CharacterClass::Mage => {
            "You emerged from the fractal abyss, equations swirling in your wake."
        }
        CharacterClass::Berserker => "The Lorenz attractor knows your rage. It feeds on it.",
        CharacterClass::Ranger => {
            "You read the prime spirals in every shadow. Nothing escapes you."
        }
        CharacterClass::Thief => {
            "Logistic map r=3.9: you are the period-doubling cascade no one sees coming."
        }
        CharacterClass::Necromancer => {
            "The dead remember everything. You remember the equations they used to die."
        }
        CharacterClass::Alchemist => {
            "Chaos is just unoptimized chemistry. You have the formula — and the flask."
        }
        CharacterClass::Paladin => {
            "Order is a lie the universe tells itself. You are the one constant in the storm."
        }
        CharacterClass::VoidWalker => {
            "Between the Mandelbrot set's boundary and infinity, you found a door. You walked through."
        }
        CharacterClass::Warlord => {
            "Armies have risen at your word. Now the chaos itself must answer to your authority."
        }
        CharacterClass::Trickster => {
            "Every shadow is a misdirection. Every step is a lie the enemy believes."
        }
        CharacterClass::Runesmith => {
            "You carve equations into steel. The weapon becomes the algorithm."
        }
        CharacterClass::Chronomancer => {
            "Time is a variable. You are its derivative."
        }
    };
    println!("  {}{}{}", ui::MAGENTA, intro, ui::RESET);
    println!();
    ui::press_enter(&format!(
        "  {}Begin your descent [ENTER]...{}",
        ui::DIM,
        ui::RESET
    ));

    let max_floor = if mode == GameMode::Story {
        10u32
    } else {
        u32::MAX
    };
    let mode_str = match mode {
        GameMode::DailySeed => "Daily",
        GameMode::Story => "Story",
        _ => "Infinite",
    };
    let daily_banner = if mode == GameMode::DailySeed {
        format!(
            "{}[DAILY RACE]{} Seed fixed for all players today.",
            ui::CYAN,
            ui::RESET
        )
    } else {
        String::new()
    };
    let mut last_roll: Option<chaos_rpg_core::chaos_pipeline::ChaosRollResult> = None;
    let mut floor_seed = seed;
    // Load any nemesis from a previous run
    let mut nemesis_record = load_nemesis();
    let mut nemesis_spawned = false;

    'game: loop {
        floor_seed = floor_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(player.floor as u64 * 31337);

        let mut floor = generate_floor(player.floor, floor_seed);
        let is_cursed_floor = player.floor > 0 && player.floor % 25 == 0;

        emit_audio(AudioEvent::FloorEntered { floor: player.floor, seed: floor_seed });
        if is_cursed_floor { emit_audio(AudioEvent::CursedFloorActivated); }

        // ── Item Volatility: every 20 floors, re-roll a random item ──────────
        if player.floor > 0 && player.floor % 20 == 0 && !player.inventory.is_empty() {
            let vol_idx = (floor_seed % player.inventory.len() as u64) as usize;
            let old_name = player.inventory[vol_idx].name.clone();
            player.inventory[vol_idx] = Item::generate(floor_seed.wrapping_add(0x766F6C6174696C65));
            let new_name = player.inventory[vol_idx].name.clone();
            emit_audio(AudioEvent::ItemVolatilityReroll);
            println!();
            println!("  {}⚡ ITEM VOLATILITY ⚡{}", ui::RED, ui::RESET);
            println!("  The math reshapes your gear.");
            println!("  {} → {}", old_name, new_name);
            println!("  (Floor {} volatility tick)", player.floor);
            println!();
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
        }

        ui::clear_screen();
        ui::show_floor_header(player.floor, &mode);
        if !daily_banner.is_empty() {
            println!("  {}", daily_banner);
            println!();
        }
        // Floor entry lore text (milestone floors and occasionally beyond floor 100)
        if let Some(lore_text) = chaos_rpg_core::lore::events::floor_transition_flavor(player.floor, floor_seed) {
            println!("  {}{}{}", ui::DIM, lore_text, ui::RESET);
            println!();
        }

        // ── Cursed floor warning ──────────────────────────────────────────────
        if is_cursed_floor {
            println!("  {}╔══════════════════════════════════════════╗{}", ui::RED, ui::RESET);
            println!("  {}║        ☠  CURSED FLOOR ☠                ║{}", ui::RED, ui::RESET);
            println!("  {}║  ALL engine outputs INVERTED this floor  ║{}", ui::RED, ui::RESET);
            println!("  {}║  Your strengths work against you.        ║{}", ui::RED, ui::RESET);
            println!("  {}║  Backfired spells become your best tool. ║{}", ui::RED, ui::RESET);
            println!("  {}╚══════════════════════════════════════════╝{}", ui::RED, ui::RESET);
            println!();
        }

        if mode == GameMode::Story {
            if let Some(event) = ui::story_event(player.floor, floor_seed) {
                println!("{}", event);
                println!();
            }
        }

        // Corruption status
        if player.corruption_stage() > 0 {
            println!(
                "  {}Corruption:{} {} [{}/{} kills to next mutation]",
                ui::RED, ui::RESET,
                player.corruption_label(),
                player.kills % 50,
                50
            );
        }

        // The Hunger warning (floor 50+)
        if player.floor >= 50 && player.rooms_without_kill >= 3 {
            let rooms_left = 5u32.saturating_sub(player.rooms_without_kill);
            println!(
                "  {}THE HUNGER: {} room(s) without a kill. {} more and you lose 5% max HP permanently.{}",
                ui::RED, player.rooms_without_kill, rooms_left, ui::RESET
            );
        }

        println!(
            "  HP: {}  Gold: {}  Floor: {}  Kills: {}",
            player.hp_bar(16),
            player.gold,
            player.floor,
            player.kills
        );
        println!();
        println!("  {}Map:{} {}", ui::DIM, ui::RESET, floor.minimap());
        println!(
            "  {}[x]=Combat [*]=Treasure [$]=Shop [~]=Shrine [!]=Trap [B]=Boss [^]=Portal [ ]=Empty [?]=Rift{}",
            ui::DIM, ui::RESET
        );
        println!();
        ui::press_enter(&format!(
            "  {}[ENTER] to begin floor {}...{}",
            ui::DIM,
            player.floor,
            ui::RESET
        ));

        'rooms: loop {
            let room = floor.current().clone();

            ui::clear_screen();
            println!(
                "  {}Floor {} — Room {}/{}{}",
                ui::YELLOW,
                player.floor,
                floor.current_room + 1,
                floor.rooms.len(),
                ui::RESET
            );
            println!("  {}", floor.minimap());
            println!();

            for line in room.ascii_border() {
                println!("  {}", line);
            }
            println!();

            println!(
                "  HP: {}  Gold: {}  Kills: {}",
                player.hp_bar(16),
                player.gold,
                player.kills
            );
            println!();
            println!("  {}[E]{} Enter room   {}[C]{} Character   {}[B]{} Body chart",
                ui::GREEN, ui::RESET, ui::CYAN, ui::RESET, ui::YELLOW, ui::RESET);
            println!("  {}[P]{} Skill tree   {}[F]{} Factions    {}[T]{} Last trace",
                ui::MAGENTA, ui::RESET, ui::BRIGHT_CYAN, ui::RESET, ui::DIM, ui::RESET);
            println!("  {}[I]{} Equipment    — view/equip/unequip items",
                ui::GREEN, ui::RESET);
            if floor.rooms_remaining() == 0 {
                println!(
                    "  {}[D] Descend to floor {}{}",
                    ui::CYAN,
                    player.floor + 1,
                    ui::RESET
                );
            }
            if ui::is_auto_mode() {
                println!("  {}[AUTO PILOT ON — Z to stop]{}", ui::GREEN, ui::RESET);
            }
            println!();

            // Auto-play: spend any pending passive points, then enter room / descend
            let input = if ui::is_auto_mode() {
                if player.skill_points > 0 {
                    let msgs = player.auto_allocate_passives(
                        floor_seed.wrapping_add(player.rooms_cleared as u64 * 777));
                    for m in &msgs {
                        println!("  {}[AUTO] {}{}", ui::GREEN, m, ui::RESET);
                    }
                }
                if floor.rooms_remaining() == 0 {
                    println!("  {}[AUTO] All rooms done — descending.{}", ui::DIM, ui::RESET);
                    "d".to_string()
                } else {
                    println!("  {}[AUTO] Entering next room...{}", ui::DIM, ui::RESET);
                    "e".to_string()
                }
            } else {
                ui::prompt("  > ").to_lowercase()
            };

            match input.trim() {
                "c" => {
                    ui::clear_screen();
                    ui::show_character_sheet(&player);
                    ui::show_character_lore_section(&player.character_lore);
                    println!("  {}[L] Edit character lore  [ENTER] Continue{}", ui::DIM, ui::RESET);
                    let cs_input = ui::prompt("  > ");
                    if cs_input.trim().eq_ignore_ascii_case("l") {
                        player.character_lore = ui::show_lore_editor(&player);
                    }
                    continue 'rooms;
                }
                "b" => {
                    ui::clear_screen();
                    println!();
                    println!("  {}=== BODY CHART ==={}", ui::YELLOW, ui::RESET);
                    println!();
                    for line in player.body.display_lines() {
                        println!("  {}", line);
                    }
                    println!();
                    println!("  {}{}", ui::DIM, player.body.combat_summary());
                    println!("{}", ui::RESET);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    continue 'rooms;
                }
                "p" => {
                    ui::show_passive_tree_ui(&mut player, floor_seed);
                    continue 'rooms;
                }
                "f" => {
                    ui::clear_screen();
                    ui::show_faction_rep(&player);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    continue 'rooms;
                }
                "t" => {
                    if let Some(ref roll) = last_roll {
                        for line in roll.display_lines() {
                            println!("{}", line);
                        }
                    } else {
                        println!("  {}No chaos roll yet.{}", ui::DIM, ui::RESET);
                    }
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    continue 'rooms;
                }
                "i" => {
                    ui::show_equipment_screen(&mut player);
                    continue 'rooms;
                }
                "z" => {
                    let new_state = !ui::is_auto_mode();
                    ui::set_auto_mode(new_state);
                    if new_state {
                        println!("  {}[AUTO MODE ON]{}", ui::GREEN, ui::RESET);
                    } else {
                        println!("  {}[Auto mode off]{}", ui::DIM, ui::RESET);
                        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    }
                    continue 'rooms;
                }
                "d" if floor.rooms_remaining() == 0 => {
                    player.floor += 1;
                    if player.floor > max_floor {
                        emit_audio(AudioEvent::Victory);
                        ui::show_victory(&player);
                        println!();
                        for line in player.run_summary() {
                            println!("{}", line);
                        }
                        println!();
                        end_game_score(&player, true, mode_str);
                        return;
                    }
                    break 'rooms;
                }
                _ => {
                    let room_seed = floor_seed.wrapping_add(floor.current_room as u64 * 9973);
                    let kills_before = player.kills;

                    let outcome = handle_room(
                        &room,
                        &mut player,
                        room_seed,
                        &mut last_roll,
                        is_cursed_floor,
                        &nemesis_record,
                        &mut nemesis_spawned,
                        mode_str,
                    );

                    // ── The Hunger: track rooms without a kill (floor 50+) ───
                    if player.floor >= 50 {
                        if player.kills > kills_before {
                            player.rooms_without_kill = 0;
                        } else {
                            player.rooms_without_kill += 1;
                            if player.rooms_without_kill >= 5 {
                                let loss = (player.max_hp / 20).max(1);
                                player.max_hp = (player.max_hp - loss).max(1);
                                if player.current_hp > player.max_hp {
                                    player.current_hp = player.max_hp;
                                }
                                player.rooms_without_kill = 0;
                                println!();
                                println!("  {}THE HUNGER CLAIMS {} MAX HP (now {}).{}", ui::RED, loss, player.max_hp, ui::RESET);
                                println!("  {}5 rooms without a kill. Feed the hunger.{}", ui::DIM, ui::RESET);
                                if !player.is_alive() {
                                    println!("  {}THE HUNGER KILLS YOU.{}", ui::RED, ui::RESET);
                                    ui::show_game_over(&player);
                                    save_nemesis_on_death(&player, "THE HUNGER", player.floor, &mut nemesis_record);
                                    end_game_score(&player, false, mode_str);
                                    return;
                                }
                                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                            }
                        }
                    }

                    match outcome {
                        RoomOutcome::PlayerDied => {
                            ui::show_game_over(&player);
                            println!();
                            for line in player.run_summary() {
                                println!("{}", line);
                            }
                            println!();
                            end_game_score(&player, false, mode_str);
                            return;
                        }
                        RoomOutcome::PortalTaken => {
                            player.rooms_cleared += 1;
                            player.floor += 1;
                            if player.floor > max_floor {
                                ui::show_victory(&player);
                                println!();
                                for line in player.run_summary() {
                                    println!("{}", line);
                                }
                                println!();
                                end_game_score(&player, true, mode_str);
                                return;
                            }
                            break 'rooms;
                        }
                        RoomOutcome::Continue => {
                            player.rooms_cleared += 1;
                            if floor.rooms_remaining() > 0 {
                                floor.advance();
                            }
                        }
                    }
                    continue 'rooms;
                }
            }
        }

        if player.floor > max_floor {
            break 'game;
        }
    }
}

// ─── ROOM OUTCOMES ───────────────────────────────────────────────────────────

enum RoomOutcome {
    Continue,
    PlayerDied,
    PortalTaken,
}

// ─── ROOM DISPATCHER ─────────────────────────────────────────────────────────

fn save_nemesis_on_death(
    player: &Character,
    killer_name: &str,
    floor: u32,
    nemesis_record: &mut Option<NemesisRecord>,
) {
    let kill_method = if player.spells_cast > player.kills * 2 {
        "spell"
    } else {
        "physical"
    };
    let class_name = player.class.name().to_string();
    if let Some(ref mut existing) = nemesis_record {
        if existing.enemy_name == killer_name {
            existing.escalate();
            save_nemesis(existing);
            return;
        }
    }
    let new_nemesis = NemesisRecord::new(
        killer_name.to_string(),
        floor,
        20 + floor as i64 * 3,
        class_name,
        kill_method,
    );
    save_nemesis(&new_nemesis);
    *nemesis_record = Some(new_nemesis);
}

fn handle_room(
    room: &Room,
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<chaos_rpg_core::chaos_pipeline::ChaosRollResult>,
    is_cursed: bool,
    nemesis_record: &Option<NemesisRecord>,
    nemesis_spawned: &mut bool,
    mode_str: &str,
) -> RoomOutcome {
    match room.room_type {
        RoomType::Combat => {
            // Nemesis spawn: check if nemesis should appear
            if !*nemesis_spawned {
                if let Some(ref nemesis) = nemesis_record {
                    let spawn_roll = seed.wrapping_mul(0x6E656D65_73697300) % 100;
                    // Spawn chance: 20% after floor 3, guaranteed if at nemesis floor
                    let spawn_chance = if player.floor >= nemesis.floor_killed_at { 40 } else { 20 };
                    if player.floor >= 3 && spawn_roll < spawn_chance {
                        *nemesis_spawned = true;
                        return do_nemesis_encounter(player, nemesis, seed, last_roll, is_cursed, mode_str);
                    }
                }
            }

            // After floor 50: 20% chance any combat becomes a unique boss
            // After floor 100: every 3rd room is a boss
            let unique_boss_roll = seed.wrapping_mul(0x756E6971_75650000) % 100;
            let spawn_unique = (player.floor >= 100 && player.rooms_cleared % 3 == 0)
                || (player.floor >= 50 && unique_boss_roll < 20);

            if spawn_unique {
                if let Some(boss_id) = random_unique_boss(player.floor, seed) {
                    return do_unique_boss_encounter(player, boss_id, seed, last_roll);
                }
            }

            let mut enemy = room_enemy(room);
            // StatMirror: HP = player's highest stat
            if enemy.floor_ability == FloorAbility::StatMirror {
                let (stat_name, stat_val) = player.highest_stat();
                enemy.hp = stat_val.max(1);
                enemy.max_hp = enemy.hp;
                println!("  {}⚠ STAT MIRROR: This enemy copied your {} ({}) as its HP!{}",
                    ui::RED, stat_name, stat_val, ui::RESET);
            }
            do_combat_encounter(player, &mut enemy, seed, last_roll, false, is_cursed, mode_str)
        }

        RoomType::Boss => {
            // Every 10 floors: gauntlet (3 fights back-to-back, no healing)
            let is_gauntlet = player.floor % 10 == 0;

            // Boss every 5 floors: check for unique boss
            let use_unique = player.floor % 5 == 0;
            if use_unique {
                if let Some(boss_id) = random_unique_boss(player.floor, seed) {
                    if is_gauntlet {
                        return do_boss_gauntlet(player, seed, last_roll, is_cursed, Some(boss_id), mode_str);
                    }
                    return do_unique_boss_encounter(player, boss_id, seed, last_roll);
                }
            }

            let mut enemy = room_enemy(room);
            enemy.hp = (enemy.hp as f64 * 2.5) as i64;
            enemy.max_hp = enemy.hp;
            enemy.base_damage = (enemy.base_damage as f64 * 1.8) as i64;
            enemy.xp_reward *= 3;
            enemy.gold_reward *= 3;

            if is_gauntlet {
                do_boss_gauntlet(player, seed, last_roll, is_cursed, None, mode_str)
            } else {
                do_combat_encounter(player, &mut enemy, seed, last_roll, true, is_cursed, mode_str)
            }
        }

        RoomType::Treasure => {
            let item = Item::generate(seed);
            let gold_bonus = (seed % 30 + 10) as i64 * player.floor as i64;

            println!("  {}* TREASURE ROOM *{}", ui::YELLOW, ui::RESET);
            println!("  {}{}{}", ui::DIM, chaos_rpg_core::lore::world::treasure_room_flavor(seed), ui::RESET);
            println!();
            for line in item.display_box() {
                println!("  {}", line);
            }
            // Show item flavor text
            if !item.flavor_text.is_empty() {
                println!("  {}  \u{201c}{}\u{201d}{}", ui::DIM, item.flavor_text, ui::RESET);
            }
            println!();
            println!("  {}You find {} gold!{}", ui::YELLOW, gold_bonus, ui::RESET);
            player.gold += gold_bonus;

            println!();
            let pick = ui::prompt(&format!(
                "  {}[P] Pick up item  [any] Leave it  > {}",
                ui::CYAN, ui::RESET
            ));
            if pick.trim().eq_ignore_ascii_case("p") {
                for modifier in &item.stat_modifiers {
                    apply_stat_modifier(player, &modifier.stat, modifier.value);
                }
                player.add_item(item);
                println!(
                    "  {}Item added to inventory! (Use [I#] in combat){}",
                    ui::GREEN,
                    ui::RESET
                );
            } else {
                println!("  {}You leave the item behind.{}", ui::DIM, ui::RESET);
            }

            // 25% chance to also find a spell scroll
            if seed.is_multiple_of(4) {
                let spell = chaos_rpg_core::spells::Spell::generate(seed.wrapping_add(54321));
                println!();
                println!("  {}+ SPELL SCROLL FOUND +{}", ui::CYAN, ui::RESET);
                for line in spell.display_box() {
                    println!("  {}", line);
                }
                println!();
                let pick_spell = ui::prompt(&format!(
                    "  {}[L] Learn spell  [any] Leave it  > {}",
                    ui::CYAN, ui::RESET
                ));
                if pick_spell.trim().eq_ignore_ascii_case("l") {
                    player.add_spell(spell);
                    println!(
                        "  {}Spell learned! Use [S#] in combat.{}",
                        ui::CYAN,
                        ui::RESET
                    );
                } else {
                    println!("  {}You leave the scroll behind.{}", ui::DIM, ui::RESET);
                }
            }

            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }

        RoomType::Shop => {
            emit_audio(AudioEvent::ShopEntered);
            if ui::is_auto_mode() {
                println!("  {}[AUTO] Shop skipped — auto mode active. Type 'z' on the floor to disable.{}", ui::DIM, ui::RESET);
                return RoomOutcome::Continue;
            }
            let mut npc = shop_npc(player.floor, seed);
            println!("  {}$ SHOP ${}", ui::CYAN, ui::RESET);
            println!();
            println!("  {}", npc.greeting());
            println!();

            let heal_cost = 15 + player.floor as i64 * 2;
            println!(
                "  [H] Healing potion — {}{}g{} (+40 HP)",
                ui::YELLOW,
                heal_cost,
                ui::RESET
            );
            println!();

            for (i, item) in npc.inventory.iter().enumerate() {
                let price = npc.sale_price(item.value, player.stats.cunning);
                let rarity_color = item.rarity.color_code();
                println!(
                    "  [{}] {}{}{} — {}{}g{}",
                    i + 1,
                    rarity_color,
                    item.name,
                    ui::RESET,
                    ui::YELLOW,
                    price,
                    ui::RESET
                );
                for m in &item.stat_modifiers {
                    let sign = if m.value >= 0 { "+" } else { "" };
                    println!(
                        "      {}  {}: {}{}{}",
                        ui::DIM,
                        m.stat,
                        sign,
                        m.value,
                        ui::RESET
                    );
                }
            }

            println!();
            println!("  [0] Leave shop");
            println!();
            println!("  {}Your gold: {}{}", ui::YELLOW, player.gold, ui::RESET);
            println!();

            loop {
                let input = ui::prompt("  Buy > ");
                let trimmed = input.trim().to_string();

                if trimmed == "0" || trimmed.eq_ignore_ascii_case("leave") {
                    break;
                }

                if trimmed.eq_ignore_ascii_case("h") {
                    if player.gold >= heal_cost {
                        player.gold -= heal_cost;
                        player.heal_scaled(40); // potions respect anti-heal scaling (floor 50+)
                        println!("  {}You drink the potion. +40 HP.{}", ui::GREEN, ui::RESET);
                    } else {
                        println!(
                            "  {}Need {}g. You have {}g.{}",
                            ui::RED,
                            heal_cost,
                            player.gold,
                            ui::RESET
                        );
                    }
                    continue;
                }

                if let Ok(idx) = trimmed.parse::<usize>() {
                    if idx >= 1 && idx <= npc.inventory.len() {
                        let item = npc.inventory[idx - 1].clone();
                        let price = npc.sale_price(item.value, player.stats.cunning);
                        if player.gold >= price {
                            player.gold -= price;
                            npc.inventory.remove(idx - 1);
                            // Weapons/armor go to inventory for combat use;
                            // consumables apply immediately and are consumed.
                            if item.is_weapon || item.stat_modifiers.is_empty() {
                                println!(
                                    "  {}Purchased {}! Added to inventory.{}",
                                    ui::GREEN,
                                    item.name,
                                    ui::RESET
                                );
                                player.add_item(item);
                            } else {
                                // Consumable: apply modifiers now
                                for modifier in item.stat_modifiers.clone() {
                                    apply_stat_modifier(player, &modifier.stat, modifier.value);
                                }
                                println!(
                                    "  {}Used {}! Stats updated.{}",
                                    ui::GREEN,
                                    item.name,
                                    ui::RESET
                                );
                            }
                            println!("  {}Your gold: {}{}", ui::YELLOW, player.gold, ui::RESET);
                        } else {
                            println!(
                                "  {}Need {}g, have {}g.{}",
                                ui::RED,
                                price,
                                player.gold,
                                ui::RESET
                            );
                        }
                    }
                }
            }
            RoomOutcome::Continue
        }

        RoomType::Shrine => {
            println!("  {}~ SHRINE ~{}", ui::MAGENTA, ui::RESET);
            println!();
            println!("  {}", room.description);
            println!();

            let roll = chaos_roll_verbose(player.stats.entropy as f64 * 0.01, seed);
            *last_roll = Some(roll.clone());

            for line in roll.display_lines() {
                println!("{}", line);
            }

            let stats = [
                "vitality",
                "force",
                "mana",
                "cunning",
                "precision",
                "entropy",
                "luck",
            ];
            let stat_idx = (seed % stats.len() as u64) as usize;
            let stat_name = stats[stat_idx];
            let buff = 3 + roll.to_range(1, 10) as i64 + player.floor as i64 / 2;

            apply_stat_modifier(player, stat_name, buff);
            println!(
                "  {}The shrine blesses you! +{} {}!{}",
                ui::MAGENTA,
                buff,
                stat_name,
                ui::RESET
            );

            let hp_restore = player.max_hp / 5;
            player.heal(hp_restore);
            println!(
                "  {}You feel restored. +{} HP.{}",
                ui::GREEN,
                hp_restore,
                ui::RESET
            );

            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }

        RoomType::Trap => {
            println!("  {}! TRAP ROOM !{}", ui::RED, ui::RESET);
            println!();
            println!("  {}", room.description);
            println!();

            let diff = match player.floor {
                1..=3 => Difficulty::Easy,
                4..=7 => Difficulty::Medium,
                _ => Difficulty::Hard,
            };
            let check = perform_skill_check(player, SkillType::Perception, diff, seed);
            *last_roll = Some(check.chaos_result.clone());

            for line in check.display_lines() {
                println!("{}", line);
            }

            if check.passed {
                println!("  {}You spot and avoid the trap!{}", ui::GREEN, ui::RESET);
            } else {
                let trap_damage = 5 + player.floor as i64 * 3 + (seed % 10) as i64;
                player.take_damage(trap_damage);
                println!(
                    "  {}TRAP TRIGGERED! -{} HP!{}",
                    ui::RED,
                    trap_damage,
                    ui::RESET
                );
                if !player.is_alive() {
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return RoomOutcome::PlayerDied;
                }
            }

            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }

        RoomType::Portal => {
            println!("  {}^ PORTAL ^{}", ui::CYAN, ui::RESET);
            println!();
            println!("  {}", room.description);
            println!();

            let take = if ui::is_auto_mode() {
                println!("  {}[AUTO] Stepping through portal!{}", ui::DIM, ui::RESET);
                true
            } else {
                println!("  Step through to the next floor? [Y/N]");
                println!();
                ui::prompt("  > ").trim().eq_ignore_ascii_case("y")
            };
            if take {
                println!("  {}You step through the portal!{}", ui::CYAN, ui::RESET);
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return RoomOutcome::PortalTaken;
            }
            println!("  {}You resist the portal's pull.{}", ui::DIM, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }

        RoomType::Empty => {
            println!("  {}  EMPTY ROOM  {}", ui::DIM, ui::RESET);
            println!();
            println!("  {}", room.description);
            println!();

            let hp_gain = 5 + player.floor as i64 * 2;
            player.heal(hp_gain);
            println!(
                "  {}The stillness restores you. +{} HP.{}",
                ui::GREEN,
                hp_gain,
                ui::RESET
            );

            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }

        RoomType::ChaosRift => {
            println!("  {}? CHAOS RIFT ?{}", ui::MAGENTA, ui::RESET);
            println!();
            println!("  REALITY ERROR. MATHEMATICAL EXCEPTION.");
            println!();

            let roll = chaos_roll_verbose(player.stats.entropy as f64 * 0.015, seed);
            *last_roll = Some(roll.clone());

            for line in roll.display_lines() {
                println!("{}", line);
            }

            let outcome_idx = seed.wrapping_mul(player.floor as u64 * 7 + 1) % 6;
            match outcome_idx {
                0 => {
                    let gold = (seed % 100 + 50) as i64 * player.floor as i64;
                    player.gold += gold;
                    println!("  {}CHAOS BOUNTY: +{} gold!{}", ui::YELLOW, gold, ui::RESET);
                }
                1 => {
                    let damage = (player.max_hp / 4).max(1);
                    player.take_damage(damage);
                    println!(
                        "  {}CHAOS PUNISHMENT: -{} HP!{}",
                        ui::RED,
                        damage,
                        ui::RESET
                    );
                    if !player.is_alive() {
                        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                        return RoomOutcome::PlayerDied;
                    }
                }
                2 => {
                    let stat_bonus = 5 + player.floor as i64;
                    apply_stat_modifier(player, "entropy", stat_bonus);
                    println!(
                        "  {}CHAOS ASCENSION: +{} Entropy!{}",
                        ui::MAGENTA,
                        stat_bonus,
                        ui::RESET
                    );
                }
                3 => {
                    let heal = player.max_hp / 3;
                    player.heal_scaled(heal); // chaos blessings respect anti-heal
                    println!("  {}CHAOS BLESSING: +{} HP!{}", ui::GREEN, heal, ui::RESET);
                }
                4 => {
                    let gold_loss = player.gold / 4;
                    player.gold -= gold_loss;
                    let stat_gain = 10 + player.floor as i64;
                    apply_stat_modifier(player, "luck", stat_gain);
                    println!(
                        "  {}CHAOS TRADE: -{} gold, +{} Luck!{}",
                        ui::YELLOW,
                        gold_loss,
                        stat_gain,
                        ui::RESET
                    );
                }
                _ => {
                    for stat in &[
                        "vitality",
                        "force",
                        "mana",
                        "cunning",
                        "precision",
                        "entropy",
                        "luck",
                    ] {
                        apply_stat_modifier(player, stat, 1);
                    }
                    println!("  {}CHAOS HARMONY: All stats +1!{}", ui::MAGENTA, ui::RESET);
                }
            }

            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }

        RoomType::CraftingBench => {
            if ui::is_auto_mode() {
                println!("  {}[AUTO] Crafting bench skipped.{}", ui::DIM, ui::RESET);
                return RoomOutcome::Continue;
            }
            do_crafting_bench(player, seed, last_roll);
            RoomOutcome::Continue
        }
    }
}

// ─── CRAFTING ────────────────────────────────────────────────────────────────

/// 6 crafting operations, each using the chaos pipeline on items in inventory.
fn do_crafting_bench(
    player: &mut Character,
    seed: u64,
    _last_roll: &mut Option<chaos_rpg_core::chaos_pipeline::ChaosRollResult>,
) {
    use chaos_rpg_core::chaos_pipeline::chaos_roll_verbose;

    println!("  {}⚒  CRAFTING BENCH  ⚒{}", ui::CYAN, ui::RESET);
    println!();
    println!("  Mathematical tools await. Your items can be remade.");
    println!();

    if player.inventory.is_empty() {
        println!(
            "  {}Your inventory is empty — nothing to craft with.{}",
            ui::DIM,
            ui::RESET
        );
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
        return;
    }

    // Show inventory
    println!("  {}Inventory:{}", ui::YELLOW, ui::RESET);
    for (i, item) in player.inventory.iter().enumerate() {
        let rc = item.rarity.color_code();
        println!("  [{}] {}{}{}", i + 1, rc, item.name, ui::RESET);
    }
    println!();

    // Pick item
    let item_input = ui::prompt("  Select item # (or 0 to leave) > ");
    let item_idx = match item_input.trim().parse::<usize>() {
        Ok(0) => {
            println!("  {}You leave the bench.{}", ui::DIM, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return;
        }
        Ok(n) if n >= 1 && n <= player.inventory.len() => n - 1,
        _ => {
            println!("  {}Invalid selection.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return;
        }
    };

    println!();
    println!("  {}Operations:{}", ui::CYAN, ui::RESET);
    println!(
        "  [1] {}Reforge{}   — chaos-reroll all stat modifiers on item",
        ui::YELLOW,
        ui::RESET
    );
    println!(
        "  [2] {}Augment{}   — add one new chaos-rolled stat modifier",
        ui::GREEN,
        ui::RESET
    );
    println!(
        "  [3] {}Annul{}     — remove one random stat modifier",
        ui::RED,
        ui::RESET
    );
    println!(
        "  [4] {}Corrupt{}   — unpredictable chaos effect (can be good or bad)",
        ui::MAGENTA,
        ui::RESET
    );
    println!(
        "  [5] {}Fuse{}      — double item value, combine rarity tier",
        ui::BRIGHT_CYAN,
        ui::RESET
    );
    println!(
        "  [6] {}EngineLock{}— lock the item's chaos engine (costs 40g)",
        ui::BRIGHT_MAGENTA,
        ui::RESET
    );
    println!("  [0] Cancel");
    println!();

    let op_input = ui::prompt("  Operation > ");
    let roll = chaos_roll_verbose(player.stats.entropy as f64 * 0.01, seed);

    match op_input.trim() {
        "1" => {
            // Reforge: chaos-reroll all stat modifiers
            let item = &mut player.inventory[item_idx];
            let n = item.stat_modifiers.len().max(1);
            item.stat_modifiers.clear();
            for j in 0..n {
                let mod_seed = seed
                    .wrapping_add(j as u64 * 17777)
                    .wrapping_mul(6364136223846793005);
                let new_mod = chaos_rpg_core::items::StatModifier::generate_random(mod_seed);
                item.stat_modifiers.push(new_mod);
            }
            println!(
                "  {}REFORGED! {} modifiers chaos-rolled anew.{}",
                ui::YELLOW,
                n,
                ui::RESET
            );
        }
        "2" => {
            // Augment: add a new modifier
            let item = &mut player.inventory[item_idx];
            let aug_seed = seed
                .wrapping_mul(0xdeadbeef)
                .wrapping_add(item.value as u64);
            let new_mod = chaos_rpg_core::items::StatModifier::generate_random(aug_seed);
            let stat = new_mod.stat.clone();
            let val = new_mod.value;
            item.stat_modifiers.push(new_mod);
            item.value = (item.value as f64 * 1.2) as i64;
            println!(
                "  {}AUGMENTED! Added: {} {:+}{} to {}{}{}.",
                ui::GREEN,
                ui::YELLOW,
                val,
                ui::RESET,
                ui::GREEN,
                stat,
                ui::RESET
            );
        }
        "3" => {
            // Annul: remove one random modifier
            let item = &mut player.inventory[item_idx];
            if item.stat_modifiers.is_empty() {
                println!("  {}No modifiers to remove.{}", ui::DIM, ui::RESET);
            } else {
                let remove_idx = (seed % item.stat_modifiers.len() as u64) as usize;
                let removed = item.stat_modifiers.remove(remove_idx);
                println!(
                    "  {}ANNULLED: removed {} {:+}.{}",
                    ui::RED,
                    removed.stat,
                    removed.value,
                    ui::RESET
                );
            }
        }
        "4" => {
            // Corrupt: chaotic outcome
            let item = &mut player.inventory[item_idx];
            let outcome = roll.to_range(0, 5);
            match outcome {
                0 => {
                    // Gain socket
                    if item.socket_count < 6 {
                        item.socket_count += 1;
                        println!("  {}CORRUPTED: +1 socket!{}", ui::MAGENTA, ui::RESET);
                    } else {
                        println!(
                            "  {}CORRUPTED: item glows... but nothing changes.{}",
                            ui::DIM,
                            ui::RESET
                        );
                    }
                }
                1 => {
                    // Double a modifier
                    if !item.stat_modifiers.is_empty() {
                        let idx2 =
                            (seed.wrapping_add(99) % item.stat_modifiers.len() as u64) as usize;
                        item.stat_modifiers[idx2].value *= 2;
                        println!(
                            "  {}CORRUPTED: a modifier was doubled!{}",
                            ui::MAGENTA,
                            ui::RESET
                        );
                    } else {
                        println!(
                            "  {}CORRUPTED: sparks fly but do nothing.{}",
                            ui::DIM,
                            ui::RESET
                        );
                    }
                }
                2 => {
                    // Add corruption tag
                    item.corruption = Some("Chaos-Touched".to_string());
                    let val_bonus = (item.value as f64 * 0.5) as i64;
                    item.value += val_bonus;
                    println!(
                        "  {}CORRUPTED: item is now Chaos-Touched (+50% value)!{}",
                        ui::MAGENTA,
                        ui::RESET
                    );
                }
                3 => {
                    // Lose one modifier
                    if !item.stat_modifiers.is_empty() {
                        item.stat_modifiers.pop();
                        println!(
                            "  {}CORRUPTED: a modifier dissolved into void.{}",
                            ui::RED,
                            ui::RESET
                        );
                    }
                }
                4 => {
                    // Negate all modifiers
                    for m in &mut item.stat_modifiers {
                        m.value = -m.value;
                    }
                    println!(
                        "  {}CORRUPTED: all modifiers INVERTED!{}",
                        ui::RED,
                        ui::RESET
                    );
                }
                _ => {
                    // Mirror: flip is_weapon flag
                    item.is_weapon = !item.is_weapon;
                    println!(
                        "  {}CORRUPTED: item type transmogrified!{}",
                        ui::MAGENTA,
                        ui::RESET
                    );
                }
            }
        }
        "5" => {
            // Fuse: double value, upgrade rarity
            let item = &mut player.inventory[item_idx];
            item.value *= 2;
            use chaos_rpg_core::items::Rarity;
            item.rarity = match item.rarity {
                Rarity::Common => Rarity::Uncommon,
                Rarity::Uncommon => Rarity::Rare,
                Rarity::Rare => Rarity::Epic,
                Rarity::Epic => Rarity::Legendary,
                Rarity::Legendary => Rarity::Mythical,
                Rarity::Mythical => Rarity::Divine,
                Rarity::Divine => Rarity::Beyond,
                Rarity::Beyond | Rarity::Artifact => Rarity::Artifact,
            };
            println!(
                "  {}FUSED! Value doubled, rarity upgraded to {}{}{}.{}",
                ui::BRIGHT_CYAN,
                item.rarity.color_code(),
                item.rarity.name(),
                ui::BRIGHT_CYAN,
                ui::RESET
            );
        }
        "6" => {
            // Engine Lock: costs 40g
            let cost = 40 + player.floor as i64 * 5;
            if player.gold < cost {
                println!(
                    "  {}Not enough gold. Need {}g, have {}g.{}",
                    ui::RED,
                    cost,
                    player.gold,
                    ui::RESET
                );
            } else {
                player.gold -= cost;
                let engines = [
                    "Lorenz",
                    "Zeta",
                    "Collatz",
                    "Mandelbrot",
                    "Fibonacci",
                    "Euler",
                    "Linear",
                    "SharpEdge",
                    "Orbit",
                    "Recursive",
                ];
                let engine_idx = (seed % engines.len() as u64) as usize;
                let locked_engine = engines[engine_idx].to_string();
                let item = &mut player.inventory[item_idx];
                item.engine_locks.push(locked_engine.clone());
                println!(
                    "  {}ENGINE LOCKED: {} engine embedded into {}.{}",
                    ui::BRIGHT_MAGENTA,
                    locked_engine,
                    item.name,
                    ui::RESET
                );
            }
        }
        _ => {
            println!("  {}Cancelled.{}", ui::DIM, ui::RESET);
        }
    }

    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
}

// ─── COMBAT ──────────────────────────────────────────────────────────────────

// ─── UNIQUE BOSS / NEMESIS / GAUNTLET DISPATCHERS ────────────────────────────

fn do_unique_boss_encounter(
    player: &mut Character,
    boss_id: u8,
    seed: u64,
    last_roll: &mut Option<chaos_rpg_core::chaos_pipeline::ChaosRollResult>,
) -> RoomOutcome {
    ui::clear_screen();
    println!("\n  {}╔══════════════════════════════════╗{}", ui::RED, ui::RESET);
    println!("  {}║   ★  UNIQUE BOSS ENCOUNTER  ★   ║{}", ui::RED, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::RED, ui::RESET);
    println!();
    println!("  {}{}{}  approaches.", ui::RED, boss_name(boss_id), ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    match run_unique_boss(boss_id, player, seed, last_roll) {
        BossOutcome::PlayerWon { xp: _, gold: _ } => {
            println!("  {}★ Boss defeated!{}", ui::YELLOW, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }
        BossOutcome::PlayerDied => RoomOutcome::PlayerDied,
        BossOutcome::Escaped => RoomOutcome::Continue,
    }
}

fn do_nemesis_encounter(
    player: &mut Character,
    nemesis: &NemesisRecord,
    seed: u64,
    last_roll: &mut Option<chaos_rpg_core::chaos_pipeline::ChaosRollResult>,
    is_cursed: bool,
    mode_str: &str,
) -> RoomOutcome {
    ui::clear_screen();
    println!("\n  {}☠  NEMESIS RETURNS  ☠{}", ui::RED, ui::RESET);
    println!();
    println!("  {} remembers you.", nemesis.enemy_name);
    println!("  Killed {} {} time(s). Floor {}.", nemesis.killed_player_class, nemesis.times_killed_player, nemesis.floor_killed_at);
    println!("  {}HP +{}%  Damage +{}%{}", ui::RED, nemesis.hp_bonus_pct, nemesis.damage_bonus_pct, ui::RESET);
    println!("  {}{}{}", ui::DIM, nemesis.resistance_label(), ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER] to face your past...{}", ui::DIM, ui::RESET));

    let base_floor = nemesis.floor_killed_at;
    let mut nemesis_enemy = generate_enemy(base_floor.max(1), seed);
    nemesis_enemy.name = format!("★ {}", nemesis.enemy_name);
    // Apply nemesis bonuses
    nemesis_enemy.hp = (nemesis_enemy.hp * (100 + nemesis.hp_bonus_pct as i64) / 100).max(1);
    nemesis_enemy.max_hp = nemesis_enemy.hp;
    nemesis_enemy.base_damage = (nemesis_enemy.base_damage * (100 + nemesis.damage_bonus_pct as i64) / 100).max(1);
    nemesis_enemy.xp_reward *= 5;
    nemesis_enemy.gold_reward *= 3;

    let result = do_combat_encounter(player, &mut nemesis_enemy, seed, last_roll, true, is_cursed, mode_str);
    if matches!(result, RoomOutcome::Continue) {
        // Nemesis killed! Clear it.
        chaos_rpg_core::nemesis::clear_nemesis();
        println!("  {}Your Nemesis is defeated. The grudge is settled.{}", ui::YELLOW, ui::RESET);
        let (_, stat_name) = player.highest_stat();
        println!("  {}Bonus loot: highest stat ({}) +50{}", ui::CYAN, stat_name, ui::RESET);
        // Reward: boost highest stat
        let (sname, _) = player.highest_stat();
        match sname {
            "Vitality"  => player.stats.vitality  += 50,
            "Force"     => player.stats.force      += 50,
            "Mana"      => player.stats.mana       += 50,
            "Cunning"   => player.stats.cunning    += 50,
            "Precision" => player.stats.precision  += 50,
            "Entropy"   => player.stats.entropy    += 50,
            _           => player.stats.luck       += 50,
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
    result
}

fn do_boss_gauntlet(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<chaos_rpg_core::chaos_pipeline::ChaosRollResult>,
    is_cursed: bool,
    final_boss_id: Option<u8>,
    mode_str: &str,
) -> RoomOutcome {
    ui::clear_screen();
    println!("\n  {}╔══════════════════════════════════╗{}", ui::RED, ui::RESET);
    println!("  {}║      FLOOR BOSS GAUNTLET         ║{}", ui::RED, ui::RESET);
    println!("  {}║  Three fights. No healing.       ║{}", ui::RED, ui::RESET);
    println!("  {}║  HP carries over between fights. ║{}", ui::RED, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::RED, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER] to enter the gauntlet...{}", ui::DIM, ui::RESET));

    // Fight 1: regular strong enemy
    let mut e1 = generate_enemy(player.floor, seed.wrapping_add(1));
    e1.hp = (e1.hp as f64 * 2.0) as i64;
    e1.max_hp = e1.hp;
    println!("  {}GAUNTLET: Fight 1/3{}", ui::YELLOW, ui::RESET);
    match do_combat_encounter(player, &mut e1, seed.wrapping_add(1), last_roll, false, is_cursed, mode_str) {
        RoomOutcome::PlayerDied => return RoomOutcome::PlayerDied,
        _ => {}
    }

    // Fight 2: stronger enemy
    let mut e2 = generate_enemy(player.floor, seed.wrapping_add(2));
    e2.hp = (e2.hp as f64 * 3.0) as i64;
    e2.max_hp = e2.hp;
    e2.base_damage = (e2.base_damage as f64 * 1.5) as i64;
    println!("  {}GAUNTLET: Fight 2/3{}", ui::YELLOW, ui::RESET);
    match do_combat_encounter(player, &mut e2, seed.wrapping_add(2), last_roll, false, is_cursed, mode_str) {
        RoomOutcome::PlayerDied => return RoomOutcome::PlayerDied,
        _ => {}
    }

    // Fight 3: boss with destiny roll
    println!("  {}GAUNTLET: Fight 3/3 — THE BOSS{}", ui::RED, ui::RESET);
    if let Some(boss_id) = final_boss_id {
        return do_unique_boss_encounter(player, boss_id, seed.wrapping_add(3), last_roll);
    }
    let mut boss = generate_enemy(player.floor, seed.wrapping_add(3));
    let destiny = chaos_rpg_core::chaos_pipeline::destiny_roll(0.5, seed.wrapping_add(31337));
    let power_mult = (destiny.final_value + 1.5).max(0.5);
    boss.hp = ((boss.hp as f64 * 4.0 * power_mult) as i64).max(1);
    boss.max_hp = boss.hp;
    boss.base_damage = ((boss.base_damage as f64 * 2.0 * power_mult) as i64).max(1);
    boss.xp_reward *= 5;
    boss.gold_reward *= 5;
    println!("  {}Destiny roll: {:.3} — power multiplier: {:.2}x{}", ui::MAGENTA, destiny.final_value, power_mult, ui::RESET);
    do_combat_encounter(player, &mut boss, seed.wrapping_add(3), last_roll, true, is_cursed, mode_str)
}

fn do_combat_encounter(
    player: &mut Character,
    enemy: &mut Enemy,
    seed: u64,
    last_roll: &mut Option<chaos_rpg_core::chaos_pipeline::ChaosRollResult>,
    is_boss: bool,
    is_cursed: bool,
    mode_str: &str,
) -> RoomOutcome {
    if is_boss {
        emit_audio(AudioEvent::BossEncounterStart { boss_tier: 1 });
        println!("  {}B O S S  E N C O U N T E R{}", ui::RED, ui::RESET);
        println!();
    }

    // NullifyAura announcement
    if enemy.floor_ability == FloorAbility::NullifyAura {
        println!("  {}⚠ NULLIFY AURA: Your first action will return 0.0 from all engines!{}", ui::RED, ui::RESET);
    }
    if enemy.floor_ability == FloorAbility::EngineTheft {
        println!("  {}⚠ ENGINE THEFT: Each hit will steal 1 engine from your roll chain!{}", ui::YELLOW, ui::RESET);
    }

    ui::println_color(
        ui::RED,
        &format!("  A {} appears! [{}]", enemy.name, enemy.tier.name()),
    );
    ui::show_enemy(enemy);
    println!();
    ui::press_enter(&format!("  {}[ENTER] to fight...{}", ui::DIM, ui::RESET));

    let mut state = CombatState::new(seed);
    state.is_cursed = is_cursed;
    let level_before = player.level;

    loop {
        // Tick status effects at the start of each round
        {
            let (tick_dmg, tick_msgs) = player.tick_status_effects();
            if tick_dmg > 0 || !tick_msgs.is_empty() {
                for msg in &tick_msgs {
                    ui::println_color(ui::MAGENTA, &format!("  {}", msg));
                }
                if tick_dmg > 0 {
                    println!("  {}Status damage: -{} HP{}", ui::RED, tick_dmg, ui::RESET);
                }
                if !player.is_alive() {
                    ui::show_game_over(player);
                    for line in player.run_summary() {
                        println!("{}", line);
                    }
                    end_game_score(player, false, mode_str);
                    return RoomOutcome::PlayerDied;
                }
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            }
        }

        ui::clear_screen();
        ui::show_combat_menu(player, enemy, state.turn + 1);

        let action = ui::read_combat_action();
        // Capture display name before action is moved into resolve_action
        let action_label = combat_action_label(&action);

        let (events, outcome) = resolve_action(player, enemy, action, &mut state);

        if let Some(ref roll) = state.last_roll {
            *last_roll = Some(roll.clone());
        }

        // ── Track cause of death (last enemy hit this round) ─────────────────
        {
            use chaos_rpg_core::combat::CombatEvent;
            let last_hit = events.iter().rev().find_map(|ev| {
                if let CombatEvent::EnemyAttack { damage, is_crit } = ev {
                    Some((*damage, *is_crit))
                } else { None }
            });
            if let Some((dmg, crit)) = last_hit {
                let crit_tag = if crit { " [CRIT]" } else { "" };
                player.run_stats.cause_of_death =
                    format!("Floor {} — {} hit for {}{}", player.floor, enemy.name, dmg, crit_tag);
                player.run_stats.final_blow_damage = dmg;
            }
        }

        // ── Misery tracking ──────────────────────────────────────────────────
        {
            use chaos_rpg_core::combat::CombatEvent;
            for event in &events {
                match event {
                    CombatEvent::EnemyAttack { damage, is_crit } => {
                        let new_ms = player.misery.add_misery(MiserySource::DamageTaken, *damage as f64);
                        if *is_crit { player.misery.add_misery(MiserySource::Headshot, 0.0); }
                        player.run_stats.record_damage_taken(*damage, *is_crit);
                        for ms in new_ms {
                            println!("  \x1b[35m[MISERY] {} — {}\x1b[0m", ms.title(), ms.flavor());
                        }
                    }
                    CombatEvent::PlayerFleeFailed => {
                        player.misery.add_misery(MiserySource::FleeFailed, 0.0);
                    }
                    CombatEvent::PlayerAttack { damage, is_crit } => {
                        player.run_stats.record_damage_dealt(*damage, None, *is_crit);
                        // Defiance roll tracking
                        let new_passives = player.misery.increment_defiance_roll();
                        for p in new_passives {
                            println!("  \x1b[96m[DEFIANCE] {} UNLOCKED!\x1b[0m", p.name());
                        }
                    }
                    _ => {}
                }
            }
            // Cosmic Joke flavor (15% chance per round for negative chars)
            if player.misery.cosmic_joke {
                if let Some(line) = chaos_rpg_core::misery_system::MiseryState::cosmic_joke_combat_line(
                    player.seed, player.rooms_cleared as u64) {
                    println!("  \x1b[2m{}\x1b[0m", line);
                }
            }
            // Underdog mercy — pity skip
            let pity_chance = chaos_rpg_core::misery_system::MiseryState::enemy_pity_chance(player.stats.total());
            if pity_chance > 0.0 {
                let roll = (player.seed.wrapping_mul(player.rooms_cleared as u64 + 1)) % 100;
                if roll < (pity_chance * 100.0) as u64 {
                    player.misery.add_misery(MiserySource::EnemyPitiedYou, 0.0);
                    player.run_stats.enemies_pitied_you += 1;
                    println!("  \x1b[2m{} looks at you with pity. It cannot bring itself to attack.\x1b[0m", enemy.name);
                }
            }
        }

        ui::display_combat_events(&events);
        println!();

        // ── CHAOS ENGINE TRACE ──────────────────────────────────────────────
        // Always show the chain that produced this result. This is the entire
        // personality of the game — "the Lorenz Attractor conspired with
        // Euler's Totient to produce this outcome."
        let player_outcome = events_to_outcome_str(&events);
        if let Some(ref roll) = state.last_roll {
            for line in roll.combat_trace_lines(&action_label, &player_outcome) {
                println!("{}", line);
            }
        }

        // Show enemy's chain as a compact line (their counterattack roll)
        if let Some(ref roll) = state.enemy_last_roll {
            let enemy_outcome = events_to_enemy_outcome_str(&events, &enemy.name);
            println!("{}", roll.enemy_trace_line(&enemy.name, &enemy_outcome));
        }
        println!();

        match outcome {
            CombatOutcome::PlayerWon { xp, gold } => {
                emit_audio(AudioEvent::EntityDied { is_player: false });
                println!(
                    "  {}Victory! +{} XP, +{} gold.{}",
                    ui::YELLOW,
                    xp,
                    gold,
                    ui::RESET
                );
                if player.level > level_before {
                    emit_audio(AudioEvent::LevelUp);
                    ui::show_level_up(player.level, "Chaos has amplified your stats!");
                    ui::show_character_sheet(player);
                    if player.skill_points > 0 {
                        if ui::is_auto_mode() {
                            let msgs = player.auto_allocate_passives(
                                seed.wrapping_add(player.level as u64 * 777));
                            for m in &msgs {
                                println!("  {}[AUTO] {}{}", ui::GREEN, m, ui::RESET);
                            }
                        } else {
                            println!(
                                "  {}You have {} skill point(s) to spend!{}",
                                ui::CYAN,
                                player.skill_points,
                                ui::RESET
                            );
                            let inp = ui::prompt("  [P] Open passive tree  [any] Skip > ");
                            if inp.trim().eq_ignore_ascii_case("p") {
                                ui::show_passive_tree_ui(player, seed);
                            }
                        }
                    }
                }

                // Loot drop — 40% chance, guaranteed from bosses
                let loot_seed = seed
                    .wrapping_add(enemy.seed)
                    .wrapping_add(state.turn as u64 * 9973);
                let drop_chance = if is_boss { 100 } else { 40 };
                if loot_seed % 100 < drop_chance {
                    let loot = Item::generate(loot_seed);
                    println!();
                    println!("  {}★ Item dropped!{}", ui::YELLOW, ui::RESET);
                    for line in loot.display_box() {
                        println!("  {}", line);
                    }
                    println!();
                    let pick = ui::prompt("  [P] Pick up  [any] Leave >");
                    if pick.trim().eq_ignore_ascii_case("p") {
                        player.add_item(loot);
                        println!(
                            "  {}Added to inventory. ({} items){}",
                            ui::GREEN,
                            player.inventory.len(),
                            ui::RESET
                        );
                    }
                }

                player.rooms_cleared += 1;
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return RoomOutcome::Continue;
            }
            CombatOutcome::PlayerDied => {
                emit_audio(AudioEvent::EntityDied { is_player: true });
                emit_audio(AudioEvent::GameOver);
                player.misery.add_misery(MiserySource::DeathRemainingEnemyHp, enemy.hp as f64);
                // Save nemesis: the enemy that just killed the player
                let kill_method = if player.spells_cast > player.kills * 2 { "spell" } else { "physical" };
                let nemesis = NemesisRecord::new(
                    enemy.name.clone(),
                    player.floor,
                    enemy.base_damage,
                    player.class.name().to_string(),
                    kill_method,
                );
                save_nemesis(&nemesis);
                ui::show_game_over(player);
                println!();
                println!("  {}☠ {} is now your Nemesis.{}", ui::RED, enemy.name, ui::RESET);
                println!("  {}It will appear in your next run — stronger and ready.{}", ui::DIM, ui::RESET);
                println!();
                for line in player.run_summary() {
                    println!("{}", line);
                }
                println!();
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return RoomOutcome::PlayerDied;
            }
            CombatOutcome::PlayerFled => {
                println!("  {}You escaped!{}", ui::GREEN, ui::RESET);
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return RoomOutcome::Continue;
            }
            CombatOutcome::Ongoing => {
                println!(
                    "  Your HP: {}  |  {} HP: {}",
                    player.current_hp, enemy.name, enemy.hp
                );
                ui::press_enter(&format!("  {}[ENTER] next round...{}", ui::DIM, ui::RESET));
            }
        }
    }
}

// ─── HELPERS ─────────────────────────────────────────────────────────────────

fn apply_stat_modifier(player: &mut Character, stat: &str, value: i64) {
    match stat.to_lowercase().as_str() {
        "vitality" => {
            player.stats.vitality += value;
            player.max_hp = 50 + player.stats.vitality * 3 + player.stats.force;
        }
        "force" => {
            player.stats.force += value;
            player.max_hp = 50 + player.stats.vitality * 3 + player.stats.force;
        }
        "mana" => player.stats.mana += value,
        "cunning" => player.stats.cunning += value,
        "precision" => player.stats.precision += value,
        "entropy" => player.stats.entropy += value,
        "luck" => player.stats.luck += value,
        _ => {}
    }
}

/// Returns a display label for the action — used as the trace header.
fn combat_action_label(action: &CombatAction) -> String {
    match action {
        CombatAction::Attack => "Attack Roll".to_string(),
        CombatAction::HeavyAttack => "Heavy Attack Roll".to_string(),
        CombatAction::Defend => "Defend Roll".to_string(),
        CombatAction::Flee => "Flee Roll".to_string(),
        CombatAction::Taunt => "Taunt Roll".to_string(),
        CombatAction::UseSpell(i) => format!("Spell Cast #{}", i + 1),
        CombatAction::UseItem(i) => format!("Item Use #{}", i + 1),
    }
}

/// Derive a short outcome string from combat events for the player trace footer.
fn events_to_outcome_str(events: &[chaos_rpg_core::combat::CombatEvent]) -> String {
    use chaos_rpg_core::combat::CombatEvent;
    for event in events {
        match event {
            CombatEvent::PlayerAttack { damage, is_crit } => {
                return if *is_crit {
                    format!("dealt {} damage (CRITICAL)", damage)
                } else {
                    format!("dealt {} damage", damage)
                };
            }
            CombatEvent::SpellCast {
                name,
                damage,
                backfired,
            } => {
                return if *backfired {
                    format!("{} BACKFIRED — took {} self-damage", name, damage)
                } else {
                    format!("{} — dealt {} damage", name, damage)
                };
            }
            CombatEvent::PlayerFled => return "escaped into the chaos".to_string(),
            CombatEvent::PlayerFleeFailed => {
                return "flee failed — math won't allow it".to_string()
            }
            CombatEvent::PlayerDefend { damage_reduced } => {
                return format!("defending — {} damage absorbed", damage_reduced);
            }
            CombatEvent::StatusApplied { name } => {
                return format!("applied {}", name);
            }
            _ => {}
        }
    }
    "roll complete".to_string()
}

/// Derive a short outcome string from combat events for the enemy trace line.
fn events_to_enemy_outcome_str(
    events: &[chaos_rpg_core::combat::CombatEvent],
    enemy_name: &str,
) -> String {
    use chaos_rpg_core::combat::CombatEvent;
    for event in events {
        if let CombatEvent::EnemyAttack { damage, is_crit } = event {
            return if *is_crit {
                format!("CRIT — {} damage to you", damage)
            } else {
                format!("{} damage to you", damage)
            };
        }
    }
    // Stunned or other case
    for event in events {
        if let CombatEvent::ChaosEvent { description } = event {
            if description.contains("stunned") || description.contains("Stunned") {
                return format!("{} is stunned — skips turn", enemy_name);
            }
        }
    }
    "acted".to_string()
}

fn end_game_score(player: &Character, won: bool, mode_str: &str) {
    use chaos_rpg_core::lore::narrative::RunNarrative;
    use chaos_rpg_core::run_history::{RunHistory, RunRecord};

    let tier = player.power_tier();
    let underdog = player.underdog_multiplier();
    let misery = player.misery.misery_index;

    // Regular scoreboard
    let entry = ScoreEntry::new(
        player.name.clone(),
        player.class.to_string(),
        player.score(),
        player.floor,
        player.kills,
        0,
    )
    .with_tier(tier.name())
    .with_misery(misery, underdog);
    let scores = save_score(entry);

    // Hall of Misery (only if they have misery to speak of)
    if misery >= 100.0 {
        let misery_entry = MiseryEntry::new(
            &player.name,
            player.class.to_string(),
            misery,
            player.floor,
            tier.name(),
            player.misery.spite_total_spent,
            player.misery.defiance_rolls,
            &player.run_stats.cause_of_death,
            underdog,
        );
        save_misery_score(misery_entry);
    }

    // Legacy / graveyard
    let epitaph = GraveyardEntry::generate_epitaph(
        player.class.to_string().as_str(),
        player.floor,
        player.kills,
        player.total_damage_dealt,
        misery,
        player.spells_cast,
        player.stats.vitality < 0 && player.stats.force < 0 && player.stats.mana < 0,
        player.run_stats.deaths_to_backfire > 0,
        tier.name(),
    );
    let graveyard_entry = GraveyardEntry {
        name: player.name.clone(),
        class: player.class.to_string(),
        level: player.level,
        floor: player.floor,
        power_tier: tier.name().to_string(),
        misery_index: misery,
        cause_of_death: player.run_stats.cause_of_death.clone(),
        kills: player.kills,
        score: player.score(),
        date: String::new(),
        epitaph: epitaph.clone(),
    };
    let mut legacy = LegacyData::load();
    legacy.record_run(
        graveyard_entry,
        player.total_damage_dealt,
        player.total_damage_taken,
        player.gold,
        misery,
        player.misery.spite_total_spent,
        player.run_stats.total_rolls,
        player.run_stats.deaths_to_backfire > 0,
        false,
        player.seed,
        tier.name(),
    );
    legacy.save();

    // ── Build and display run narrative ───────────────────────────────────────
    let pos_stats: Vec<(String, i64)> = [
        ("Vitality", player.stats.vitality),
        ("Force", player.stats.force),
        ("Mana", player.stats.mana),
        ("Cunning", player.stats.cunning),
        ("Precision", player.stats.precision),
        ("Entropy", player.stats.entropy),
        ("Luck", player.stats.luck),
    ]
    .iter()
    .filter(|(_, v)| *v > 0)
    .map(|(n, v)| (n.to_string(), *v))
    .collect();

    let neg_stats: Vec<(String, i64)> = [
        ("Vitality", player.stats.vitality),
        ("Force", player.stats.force),
        ("Mana", player.stats.mana),
        ("Cunning", player.stats.cunning),
        ("Precision", player.stats.precision),
        ("Entropy", player.stats.entropy),
        ("Luck", player.stats.luck),
    ]
    .iter()
    .filter(|(_, v)| *v < 0)
    .map(|(n, v)| (n.to_string(), *v))
    .collect();

    let narrative = RunNarrative {
        character_name: player.name.clone(),
        character_class: player.class.to_string(),
        character_background: player.background.name().to_string(),
        difficulty: player.difficulty.name().to_string(),
        game_mode: mode_str.to_string(),
        destiny_roll_value: 0.0, // not persisted; narrative still reads from events
        positive_stats: pos_stats,
        negative_stats: neg_stats,
        boon_name: player.boon.map(|b| b.name().to_string()),
        final_floor: player.floor,
        final_tier: tier.name().to_string(),
        total_kills: player.kills as u64,
        total_damage: player.total_damage_dealt,
        events: player.narrative_events.clone(),
        custom_origin: if player.character_lore.origin.is_empty() {
            None
        } else {
            Some(player.character_lore.origin.clone())
        },
        epitaph: epitaph.clone(),
        won,
    };
    let auto_narrative = narrative.generate();

    // Save to run history
    let record = RunRecord {
        date: {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Simple date string: seconds since epoch
            secs.to_string()
        },
        name: player.name.clone(),
        class: player.class.to_string(),
        difficulty: player.difficulty.name().to_string(),
        game_mode: mode_str.to_string(),
        floor: player.floor,
        level: player.level,
        kills: player.kills as u64,
        score: player.score(),
        damage_dealt: player.total_damage_dealt,
        damage_taken: player.total_damage_taken,
        highest_hit: player.run_stats.highest_single_hit,
        spells_cast: player.spells_cast,
        items_used: player.items_used,
        gold: player.gold,
        misery_index: misery,
        corruption: player.corruption,
        power_tier: tier.name().to_string(),
        cause_of_death: player.run_stats.cause_of_death.clone(),
        seed: player.seed,
        won,
        epitaph,
        auto_narrative: auto_narrative.clone(),
        character_lore: if player.character_lore.origin.is_empty()
            && player.character_lore.motivation.is_empty()
        {
            None
        } else {
            Some(player.character_lore.clone())
        },
    };
    let mut history = RunHistory::load();
    history.push(record);

    ui::show_scoreboard(&scores);
    ui::show_run_narrative(&auto_narrative);
}
