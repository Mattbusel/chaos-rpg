use chaos_rpg::{
    character::{Character, CharacterClass},
    combat::{CombatAction, CombatOutcome, CombatState, resolve_action},
    enemy::generate_enemy,
    scoreboard::{save_score, load_scores, ScoreEntry},
    ui::{self, GameMode, FloorChoice},
    chaos_pipeline::chaos_roll_verbose,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_seed() -> u64 {
    // Allow deterministic runs via CHAOS_SEED env var
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

fn main() {
    loop {
        ui::show_title();
        let mode = ui::select_mode();

        match mode {
            GameMode::Quit => {
                println!("\n  {}The chaos subsides. For now.{}", ui::DIM, ui::RESET);
                return;
            }
            GameMode::Scoreboard => {
                let scores = load_scores();
                ui::show_scoreboard(&scores);
            }
            GameMode::Story | GameMode::Infinite => {
                run_game(mode);
            }
        }
    }
}

fn run_game(mode: GameMode) {
    // Optional tutorial
    let help = ui::prompt("  Show tutorial? [y/N] >");
    if help.eq_ignore_ascii_case("y") {
        ui::show_help();
    }

    // Character creation
    let (name, class, background) = ui::create_character_ui();
    let seed = current_seed();
    let mut player = Character::roll_new(name, class, background, seed);

    ui::clear_screen();
    println!("\n  {}Destiny roll complete.{}", ui::YELLOW, ui::RESET);
    println!("  The 10 sacred algorithms have determined your fate.");
    println!();
    ui::show_character_sheet(&player);
    println!();

    let intro = match player.class {
        CharacterClass::Mage =>
            "You emerged from the fractal abyss, equations swirling in your wake.",
        CharacterClass::Berserker =>
            "The Lorenz attractor knows your rage. It feeds on it.",
        CharacterClass::Ranger =>
            "You read the prime spirals in every shadow. Nothing escapes you.",
        CharacterClass::Thief =>
            "Logistic map r=3.9: you are the period-doubling cascade no one sees coming.",
    };
    println!("  {}{}{}", ui::MAGENTA, intro, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}Begin your descent [ENTER]...{}", ui::DIM, ui::RESET));

    let max_floor = if mode == GameMode::Story { 10u32 } else { u32::MAX };
    let mut last_roll = None;
    let mut encounters_on_floor = 0u32;
    let mut floor_seed = seed;

    loop {
        ui::clear_screen();
        ui::show_floor_header(player.floor, &mode);

        if mode == GameMode::Story {
            floor_seed = floor_seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(player.floor as u64);
            if let Some(event) = ui::story_event(player.floor, floor_seed) {
                println!("{}", event);
                println!();
            }
        }

        println!(
            "  HP: {}  Gold: {}  Floor: {}  Kills: {}",
            player.hp_bar(16), player.gold, player.floor, player.kills
        );
        println!();

        let choice = ui::floor_choices();

        match choice {
            FloorChoice::ViewSheet => {
                ui::show_character_sheet(&player);
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            }

            FloorChoice::ViewTrace => {
                if let Some(ref roll) = last_roll {
                    for line in roll.display_lines() {
                        println!("{}", line);
                    }
                } else {
                    println!("  {}No chaos roll yet.{}", ui::DIM, ui::RESET);
                }
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            }

            FloorChoice::Rest => {
                if player.gold >= 10 {
                    player.gold -= 10;
                    player.heal(20);
                    println!("  {}You rest. HP +20. Gold -10.{}", ui::GREEN, ui::RESET);
                } else {
                    println!("  {}Not enough gold. Need 10.{}", ui::RED, ui::RESET);
                }
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            }

            FloorChoice::Descend => {
                player.floor += 1;
                encounters_on_floor = 0;
                floor_seed = floor_seed.wrapping_add(player.floor as u64 * 31337);

                if player.floor > max_floor {
                    // Victory
                    ui::show_victory(&player);
                    let entry = ScoreEntry::new(
                        player.name.clone(),
                        player.class.to_string(),
                        player.score(),
                        player.floor,
                        player.kills,
                        0,
                    );
                    let scores = save_score(entry);
                    ui::show_scoreboard(&scores);
                    return;
                }
            }

            FloorChoice::Explore => {
                encounters_on_floor += 1;
                let enemy_seed = floor_seed
                    .wrapping_add(encounters_on_floor as u64 * 9999991)
                    .wrapping_mul(2654435761);
                let mut enemy = generate_enemy(player.floor, enemy_seed);

                println!();
                ui::println_color(
                    ui::RED,
                    &format!("  A {} appears! [{}]", enemy.name, enemy.tier.name()),
                );
                ui::show_enemy(&enemy);
                println!();
                ui::press_enter(&format!("  {}[ENTER] to fight...{}", ui::DIM, ui::RESET));

                let mut state = CombatState::new(enemy_seed);
                let level_before = player.level;

                'combat: loop {
                    ui::clear_screen();
                    ui::show_combat_menu(&player, &enemy, state.turn + 1);

                    let action = ui::read_combat_action();

                    // Show chain trace on '?' input — handled in read_combat_action
                    // For UseSpell, do a mage-specific chaos display
                    if matches!(action, CombatAction::UseSpell(_)) {
                        let spell_seed = state.seed.wrapping_add(77777);
                        let roll = chaos_roll_verbose(
                            player.stats.mana as f64 * 0.01,
                            spell_seed,
                        );
                        for line in roll.display_lines() {
                            println!("{}", line);
                        }
                        last_roll = Some(roll);
                        ui::press_enter(&format!("  {}[ENTER] cast...{}", ui::DIM, ui::RESET));
                    }

                    let (events, outcome) =
                        resolve_action(&mut player, &mut enemy, action, &mut state);

                    if let Some(ref roll) = state.last_roll {
                        last_roll = Some(roll.clone());
                    }

                    ui::display_combat_events(&events);
                    println!();

                    match outcome {
                        CombatOutcome::PlayerWon { xp, gold } => {
                            println!(
                                "  {}Victory! +{} XP, +{} gold.{}",
                                ui::YELLOW, xp, gold, ui::RESET
                            );
                            if player.level > level_before {
                                ui::show_level_up(
                                    player.level,
                                    "Chaos has amplified your stats!",
                                );
                            }
                            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                            break 'combat;
                        }
                        CombatOutcome::PlayerDied => {
                            ui::show_game_over(&player);
                            let entry = ScoreEntry::new(
                                player.name.clone(),
                                player.class.to_string(),
                                player.score(),
                                player.floor,
                                player.kills,
                                0,
                            );
                            let scores = save_score(entry);
                            ui::show_scoreboard(&scores);
                            return;
                        }
                        CombatOutcome::PlayerFled => {
                            println!("  {}You escaped!{}", ui::GREEN, ui::RESET);
                            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                            break 'combat;
                        }
                        CombatOutcome::Ongoing => {
                            println!(
                                "  Your HP: {}  |  {} HP: {}",
                                player.current_hp, enemy.name, enemy.hp
                            );
                            ui::press_enter(&format!(
                                "  {}[ENTER] next round...{}",
                                ui::DIM, ui::RESET
                            ));
                        }
                    }
                }
            }
        }
    }
}
