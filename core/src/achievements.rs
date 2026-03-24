// CHAOS RPG — Achievement System
// Persistent across all runs. Stored in chaos_rpg_achievements.json next to the exe.

use serde::{Deserialize, Serialize};

// ── Rarity ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
    Omega,
}

impl AchievementRarity {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Common   => "COMMON",
            Self::Uncommon => "UNCOMMON",
            Self::Rare     => "RARE",
            Self::Epic     => "EPIC",
            Self::Legendary=> "LEGENDARY",
            Self::Mythic   => "MYTHIC",
            Self::Omega    => "OMEGA",
        }
    }
    pub fn stars(&self) -> &'static str {
        match self {
            Self::Common   => "[*]",
            Self::Uncommon => "[**]",
            Self::Rare     => "[***]",
            Self::Epic     => "[****]",
            Self::Legendary=> "[*****]",
            Self::Mythic   => "[######]",
            Self::Omega    => "[OMEGA]",
        }
    }
}

// ── Achievement definition ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub rarity:      AchievementRarity,
    pub unlocked:    bool,
    pub unlock_date: String,
}

impl Achievement {
    pub fn new(id: &'static str, name: &'static str, description: &'static str, rarity: AchievementRarity) -> Self {
        Self {
            id:          id.to_string(),
            name:        name.to_string(),
            description: description.to_string(),
            rarity, unlocked: false, unlock_date: String::new(),
        }
    }
}

// ── Achievement store ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementStore {
    pub achievements: Vec<Achievement>,
    /// Achievements unlocked this session (cleared on new session). Used for banner display.
    #[serde(default)]
    pub pending_banners: Vec<String>,
    /// Rarity names parallel to pending_banners for rich display.
    #[serde(default)]
    pub pending_banner_rarities: Vec<String>,
}

impl Default for AchievementStore {
    fn default() -> Self {
        Self {
            achievements: all_achievements(),
            pending_banners: Vec::new(),
            pending_banner_rarities: Vec::new(),
        }
    }
}

impl AchievementStore {
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(mut store) = serde_json::from_str::<AchievementStore>(&data) {
                // Merge in any new achievements added since last save
                let defaults = all_achievements();
                for def in defaults {
                    if !store.achievements.iter().any(|a| a.id == def.id) {
                        store.achievements.push(def);
                    }
                }
                store.pending_banners.clear();
                return store;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), json);
        }
    }

    fn path() -> std::path::PathBuf {
        let mut p = std::env::current_exe().unwrap_or_default();
        p.pop();
        p.push("chaos_rpg_achievements.json");
        p
    }

    /// Unlock an achievement by id. Returns true if newly unlocked.
    pub fn unlock(&mut self, id: &str) -> bool {
        if let Some(a) = self.achievements.iter_mut().find(|a| a.id == id) {
            if !a.unlocked {
                a.unlocked = true;
                a.unlock_date = chrono_date();
                self.pending_banners.push(a.name.to_string());
                let rarity_name = format!("{:?}", a.rarity).to_lowercase();
                self.pending_banner_rarities.push(rarity_name);
                return true;
            }
        }
        false
    }

    pub fn is_unlocked(&self, id: &str) -> bool {
        self.achievements.iter().any(|a| a.id == id && a.unlocked)
    }

    pub fn unlocked_count(&self) -> usize {
        self.achievements.iter().filter(|a| a.unlocked).count()
    }

    pub fn total_count(&self) -> usize {
        self.achievements.len()
    }

    pub fn by_rarity(&self, rarity: AchievementRarity) -> Vec<&Achievement> {
        self.achievements.iter().filter(|a| a.rarity == rarity).collect()
    }

    pub fn pop_banner(&mut self) -> Option<String> {
        if self.pending_banners.is_empty() { None } else { Some(self.pending_banners.remove(0)) }
    }

    /// Pop the next pending banner together with its rarity name string.
    pub fn pop_banner_with_rarity(&mut self) -> Option<(String, String)> {
        if self.pending_banners.is_empty() { return None; }
        let text = self.pending_banners.remove(0);
        let rarity = if !self.pending_banner_rarities.is_empty() {
            self.pending_banner_rarities.remove(0)
        } else {
            "common".to_string()
        };
        Some((text, rarity))
    }
}

fn chrono_date() -> String {
    // Simple date without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days  = secs / 86400;
    let year  = 1970 + days / 365;
    let month = (days % 365) / 30 + 1;
    let day   = (days % 365) % 30 + 1;
    format!("{}-{:02}-{:02}", year, month, day)
}

// ── Event types for checking ──────────────────────────────────────────────────

/// A snapshot of end-of-run data for achievement checking.
#[derive(Debug, Clone)]
pub struct RunSummary {
    pub floor:            u32,
    pub kills:            u64,
    pub level:            u32,
    pub class:            String,
    pub difficulty:       String,
    pub damage_dealt:     i64,
    pub damage_taken:     i64,
    pub highest_hit:      i64,
    pub spells_cast:      u32,
    pub items_used:       u32,
    pub gold:             i64,
    pub misery_index:     f64,
    pub corruption:       u32,
    pub power_tier:       String,
    pub total_stats:      i64,
    pub cause_of_death:   String,
    pub rooms_cleared:    u32,
    pub deaths_in_run:    u32,
    pub fled_count:       u32,
    pub all_stats_negative: bool,
    pub total_runs:       u32,
    pub total_deaths:     u32,
    pub won:              bool,
    pub seed:             u64,
}

/// A snapshot of a single combat round for achievement checking.
#[derive(Debug, Clone, Default)]
pub struct CombatSnapshot {
    pub damage_dealt:   i64,
    pub is_crit:        bool,
    pub is_spell:       bool,
    pub spell_name:     String,
    pub enemy_remaining_hp: i64,
    pub player_hp_pct:  f32,
    pub round:          u32,
    pub roll_value:     f64,
    pub enemy_name:     String,
    pub is_boss:        bool,
    pub fled:           bool,
    pub won_combat:     bool,
    pub took_no_damage: bool,
}

// ── Check helpers ─────────────────────────────────────────────────────────────

impl AchievementStore {
    /// Call at end of every combat round.
    pub fn check_combat(&mut self, s: &CombatSnapshot) {
        // First Blood — kill first enemy (any kill)
        if s.won_combat { self.unlock("first_blood"); }
        // Speedrunner — kill in one action
        if s.won_combat && s.round == 1 { self.unlock("speedrunner"); }
        // Untouchable — clear floor without taking damage (caller tracks)
        if s.won_combat && s.took_no_damage { self.unlock("untouchable"); }
        // Glass Cannon — 1000+ damage while below 10% HP
        if s.damage_dealt >= 1000 && s.player_hp_pct < 0.10 { self.unlock("glass_cannon"); }
        // Overkill — dealt > 10x remaining hp
        if s.enemy_remaining_hp < 0 && s.damage_dealt > s.enemy_remaining_hp.abs() * 10 {
            self.unlock("overkill");
        }
        // Paper Cut — dealt exactly 1
        if s.damage_dealt == 1 { self.unlock("paper_cut"); }
        // Negative Damage — dealt negative
        if s.damage_dealt < 0 { self.unlock("negative_damage"); }
        // The Pipeline Speaks — see chaos trace (caller triggers)
        // The Number — roll exactly 42.00
        if (s.roll_value - 42.0).abs() < 0.005 { self.unlock("the_number"); }
        // Golden Ratio
        if (s.roll_value - 1.618).abs() < 0.005 { self.unlock("golden_ratio"); }
        // Euler's Number
        if (s.roll_value - 2.718).abs() < 0.005 { self.unlock("eulers_number"); }
        // Perfectly Balanced
        if s.roll_value.abs() < 0.005 { self.unlock("perfectly_balanced"); }
        // Tactical Retreat
        if s.fled { self.unlock("first_retreat"); }
        // 404 Damage
        if s.damage_dealt == 404 { self.unlock("404_damage"); }
        // Boss-specific
        if s.is_boss && s.won_combat {
            let id = format!("boss_{}", s.enemy_name.to_lowercase().replace(' ', "_"));
            self.unlock(&id);
        }
    }

    /// Call at end of every run (win or loss).
    pub fn check_run(&mut self, r: &RunSummary) {
        // Progression
        if r.floor >= 1  { self.unlock("baby_steps"); }
        if r.floor >= 10 { self.unlock("double_digits"); }
        if r.floor >= 25 { self.unlock("quarter_century"); }
        if r.floor >= 50 { self.unlock("the_deep"); }
        if r.floor >= 75 { self.unlock("are_you_okay"); }
        if r.floor >= 100{ self.unlock("the_algorithm_awaits"); }
        if r.floor >= 101{ self.unlock("beyond_the_algorithm"); }
        if r.floor >= 200{ self.unlock("infinity_and_beyond"); }
        if r.floor >= 500{ self.unlock("omega_long_run"); }

        // Kills
        if r.kills >= 100  { self.unlock("centurion"); }
        if r.kills >= 500  { self.unlock("mass_extinction"); }
        if r.kills >= 1000 { self.unlock("genocide_route"); }

        // Gold
        if r.gold >= 10_000 { self.unlock("millionaire"); }

        // Level
        if r.level >= 99 { self.unlock("level_cap"); }

        // Death
        if r.floor == 1 && !r.won { self.unlock("humbling"); }
        if r.deaths_in_run > 0 { self.unlock("graveyard_shift"); }

        // Misery
        if r.misery_index >= 5_000.0   { self.unlock("misery_5k"); }
        if r.misery_index >= 10_000.0  { self.unlock("defiant"); }
        if r.misery_index >= 25_000.0  { self.unlock("cosmic_punchline"); }
        if r.misery_index >= 50_000.0  { self.unlock("transcendent"); }
        if r.misery_index >= 100_000.0 { self.unlock("published_failure"); }

        // Stats
        if r.all_stats_negative { self.unlock("negative_everything"); }
        if r.total_stats < 0 && r.won { self.unlock("underdog_victory"); }

        // Meta
        if r.total_runs >= 10  { self.unlock("try_try_again"); }
        if r.total_runs >= 25  { self.unlock("runs_25"); }
        if r.total_runs >= 50  { self.unlock("runs_50"); }
        if r.total_runs >= 100 { self.unlock("veteran"); }
        if r.total_runs >= 200 { self.unlock("runs_200"); }
        if r.total_runs >= 500 { self.unlock("runs_500"); }
        if r.total_deaths >= 1000 { self.unlock("thousand_deaths"); }

        // Floor milestones (new)
        if r.floor >= 15  { self.unlock("floor_15"); }
        if r.floor >= 30  { self.unlock("floor_30"); }
        if r.floor >= 150 { self.unlock("floor_150"); }
        if r.floor >= 250 { self.unlock("floor_250"); }
        if r.floor >= 300 { self.unlock("floor_300"); }
        if r.floor >= 500 { self.unlock("omega_long_run"); }

        // Spell counts
        if r.spells_cast >= 100  { self.unlock("spells_100"); }
        if r.spells_cast >= 1000 { self.unlock("spells_1000"); }

        // Score milestones
        if r.kills >= 2000 { self.unlock("max_kills_run"); }

        // Wins
        if r.won {
            self.unlock("first_clear");
            match r.difficulty.as_str() {
                "Easy"   => { self.unlock("easy_clear"); }
                "Normal" => { self.unlock("normal_clear"); }
                "Brutal" => { self.unlock("brutal_clear"); }
                "Chaos"  => { self.unlock("chaos_clear"); self.unlock("chaos_conquered"); }
                _ => {}
            }
        }

        // Difficulty at depth
        if r.difficulty == "Chaos" && r.floor >= 50 { self.unlock("chaos_floor50"); }

        // Power tier
        if r.power_tier.contains("OMEGA") { self.unlock("omega_tier"); }

        // Cause of death patterns
        if r.cause_of_death.contains("Chaos") || r.cause_of_death.contains("catastrophe") {
            self.unlock("killed_by_math");
        }
        if r.cause_of_death.contains("Accountant") { self.unlock("accountants_bill"); }
        if r.cause_of_death.contains("Committee")  { self.unlock("democratic_execution"); }
        if r.cause_of_death.contains("Ouroboros")  { self.unlock("ouroboros_loop"); }
        if r.cause_of_death.contains("Recursion")  { self.unlock("recursion_overflow"); }

        // Story mode win
        if r.won { self.unlock("first_clear"); }

        // Speedrun Story
        // (caller must pass total_rounds if needed; skipping for now)
    }

    /// Call when specific in-game events fire.
    pub fn check_event(&mut self, event: &str, value: i64) {
        match event {
            "bench_used"       => { self.unlock("bench_warmer"); }
            "shop_visited"     => { self.unlock("window_shopper"); }
            "item_picked_up"   => {
                // Loot Goblin tracks across runs — needs cumulative count
                if value >= 50 { self.unlock("loot_goblin"); }
            }
            "tutorial_opened"  => { self.unlock("read_the_manual"); }
            "char_created"     => { self.unlock("identity_crisis"); }
            "trace_viewed"     => { self.unlock("pipeline_speaks"); }
            "passive_allocated"=> { self.unlock("first_branch"); }
            "flee_succeeded"   => {
                if value >= 10 { self.unlock("tactical_retreat"); }
            }
            "corrupt_used"     => {
                if value == 5  { self.unlock("transcendent_corruption"); }
                if value == -1 { self.unlock("gone_reduced"); }
            }
            "math_absent"      => { self.unlock("math_absent"); }
            "math_absent_5"    => { self.unlock("full_dismemberment"); }
            "portal_used"      => {
                if value >= 5 { self.unlock("portal_junkie"); }
            }
            "shrine_visited"   => {
                if value >= 20 { self.unlock("shrine_hopper"); }
            }
            "trap_hit"         => {
                if value >= 30 { self.unlock("trap_magnet"); }
            }
            "theme_cycled"     => { /* track externally */ }
            "daily_completed"  => {
                if value >= 7  { self.unlock("daily_driver"); }
                if value >= 30 { self.unlock("streak"); }
            }
            "keystone_alloc"   => { self.unlock("keystone"); }
            "modded_config"        => { self.unlock("modder"); }
            "config_loaded"        => { self.unlock("config_loaded"); }
            "config_gold_bonus"    => { self.unlock("config_gold_bonus"); }
            "config_hard_mode"     => { self.unlock("config_hard_mode"); }
            "chaos_engine_viz"     => { self.unlock("chaos_engine_viz"); }
            "item_filter_used"     => { self.unlock("item_filter_used"); }
            "shatter_first"        => { self.unlock("shatter_first"); }
            "imbue_first"          => { self.unlock("imbue_first"); }
            "craft_op"             => {
                // value = cumulative craft count
                if value >= 10  { self.unlock("craft_10"); }
                if value >= 100 { self.unlock("craft_100"); }
                if value >= 500 { self.unlock("craft_500"); }
            }
            "daily_first"          => { self.unlock("daily_first"); }
            "daily_submitted"      => {
                // value = rank
                if value == 1 { self.unlock("daily_rank1"); self.unlock("daily_top3"); }
                if value <= 3 { self.unlock("daily_top3"); }
            }
            "first_spell"          => { self.unlock("first_spell"); }
            "spells_cumulative"    => {
                if value >= 100  { self.unlock("spells_100"); }
                if value >= 1000 { self.unlock("spells_1000"); }
            }
            "runs_cumulative"      => {
                if value >= 25  { self.unlock("runs_25"); }
                if value >= 50  { self.unlock("runs_50"); }
                if value >= 200 { self.unlock("runs_200"); }
                if value >= 500 { self.unlock("runs_500"); }
            }
            _ => {}
        }
    }
}

// ── Full achievement list ─────────────────────────────────────────────────────

pub fn all_achievements() -> Vec<Achievement> {
    use AchievementRarity::*;
    vec![
        // Getting Started
        Achievement::new("first_blood",       "First Blood",          "Kill your first enemy",                                        Common),
        Achievement::new("baby_steps",        "Baby Steps",           "Clear Floor 1",                                                Common),
        Achievement::new("window_shopper",    "Window Shopper",       "Visit a shop",                                                 Common),
        Achievement::new("identity_crisis",   "Identity Crisis",      "Create your first character",                                  Common),
        Achievement::new("graveyard_shift",   "Graveyard Shift",      "Die for the first time",                                       Common),
        Achievement::new("try_try_again",     "Try, Try Again",       "Start your 10th run",                                          Common),
        Achievement::new("loot_goblin",       "Loot Goblin",          "Pick up 50 items across all runs",                             Common),
        Achievement::new("bench_warmer",      "Bench Warmer",         "Use a crafting bench for the first time",                      Common),
        Achievement::new("read_the_manual",   "Read the Manual",      "Open the tutorial",                                            Common),
        Achievement::new("first_clear",       "First Clear",          "Win a run",                                                    Common),

        // Combat
        Achievement::new("overkill",          "Overkill",             "Deal more than 10x an enemy's remaining HP in one hit",        Common),
        Achievement::new("paper_cut",         "Paper Cut",            "Deal exactly 1 damage",                                        Common),
        Achievement::new("untouchable",       "Untouchable",          "Clear a floor without taking any damage",                      Uncommon),
        Achievement::new("glass_cannon",      "Glass Cannon",         "Deal 1000+ damage in one hit while below 10% HP",              Rare),
        Achievement::new("speedrunner",       "Speedrunner",          "Kill an enemy in a single action",                             Common),
        Achievement::new("first_retreat",     "Tactical Retreat",     "Successfully flee from 10 encounters",                         Uncommon),
        Achievement::new("tactical_retreat",  "Tactical Retreat Pro", "Successfully flee from 10 encounters in one run",              Uncommon),
        Achievement::new("404_damage",        "404 Damage Not Found", "Deal exactly 404 damage",                                      Rare),
        Achievement::new("negative_damage",   "Negative Damage",      "Deal negative damage (heal the enemy with an attack)",         Epic),

        // Chaos Engine
        Achievement::new("pipeline_speaks",   "The Pipeline Speaks",  "View the chaos trace visualization",                           Uncommon),
        Achievement::new("the_number",        "The Number",           "Have a chaos roll return exactly 42.00",                       Epic),
        Achievement::new("golden_ratio",      "Golden Ratio",         "Have a chaos roll return exactly 1.618",                       Epic),
        Achievement::new("eulers_number",     "Euler's Number",       "Have a chaos roll return exactly 2.718",                       Epic),
        Achievement::new("perfectly_balanced","Perfectly Balanced",   "Get a final chaos roll value of exactly 0.00",                 Epic),
        Achievement::new("killed_by_math",    "Killed by Math",       "Die to a chaos catastrophe (roll below -95)",                  Uncommon),

        // Death & Misery
        Achievement::new("humbling",          "Humbling Experience",  "Die on Floor 1",                                               Common),
        Achievement::new("misery_5k",         "Misery Loves Company", "Reach 5,000 Misery Index",                                     Uncommon),
        Achievement::new("defiant",           "Defiant",              "Reach 10,000 Misery Index and trigger Defiance",               Rare),
        Achievement::new("cosmic_punchline",  "Cosmic Punchline",     "Reach 25,000 Misery Index",                                    Epic),
        Achievement::new("transcendent",      "Transcendent Suffering","Reach 50,000 Misery Index",                                   Legendary),
        Achievement::new("published_failure", "Published Failure",    "Reach 100,000 Misery Index. Enter the Hall of Misery.",         Mythic),
        Achievement::new("accountants_bill",  "The Accountant's Bill","Die to The Accountant's invoice",                              Rare),
        Achievement::new("democratic_execution","Democratic Execution","Die to The Committee's vote",                                  Rare),
        Achievement::new("ouroboros_loop",    "Ouroboros Loop",       "Die to The Ouroboros after it healed to full 3+ times",        Rare),
        Achievement::new("recursion_overflow","Recursion Overflow",   "Die to The Recursion's reflected damage exceeding 10,000",     Epic),

        // Progression
        Achievement::new("double_digits",     "Double Digits",        "Reach Floor 10",                                               Uncommon),
        Achievement::new("quarter_century",   "Quarter Century",      "Reach Floor 25",                                               Uncommon),
        Achievement::new("the_deep",          "The Deep",             "Reach Floor 50",                                               Rare),
        Achievement::new("are_you_okay",      "Are You Okay?",        "Reach Floor 75",                                               Rare),
        Achievement::new("the_algorithm_awaits","The Algorithm Awaits","Reach Floor 100",                                             Epic),
        Achievement::new("beyond_the_algorithm","Beyond the Algorithm","Reach Floor 101+",                                            Epic),
        Achievement::new("infinity_and_beyond","Infinity and Beyond", "Reach Floor 200 in Infinite mode",                             Legendary),
        Achievement::new("centurion",         "Centurion",            "Kill 100 enemies in a single run",                             Uncommon),
        Achievement::new("mass_extinction",   "Mass Extinction",      "Kill 500 enemies in a single run",                             Rare),
        Achievement::new("genocide_route",    "Genocide Route",       "Kill 1,000 enemies in a single run",                           Legendary),
        Achievement::new("millionaire",       "Millionaire",          "Accumulate 10,000 gold in a single run",                       Rare),
        Achievement::new("level_cap",         "Level Cap? What Level Cap?","Reach level 99",                                          Epic),

        // Power & Builds
        Achievement::new("omega_tier",        "OMEGA",                "Reach OMEGA power tier",                                       Legendary),
        Achievement::new("negative_everything","Negative Everything", "Have all 7 stats be negative simultaneously",                  Epic),
        Achievement::new("underdog_victory",  "Underdog Victory",     "Complete Story mode with negative total stats",                Epic),

        // Body System
        Achievement::new("math_absent",       "MATH.ABSENT",          "Lose a body part to MATH.ABSENT severity",                     Uncommon),
        Achievement::new("full_dismemberment","Full Dismemberment",   "Have 5+ body parts at MATH.ABSENT simultaneously",             Legendary),

        // Crafting
        Achievement::new("transcendent_corruption","Transcendent Corruption","Hit the 5% Transcendent outcome on Corrupt",            Epic),
        Achievement::new("gone_reduced",      "Gone. Reduced to Atoms.","Have an item destroyed by Corrupt",                          Rare),

        // Passive Tree
        Achievement::new("first_branch",      "First Branch",         "Allocate your first passive point",                            Common),
        Achievement::new("keystone",          "Keystone",             "Reach and allocate a Keystone node",                           Rare),

        // Exploration
        Achievement::new("portal_junkie",     "Portal Junkie",        "Use 5 portals in a single run",                                Uncommon),
        Achievement::new("shrine_hopper",     "Shrine Hopper",        "Visit 20 shrines across all runs",                             Uncommon),
        Achievement::new("trap_magnet",       "Trap Magnet",          "Hit 30 traps across all runs",                                 Uncommon),

        // Daily
        Achievement::new("daily_driver",      "Daily Driver",         "Complete 7 daily seed runs",                                   Uncommon),
        Achievement::new("streak",            "Streak",               "Complete 30 daily seed runs in a row",                         Rare),

        // Meta
        Achievement::new("veteran",           "Veteran",              "Complete 100 total runs",                                      Rare),
        Achievement::new("thousand_deaths",   "Thousand Deaths",      "Die 1,000 times across all runs",                              Epic),
        Achievement::new("modder",            "Modder",               "Load a custom chaos_config.toml",                              Rare),

        // Boss Slayers
        Achievement::new("boss_the_mirror",       "Shattered Mirror",     "Defeat The Mirror",                                        Rare),
        Achievement::new("boss_the_accountant",   "Debt Free",            "Defeat The Accountant",                                    Rare),
        Achievement::new("boss_the_fibonacci_hydra","Hydra Pruner",       "Defeat The Fibonacci Hydra",                               Rare),
        Achievement::new("boss_the_eigenstate",   "Observer",             "Defeat The Eigenstate",                                    Rare),
        Achievement::new("boss_the_taxman",       "Tax Exempt",           "Defeat The Taxman",                                        Rare),
        Achievement::new("boss_the_null",         "Void Breaker",         "Defeat The Null",                                          Rare),
        Achievement::new("boss_the_paradox",      "Paradox Resolver",     "Defeat The Paradox",                                       Epic),
        Achievement::new("boss_the_recursion",    "Recursion Terminator", "Defeat The Recursion",                                     Epic),
        Achievement::new("boss_the_committee",    "Committee Disbanded",  "Defeat The Committee",                                     Epic),
        Achievement::new("boss_the_collatz_titan","Titan Felled",         "Defeat The Collatz Titan",                                 Epic),
        Achievement::new("boss_the_ouroboros",    "Ouroboros Severed",    "Defeat The Ouroboros",                                     Epic),
        Achievement::new("boss_the_algorithm_reborn","Algorithm Debugged","Defeat The Algorithm Reborn",                              Legendary),

        // OMEGA tier
        Achievement::new("omega_long_run",    "OMEGA: The Long Run",  "Reach Floor 500 in Infinite mode",                             Omega),
        Achievement::new("omega_boss_rush",   "OMEGA: Boss Rush",     "Defeat all 12 bosses in a single Infinite run",                Omega),
        Achievement::new("omega_the_algorithm","OMEGA: The Algorithm","Defeat The Algorithm Reborn with negative stats, CHAOS diff, corruption 400+", Omega),

        // ── Class mastery ─────────────────────────────────────────────────────
        Achievement::new("mage_first",        "Arcane Initiate",      "Complete a run as Mage",                                       Common),
        Achievement::new("berserker_first",   "Blood Rage",           "Complete a run as Berserker",                                  Common),
        Achievement::new("ranger_first",      "Distant Shot",         "Complete a run as Ranger",                                     Common),
        Achievement::new("thief_first",       "Five-Finger Discount", "Complete a run as Thief",                                      Common),
        Achievement::new("necro_first",       "Undying",              "Complete a run as Necromancer",                                Common),
        Achievement::new("alchemist_first",   "Volatile Mixtures",    "Complete a run as Alchemist",                                  Common),
        Achievement::new("paladin_first",     "Holy Chaos",           "Complete a run as Paladin",                                    Common),
        Achievement::new("voidwalker_first",  "Into the Void",        "Complete a run as VoidWalker",                                 Common),
        Achievement::new("warlord_first",     "Warcry",               "Complete a run as Warlord",                                    Common),
        Achievement::new("trickster_first",   "Smoke and Mirrors",    "Complete a run as Trickster",                                  Common),
        Achievement::new("runesmith_first",   "Etched in Chaos",      "Complete a run as Runesmith",                                  Common),
        Achievement::new("chrono_first",      "Temporal Drift",       "Complete a run as Chronomancer",                               Common),
        Achievement::new("all_classes",       "Class Dismissed",      "Win at least one run with every class",                        Legendary),
        Achievement::new("mage_chaos",        "Pure Chaos",           "Win a run as Mage on CHAOS difficulty",                        Epic),
        Achievement::new("berserker_floor100","Unhinged",             "Reach Floor 100 as Berserker",                                 Rare),

        // ── Combat depth ──────────────────────────────────────────────────────
        Achievement::new("no_flee",           "Stand and Fight",      "Complete a run without fleeing once",                          Uncommon),
        Achievement::new("pacifist",          "Diplomacy Failed",     "Flee from 50 encounters in a single run",                      Rare),
        Achievement::new("kill_spree_5",      "On a Roll",            "Kill 5 enemies in a row without taking damage",                Common),
        Achievement::new("kill_spree_20",     "Unstoppable",          "Kill 20 enemies in a row without taking damage",               Rare),
        Achievement::new("combo_crits",       "Chaos Cascade",        "Land 3 critical hits in a row",                                Uncommon),
        Achievement::new("survive_1hp",       "Barely Made It",       "Survive a hit that would have reduced you to 0 HP",            Uncommon),
        Achievement::new("one_hp_win",        "One in a Million",     "Defeat a boss with exactly 1 HP remaining",                    Epic),
        Achievement::new("taunt_master",      "Aggro King",           "Use Taunt 100 times across all runs",                          Uncommon),
        Achievement::new("heavy_carry",       "Heavy Lifter",         "Deal 500+ damage with a single Heavy Attack",                  Rare),
        Achievement::new("defend_100",        "Iron Turtle",          "Block 10,000 total damage across all runs",                    Rare),
        Achievement::new("max_kills_run",     "Extinction Event",     "Kill 2,000 enemies in a single Infinite run",                  Legendary),
        Achievement::new("nemesis_kill",      "Nemesis Slain",        "Kill your own Nemesis",                                        Rare),
        Achievement::new("nemesis_kill_3",    "The Cycle Ends",       "Kill 3 different Nemesis enemies across all runs",             Epic),

        // ── Spells & Magic ────────────────────────────────────────────────────
        Achievement::new("first_spell",       "Spellcaster",          "Cast your first spell",                                        Common),
        Achievement::new("spells_100",        "Apprentice Mage",      "Cast 100 spells across all runs",                              Uncommon),
        Achievement::new("spells_1000",       "Archmage",             "Cast 1,000 spells across all runs",                            Rare),
        Achievement::new("backfire_survivor", "Backfire Proof",       "Survive a catastrophic spell backfire",                        Uncommon),
        Achievement::new("backfire_10",       "The Price of Power",   "Suffer 10 spell backfires in one run",                         Rare),
        Achievement::new("full_mana_always",  "Mana Battery",         "End 10 consecutive fights at full mana",                       Uncommon),
        Achievement::new("mana_zero_kill",    "Last Drop",            "Kill an enemy with exactly 1 mana remaining",                  Rare),
        Achievement::new("spell_overkill",    "Arcane Explosion",     "Kill an enemy with a spell dealing 5x their max HP",           Epic),

        // ── Items & Economy ───────────────────────────────────────────────────
        Achievement::new("items_50_run",      "Pack Rat",             "Hold 50 items across a single run",                            Uncommon),
        Achievement::new("sell_all",          "Liquidation Sale",     "Sell everything in your inventory at a shop",                  Uncommon),
        Achievement::new("gold_zero",         "Broke",                "Spend your last gold coin",                                    Common),
        Achievement::new("gold_50k",          "Mogul",                "Accumulate 50,000 gold in one run",                            Epic),
        Achievement::new("charged_item_use",  "Charged Up",           "Use an Imbued (charged) item",                                 Common),
        Achievement::new("shatter_epic",      "Entropy Transfer",     "Shatter an Epic or higher item",                               Rare),
        Achievement::new("full_sockets",      "Socket Maximalist",    "Have an item with 6 filled gem sockets",                       Rare),
        Achievement::new("divine_item",       "Divine Intervention",  "Obtain a Divine rarity item",                                  Epic),
        Achievement::new("artifact_item",     "Beyond Rarity",        "Obtain an Artifact rarity item",                               Legendary),

        // ── Crafting depth ────────────────────────────────────────────────────
        Achievement::new("craft_10",          "Tinkerer",             "Perform 10 crafting operations",                               Common),
        Achievement::new("craft_100",         "Craftsman",            "Perform 100 crafting operations",                              Uncommon),
        Achievement::new("craft_500",         "Grandmaster",          "Perform 500 crafting operations across all runs",              Rare),
        Achievement::new("shatter_first",     "Scattershot",          "Shatter your first item",                                      Common),
        Achievement::new("imbue_first",       "Imbued",               "Imbue your first item with charges",                           Common),
        Achievement::new("engine_lock_3",     "Locked In",            "Apply 3 Engine Locks to one item",                             Rare),
        Achievement::new("reforge_legendary", "Chaos Perfectionist",  "Reforge a Legendary item",                                     Rare),
        Achievement::new("augment_to_max",    "Modifier Hoarder",     "Augment an item to 6 stat modifiers",                         Rare),

        // ── Exploration ───────────────────────────────────────────────────────
        Achievement::new("all_room_types",    "Room Service",         "Enter every room type in a single run",                        Uncommon),
        Achievement::new("chaos_rift_3",      "Rift Walker",          "Find 3 Chaos Rifts in one run",                                Uncommon),
        Achievement::new("clear_100_rooms",   "Dungeon Crawler",      "Clear 100 rooms in a single run",                              Uncommon),
        Achievement::new("clear_1000_rooms",  "Delver of Depths",     "Clear 1,000 rooms across all runs",                            Rare),
        Achievement::new("floor_25_no_gold",  "Ascetic",              "Reach Floor 25 without spending any gold",                     Rare),
        Achievement::new("no_items_floor50",  "Bare Hands",           "Reach Floor 50 with an empty inventory",                       Epic),
        Achievement::new("gauntlet_flawless", "Gauntlet Perfection",  "Complete a 3-fight Boss Gauntlet without taking damage",        Epic),
        Achievement::new("portal_chain",      "Portal Hopper",        "Use 3 portals in a row (consecutive rooms)",                   Uncommon),

        // ── Floor milestones ──────────────────────────────────────────────────
        Achievement::new("floor_15",          "Getting Serious",      "Reach Floor 15",                                               Common),
        Achievement::new("floor_30",          "Veteran Delver",       "Reach Floor 30",                                               Uncommon),
        Achievement::new("floor_150",         "Deep Dive",            "Reach Floor 150",                                              Rare),
        Achievement::new("floor_250",         "The Abyss Stares Back","Reach Floor 250",                                              Epic),
        Achievement::new("floor_300",         "Unstoppable Force",    "Reach Floor 300",                                              Epic),

        // ── Chaos Engine stats ────────────────────────────────────────────────
        Achievement::new("chain_10",          "Deep Pipeline",        "Have a chaos roll with a chain depth of 10+",                  Rare),
        Achievement::new("all_positive_chain","All Green",            "Have every step in a chain be positive",                       Uncommon),
        Achievement::new("all_negative_chain","All Red",              "Have every step in a chain be negative",                       Uncommon),
        Achievement::new("perfect_zero",      "Null Result",          "Get a chaos final value within 0.001 of 0.000",                Epic),
        Achievement::new("max_value",         "Overflow",             "Get a chaos final value of 1.000",                             Rare),
        Achievement::new("min_value",         "Underflow",            "Get a chaos final value of -1.000",                            Rare),
        Achievement::new("pi_roll",           "Pi in the Sky",        "Have a chaos roll output approximately 3.14159",               Epic),

        // ── Difficulty ────────────────────────────────────────────────────────
        Achievement::new("easy_clear",        "Warming Up",           "Win a run on Easy difficulty",                                 Common),
        Achievement::new("normal_clear",      "By the Book",          "Win a run on Normal difficulty",                               Common),
        Achievement::new("brutal_clear",      "Brutal Efficiency",    "Win a run on Brutal difficulty",                               Rare),
        Achievement::new("chaos_clear",       "Chaos Conquered",      "Win a run on CHAOS difficulty",                                Epic),
        Achievement::new("chaos_floor50",     "Madness Maintained",   "Reach Floor 50 on CHAOS difficulty",                           Rare),
        Achievement::new("chaos_all_classes", "Perfect Madness",      "Win on CHAOS difficulty with all 12 classes",                  Omega),

        // ── Daily seed ────────────────────────────────────────────────────────
        Achievement::new("daily_first",       "Daily Player",         "Complete your first daily seed run",                           Common),
        Achievement::new("daily_win",         "Daily Victor",         "Win a daily seed run",                                         Rare),
        Achievement::new("daily_top3",        "Podium",               "Finish in the top 3 on the daily leaderboard",                 Epic),
        Achievement::new("daily_rank1",       "Number One",           "Finish #1 on the daily leaderboard",                          Legendary),
        Achievement::new("daily_30",          "Dedicated",            "Play 30 daily seed runs",                                      Uncommon),

        // ── Mod / config ──────────────────────────────────────────────────────
        Achievement::new("config_loaded",     "Modder",               "Load and play with a custom chaos_config.toml",                Rare),
        Achievement::new("config_gold_bonus", "Cheat Mode?",          "Play with starting_gold_bonus > 0 in config",                  Common),
        Achievement::new("config_hard_mode",  "Self Imposed Misery",  "Play with difficulty_modifier >= 2.0 in config",               Rare),

        // ── Meta / misc ───────────────────────────────────────────────────────
        Achievement::new("runs_25",           "Persistent",           "Complete 25 runs",                                             Common),
        Achievement::new("runs_50",           "Seasoned",             "Complete 50 runs",                                             Uncommon),
        Achievement::new("runs_200",          "Obsessed",             "Complete 200 runs",                                            Epic),
        Achievement::new("runs_500",          "Cannot Stop",          "Complete 500 runs",                                            Legendary),
        Achievement::new("score_1m",          "Score Millionaire",    "Achieve a score of 1,000,000 in one run",                      Rare),
        Achievement::new("score_10m",         "High Score Hunter",    "Achieve a score of 10,000,000 in one run",                     Epic),
        Achievement::new("all_boons",         "Boon Collector",       "Use every boon at least once across all runs",                 Rare),
        Achievement::new("level_50",          "Power Spike",          "Reach level 50",                                               Uncommon),
        Achievement::new("level_75",          "Ascended",             "Reach level 75",                                               Rare),
        Achievement::new("passive_50",        "Node Farmer",          "Allocate 50 passive points in one run",                        Rare),
        Achievement::new("passive_100",       "Tree Hugger",          "Allocate 100 passive points in one run",                       Epic),
        Achievement::new("multi_nemesis",     "They Never Rest",      "Have 5+ Nemesis entries across all characters",                Rare),
        Achievement::new("story_perfect",     "Story Complete",       "Win Story mode with 0 deaths in the run",                      Epic),
        Achievement::new("chaos_engine_viz",  "I See the Pattern",    "Open the Chaos Engine Visualizer",                             Common),
        Achievement::new("item_filter_used",  "Organised Chaos",      "Use the item filter in the crafting bench",                    Common),
    ]
}
