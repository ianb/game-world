//! Snapshots: the complete state, versioned, postcard-encoded. The input log is the save game;
//! a snapshot is the quick-load accelerator and the late-joiner mechanism.
use crate::protocol::PROTOCOL_VERSION;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

/// Bumped whenever the state layout changes. Old snapshots are refused, not migrated.
pub const SNAPSHOT_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
struct Envelope<S> {
    snapshot_version: u32,
    protocol_version: u32,
    state: S,
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
pub fn encode<S: Serialize + Clone>(state: &S) -> Vec<u8> {
    let env = Envelope {
        snapshot_version: SNAPSHOT_VERSION,
        protocol_version: PROTOCOL_VERSION,
        state: state.clone(),
    };
    postcard::to_allocvec(&env).expect("state is always serializable")
}

pub fn decode<S: DeserializeOwned>(bytes: &[u8]) -> Result<S, SnapshotError> {
    // Read the two version varints first so a layout change reports a version error rather
    // than a malformed-bytes error.
    let (snapshot_version, rest): (u32, &[u8]) =
        postcard::take_from_bytes(bytes).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    if snapshot_version != SNAPSHOT_VERSION {
        return Err(SnapshotError::Version {
            found: snapshot_version,
        });
    }
    let (protocol_version, _): (u32, &[u8]) =
        postcard::take_from_bytes(rest).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    if protocol_version != PROTOCOL_VERSION {
        return Err(SnapshotError::Protocol {
            found: protocol_version,
        });
    }
    let env: Envelope<S> =
        postcard::from_bytes(bytes).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    Ok(env.state)
}
