use std::collections::HashMap;
use std::{env, fs};
use std::process::Command;
use polars_core::prelude::*;
use serde::{Deserialize, Serialize};

use ulid::Ulid;
use walkdir::WalkDir;
use std::io::Cursor;
use std::path::PathBuf;
use glob::glob;
use serde::ser::SerializeMap;


use crate::bundle::export_nf_workflow;
use crate::dataframe::{nf_wf_vec_to_df, workflow_vec_to_df};
use crate::settings::list_available_workflows;
use crate::tempdir::TempDir;
use crate::workflow::Workflow;
use crate::{bundle, tempdir};
use crate::{dataframe::{nextflow_vec_to_df, print_polars_df}, bundle::anyvalue_to_str};





















pub fn nextflow_run_manager(list: &bool, nxf_bin: &Option<String>, nxf_work: &Option<String>, runid: &Option<String>, twome: &Option<String>, force: &bool) {

}


fn extract_nextflow_workflow_config(workflow_id: &str) -> (String, HashMap<String, String>) {

    let output = Command::new("nextflow")
        .arg("config")
        .arg(workflow_id)
        .output()
        .expect("failed to execute process");

    let wf_config = String::from_utf8_lossy(&output.stdout).into_owned();
    //println!("{}", wf_config);
    let config = crate::xnf_parser::nextflow_parser(&wf_config);
    let mut version = String::from("?");
    if config.get("manifest.version").is_some() {
        version = String::from(config.get("manifest.version").unwrap());
    }
    //println!("workflow version [{}]", version);
    return (version, config);
}


fn parse_nextflow_workflow_info(workflow_id: &str) -> String {
    let lp = "local path  :";
    let output = Command::new("nextflow")
        .arg("info")
        .arg(workflow_id)
        .output()
        .expect("failed to execute process");
    
    let wf_info = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut wf_path = String::from("undefined");
    //println!("{}", wf_info);
    let wf_info_lines = wf_info.lines();
    for line in wf_info_lines {
        if line.contains(lp) {
            // println!("{}", line);
            wf_path = String::from(line.replace(lp, "").trim());
        }
    }
    return wf_path;
}

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct NextflowAssetWorkflow {
    pub workflow: String,
    pub path: String,
    pub version: String,
    pub config: HashMap<String, String>,
}

fn nextflow_workflow_pull(nxf_bin: &String, workflow: &String) -> Option<NextflowAssetWorkflow> {

    let output = Command::new(nxf_bin)
        .arg("pull")
        .arg(workflow)
        .output()
        .expect("failed to execute process");

    let s = String::from_utf8_lossy(&output.stdout).into_owned();
    let trimmed_s = s.trim();

    if trimmed_s.contains("WARN: Cannot read project manifest") {
        eprintln!("FUBAR - error with nextflow pull\n{}", trimmed_s);
        return None;
    }
    
    let wf_path = parse_nextflow_workflow_info(workflow);
    let (wf_version, config) = extract_nextflow_workflow_config(workflow);

    let wf = NextflowAssetWorkflow{
        workflow: String::from(workflow),
        path: String::from(wf_path),
        version: String::from(wf_version),
        config: config,
    };

    return Some(wf);
}


fn get_local_artifacts(nxf_bin: &String) -> Vec<NextflowAssetWorkflow> {
    let mut artifacts: Vec<NextflowAssetWorkflow> = Vec::new();

    // run nextflow list
    let output = Command::new(nxf_bin)
        .arg("list")
        .output()
        .expect("failed to execute process");

    let s = String::from_utf8_lossy(&output.stdout).into_owned();
    let trimmed_s = s.trim();

    let lines = trimmed_s.lines();
    for line in lines {
        // println!("split item {}", line);
        // if we are here, then this is a legit nextflow workflow ---
        let wf_path = parse_nextflow_workflow_info(line);
        let (wf_version, config) = extract_nextflow_workflow_config(line);

        let wf = NextflowAssetWorkflow{
            workflow: String::from(line),
            path: String::from(wf_path),
            version: String::from(wf_version),
            config: config,
        };
        artifacts.push(wf);
    }
    return artifacts;
}


fn list_installed_nextflow_artifacts(nxf_bin: &String) -> Option<DataFrame> {
    let artifacts = get_local_artifacts(nxf_bin);
    let df = nf_wf_vec_to_df(artifacts);
    return Some(df);
}


fn get_workflow_entity(key: &String, extant_artifacts: &Vec<NextflowAssetWorkflow>) -> Option<NextflowAssetWorkflow> {
    for artif in extant_artifacts {
        if artif.workflow == key.to_owned() {
            return Some(artif.to_owned());
        }
    }
    return None;
}




pub fn nextflow_artifact_manager(list: &bool, workflow: &Vec<String>, nxf_bin: &Option<String>, pull: &bool, twome: &Option<String>, force: &bool, _docker: &bool) {
    

}