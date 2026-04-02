//! Representation of an EPI2ME Desktop analysis record.
//!
//! Oxford Nanopore's EPI2ME Desktop experience is more than a plain results
//! folder: analyses also have GUI-facing metadata such as workflow identity,
//! timestamps, status, and provenance. This module captures that shape so a
//! bundled analysis can be exported from or imported into a Desktop-like
//! environment.

use std::{env, path::PathBuf};

use crate::{
    app_db::Epi2MeAnalysis,
    epi2me_workflow::clip_relative_path,
    nextflow_log_parser::NextFlowLogs,
    xmanifest::{sha256_digest, FileManifest},
};
use glob::glob;
use serde::{Deserialize, Serialize};

/// Serializable model of one Desktop analysis entry.
///
/// The field names intentionally follow the application's existing mixed-case
/// schema so the JSON and database-adjacent layers can interoperate without a
/// translation table.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Epi2meDesktopAnalysis {
    /// Stable analysis identifier used by the GUI and local database.
    pub id: String,
    /// On-disk location of the analysis root.
    pub path: String,
    /// Human-readable Nextflow run name shown in EPI2ME Desktop.
    pub name: String,
    /// Final workflow status as expected by the Desktop UI.
    pub status: String,
    /// Workflow repository name, for example `wf-human-variation`.
    pub workflowRepo: String,
    /// Workflow owner or project namespace, often `epi2me-labs`.
    pub workflowUser: String,
    /// Workflow revision / commit recorded for the run.
    pub workflowCommit: String,
    /// User-facing workflow version derived from logs or local metadata.
    pub workflowVersion: String,
    /// Analysis creation timestamp.
    pub createdAt: String,
    /// Analysis update timestamp.
    pub updatedAt: String,
    /// File inventory used when bundling this analysis.
    pub files: Vec<FileManifest>,
}

impl Epi2meDesktopAnalysis {
    /// Synthesizes a Desktop-style analysis record from a raw CLI Nextflow run.
    pub fn init(
        ulid_str: &String,
        source: &PathBuf,
        nextflow_stdout: &String,
        timestamp: &String,
    ) -> Self {
        println!("get_analysis_struct_from_cli");

        let nlp = NextFlowLogs::init(nextflow_stdout);
        nlp.test();

        Epi2meDesktopAnalysis {
            id: String::from(ulid_str),
            path: String::from(source.to_str().unwrap()),
            name: nlp.get_value("name"),
            status: String::from("COMPLETED"),
            workflowRepo: nlp.get_value("pname"),
            workflowUser: nlp.get_value("project"),
            workflowCommit: nlp.get_value("revision"),
            workflowVersion: nlp.get_value("version"),
            createdAt: String::from(timestamp),
            updatedAt: String::from(timestamp),
            files: Vec::<FileManifest>::new(),
        }
    }

    /// Converts the export/import model into the database-oriented app model.
    pub fn as_epi2me_analysis(&self) -> Epi2MeAnalysis {
        Epi2MeAnalysis {
            id: self.id.clone(),
            path: self.path.clone(),
            name: self.name.clone(),
            status: self.status.clone(),
            workflowRepo: self.workflowRepo.clone(),
            workflowUser: self.workflowUser.clone(),
            workflowCommit: self.workflowCommit.clone(),
            workflowVersion: self.workflowVersion.clone(),
            createdAt: self.createdAt.clone(),
            updatedAt: self.updatedAt.clone(),
        }
    }

    /// Recursively inventories analysis files for bundling.
    pub fn fish_files(&mut self, source: &PathBuf, local_prefix: &PathBuf) {
        let globpat = &source.clone().into_os_string().into_string().unwrap();
        let result = [&globpat, "/**/*.*"].join("");

        println!("fishing for files at [{}]", result);

        let _ = env::set_current_dir(globpat);

        for entry in glob(&result).expect("Failed to read glob pattern") {
            if let Ok(e) = entry {
                let fname = &e.file_name().and_then(|s| s.to_str());
                if e.is_file() && !fname.unwrap().contains("4u_manifest.json") {
                    let fp = e.as_os_str().to_str().unwrap();
                    let relative_path = clip_relative_path(&e, local_prefix);
                    let checksum = sha256_digest(fp);
                    let file_size = e.metadata().unwrap().len();

                    self.files.push(FileManifest {
                        filename: String::from(
                            e.file_name().unwrap().to_os_string().to_str().unwrap(),
                        ),
                        relative_path: relative_path.to_string_lossy().to_string(),
                        size: file_size,
                        md5sum: checksum,
                    });
                }
            }
        }
    }

    /// Returns the current file manifest vector.
    pub fn get_files(&self) -> Vec<FileManifest> {
        self.files.clone()
    }

    /// Returns the total payload size of all inventoried files.
    pub fn get_files_size(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }
}
