//! Chaos dice as a [`rand`] distribution.
//!
//! Already using `rand`? [`ChaosDie`] plugs the chaos pipeline into any
//! `rand::Rng`: the generator picks the seed and input, and the chain of math
//! engines decides the face, exactly as the game does.
//!
//! Chaos dice are deliberately not fair. Measured with the `dice_stats`
//! example over 100,000 rolls of a d20: a 20 comes up 10.2% of the time and
//! a 19 only 1.0% (a fair die gives 5% each). Use them for flavour, not where
//! fairness matters.
//!
//! ```
//! use chaos_rpg_core::dice::ChaosDie;
//! use rand::{rngs::StdRng, Rng, SeedableRng};
//!
//! let mut rng = StdRng::seed_from_u64(7);
//! let d20 = ChaosDie::d20();
//! let faces: Vec<u8> = (0..5).map(|_| rng.sample(&d20)).collect();
//! assert!(faces.iter().all(|f| (1..=20).contains(f)));
//!
//! // Same generator state, same rolls.
//! let mut again = StdRng::seed_from_u64(7);
//! assert_eq!(faces, (0..5).map(|_| again.sample(&d20)).collect::<Vec<u8>>());
//! ```

use crate::chaos_pipeline::{chaos_roll_verbose, ChaosRollResult};
use rand::distributions::Distribution;
use rand::Rng;

/// A die with `sides` faces (1 to `sides`) rolled through the chaos pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChaosDie {
    sides: u8,
}

impl ChaosDie {
    /// A die with `sides` faces; fewer than 2 sides is treated as 2.
    pub fn new(sides: u8) -> Self {
        Self { sides: sides.max(2) }
    }

    /// A six-sided die.
    pub fn d6() -> Self {
        Self::new(6)
    }

    /// A twenty-sided die.
    pub fn d20() -> Self {
        Self::new(20)
    }

    /// Number of faces.
    pub fn sides(&self) -> u8 {
        self.sides
    }

    /// Roll with an explicit pipeline input and seed, returning the face and
    /// the full chain (useful for showing the maths, as the game's combat log
    /// does).
    pub fn roll_verbose(&self, input: f64, seed: u64) -> (u8, ChaosRollResult) {
        let roll = chaos_roll_verbose(input, seed);
        let face = roll.to_range(1, self.sides as i64) as u8;
        (face, roll)
    }
}

impl Distribution<u8> for ChaosDie {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u8 {
        let input: f64 = rng.gen_range(-1.0..=1.0);
        let seed: u64 = rng.gen();
        self.roll_verbose(input, seed).0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn faces_stay_in_range_and_all_appear() {
        let mut rng = StdRng::seed_from_u64(1);
        for sides in [2u8, 6, 20, 100] {
            let die = ChaosDie::new(sides);
            let mut seen = vec![false; sides as usize + 1];
            for _ in 0..20_000 {
                let f = rng.sample(die);
                assert!((1..=sides).contains(&f));
                seen[f as usize] = true;
            }
            // Chaos dice are not fair (see the dice_stats example), but on
            // the small dice every face does come up.
            if sides <= 20 {
                assert!(seen[1..].iter().all(|&s| s), "d{sides}: some face never came up");
            }
        }
        assert_eq!(ChaosDie::new(0).sides(), 2);
    }

    #[test]
    fn verbose_roll_matches_pipeline() {
        let (face, roll) = ChaosDie::d20().roll_verbose(0.5, 666);
        assert_eq!(face, roll.as_d20());
        assert!(!roll.chain.is_empty());
    }
}
