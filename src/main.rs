use chaos_rpg::{
    chaos_pipeline::chaos_roll_verbose,
    character::{Character, CharacterClass},
    combat::{resolve_action, CombatAction, CombatOutcome, CombatState},
    enemy::Enemy,
    items::Item,
    npcs::shop_npc,
    scoreboard::{save_score, ScoreEntry},
    skill_checks::{perform_skill_check, Difficulty, SkillType},
    ui::{self, GameMode},
    world::{generate_floor, room_enemy, Room, RoomType},
};
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
                let scores = chaos_rpg::scoreboard::load_scores();
                ui::show_scoreboard(&scores);
            }
            GameMode::Story | GameMode::Infinite => {
                run_game(mode);
            }
        }
    }
}

fn run_game(mode: GameMode) {
    let help = ui::prompt("  Show tutorial? [y/N] >");
    if help.eq_ignore_ascii_case("y") {
        ui::show_help();
    }

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
    let mut last_roll: Option<chaos_rpg::chaos_pipeline::ChaosRollResult> = None;
    let mut floor_seed = seed;

    'game: loop {
        floor_seed = floor_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(player.floor as u64 * 31337);

        let mut floor = generate_floor(player.floor, floor_seed);

        ui::clear_screen();
        ui::show_floor_header(player.floor, &mode);

        if mode == GameMode::Story {
            if let Some(event) = ui::story_event(player.floor, floor_seed) {
                println!("{}", event);
                println!();
            }
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
            println!("  [E] Enter room   [C] Character sheet   [T] Last chaos trace");
            if floor.rooms_remaining() == 0 {
                println!(
                    "  {}[D] Descend to floor {}{}",
                    ui::CYAN,
                    player.floor + 1,
                    ui::RESET
                );
            }
            println!();

            let input = ui::prompt("  > ").to_lowercase();

            match input.trim() {
                "c" => {
                    ui::show_character_sheet(&player);
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
                "d" if floor.rooms_remaining() == 0 => {
                    player.floor += 1;
                    if player.floor > max_floor {
                        ui::show_victory(&player);
                        println!();
                        for line in player.run_summary() {
                            println!("{}", line);
                        }
                        println!();
                        end_game_score(&player);
                        return;
                    }
                    break 'rooms;
                }
                _ => {
                    let outcome = handle_room(
                        &room,
                        &mut player,
                        floor_seed.wrapping_add(floor.current_room as u64 * 9973),
                        &mut last_roll,
                    );

                    match outcome {
                        RoomOutcome::PlayerDied => {
                            ui::show_game_over(&player);
                            println!();
                            for line in player.run_summary() {
                                println!("{}", line);
                            }
                            println!();
                            end_game_score(&player);
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
                                end_game_score(&player);
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

fn handle_room(
    room: &Room,
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<chaos_rpg::chaos_pipeline::ChaosRollResult>,
) -> RoomOutcome {
    match room.room_type {
        RoomType::Combat => {
            let mut enemy = room_enemy(room);
            do_combat_encounter(player, &mut enemy, seed, last_roll, false)
        }

        RoomType::Boss => {
            let mut enemy = room_enemy(room);
            enemy.hp = (enemy.hp as f64 * 2.5) as i64;
            enemy.max_hp = enemy.hp;
            enemy.base_damage = (enemy.base_damage as f64 * 1.8) as i64;
            enemy.xp_reward *= 3;
            enemy.gold_reward *= 3;
            do_combat_encounter(player, &mut enemy, seed, last_roll, true)
        }

        RoomType::Treasure => {
            let item = Item::generate(seed);
            let gold_bonus = (seed % 30 + 10) as i64 * player.floor as i64;

            println!("  {}* TREASURE ROOM *{}", ui::YELLOW, ui::RESET);
            println!();
            for line in item.display_box() {
                println!("  {}", line);
            }
            println!();
            println!("  {}You find {} gold!{}", ui::YELLOW, gold_bonus, ui::RESET);
            player.gold += gold_bonus;

            for modifier in &item.stat_modifiers {
                apply_stat_modifier(player, &modifier.stat, modifier.value);
            }
            player.add_item(item);
            println!("  {}Item added to inventory! (Use [I#] in combat){}", ui::GREEN, ui::RESET);

            // 25% chance to also find a spell scroll
            if seed % 4 == 0 {
                let spell = chaos_rpg::spells::Spell::generate(seed.wrapping_add(54321));
                println!();
                println!("  {}+ SPELL SCROLL FOUND +{}", ui::CYAN, ui::RESET);
                for line in spell.display_box() {
                    println!("  {}", line);
                }
                player.add_spell(spell);
                println!("  {}Spell learned! Use [S#] in combat.{}", ui::CYAN, ui::RESET);
            }

            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            RoomOutcome::Continue
        }

        RoomType::Shop => {
            let npc = shop_npc(player.floor, seed);
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
                        player.heal(40);
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
                        let item = &npc.inventory[idx - 1];
                        let price = npc.sale_price(item.value, player.stats.cunning);
                        if player.gold >= price {
                            player.gold -= price;
                            let mods = item.stat_modifiers.clone();
                            for modifier in &mods {
                                apply_stat_modifier(player, &modifier.stat, modifier.value);
                            }
                            println!("  {}Purchased! Stats updated.{}", ui::GREEN, ui::RESET);
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
            println!("  Step through to the next floor? [Y/N]");
            println!();

            let input = ui::prompt("  > ");
            if input.trim().eq_ignore_ascii_case("y") {
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
                    player.heal(heal);
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
    }
}

// ─── COMBAT ──────────────────────────────────────────────────────────────────

fn do_combat_encounter(
    player: &mut Character,
    enemy: &mut Enemy,
    seed: u64,
    last_roll: &mut Option<chaos_rpg::chaos_pipeline::ChaosRollResult>,
    is_boss: bool,
) -> RoomOutcome {
    if is_boss {
        println!("  {}B O S S  E N C O U N T E R{}", ui::RED, ui::RESET);
        println!();
    }

    ui::println_color(
        ui::RED,
        &format!("  A {} appears! [{}]", enemy.name, enemy.tier.name()),
    );
    ui::show_enemy(enemy);
    println!();
    ui::press_enter(&format!("  {}[ENTER] to fight...{}", ui::DIM, ui::RESET));

    let mut state = CombatState::new(seed);
    let level_before = player.level;

    loop {
        ui::clear_screen();
        ui::show_combat_menu(player, enemy, state.turn + 1);

        let action = ui::read_combat_action();

        if matches!(action, CombatAction::UseSpell(_)) {
            let spell_seed = state.seed.wrapping_add(77777);
            let roll = chaos_roll_verbose(player.stats.mana as f64 * 0.01, spell_seed);
            for line in roll.display_lines() {
                println!("{}", line);
            }
            *last_roll = Some(roll);
            ui::press_enter(&format!("  {}[ENTER] cast...{}", ui::DIM, ui::RESET));
        }

        let (events, outcome) = resolve_action(player, enemy, action, &mut state);

        if let Some(ref roll) = state.last_roll {
            *last_roll = Some(roll.clone());
        }

        ui::display_combat_events(&events);
        println!();

        match outcome {
            CombatOutcome::PlayerWon { xp, gold } => {
                println!(
                    "  {}Victory! +{} XP, +{} gold.{}",
                    ui::YELLOW,
                    xp,
                    gold,
                    ui::RESET
                );
                if player.level > level_before {
                    ui::show_level_up(player.level, "Chaos has amplified your stats!");
                }
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return RoomOutcome::Continue;
            }
            CombatOutcome::PlayerDied => {
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

fn end_game_score(player: &Character) {
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
}
