use std::collections::HashMap;

use url::{Position, Url};

const DEFAULT_VERSION: &str = "dev";
const LAUNCH_PREFIX: &str = "Launching `";
const REVISION_KEY: &str = " - revision: ";
const VERSION_MARKER: &str = "||||||||||";

pub struct NextFlowLogs {
    facet: HashMap<String, String>,
}

impl NextFlowLogs {
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

    pub fn get_value(&self, key: &str) -> String {
        self.facet.get(key).cloned().unwrap_or_default()
    }

    pub fn test(&self) {
        for (k, v) in self.facet.iter() {
            log::debug!("{k}\t\t{v}");
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LaunchLine {
    name: String,
    revision: String,
    project: String,
    pname: String,
}

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
