use std::env;
use std::process::Command;
use polars_core::prelude::*;
use std::io::Cursor;
use std::path::PathBuf;

use crate::{dataframe::{nextflow_vec_to_df, print_polars_df}, bundle::anyvalue_to_str, epi2me_db};



pub fn get_nextflow_path(nxf_bin: Option<String>) -> Option<String> {
    println!("getting nextflow path ...");

    let mut nextflow_bin: Option<String> = None;

    // nxf_bin path
    if nxf_bin.is_some() {
        let x = PathBuf::from(nxf_bin.unwrap());
        // does this actually exist?
        if x.exists() && x.is_file() {
            nextflow_bin = Some(String::from(x.to_str().unwrap()));
        }
    } else {
        // which nextflow -- handle output state
        let output = Command::new("which")
            .arg("nextflow")
            .output()
            .expect("failed to execute process");


        let mut s = String::from_utf8_lossy(&output.stdout).into_owned();
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
        if s.len() > 0 {
            println!("nextflow candidate at [{}]", s);
            let x = PathBuf::from(&s);
            if x.exists() && x.is_file() {
                // is that enough for now?
                nextflow_bin = Some(s);
            }
        }
    }

    if nextflow_bin.is_some() {
        println!("Using nxf_bin found at [{:?}]", &nextflow_bin.clone().unwrap());
    } else {
        println!("unable to resolve a functional location for nextflow!");
    }
    return nextflow_bin;
}



#[derive(serde::Deserialize)]
struct Row<'a> {
    timestamp: &'a str,
    duration: &'a str,
    run_name: &'a str,
    status: &'a str,
    revision_id: &'a str,
    session_id: &'a str,
    command: &'a str,
}

#[derive(Debug)]
pub struct NxfLogItem {
    pub timestamp: String,
    pub duration: String,
    pub run_name: String,
    pub status: String,
    pub revision_id: String,
    pub session_id: String,
    pub command: String,
}

impl Default for NxfLogItem {
    fn default() -> NxfLogItem {
        NxfLogItem {
            timestamp: String::from(""),
            duration: String::from(""),
            run_name: String::from(""),
            status: String::from(""),
            revision_id: String::from(""),
            session_id: String::from(""),
            command: String::from(""),
        }
    }
}

pub fn parse_nextflow_folder(nxf_workdir: Option<String>, nxf_bin: Option<String>) -> Option<DataFrame> {

    let nextflow_bin = get_nextflow_path(nxf_bin);
    if nextflow_bin.is_some() && nxf_workdir.is_some() {
        println!("Looking for nxf artifacts at [{}]", nxf_workdir.clone().unwrap());

        let output = Command::new(nextflow_bin.unwrap())
            .current_dir(nxf_workdir.clone().unwrap())
            .arg("log")
            .output()
            .expect("failed to execute process");

        /* 
        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        */

        let mut vec: Vec<NxfLogItem> = Vec::new();

        let file = Cursor::new(output.stdout);
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(file);

        for result in rdr.records() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here..
            let record = result;
            if record.is_ok() {
                let x = record.unwrap();
                let row: Row = x.deserialize(None).unwrap();
                
                let y: NxfLogItem  =  NxfLogItem {
                    timestamp: String::from(row.timestamp),
                    duration: String::from(row.duration),
                    run_name: String::from(row.run_name),
                    status: String::from(row.status),
                    revision_id: String::from(row.revision_id),
                    session_id: String::from(row.session_id),
                    command: String::from(row.command),
            
                    ..Default::default()
                };

                let ok: &str = "OK";
                if row.status.trim().eq(ok) {
                    vec.push(y);
                }
            }
        }

            // and wrangle observations into a dataframe
            let df = nextflow_vec_to_df(vec);

            //print_nxf_log(&df);
            return Some(df);
        }

        return None;
}



pub fn validate_db_entry(runid: String, polardb: &DataFrame) -> Option<NxfLogItem> {
    // is runid in name field and unique

    let nameidx = polardb.find_idx_by_name("run_name");
    if nameidx.is_some() {
        let nameseries = polardb.select_at_idx(nameidx.unwrap());
        if nameseries.is_some() {
            //println!("{:?}", nameseries);
            let x = nameseries.unwrap();
            let mut idx: usize = 0;
            for anyvalue in x.iter() {
                let value = anyvalue.get_str();
                if value.is_some() {
                    let val = value.unwrap().trim();
                    // println!("value [{:?}]", val);
                    if val == runid {
                        println!("*match*");
                        let single_row = polardb.get(idx);
                        if single_row.is_some() {
                            // println!("row == {:?}", &single_row);
                            let unwrapped_row = single_row.unwrap();
                            let entry = NxfLogItem {
                                timestamp: String::from(anyvalue_to_str(unwrapped_row.get(0)).trim()),
                                duration: String::from(anyvalue_to_str(unwrapped_row.get(1)).trim()),
                                run_name: String::from(anyvalue_to_str(unwrapped_row.get(2)).trim()),
                                status: String::from(anyvalue_to_str(unwrapped_row.get(3)).trim()),
                                revision_id: String::from(anyvalue_to_str(unwrapped_row.get(4)).trim()),
                                session_id: String::from(anyvalue_to_str(unwrapped_row.get(5)).trim()),
                                command: String::from(anyvalue_to_str(unwrapped_row.get(6)).trim()),
                            };
                            println!("{:?}", entry);
                            return Some(entry);
                        }
                    }
                }
                idx += 1;
            }
            
        }
    }

    return None;
}


fn locate_wf_analysis_dir(wf_analysis: &NxfLogItem, src_dir: &PathBuf) -> Option<PathBuf> {

    let command = &wf_analysis.command;
    println!("processing command [{:?}]", &command);

    let hooks = vec![" --out_dir "];
    let nextfield = " -";
    for hook in hooks {
        if command.contains(&hook) {
            println!("filtering on [{}]", &hook);
            let mut index = command.find(&hook).unwrap();
            index += &hook.len();
            let mut substr = command[index..].trim();
            println!("substr == {}", substr);
            // let's further filter on the next parameter ...
            if substr.contains(nextfield) {
                index = substr.find(nextfield).unwrap();
                substr = &substr[..index].trim();
                println!("substr == {}", substr);
            }

            let mut test_dir = src_dir.to_owned();
            test_dir.push(substr);

            if test_dir.exists() && test_dir.is_dir() {
                println!("we have a candidate folder at [{:?}]", test_dir);
                return Some(test_dir);
            }
        }
    }
    return None;
}


fn locate_nextflow_log() -> Option<Vec<String>> {

    return None;
}



pub fn bundle_cli_run(wf_analysis: NxfLogItem, src_dir: &PathBuf) {

    // receive the path of the folder to bundle - validate that it exists and is compliant
    let analysis_path = locate_wf_analysis_dir(&wf_analysis, src_dir);
    if analysis_path.is_none() {
        eprintln!("Unable to resolve path for analysis directory");
        return;
    }

    // create a working folder that will be populated
    let tempdir = epi2me_db::get_tempdir();
    if tempdir.is_none() {
        return;
    }
    let temp_dir = tempdir.unwrap();
    println!("using tempdir at [{:?}]", &temp_dir);

    // nextflow.log
    /*
     This is named identically; in a CLI run this will be hidden (.nextflow) and will acquire numeric suffix
     to differentiate between different runs

     they may be matched on the basis of the provided run_name - this should be unique - esp. in combination with
     the command line used */
    let nextflow_log = locate_nextflow_log();



    // invoke.log
    /*
    This log is required for presentation in the EPI2ME GUI but is specific to the EPI2ME application;
    we will create a simple import logging here to produce some informative content - but not aiming
    to replicate or reproduce; will probably describe preparation of these package information */


    // launch.json
    /* internal to the EPI2ME application; unclear if this is required? */


    // local.config
    /* this appears to be a collection of nextflow config parameters that have been provided to nextflow
    via the app - can this be skipped / kept light - if an additional nextflow.config has been supplied
    at the command line then perhaps this file should be populated here (future release?) */


    // nextflow.stdout
    /* this can perhaps be parsed from the nextflow.log */

    // params.json
    /* this is probably the parameters that have been specified in the GUI and determined from the nf-core schema;
    this is a nice to have but does not appear to be used post-analysis (until concept of re-running is developed) */

    // progress.json
    /* this is the summary of tasks that have been run (through the GUI) and the final state and count of completed
    tasks - this is eye candy from the application side; information should be entirely parseable from the nextflow.log */
}



pub fn nextflow_manager(list: &bool, nxf_bin: &Option<String>, nxf_work: &Option<String>, runid: &Option<String>) {
    let mut nxf_workdir = nxf_work.clone();
    if nxf_workdir.is_none() {
        nxf_workdir = Some(env::current_dir().unwrap().to_string_lossy().into_owned());
        println!("Setting nextflow workdir to cwd [{:?}]", nxf_workdir.clone().unwrap());
    }

    let localruns = parse_nextflow_folder(nxf_workdir.clone(), nxf_bin.clone());
    if localruns.is_none() {
        println!("No local nextflow run folders found at specified path");
        return;
    }
    let src_dir = PathBuf::from(nxf_workdir.unwrap());

    if *list {
        print_polars_df(&localruns.unwrap());
        // todo - how do we print out dataframe with a more considered number of columns?
    } else {
        if runid.is_none() {
            println!("EPI2ME analysis twome archiving requires a --runid identifier (run_name)");
            return;
        } else {
            let wf_analysis = validate_db_entry(runid.as_ref().unwrap().to_string(), localruns.as_ref().unwrap());
            if wf_analysis.is_none() {
                println!("Unable to resolve specified EPI2ME analysis [{}] - check name", runid.as_ref().unwrap());
                return;
            }
            bundle_cli_run(wf_analysis.unwrap(), &src_dir);
        }
    }
}