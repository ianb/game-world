//! Loading from a content directory: `zones/*.json`, `buildings/*.json`, `villagers/*.json`.
//! Files are read in sorted name order so ids are assigned deterministically.
use crate::model::{Building, Content, Villager, Zone};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

fn read_all<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>, LoadError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|source| LoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path).map_err(|source| LoadError::Io {
                path: path.clone(),
                source,
            })?;
            serde_json::from_str(&text).map_err(|source| LoadError::Json { path, source })
        })
        .collect()
}

pub fn load_dir(root: &Path) -> Result<Content, LoadError> {
    Ok(Content {
        zones: read_all::<Zone>(&root.join("zones"))?,
        buildings: read_all::<Building>(&root.join("buildings"))?,
        villagers: read_all::<Villager>(&root.join("villagers"))?,
    })
}

/// A whole town from one JSON document, for tests and fixtures.
pub fn load_str(json: &str) -> Result<Content, serde_json::Error> {
    serde_json::from_str(json)
}
