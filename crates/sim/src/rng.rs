//! Seeded, portable random streams held in the state. One stream per concern and per
//! villager, so adding a roll in one villager's code never shifts anyone else's.
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

/// Two streams per villager: `mind` for that villager's character code, `body` for its
/// needs. Seeded from the villager's id rather than its position, so adding or reordering
/// villagers never shifts anyone else's rolls.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VillagerStreams {
    pub mind: Pcg32,
    pub body: Pcg32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Streams {
    /// Sim-level social rolls (who approaches whom). Reserved.
    pub social: Pcg32,
    /// Indexed by villager.
    pub villagers: Vec<VillagerStreams>,
    /// Every animal's wandering and laying.
    pub animals: Pcg32,
}

impl Streams {
    /// `ids` are the villagers' content ids, in villager order.
    #[must_use]
    pub fn from_seed<S: AsRef<str>>(seed: u64, ids: &[S]) -> Streams {
        let villagers = ids
            .iter()
            .map(|id| {
                let key = xxhash_rust::xxh3::xxh3_64(id.as_ref().as_bytes());
                VillagerStreams {
                    mind: Pcg32::seed_from_u64(split_mix(seed ^ key, 0)),
                    body: Pcg32::seed_from_u64(split_mix(seed ^ key, 1)),
                }
            })
            .collect();
        Streams {
            social: Pcg32::seed_from_u64(split_mix(seed, 2)),
            villagers,
            animals: Pcg32::seed_from_u64(split_mix(seed, 3)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    #[test]
    fn streams_are_independent_and_reproducible() {
        let mut a = Streams::from_seed(7, &["mara", "tom"]);
        let b = Streams::from_seed(7, &["mara", "tom"]);
        let first: Vec<u32> = (0..4)
            .map(|_| a.villagers[0].mind.random_range(0..1000u32))
            .collect();
        assert_eq!(a.villagers[1], b.villagers[1], "other villagers untouched");
        assert_eq!(a.villagers[0].body, b.villagers[0].body, "body untouched");
        let mut c = Streams::from_seed(7, &["mara", "tom"]);
        let again: Vec<u32> = (0..4)
            .map(|_| c.villagers[0].mind.random_range(0..1000u32))
            .collect();
        assert_eq!(first, again);
        assert_ne!(
            Streams::from_seed(8, &["mara"]).villagers[0],
            Streams::from_seed(7, &["mara"]).villagers[0]
        );
    }

    #[test]
    fn a_villager_keeps_their_streams_when_others_are_added() {
        let two = Streams::from_seed(7, &["mara", "tom"]);
        let three = Streams::from_seed(7, &["anna", "mara", "tom"]);
        assert_eq!(two.villagers[0], three.villagers[1]);
        assert_eq!(two.villagers[1], three.villagers[2]);
    }

    #[test]
    fn known_values_pin_the_algorithm() {
        // If these change, replays recorded with the old algorithm no longer verify.
        let mut s = Streams::from_seed(42, &["mara"]);
        let v: Vec<u32> = (0..3)
            .map(|_| s.villagers[0].mind.random_range(0..100u32))
            .collect();
        insta::assert_debug_snapshot!(v);
    }
}
