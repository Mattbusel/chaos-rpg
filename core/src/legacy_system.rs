//! Cross-run persistence: lifetime stats, achievements, unlocks, and the Graveyard.
//!
//! Saved to ~/.chaos_rpg/legacy.json. Never grants combat power — only cosmetics,
//! information, and quality-of-life unlocks.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// ── Achievement IDs ───────────────────────────────────────────────────────────

/// A cross-run achievement; see [`AchievementId::name`] and [`AchievementId::description`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    // Negative-run achievements
    /// Finish a run at BELOW AVERAGE power tier.
    NotGreat,
    /// Survive 10 floors at CURSED tier or below.
    TechnicallyAlive,
    /// Enter the Defiance state.
    Defiant,
    /// Spend 500 Spite in a single run.
    Spiteful,
    /// Trigger the Cosmic Joke event.
    TheJokesOnMe,
    /// Generate an in-game academic paper.
    PublishedFailure,
    /// Reach THE VOID power tier.
    RockBottom,
    /// Reach a Misery Index of 1,000,000.
    NegativeGod,
    /// Die to a Singularity Moth on floor 1.
    DieToAMoth,
    /// Die to your own spell backfire 5 times.
    SelfInflicted,
    /// Die to exactly 1 damage.
    OneHitWonder,
    /// Die from a headshot while the rest of the body is at full HP.
    Headshot,
    /// Die to the same enemy type 3 times.
    NemesisOrigin,
    /// Have all 7 stats negative at the same time.
    MathIsHard,
    /// Have an Undecidable item vanish 3 times in one run.
    TrustIssues,
    /// Visit 10 shops without buying anything.
    WindowShopper,
    /// Start at ABYSSAL tier or below and finish at CHAMPION or above.
    TheComeback,
    /// Have a stat total of exactly 0.
    PerfectlyBalanced,
    /// Die to overflow damage.
    OverflowVictim,
    /// Die to an enemy with lower stats than you.
    CosmicIrony,
    // Positive-run achievements
    /// Reach BEYOND MATH power tier.
    BeyondMath,
    /// Reach AXIOM power tier.
    AxiomReached,
    /// Reach THEOREM power tier.
    TheoremReached,
    /// Reach ALEPH-0 power tier.
    AlephZeroReached,
    /// Reach OMEGA power tier.
    OmegaReached,
    /// Reach floor 100 in Infinite mode.
    Floor100,
    /// Deal 1,000,000 damage in a single run.
    MillionDamage,
    /// Complete Story mode with no combat kills.
    Pacifist,
    /// Kill a boss in a single hit.
    OnePunch,
    /// Complete Story mode in under 50 actions.
    SpeedDemon,
    /// Hold 50 or more items at once.
    Hoarder,
    /// Learn 100 or more spells in a single run.
    Polyglot,
    /// Allocate 400 or more passive tree nodes in a single run.
    TreeHugger,
    /// Play 10 different seeded runs.
    SeedSharer,
}

impl AchievementId {
    /// Display name shown when the achievement unlocks.
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

    /// One-line description of what earns the achievement.
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

/// A cosmetic or information unlock earned across runs (never combat power).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnlockId {
    /// Lets you pick a colour for your character name.
    NameColorOption,
    /// Shows the Misery Index at all times.
    MiseryIndexAlwaysVisible,
    /// A Defiance badge next to your name.
    DefBadge,
    /// Draws chaos engine traces in red.
    RedEngineTraces,
    /// Enables extra self-aware flavour text.
    MetaAwareFlavour,
    /// Shows a mock DOI next to your scoreboard entries.
    DoiOnScoreboard,
    /// Grants the title "The Void".
    TheVoidTitle,
    /// Shows Misery as a regular stat on the character sheet.
    MiseryAsVisibleStat,
    /// A Singularity Moth badge.
    SingularityMothBadge,
    /// Grants the title "At Great Personal Cost".
    AtGreatPersonalCostTitle,
    /// Grants the title "Fragile".
    FragileTitle,
    /// Grants the title "Glass Skull".
    GlassSkullTitle,
    /// Grants the title "Mathematical Impossibility".
    MathematicalImpossibilityTitle,
    /// Shows a stability rating on items.
    ItemStabilityRating,
    /// Unlocks unique merchant dialogue.
    MerchantUniqueDialogue,
    /// Grants the title "Underdog".
    UnderdogTitle,
    /// Grants the title "Zero".
    ZeroTitle,
    /// Tracks overflow damage events during a run.
    OverflowTracker,
    /// Shows your stats next to the enemy's in combat.
    EnemyStatComparison,
    /// Adds an infinity decoration to the UI.
    InfinityDecoration,
    /// Unlocks the dialogue where the game formally concedes.
    GameFormallyConcedesDialogue,
    /// Grants the title "Centurion".
    CenturionTitle,
    /// Adds particle effects to damage numbers.
    DamageParticleEffects,
    /// Shows a running count of your actions.
    ActionCounter,
    /// Shows the expanded inventory view.
    ExpandedInventoryDisplay,
    /// Shows badges for spell schools.
    SpellSchoolBadges,
    /// Makes allocated passive tree nodes glow.
    PassiveTreeGlow,
    /// Keeps a log of seeds you have played.
    SeedHistoryLog,
}

// ── Per-engine lifetime stats ─────────────────────────────────────────────────

/// Lifetime statistics for one chaos engine across all runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineLifetimeStats {
    /// Index of the engine in the chaos pipeline (0 to 9).
    pub engine_id: u8,
    /// How many rolls this engine has taken part in.
    pub total_uses: u64,
    /// Sum of every output, used for the average.
    pub total_output: f64,
    /// Highest output ever produced.
    pub best_output: f64,
    /// Lowest output ever produced.
    pub worst_output: f64,
    /// Rolls where this engine was in the chain of a killing blow you dealt.
    pub times_in_killing_blow: u64,
    /// Rolls where this engine was in the chain of the blow that killed you.
    pub times_in_death_blow: u64,
}

impl EngineLifetimeStats {
    /// Average output per use, or 0 if never used.
    pub fn avg(&self) -> f64 {
        if self.total_uses == 0 { 0.0 } else { self.total_output / self.total_uses as f64 }
    }
}

// ── Graveyard entry ───────────────────────────────────────────────────────────

/// One dead character in the Graveyard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraveyardEntry {
    /// Character name.
    pub name: String,
    /// Character class name.
    pub class: String,
    /// Character level at death.
    pub level: u32,
    /// Floor the character died on.
    pub floor: u32,
    /// Power tier name at death.
    pub power_tier: String,
    /// Misery Index at death.
    pub misery_index: f64,
    /// What killed the character.
    pub cause_of_death: String,
    /// Enemies killed during the run.
    pub kills: u32,
    /// Final score.
    pub score: u64,
    /// Date of death as text.
    pub date: String,
    /// Generated epitaph shown on the tombstone.
    pub epitaph: String,
}

impl GraveyardEntry {
    /// Pick an epitaph for a dead character from how the run went (backfire death, all stats negative, misery, class and so on).
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
            "Necromancer"=> "They came back from worse than this before. They did not come back from this.".to_string(),
            "Paladin"    => "The regen wasn't enough. Nothing personal — just statistics.".to_string(),
            _            => format!("A {class} of floor {floor}. The chaos engine is indifferent but notes the record."),
        }
    }

    /// ASCII-art tombstone with name, level, floor, score, epitaph and killer.
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

/// What you know about one enemy type across all runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BestiaryEntry {
    /// Enemy type name.
    pub enemy_name: String,
    /// Times you have met this enemy.
    pub encounters: u64,
    /// Times you killed it.
    pub kills_by_player: u64,
    /// Times it killed you.
    pub times_killed_player: u64,
    /// Total damage it has dealt to you.
    pub total_damage_taken_from: i64,
    /// Total damage you have dealt to it.
    pub total_damage_dealt_to: i64,
}

// ── Legacy data ───────────────────────────────────────────────────────────────

/// Everything kept between runs: lifetime totals, records, achievements, unlocks, bestiary and graveyard.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LegacyData {
    /// Runs played.
    pub total_runs: u64,
    /// Enemies killed across all runs.
    pub total_kills: u64,
    /// Floors cleared across all runs.
    pub total_floors: u64,
    /// Damage dealt across all runs.
    pub total_damage_dealt: i64,
    /// Damage taken across all runs.
    pub total_damage_taken: i64,
    /// Gold earned across all runs.
    pub total_gold: i64,
    /// Misery Index accumulated across all runs.
    pub total_misery: f64,
    /// Spite spent across all runs.
    pub total_spite_spent: f64,
    /// Chaos engine rolls across all runs.
    pub total_engine_rolls: u64,
    /// Deaths caused by your own spell backfire.
    pub total_backfire_deaths: u32,
    /// Seeded runs played.
    pub total_seeded_runs: u32,
    /// Biggest single hit ever dealt.
    pub highest_single_hit: i64,
    /// Deepest floor ever reached.
    pub highest_floor: u32,
    /// Highest power tier ever reached.
    pub highest_power_tier: String,
    /// Lowest power tier ever reached.
    pub lowest_power_tier: String,
    /// Highest Misery Index in a single run.
    pub highest_misery_single_run: f64,
    /// Most floors cleared in one run.
    pub longest_run_floors: u32,
    /// Fewest floors cleared in one run.
    pub shortest_run_floors: u32,
    /// Total play time in seconds.
    pub total_play_time_seconds: u64,
    /// Per-engine lifetime statistics.
    pub per_engine_lifetime: Vec<EngineLifetimeStats>,
    /// Achievements earned.
    pub achievements: HashSet<AchievementId>,
    /// Unlocks earned.
    pub unlocks: HashSet<UnlockId>,
    /// Bestiary entries keyed by enemy name.
    pub enemy_bestiary: HashMap<String, BestiaryEntry>,
    /// Dead characters, newest last.
    pub character_graveyard: Vec<GraveyardEntry>,
    /// Backfire deaths counted toward the Self Inflicted achievement.
    pub backfire_death_count: u32,  // for SelfInflicted achievement
    /// Seeds of seeded runs played, for the Seed Sharer achievement.
    pub seeded_seeds_played: Vec<u64>,
    /// Shops visited without buying, for the Window Shopper achievement.
    pub window_shopping_runs: u32,  // shops visited without buying
    /// Runs in a row that finished at a negative power tier.
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
    /// Load legacy data from `~/.chaos_rpg/chaos_rpg_legacy.json`, or start empty if missing or unreadable.
    pub fn load() -> Self {
        let path = legacy_path();
        let Ok(bytes) = std::fs::read(&path) else { return Self::default(); };
        serde_json::from_slice(&bytes).unwrap_or_default()
    }

    /// Save legacy data to `~/.chaos_rpg/chaos_rpg_legacy.json`, ignoring write errors.
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
