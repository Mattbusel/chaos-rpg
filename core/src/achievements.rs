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
}

impl Default for AchievementStore {
    fn default() -> Self {
        Self {
            achievements: all_achievements(),
            pending_banners: Vec::new(),
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
        if r.total_runs >= 100 { self.unlock("veteran"); }
        if r.total_deaths >= 1000 { self.unlock("thousand_deaths"); }

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
            "modded_config"    => { self.unlock("modder"); }
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
    ]
}
