//! State hashes for desync detection: serialize each subsystem with postcard and hash the
//! bytes with xxh3. Anything not in the snapshot is not in the hash, by construction.
use serde::Serialize;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HashParts {
    pub tick: u64,
    pub rng: u64,
    pub queue: u64,
    pub world: u64,
    pub inputs: u64,
    pub combined: u64,
}

impl HashParts {
    #[must_use]
    pub fn combine(tick: u64, rng: u64, queue: u64, world: u64, inputs: u64) -> HashParts {
        let mut bytes = Vec::with_capacity(40);
        for p in [tick, rng, queue, world, inputs] {
            bytes.extend_from_slice(&p.to_le_bytes());
        }
        HashParts {
            tick,
            rng,
            queue,
            world,
            inputs,
            combined: xxh3_64(&bytes),
        }
    }

    #[must_use]
    pub fn hex(&self) -> String {
        format!("{:016x}", self.combined)
    }

    /// Names the first differing part, for bisecting a desync.
    #[must_use]
    pub fn first_difference(&self, other: &HashParts) -> Option<&'static str> {
        [
            ("tick", self.tick, other.tick),
            ("rng", self.rng, other.rng),
            ("queue", self.queue, other.queue),
            ("world", self.world, other.world),
            ("inputs", self.inputs, other.inputs),
        ]
        .into_iter()
        .find(|(_, a, b)| a != b)
        .map(|(name, _, _)| name)
    }
}

/// Hash of the postcard encoding of any serializable value.
pub fn hash_of<T: Serialize>(value: &T) -> u64 {
    let bytes = postcard::to_allocvec(value).expect("state is always serializable");
    xxh3_64(&bytes)
}
