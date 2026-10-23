//! Snapshots: the complete state, versioned, postcard-encoded. The input log is the save game;
//! a snapshot is the quick-load accelerator and the late-joiner mechanism.
use crate::protocol::PROTOCOL_VERSION;
use crate::sim::SimState;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Bumped whenever `SimState`'s layout changes. Old snapshots are refused, not migrated.
pub const SNAPSHOT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Envelope {
    snapshot_version: u32,
    protocol_version: u32,
    state: SimState,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    #[error("snapshot version {found} not supported (want {SNAPSHOT_VERSION})")]
    Version { found: u32 },
    #[error("snapshot protocol version {found} not supported (want {PROTOCOL_VERSION})")]
    Protocol { found: u32 },
    #[error("snapshot bytes malformed: {0}")]
    Malformed(String),
}

#[must_use]
pub fn encode(state: &SimState) -> Vec<u8> {
    let env = Envelope {
        snapshot_version: SNAPSHOT_VERSION,
        protocol_version: PROTOCOL_VERSION,
        state: state.clone(),
    };
    postcard::to_allocvec(&env).expect("state is always serializable")
}

pub fn decode(bytes: &[u8]) -> Result<SimState, SnapshotError> {
    let env: Envelope =
        postcard::from_bytes(bytes).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    if env.snapshot_version != SNAPSHOT_VERSION {
        return Err(SnapshotError::Version {
            found: env.snapshot_version,
        });
    }
    if env.protocol_version != PROTOCOL_VERSION {
        return Err(SnapshotError::Protocol {
            found: env.protocol_version,
        });
    }
    Ok(env.state)
}
