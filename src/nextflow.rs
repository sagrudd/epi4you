use std::process::Command;
use polars_core::prelude::*;
use std::io::Cursor;
use std::path::PathBuf;

use crate::dataframe::nextflow_vec_to_df;



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



pub fn validate_db_entry(_runid: String, polardb: &DataFrame) -> bool {
    // is runid in name field and unique

    let nameidx = polardb.find_idx_by_name("run_name");
    if nameidx.is_some() {
        let nameseries = polardb.select_at_idx(nameidx.unwrap());
        if nameseries.is_some() {
            println!("{:?}", nameseries);
            let _x = nameseries.unwrap();
            
            
        }
    }

    return false;
}

