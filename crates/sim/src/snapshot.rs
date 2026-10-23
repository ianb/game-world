//! Snapshots: the complete state, versioned, postcard-encoded. The input log is the save game;
//! a snapshot is the quick-load accelerator and the late-joiner mechanism.
use crate::protocol::PROTOCOL_VERSION;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

/// Bumped whenever the state layout changes. Old snapshots are refused, not migrated.
pub const SNAPSHOT_VERSION: u32 = 4;

#[derive(Serialize, Deserialize)]
struct Envelope<S> {
    snapshot_version: u32,
    protocol_version: u32,
    /// The character set's own layout version, so a changed character struct is refused
    /// rather than decoded into the wrong fields.
    layout_version: u32,
    state: S,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    #[error("snapshot version {found} not supported (want {SNAPSHOT_VERSION})")]
    Version { found: u32 },
    #[error("snapshot protocol version {found} not supported (want {PROTOCOL_VERSION})")]
    Protocol { found: u32 },
    #[error("snapshot character layout {found} not supported (want {want})")]
    Layout { found: u32, want: u32 },
    #[error("snapshot bytes malformed: {0}")]
    Malformed(String),
}

#[must_use]
pub fn encode<S: Serialize + Clone>(state: &S, layout_version: u32) -> Vec<u8> {
    let env = Envelope {
        snapshot_version: SNAPSHOT_VERSION,
        protocol_version: PROTOCOL_VERSION,
        layout_version,
        state: state.clone(),
    };
    postcard::to_allocvec(&env).expect("state is always serializable")
}

pub fn decode<S: DeserializeOwned>(bytes: &[u8], layout_version: u32) -> Result<S, SnapshotError> {
    // Read the two version varints first so a layout change reports a version error rather
    // than a malformed-bytes error.
    let (snapshot_version, rest): (u32, &[u8]) =
        postcard::take_from_bytes(bytes).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    if snapshot_version != SNAPSHOT_VERSION {
        return Err(SnapshotError::Version {
            found: snapshot_version,
        });
    }
    let (protocol_version, rest): (u32, &[u8]) =
        postcard::take_from_bytes(rest).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    if protocol_version != PROTOCOL_VERSION {
        return Err(SnapshotError::Protocol {
            found: protocol_version,
        });
    }
    let (found_layout, _): (u32, &[u8]) =
        postcard::take_from_bytes(rest).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    if found_layout != layout_version {
        return Err(SnapshotError::Layout {
            found: found_layout,
            want: layout_version,
        });
    }
    let (env, trailing): (Envelope<S>, &[u8]) =
        postcard::take_from_bytes(bytes).map_err(|e| SnapshotError::Malformed(e.to_string()))?;
    if !trailing.is_empty() {
        return Err(SnapshotError::Malformed(format!(
            "{} trailing bytes",
            trailing.len()
        )));
    }
    Ok(env.state)
}
