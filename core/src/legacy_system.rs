//! Cross-run persistence: lifetime stats, achievements, unlocks, and the Graveyard.
//!
//! Saved to ~/.chaos_rpg/legacy.json. Never grants combat power — only cosmetics,
//! information, and quality-of-life unlocks.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// ── Achievement IDs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    // Negative-run achievements
    NotGreat,
    TechnicallyAlive,
    Defiant,
    Spiteful,
    TheJokesOnMe,
    PublishedFailure,
    RockBottom,
    NegativeGod,
    DieToAMoth,
    SelfInflicted,
    OneHitWonder,
    Headshot,
    NemesisOrigin,
    MathIsHard,
    TrustIssues,
    WindowShopper,
    TheComeback,
    PerfectlyBalanced,
    OverflowVictim,
    CosmicIrony,
    // Positive-run achievements
    BeyondMath,
    AxiomReached,
    TheoremReached,
    AlephZeroReached,
    OmegaReached,
    Floor100,
    MillionDamage,
    Pacifist,
    OnePunch,
    SpeedDemon,
    Hoarder,
    Polyglot,
    TreeHugger,
    SeedSharer,
}

impl AchievementId {
    pub fn name(self) -> &'static str {
        match self {
            AchievementId::NotGreat         => "Not Great",
            AchievementId::TechnicallyAlive => "Technically Alive",
            AchievementId::Defiant          => "Defiant",
            AchievementId::Spiteful         => "Spiteful",
            AchievementId::TheJokesOnMe     => "The Joke's On Me",
            AchievementId::PublishedFailure => "Published Failure",
            AchievementId::RockBottom       => "Rock Bottom",
            AchievementId::NegativeGod      => "Negative God",
            AchievementId::DieToAMoth       => "Die to a Moth",
            AchievementId::SelfInflicted    => "Self-Inflicted",
            AchievementId::OneHitWonder     => "One Hit Wonder",
            AchievementId::Headshot         => "Headshot",
            AchievementId::NemesisOrigin    => "Nemesis: Origin",
            AchievementId::MathIsHard       => "Math Is Hard",
            AchievementId::TrustIssues      => "Trust Issues",
            AchievementId::WindowShopper    => "Window Shopper",
            AchievementId::TheComeback      => "The Comeback",
            AchievementId::PerfectlyBalanced=> "Perfectly Balanced",
            AchievementId::OverflowVictim   => "Overflow Victim",
            AchievementId::CosmicIrony      => "Cosmic Irony",
            AchievementId::BeyondMath       => "Beyond Math",
            AchievementId::AxiomReached     => "Axiom",
            AchievementId::TheoremReached   => "Theorem",
            AchievementId::AlephZeroReached => "Aleph-0",
            AchievementId::OmegaReached     => "Omega",
            AchievementId::Floor100         => "Floor 100",
            AchievementId::MillionDamage    => "Million Damage",
            AchievementId::Pacifist         => "Pacifist",
            AchievementId::OnePunch         => "One Punch",
            AchievementId::SpeedDemon       => "Speed Demon",
            AchievementId::Hoarder          => "Hoarder",
            AchievementId::Polyglot         => "Polyglot",
            AchievementId::TreeHugger       => "Tree Hugger",
            AchievementId::SeedSharer       => "Seed Sharer",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            AchievementId::NotGreat          => "Finish a run at BELOW AVERAGE tier",
            AchievementId::TechnicallyAlive  => "Survive 10 floors at CURSED or below",
            AchievementId::Defiant           => "Enter Defiance state",
            AchievementId::Spiteful          => "Spend 500 Spite in a single run",
            AchievementId::TheJokesOnMe      => "Trigger Cosmic Joke",
            AchievementId::PublishedFailure  => "Generate an in-game academic paper",
            AchievementId::RockBottom        => "Reach THE VOID tier",
            AchievementId::NegativeGod       => "Reach 1,000,000 Misery Index",
            AchievementId::DieToAMoth        => "Die to a Singularity Moth on floor 1",
            AchievementId::SelfInflicted     => "Die to your own spell backfire 5 times",
            AchievementId::OneHitWonder      => "Die to exactly 1 damage",
            AchievementId::Headshot          => "Die from a headshot with full body HP elsewhere",
            AchievementId::NemesisOrigin     => "Die to the same enemy type 3 times",
            AchievementId::MathIsHard        => "Have all 7 stats negative simultaneously",
            AchievementId::TrustIssues       => "Have an Undecidable item vanish 3 times in one run",
            AchievementId::WindowShopper     => "Visit 10 shops without buying anything",
            AchievementId::TheComeback       => "Start ABYSSAL or below, finish CHAMPION or above",
            AchievementId::PerfectlyBalanced => "Have a stat total of exactly 0",
            AchievementId::OverflowVictim    => "Die to overflow damage",
            AchievementId::CosmicIrony       => "Die to an enemy with lower stats than you",
            AchievementId::BeyondMath        => "Reach BEYOND MATH tier",
            AchievementId::AxiomReached      => "Reach AXIOM tier",
            AchievementId::TheoremReached    => "Reach THEOREM tier",
            AchievementId::AlephZeroReached  => "Reach ALEPH-0 tier",
            AchievementId::OmegaReached      => "Reach ΩMEGA tier",
            AchievementId::Floor100          => "Reach floor 100 in infinite mode",
            AchievementId::MillionDamage     => "Deal 1,000,000 damage in a single run",
            AchievementId::Pacifist          => "Complete story mode with 0 combat kills",
            AchievementId::OnePunch          => "Kill a boss in a single hit",
            AchievementId::SpeedDemon        => "Complete story mode in under 50 actions",
            AchievementId::Hoarder           => "Hold 50+ items simultaneously",
            AchievementId::Polyglot          => "Learn 100+ spells in a single run",
            AchievementId::TreeHugger        => "Allocate 400+ passive nodes in a single run",
            AchievementId::SeedSharer        => "Play 10 different seeded runs",
        }
    }
}

// ── Unlocks ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnlockId {
    NameColorOption,
    MiseryIndexAlwaysVisible,
    DefBadge,
    RedEngineTraces,
    MetaAwareFlavour,
    DoiOnScoreboard,
    TheVoidTitle,
    MiseryAsVisibleStat,
    SingularityMothBadge,
    AtGreatPersonalCostTitle,
    FragileTitle,
    GlassSkullTitle,
    MathematicalImpossibilityTitle,
    ItemStabilityRating,
    MerchantUniqueDialogue,
    UnderdogTitle,
    ZeroTitle,
    OverflowTracker,
    EnemyStatComparison,
    InfinityDecoration,
    GameFormallyConcedesDialogue,
    CenturionTitle,
    DamageParticleEffects,
    ActionCounter,
    ExpandedInventoryDisplay,
    SpellSchoolBadges,
    PassiveTreeGlow,
    SeedHistoryLog,
}

// ── Per-engine lifetime stats ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineLifetimeStats {
    pub engine_id: u8,
    pub total_uses: u64,
    pub total_output: f64,
    pub best_output: f64,
    pub worst_output: f64,
    pub times_in_killing_blow: u64,
    pub times_in_death_blow: u64,
}

impl EngineLifetimeStats {
    pub fn avg(&self) -> f64 {
        if self.total_uses == 0 { 0.0 } else { self.total_output / self.total_uses as f64 }
    }
}

// ── Graveyard entry ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraveyardEntry {
    pub name: String,
    pub class: String,
    pub level: u32,
    pub floor: u32,
    pub power_tier: String,
    pub misery_index: f64,
    pub cause_of_death: String,
    pub kills: u32,
    pub score: u64,
    pub date: String,
    pub epitaph: String,
}

impl GraveyardEntry {
    pub fn generate_epitaph(
        class: &str,
        floor: u32,
        kills: u32,
        damage_dealt: i64,
        misery_index: f64,
        spells_cast: u32,
        all_stats_negative: bool,
        died_to_backfire: bool,
        power_tier: &str,
    ) -> String {
        if floor == 1 && kills == 0 {
            return "The shortest poem ever written.".into();
        }
        if died_to_backfire {
            return "They wielded power they couldn't control. The cost was personal.".into();
        }
        if all_stats_negative {
            return "Born in deficit. Lived in deficit. Died in surplus — of misery.".into();
        }
        if misery_index >= 50_000.0 {
            return format!(
                "They suffered {:.0} units of misery. More than the math intended. \
                 The algorithms send condolences.", misery_index
            );
        }
        if power_tier == "ΩMEGA" {
            return "They broke everything. We're still cleaning up.".into();
        }
        if spells_cast == 0 {
            return "They solved every problem with violence. It worked until it didn't.".into();
        }
        if damage_dealt > 0 && kills < 3 {
            return format!(
                "Quality over quantity. {} points of damage. {kills} kills. \
                 Each swing was a mathematical event.", damage_dealt
            );
        }
        // class-specific fallbacks
        match class {
            "Mage"       => format!("Floor {floor}. Level unknown to the prime numbers. Remembered by the mana pool."),
            "Berserker"  => format!("Rage carried them to floor {floor}. Math brought them back down."),
            "Necromancer"=> format!("They came back from worse than this before. They did not come back from this."),
            "Paladin"    => format!("The regen wasn't enough. Nothing personal — just statistics."),
            _            => format!("A {class} of floor {floor}. The chaos engine is indifferent but notes the record."),
        }
    }

    pub fn render_tombstone(&self) -> String {
        format!(
            "┌─────────────────────────┐\n\
             │       R.I.P.            │\n\
             │  {:25}│\n\
             │  Lv.{:<3} {:17}│\n\
             │  Floor {:3} — {:8} pts│\n\
             │                         │\n\
             │  {}│\n\
             │                         │\n\
             │  Killed by: {:12} │\n\
             └─────────────────────────┘",
            self.name,
            self.level, self.class,
            self.floor, self.score,
            Self::wrap_epitaph_short(&self.epitaph),
            Self::truncate(&self.cause_of_death, 13),
        )
    }

    fn wrap_epitaph_short(s: &str) -> String {
        if s.len() <= 25 { format!("{:<25}", s) } else { format!("{:.22}...", s) }
    }
    fn truncate(s: &str, n: usize) -> &str {
        &s[..s.len().min(n)]
    }
}

// ── Bestiary entry ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BestiaryEntry {
    pub enemy_name: String,
    pub encounters: u64,
    pub kills_by_player: u64,
    pub times_killed_player: u64,
    pub total_damage_taken_from: i64,
    pub total_damage_dealt_to: i64,
}

// ── Legacy data ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegacyData {
    pub total_runs: u64,
    pub total_kills: u64,
    pub total_floors: u64,
    pub total_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub total_gold: i64,
    pub total_misery: f64,
    pub total_spite_spent: f64,
    pub total_engine_rolls: u64,
    pub total_backfire_deaths: u32,
    pub total_seeded_runs: u32,
    pub highest_single_hit: i64,
    pub highest_floor: u32,
    pub highest_power_tier: String,
    pub lowest_power_tier: String,
    pub highest_misery_single_run: f64,
    pub longest_run_floors: u32,
    pub shortest_run_floors: u32,
    pub total_play_time_seconds: u64,
    pub per_engine_lifetime: Vec<EngineLifetimeStats>,
    pub achievements: HashSet<AchievementId>,
    pub unlocks: HashSet<UnlockId>,
    pub enemy_bestiary: HashMap<String, BestiaryEntry>,
    pub character_graveyard: Vec<GraveyardEntry>,
    pub backfire_death_count: u32,  // for SelfInflicted achievement
    pub seeded_seeds_played: Vec<u64>,
    pub window_shopping_runs: u32,  // shops visited without buying
    pub consecutive_negative_runs: u32,
}

const LEGACY_FILE: &str = "chaos_rpg_legacy.json";

fn legacy_path() -> PathBuf {
    if let Some(home) = dirs_home() {
        home.join(".chaos_rpg").join(LEGACY_FILE)
    } else {
        PathBuf::from(LEGACY_FILE)
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .ok().map(PathBuf::from)
}

impl LegacyData {
    pub fn load() -> Self {
        let path = legacy_path();
        let Ok(bytes) = std::fs::read(&path) else { return Self::default(); };
        serde_json::from_slice(&bytes).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = legacy_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Merge a completed run into legacy data. Returns newly earned achievements.
    pub fn record_run(
        &mut self,
        entry: GraveyardEntry,
        damage_dealt: i64,
        damage_taken: i64,
        gold: i64,
        misery: f64,
        spite_spent: f64,
        engine_rolls: u64,
        backfire_died: bool,
        seeded: bool,
        seed: u64,
        power_tier_name: &str,
    ) -> Vec<AchievementId> {
        self.total_runs += 1;
        self.total_kills += entry.kills as u64;
        self.total_floors += entry.floor as u64;
        self.total_damage_dealt += damage_dealt;
        self.total_damage_taken += damage_taken;
        self.total_gold += gold;
        self.total_misery += misery;
        self.total_spite_spent += spite_spent;
        self.total_engine_rolls += engine_rolls;
        if backfire_died {
            self.total_backfire_deaths += 1;
            self.backfire_death_count += 1;
        }
        if seeded && !self.seeded_seeds_played.contains(&seed) {
            self.seeded_seeds_played.push(seed);
            self.total_seeded_runs += 1;
        }
        if entry.floor > self.highest_floor { self.highest_floor = entry.floor; }
        if entry.floor > 0 && (self.shortest_run_floors == 0 || entry.floor < self.shortest_run_floors) {
            self.shortest_run_floors = entry.floor;
        }
        if misery > self.highest_misery_single_run { self.highest_misery_single_run = misery; }

        // Update best/worst power tiers
        if self.highest_power_tier.is_empty() { self.highest_power_tier = power_tier_name.to_string(); }
        if self.lowest_power_tier.is_empty()  { self.lowest_power_tier  = power_tier_name.to_string(); }

        self.character_graveyard.push(entry);

        // Check achievements
        self.check_achievements()
    }

    fn check_achievements(&mut self) -> Vec<AchievementId> {
        let mut new_achievements = Vec::new();
        let candidates = [
            (AchievementId::SeedSharer,      self.total_seeded_runs >= 10),
            (AchievementId::SelfInflicted,   self.backfire_death_count >= 5),
        ];
        for (id, cond) in candidates {
            if cond && !self.achievements.contains(&id) {
                self.achievements.insert(id);
                new_achievements.push(id);
            }
        }
        new_achievements
    }

    /// Check single-run achievements against run data. Returns newly earned.
    pub fn check_run_achievements(
        &mut self,
        power_tier: &str,
        floor: u32,
        kills: u32,
        misery: f64,
        spite_spent: f64,
        in_defiance: bool,
        cosmic_joke: bool,
        paper_generated: bool,
        stat_total: i64,
        died_to_one_damage: bool,
        died_to_headshot: bool,
        died_to_overflow: bool,
        all_stats_negative: bool,
        comeback: bool,
    ) -> Vec<AchievementId> {
        let mut new_achievements = Vec::new();
        let mut check = |id: AchievementId, cond: bool| {
            if cond && !self.achievements.contains(&id) {
                self.achievements.insert(id);
                new_achievements.push(id);
            }
        };
        check(AchievementId::NotGreat,          power_tier == "BELOW AVERAGE");
        check(AchievementId::TechnicallyAlive,  floor >= 10 && ["CURSED","DAMNED","FORSAKEN","ABYSSAL","ANTI-CHAMPION","VOID-TOUCHED","MATHEMATICAL ERROR","NEGATIVE INFINITY","ANTI-AXIOM","PARADOX","DIVISION BY ZERO","NEGATIVE ALEPH","RUSSELL'S PARADOX","GODEL'S GHOST","ABSOLUTE ZERO","HEAT DEATH","THE VOID"].contains(&power_tier));
        check(AchievementId::Defiant,           in_defiance);
        check(AchievementId::Spiteful,          spite_spent >= 500.0);
        check(AchievementId::TheJokesOnMe,      cosmic_joke);
        check(AchievementId::PublishedFailure,  paper_generated);
        check(AchievementId::RockBottom,        power_tier == "THE VOID");
        check(AchievementId::NegativeGod,       misery >= 1_000_000.0);
        check(AchievementId::MathIsHard,        all_stats_negative);
        check(AchievementId::TheComeback,       comeback);
        check(AchievementId::PerfectlyBalanced, stat_total == 0);
        check(AchievementId::OverflowVictim,    died_to_overflow);
        check(AchievementId::OneHitWonder,      died_to_one_damage);
        check(AchievementId::Headshot,          died_to_headshot);
        check(AchievementId::BeyondMath,        power_tier == "BEYOND MATH");
        check(AchievementId::AxiomReached,      power_tier == "AXIOM");
        check(AchievementId::TheoremReached,    power_tier == "THEOREM");
        check(AchievementId::AlephZeroReached,  power_tier == "ALEPH-0");
        check(AchievementId::OmegaReached,      power_tier == "ΩMEGA");
        check(AchievementId::Floor100,          floor >= 100);
        new_achievements
    }

    /// Render Hall of Misery table from graveyard entries sorted by misery.
    pub fn hall_of_misery_display(&self) -> String {
        let mut entries: Vec<&GraveyardEntry> = self.character_graveyard.iter()
            .filter(|e| e.misery_index > 0.0)
            .collect();
        entries.sort_by(|a, b| b.misery_index.partial_cmp(&a.misery_index).unwrap());
        entries.truncate(10);

        let mut out = String::from(
            "╔═══════════════════════════════════════════════════════════════╗\n\
             ║            HALL OF MISERY — TOP SUFFERERS                    ║\n\
             ╠════╤═══════════════╤═══════════╤═════════╤═══════╤═══════════╣\n\
             ║  # │ Name          │ Class     │ Misery  │ Floor │ Score     ║\n\
             ╠════╪═══════════════╪═══════════╪═════════╪═══════╪═══════════╣\n"
        );
        for (i, e) in entries.iter().enumerate() {
            out.push_str(&format!(
                "║ {:2} │ {:13} │ {:9} │ {:>7.0} │ {:5} │ {:9} ║\n",
                i + 1,
                Self::trunc(&e.name, 13),
                Self::trunc(&e.class, 9),
                e.misery_index,
                e.floor,
                e.score,
            ));
        }
        if entries.is_empty() {
            out.push_str("║             No suffering recorded yet.                        ║\n");
        }
        out.push_str("╚════╧═══════════════╧═══════════╧═════════╧═══════╧═══════════╝");
        out
    }

    fn trunc(s: &str, n: usize) -> String {
        if s.len() <= n { format!("{:<width$}", s, width=n) }
        else { format!("{:.width$}", s, width=n.saturating_sub(1)) + "…" }
    }
}
