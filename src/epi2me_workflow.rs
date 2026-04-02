//! Representation of an installed EPI2ME workflow tree.
//!
//! EPI2ME Desktop distributes workflows as versioned filesystem payloads.
//! `epi4you` can package those workflow assets directly so they can travel with
//! an analysis archive or be reinstalled elsewhere.

use crate::xmanifest::FileManifest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Serializable description of one installed workflow and its files.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Epi2meWorkflow {
    /// Workflow namespace / project, typically something like `epi2me-labs`.
    pub project: String,
    /// Workflow name under that namespace.
    pub name: String,
    /// Version string associated with the installed workflow.
    pub version: String,
    /// Files that make up the workflow installation tree.
    pub files: Vec<FileManifest>,
}

/// Returns the relative parent directory that should contain a given file.
pub fn clip_relative_path(e: &PathBuf, local_prefix: &PathBuf) -> PathBuf {
    let mut relative_path = get_relative_path(e, local_prefix);
    let _ = relative_path.pop();
    relative_path
}

/// Returns the path of `e` relative to the supplied EPI2ME installation root.
pub fn get_relative_path(e: &PathBuf, local_prefix: &PathBuf) -> PathBuf {
    PathBuf::from(e.strip_prefix(local_prefix).unwrap())
}
