//! Skill checks — verbose engine chain display for all non-combat decisions.
//!
//! Every skill check shows its full chaos engine chain. The math is visible.
//! You can watch your fate being calculated in real time.

use crate::chaos_pipeline::{biased_chaos_roll, chaos_roll_verbose, ChaosRollResult};
use crate::character::Character;

// ─── SKILL CHECK TYPES ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A skill the player can test outside combat.
pub enum SkillType {
    /// Spot traps and hidden items (precision).
    Perception,    // spot traps, hidden items
    /// Sneak past enemies (cunning).
    Stealth,       // sneak past enemies
    /// Open locked chests (cunning).
    Lockpick,      // open locked chests
    /// Get better prices and talk past fights (cunning plus half luck).
    Persuasion,    // better NPC prices, avoid fights
    /// Identify magic items and spells (mana).
    Arcana,        // identify magic items, spells
    /// Survive environmental effects (vitality plus half force).
    Athletics,     // survive environmental effects
    /// A pure chaos roll with no stat bias (luck).
    Luck,          // pure chaos roll with no stat bias
    /// An unbiased roll that is harder to predict (entropy).
    ChaosAffinity, // harder to predict, higher ceiling
}

impl SkillType {
    /// Display name of the skill.
    pub fn name(&self) -> &'static str {
        match self {
            SkillType::Perception => "Perception",
            SkillType::Stealth => "Stealth",
            SkillType::Lockpick => "Lockpick",
            SkillType::Persuasion => "Persuasion",
            SkillType::Arcana => "Arcana",
            SkillType::Athletics => "Athletics",
            SkillType::Luck => "Luck",
            SkillType::ChaosAffinity => "Chaos Affinity",
        }
    }

    /// The character's stat value used for this skill.
    pub fn governing_stat(&self, character: &Character) -> i64 {
        match self {
            SkillType::Perception => character.stats.precision,
            SkillType::Stealth => character.stats.cunning,
            SkillType::Lockpick => character.stats.cunning,
            SkillType::Persuasion => character.stats.cunning + character.stats.luck / 2,
            SkillType::Arcana => character.stats.mana,
            SkillType::Athletics => character.stats.vitality + character.stats.force / 2,
            SkillType::Luck => character.stats.luck,
            SkillType::ChaosAffinity => character.stats.entropy,
        }
    }
}

// ─── DIFFICULTY ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How hard a skill check is.
pub enum Difficulty {
    /// DC 10.
    Trivial,    // DC 10
    /// DC 25.
    Easy,       // DC 25
    /// DC 40.
    Medium,     // DC 40
    /// DC 60.
    Hard,       // DC 60
    /// DC 80.
    Extreme,    // DC 80
    /// DC 95.
    Impossible, // DC 95 — the math itself fights you
}

impl Difficulty {
    /// Display name of the difficulty.
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Trivial => "Trivial",
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Extreme => "Extreme",
            Difficulty::Impossible => "IMPOSSIBLE",
        }
    }

    /// Number the roll must reach (10 to 95 on a 1 to 100 scale).
    pub fn dc(&self) -> i64 {
        match self {
            Difficulty::Trivial => 10,
            Difficulty::Easy => 25,
            Difficulty::Medium => 40,
            Difficulty::Hard => 60,
            Difficulty::Extreme => 80,
            Difficulty::Impossible => 95,
        }
    }
}

// ─── SKILL CHECK RESULT ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
/// Outcome of a skill check.
pub struct SkillCheckResult {
    /// Skill tested.
    pub skill: SkillType,
    /// Difficulty attempted.
    pub difficulty: Difficulty,
    /// Roll from 1 to 100.
    pub roll_value: i64, // 1-100
    /// Number the roll had to reach.
    pub dc: i64,
    /// True if the check succeeded.
    pub passed: bool,
    /// Roll minus DC: positive by how much it passed, negative by how much it failed.
    pub margin: i64, // how much over/under DC
    /// The chaos roll behind the result.
    pub chaos_result: ChaosRollResult,
    /// Text describing what happened.
    pub narrative: String,
}

impl SkillCheckResult {
    /// Generate verbose display lines for terminal rendering
    pub fn display_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push("".to_string());
        lines.push(format!(
            "  ╔══ SKILL CHECK: {} [{}] ══╗",
            self.skill.name(),
            self.difficulty.name()
        ));

        // Engine chain
        for line in self.chaos_result.display_lines() {
            lines.push(line);
        }

        let result_color = if self.passed { "\x1b[32m" } else { "\x1b[31m" };
        let reset = "\x1b[0m";
        let margin_str = if self.margin >= 0 {
            format!("+{}", self.margin)
        } else {
            format!("{}", self.margin)
        };

        lines.push(format!(
            "  Roll: {}  DC: {}  Margin: {}",
            self.roll_value, self.dc, margin_str
        ));
        lines.push(format!(
            "  {}{}{} — {}",
            result_color,
            if self.passed { "SUCCESS" } else { "FAILURE" },
            reset,
            self.narrative
        ));
        lines.push("".to_string());

        lines
    }
}

// ─── SKILL CHECK EXECUTION ───────────────────────────────────────────────────

/// Roll a skill check through the chaos pipeline, biased by the governing stat (Luck and Chaos Affinity roll unbiased), against the difficulty's DC.
pub fn perform_skill_check(
    character: &Character,
    skill: SkillType,
    difficulty: Difficulty,
    seed: u64,
) -> SkillCheckResult {
    let governing_stat = skill.governing_stat(character);
    let bias = (governing_stat as f64 / 100.0).clamp(-0.5, 0.5);

    let chaos_result = if matches!(skill, SkillType::Luck | SkillType::ChaosAffinity) {
        // Luck and Chaos Affinity get pure unbiased rolls
        chaos_roll_verbose(seed as f64 * 1e-11, seed)
    } else {
        biased_chaos_roll(governing_stat as f64 * 0.01, bias, seed)
    };

    let roll_value = chaos_result.to_range(1, 100);
    let dc = difficulty.dc();
    let passed = roll_value >= dc;
    let margin = roll_value - dc;

    let narrative = generate_narrative(skill, difficulty, passed, margin, character);

    SkillCheckResult {
        skill,
        difficulty,
        roll_value,
        dc,
        passed,
        margin,
        chaos_result,
        narrative,
    }
}

fn generate_narrative(
    skill: SkillType,
    difficulty: Difficulty,
    passed: bool,
    margin: i64,
    character: &Character,
) -> String {
    let name = &character.name;

    if margin > 30 {
        // Exceptional success
        match skill {
            SkillType::Perception => format!(
                "{} sees the trap before it even activates. Impressive.",
                name
            ),
            SkillType::Stealth => format!("{} becomes one with the math. Invisible.", name),
            SkillType::Lockpick => {
                format!("{} doesn't just pick the lock — they befriend it.", name)
            }
            SkillType::Persuasion => format!("{} is so persuasive the merchant PAYS them.", name),
            SkillType::Arcana => format!("{} understands the equations on a cellular level.", name),
            SkillType::Athletics => {
                format!("{} transcends the physical limitation. Briefly.", name)
            }
            SkillType::Luck => format!("{} is beloved by prime numbers today.", name),
            SkillType::ChaosAffinity => format!("{} and the chaos are one. Terrifying.", name),
        }
    } else if passed {
        match skill {
            SkillType::Perception => format!("{} notices the irregularity.", name),
            SkillType::Stealth => format!("{} moves without disturbing the logistic map.", name),
            SkillType::Lockpick => format!("{} coaxes the mechanism open.", name),
            SkillType::Persuasion => format!("{} makes a compelling argument.", name),
            SkillType::Arcana => format!("{} reads the equations correctly.", name),
            SkillType::Athletics => format!("{} powers through.", name),
            SkillType::Luck => format!("{} gets lucky. For now.", name),
            SkillType::ChaosAffinity => format!("{} rides the chaos wave.", name),
        }
    } else if margin > -20 {
        // Close failure
        format!(
            "{} almost succeeds at {} but the {} prevents it.",
            name,
            skill.name(),
            difficulty.name()
        )
    } else {
        // Catastrophic failure
        match difficulty {
            Difficulty::Impossible => {
                format!(
                    "{} fails catastrophically. The {} was working against them from the start.",
                    name,
                    skill.name()
                )
            }
            _ => format!(
                "{} fails the {} check. The mathematics are unforgiving.",
                name,
                skill.name()
            ),
        }
    }
}

/// Quick pass/fail check without full verbose display
pub fn quick_check(
    character: &Character,
    skill: SkillType,
    difficulty: Difficulty,
    seed: u64,
) -> bool {
    perform_skill_check(character, skill, difficulty, seed).passed
}

/// Trap check — perception-based, triggered on entering certain rooms
pub fn trap_check(character: &Character, seed: u64) -> SkillCheckResult {
    let diff = match character.floor {
        1..=3 => Difficulty::Easy,
        4..=7 => Difficulty::Medium,
        8..=12 => Difficulty::Hard,
        _ => Difficulty::Extreme,
    };
    perform_skill_check(character, SkillType::Perception, diff, seed)
}

/// Stealth check — used to potentially skip combat
pub fn stealth_check(character: &Character, enemy_level: u32, seed: u64) -> SkillCheckResult {
    let diff = match enemy_level {
        0..=2 => Difficulty::Easy,
        3..=5 => Difficulty::Medium,
        6..=9 => Difficulty::Hard,
        10..=14 => Difficulty::Extreme,
        _ => Difficulty::Impossible,
    };
    perform_skill_check(character, SkillType::Stealth, diff, seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::{Background, CharacterClass};

    fn test_char() -> Character {
        Character::roll_new(
            "Test".to_string(),
            CharacterClass::Thief,
            Background::Outcast,
            42,
            crate::character::Difficulty::Normal,
        )
    }

    #[test]
    fn skill_check_produces_valid_result() {
        let c = test_char();
        let result = perform_skill_check(&c, SkillType::Stealth, Difficulty::Medium, 99);
        assert!(result.roll_value >= 1 && result.roll_value <= 100);
        assert!(result.passed == (result.roll_value >= result.dc));
    }

    #[test]
    fn display_lines_not_empty() {
        let c = test_char();
        let result = perform_skill_check(&c, SkillType::Luck, Difficulty::Easy, 1);
        let lines = result.display_lines();
        assert!(lines.len() > 5);
    }

    #[test]
    fn high_cunning_thief_passes_stealth_more() {
        let mut thief = test_char();
        thief.stats.cunning = 120;
        let mut passes = 0;
        for seed in 0..20u64 {
            if quick_check(&thief, SkillType::Stealth, Difficulty::Easy, seed) {
                passes += 1;
            }
        }
        assert!(
            passes >= 12,
            "High-cunning thief should pass easy stealth often"
        );
    }
}
