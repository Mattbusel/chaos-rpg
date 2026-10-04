//! Per-run statistics tracker.
//!
//! RunStats is embedded in Character (with serde default) and updated inline
//! during play. At run-end it drives the death screen and engine report card.

use serde::{Deserialize, Serialize};

// ── Per-engine stats ──────────────────────────────────────────────────────────

/// Statistics for one math engine accumulated during a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRunStats {
    /// Which of the 10 math engines this is (0 to 9, same order as `ENGINE_NAMES`).
    pub engine_id: u8,           // 0-9
    /// Display name of the engine.
    pub name: String,
    /// How many times the engine ran in a roll chain this run.
    pub uses: u64,
    /// Sum of every output, used to compute the average.
    pub output_sum: f64,         // for average
    /// Highest output this engine produced this run.
    pub best_output: f64,
    /// Lowest output this engine produced this run.
    pub worst_output: f64,
    /// How many killing blows on enemies had this engine in their chain.
    pub times_in_killing_blow: u64,  // engine was in chain for a kill
    /// How many hits that killed the player had this engine in their chain.
    pub times_in_death_blow: u64,    // engine was in chain that killed player
}

impl EngineRunStats {
    /// Fresh stats for engine `id`, with its name looked up from `ENGINE_NAMES`.
    pub fn new(id: u8) -> Self {
        Self {
            engine_id: id,
            name: ENGINE_NAMES[id as usize % ENGINE_NAMES.len()].to_string(),
            uses: 0,
            output_sum: 0.0,
            best_output: f64::NEG_INFINITY,
            worst_output: f64::INFINITY,
            times_in_killing_blow: 0,
            times_in_death_blow: 0,
        }
    }

    /// Count one use of the engine and fold `output` into the sum, best and worst.
    pub fn record(&mut self, output: f64) {
        self.uses += 1;
        self.output_sum += output;
        if output > self.best_output  { self.best_output  = output; }
        if output < self.worst_output { self.worst_output = output; }
    }

    /// Mean output over all uses, or 0.0 if the engine never ran.
    pub fn avg_output(&self) -> f64 {
        if self.uses == 0 { 0.0 } else { self.output_sum / self.uses as f64 }
    }
}

/// Display names of the 10 chaos engines, indexed by engine id.
pub const ENGINE_NAMES: &[&str] = &[
    "Lorenz Attractor",
    "Fourier Harmonic",
    "Prime Density Sieve",
    "Riemann Zeta Partial",
    "Fibonacci Spiral",
    "Mandelbrot Escape",
    "Logistic Map",
    "Euler's Totient",
    "Collatz Chain",
    "Modular Exp Hash",
];

// ── Roll outcome counters ─────────────────────────────────────────────────────

/// Outcome classification for a chaos roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollOutcome {
    /// The roll succeeded.
    Success,
    /// The roll failed.
    Failure,
    /// The roll was a critical success.
    Critical,
    /// The roll was a catastrophic failure.
    Catastrophe,
}

// ── Main RunStats struct ──────────────────────────────────────────────────────

/// All statistics accumulated during a single run. Serialized with Character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStats {
    // ── Combat ────────────────────────────────────────────────────────────────
    /// Total damage the player dealt this run.
    pub damage_dealt: i64,
    /// Total damage the player took this run.
    pub damage_taken: i64,
    /// Critical hits the player landed.
    pub crits_landed: u32,
    /// Critical hits the player received.
    pub crits_received: u32,
    /// Times a roll backfired on the player.
    pub total_backfires: u32,
    /// Times a backfire killed the player (0 or 1 per run).
    pub deaths_to_backfire: u32,
    /// Integer overflows the player caused with damage.
    pub overflows_caused: u32,
    /// Deaths caused by an overflow.
    pub overflow_deaths: u32,
    /// Fights where the enemy got away (counted on failed flee attempts).
    pub enemies_fled: u32,
    /// Times an enemy spared the player out of pity.
    pub enemies_pitied_you: u32,
    /// Times the player talked an enemy out of fighting.
    pub enemies_talked_down: u32,
    /// Largest single hit the player dealt.
    pub highest_single_hit: i64,
    /// Smallest single hit the player dealt; negative means the hit healed the enemy.
    pub lowest_single_hit: i64,        // can be negative (accidentally healed enemy)
    /// Name of the spell behind the largest hit, if it was a spell.
    pub highest_single_hit_spell: String,
    /// Longest run of consecutive critical hits.
    pub combo_peak: u32,               // longest hit combo
    /// Current run of consecutive critical hits.
    pub combo_current: u32,
    /// Times the player tried to flee.
    pub flee_attempts: u32,
    /// Times the player fled successfully.
    pub flee_successes: u32,
    /// Attacks that missed.
    pub attacks_missed: u32,

    // ── Engine statistics (10 slots) ─────────────────────────────────────────
    /// Per-engine statistics, indexed by engine id.
    pub engines: [EngineRunStats; 10],
    /// Number of chaos rolls made this run.
    pub total_rolls: u64,
    /// Rolls that succeeded.
    pub rolls_success: u64,
    /// Rolls that failed.
    pub rolls_failure: u64,
    /// Rolls that were critical successes.
    pub rolls_critical: u64,
    /// Rolls that were catastrophes.
    pub rolls_catastrophe: u64,
    /// Longest streak of successful (or critical) rolls.
    pub longest_positive_streak: u32,
    /// Longest streak of failed (or catastrophic) rolls.
    pub longest_negative_streak: u32,
    streak_positive_current: u32,
    streak_negative_current: u32,

    // ── Economy ───────────────────────────────────────────────────────────────
    /// Gold picked up this run.
    pub gold_collected: i64,
    /// Gold spent this run.
    pub gold_spent: i64,
    /// Items found this run.
    pub items_found: u32,
    /// Items equipped this run.
    pub items_equipped: u32,
    /// Spells learned this run.
    pub spells_learned: u32,
    /// Spells cast this run.
    pub spells_cast: u32,
    /// Name of the spell cast most often.
    pub most_cast_spell: String,
    /// How many times the most-cast spell was cast.
    pub most_cast_spell_count: u32,
    /// Cast count for every spell used, as (name, count).
    pub spell_cast_counts: Vec<(String, u32)>,
    /// Shops visited that had nothing to sell.
    pub shops_visited_empty: u32,
    /// Passive tree nodes allocated this run.
    pub passive_nodes_allocated: u32,

    // ── Body ─────────────────────────────────────────────────────────────────
    /// Injuries the player suffered.
    pub injuries_sustained: u32,
    /// Body parts severed.
    pub body_parts_severed: u32,
    /// Body parts reduced to zero health.
    pub body_parts_at_zero: u32,
    /// Whether the head survived to the end of the run.
    pub head_survived: bool,

    // ── Run summary ───────────────────────────────────────────────────────────
    /// Deepest floor reached.
    pub floors_reached: u32,
    /// Rooms cleared.
    pub rooms_cleared: u32,
    /// Enemies killed.
    pub kill_count: u32,
    /// What killed the player, as shown on the death screen.
    pub cause_of_death: String,
    /// Damage of the hit that killed the player.
    pub final_blow_damage: i64,
    /// Engine chain behind the killing hit, by engine name.
    pub final_blow_engine_chain: Vec<String>,
    /// Final roll value of the killing hit.
    pub final_blow_roll_result: f64,
}

impl Default for RunStats {
    fn default() -> Self {
        Self {
            damage_dealt: 0,
            damage_taken: 0,
            crits_landed: 0,
            crits_received: 0,
            total_backfires: 0,
            deaths_to_backfire: 0,
            overflows_caused: 0,
            overflow_deaths: 0,
            enemies_fled: 0,
            enemies_pitied_you: 0,
            enemies_talked_down: 0,
            highest_single_hit: 0,
            lowest_single_hit: 0,
            highest_single_hit_spell: String::new(),
            combo_peak: 0,
            combo_current: 0,
            flee_attempts: 0,
            flee_successes: 0,
            attacks_missed: 0,
            engines: [
                EngineRunStats::new(0), EngineRunStats::new(1),
                EngineRunStats::new(2), EngineRunStats::new(3),
                EngineRunStats::new(4), EngineRunStats::new(5),
                EngineRunStats::new(6), EngineRunStats::new(7),
                EngineRunStats::new(8), EngineRunStats::new(9),
            ],
            total_rolls: 0,
            rolls_success: 0,
            rolls_failure: 0,
            rolls_critical: 0,
            rolls_catastrophe: 0,
            longest_positive_streak: 0,
            longest_negative_streak: 0,
            streak_positive_current: 0,
            streak_negative_current: 0,
            gold_collected: 0,
            gold_spent: 0,
            items_found: 0,
            items_equipped: 0,
            spells_learned: 0,
            spells_cast: 0,
            most_cast_spell: String::new(),
            most_cast_spell_count: 0,
            spell_cast_counts: Vec::new(),
            shops_visited_empty: 0,
            passive_nodes_allocated: 0,
            injuries_sustained: 0,
            body_parts_severed: 0,
            body_parts_at_zero: 0,
            head_survived: true,
            floors_reached: 1,
            rooms_cleared: 0,
            kill_count: 0,
            cause_of_death: String::from("Unknown"),
            final_blow_damage: 0,
            final_blow_engine_chain: Vec::new(),
            final_blow_roll_result: 0.0,
        }
    }
}

impl RunStats {
    /// Empty stats for a new run.
    pub fn new() -> Self { Self::default() }

    // ── Recording methods ─────────────────────────────────────────────────────

    /// Record a hit the player dealt, updating totals, the biggest and smallest hit, and the crit combo.
    pub fn record_damage_dealt(&mut self, amount: i64, spell_name: Option<&str>, is_crit: bool) {
        self.damage_dealt += amount;
        if amount > self.highest_single_hit {
            self.highest_single_hit = amount;
            if let Some(s) = spell_name { self.highest_single_hit_spell = s.to_string(); }
        }
        if amount < self.lowest_single_hit {
            self.lowest_single_hit = amount;
        }
        if is_crit {
            self.crits_landed += 1;
            self.combo_current += 1;
            if self.combo_current > self.combo_peak { self.combo_peak = self.combo_current; }
        } else {
            self.combo_current = 0;
        }
    }

    /// Record damage the player took.
    pub fn record_damage_taken(&mut self, amount: i64, is_crit: bool) {
        self.damage_taken += amount;
        if is_crit { self.crits_received += 1; }
    }

    /// Record one engine's output in a roll and update the success and failure streaks.
    pub fn record_engine_roll(&mut self, engine_id: u8, output: f64, outcome: RollOutcome) {
        self.total_rolls += 1;
        if let Some(e) = self.engines.get_mut(engine_id as usize) {
            e.record(output);
        }
        match outcome {
            RollOutcome::Success     => { self.rolls_success += 1; self.streak_positive_current += 1; self.streak_negative_current = 0; }
            RollOutcome::Critical    => { self.rolls_critical += 1; self.streak_positive_current += 1; self.streak_negative_current = 0; }
            RollOutcome::Failure     => { self.rolls_failure += 1; self.streak_negative_current += 1; self.streak_positive_current = 0; }
            RollOutcome::Catastrophe => { self.rolls_catastrophe += 1; self.streak_negative_current += 1; self.streak_positive_current = 0; }
        }
        if self.streak_positive_current > self.longest_positive_streak {
            self.longest_positive_streak = self.streak_positive_current;
        }
        if self.streak_negative_current > self.longest_negative_streak {
            self.longest_negative_streak = self.streak_negative_current;
        }
    }

    /// Count a kill and credit the engine that was in the killing chain.
    pub fn record_kill(&mut self, engine_id: u8) {
        self.kill_count += 1;
        if let Some(e) = self.engines.get_mut(engine_id as usize) {
            e.times_in_killing_blow += 1;
        }
    }

    /// Count a spell cast and keep track of the most-cast spell.
    pub fn record_spell_cast(&mut self, spell_name: &str) {
        self.spells_cast += 1;
        if let Some(entry) = self.spell_cast_counts.iter_mut().find(|(n, _)| n == spell_name) {
            entry.1 += 1;
            if entry.1 > self.most_cast_spell_count {
                self.most_cast_spell_count = entry.1;
                self.most_cast_spell = spell_name.to_string();
            }
        } else {
            self.spell_cast_counts.push((spell_name.to_string(), 1));
        }
    }

    /// Count a backfire, and whether it killed the player.
    pub fn record_backfire(&mut self, killed_player: bool) {
        self.total_backfires += 1;
        if killed_player { self.deaths_to_backfire += 1; }
    }

    /// Count a flee attempt; a failed attempt also counts as the enemy getting away.
    pub fn record_flee_attempt(&mut self, succeeded: bool) {
        self.flee_attempts += 1;
        if succeeded { self.flee_successes += 1; } else { self.enemies_fled += 1; /* sic: enemy escaped */ }
    }

    /// Record how the player died: cause, final damage, engine chain, roll value and the engine to blame.
    pub fn set_death(
        &mut self,
        cause: &str,
        final_dmg: i64,
        chain: Vec<String>,
        roll_result: f64,
        killing_engine_id: u8,
    ) {
        self.cause_of_death = cause.to_string();
        self.final_blow_damage = final_dmg;
        self.final_blow_engine_chain = chain;
        self.final_blow_roll_result = roll_result;
        if let Some(e) = self.engines.get_mut(killing_engine_id as usize) {
            e.times_in_death_blow += 1;
        }
    }

    // ── Derived statistics ────────────────────────────────────────────────────

    /// The engine with the lowest average output this run (Collatz if none ran).
    pub fn nemesis_engine(&self) -> &EngineRunStats {
        self.engines.iter()
            .filter(|e| e.uses > 0)
            .min_by(|a, b| a.avg_output().partial_cmp(&b.avg_output()).unwrap())
            .unwrap_or(&self.engines[8]) // Collatz default
    }

    /// The engine with the highest average output this run (Mandelbrot if none ran).
    pub fn ally_engine(&self) -> &EngineRunStats {
        self.engines.iter()
            .filter(|e| e.uses > 0)
            .max_by(|a, b| a.avg_output().partial_cmp(&b.avg_output()).unwrap())
            .unwrap_or(&self.engines[5]) // Mandelbrot default
    }

    /// Share of rolls that succeeded or were critical, from 0.0 to 1.0.
    pub fn success_rate(&self) -> f64 {
        if self.total_rolls == 0 { return 0.0; }
        (self.rolls_success + self.rolls_critical) as f64 / self.total_rolls as f64
    }

    // ── Engine report card ────────────────────────────────────────────────────

    /// A text table of every engine used this run, for the end screen.
    pub fn engine_report_card(&self) -> String {
        let mut out = String::from(
            "ENGINE REPORT CARD\n\
             ┌──────────────────────┬──────┬─────────┬─────────┬───────────┐\n\
             │ Engine               │ Uses │ Avg Out │ Best    │ Worst     │\n\
             ├──────────────────────┼──────┼─────────┼─────────┼───────────┤\n"
        );
        for e in &self.engines {
            if e.uses == 0 { continue; }
            out.push_str(&format!(
                "│ {:20} │ {:4} │ {:+7.3} │ {:+7.3} │ {:+9.3} │\n",
                e.name, e.uses, e.avg_output(), e.best_output, e.worst_output
            ));
        }
        let nemesis = self.nemesis_engine();
        let ally = self.ally_engine();
        out.push_str(&format!(
            "├──────────────────────┴──────┴─────────┴─────────┴───────────┤\n\
             │ YOUR NEMESIS: {:20} (avg {:+.3})           │\n\
             │ YOUR ALLY:    {:20} (avg {:+.3})           │\n\
             └──────────────────────────────────────────────────────────────┘",
            nemesis.name, nemesis.avg_output(),
            ally.name, ally.avg_output(),
        ));
        out
    }

    // ── Death screen text ─────────────────────────────────────────────────────

    /// Text lines for the death screen summarising the run.
    pub fn death_screen_lines(&self, char_name: &str, class_name: &str,
                               power_tier: &str, misery_index: f64,
                               underdog: f64, defiance_rolls: u64,
                               spite_spent: f64) -> Vec<String> {
        vec![
            format!("  Name: {:12}  Class: {}", char_name, class_name),
            format!("  Power Tier: {:25}  Misery: {:.0}", power_tier, misery_index),
            format!("  Underdog: ×{:.1}    Defiance Rolls: {}    Spite Spent: {:.0}",
                underdog, defiance_rolls, spite_spent),
            String::new(),
            format!("  Floors reached: {:4}  Rooms cleared: {}", self.floors_reached, self.rooms_cleared),
            format!("  Enemies slain: {:5}  Enemies fled: {}  Enemies pitied you: {}",
                self.kill_count, self.flee_successes, self.enemies_pitied_you),
            format!("  Damage dealt:  {:8}  Damage taken: {}", self.damage_dealt, self.damage_taken),
            format!("  Highest hit: {} ({})", self.highest_single_hit, self.highest_single_hit_spell),
            format!("  Backfires: {}  Crits: {}  Overflows: {}", self.total_backfires, self.crits_landed, self.overflows_caused),
            String::new(),
            format!("  Total chaos rolls: {}  Success: {:.1}%",
                self.total_rolls, self.success_rate() * 100.0),
            format!("  Longest positive streak: {}  Longest negative: {}",
                self.longest_positive_streak, self.longest_negative_streak),
            String::new(),
            format!("  Gold collected: {:6}  Gold spent: {}", self.gold_collected, self.gold_spent),
            format!("  Spells cast: {}  Most used: {} ({}×)",
                self.spells_cast, self.most_cast_spell, self.most_cast_spell_count),
            format!("  Passive nodes: {}", self.passive_nodes_allocated),
            String::new(),
            format!("  Cause of death: {}", self.cause_of_death),
            format!("  Final blow: {} damage", self.final_blow_damage),
        ]
    }
}
