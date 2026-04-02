//! High-level orchestration for converting CLI Nextflow runs into `.2me`
//! archives.
//!
//! EPI2ME Desktop workflows are built on top of Nextflow, but Desktop stores
//! additional metadata and presentation files around the raw workflow output.
//! This module is the adapter that starts from a plain `nextflow log` view and
//! progressively reconstructs enough EPI2ME-shaped state for import.

use std::{fs, io::Cursor, path::PathBuf, process::Command};

use ulid::Ulid;
use walkdir::WalkDir;

use crate::{
    bundle,
    dataframe::{self, nextflow_vec_to_df},
    epi4you_errors::Epi4youError,
    nextflow::{
        nextflow_analysis::NextflowAnalysis,
        nextflow_log_item::{NxfLogItem, Row},
    },
    tempdir::TempDir,
};

/// Snapshot of a directory that contains one or more local Nextflow runs.
///
/// The struct owns:
///
/// - the directory we will inspect,
/// - the resolved `nextflow` binary used to inspect it, and
/// - parsed `nextflow log` rows that represent successful candidate runs.
pub struct NextFlowResultFolder {
    folder: PathBuf,
    nxf_bin: PathBuf,
    vec: Vec<NxfLogItem>,
}

impl NextFlowResultFolder {
    /// Builds a run index for a Nextflow working directory.
    ///
    /// In the broader project, this is the entry point for "take a CLI run and
    /// make it look enough like an EPI2ME Desktop run that we can bundle it".
    pub fn init(folder: PathBuf, nxf_bin: Option<String>) -> Result<Self, Epi4youError> {
        if !folder.exists() {
            return Err(Epi4youError::RequiredPathMissing(folder));
        } else if folder.is_file() {
            return Err(Epi4youError::FileFoundWhenFolderExpected(folder));
        }

        let mut folder = NextFlowResultFolder {
            folder,
            nxf_bin: NextFlowResultFolder::get_nextflow_path(nxf_bin)?,
            vec: Vec::<NxfLogItem>::new(),
        };
        folder.parse_nextflow_folder()?;

        Ok(folder)
    }

    /// Resolves the `nextflow` executable that should be used for discovery.
    ///
    /// A caller may pass an explicit path, but in normal workstation usage we
    /// fall back to `which nextflow`.
    fn get_nextflow_path(nxf_bin: Option<String>) -> Result<PathBuf, Epi4youError> {
        log::info!("getting nextflow path ...");

        let mut nextflow_bin: Option<PathBuf> = None;

        if let Some(nxf_bin) = nxf_bin {
            let x = PathBuf::from(nxf_bin);
            if x.exists() && x.is_file() {
                nextflow_bin = Some(x);
            } else if x.exists() && x.is_dir() {
                return Err(Epi4youError::FolderFoundWhenFileExpected(x));
            }
        } else {
            let output = Command::new("which")
                .arg("nextflow")
                .output()
                .map_err(|_| Epi4youError::FailedToRunCommand(String::from("which nextflow")))?;

            let mut s = String::from_utf8_lossy(&output.stdout).into_owned();
            if s.ends_with('\n') {
                s.pop();
                if s.ends_with('\r') {
                    s.pop();
                }
            }
            if s.len() > 0 {
                log::debug!("nextflow candidate at [{}]", s);
                let x = PathBuf::from(&s);
                if x.exists() && x.is_file() {
                    nextflow_bin = Some(x);
                } else if x.exists() && x.is_dir() {
                    return Err(Epi4youError::FolderFoundWhenFileExpected(x));
                }
            }
        }

        if let Some(nextflow_bin) = nextflow_bin {
            log::info!("Using nxf_bin found at [{:?}]", &nextflow_bin);
            Ok(nextflow_bin)
        } else {
            log::error!("unable to resolve a functional location for nextflow!");
            Err(Epi4youError::UnableToLocateNextflowBinary)
        }
    }

    /// Executes `nextflow log` and retains only successful run records.
    ///
    /// The resulting in-memory index is what powers `--list` and `--runid`
    /// selection in the CLI capture flow.
    fn parse_nextflow_folder(&mut self) -> Result<(), Epi4youError> {
        log::info!(
            "Looking for nxf artifacts at [{}]",
            &self.folder.to_string_lossy()
        );

        let output = Command::new(&self.nxf_bin)
            .current_dir(&self.folder)
            .arg("log")
            .output()
            .map_err(|_| Epi4youError::FailedToRunCommand(String::from("nextflow log")))?;

        if !output.status.success() {
            return Err(Epi4youError::FailedToRunCommand(String::from(
                "nextflow log",
            )));
        }

        let file = Cursor::new(output.stdout);
        let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(file);

        for record in rdr.records().flatten() {
            if let Ok(row) = record.deserialize::<Row>(None) {
                let item = NxfLogItem::init(row.clone())?;
                if row.get_status().trim() == "OK" {
                    self.vec.push(item);
                }
            }
        }

        Ok(())
    }

    /// Prints the discovered successful runs in tabular form.
    pub fn list_runs(&self) {
        let df = nextflow_vec_to_df(self.vec.clone());
        dataframe::print_polars_df(&df);
    }

    /// Finds one parsed run by its Nextflow run name.
    ///
    /// `epi4you` uses the Nextflow `run_name` as the CLI-facing identifier for
    /// packaging a historical run into a Desktop-compatible bundle.
    pub fn verify_cli_entity(&self, runid: String) -> Result<NxfLogItem, Epi4youError> {
        self.vec
            .iter()
            .find(|entry| entry.run_name.trim() == runid)
            .cloned()
            .ok_or(Epi4youError::SpecifiedNextflowRunNotFound(runid))
    }

    /// Bundles one selected CLI run as a `.2me` archive.
    ///
    /// Conceptually this method performs four translations:
    ///
    /// 1. locate the real analysis output directory,
    /// 2. distill the Nextflow log into EPI2ME-like helper files,
    /// 3. stage output files into a temporary EPI2ME-style layout, and
    /// 4. delegate final manifest/tar creation to the bundle layer.
    pub fn bundle_cli_run(
        &self,
        temp_dir: &TempDir,
        wf_analysis: NxfLogItem,
        twome: &str,
        force: &bool,
    ) -> Result<(), Epi4youError> {
        let ulid_str = Ulid::new().to_string();
        let analysis = NextflowAnalysis::init(wf_analysis.clone(), self.folder.clone())?;

        let nextflow_log_str = analysis.locate_nextflow_log(&temp_dir.path)?;
        let nextflow_stdout = analysis.extract_log_stdout(&nextflow_log_str, &temp_dir.path)?;
        let _progress_json =
            analysis.prepare_progress_json(&nextflow_stdout, &temp_dir.path, &ulid_str)?;

        let local_output = temp_dir.path.join("output");
        fs::create_dir_all(&local_output)
            .map_err(|_| Epi4youError::FailedToCreateFolder(local_output.clone()))?;

        log::info!("TempDir == {}", temp_dir);
        log::info!("AnalysisPath == {:?}", &analysis.get_analysis_dir());
        let analysis_dir = analysis.get_analysis_dir();
        for entry in WalkDir::new(&analysis_dir).into_iter().flatten() {
            if let Ok(relative_path) = entry.path().strip_prefix(&analysis_dir) {
                let destination = local_output.join(relative_path);

                if entry.path().is_dir() {
                    fs::create_dir_all(&destination)
                        .map_err(|_| Epi4youError::FailedToCreateFolder(destination.clone()))?;
                } else if entry.path().is_file() {
                    fs::copy(entry.path(), &destination)
                        .map_err(|_| Epi4youError::FailedToWritePath(destination.clone()))?;
                }
            }
        }

        let dest = PathBuf::from(twome);
        if dest.exists() && !*force {
            log::error!(
                "twome destination [{:?}] already exists - use `--force`?",
                dest
            );
            return Err(Epi4youError::FileAlreadyExistsUnforcedExecution(dest));
        }

        bundle::export_cli_run(
            &ulid_str,
            temp_dir.path.clone(),
            temp_dir.clone(),
            dest,
            &nextflow_stdout,
            &wf_analysis.timestamp,
            force,
        );

        Ok(())
    }
}
