//! Seeded, portable random streams held in the state. One stream per concern so that
//! adding a cosmetic roll never shifts a gameplay stream.
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

/// `SplitMix64` step: derives independent sub-seeds from one master seed. Integer only.
#[must_use]
pub fn split_mix(seed: u64, index: u64) -> u64 {
    let mut z = seed.wrapping_add(index.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Streams {
    /// Schedule jitter: when exactly a villager leaves, small waits.
    pub schedule: Pcg32,
    /// Idle choices: which way to wander, what to look at.
    pub idle: Pcg32,
    /// Social: who approaches whom, exchange outcomes.
    pub social: Pcg32,
    /// Bodies: how fast needs rise, hour to hour.
    pub body: Pcg32,
}

impl Streams {
    #[must_use]
    pub fn from_seed(seed: u64) -> Streams {
        Streams {
            schedule: Pcg32::seed_from_u64(split_mix(seed, 0)),
            idle: Pcg32::seed_from_u64(split_mix(seed, 1)),
            social: Pcg32::seed_from_u64(split_mix(seed, 2)),
            body: Pcg32::seed_from_u64(split_mix(seed, 3)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn streams_are_independent_and_reproducible() {
        let mut a = Streams::from_seed(7);
        let mut b = Streams::from_seed(7);
        let first: Vec<u32> = (0..4).map(|_| a.idle.random_range(0..1000u32)).collect();
        assert_eq!(a.schedule, b.schedule, "unused streams untouched");
        let again: Vec<u32> = (0..4).map(|_| b.idle.random_range(0..1000u32)).collect();
        assert_eq!(first, again);
        assert_ne!(Streams::from_seed(8).idle, Streams::from_seed(7).idle);
    }

    #[test]
    fn known_values_pin_the_algorithm() {
        // If these change, replays recorded with the old algorithm no longer verify.
        let mut s = Streams::from_seed(42);
        let v: Vec<u32> = (0..3).map(|_| s.schedule.random_range(0..100u32)).collect();
        insta::assert_debug_snapshot!(v);
    }
}
