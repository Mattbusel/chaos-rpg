//! Extended status-effect system for CHAOS RPG.
//!
//! Provides a self-contained [`StatusRegistry`] that tracks all active
//! buffs and debuffs on a combatant, handles per-turn tick-down, stacking
//! rules, and conflict resolution (e.g. *Blessed* cancels *Cursed*).
//!
//! ## Design
//!
//! * Every effect has a **kind** ([`EffectKind`]) and a remaining **duration**
//!   in turns.
//! * Effects are stored in a `Vec` and searched by kind for fast access.
//! * On each turn, call [`StatusRegistry::tick`]; it decrements all durations
//!   and removes expired effects, returning a list of [`TickEvent`]s that the
//!   combat log can display.
//! * Effects can be **queried** for stat modifiers that the combat system
//!   applies when computing damage, defence, and other derived values.
//!
//! ## Usage
//!
//! ```rust
//! use chaos_rpg::status_effects::{StatusRegistry, EffectKind};
//!
//! let mut reg = StatusRegistry::new();
//! reg.apply(EffectKind::Burning, 3);
//! reg.apply(EffectKind::Enraged, 2);
//!
//! // Each combat turn:
//! let events = reg.tick(/* base_hp */ 100);
//! for ev in events {
//!     println!("{}", ev.description);
//! }
//!
//! println!("damage_mult: {:.2}", reg.damage_multiplier());
//! ```

use serde::{Deserialize, Serialize};

// ─── EFFECT KINDS ────────────────────────────────────────────────────────────

/// All possible status-effect kinds that can be applied to a combatant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectKind {
    // ── Buffs ─────────────────────────────────────────────────────────────
    /// +30% outgoing damage for duration.
    Enraged,
    /// +20% incoming damage reduction.
    Blessed,
    /// Regenerate `regen_hp` HP per turn.
    Regenerating,
    /// +25% crit chance modifier.
    FocusedAim,
    /// Next attack ignores all defence reductions.
    ArmorPierce,
    /// +40% speed (extra chaos roll on attack).
    Hasted,

    // ── Debuffs ───────────────────────────────────────────────────────────
    /// Lose `dot_dmg` HP per turn (fire damage).
    Burning,
    /// Lose `dot_dmg` HP per turn (poison).
    Poisoned,
    /// Cannot act for the duration (skips turn).
    Stunned,
    /// -30% outgoing damage (fear lowers force).
    Feared,
    /// -25% outgoing damage (weakened muscles).
    Weakened,
    /// -20% incoming damage reduction lost (armour crumbles).
    Cursed,
    /// Attack rolls are halved in effectiveness.
    Blinded,

    // ── Mixed ─────────────────────────────────────────────────────────────
    /// Double damage dealt AND double damage received (berserk mode).
    Berserk,
    /// Chaos is fully amplified — extreme random modifier each turn.
    ChaosCharged,
}

impl EffectKind {
    pub fn name(self) -> &'static str {
        match self {
            EffectKind::Enraged => "ENRAGED",
            EffectKind::Blessed => "BLESSED",
            EffectKind::Regenerating => "REGENERATING",
            EffectKind::FocusedAim => "FOCUSED AIM",
            EffectKind::ArmorPierce => "ARMOR PIERCE",
            EffectKind::Hasted => "HASTED",
            EffectKind::Burning => "BURNING",
            EffectKind::Poisoned => "POISONED",
            EffectKind::Stunned => "STUNNED",
            EffectKind::Feared => "FEARED",
            EffectKind::Weakened => "WEAKENED",
            EffectKind::Cursed => "CURSED",
            EffectKind::Blinded => "BLINDED",
            EffectKind::Berserk => "BERSERK",
            EffectKind::ChaosCharged => "CHAOS CHARGED",
        }
    }

    pub fn is_buff(self) -> bool {
        matches!(
            self,
            EffectKind::Enraged
                | EffectKind::Blessed
                | EffectKind::Regenerating
                | EffectKind::FocusedAim
                | EffectKind::ArmorPierce
                | EffectKind::Hasted
        )
    }

    pub fn is_debuff(self) -> bool {
        matches!(
            self,
            EffectKind::Burning
                | EffectKind::Poisoned
                | EffectKind::Stunned
                | EffectKind::Feared
                | EffectKind::Weakened
                | EffectKind::Cursed
                | EffectKind::Blinded
        )
    }

    /// The effect that cancels this one, if any.
    pub fn antidote(self) -> Option<EffectKind> {
        match self {
            EffectKind::Burning => Some(EffectKind::Blessed),
            EffectKind::Poisoned => Some(EffectKind::Regenerating),
            EffectKind::Cursed => Some(EffectKind::Blessed),
            EffectKind::Feared => Some(EffectKind::Enraged),
            EffectKind::Weakened => Some(EffectKind::Enraged),
            EffectKind::Blinded => Some(EffectKind::FocusedAim),
            EffectKind::Stunned => Some(EffectKind::Hasted),
            _ => None,
        }
    }
}

// ─── ACTIVE EFFECT ───────────────────────────────────────────────────────────

/// A single active status effect with its remaining duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub kind: EffectKind,
    /// Remaining turns before the effect expires. `u32::MAX` = permanent.
    pub turns_remaining: u32,
    /// Intensity (scales DoT damage, regen amount, etc.).
    pub intensity: i64,
}

impl ActiveEffect {
    fn new(kind: EffectKind, turns: u32) -> Self {
        let intensity = match kind {
            EffectKind::Burning | EffectKind::Poisoned => 8,
            EffectKind::Regenerating => 10,
            _ => 0,
        };
        Self { kind, turns_remaining: turns, intensity }
    }
}

// ─── TICK EVENT ──────────────────────────────────────────────────────────────

/// An event produced by [`StatusRegistry::tick`] — suitable for the combat log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickEvent {
    pub kind: EffectKind,
    pub event_type: TickEventType,
    pub description: String,
    /// HP delta (positive = gained, negative = lost) caused by this tick.
    pub hp_delta: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TickEventType {
    DotDamage,
    HotHeal,
    Expired,
    Ongoing,
}

// ─── REGISTRY ────────────────────────────────────────────────────────────────

/// Manages all active status effects for a single combatant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusRegistry {
    effects: Vec<ActiveEffect>,
}

impl StatusRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Application ──────────────────────────────────────────────────────────

    /// Apply a new effect. If the same kind already exists, refresh / extend
    /// the duration to the maximum of old and new. Returns `true` if the
    /// effect was added or extended; `false` if it was cancelled by an antidote.
    pub fn apply(&mut self, kind: EffectKind, turns: u32) -> bool {
        // Check if an antidote already counters this effect.
        if let Some(anti) = kind.antidote() {
            if self.has(anti) {
                // Counter-application: remove the antidote and don't add.
                self.remove(anti);
                return false;
            }
        }
        // Check if this new effect cancels an existing opposite.
        // e.g. Blessed cancels Cursed and Burning.
        let opposites: Vec<EffectKind> = self
            .effects
            .iter()
            .filter(|e| e.kind.antidote() == Some(kind))
            .map(|e| e.kind)
            .collect();
        for opp in opposites {
            self.remove(opp);
        }

        // Extend or add.
        if let Some(existing) = self.effects.iter_mut().find(|e| e.kind == kind) {
            existing.turns_remaining = existing.turns_remaining.max(turns);
        } else {
            self.effects.push(ActiveEffect::new(kind, turns));
        }
        true
    }

    /// Apply an effect with a custom intensity (for scaled DoT/HoT).
    pub fn apply_with_intensity(&mut self, kind: EffectKind, turns: u32, intensity: i64) -> bool {
        if self.apply(kind, turns) {
            if let Some(e) = self.effects.iter_mut().find(|e| e.kind == kind) {
                e.intensity = intensity;
            }
            true
        } else {
            false
        }
    }

    /// Remove all stacks of the given effect kind.
    pub fn remove(&mut self, kind: EffectKind) {
        self.effects.retain(|e| e.kind != kind);
    }

    /// Clear every active effect.
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    // ── Query ─────────────────────────────────────────────────────────────────

    /// True if the given effect kind is currently active.
    pub fn has(&self, kind: EffectKind) -> bool {
        self.effects.iter().any(|e| e.kind == kind)
    }

    /// Remaining turns for the given kind, or 0 if not active.
    pub fn turns_remaining(&self, kind: EffectKind) -> u32 {
        self.effects
            .iter()
            .find(|e| e.kind == kind)
            .map(|e| e.turns_remaining)
            .unwrap_or(0)
    }

    /// Slice of all currently active effects (for UI display).
    pub fn active(&self) -> &[ActiveEffect] {
        &self.effects
    }

    /// Number of active effects.
    pub fn count(&self) -> usize {
        self.effects.len()
    }

    // ── Derived combat modifiers ──────────────────────────────────────────────

    /// Multiplier applied to outgoing damage (product of all relevant effects).
    pub fn damage_multiplier(&self) -> f64 {
        let mut mult = 1.0_f64;
        for e in &self.effects {
            mult *= match e.kind {
                EffectKind::Enraged => 1.30,
                EffectKind::Feared => 0.70,
                EffectKind::Weakened => 0.75,
                EffectKind::Berserk => 2.00,
                EffectKind::ChaosCharged => 1.50,
                EffectKind::Blinded => 0.60,
                _ => 1.0,
            };
        }
        mult
    }

    /// Multiplier applied to incoming damage (< 1.0 = damage reduction).
    pub fn damage_taken_multiplier(&self) -> f64 {
        let mut mult = 1.0_f64;
        for e in &self.effects {
            mult *= match e.kind {
                EffectKind::Blessed => 0.80,
                EffectKind::Cursed => 1.20,
                EffectKind::Berserk => 2.00,
                _ => 1.0,
            };
        }
        mult
    }

    /// Crit-chance additive modifier (e.g. 0.25 = +25%).
    pub fn crit_chance_bonus(&self) -> f64 {
        let mut bonus = 0.0_f64;
        if self.has(EffectKind::FocusedAim) {
            bonus += 0.25;
        }
        if self.has(EffectKind::Hasted) {
            bonus += 0.10;
        }
        if self.has(EffectKind::ChaosCharged) {
            bonus += 0.20;
        }
        bonus
    }

    /// True when the combatant is unable to act this turn.
    pub fn is_stunned(&self) -> bool {
        self.has(EffectKind::Stunned)
    }

    /// True when the next attack should ignore armour.
    pub fn has_armor_pierce(&self) -> bool {
        self.has(EffectKind::ArmorPierce)
    }

    // ── Tick ─────────────────────────────────────────────────────────────────

    /// Advance one combat turn. Applies DoT/HoT, decrements all durations,
    /// removes expired effects.  Returns events for the combat log.
    ///
    /// `base_hp` is used to clamp DoT so it cannot kill outright (leaves 1 HP).
    pub fn tick(&mut self, current_hp: i64) -> Vec<TickEvent> {
        let mut events: Vec<TickEvent> = Vec::new();
        let mut hp_running = current_hp;

        // First pass: DoT / HoT events.
        for e in self.effects.iter() {
            match e.kind {
                EffectKind::Burning | EffectKind::Poisoned => {
                    let dmg = (e.intensity).min(hp_running - 1).max(0);
                    hp_running -= dmg;
                    events.push(TickEvent {
                        kind: e.kind,
                        event_type: TickEventType::DotDamage,
                        description: format!(
                            "{} burns for {} damage! ({} turns left)",
                            e.kind.name(),
                            dmg,
                            e.turns_remaining.saturating_sub(1)
                        ),
                        hp_delta: -dmg,
                    });
                }
                EffectKind::Regenerating => {
                    events.push(TickEvent {
                        kind: e.kind,
                        event_type: TickEventType::HotHeal,
                        description: format!(
                            "REGENERATING heals {} HP. ({} turns left)",
                            e.intensity,
                            e.turns_remaining.saturating_sub(1)
                        ),
                        hp_delta: e.intensity,
                    });
                }
                _ => {}
            }
        }

        // Second pass: decrement and collect expired.
        let mut expired: Vec<EffectKind> = Vec::new();
        for e in self.effects.iter_mut() {
            if e.turns_remaining == u32::MAX {
                continue; // permanent
            }
            if e.turns_remaining > 0 {
                e.turns_remaining -= 1;
            }
            if e.turns_remaining == 0 {
                expired.push(e.kind);
            }
        }
        for kind in &expired {
            events.push(TickEvent {
                kind: *kind,
                event_type: TickEventType::Expired,
                description: format!("{} has worn off.", kind.name()),
                hp_delta: 0,
            });
        }

        // Remove expired.
        self.effects.retain(|e| e.turns_remaining > 0 || e.turns_remaining == u32::MAX);

        // Remove one-shot effects that fire only once.
        self.effects.retain(|e| e.kind != EffectKind::ArmorPierce || e.turns_remaining > 0);

        events
    }

    /// Render a compact status bar string, e.g. `[ENRAGED 3] [BURNING 2]`.
    pub fn status_bar(&self) -> String {
        if self.effects.is_empty() {
            return String::new();
        }
        self.effects
            .iter()
            .map(|e| {
                if e.turns_remaining == u32::MAX {
                    format!("[{}∞]", e.kind.name())
                } else {
                    format!("[{} {}]", e.kind.name(), e.turns_remaining)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ─── HELPER: build from legacy StatusEffect ──────────────────────────────────

/// Convert from the `character::StatusEffect` enum to the new registry.
/// This allows incremental migration while keeping the old interface working.
impl StatusRegistry {
    pub fn from_legacy_statuses(statuses: &[crate::character::StatusEffect]) -> Self {
        let mut reg = Self::new();
        for s in statuses {
            match s {
                crate::character::StatusEffect::Enraged(t) => {
                    reg.apply(EffectKind::Enraged, *t as u32);
                }
                crate::character::StatusEffect::Blessed(t) => {
                    reg.apply(EffectKind::Blessed, *t as u32);
                }
                crate::character::StatusEffect::Burning(t) => {
                    reg.apply(EffectKind::Burning, *t as u32);
                }
                crate::character::StatusEffect::Poisoned(t) => {
                    reg.apply(EffectKind::Poisoned, *t as u32);
                }
                crate::character::StatusEffect::Stunned(t) => {
                    reg.apply(EffectKind::Stunned, *t as u32);
                }
                _ => {} // Other effects not yet mirrored in the registry
            }
        }
        reg
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_and_has() {
        let mut reg = StatusRegistry::new();
        assert!(!reg.has(EffectKind::Burning));
        reg.apply(EffectKind::Burning, 3);
        assert!(reg.has(EffectKind::Burning));
    }

    #[test]
    fn tick_decrements_duration() {
        let mut reg = StatusRegistry::new();
        reg.apply(EffectKind::Feared, 2);
        reg.tick(100);
        assert_eq!(reg.turns_remaining(EffectKind::Feared), 1);
        reg.tick(100);
        assert!(!reg.has(EffectKind::Feared));
    }

    #[test]
    fn dot_damage_emitted_and_hp_not_negative() {
        let mut reg = StatusRegistry::new();
        reg.apply_with_intensity(EffectKind::Burning, 3, 50);
        let events = reg.tick(10);
        let dot = events.iter().find(|e| e.event_type == TickEventType::DotDamage);
        assert!(dot.is_some());
        // hp delta should not exceed hp-1 (leaves 1 HP)
        let delta = dot.unwrap().hp_delta;
        assert!(delta >= -(10 - 1), "DoT should leave at least 1 HP");
    }

    #[test]
    fn antidote_cancels_debuff() {
        let mut reg = StatusRegistry::new();
        reg.apply(EffectKind::Burning, 3);
        // Blessed is the antidote for Burning
        reg.apply(EffectKind::Blessed, 2);
        // Burning should be cancelled
        assert!(!reg.has(EffectKind::Burning));
    }

    #[test]
    fn damage_multiplier_stacks() {
        let mut reg = StatusRegistry::new();
        reg.apply(EffectKind::Enraged, 5);
        reg.apply(EffectKind::Berserk, 2);
        let mult = reg.damage_multiplier();
        // 1.30 * 2.00 = 2.60
        assert!((mult - 2.60).abs() < 0.001);
    }

    #[test]
    fn status_bar_renders() {
        let mut reg = StatusRegistry::new();
        reg.apply(EffectKind::Hasted, 4);
        reg.apply(EffectKind::Poisoned, 2);
        let bar = reg.status_bar();
        assert!(bar.contains("HASTED"));
        assert!(bar.contains("POISONED"));
    }

    #[test]
    fn remove_clears_kind() {
        let mut reg = StatusRegistry::new();
        reg.apply(EffectKind::Stunned, 5);
        reg.remove(EffectKind::Stunned);
        assert!(!reg.has(EffectKind::Stunned));
    }

    #[test]
    fn extend_duration_on_re_apply() {
        let mut reg = StatusRegistry::new();
        reg.apply(EffectKind::Enraged, 2);
        reg.apply(EffectKind::Enraged, 5); // should extend to 5
        assert_eq!(reg.turns_remaining(EffectKind::Enraged), 5);
    }
}
