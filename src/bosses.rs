//! Unique boss encounters with hand-crafted mechanics.
//!
//! Each boss targets a different build archetype:
//! The Mirror counters high-stats, The Accountant counters glass cannon,
//! The Taxman counters gold hoarders, The Null counters chaos-scalers,
//! The Paradox counters stat-dumpers, The Algorithm Reborn counters grinders.

use crate::character::Character;
use crate::chaos_pipeline::{biased_chaos_roll, chaos_roll_verbose, roll_damage, ChaosRollResult};
use crate::ui;

// ─── OUTCOME ─────────────────────────────────────────────────────────────────

pub enum BossOutcome {
    PlayerWon { xp: u64, gold: i64 },
    PlayerDied,
    Escaped,
}

// ─── SPRITES ─────────────────────────────────────────────────────────────────

const SPRITE_MIRROR: &str = "  ╔═══════╗\n  ║  YOU  ║\n  ║ (x_x) ║\n  ╚═══════╝";
const SPRITE_ACCOUNTANT: &str = "  ┌─────────┐\n  │ LEDGER  │\n  │ $ $ $ $ │\n  └─────────┘";
const SPRITE_HYDRA: &str = "  {o} {o}\n  /\\/\\/\\\n  \\  /\n   \\/";
const SPRITE_EIGENSTATE: &str = "  ?╔══╗?\n  ║A?B║\n  ?╚══╝?";
const SPRITE_TAXMAN: &str = "  [TAX]\n  (>.<)\n  /|$|\\\n  d   b";
const SPRITE_NULL: &str = "  [ NULL ]\n  |      |\n  | 0.00 |\n  [______]";
const SPRITE_OUROBOROS: &str = "  ~~~>--\n  |   (O)\n  <~~~ /";
const SPRITE_COLLATZ: &str = "  ╔══════╗\n  ║3n+1  ║\n  ║  /2  ║\n  ╚══════╝";
const SPRITE_COMMITTEE: &str = "  [A][B][C]\n  [D] [E]\n  COMMITTEE";
const SPRITE_RECURSION: &str = "  ↻↻↻↻↻↻\n  ↻ DMG ↻\n  ↻↻↻↻↻↻";
const SPRITE_PARADOX: &str = "  ∞ ≠ ∞\n  ??? \n  ∅";
const SPRITE_ALGORITHM: &str = "  ▓▓▓▓▓▓▓▓\n  ▓THE ALG▓\n  ▓REBORN ▓\n  ▓▓▓▓▓▓▓▓";

// ─── POOL SELECTION ──────────────────────────────────────────────────────────

/// Boss IDs unlocked by floor.
pub fn boss_pool_for_floor(floor: u32) -> Vec<u8> {
    let mut pool = Vec::new();
    if floor >= 5   { pool.push(1); }  // The Mirror
    if floor >= 10  { pool.push(2); }  // The Accountant
    if floor >= 15  { pool.push(3); }  // Fibonacci Hydra
    if floor >= 15  { pool.push(4); }  // The Eigenstate
    if floor >= 20  { pool.push(5); }  // The Taxman
    if floor >= 25  { pool.push(6); }  // The Null
    if floor >= 30  { pool.push(7); }  // The Ouroboros
    if floor >= 35  { pool.push(8); }  // The Collatz Titan
    if floor >= 40  { pool.push(9); }  // The Committee
    if floor >= 50  { pool.push(10); } // The Recursion
    if floor >= 75  { pool.push(11); } // The Paradox
    if floor >= 100 { pool.push(12); } // The Algorithm Reborn
    pool
}

pub fn random_unique_boss(floor: u32, seed: u64) -> Option<u8> {
    let pool = boss_pool_for_floor(floor);
    if pool.is_empty() { return None; }
    let idx = (seed.wrapping_mul(0x9e3779b9) % pool.len() as u64) as usize;
    Some(pool[idx])
}

pub fn boss_name(id: u8) -> &'static str {
    match id {
        1  => "THE MIRROR",
        2  => "THE ACCOUNTANT",
        3  => "THE FIBONACCI HYDRA",
        4  => "THE EIGENSTATE",
        5  => "THE TAXMAN",
        6  => "THE NULL",
        7  => "THE OUROBOROS",
        8  => "THE COLLATZ TITAN",
        9  => "THE COMMITTEE",
        10 => "THE RECURSION",
        11 => "THE PARADOX",
        12 => "THE ALGORITHM REBORN",
        _  => "UNKNOWN HORROR",
    }
}

/// Dispatch to the correct boss fight.
pub fn run_unique_boss(
    id: u8,
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    match id {
        1  => fight_the_mirror(player, seed, last_roll),
        2  => fight_the_accountant(player, seed, last_roll),
        3  => fight_fibonacci_hydra(player, seed, last_roll),
        4  => fight_the_eigenstate(player, seed, last_roll),
        5  => fight_the_taxman(player, seed, last_roll),
        6  => fight_the_null(player, seed, last_roll),
        7  => fight_the_ouroboros(player, seed, last_roll),
        8  => fight_the_collatz_titan(player, seed, last_roll),
        9  => fight_the_committee(player, seed, last_roll),
        10 => fight_the_recursion(player, seed, last_roll),
        11 => fight_the_paradox(player, seed, last_roll),
        12 => fight_the_algorithm_reborn(player, seed, last_roll),
        _  => BossOutcome::Escaped,
    }
}

// ─── HELPERS ─────────────────────────────────────────────────────────────────

fn boss_reward(floor: u32, base_xp: u64, base_gold: i64) -> (u64, i64) {
    let xp = base_xp + floor as u64 * 150;
    let gold = base_gold + floor as i64 * 30;
    (xp, gold)
}

fn grant_victory(player: &mut Character, xp: u64, gold: i64) {
    player.kills += 1;
    player.gold += gold;
    player.gain_xp(xp);
}

fn advance_seed(s: u64) -> u64 {
    s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

// ─── 1. THE MIRROR ───────────────────────────────────────────────────────────

fn fight_the_mirror(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    let mirror_max_hp = player.max_hp;
    let mirror_force = player.stats.force;
    let mirror_mana = player.stats.mana;
    let mirror_base_dmg = 5 + mirror_force / 5 + player.stats.precision / 10;
    let mirror_spell_count = player.known_spells.len();

    let mut mirror_hp = mirror_max_hp;
    let mut sc = seed;
    let mut turn = 0u32;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::MAGENTA, ui::RESET);
    println!("  {}║          THE MIRROR              ║{}", ui::MAGENTA, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::MAGENTA, ui::RESET);
    println!();
    for line in SPRITE_MIRROR.lines() { println!("  {}", line); }
    println!();
    println!("  Your exact reflection. Same HP. Same stats. Same spells.");
    println!("  But NOT your passive. And its chaos seeds diverge from yours.");
    println!("  {}Find the asymmetry. Create the gap. Win through chaos.{}", ui::DIM, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER] to fight...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        ui::clear_screen();
        println!("\n  {}THE MIRROR — Turn {}{}", ui::MAGENTA, turn, ui::RESET);
        println!("  Your HP:    {}{}{} / {}", ui::GREEN, player.current_hp, ui::RESET, player.max_hp);
        println!("  Mirror HP:  {}{}{} / {}", ui::RED, mirror_hp, ui::RESET, mirror_max_hp);
        println!();
        println!("  [A] Attack   [D] Defend   [F] Flee");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());
        let is_defend = input.trim() == "d";

        match input.trim() {
            "f" => {
                let flee_roll = chaos_roll_verbose(player.stats.luck as f64 * 0.01, sc.wrapping_add(9999));
                if flee_roll.is_success() {
                    println!("  {}You broke the reflection and fled.{}", ui::GREEN, ui::RESET);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return BossOutcome::Escaped;
                }
                println!("  {}The Mirror holds. You cannot look away.{}", ui::RED, ui::RESET);
            }
            "d" => {
                println!("  {}You raise your guard.{}", ui::CYAN, ui::RESET);
            }
            _ => {
                // Attack
                let base = 5 + player.stats.force / 5 + player.stats.precision / 10;
                let mut dmg = roll_damage(base, player.stats.force, sc);
                if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
                if roll.is_catastrophe() { dmg = 0; }
                mirror_hp = (mirror_hp - dmg).max(0);
                for line in roll.combat_trace_lines("Attack vs Mirror", &format!("dealt {dmg} damage")) {
                    println!("{}", line);
                }
                if mirror_hp <= 0 {
                    let (xp, gold) = boss_reward(player.floor, 800, 200);
                    grant_victory(player, xp, gold);
                    println!("\n  {}THE MIRROR SHATTERS.{}", ui::YELLOW, ui::RESET);
                    println!("  Glass rains. Your reflection is gone.");
                    println!("  +{xp} XP, +{gold} gold");
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return BossOutcome::PlayerWon { xp, gold };
                }
            }
        }

        // Mirror's turn — different seed so rolls diverge
        let mseed = sc.wrapping_add(0xfeedface).wrapping_mul(turn as u64 + 7);
        let mroll = chaos_roll_verbose(mirror_mana as f64 * 0.01, mseed);

        let raw_dmg = if mirror_spell_count > 0 && mroll.is_success() {
            let sidx = (mseed % mirror_spell_count as u64) as usize;
            let spell = &player.known_spells[sidx];
            let d = spell.calc_damage(mirror_mana);
            println!("  {}Mirror casts {} for {}!{}", ui::RED, spell.name, d, ui::RESET);
            d
        } else {
            let d = roll_damage(mirror_base_dmg, mirror_force, mseed);
            println!("  {}Mirror attacks for {}!{}", ui::RED, d, ui::RESET);
            d
        };

        let incoming = if is_defend {
            let reduction = player.stats.vitality / 3 + player.stats.force / 5;
            println!("  {}You absorb {} damage.{}", ui::CYAN, reduction.min(raw_dmg), ui::RESET);
            (raw_dmg - reduction).max(1)
        } else {
            raw_dmg
        };

        let final_dmg = if mroll.is_critical() { (incoming as f64 * 1.5) as i64 } else { incoming };
        player.take_damage(final_dmg);

        if !player.is_alive() {
            println!("\n  {}Your reflection laughs as you fall.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        println!();
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 2. THE ACCOUNTANT ───────────────────────────────────────────────────────

fn fight_the_accountant(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    // Records damage dealt, healing received, then sends the bill.
    let mut sc = seed;
    let mut turn = 0u32;
    let mut defend_count = 0u32;
    let mut damage_this_fight = 0i64;
    let lifetime_dmg = player.total_damage_dealt;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::CYAN, ui::RESET);
    println!("  {}║         THE ACCOUNTANT           ║{}", ui::CYAN, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::CYAN, ui::RESET);
    println!();
    for line in SPRITE_ACCOUNTANT.lines() { println!("  {}", line); }
    println!();
    println!("  No HP. No attack. Just a ledger.");
    println!("  It records your lifetime damage for 5 rounds.");
    println!("  Then it sends you THE BILL.");
    println!("  {}Defend to reduce the multiplier. Glass cannons die here.{}", ui::DIM, ui::RESET);
    println!("  {}Lifetime damage on record: {}{}", ui::YELLOW, lifetime_dmg, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER] to face judgment...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        if turn > 5 {
            // The bill arrives
            let bill_base = lifetime_dmg + damage_this_fight;
            let defend_reduction = (defend_count as f64 * 0.20).min(0.80);
            let bill = (bill_base as f64 * (1.0 - defend_reduction)) as i64;
            let bill_dmg = bill.max(1);

            ui::clear_screen();
            println!("\n  {}THE BILL HAS ARRIVED.{}", ui::RED, ui::RESET);
            println!();
            println!("  Lifetime damage dealt: {}{}{}", ui::YELLOW, lifetime_dmg, ui::RESET);
            println!("  Damage this fight:     {}{}{}", ui::YELLOW, damage_this_fight, ui::RESET);
            println!("  Defend reduction:      {}{}%{}", ui::CYAN, (defend_reduction * 100.0) as u32, ui::RESET);
            println!();
            println!("  {}BILL: {} damage{}", ui::RED, bill_dmg, ui::RESET);
            println!();
            ui::press_enter(&format!("  {}[ENTER] to receive payment...{}", ui::DIM, ui::RESET));

            player.take_damage(bill_dmg);
            if !player.is_alive() {
                println!("  {}Your power was your undoing.{}", ui::RED, ui::RESET);
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return BossOutcome::PlayerDied;
            }
            let (xp, gold) = boss_reward(player.floor, 600, 300);
            grant_victory(player, xp, gold);
            println!("  {}You survived the bill! The Accountant closes his ledger.{}", ui::GREEN, ui::RESET);
            println!("  +{xp} XP, +{gold} gold");
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerWon { xp, gold };
        }

        ui::clear_screen();
        println!("\n  {}THE ACCOUNTANT — Recording round {}/5{}", ui::CYAN, turn, ui::RESET);
        println!("  {}Your HP: {} / {}{}", ui::GREEN, player.current_hp, player.max_hp, ui::RESET);
        println!("  {}Lifetime damage recorded: {}{}", ui::YELLOW, lifetime_dmg + damage_this_fight, ui::RESET);
        println!("  {}Defend actions so far: {} ({}% reduction){}", ui::CYAN,
            defend_count, (defend_count * 20).min(80), ui::RESET);
        println!();
        println!("  [A] Attack (adds to ledger)  [D] Defend (reduces bill 20%)");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());

        match input.trim() {
            "d" => {
                defend_count += 1;
                println!("  {}Defend #{}: bill reduction now {}%.{}", ui::CYAN,
                    defend_count, (defend_count * 20).min(80), ui::RESET);
            }
            _ => {
                let base = 5 + player.stats.force / 5;
                let mut dmg = roll_damage(base, player.stats.force, sc);
                if roll.is_critical() { dmg = (dmg as f64 * 1.5) as i64; }
                damage_this_fight += dmg;
                println!("  {}You deal {} (now recorded in the ledger).{}", ui::YELLOW, dmg, ui::RESET);
            }
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 3. FIBONACCI HYDRA ──────────────────────────────────────────────────────

fn fight_fibonacci_hydra(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    const PHI: f64 = 1.6180339887498948482;
    let base_hp = (200 + player.floor as i64 * 30) as f64;
    let fib_seq = [1u32, 1, 2, 3, 5, 8, 13];
    let mut generation = 0usize;
    let mut sc = seed;
    let mut total_splits = 0u32;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::GREEN, ui::RESET);
    println!("  {}║       THE FIBONACCI HYDRA        ║{}", ui::GREEN, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::GREEN, ui::RESET);
    println!();
    for line in SPRITE_HYDRA.lines() { println!("  {}", line); }
    println!();
    println!("  One head. Kill it — it splits. Then splits again.");
    println!("  Splits follow Fibonacci: 1, 1, 2, 3, 5, 8, 13...");
    println!("  Each child has {}{}% of parent HP (1/φ). The swarm GROWS.{}",
        ui::RED, (100.0 / PHI) as u32, ui::RESET);
    println!("  Clear a full generation in one round, or survive 10 splits to win.");
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    // Fight each generation
    loop {
        if generation >= fib_seq.len() || total_splits >= 10 {
            let (xp, gold) = boss_reward(player.floor, 1000, 250);
            grant_victory(player, xp, gold);
            println!("\n  {}THE HYDRA COLLAPSES under its own mathematical weight!{}", ui::YELLOW, ui::RESET);
            println!("  +{xp} XP, +{gold} gold");
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerWon { xp, gold };
        }

        let count = fib_seq[generation];
        let gen_hp_each = (base_hp / PHI.powi(generation as i32)).max(1.0) as i64;
        let mut heads_hp: Vec<i64> = vec![gen_hp_each; count as usize];
        let mut cleared_in_one = true;
        let mut heads_killed_this_gen = 0u32;

        println!("\n  {}GENERATION {} — {} head(s), {} HP each{}",
            ui::YELLOW, generation + 1, count, gen_hp_each, ui::RESET);

        for head_idx in 0..count as usize {
            sc = advance_seed(sc);
            let head_dmg = 8 + player.floor as i64 * 2 + (generation as i64 * 4);

            ui::clear_screen();
            println!("  {}HYDRA Gen {} — Head {}/{}{}", ui::GREEN, generation + 1, head_idx + 1, count, ui::RESET);
            println!("  {}Your HP: {} / {}{}", ui::GREEN, player.current_hp, player.max_hp, ui::RESET);
            println!("  {}Head HP: {} / {}{}", ui::RED, heads_hp[head_idx], gen_hp_each, ui::RESET);
            println!("  {}Total splits: {}/10{}", ui::DIM, total_splits, ui::RESET);
            println!();
            println!("  [A] Attack  [D] Defend");
            println!();

            // Fight this head until it dies
            loop {
                let input = ui::prompt("  > ").to_lowercase();
                sc = advance_seed(sc);
                let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
                *last_roll = Some(roll.clone());

                match input.trim() {
                    "d" => {
                        cleared_in_one = false;
                        let _hroll = chaos_roll_verbose(0.5, sc.wrapping_add(42));
                        let dmg = (roll_damage(head_dmg, head_dmg, sc) / 2).max(1);
                        player.take_damage(dmg);
                        println!("  {}Head attacks for {} (defended)!{}", ui::RED, dmg, ui::RESET);
                    }
                    _ => {
                        let base = 5 + player.stats.force / 5;
                        let mut dmg = roll_damage(base, player.stats.force, sc);
                        if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
                        if roll.is_catastrophe() { dmg = 0; }
                        heads_hp[head_idx] = (heads_hp[head_idx] - dmg).max(0);
                        println!("  {}Dealt {} damage. Head HP: {}{}", ui::GREEN, dmg, heads_hp[head_idx], ui::RESET);

                        // Head counterattack
                        let hdmg = roll_damage(head_dmg, head_dmg, sc.wrapping_add(1));
                        player.take_damage(hdmg);
                        println!("  {}Head retaliates for {}!{}", ui::RED, hdmg, ui::RESET);
                    }
                }

                if !player.is_alive() {
                    println!("\n  {}The swarm overwhelms you.{}", ui::RED, ui::RESET);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return BossOutcome::PlayerDied;
                }

                if heads_hp[head_idx] <= 0 {
                    heads_killed_this_gen += 1;
                    total_splits += 1;
                    println!("  {}Head slain! It splits... ({} splits total){}", ui::YELLOW, total_splits, ui::RESET);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    break;
                }
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            }
        }

        if heads_killed_this_gen < count {
            cleared_in_one = false;
        }

        generation += 1;

        if cleared_in_one && count > 1 {
            let (xp, gold) = boss_reward(player.floor, 1200, 300);
            grant_victory(player, xp, gold);
            println!("\n  {}GENERATION CLEARED IN ONE SWEEP! Hydra cannot split further.{}", ui::YELLOW, ui::RESET);
            println!("  +{xp} XP, +{gold} gold");
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerWon { xp, gold };
        }
    }
}

// ─── 4. THE EIGENSTATE ───────────────────────────────────────────────────────

fn fight_the_eigenstate(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    let mut sc = seed;
    let mut turn = 0u32;
    let oneshot_dmg = player.max_hp + 1;
    let tanky_hp_max = 500 + player.floor as i64 * 100;
    let mut tanky_hp = tanky_hp_max;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::CYAN, ui::RESET);
    println!("  {}║         THE EIGENSTATE           ║{}", ui::CYAN, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::CYAN, ui::RESET);
    println!();
    for line in SPRITE_EIGENSTATE.lines() { println!("  {}", line); }
    println!();
    println!("  Superposition. Form A: massive HP, no attack.");
    println!("  Form B: 1 HP, instant-kill attack.");
    println!("  Your Luck determines the form — but {}inverted{}.", ui::RED, ui::RESET);
    println!("  {}Higher Luck = more Form B encounters. {}[T] Taunt to reveal safely.{}",
        ui::YELLOW, ui::DIM, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        // Form determination: positive chaos_roll (biased by inverted Luck) = Form A
        let luck_bias = -(player.stats.luck as f64 / 200.0).clamp(-0.8, 0.8); // inverted
        let form_roll = biased_chaos_roll(luck_bias, luck_bias, sc);
        let is_form_a = form_roll.final_value > 0.0;
        *last_roll = Some(form_roll.clone());

        ui::clear_screen();
        println!("\n  {}THE EIGENSTATE — Turn {}{}", ui::CYAN, turn, ui::RESET);
        println!("  {}Your HP: {} / {}{}", ui::GREEN, player.current_hp, player.max_hp, ui::RESET);
        println!("  {}Tanky form HP: {} / {}{}", ui::DIM, tanky_hp, tanky_hp_max, ui::RESET);
        println!("  Form is currently: {}?????{}", ui::MAGENTA, ui::RESET);
        println!();
        println!("  [A] Attack (danger: reveals and commits)");
        println!("  [T] Taunt  (safe reveal — see form before committing)");
        println!("  [D] Defend (survive Form B one-shot)");
        println!();

        let input = ui::prompt("  > ").to_lowercase();

        match input.trim() {
            "t" => {
                // Taunt reveals the form safely
                if is_form_a {
                    println!("  {}FORM A revealed — massive HP, no attack. Attack next round!{}", ui::GREEN, ui::RESET);
                } else {
                    println!("  {}FORM B revealed — 1 HP, one-shot attack. DEFEND next round!{}", ui::RED, ui::RESET);
                }
                // The eigenstate counterattacks lightly
                let taunt_dmg = 5 + player.floor as i64 / 2;
                player.take_damage(taunt_dmg);
                println!("  {}The Eigenstate probes you for {} damage.{}", ui::RED, taunt_dmg, ui::RESET);
            }
            "d" => {
                // Defending survives Form B one-shot
                if is_form_a {
                    println!("  {}Form A — you defended, no incoming attack.{}", ui::CYAN, ui::RESET);
                } else {
                    println!("  {}Form B ATTACKS — but you defend! Survived by the margin of Vitality.{}", ui::GREEN, ui::RESET);
                    let reduced = (oneshot_dmg - player.stats.vitality * 2).max(1);
                    player.take_damage(reduced);
                    println!("  {}Took {} damage.{}", ui::RED, reduced, ui::RESET);
                }
            }
            _ => {
                // Attack: commits to a form
                if is_form_a {
                    // Form A: deal damage to tanky form
                    let base = 5 + player.stats.force / 5;
                    let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
                    let mut dmg = roll_damage(base, player.stats.force, sc);
                    if roll.is_critical() { dmg = (dmg as f64 * 1.5) as i64; }
                    tanky_hp = (tanky_hp - dmg).max(0);
                    println!("  {}Form A! Dealt {} damage. Tanky HP: {}{}", ui::GREEN, dmg, tanky_hp, ui::RESET);

                    if tanky_hp <= 0 {
                        let (xp, gold) = boss_reward(player.floor, 700, 180);
                        grant_victory(player, xp, gold);
                        println!("\n  {}The Eigenstate collapses to a definite state. It dies.{}", ui::YELLOW, ui::RESET);
                        println!("  +{xp} XP, +{gold} gold");
                        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                        return BossOutcome::PlayerWon { xp, gold };
                    }
                } else {
                    // Form B: 1 HP but you took the one-shot
                    println!("  {}Form B — 1 HP! You kill it...{}", ui::GREEN, ui::RESET);
                    println!("  {}...but it fires first. {} DAMAGE.{}", ui::RED, oneshot_dmg, ui::RESET);
                    player.take_damage(oneshot_dmg);
                }
            }
        }

        if !player.is_alive() {
            println!("\n  {}The Eigenstate collapses onto you.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        println!();
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 5. THE TAXMAN ───────────────────────────────────────────────────────────

fn fight_the_taxman(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    let stolen_gold = player.gold;
    player.gold = 0;
    let mut taxman_hp = stolen_gold.max(1);
    let taxman_max_hp = taxman_hp;
    let mut sc = seed;
    let mut turn = 0u32;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::YELLOW, ui::RESET);
    println!("  {}║           THE TAXMAN             ║{}", ui::YELLOW, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::YELLOW, ui::RESET);
    println!();
    for line in SPRITE_TAXMAN.lines() { println!("  {}", line); }
    println!();
    println!("  Your gold: {}SEIZED.{}", ui::RED, ui::RESET);
    println!("  Taxman HP = your stolen gold: {}{}{}", ui::YELLOW, stolen_gold, ui::RESET);
    println!("  Every damage point = 1 gold returned.");
    println!("  He attacks for 1% of remaining HP each round.");
    println!("  {}Kill him before the 1% drains you.{}", ui::DIM, ui::RESET);
    println!();

    if stolen_gold == 0 {
        println!("  {}He looks at your empty wallet and walks away.{}", ui::DIM, ui::RESET);
        let (xp, gold) = boss_reward(player.floor, 300, 0);
        grant_victory(player, xp, gold);
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
        return BossOutcome::PlayerWon { xp, gold };
    }

    ui::press_enter(&format!("  {}[ENTER] to fight for your gold...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        ui::clear_screen();
        println!("\n  {}THE TAXMAN — Turn {}{}", ui::YELLOW, turn, ui::RESET);
        println!("  Your HP:    {}{}{} / {}", ui::GREEN, player.current_hp, player.max_hp, ui::RESET);
        println!("  Taxman HP:  {}{}{} / {} (= gold owed)",
            ui::YELLOW, taxman_hp, ui::RESET, taxman_max_hp);
        println!("  Gold recovered: {}{}{}", ui::YELLOW, taxman_max_hp - taxman_hp, ui::RESET);
        println!();
        println!("  [A] Attack   [D] Defend");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());
        let is_defend = input.trim() == "d";

        if !is_defend {
            let base = 5 + player.stats.force / 5 + player.stats.precision / 10;
            let mut dmg = roll_damage(base, player.stats.force, sc);
            if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
            if roll.is_catastrophe() { dmg = 0; }
            taxman_hp = (taxman_hp - dmg).max(0);
            println!("  {}Dealt {} — recovered {} gold.{}", ui::GREEN, dmg, dmg, ui::RESET);
        } else {
            println!("  {}You defend.{}", ui::CYAN, ui::RESET);
        }

        if taxman_hp <= 0 {
            let interest = taxman_max_hp / 5;
            let total = taxman_max_hp + interest;
            player.gold += total;
            let (xp, gold) = boss_reward(player.floor, 900, 0);
            grant_victory(player, xp, gold);
            println!("\n  {}THE TAXMAN DEFEATED!{}", ui::YELLOW, ui::RESET);
            println!("  Gold returned: {} + {}% interest = {}", taxman_max_hp, 20, total);
            println!("  +{xp} XP");
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerWon { xp, gold: total };
        }

        // Taxman attacks: 1% of remaining HP as damage
        let tax_atk = ((taxman_hp as f64 * 0.01) as i64).max(1);
        let incoming = if is_defend { (tax_atk / 2).max(1) } else { tax_atk };
        player.take_damage(incoming);
        println!("  {}The Taxman bills you {} HP (1% of remaining).{}", ui::RED, incoming, ui::RESET);

        if !player.is_alive() {
            player.gold = 0; // gold lost forever
            println!("\n  {}You fall. Your gold is gone forever.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 6. THE NULL ─────────────────────────────────────────────────────────────

fn fight_the_null(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    let null_max_hp = 300 + player.floor as i64 * 80;
    let mut null_hp = null_max_hp;
    let mut sc = seed;
    let mut turn = 0u32;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::DIM, ui::RESET);
    println!("  {}║            THE NULL              ║{}", ui::DIM, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::DIM, ui::RESET);
    println!();
    for line in SPRITE_NULL.lines() { println!("  {}", line); }
    println!();
    println!("  {}[NUL] applied.{} All your chaos rolls return 0.0.", ui::RED, ui::RESET);
    println!("  No crits. No backfires. No scaling. Just base stats.");
    println!("  The Null attacks with 10 unrestricted engines every turn.");
    println!("  {}Flat damage + flat defense + flat items. Raw math.{}", ui::DIM, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        ui::clear_screen();
        println!("\n  {}THE NULL — Turn {}{}", ui::DIM, turn, ui::RESET);
        println!("  Your HP:  {}{}{} / {} {}[NUL active — 0.0 engine output]{}",
            ui::GREEN, player.current_hp, ui::RESET, player.max_hp, ui::RED, ui::RESET);
        println!("  Null HP:  {}{}{} / {}", ui::RED, null_hp, ui::RESET, null_max_hp);
        println!();
        println!("  [A] Attack (base Force damage, no chaos)  [D] Defend");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        // Player rolls ALWAYS return 0.0 — no crits, no variance, just base
        let null_roll = ChaosRollResult {
            final_value: 0.0,
            chain: vec![crate::chaos_pipeline::ChainStep {
                engine_name: "[NUL] suppressed",
                input: 0.0,
                output: 0.0,
                seed_used: sc,
            }],
            game_value: 50,
        };
        *last_roll = Some(null_roll);
        let is_defend = input.trim() == "d";

        if !is_defend {
            // Flat damage: base stat formula, no chaos multiplier
            let dmg = (5 + player.stats.force / 5 + player.stats.precision / 10).max(1);
            null_hp = (null_hp - dmg).max(0);
            println!("  {}Base attack: {} damage (no chaos scaling).{}", ui::YELLOW, dmg, ui::RESET);

            if null_hp <= 0 {
                let (xp, gold) = boss_reward(player.floor, 700, 200);
                grant_victory(player, xp, gold);
                println!("\n  {}THE NULL DISSIPATES. The engines come back online.{}", ui::YELLOW, ui::RESET);
                println!("  +{xp} XP, +{gold} gold");
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return BossOutcome::PlayerWon { xp, gold };
            }
        } else {
            println!("  {}You defend.{}", ui::CYAN, ui::RESET);
        }

        // Null attacks with 10 engines (destiny roll)
        let null_attack_roll = crate::chaos_pipeline::destiny_roll(0.5, sc.wrapping_add(turn as u64));
        let null_base_dmg = 20 + player.floor as i64 * 5;
        let null_mult = (null_attack_roll.final_value + 1.5).max(0.1);
        let mut null_dmg = (null_base_dmg as f64 * null_mult) as i64;
        if is_defend {
            null_dmg = (null_dmg - player.stats.vitality / 3).max(1);
        }
        player.take_damage(null_dmg.max(1));
        println!("  {}The Null strikes with full 10-engine power: {} damage!{}", ui::RED, null_dmg, ui::RESET);

        if !player.is_alive() {
            println!("\n  {}Without chaos, you were only human.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 7. THE OUROBOROS ────────────────────────────────────────────────────────

fn fight_the_ouroboros(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    let peak_hit = (player.total_damage_dealt / player.kills.max(1) as i64 * 3).max(300);
    let max_hp = peak_hit.max(500 + player.floor as i64 * 60);
    let mut ouroboros_hp = max_hp;
    let ouroboros_atk = 15 + player.floor as i64 * 4;
    let mut sc = seed;
    let mut turn = 0u32;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::GREEN, ui::RESET);
    println!("  {}║         THE OUROBOROS            ║{}", ui::GREEN, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::GREEN, ui::RESET);
    println!();
    for line in SPRITE_OUROBOROS.lines() { println!("  {}", line); }
    println!();
    println!("  Heals to full every 3 rounds. Non-negotiable.");
    println!("  HP: {} (= 3× your average hit damage)", max_hp);
    println!("  {}Deal its full HP in 3 rounds or watch it reset.{}", ui::RED, ui::RESET);
    println!("  Combo builds. Heavy finishers. Burst damage only.");
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        // Heal every 3 rounds
        if turn > 1 && (turn - 1) % 3 == 0 {
            ouroboros_hp = max_hp;
            println!("\n  {}⟳ OUROBOROS HEALS TO FULL. Rounds reset.{}\n", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
        }

        let round_in_cycle = (turn - 1) % 3 + 1;

        ui::clear_screen();
        println!("\n  {}OUROBOROS — Turn {} (round {}/3 until heal){}",
            ui::GREEN, turn, round_in_cycle, ui::RESET);
        println!("  Your HP:     {}{}{} / {}", ui::GREEN, player.current_hp, ui::RESET, player.max_hp);
        println!("  Ouroboros:   {}{}{} / {}", ui::RED, ouroboros_hp, ui::RESET, max_hp);
        println!();
        println!("  [A] Attack   [H] Heavy Attack (+combo)   [D] Defend");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());
        let is_defend = input.trim() == "d";

        if !is_defend {
            let base = if input.trim() == "h" {
                12 + player.stats.force / 4
            } else {
                5 + player.stats.force / 5
            };
            let mut dmg = roll_damage(base, player.stats.force, sc);
            if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
            if roll.is_catastrophe() { dmg = 0; }
            ouroboros_hp = (ouroboros_hp - dmg).max(0);
            println!("  {}Dealt {}.{}", ui::GREEN, dmg, ui::RESET);

            if ouroboros_hp <= 0 {
                let (xp, gold) = boss_reward(player.floor, 900, 220);
                grant_victory(player, xp, gold);
                println!("\n  {}THE OUROBOROS DIES BEFORE ITS RESET.{}", ui::YELLOW, ui::RESET);
                println!("  +{xp} XP, +{gold} gold");
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return BossOutcome::PlayerWon { xp, gold };
            }
        }

        let oroll = chaos_roll_verbose(0.3, sc.wrapping_add(turn as u64 * 7919));
        let mut odm = roll_damage(ouroboros_atk, ouroboros_atk, sc.wrapping_add(1));
        if is_defend { odm = (odm - player.stats.vitality / 3).max(1); }
        if oroll.is_critical() { odm = (odm as f64 * 1.5) as i64; }
        player.take_damage(odm);
        println!("  {}Ouroboros attacks for {}!{}", ui::RED, odm, ui::RESET);

        if !player.is_alive() {
            println!("\n  {}The serpent swallows you whole.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 8. THE COLLATZ TITAN ────────────────────────────────────────────────────

fn fight_the_collatz_titan(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    // Starting HP: chaos-rolled 1000-9999
    let start_hp = (chaos_roll_verbose(0.5, seed).to_range(1000, 9999)).max(1000);
    let mut titan_hp = start_hp;
    let mut sc = seed;
    let mut turn = 0u32;
    let titan_atk = 10 + player.floor as i64 * 3;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::RED, ui::RESET);
    println!("  {}║        THE COLLATZ TITAN         ║{}", ui::RED, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::RED, ui::RESET);
    println!();
    for line in SPRITE_COLLATZ.lines() { println!("  {}", line); }
    println!();
    println!("  Starting HP: {}", start_hp);
    println!("  Each round: if even → HP÷2. If odd → HP×3+1.");
    println!("  {}Attack when it's LOW. Wait out the halving cascade.{}", ui::YELLOW, ui::RESET);
    println!("  {}Some seeds take hundreds of steps. Patience pays off.{}", ui::DIM, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        // Apply Collatz transformation to Titan HP
        let collatz_before = titan_hp;
        if titan_hp % 2 == 0 {
            titan_hp /= 2;
        } else {
            titan_hp = titan_hp * 3 + 1;
        }
        let next_step = if titan_hp % 2 == 0 { titan_hp / 2 } else { titan_hp * 3 + 1 };

        ui::clear_screen();
        println!("\n  {}COLLATZ TITAN — Turn {}{}", ui::RED, turn, ui::RESET);
        println!("  Your HP:    {}{}{} / {}", ui::GREEN, player.current_hp, ui::RESET, player.max_hp);
        println!("  Titan HP:   {}{}{} (was {} → Collatz → {})", ui::RED, titan_hp, ui::RESET, collatz_before, titan_hp);
        println!("  Next step:  {}{}{}  {}(attack now?){}", ui::YELLOW, next_step, ui::RESET,
            if next_step < titan_hp { ui::GREEN } else { ui::DIM }, ui::RESET);
        println!();
        println!("  [A] Attack now  [W] Wait (skip turn — Titan still attacks)  [D] Defend");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());

        match input.trim() {
            "w" => {
                println!("  {}You wait...{}", ui::DIM, ui::RESET);
            }
            "d" => {
                println!("  {}You defend.{}", ui::CYAN, ui::RESET);
                let odm = (roll_damage(titan_atk, titan_atk, sc) / 2).max(1);
                player.take_damage(odm);
                println!("  {}Titan attacks for {}!{}", ui::RED, odm, ui::RESET);
                if !player.is_alive() {
                    println!("\n  {}The Titan stomps you during a tripling cascade.{}", ui::RED, ui::RESET);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return BossOutcome::PlayerDied;
                }
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                continue;
            }
            _ => {
                // Attack
                let base = 5 + player.stats.force / 5 + player.stats.entropy / 10;
                let mut dmg = roll_damage(base, player.stats.force, sc);
                if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
                if roll.is_catastrophe() { dmg = 0; }
                titan_hp = (titan_hp - dmg).max(0);
                println!("  {}Dealt {} to the Titan. HP now: {}{}", ui::GREEN, dmg, titan_hp, ui::RESET);
            }
        }

        if titan_hp <= 0 {
            let (xp, gold) = boss_reward(player.floor, 850, 230);
            grant_victory(player, xp, gold);
            println!("\n  {}THE COLLATZ TITAN REACHES 1. SEQUENCE COMPLETE.{}", ui::YELLOW, ui::RESET);
            println!("  +{xp} XP, +{gold} gold");
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerWon { xp, gold };
        }

        // Titan counterattacks
        let odm = roll_damage(titan_atk, titan_atk, sc.wrapping_add(1));
        player.take_damage(odm);
        println!("  {}Titan attacks for {}!{}", ui::RED, odm, ui::RESET);

        if !player.is_alive() {
            println!("\n  {}The sequence was too long. You ran out of HP.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 9. THE COMMITTEE ────────────────────────────────────────────────────────

fn fight_the_committee(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    // 5 members, each immune to one damage type
    // Immunities: 0=physical, 1=spell, 2=item, 3=buff, 4=taunt-only
    let member_names = ["Member A", "Member B", "Member C", "Member D", "Member E"];
    let member_hp_base = 200 + player.floor as i64 * 40;
    let immunities = [
        "physical",
        "spell",
        "item",
        "buff",
        "taunt_only", // Member E: only vulnerable to Taunt
    ];
    // Shuffle immunities by seed
    let mut immunity_order: Vec<usize> = (0..5).collect();
    let mut sc = seed;
    for i in 0..5 {
        sc = advance_seed(sc);
        let j = (sc % (5 - i as u64)) as usize + i;
        immunity_order.swap(i, j);
    }

    println!("\n  {}╔══════════════════════════════════╗{}", ui::CYAN, ui::RESET);
    println!("  {}║          THE COMMITTEE           ║{}", ui::CYAN, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::CYAN, ui::RESET);
    println!();
    for line in SPRITE_COMMITTEE.lines() { println!("  {}", line); }
    println!();
    println!("  Five enemies. Each immune to a different damage type.");
    println!("  Wrong attack = {}heals the enemy{}. Discover immunities by trial.", ui::RED, ui::RESET);
    println!("  {}One member only responds to Taunt.{}", ui::YELLOW, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    let mut members_alive = vec![true; 5];
    let mut member_hp: Vec<i64> = vec![member_hp_base; 5];
    let mut discovered: Vec<Option<&'static str>> = vec![None; 5];

    loop {
        sc = advance_seed(sc);

        // Check if all members are dead
        if members_alive.iter().all(|&a| !a) {
            let (xp, gold) = boss_reward(player.floor, 1100, 280);
            grant_victory(player, xp, gold);
            println!("\n  {}THE COMMITTEE IS DISSOLVED.{}", ui::YELLOW, ui::RESET);
            println!("  +{xp} XP, +{gold} gold");
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerWon { xp, gold };
        }

        ui::clear_screen();
        println!("\n  {}THE COMMITTEE{}", ui::CYAN, ui::RESET);
        println!("  {}Your HP: {} / {}{}", ui::GREEN, player.current_hp, player.max_hp, ui::RESET);
        println!();
        for i in 0..5 {
            if members_alive[i] {
                let immunity_hint = discovered[i].unwrap_or("???");
                println!("  [{}] {} — HP: {}  Immune: {}", i + 1, member_names[i], member_hp[i], immunity_hint);
            } else {
                println!("  [{}] {} — {}DEAD{}", i + 1, member_names[i], ui::DIM, ui::RESET);
            }
        }
        println!();
        println!("  Actions: [1-5] target member, then [A]ttack [S]pell [I]tem [T]aunt [D]efend");
        println!();

        let target_input = ui::prompt("  Target member # > ");
        let target_idx = match target_input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= 5 && members_alive[n - 1] => n - 1,
            _ => {
                println!("  {}Invalid target.{}", ui::RED, ui::RESET);
                continue;
            }
        };

        let action_input = ui::prompt("  Action > ").to_lowercase();
        let attack_type = match action_input.trim() {
            "a" => "physical",
            "s" => "spell",
            "i" => "item",
            "t" => "taunt_only",
            "d" => {
                // Defend — all members attack
                let total_dmg: i64 = members_alive.iter().enumerate()
                    .filter(|(_, &a)| a)
                    .map(|(_, _)| {
                        let d = 8 + player.floor as i64;
                        d
                    }).sum();
                let reduced = (total_dmg - player.stats.vitality / 3).max(1);
                player.take_damage(reduced);
                println!("  {}You defend. Committee attacks for {}.{}", ui::CYAN, reduced, ui::RESET);
                if !player.is_alive() {
                    println!("\n  {}The vote was unanimous: you die.{}", ui::RED, ui::RESET);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return BossOutcome::PlayerDied;
                }
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                continue;
            }
            _ => "physical",
        };

        let member_immunity = immunities[immunity_order[target_idx]];
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());

        if attack_type == member_immunity {
            // Wrong type — heals enemy
            let heal_amt = 50 + player.floor as i64 * 5;
            member_hp[target_idx] += heal_amt;
            discovered[target_idx] = Some(member_immunity);
            println!("  {}IMMUNE! {} heals for {}! Immunity discovered: {}{}",
                ui::RED, member_names[target_idx], heal_amt, member_immunity, ui::RESET);
        } else {
            // Correct type — deal damage
            let base = 5 + player.stats.force / 5;
            let mut dmg = roll_damage(base, player.stats.force, sc);
            if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
            if roll.is_catastrophe() { dmg = 0; }
            member_hp[target_idx] = (member_hp[target_idx] - dmg).max(0);
            println!("  {}Hit! {} takes {} damage (HP: {}).{}",
                ui::GREEN, member_names[target_idx], dmg, member_hp[target_idx], ui::RESET);
            if member_hp[target_idx] <= 0 {
                members_alive[target_idx] = false;
                let remaining = members_alive.iter().filter(|&&a| a).count();
                println!("  {}Member {} eliminated! {} remain.{}", ui::YELLOW, target_idx + 1, remaining, ui::RESET);
            }
        }

        // Surviving members attack
        for i in 0..5 {
            if members_alive[i] {
                let mdmg = 6 + player.floor as i64 / 2;
                player.take_damage(mdmg);
            }
        }
        let alive_count = members_alive.iter().filter(|&&a| a).count();
        let total_atk = alive_count as i64 * (6 + player.floor as i64 / 2);
        if total_atk > 0 {
            println!("  {}Committee attacks ({} members): {} total damage.{}", ui::RED, alive_count, total_atk, ui::RESET);
        }

        if !player.is_alive() {
            println!("\n  {}The committee voted you dead.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 10. THE RECURSION ───────────────────────────────────────────────────────

fn fight_the_recursion(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    let recursion_max_hp = player.max_hp; // Starts with same HP as you
    let mut recursion_hp = recursion_max_hp;
    let mut sc = seed;
    let mut turn = 0u32;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::MAGENTA, ui::RESET);
    println!("  {}║          THE RECURSION           ║{}", ui::MAGENTA, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::MAGENTA, ui::RESET);
    println!();
    for line in SPRITE_RECURSION.lines() { println!("  {}", line); }
    println!();
    println!("  Every hit you deal is reflected back to you simultaneously.");
    println!("  HP: {} (= your max HP)", recursion_max_hp);
    println!("  {}Win by having more HP than it. Trade blows, keep defending.{}", ui::DIM, ui::RESET);
    println!("  {}Defend reduces YOUR incoming reflection. Recursion takes full.{}", ui::YELLOW, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        ui::clear_screen();
        println!("\n  {}THE RECURSION — Turn {}{}", ui::MAGENTA, turn, ui::RESET);
        println!("  Your HP:      {}{}{} / {}", ui::GREEN, player.current_hp, ui::RESET, player.max_hp);
        println!("  Recursion HP: {}{}{} / {}", ui::RED, recursion_hp, ui::RESET, recursion_max_hp);
        println!();
        println!("  [A] Attack (deal+take damage)   [D] Defend (reflect reduced)   [F] Flee");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());

        match input.trim() {
            "f" => {
                let fr = chaos_roll_verbose(player.stats.luck as f64 * 0.01, sc.wrapping_add(13));
                if fr.is_success() {
                    println!("  {}You break the loop and escape.{}", ui::GREEN, ui::RESET);
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return BossOutcome::Escaped;
                }
                println!("  {}The recursion loops you back.{}", ui::RED, ui::RESET);
            }
            "d" => {
                // Defend: deal some damage, take reduced reflection
                let base = 3 + player.stats.force / 8;
                let dmg = roll_damage(base, player.stats.force, sc);
                let reflection = (dmg - player.stats.vitality / 2).max(1);
                recursion_hp = (recursion_hp - dmg).max(0);
                player.take_damage(reflection);
                println!("  {}Dealt {} — reflected {} (reduced by VIT).{}", ui::CYAN, dmg, reflection, ui::RESET);
            }
            _ => {
                // Attack: deal damage, take same damage back
                let base = 5 + player.stats.force / 5;
                let mut dmg = roll_damage(base, player.stats.force, sc);
                if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
                if roll.is_catastrophe() { dmg = 0; }
                recursion_hp = (recursion_hp - dmg).max(0);
                player.take_damage(dmg); // reflection — same amount
                println!("  {}Dealt {} and took {} reflected damage simultaneously.{}",
                    ui::YELLOW, dmg, dmg, ui::RESET);
            }
        }

        if recursion_hp <= 0 {
            let (xp, gold) = boss_reward(player.floor, 950, 240);
            grant_victory(player, xp, gold);
            println!("\n  {}THE RECURSION STACK OVERFLOWS.{}", ui::YELLOW, ui::RESET);
            println!("  +{xp} XP, +{gold} gold");
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerWon { xp, gold };
        }

        if !player.is_alive() {
            println!("\n  {}The recursion consumed you. Stack overflow.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 11. THE PARADOX ─────────────────────────────────────────────────────────

fn fight_the_paradox(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    let mut sc = seed;
    let mut turn = 0u32;
    let mut cunning_bonus = 0i64; // accumulates from [O]bserve
    let mut failed_talks = 0u32;
    let base_difficulty = (40 + player.floor as i64 / 2).min(90);

    let questions = [
        "Is P equal to NP?",
        "Does the Collatz conjecture always terminate?",
        "Is the Riemann Hypothesis true?",
        "Can mathematics be both complete and consistent?",
        "What is the last digit of Graham's number?",
        "Is the halting problem decidable for chaos engines?",
        "Define infinity. But briefly.",
        "Is a random number truly random if it was computed?",
    ];

    println!("\n  {}╔══════════════════════════════════╗{}", ui::DIM, ui::RESET);
    println!("  {}║           THE PARADOX            ║{}", ui::DIM, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::DIM, ui::RESET);
    println!();
    for line in SPRITE_PARADOX.lines() { println!("  {}", line); }
    println!();
    println!("  Immune to damage. Cannot be fled. Cannot be killed.");
    println!("  The only way out: {}[T] Talk{} — CUNNING-based chaos roll.", ui::YELLOW, ui::RESET);
    println!("  Fail = lose a spell. Fail again = lose an item.");
    println!("  {}[O] Observe{}: skip a turn, +5 CUNNING bonus to next talk.", ui::CYAN, ui::RESET);
    println!("  {}You cannot attack. You cannot run. Only words work.{}", ui::DIM, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        let question_idx = (sc % questions.len() as u64) as usize;

        ui::clear_screen();
        println!("\n  {}THE PARADOX — Turn {}{}", ui::DIM, turn, ui::RESET);
        println!("  {}Your HP: {} / {}{}", ui::GREEN, player.current_hp, player.max_hp, ui::RESET);
        println!("  {}Failed talks: {}   Cunning bonus: +{}{}", ui::YELLOW, failed_talks, cunning_bonus, ui::RESET);
        println!("  Spells remaining: {}   Items remaining: {}", player.known_spells.len(), player.inventory.len());
        println!();
        println!("  {}The Paradox asks:{} {}", ui::MAGENTA, ui::RESET, questions[question_idx]);
        println!();
        println!("  [T] Talk (CUNNING roll)   [O] Observe (+5 CUN bonus next round)");
        println!();

        let input = ui::prompt("  > ").to_lowercase();

        match input.trim() {
            "o" => {
                cunning_bonus += 5;
                println!("  {}You observe carefully. +5 Cunning bonus.{}", ui::CYAN, ui::RESET);
                // The Paradox strips something small for wasting time
                if turn > 3 {
                    let small_dmg = 5 + player.floor as i64;
                    player.take_damage(small_dmg);
                    println!("  {}The Paradox grows impatient. -{} HP.{}", ui::RED, small_dmg, ui::RESET);
                }
            }
            "t" => {
                let effective_cunning = player.stats.cunning + cunning_bonus;
                let talk_roll = biased_chaos_roll(
                    effective_cunning as f64 * 0.01,
                    effective_cunning as f64 / 200.0,
                    sc,
                );
                *last_roll = Some(talk_roll.clone());
                let roll_val = talk_roll.to_range(0, 100);
                let difficulty = base_difficulty + failed_talks as i64 * 10;
                cunning_bonus = 0; // reset bonus

                for line in talk_roll.combat_trace_lines("Talk (CUNNING)", &format!("rolled {roll_val} vs difficulty {difficulty}")) {
                    println!("{}", line);
                }

                if roll_val >= difficulty {
                    let (xp, gold) = boss_reward(player.floor, 1200, 350);
                    grant_victory(player, xp, gold);
                    println!("\n  {}THE PARADOX ACCEPTS YOUR ANSWER.{}", ui::YELLOW, ui::RESET);
                    println!("  It dissolves into unresolved equations.");
                    println!("  +{xp} XP, +{gold} gold");
                    ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                    return BossOutcome::PlayerWon { xp, gold };
                } else {
                    failed_talks += 1;
                    println!("  {}WRONG. The Paradox frowns.{}", ui::RED, ui::RESET);
                    if failed_talks == 1 && !player.known_spells.is_empty() {
                        let lost = player.known_spells.remove(0);
                        println!("  {}Spell ERASED: {}{}", ui::RED, lost.name, ui::RESET);
                    } else if failed_talks >= 2 && !player.inventory.is_empty() {
                        let lost = player.inventory.remove(0);
                        println!("  {}Item DESTROYED: {}{}", ui::RED, lost.name, ui::RESET);
                    } else if failed_talks > 2 {
                        // Pure HP damage as last resort
                        let dmg = player.max_hp / 4;
                        player.take_damage(dmg);
                        println!("  {}The Paradox strikes you for {}!{}", ui::RED, dmg, ui::RESET);
                    }
                }
            }
            _ => {
                println!("  {}The Paradox ignores your action.{}", ui::DIM, ui::RESET);
            }
        }

        if !player.is_alive() {
            println!("\n  {}The Paradox proved you finite.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}

// ─── 12. THE ALGORITHM REBORN ────────────────────────────────────────────────

fn fight_the_algorithm_reborn(
    player: &mut Character,
    seed: u64,
    last_roll: &mut Option<ChaosRollResult>,
) -> BossOutcome {
    // HP = player's total XP (the longer you've been playing, the harder it is)
    let algo_max_hp = (player.xp as i64).max(10_000);
    let mut algo_hp = algo_max_hp;
    let algo_atk = 30 + player.floor as i64 * 8;
    let mut sc = seed;
    let mut turn = 0u32;

    println!("\n  {}╔══════════════════════════════════╗{}", ui::RED, ui::RESET);
    println!("  {}║     THE ALGORITHM REBORN         ║{}", ui::RED, ui::RESET);
    println!("  {}╚══════════════════════════════════╝{}", ui::RED, ui::RESET);
    println!();
    for line in SPRITE_ALGORITHM.lines() { println!("  {}", line); }
    println!();
    println!("  HP = your lifetime XP: {}{}{}", ui::YELLOW, algo_max_hp, ui::RESET);
    println!("  {}The longer you survived, the harder this fight.{}", ui::RED, ui::RESET);
    println!("  It uses the INVERSE of your passive engine nodes.");
    println!("  {}All its rolls are inverted — your crits are its crits.{}", ui::DIM, ui::RESET);
    println!();
    ui::press_enter(&format!("  {}[ENTER] for the final confrontation...{}", ui::DIM, ui::RESET));

    loop {
        turn += 1;
        sc = advance_seed(sc);

        ui::clear_screen();
        println!("\n  {}THE ALGORITHM REBORN — Turn {}{}", ui::RED, turn, ui::RESET);
        println!("  Your HP:       {}{}{} / {}", ui::GREEN, player.current_hp, ui::RESET, player.max_hp);
        println!("  Algorithm HP:  {}{}{} / {}", ui::RED, algo_hp, ui::RESET, algo_max_hp);
        println!();
        println!("  [A] Attack   [H] Heavy Attack   [D] Defend   [F] Flee");
        println!();

        let input = ui::prompt("  > ").to_lowercase();
        let roll = chaos_roll_verbose(player.stats.force as f64 * 0.01, sc);
        *last_roll = Some(roll.clone());
        let is_defend = input.trim() == "d";

        if input.trim() == "f" {
            let fr = chaos_roll_verbose(player.stats.luck as f64 * 0.01, sc.wrapping_add(31337));
            if fr.is_success() {
                println!("  {}The Algorithm loses track of you. You escape.{}", ui::GREEN, ui::RESET);
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return BossOutcome::Escaped;
            }
            println!("  {}The Algorithm predicted your escape route.{}", ui::RED, ui::RESET);
        } else if !is_defend {
            let base = if input.trim() == "h" {
                12 + player.stats.force / 4
            } else {
                5 + player.stats.force / 5
            };
            let mut dmg = roll_damage(base, player.stats.force, sc);
            if roll.is_critical()    { dmg = (dmg as f64 * 1.5) as i64; }
            if roll.is_catastrophe() { dmg = 0; }
            algo_hp = (algo_hp - dmg).max(0);

            for line in roll.combat_trace_lines("Attack vs Algorithm", &format!("dealt {dmg}")) {
                println!("{}", line);
            }

            if algo_hp <= 0 {
                let (xp, gold) = boss_reward(player.floor, 5000, 2000);
                grant_victory(player, xp, gold);
                println!("\n  {}THE ALGORITHM REBORN CRASHES. EXCEPTION: UNHANDLED HERO.{}", ui::YELLOW, ui::RESET);
                println!("  You have beaten math itself.");
                println!("  +{xp} XP, +{gold} gold");
                ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
                return BossOutcome::PlayerWon { xp, gold };
            }
        }

        // Algorithm uses INVERTED engine outputs (cursed roll)
        let algo_roll = crate::chaos_pipeline::cursed_chaos_roll(0.5, sc.wrapping_add(0xA16_0726_1706_8D00));
        let algo_mult = (algo_roll.final_value + 1.5).max(0.1);
        let mut algo_dmg = (algo_atk as f64 * algo_mult) as i64;
        if is_defend {
            algo_dmg = (algo_dmg - player.stats.vitality / 2).max(1);
        }
        player.take_damage(algo_dmg.max(1));
        println!("  {}The Algorithm strikes for {} (inverted engine chain).{}", ui::RED, algo_dmg, ui::RESET);

        if !player.is_alive() {
            println!("\n  {}The Algorithm assimilates you.{}", ui::RED, ui::RESET);
            ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
            return BossOutcome::PlayerDied;
        }
        ui::press_enter(&format!("  {}[ENTER]...{}", ui::DIM, ui::RESET));
    }
}
