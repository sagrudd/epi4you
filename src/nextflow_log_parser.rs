//! Parsing helpers for extracting EPI2ME-relevant metadata from `nextflow`
//! textual logs.
//!
//! The EPI2ME Desktop UI ultimately wants workflow identity information such as
//! repository, revision, version, and run name. When bundling raw CLI runs we
//! do not have the Desktop database record, so we recover that information from
//! the preserved Nextflow output instead.

use std::collections::HashMap;

use url::{Position, Url};

/// Default workflow version when the log does not expose one clearly.
const DEFAULT_VERSION: &str = "dev";
/// Marker used by Nextflow's launch line in the stdout/log text we keep.
const LAUNCH_PREFIX: &str = "Launching `";
/// Text preceding the workflow revision token.
const REVISION_KEY: &str = " - revision: ";
/// Sentinel found in lines that expose version text in preserved stdout.
const VERSION_MARKER: &str = "||||||||||";

/// Small metadata bag extracted from a Nextflow launch transcript.
///
/// The keys intentionally mirror the fields that `epi4you` later uses to build
/// an [`Epi2meDesktopAnalysis`](crate::epi2me_desktop_analysis::Epi2meDesktopAnalysis).
pub struct NextFlowLogs {
    facet: HashMap<String, String>,
}

impl NextFlowLogs {
    /// Parses a reduced `nextflow.stdout` transcript into named metadata values.
    pub fn init(log: &str) -> Self {
        let mut facet = HashMap::<String, String>::new();

        let mut name = String::new();
        let mut revision = String::new();
        let mut project = String::new();
        let mut pname = String::new();
        let mut version = String::from(DEFAULT_VERSION);

        for line in log.lines() {
            if let Some(parsed) = parse_launch_line(line) {
                name = parsed.name;
                revision = parsed.revision;
                project = parsed.project;
                pname = parsed.pname;
                continue;
            }

            if !pname.is_empty() && line.contains(VERSION_MARKER) && line.contains(&pname) {
                version = parse_version(line, &pname);
            }
        }

        facet.insert("name".into(), name);
        facet.insert("revision".into(), revision);
        facet.insert("project".into(), project);
        facet.insert("pname".into(), pname);
        facet.insert("version".into(), version);

        NextFlowLogs { facet }
    }

    /// Returns one parsed value or an empty string if absent.
    pub fn get_value(&self, key: &str) -> String {
        self.facet.get(key).cloned().unwrap_or_default()
    }

    /// Emits all parsed key/value pairs to the logger for debugging.
    pub fn test(&self) {
        for (k, v) in self.facet.iter() {
            log::debug!("{k}\t\t{v}");
        }
    }
}

/// Structured representation of one launch line.
#[derive(Debug, PartialEq, Eq)]
struct LaunchLine {
    name: String,
    revision: String,
    project: String,
    pname: String,
}

/// Parses the Nextflow "Launching ..." line into its core identifiers.
fn parse_launch_line(line: &str) -> Option<LaunchLine> {
    let clipped_line = line.split_once(LAUNCH_PREFIX)?.1;
    let url_str = clipped_line.split('`').next()?;
    let name = clipped_line
        .split_once('[')?
        .1
        .split(']')
        .next()?
        .to_string();

    let revision = clipped_line
        .split_once(REVISION_KEY)?
        .1
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();

    let (project, pname) = parse_project_and_name(url_str);

    Some(LaunchLine {
        name,
        revision,
        project,
        pname,
    })
}

/// Extracts `{project, workflow}` from either a full URL or a shorthand tuple.
///
/// EPI2ME workflows are commonly referenced as GitHub paths such as
/// `epi2me-labs/wf-human-variation`, but some logs collapse this to a shorter
/// workflow-only form. The fallback keeps `epi4you` useful in both cases.
fn parse_project_and_name(url_str: &str) -> (String, String) {
    if let Ok(url) = Url::parse(url_str) {
        let data_url_payload = &url[Position::AfterHost..].trim_start_matches('/');
        if let Some((project, pname)) = data_url_payload.split_once('/') {
            return (project.to_string(), pname.to_string());
        }
    }

    let fallback_name = url_str.split('/').next().unwrap_or_default().to_string();
    ("epi2me-labs".to_string(), fallback_name)
}

/// Extracts a simplified workflow version token from a log line.
fn parse_version(line: &str, pname: &str) -> String {
    line.split_once(pname)
        .map(|(_, suffix)| suffix.trim())
        .and_then(|suffix| suffix.split('-').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_VERSION)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{parse_launch_line, parse_version, NextFlowLogs};

    #[test]
    fn parses_launch_line_with_url() {
        let line = "Apr-02 12:00:00.000 [main] INFO  nextflow.cli.CmdRun - Launching `https://github.com/epi2me-labs/wf-human-variation` [kind_curie] - revision: 123abc [v1.2.3]";
        let parsed = parse_launch_line(line).unwrap();

        assert_eq!(parsed.name, "kind_curie");
        assert_eq!(parsed.revision, "123abc");
        assert_eq!(parsed.project, "epi2me-labs");
        assert_eq!(parsed.pname, "wf-human-variation");
    }

    #[test]
    fn parses_launch_line_with_tuple_style_reference() {
        let line = "Apr-02 12:00:00.000 [main] INFO  nextflow.cli.CmdRun - Launching `wf-basecalling` [brave_hopper] - revision: dev";
        let parsed = parse_launch_line(line).unwrap();

        assert_eq!(parsed.name, "brave_hopper");
        assert_eq!(parsed.revision, "dev");
        assert_eq!(parsed.project, "epi2me-labs");
        assert_eq!(parsed.pname, "wf-basecalling");
    }

    #[test]
    fn parses_version_without_dash_suffix() {
        let version = parse_version("|||||||||| wf-basecalling 1.0.0", "wf-basecalling");
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn aggregates_log_metadata() {
        let log = "\
Apr-02 12:00:00.000 [main] INFO  nextflow.cli.CmdRun - Launching `https://github.com/epi2me-labs/wf-human-variation` [kind_curie] - revision: 123abc [v1.2.3]
|||||||||| wf-human-variation 1.2.3-extra
";

        let parsed = NextFlowLogs::init(log);

        assert_eq!(parsed.get_value("name"), "kind_curie");
        assert_eq!(parsed.get_value("revision"), "123abc");
        assert_eq!(parsed.get_value("project"), "epi2me-labs");
        assert_eq!(parsed.get_value("pname"), "wf-human-variation");
        assert_eq!(parsed.get_value("version"), "1.2.3");
    }
}
