//! Misery Index, Spite resource, Defiance state, Cosmic Joke, and Underdog mechanics.
//!
//! For negative power-tier characters, this system transforms suffering into
//! narrative and mechanical depth. The worse you are, the funnier/more-interesting
//! the game becomes.

use serde::{Deserialize, Serialize};

// ── Misery sources ────────────────────────────────────────────────────────────

/// Every event that contributes to the Misery Index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiserySource {
    DamageTaken,        // +damage_amount
    SpellBackfire,      // +backfire_damage × 2
    Headshot,           // +50
    AttackMissed,       // +10
    EnemyDodged,        // +20
    SkillCheckFailed,   // +difficulty value
    FleeFailed,         // +100
    ItemVanished,       // +item stat magnitude
    EnemyPitiedYou,     // +200 (the pity itself is misery)
    ShopTooExpensive,   // +50
    StatDecreaseOnLevelUp, // +abs(decrease) × 10
    DeathRemainingEnemyHp, // +enemy remaining HP
}

// ── Milestones ────────────────────────────────────────────────────────────────

/// Milestones unlocked as Misery Index grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MiseryMilestone {
    ItGetsWorse,         // 100
    BelovedOfMurphy,     // 500
    ProfessionalVictim,  // 1_000
    UniversesPunchingBag,// 2_500
    Sisyphus,            // 5_000 — unlocks Spite
    StatisticalImpossibility, // 10_000 — unlocks Defiance
    ThePunchline,        // 25_000 — unlocks Cosmic Joke
    ActuallyImpressive,  // 50_000 — unlocks Transcendent Misery
    TheyWillWritePapers, // 100_000 — unlocks Published Failure
    TheMostWretched,     // 500_000 — Hall of Misery entry
    NegativeGod,         // 1_000_000
}

impl MiseryMilestone {
    pub fn threshold(self) -> f64 {
        match self {
            MiseryMilestone::ItGetsWorse            => 100.0,
            MiseryMilestone::BelovedOfMurphy         => 500.0,
            MiseryMilestone::ProfessionalVictim      => 1_000.0,
            MiseryMilestone::UniversesPunchingBag    => 2_500.0,
            MiseryMilestone::Sisyphus                => 5_000.0,
            MiseryMilestone::StatisticalImpossibility=> 10_000.0,
            MiseryMilestone::ThePunchline            => 25_000.0,
            MiseryMilestone::ActuallyImpressive      => 50_000.0,
            MiseryMilestone::TheyWillWritePapers     => 100_000.0,
            MiseryMilestone::TheMostWretched         => 500_000.0,
            MiseryMilestone::NegativeGod             => 1_000_000.0,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            MiseryMilestone::ItGetsWorse             => "It Gets Worse",
            MiseryMilestone::BelovedOfMurphy          => "Beloved of Murphy",
            MiseryMilestone::ProfessionalVictim       => "Professional Victim",
            MiseryMilestone::UniversesPunchingBag     => "The Universe's Punching Bag",
            MiseryMilestone::Sisyphus                 => "Sisyphus Would Quit",
            MiseryMilestone::StatisticalImpossibility => "Statistical Impossibility",
            MiseryMilestone::ThePunchline             => "The Punchline",
            MiseryMilestone::ActuallyImpressive       => "Actually Impressive",
            MiseryMilestone::TheyWillWritePapers      => "They Will Write Papers About You",
            MiseryMilestone::TheMostWretched          => "THE MOST WRETCHED",
            MiseryMilestone::NegativeGod              => "NEGATIVE GOD",
        }
    }

    pub fn flavor(self) -> &'static str {
        match self {
            MiseryMilestone::ItGetsWorse             => "The math has noticed you specifically.",
            MiseryMilestone::BelovedOfMurphy          => "Whatever can go wrong, has.",
            MiseryMilestone::ProfessionalVictim       => "+5% Underdog XP. You've earned something.",
            MiseryMilestone::UniversesPunchingBag     => "+10% Underdog XP. The universe respects the persistence, if not the results.",
            MiseryMilestone::Sisyphus                 => "SPITE UNLOCKED. Your suffering is now a weapon.",
            MiseryMilestone::StatisticalImpossibility => "DEFIANCE UNLOCKED. You should be dead. The math agrees. You disagree.",
            MiseryMilestone::ThePunchline             => "COSMIC JOKE UNLOCKED. The universe has decided you're funny.",
            MiseryMilestone::ActuallyImpressive       => "TRANSCENDENT MISERY UNLOCKED. Misery is now your identity.",
            MiseryMilestone::TheyWillWritePapers      => "PUBLISHED FAILURE UNLOCKED. A paper will be written about you.",
            MiseryMilestone::TheMostWretched          => "HALL OF MISERY. Your name will live in infamy.",
            MiseryMilestone::NegativeGod              => "The inverse of ΩMEGA. You have suffered more than any being should.",
        }
    }

    pub const ALL: &'static [MiseryMilestone] = &[
        MiseryMilestone::ItGetsWorse,
        MiseryMilestone::BelovedOfMurphy,
        MiseryMilestone::ProfessionalVictim,
        MiseryMilestone::UniversesPunchingBag,
        MiseryMilestone::Sisyphus,
        MiseryMilestone::StatisticalImpossibility,
        MiseryMilestone::ThePunchline,
        MiseryMilestone::ActuallyImpressive,
        MiseryMilestone::TheyWillWritePapers,
        MiseryMilestone::TheMostWretched,
        MiseryMilestone::NegativeGod,
    ];
}

// ── Spite actions ─────────────────────────────────────────────────────────────

/// Actions the player can spend Spite to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpiteAction {
    /// Next attack deals Misery/100 guaranteed damage (costs 50).
    SpitefulStrike,
    /// Survive one killing blow at 1 HP (costs 30).
    BitterEndurance,
    /// Enemy takes total Misery damage when encountered (costs 100). Cross-run revenge.
    VengefulEcho,
    /// Inflict your worst stat as a debuff on enemy (costs 75).
    MiseryLovesCompany,
    /// Reroll the current enemy entirely (costs 200).
    CosmicComplaint,
}

impl SpiteAction {
    pub fn cost(self) -> f64 {
        match self {
            SpiteAction::SpitefulStrike    => 50.0,
            SpiteAction::BitterEndurance   => 30.0,
            SpiteAction::VengefulEcho      => 100.0,
            SpiteAction::MiseryLovesCompany=> 75.0,
            SpiteAction::CosmicComplaint   => 200.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SpiteAction::SpitefulStrike     => "Spiteful Strike",
            SpiteAction::BitterEndurance    => "Bitter Endurance",
            SpiteAction::VengefulEcho       => "Vengeful Echo",
            SpiteAction::MiseryLovesCompany => "Misery Loves Company",
            SpiteAction::CosmicComplaint    => "Cosmic Complaint",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            SpiteAction::SpitefulStrike     => "Deal MISERY/100 guaranteed damage, bypassing all rolls.",
            SpiteAction::BitterEndurance    => "Survive a killing blow at 1 HP. Fueled by refusal to die.",
            SpiteAction::VengefulEcho       => "Your nemesis takes your Misery Index as damage on encounter.",
            SpiteAction::MiseryLovesCompany => "Inflict your worst stat as a debuff on the enemy.",
            SpiteAction::CosmicComplaint    => "Force the chaos pipeline to reroll the current enemy entirely.",
        }
    }
}

// ── Defiance passives ─────────────────────────────────────────────────────────

/// Passives unlocked by surviving N rolls in Defiance mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefiancePassive {
    /// 1000 rolls: −50% incoming damage.
    ImmovableObject,
    /// 5000 rolls: defense = |lowest_stat| / 10.
    ParadoxArmor,
    /// 10000 rolls: survival probability displayed on every roll.
    MathematicalImprobability,
}

impl DefiancePassive {
    pub fn rolls_required(self) -> u64 {
        match self {
            DefiancePassive::ImmovableObject           => 1_000,
            DefiancePassive::ParadoxArmor              => 5_000,
            DefiancePassive::MathematicalImprobability => 10_000,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            DefiancePassive::ImmovableObject           => "Immovable Object",
            DefiancePassive::ParadoxArmor              => "Paradox Armor",
            DefiancePassive::MathematicalImprobability => "Mathematical Improbability",
        }
    }
}

// ── Main MiseryState struct ───────────────────────────────────────────────────

/// All negative-run mechanics for a character. Serialized with the character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiseryState {
    pub misery_index: f64,
    pub spite: f64,
    pub spite_total_spent: f64,
    /// Chaos rolls survived since entering Defiance.
    pub defiance_rolls: u64,
    pub in_defiance: bool,
    /// Passives earned through Defiance.
    pub defiance_passives: Vec<DefiancePassive>,
    pub cosmic_joke: bool,
    pub transcendent_misery: bool,
    /// Milestones already hit (to avoid double-triggering).
    pub milestones_hit: Vec<MiseryMilestone>,
    /// Pending milestone notifications to show the player (consumed by UI).
    pub pending_notifications: Vec<MiseryMilestone>,
    /// Whether Bitter Endurance is primed (will trigger on next kill shot).
    pub bitter_endurance_primed: bool,
    /// Guaranteed damage charge from Spiteful Strike (0 if not active).
    pub spiteful_strike_charge: f64,
    /// Total Spite ever accumulated this run (for stats).
    pub spite_total_earned: f64,
    /// Per-milestone XP bonus from milestones.
    pub milestone_xp_bonus: f64,
}

impl Default for MiseryState {
    fn default() -> Self {
        Self {
            misery_index: 0.0,
            spite: 0.0,
            spite_total_spent: 0.0,
            defiance_rolls: 0,
            in_defiance: false,
            defiance_passives: Vec::new(),
            cosmic_joke: false,
            transcendent_misery: false,
            milestones_hit: Vec::new(),
            pending_notifications: Vec::new(),
            bitter_endurance_primed: false,
            spiteful_strike_charge: 0.0,
            spite_total_earned: 0.0,
            milestone_xp_bonus: 0.0,
        }
    }
}

impl MiseryState {
    pub fn new() -> Self { Self::default() }

    // ── Misery accumulation ───────────────────────────────────────────────────

    /// Add misery from a specific source with a magnitude value.
    /// Returns newly triggered milestones.
    pub fn add_misery(&mut self, source: MiserySource, magnitude: f64) -> Vec<MiseryMilestone> {
        let amount = match source {
            MiserySource::DamageTaken          => magnitude,
            MiserySource::SpellBackfire        => magnitude * 2.0,
            MiserySource::Headshot             => 50.0,
            MiserySource::AttackMissed         => 10.0,
            MiserySource::EnemyDodged          => 20.0,
            MiserySource::SkillCheckFailed     => magnitude,
            MiserySource::FleeFailed           => 100.0,
            MiserySource::ItemVanished         => magnitude,
            MiserySource::EnemyPitiedYou       => 200.0,
            MiserySource::ShopTooExpensive     => 50.0,
            MiserySource::StatDecreaseOnLevelUp=> magnitude * 10.0,
            MiserySource::DeathRemainingEnemyHp=> magnitude,
        };

        // Spite accumulates at roughly Misery / 10 rate, only from active suffering
        let spite_gain = match source {
            MiserySource::DamageTaken | MiserySource::SpellBackfire |
            MiserySource::FleeFailed  | MiserySource::Headshot => amount * 0.1,
            _ => 0.0,
        };

        self.misery_index += amount;
        if spite_gain > 0.0 && self.milestones_hit.contains(&MiseryMilestone::Sisyphus) {
            self.add_spite(spite_gain);
        }

        self.check_milestones()
    }

    fn check_milestones(&mut self) -> Vec<MiseryMilestone> {
        let mut newly_hit = Vec::new();
        for &ms in MiseryMilestone::ALL {
            if !self.milestones_hit.contains(&ms) && self.misery_index >= ms.threshold() {
                self.milestones_hit.push(ms);
                self.pending_notifications.push(ms);
                newly_hit.push(ms);
                // Apply milestone effects
                match ms {
                    MiseryMilestone::Sisyphus => { /* Spite unlocked — no action needed */ }
                    MiseryMilestone::StatisticalImpossibility => { self.in_defiance = true; }
                    MiseryMilestone::ThePunchline  => { self.cosmic_joke = true; }
                    MiseryMilestone::ActuallyImpressive => { self.transcendent_misery = true; }
                    MiseryMilestone::ProfessionalVictim  => { self.milestone_xp_bonus += 0.05; }
                    MiseryMilestone::UniversesPunchingBag=> { self.milestone_xp_bonus += 0.10; }
                    _ => {}
                }
            }
        }
        newly_hit
    }

    // ── Spite ─────────────────────────────────────────────────────────────────

    pub fn add_spite(&mut self, amount: f64) {
        self.spite += amount;
        self.spite_total_earned += amount;
    }

    /// Attempt to spend Spite on an action. Returns true if successful.
    pub fn spend_spite(&mut self, action: SpiteAction) -> bool {
        let cost = action.cost();
        if self.spite < cost { return false; }
        self.spite -= cost;
        self.spite_total_spent += cost;
        match action {
            SpiteAction::BitterEndurance => self.bitter_endurance_primed = true,
            SpiteAction::SpitefulStrike  => self.spiteful_strike_charge = self.misery_index / 100.0,
            _ => {}
        }
        true
    }

    /// Decay Spite by 1 per room when not in combat.
    pub fn tick_room_decay(&mut self) {
        self.spite = (self.spite - 1.0).max(0.0);
    }

    /// Consume the Bitter Endurance charge, returns true if it was active.
    pub fn consume_bitter_endurance(&mut self) -> bool {
        if self.bitter_endurance_primed {
            self.bitter_endurance_primed = false;
            true
        } else {
            false
        }
    }

    /// Consume the Spiteful Strike charge, returning the guaranteed damage (0 if none).
    pub fn consume_spiteful_strike(&mut self) -> f64 {
        let dmg = self.spiteful_strike_charge;
        self.spiteful_strike_charge = 0.0;
        dmg
    }

    // ── Defiance ──────────────────────────────────────────────────────────────

    /// Increment the defiance roll counter and check for new passives.
    /// Returns newly unlocked passives.
    pub fn increment_defiance_roll(&mut self) -> Vec<DefiancePassive> {
        if !self.in_defiance { return Vec::new(); }
        self.defiance_rolls += 1;
        // Every 100 rolls → +1 to all stats (tracked externally, we just signal)
        let mut new_passives = Vec::new();
        for &p in &[DefiancePassive::ImmovableObject, DefiancePassive::ParadoxArmor,
                    DefiancePassive::MathematicalImprobability] {
            if self.defiance_rolls >= p.rolls_required() && !self.defiance_passives.contains(&p) {
                self.defiance_passives.push(p);
                new_passives.push(p);
            }
        }
        new_passives
    }

    /// Returns true if every-100-rolls stat bonus should trigger.
    pub fn should_grant_defiance_stat_bonus(&self) -> bool {
        self.in_defiance && self.defiance_rolls > 0 && self.defiance_rolls % 100 == 0
    }

    pub fn has_immovable_object(&self) -> bool {
        self.defiance_passives.contains(&DefiancePassive::ImmovableObject)
    }

    pub fn has_paradox_armor(&self) -> bool {
        self.defiance_passives.contains(&DefiancePassive::ParadoxArmor)
    }

    // ── Underdog multiplier ───────────────────────────────────────────────────

    /// XP / score multiplier for negative stat totals. Always ≥ 1.0.
    pub fn underdog_multiplier(stat_total: i64) -> f64 {
        if stat_total >= 0 { return 1.0; }
        let neg = (-stat_total) as f64;
        (1.0 + neg.log10().max(0.0) * 1.5).min(10.0)
    }

    /// Loot rarity bonus tiers from underdog multiplier (+1 per 2×).
    pub fn underdog_loot_bonus(stat_total: i64) -> u32 {
        let mult = Self::underdog_multiplier(stat_total);
        ((mult - 1.0) / 2.0) as u32
    }

    /// Chance (0.0–1.0) that an enemy pity-skips its attack this round.
    pub fn enemy_pity_chance(stat_total: i64) -> f64 {
        if stat_total >= -200 { return 0.0; }
        let neg = (-stat_total) as f64;
        // Scales from ~5% at -200 up to 25% at -1_000_000_000
        (neg.log10() * 0.04 - 0.04 * 200_f64.log10()).clamp(0.0, 0.25)
    }

    /// XP multiplier including milestone bonuses.
    pub fn total_xp_multiplier(&self, stat_total: i64) -> f64 {
        Self::underdog_multiplier(stat_total) * (1.0 + self.milestone_xp_bonus)
    }

    // ── Spite passive: Refuse to Die ─────────────────────────────────────────

    /// True if the passive "Refuse to Die" is active (≥500 accumulated spite & ≥200 current).
    pub fn refuse_to_die_active(&self) -> bool {
        self.spite_total_earned >= 500.0 && self.spite >= 200.0
    }

    // ── Cosmic Joke text generation ───────────────────────────────────────────

    /// Returns an occasional cosmic joke flavor line, or None.
    /// `floor_seed` is used so jokes are deterministic per floor.
    pub fn cosmic_joke_combat_line(floor_seed: u64, frame: u64) -> Option<&'static str> {
        const LINES: &[&str] = &[
            "The Lorenz Attractor briefly considers feeling sorry for you.",
            "Even the butterfly feels bad about what's happening to you.",
            "A nearby Fibonacci Spiral averts its gaze.",
            "The Collatz Chain pauses to acknowledge your suffering before continuing.",
            "Your stat sheet has developed sentience and filed for emancipation.",
            "The Prime Density Sieve has flagged your case as 'statistically concerning'.",
            "Somewhere, a set theorist is crying and doesn't know why.",
            "The Mandelbrot set renders a tiny frowny face at your coordinates.",
        ];
        // Show a cosmic joke ~15% of combat rounds, deterministically
        let roll = (floor_seed.wrapping_mul(6271).wrapping_add(frame.wrapping_mul(9973))) % 100;
        if roll < 15 {
            Some(LINES[(roll as usize) % LINES.len()])
        } else {
            None
        }
    }

    // ── Published Failure paper generation ───────────────────────────────────

    /// Generate the in-game "academic paper" text for this run.
    pub fn generate_paper(
        &self,
        name: &str,
        class: &str,
        level: u32,
        floor: u32,
        stat_total: i64,
        seed: u64,
    ) -> String {
        let p_survival = Self::survival_probability(stat_total, floor);
        let coin_flips = if p_survival > 0.0 {
            (-p_survival.log2()).round() as u64
        } else { 99 };

        format!(
"┌─────────────────────────────────────────────────────────────┐
│           PREPRINT — Not Peer Reviewed                      │
│                                                             │
│  \"On the Statistical Impossibility of {name}:               │
│   A Case Study in Mathematical Suffering\"                   │
│                                                             │
│  Abstract: We present the case of {name}, a Level {level} {class}  │
│  who achieved a Misery Index of {:.0} across {floor} floors of  │
│  Chaos RPG. With a stat total of {stat_total}, this character   │
│  represents a {:.2e} probability event — equivalent to a    │
│  coin landing on its edge {coin_flips} times consecutively.       │
│  The subject's continued existence challenges several        │
│  assumptions in chaos theory and raises questions about      │
│  the nature of spite as a survival mechanism.               │
│                                                             │
│  Keywords: mathematical suffering, chaos pipeline abuse,    │
│  negative statistics, spite-based survival, underdog theory │
│                                                             │
│  DOI: 10.chaos-rpg/{seed:016x}                              │
│  Published in: The Journal of Computational Misery          │
└─────────────────────────────────────────────────────────────┘",
            self.misery_index,
            p_survival,
        )
    }

    /// Rough estimate of probability of reaching `floor` with `stat_total`.
    fn survival_probability(stat_total: i64, floor: u32) -> f64 {
        if stat_total >= 0 || floor == 0 { return 1.0; }
        let neg = (-stat_total).max(1) as f64;
        let base = 1.0 / (1.0 + neg * 0.001);
        base.powi(floor as i32)
    }

    // ── Drain pending notifications ───────────────────────────────────────────

    /// Take all pending notifications, clearing the queue.
    pub fn drain_notifications(&mut self) -> Vec<MiseryMilestone> {
        std::mem::take(&mut self.pending_notifications)
    }

    // ── Display helpers ───────────────────────────────────────────────────────

    /// Primary displayed metric label and value for the character sheet.
    /// Returns ("MISERY", index) for high-misery chars, ("POWER", tier_name) otherwise.
    pub fn display_primary<'a>(&self, stat_total: i64, tier_name: &'a str) -> (&'static str, String) {
        if stat_total < 0 && self.misery_index >= 1_000.0 {
            ("MISERY", format!("{:.0}", self.misery_index))
        } else {
            ("POWER", tier_name.to_string())
        }
    }
}
