//! Per-run statistics tracker.
//!
//! RunStats is embedded in Character (with serde default) and updated inline
//! during play. At run-end it drives the death screen and engine report card.

use serde::{Deserialize, Serialize};

// ── Per-engine stats ──────────────────────────────────────────────────────────

/// Statistics for one math engine accumulated during a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRunStats {
    pub engine_id: u8,           // 0-9
    pub name: String,
    pub uses: u64,
    pub output_sum: f64,         // for average
    pub best_output: f64,
    pub worst_output: f64,
    pub times_in_killing_blow: u64,  // engine was in chain for a kill
    pub times_in_death_blow: u64,    // engine was in chain that killed player
}

impl EngineRunStats {
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

    pub fn record(&mut self, output: f64) {
        self.uses += 1;
        self.output_sum += output;
        if output > self.best_output  { self.best_output  = output; }
        if output < self.worst_output { self.worst_output = output; }
    }

    pub fn avg_output(&self) -> f64 {
        if self.uses == 0 { 0.0 } else { self.output_sum / self.uses as f64 }
    }
}

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
pub enum RollOutcome { Success, Failure, Critical, Catastrophe }

// ── Main RunStats struct ──────────────────────────────────────────────────────

/// All statistics accumulated during a single run. Serialized with Character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStats {
    // ── Combat ────────────────────────────────────────────────────────────────
    pub damage_dealt: i64,
    pub damage_taken: i64,
    pub crits_landed: u32,
    pub crits_received: u32,
    pub total_backfires: u32,
    pub deaths_to_backfire: u32,
    pub overflows_caused: u32,
    pub overflow_deaths: u32,
    pub enemies_fled: u32,
    pub enemies_pitied_you: u32,
    pub enemies_talked_down: u32,
    pub highest_single_hit: i64,
    pub lowest_single_hit: i64,        // can be negative (accidentally healed enemy)
    pub highest_single_hit_spell: String,
    pub combo_peak: u32,               // longest hit combo
    pub combo_current: u32,
    pub flee_attempts: u32,
    pub flee_successes: u32,
    pub attacks_missed: u32,

    // ── Engine statistics (10 slots) ─────────────────────────────────────────
    pub engines: [EngineRunStats; 10],
    pub total_rolls: u64,
    pub rolls_success: u64,
    pub rolls_failure: u64,
    pub rolls_critical: u64,
    pub rolls_catastrophe: u64,
    pub longest_positive_streak: u32,
    pub longest_negative_streak: u32,
    streak_positive_current: u32,
    streak_negative_current: u32,

    // ── Economy ───────────────────────────────────────────────────────────────
    pub gold_collected: i64,
    pub gold_spent: i64,
    pub items_found: u32,
    pub items_equipped: u32,
    pub spells_learned: u32,
    pub spells_cast: u32,
    pub most_cast_spell: String,
    pub most_cast_spell_count: u32,
    pub spell_cast_counts: Vec<(String, u32)>,
    pub shops_visited_empty: u32,
    pub passive_nodes_allocated: u32,

    // ── Body ─────────────────────────────────────────────────────────────────
    pub injuries_sustained: u32,
    pub body_parts_severed: u32,
    pub body_parts_at_zero: u32,
    pub head_survived: bool,

    // ── Run summary ───────────────────────────────────────────────────────────
    pub floors_reached: u32,
    pub rooms_cleared: u32,
    pub kill_count: u32,
    pub cause_of_death: String,
    pub final_blow_damage: i64,
    pub final_blow_engine_chain: Vec<String>,
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
    pub fn new() -> Self { Self::default() }

    // ── Recording methods ─────────────────────────────────────────────────────

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

    pub fn record_damage_taken(&mut self, amount: i64, is_crit: bool) {
        self.damage_taken += amount;
        if is_crit { self.crits_received += 1; }
    }

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

    pub fn record_kill(&mut self, engine_id: u8) {
        self.kill_count += 1;
        if let Some(e) = self.engines.get_mut(engine_id as usize) {
            e.times_in_killing_blow += 1;
        }
    }

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

    pub fn record_backfire(&mut self, killed_player: bool) {
        self.total_backfires += 1;
        if killed_player { self.deaths_to_backfire += 1; }
    }

    pub fn record_flee_attempt(&mut self, succeeded: bool) {
        self.flee_attempts += 1;
        if succeeded { self.flee_successes += 1; } else { self.enemies_fled += 1; /* sic: enemy escaped */ }
    }

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

    pub fn nemesis_engine(&self) -> &EngineRunStats {
        self.engines.iter()
            .filter(|e| e.uses > 0)
            .min_by(|a, b| a.avg_output().partial_cmp(&b.avg_output()).unwrap())
            .unwrap_or(&self.engines[8]) // Collatz default
    }

    pub fn ally_engine(&self) -> &EngineRunStats {
        self.engines.iter()
            .filter(|e| e.uses > 0)
            .max_by(|a, b| a.avg_output().partial_cmp(&b.avg_output()).unwrap())
            .unwrap_or(&self.engines[5]) // Mandelbrot default
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_rolls == 0 { return 0.0; }
        (self.rolls_success + self.rolls_critical) as f64 / self.total_rolls as f64
    }

    // ── Engine report card ────────────────────────────────────────────────────

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
