//! Representation of an EPI2ME Desktop analysis record.
//!
//! Oxford Nanopore's EPI2ME Desktop experience is more than a plain results
//! folder: analyses also have GUI-facing metadata such as workflow identity,
//! timestamps, status, and provenance. This module captures that shape so a
//! bundled analysis can be exported from or imported into a Desktop-like
//! environment.

use std::{env, path::PathBuf};

use crate::{
    app_db::{self, validate_db_entry, Epi2MeAnalysis},
    dataframe::get_zero_val,
    epi2me_workflow::clip_relative_path,
    nextflow_log_parser::NextFlowLogs,
    xmanifest::{sha256_digest, FileManifest},
};
use glob::glob;
use polars::frame::DataFrame;
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

/*
impl Default for Epi2meDesktopAnalysis {
    fn default() -> Epi2meDesktopAnalysis {

        Epi2meDesktopAnalysis {
            id: String::from("undefined"),
            path: String::from("undefined"),
            name: String::from("undefined"),
            status: String::from("undefined"),
            workflowRepo: String::from("undefined"),
            workflowUser: String::from("undefined"),
            workflowCommit: String::from("undefined"),
            workflowVersion: String::from("undefined"),
            createdAt: String::from("undefined"),
            updatedAt: String::from("undefined"),
            files: Vec::<FileManifest>::new(),
        }
    }
}
    */

impl Epi2meDesktopAnalysis {
    /// Rehydrates an analysis description from the local Desktop database view.
    ///
    /// This path is used when `epi4you` is bundling an analysis that already
    /// exists in the EPI2ME Desktop environment.
    pub fn from_run_id(runid: &String, polardb: &DataFrame) -> Self {
        if validate_db_entry(runid, polardb) {
            let stacked = app_db::get_db_id_entry(runid, polardb).unwrap();
            // this is obligate pass due to previous validation

            let x = Epi2meDesktopAnalysis {
                id: get_zero_val(&stacked, &String::from("id")),
                path: get_zero_val(&stacked, &String::from("path")),
                name: get_zero_val(&stacked, &String::from("name")),
                status: get_zero_val(&stacked, &String::from("status")),
                workflowRepo: get_zero_val(&stacked, &String::from("workflowRepo")),
                workflowUser: get_zero_val(&stacked, &String::from("workflowUser")),
                workflowCommit: get_zero_val(&stacked, &String::from("workflowCommit")),
                workflowVersion: get_zero_val(&stacked, &String::from("workflowVersion")),
                createdAt: get_zero_val(&stacked, &String::from("createdAt")),
                updatedAt: get_zero_val(&stacked, &String::from("updatedAt")),
                files: Vec::<FileManifest>::new(),
            };
            return x;
        }
        panic!();
    }

    /// Synthesizes a Desktop-style analysis record from a raw CLI Nextflow run.
    ///
    /// This is one of the most important translation points in the project:
    /// there is no native EPI2ME database row here yet, so we infer the fields
    /// that Desktop expects from `nextflow.stdout` and the staged bundle path.
    pub fn init(
        ulid_str: &String,
        source: &PathBuf,
        nextflow_stdout: &String,
        timestamp: &String,
    ) -> Self {
        println!("get_analysis_struct_from_cli");

        let mut log = PathBuf::from(source);
        log.push("nextflow.stdout");

        // println!("{}", nextflow_stdout);

        let nlp = NextFlowLogs::init(nextflow_stdout);
        nlp.test();

        // panic!();

        let name = nlp.get_value("name");
        let pname = nlp.get_value("pname");
        let project = nlp.get_value("project");
        let revision = nlp.get_value("revision");
        let version = nlp.get_value("version");

        let x = Epi2meDesktopAnalysis {
            id: String::from(ulid_str),
            path: String::from(source.to_str().unwrap()),
            name: String::from(name),
            status: String::from("COMPLETED"),
            workflowRepo: pname,
            workflowUser: project,
            workflowCommit: String::from(revision),
            workflowVersion: version,
            createdAt: String::from(timestamp),
            updatedAt: String::from(timestamp),
            files: Vec::<FileManifest>::new(),
        };

        println!("{:?}", x);
        return x;
    }

    /// Converts the export/import model into the database-oriented app model.
    pub fn as_epi2me_analysis(&self) -> Epi2MeAnalysis {
        return Epi2MeAnalysis {
            id: String::from(&self.id),
            path: String::from(&self.path),
            name: String::from(&self.name),
            status: String::from(&self.status),
            workflowRepo: String::from(&self.workflowRepo),
            workflowUser: String::from(&self.workflowUser),
            workflowCommit: String::from(&self.workflowCommit),
            workflowVersion: String::from(&self.workflowVersion),
            createdAt: String::from(&self.createdAt),
            updatedAt: String::from(&self.updatedAt),
        };
    }

    /// Recursively inventories analysis files for bundling.
    ///
    /// Files are recorded relative to `local_prefix` so the tarball can later be
    /// reconstructed without baking absolute workstation paths into the archive.
    pub fn fish_files(&mut self, source: &PathBuf, local_prefix: &PathBuf) {
        let globpat = &source.clone().into_os_string().into_string().unwrap();
        let result = [&globpat, "/**/*.*"].join("");

        // let mut files: Vec<FileManifest> = Vec::new();

        println!("fishing for files at [{}]", result);

        let _ = env::set_current_dir(&globpat);

        for entry in glob(&result).expect("Failed to read glob pattern") {
            if entry.is_ok() {
                let e = entry.unwrap();
                let fname = &e.file_name().and_then(|s| s.to_str());
                if e.is_file() && !fname.unwrap().contains("4u_manifest.json") {
                    // don't yet package the manifest - it'll be added later
                    let fp = e.as_os_str().to_str().unwrap();

                    let mut parent = e.clone();
                    let _ = parent.pop();

                    let relative_path = clip_relative_path(&e, &local_prefix);
                    //println!("{}", &fp);

                    let checksum = sha256_digest(&fp);

                    //println!("file [{}] with checksum [{}]", &fp, &vv);
                    let file_size = e.metadata().unwrap().len();

                    self.files.push(FileManifest {
                        filename: String::from(
                            e.file_name().unwrap().to_os_string().to_str().unwrap(),
                        ),
                        relative_path: String::from(
                            relative_path.clone().to_string_lossy().to_string(),
                        ),
                        size: file_size,
                        md5sum: checksum,
                    })
                }
            }
        }
    }

    /// Returns the current file manifest vector.
    pub fn get_files(&self) -> Vec<FileManifest> {
        return self.files.clone();
    }

    /// Returns the total payload size of all inventoried files.
    pub fn get_files_size(&self) -> u64 {
        let mut size: u64 = 0;
        for file in self.files.clone() {
            size += file.size;
        }
        return size;
    }
}
