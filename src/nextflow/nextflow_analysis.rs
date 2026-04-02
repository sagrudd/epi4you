//! Helpers for turning one parsed Nextflow run into a staged analysis payload.
//!
//! EPI2ME Desktop stores more than "just the output directory": it also keeps
//! lightweight status and progress artefacts that make a completed analysis
//! browsable in the GUI. These helpers reconstruct enough of that surrounding
//! shape for `epi4you` imports and exports.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use glob::glob;

use crate::epi4you_errors::Epi4youError;

use super::{
    nextflow_log_item::NxfLogItem,
    nextflow_progress::{ProgressItem, ProgressJson},
};

/// Resolved view of one Nextflow analysis on disk.
///
/// This type couples the `nextflow log` row with the concrete directories and
/// derived helper files we need to build a bundle.
pub struct NextflowAnalysis {
    wf_analysis: NxfLogItem,
    src_dir: PathBuf,
    folder: PathBuf,
}

impl NextflowAnalysis {
    /// Resolves the output directory associated with a log row.
    ///
    /// The preferred source of truth is the original command line, especially
    /// `--out_dir` / `--out-dir`. If that cannot be recovered, we fall back to
    /// the common EPI2ME-style `output/` convention.
    pub fn init(wf_analysis: NxfLogItem, analysis_folder: PathBuf) -> Result<Self, Epi4youError> {
        log::info!("processing command [{:?}]", &wf_analysis.command);

        let candidate = resolve_analysis_dir(&wf_analysis.command, &analysis_folder)
            .or_else(|| fallback_output_dir(&analysis_folder))
            .ok_or(Epi4youError::NextflowAnalysisFolderNotFound)?;

        log::info!("using analysis folder [{:?}]", candidate);

        Ok(NextflowAnalysis {
            wf_analysis,
            src_dir: analysis_folder,
            folder: candidate,
        })
    }

    /// Returns the resolved analysis output directory.
    pub fn get_analysis_dir(&self) -> PathBuf {
        self.folder.clone()
    }

    /// Locates the `.nextflow.log*` file that belongs to this run.
    ///
    /// Multiple logs may exist after repeated runs in the same directory. We
    /// choose the one that mentions the specific `run_name` and then copy it
    /// into the temporary staging area under the stable name `nextflow.log`.
    pub fn locate_nextflow_log(&self, tmp_dir: &PathBuf) -> Result<String, Epi4youError> {
        log::info!("locating nextflow logs ...");

        let mut candidate_logs: Vec<String> = Vec::new();
        let mut candidate_pbs: Vec<PathBuf> = Vec::new();

        let mut glob_fish_str = self.src_dir.to_string_lossy().into_owned();
        glob_fish_str.push(std::path::MAIN_SEPARATOR);
        glob_fish_str.push_str(".nextflow.log*");

        for entry in glob(&glob_fish_str).map_err(|_| Epi4youError::FailedToParseFileContent)? {
            if let Ok(cand_logfile) = entry {
                let log = get_matched_nexflow_log(&cand_logfile, &self.wf_analysis.run_name);
                if let Some(log) = log {
                    candidate_logs.push(log);
                    candidate_pbs.push(cand_logfile);
                }
            }
        }

        match candidate_logs.len() {
            0 => {
                log::error!(
                    "failed to locate appropriately tagged logfile - have you been housekeeping?"
                );
                Err(Epi4youError::FileSelectionFailedFileNotFound)
            }
            1 => {
                let mut target = tmp_dir.clone();
                target.push("nextflow.log");
                fs::copy(&candidate_pbs[0], &target)
                    .map_err(|_| Epi4youError::FailedToWritePath(target.clone()))?;
                log::info!("populating nextflow.log to [{:?}]", target);
                Ok(candidate_logs.remove(0))
            }
            _ => {
                log::error!("log file selection is ambiguous - more than one match");
                Err(Epi4youError::FileSelectionIsAmbiguous)
            }
        }
    }

    /// Distills the full Nextflow log into the subset EPI2ME-style metadata
    /// extraction cares about.
    ///
    /// The resulting text is written as `nextflow.stdout` because downstream
    /// parsing logic only needs the user-facing launch and task submission
    /// lines, not the entire debug-oriented Nextflow log.
    pub fn extract_log_stdout(
        &self,
        nf_log: &str,
        tmp_dir: &PathBuf,
    ) -> Result<String, Epi4youError> {
        let allowed = ["[main] INFO", "[main] WARN", "[Task submitter] INFO"];
        let disallowed = ["DEBUG", "[Task monitor]", "org.pf4j"];

        let mut cache = String::new();
        let mut capture = false;

        for mut line in nf_log.lines() {
            if allowed.iter().any(|allowed_key| line.contains(allowed_key)) {
                capture = true;
            }

            if disallowed
                .iter()
                .any(|disallowed_key| line.contains(disallowed_key))
            {
                capture = false;
            }

            if capture {
                if allowed.iter().any(|allowed_key| line.contains(allowed_key)) {
                    if let Some((_, payload)) = line.split_once(" - ") {
                        line = payload;
                    }
                }

                cache.push_str(line);
                cache.push('\n');
            }
        }

        let mut target = tmp_dir.clone();
        target.push("nextflow.stdout");
        fs::write(&target, &cache).map_err(|_| Epi4youError::FailedToWritePath(target.clone()))?;
        println!("populating nextflow.stdout to [{:?}]", target);
        Ok(cache)
    }

    /// Builds a compact `progress.json` file from submitted-process lines.
    ///
    /// This mirrors the sort of task summary EPI2ME Desktop presents in its UI.
    /// It is intentionally lossy: the goal is to recover a useful final
    /// completed-process picture rather than every transient scheduler event.
    pub fn prepare_progress_json(
        &self,
        nextflow_stdout: &str,
        temp_dir: &PathBuf,
        ulid_str: &str,
    ) -> Result<PathBuf, Epi4youError> {
        let mut progress = ProgressJson {
            name: ulid_str.to_owned(),
            key: HashMap::new(),
        };

        let mut process_counter: HashMap<String, u16> = HashMap::new();
        let mut bfx_process: Vec<String> = Vec::new();

        let subproc = "Submitted process >";
        for mut line in nextflow_stdout.lines() {
            if line.starts_with('[') && line.contains(subproc) {
                let idx = line.find(subproc).unwrap() + subproc.len();
                line = line[idx..].trim();

                if let Some((trimmed, _)) = line.split_once(" (") {
                    line = trimmed.trim();
                }

                if let Some(count) = process_counter.get_mut(line) {
                    *count += 1;
                } else {
                    process_counter.insert(line.to_owned(), 1);
                    bfx_process.push(line.to_owned());
                }
            }
        }

        for key in bfx_process {
            let val = process_counter.get(&key).unwrap();
            let pi = ProgressItem {
                status: String::from("COMPLETED"),
                tag: String::from("null"),
                total: *val,
                complete: *val,
            };
            progress.key.insert(key, pi);
        }

        let serialized =
            serde_json::to_string(&progress).map_err(|_| Epi4youError::FailedToParseFileContent)?;
        let mut target = temp_dir.clone();
        target.push("progress.json");
        fs::write(&target, serialized)
            .map_err(|_| Epi4youError::FailedToWritePath(target.clone()))?;
        println!("populating progress.json to [{:?}]", target);
        Ok(target)
    }
}

/// Attempts to resolve the analysis directory from the original CLI command.
fn resolve_analysis_dir(command: &str, analysis_folder: &Path) -> Option<PathBuf> {
    let output_dir = parse_output_dir(command)?;
    let candidate = analysis_folder.join(output_dir);

    if candidate.exists() && candidate.is_dir() {
        Some(candidate)
    } else {
        None
    }
}

/// Falls back to the common `output/` directory used by many EPI2ME workflows.
fn fallback_output_dir(analysis_folder: &Path) -> Option<PathBuf> {
    let candidate = analysis_folder.join("output");
    if candidate.exists() && candidate.is_dir() {
        Some(candidate)
    } else {
        None
    }
}

/// Parses supported output-directory flags from a Nextflow command line.
///
/// Supporting both `--out_dir` and `--out-dir` lets the code tolerate
/// historical differences between workflow parameter styles.
fn parse_output_dir(command: &str) -> Option<&str> {
    const KEYS: [&str; 4] = ["--out_dir=", "--out-dir=", "--out_dir", "--out-dir"];

    for token in command.split_whitespace() {
        if let Some(value) = token
            .strip_prefix(KEYS[0])
            .or_else(|| token.strip_prefix(KEYS[1]))
        {
            return (!value.is_empty()).then_some(value);
        }
    }

    let mut tokens = command.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == KEYS[2] || token == KEYS[3] {
            return tokens
                .next()
                .map(|value| value.trim_matches('"').trim_matches('\''))
                .filter(|value| !value.is_empty());
        }
    }

    None
}

/// Returns the full text of a candidate log if it belongs to the named run.
fn get_matched_nexflow_log(cand_logfile: &PathBuf, run_name: &str) -> Option<String> {
    let content = fs::read_to_string(cand_logfile).ok()?;
    content.contains(run_name).then_some(content)
}

#[cfg(test)]
mod tests {
    use super::{parse_output_dir, resolve_analysis_dir};
    use std::{fs, path::PathBuf};

    #[test]
    fn parses_output_dir_argument_with_equals() {
        assert_eq!(
            parse_output_dir("nextflow run wf --out_dir=results"),
            Some("results")
        );
        assert_eq!(
            parse_output_dir("nextflow run wf --out-dir=results"),
            Some("results")
        );
    }

    #[test]
    fn parses_output_dir_argument_with_separate_value() {
        assert_eq!(
            parse_output_dir("nextflow run wf --out_dir results -profile test"),
            Some("results")
        );
        assert_eq!(
            parse_output_dir("nextflow run wf --out-dir 'results' -profile test"),
            Some("results")
        );
    }

    #[test]
    fn resolves_existing_analysis_dir() {
        let root = unique_test_dir("analysis-dir");
        let results = root.join("results");
        fs::create_dir_all(&results).unwrap();

        let resolved = resolve_analysis_dir("nextflow run wf --out_dir results", &root);
        assert_eq!(resolved, Some(results));

        fs::remove_dir_all(root).unwrap();
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "epi4you-{prefix}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        path
    }
}
