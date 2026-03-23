//! Dwarf Fortress-style body part system.
//!
//! Each character has 13 body parts with individual HP pools, chaos-rolled at
//! creation. Damage targets specific parts. Injuries persist between fights and
//! inflict stat penalties. Negative-HP parts are cursed — still attached,
//! actively draining HP each turn.

use crate::chaos_pipeline::chaos_roll_verbose;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── BODY PARTS ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyPart {
    Head,
    Torso,
    Neck,
    LeftArm,
    RightArm,
    LeftHand,
    RightHand,
    LeftLeg,
    RightLeg,
    LeftFoot,
    RightFoot,
    LeftEye,
    RightEye,
}

impl BodyPart {
    pub const ALL: &'static [BodyPart] = &[
        BodyPart::Head,
        BodyPart::Torso,
        BodyPart::Neck,
        BodyPart::LeftArm,
        BodyPart::RightArm,
        BodyPart::LeftHand,
        BodyPart::RightHand,
        BodyPart::LeftLeg,
        BodyPart::RightLeg,
        BodyPart::LeftFoot,
        BodyPart::RightFoot,
        BodyPart::LeftEye,
        BodyPart::RightEye,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BodyPart::Head => "Head",
            BodyPart::Torso => "Torso",
            BodyPart::Neck => "Neck",
            BodyPart::LeftArm => "Left Arm",
            BodyPart::RightArm => "Right Arm",
            BodyPart::LeftHand => "Left Hand",
            BodyPart::RightHand => "Right Hand",
            BodyPart::LeftLeg => "Left Leg",
            BodyPart::RightLeg => "Right Leg",
            BodyPart::LeftFoot => "Left Foot",
            BodyPart::RightFoot => "Right Foot",
            BodyPart::LeftEye => "Left Eye",
            BodyPart::RightEye => "Right Eye",
        }
    }

    /// Hit probability weight (higher = more likely to be targeted).
    pub fn hit_weight(self) -> u32 {
        match self {
            BodyPart::Torso => 30,
            BodyPart::Head => 15,
            BodyPart::LeftArm => 10,
            BodyPart::RightArm => 10,
            BodyPart::LeftLeg => 10,
            BodyPart::RightLeg => 10,
            BodyPart::Neck => 5,
            BodyPart::LeftHand => 3,
            BodyPart::RightHand => 3,
            BodyPart::LeftFoot => 2,
            BodyPart::RightFoot => 2,
            BodyPart::LeftEye => 1,
            BodyPart::RightEye => 1,
        }
    }

    /// HP fraction relative to the torso (torso = 1.0).
    pub fn hp_factor(self) -> f64 {
        match self {
            BodyPart::Torso => 1.0,
            BodyPart::Head => 0.50,
            BodyPart::Neck => 0.30,
            BodyPart::LeftArm => 0.40,
            BodyPart::RightArm => 0.40,
            BodyPart::LeftLeg => 0.50,
            BodyPart::RightLeg => 0.50,
            BodyPart::LeftHand => 0.25,
            BodyPart::RightHand => 0.25,
            BodyPart::LeftFoot => 0.20,
            BodyPart::RightFoot => 0.20,
            BodyPart::LeftEye => 0.10,
            BodyPart::RightEye => 0.10,
        }
    }

    pub fn armor_slot(self) -> &'static str {
        match self {
            BodyPart::Head => "Helmet",
            BodyPart::Torso => "Chestplate",
            BodyPart::LeftArm | BodyPart::RightArm => "Pauldron",
            BodyPart::LeftLeg | BodyPart::RightLeg => "Greaves",
            BodyPart::Neck => "Amulet",
            BodyPart::LeftHand | BodyPart::RightHand => "Gloves",
            BodyPart::LeftFoot | BodyPart::RightFoot => "Boots",
            BodyPart::LeftEye | BodyPart::RightEye => "Goggles",
        }
    }
}

// ─── INJURY SEVERITY ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjurySeverity {
    Bruised,              // <25% of part max HP lost
    Fractured,            // 25–50%
    Shattered,            // 50–75%
    Severed,              // 75–99%
    MathematicallyAbsent, // HP ≤ 0 — attached but cursed
}

impl InjurySeverity {
    pub fn name(&self) -> &'static str {
        match self {
            InjurySeverity::Bruised => "Bruised",
            InjurySeverity::Fractured => "Fractured",
            InjurySeverity::Shattered => "Shattered",
            InjurySeverity::Severed => "Severed",
            InjurySeverity::MathematicallyAbsent => "MATH.ABSENT",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            InjurySeverity::Bruised => "\x1b[33m",
            InjurySeverity::Fractured => "\x1b[91m",
            InjurySeverity::Shattered => "\x1b[31m",
            InjurySeverity::Severed => "\x1b[35m",
            InjurySeverity::MathematicallyAbsent => "\x1b[95m",
        }
    }

    fn level(&self) -> u8 {
        match self {
            InjurySeverity::Bruised => 1,
            InjurySeverity::Fractured => 2,
            InjurySeverity::Shattered => 3,
            InjurySeverity::Severed => 4,
            InjurySeverity::MathematicallyAbsent => 5,
        }
    }

    pub fn from_damage_ratio(damage_taken: i64, max_hp: i64) -> Option<Self> {
        if damage_taken <= 0 || max_hp <= 0 {
            return None;
        }
        let ratio = damage_taken as f64 / max_hp as f64;
        Some(if ratio >= 1.0 {
            InjurySeverity::MathematicallyAbsent
        } else if ratio >= 0.75 {
            InjurySeverity::Severed
        } else if ratio >= 0.50 {
            InjurySeverity::Shattered
        } else if ratio >= 0.25 {
            InjurySeverity::Fractured
        } else {
            InjurySeverity::Bruised
        })
    }
}

impl PartialOrd for InjurySeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InjurySeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }
}

// ─── BODY PART STATE ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPartState {
    pub current_hp: i64,
    pub max_hp: i64,
    pub injury: Option<InjurySeverity>,
    /// Defense from armor equipped to this slot.
    pub armor_defense: i64,
    pub armor_name: Option<String>,
}

// ─── BODY PENALTIES ──────────────────────────────────────────────────────────

/// Stat adjustments from body injuries. Mirrors StatBlock fields to avoid
/// a circular dependency between body.rs and character.rs.
#[derive(Debug, Clone, Default)]
pub struct BodyPenalties {
    pub vitality: i64,
    pub force: i64,
    pub mana: i64,
    pub cunning: i64,
    pub precision: i64,
    pub entropy: i64,
    pub luck: i64,
}

impl BodyPenalties {
    fn apply(&mut self, part: BodyPart, sev: &InjurySeverity, both_eyes_gone: bool) {
        match sev {
            InjurySeverity::Bruised => match part {
                BodyPart::LeftArm | BodyPart::RightArm => self.force -= 3,
                BodyPart::LeftLeg | BodyPart::RightLeg => self.precision -= 3,
                _ => {}
            },
            InjurySeverity::Fractured => match part {
                BodyPart::LeftArm | BodyPart::RightArm => {
                    self.force -= 8;
                    self.precision -= 4;
                }
                BodyPart::LeftLeg | BodyPart::RightLeg => {
                    self.precision -= 8;
                    self.cunning -= 3;
                }
                BodyPart::Head => {
                    self.mana -= 5;
                    self.cunning -= 5;
                }
                _ => {}
            },
            InjurySeverity::Shattered => match part {
                BodyPart::LeftArm | BodyPart::RightArm => {
                    self.force -= 20;
                    self.precision -= 10;
                }
                BodyPart::LeftLeg | BodyPart::RightLeg => {
                    self.precision -= 20;
                    self.luck -= 10;
                }
                BodyPart::Head => {
                    self.mana -= 15;
                    self.cunning -= 15;
                }
                BodyPart::LeftEye | BodyPart::RightEye => self.precision -= 25,
                _ => {}
            },
            InjurySeverity::Severed => match part {
                BodyPart::LeftArm | BodyPart::RightArm => {
                    self.force -= 35;
                    self.precision -= 20;
                }
                BodyPart::LeftLeg | BodyPart::RightLeg => {
                    self.precision -= 35;
                    self.luck -= 20;
                }
                BodyPart::Head => {
                    self.mana -= 30;
                    self.cunning -= 30;
                }
                BodyPart::LeftEye | BodyPart::RightEye => {
                    self.precision -= 40;
                    self.entropy += 15;
                }
                BodyPart::Neck => {
                    self.vitality -= 20;
                    self.mana -= 20;
                }
                _ => {}
            },
            InjurySeverity::MathematicallyAbsent => {
                match part {
                    BodyPart::LeftArm | BodyPart::RightArm => {
                        self.force -= 50;
                        self.precision -= 30;
                    }
                    BodyPart::LeftLeg | BodyPart::RightLeg => {
                        self.precision -= 50;
                        self.luck -= 30;
                    }
                    BodyPart::LeftEye | BodyPart::RightEye => {
                        if both_eyes_gone {
                            // Both eyes gone: precision near-zero, other senses sharpen.
                            self.precision -= 100;
                            self.entropy += 50;
                        } else {
                            self.precision -= 50;
                            self.entropy += 20;
                        }
                    }
                    BodyPart::Neck => {
                        self.vitality -= 50;
                        self.mana -= 30;
                    }
                    _ => {}
                }
            }
        }
    }
}

// ─── BODY ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub parts: HashMap<BodyPart, BodyPartState>,
}

impl Body {
    /// Generate a new body. HP per part is chaos-rolled from `base_vitality`.
    pub fn generate(base_vitality: i64, seed: u64) -> Self {
        let torso_hp = (base_vitality * 2 + 20).max(10);
        let mut parts = HashMap::new();
        let mut s = seed;
        for part in BodyPart::ALL {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let base = (torso_hp as f64 * part.hp_factor()) as i64;
            let chaos = chaos_roll_verbose(base as f64 * 0.01, s);
            let variance = (base as f64 * chaos.final_value.abs() * 0.5) as i64;
            let hp = (base + variance).max(1);
            parts.insert(
                *part,
                BodyPartState {
                    current_hp: hp,
                    max_hp: hp,
                    injury: None,
                    armor_defense: 0,
                    armor_name: None,
                },
            );
        }
        Body { parts }
    }

    /// Which body part does an attack hit? Weighted random from seed.
    pub fn target_part(seed: u64) -> BodyPart {
        let total: u32 = BodyPart::ALL.iter().map(|p| p.hit_weight()).sum();
        let roll = (seed % total as u64) as u32;
        let mut acc = 0u32;
        for part in BodyPart::ALL {
            acc += part.hit_weight();
            if roll < acc {
                return *part;
            }
        }
        BodyPart::Torso
    }

    /// Apply `raw_dmg` to `part` (reduced by armor). Returns (actual_dmg, injury).
    pub fn damage_part(&mut self, part: BodyPart, raw_dmg: i64) -> (i64, Option<InjurySeverity>) {
        let state = self.parts.entry(part).or_insert_with(|| BodyPartState {
            current_hp: 10,
            max_hp: 10,
            injury: None,
            armor_defense: 0,
            armor_name: None,
        });
        let dmg = (raw_dmg - state.armor_defense).max(0);
        state.current_hp -= dmg;
        let lost = state.max_hp - state.current_hp;
        let new_sev = InjurySeverity::from_damage_ratio(lost, state.max_hp);
        // Injuries only worsen from taking damage; only heals improve them.
        let worsen = match (&state.injury, &new_sev) {
            (None, Some(_)) => true,
            (Some(old), Some(new)) if new > old => true,
            _ => false,
        };
        if worsen {
            state.injury = new_sev.clone();
        }
        (dmg, state.injury.clone())
    }

    /// Heal a specific part by `amount` HP.
    pub fn heal_part(&mut self, part: BodyPart, amount: i64) {
        if let Some(s) = self.parts.get_mut(&part) {
            s.current_hp = (s.current_hp + amount).min(s.max_hp);
            let lost = s.max_hp - s.current_hp;
            s.injury = InjurySeverity::from_damage_ratio(lost, s.max_hp);
        }
    }

    /// Heal all parts by `pct` fraction of their individual max HP.
    pub fn heal_all_pct(&mut self, pct: f64) {
        for s in self.parts.values_mut() {
            let amt = (s.max_hp as f64 * pct) as i64;
            s.current_hp = (s.current_hp + amt).min(s.max_hp);
            let lost = s.max_hp - s.current_hp;
            s.injury = InjurySeverity::from_damage_ratio(lost, s.max_hp);
        }
    }

    /// Head at ≤0 HP = instant death regardless of torso HP.
    pub fn head_destroyed(&self) -> bool {
        self.parts
            .get(&BodyPart::Head)
            .map(|s| s.current_hp <= 0)
            .unwrap_or(false)
    }

    /// Sum of non-negative part HP.
    pub fn total_hp(&self) -> i64 {
        self.parts.values().map(|s| s.current_hp.max(0)).sum()
    }

    /// Sum of all part max HP.
    pub fn total_max_hp(&self) -> i64 {
        self.parts.values().map(|s| s.max_hp).sum()
    }

    /// Stat penalties from current injuries.
    pub fn penalties(&self) -> BodyPenalties {
        let left_eye_gone = self
            .parts
            .get(&BodyPart::LeftEye)
            .map(|s| s.current_hp <= 0)
            .unwrap_or(false);
        let right_eye_gone = self
            .parts
            .get(&BodyPart::RightEye)
            .map(|s| s.current_hp <= 0)
            .unwrap_or(false);
        let both_eyes_gone = left_eye_gone && right_eye_gone;

        let mut p = BodyPenalties::default();
        for (part, state) in &self.parts {
            if let Some(sev) = &state.injury {
                p.apply(*part, sev, both_eyes_gone);
            }
            // Negative HP = cursed: ongoing luck/entropy drain.
            if state.current_hp < 0 {
                p.luck -= (-state.current_hp / 5).min(10);
                p.entropy -= 5;
            }
        }
        p
    }

    /// HP drained per combat turn from negative-HP (cursed) parts.
    pub fn curse_drain_per_turn(&self) -> i64 {
        self.parts
            .values()
            .filter(|s| s.current_hp < 0)
            .map(|s| (-s.current_hp / 10).clamp(1, 8))
            .sum()
    }

    /// Penalty (0.0–0.80) to flee success from leg/foot injuries.
    pub fn flee_penalty_pct(&self) -> f64 {
        let penalty: f64 = [
            BodyPart::LeftLeg,
            BodyPart::RightLeg,
            BodyPart::LeftFoot,
            BodyPart::RightFoot,
        ]
        .iter()
        .filter_map(|p| self.parts.get(p).and_then(|s| s.injury.as_ref()))
        .map(|sev| match sev {
            InjurySeverity::Bruised => 0.05,
            InjurySeverity::Fractured => 0.10,
            InjurySeverity::Shattered => 0.20,
            InjurySeverity::Severed => 0.35,
            InjurySeverity::MathematicallyAbsent => 0.50,
        })
        .sum();
        penalty.min(0.80)
    }

    /// One-line summary of the worst injuries (for combat HUD).
    pub fn combat_summary(&self) -> String {
        let mut critical: Vec<String> = self
            .parts
            .iter()
            .filter(|(_, s)| {
                s.current_hp <= 0
                    || matches!(
                        s.injury,
                        Some(InjurySeverity::Shattered)
                            | Some(InjurySeverity::Severed)
                            | Some(InjurySeverity::MathematicallyAbsent)
                    )
            })
            .map(|(p, s)| {
                let label = s.injury.as_ref().map(|i| i.name()).unwrap_or("GONE");
                format!("{}: {}", p.name(), label)
            })
            .collect();
        critical.sort();
        if critical.is_empty() {
            "All parts intact".to_string()
        } else {
            critical.join(" | ")
        }
    }

    /// Full display lines for the character sheet body diagram.
    pub fn display_lines(&self) -> Vec<String> {
        const RESET: &str = "\x1b[0m";
        const DIM: &str = "\x1b[2m";
        const GREEN: &str = "\x1b[32m";
        const YELLOW: &str = "\x1b[33m";
        const RED: &str = "\x1b[31m";
        const MAGENTA: &str = "\x1b[35m";

        let mut lines = Vec::new();
        lines.push(format!(
            "{}  ╔═══ BODY ══════════════════════════════════╗{}",
            MAGENTA, RESET
        ));
        for part in BodyPart::ALL {
            let s = match self.parts.get(part) {
                Some(s) => s,
                None => continue,
            };
            let pct = if s.max_hp > 0 {
                (s.current_hp as f64 / s.max_hp as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let hp_col = if s.current_hp <= 0 {
                MAGENTA
            } else if pct < 0.30 {
                RED
            } else if pct < 0.60 {
                YELLOW
            } else {
                GREEN
            };
            let filled = (pct * 10.0) as usize;
            let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(10 - filled));
            let inj = match &s.injury {
                None => format!("{}ok         {}", GREEN, RESET),
                Some(i) => format!("{}{:<11}{}", i.color_code(), i.name(), RESET),
            };
            let armor_str = if s.armor_defense > 0 {
                format!(" {}[{:+}ARM]{}", DIM, s.armor_defense, RESET)
            } else {
                String::new()
            };
            let curse = if s.current_hp < 0 { " ⚡CURSED" } else { "" };
            lines.push(format!(
                "{}  ║ {:<13}{}{:<12}{}{:>4}/{:<4} {}{}{}",
                MAGENTA,
                part.name(),
                hp_col,
                bar,
                RESET,
                s.current_hp,
                s.max_hp,
                inj,
                armor_str,
                curse
            ));
        }
        lines.push(format!(
            "{}  ╚═══════════════════════════════════════════╝{}",
            MAGENTA, RESET
        ));
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_generates_all_parts() {
        let b = Body::generate(50, 42);
        for part in BodyPart::ALL {
            assert!(b.parts.contains_key(part), "missing {}", part.name());
            let s = &b.parts[part];
            assert!(s.current_hp > 0);
            assert!(s.max_hp >= s.current_hp);
        }
    }

    #[test]
    fn damage_reduces_hp_and_sets_injury() {
        let mut b = Body::generate(50, 99);
        let head_max = b.parts[&BodyPart::Head].max_hp;
        let (dmg, inj) = b.damage_part(BodyPart::Head, head_max / 2 + 1);
        assert!(dmg > 0);
        assert!(inj.is_some());
        assert!(b.parts[&BodyPart::Head].current_hp < head_max);
    }

    #[test]
    fn armor_absorbs_damage() {
        let mut b = Body::generate(50, 7);
        b.parts.get_mut(&BodyPart::Torso).unwrap().armor_defense = 50;
        let (dmg, _) = b.damage_part(BodyPart::Torso, 30);
        assert_eq!(dmg, 0, "armor should absorb all 30 damage");
    }

    #[test]
    fn flee_penalty_nonzero_after_leg_injury() {
        let mut b = Body::generate(50, 55);
        assert_eq!(b.flee_penalty_pct(), 0.0);
        let ll_max = b.parts[&BodyPart::LeftLeg].max_hp;
        b.damage_part(BodyPart::LeftLeg, ll_max);
        assert!(b.flee_penalty_pct() > 0.0);
    }

    #[test]
    fn head_destroyed_triggers_on_zero_hp() {
        let mut b = Body::generate(50, 13);
        assert!(!b.head_destroyed());
        let head_max = b.parts[&BodyPart::Head].max_hp;
        b.damage_part(BodyPart::Head, head_max + 100);
        assert!(b.head_destroyed());
    }
}
