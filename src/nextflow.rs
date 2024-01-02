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


use crate::tempdir::TempDir;
use crate::{bundle, tempdir};
use crate::{dataframe::{nextflow_vec_to_df, print_polars_df}, bundle::anyvalue_to_str};



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


fn get_matched_nexflow_log(logfile: &PathBuf, runid: &String) -> Option<String> {
    let contents = fs::read_to_string(logfile).unwrap();
    if contents.contains(runid) {
        //println!("{contents}");
        println!("logfile match [{:?}]", logfile);
        return Some(contents);
    }
    return None;
}


fn locate_nextflow_log(src_dir: &PathBuf, wf_analysis: NxfLogItem, tmp_dir: &PathBuf) -> Option<String> {
    println!("locating nextflow logs ...");


    let mut candidate_logs: Vec<String> = Vec::new();
    let mut candidate_pbs: Vec<PathBuf> = Vec::new();

    let mut glob_fish_str: String = String::from(src_dir.clone().into_os_string().to_str().unwrap());
    glob_fish_str.push_str(&std::path::MAIN_SEPARATOR.to_string());
    glob_fish_str.push_str(".nextflow.log*");


    for entry in glob(&glob_fish_str).expect("Failed to read glob pattern") {
        if entry.is_ok() {
            let cand_logfile = entry.unwrap();
            //println!("scanning file {:?}", cand_logfile);

            let log = get_matched_nexflow_log(&cand_logfile, &wf_analysis.run_name);
            if log.is_some() {
                candidate_logs.push(log.unwrap());
                candidate_pbs.push(cand_logfile);
            }
        }
    }

    if candidate_logs.len() > 1 {
        eprintln!("log file selection is ambiguous - more than one match");
        return None;
    } else if candidate_logs.len() == 1 {
        // copy the logfile to the new working directory ...
        let mut target = tmp_dir.clone();
        target.push("nextflow.log");
        let _ = fs::copy(candidate_pbs.get(0).unwrap(), &target);
        println!("populating nextflow.log to [{:?}]", target);
        return Some(String::from(candidate_logs.get(0).unwrap()));
    } else {
        eprintln!("failed to locate appropriately tagged logfile - have you been housekeeping?");
        return None;
    }
}


fn extract_log_stdout(nf_log: String, tmp_dir: &PathBuf) -> Option<String> {
    /* the challenge here is that many entries are multi-line; we need to
    select for the fields of interest and exclude fields that are irrelevant */

    let allowed = vec![
        String::from("[main] INFO"),
        String::from("[main] WARN"),
        String::from("[Task submitter] INFO"),
    ];

    let disallowed = vec![
        String::from("DEBUG"),
        // String::from("[Task submitter]"),
        String::from("[Task monitor]"),
        String::from("org.pf4j"),
    ];

    let mut cache = String::new();
    let mut capture: bool = false;

    let lines = nf_log.split("\n");

    for mut line in lines {
        for allowed_key in &allowed {
            if line.contains(allowed_key) {
                capture = true;
            }
        }
        for disallowed_key in &disallowed {
            if line.contains(disallowed_key) {
                capture = false;
            }
        }
        if capture {
            for allowed_key in &allowed {
                if line.contains(allowed_key) {
                    let mut idx = line.find(" - ").unwrap();
                    idx += 3;
                    line = &line[idx..];
                }
            }

            cache.push_str(line);
            cache.push('\n');
        }
    }
    
    //println!("{cache}");
    let mut target = tmp_dir.clone();
    target.push("nextflow.stdout");
    let status = fs::write(&target, &cache);
    if status.is_ok() {
        println!("populating nextflow.stdout to [{:?}]", target);
        return Some(cache);
    }

    return None;
}




fn prepare_progress_json(nextflow_stdout: &String, temp_dir: &PathBuf, ulid_str: &String) -> Option<PathBuf> {

    /*
    {"01HFV9GNS1PPPRCBB8JWBB4W64":
    {"pipeline:variantCallingPipeline:sanitizeRefFile":{"status":"COMPLETED","tag":null,"total":1,"complete":1},
    "fastcat":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:getVersions":{"status":"COMPLETED","tag":null,"total":1,"complete":1},
    "pipeline:getParams":{"status":"COMPLETED","tag":null,"total":1,"complete":1},
    "pipeline:variantCallingPipeline:lookupMedakaModel":{"status":"COMPLETED","tag":null,"total":1,"complete":1},
    "pipeline:subsetReads":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:porechop":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:addMedakaToVersionsFile":{"status":"COMPLETED","tag":null,"total":1,"complete":1},
    "pipeline:variantCallingPipeline:alignReads":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:variantCallingPipeline:bamstats":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:variantCallingPipeline:mosdepth":{"status":"COMPLETED","tag":null,"total":10,"complete":10},
    "pipeline:variantCallingPipeline:downsampleBAMforMedaka":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:variantCallingPipeline:concatMosdepthResultFiles":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:variantCallingPipeline:medakaConsensus":{"status":"COMPLETED","tag":null,"total":10,"complete":10},
    "pipeline:variantCallingPipeline:medakaVariant":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:collectFilesInDir":{"status":"COMPLETED","tag":null,"total":2,"complete":2},
    "pipeline:makeReport":{"status":"COMPLETED","tag":null,"total":1,"complete":1},
    "output":{"status":"COMPLETED","tag":null,"total":14,"complete":14}
    }
    } 
    */

    let v:HashMap<String, ProgressItem> = HashMap::new();
    let mut x = ProgressJson{name: ulid_str.to_owned(), key: v};

    let mut book_reviews: HashMap<String, u16> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    let subproc = "Submitted process >";
    let lines = nextflow_stdout.split("\n");
    for mut line in lines {
        if line.starts_with("[") && line.contains(subproc) {
            let idx = line.find(subproc).unwrap() + subproc.len();
            line = &line[idx..].trim();

            if line.contains(" (") {
                line = &line[..line.find(" (").unwrap()].trim()
            }

            if book_reviews.contains_key(line) {
                book_reviews.insert(line.to_owned(), book_reviews.get(line).unwrap() + 1);
            } else {
                book_reviews.insert(line.to_owned(), 1);
                order.push(line.to_owned());
            }
        }
    }

    for key in order {
        let val = book_reviews.get(&key).unwrap();
        let pi = ProgressItem{status: String::from("COMPLETED"), tag: String::from("null"), total: *val, complete: *val};
        x.key.insert(key, pi);
    }

    let s = serde_json::to_string(&x);

    if s.is_err() {
        eprintln!("{:?}", s.err());
    } else {
        let mut target = temp_dir.clone();
        target.push("progress.json");
        let status = fs::write(&target, &s.unwrap());
        if status.is_ok() {
            println!("populating progress.json to [{:?}]", target);
            return Some(target);
        }
    }
    return None;
}

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct ProgressItem {
    pub status: String,
    pub tag: String,
    pub total: u16,
    pub complete: u16,
}

#[derive(Debug, Deserialize)]
pub struct ProgressJson {
    // #[serde(rename = ulid_str)]
    pub name: String,
    pub key: HashMap<String, ProgressItem>,
}

impl Serialize for ProgressJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let item_name = self.name.to_owned();
        let mut struct_ser = serializer.serialize_map(Some(1))?;
        struct_ser.serialize_entry(&item_name, &self.key)?;
        struct_ser.end()
    }
}


pub fn bundle_cli_run(temp_dir: &TempDir, wf_analysis: NxfLogItem, src_dir: &PathBuf, twome: &Option<String>) {

    // assign a ULID for this bundle ...
    let ulid_str = Ulid::new().to_string();

    // receive the path of the folder to bundle - validate that it exists and is compliant
    let analysis_path = locate_wf_analysis_dir(&wf_analysis, src_dir);
    if analysis_path.is_none() {
        eprintln!("Unable to resolve path for analysis directory");
        return;
    }

    // nextflow.log
    /*
     This is named identically; in a CLI run this will be hidden (.nextflow) and will acquire numeric suffix
     to differentiate between different runs

     they may be matched on the basis of the provided run_name - this should be unique - esp. in combination with
     the command line used */
    let nextflow_log = locate_nextflow_log(src_dir, wf_analysis, &temp_dir.path);
    if nextflow_log.is_none() {
        return;
    }

    // nextflow.stdout
    /* this can perhaps be parsed from the nextflow.log */
    let nextflow_stdout = extract_log_stdout(nextflow_log.unwrap(), &temp_dir.path);
    if nextflow_stdout.is_none() {
        return;
    }


    // progress.json
    /* this is the summary of tasks that have been run (through the GUI) and the final state and count of completed
    tasks - this is eye candy from the application side; information should be entirely parseable from the nextflow.log */
    let progress_json = prepare_progress_json(&nextflow_stdout.clone().unwrap(), &temp_dir.path, &ulid_str);
    if progress_json.is_none() {
        eprintln!("issue with packaging the workflow progress json");
        return;
    }

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


    // params.json
    /* this is probably the parameters that have been specified in the GUI and determined from the nf-core schema;
    this is a nice to have but does not appear to be used post-analysis (until concept of re-running is developed) */

    /*
    Now is a good time to populate the temp directory output folder with the contents of the locally specified
    command line nextflow run ... 
     */
    let mut outdir = PathBuf::from(&temp_dir.path);
    outdir.push("output");
    let mkdir = fs::create_dir(outdir);
    if mkdir.is_err() {
        eprintln!("creating output folder failed with {:?}", mkdir.err());
        return;
    }

    //let paths = fs::read_dir(analysis_path.unwrap()).unwrap();
    //for path in paths {
    //    let xpath = path.unwrap().path().clone();
    //    println!("handling xpath {:?}", xpath);
    //}

    println!("TempDir == {}", temp_dir);
    println!("AnalysisPath == {:?}", &analysis_path.clone().unwrap());
    let mut local_output = temp_dir.path.clone();
    local_output.push("output");
    let _create_d = fs::create_dir(&local_output);
    let ap = &analysis_path.clone().unwrap();
    for entry in WalkDir::new(&analysis_path.unwrap()) {
        if entry.is_ok() {
            let ent = entry.unwrap();
            let core_p = ent.path().strip_prefix(ap);
            if core_p.is_ok() {
                let gg = core_p.unwrap();
                let mut dest_f = local_output.clone();
                dest_f.push(&gg);
                println!("src {:?} -> ", &dest_f);

                if gg.is_dir() {
                    let _create_d = fs::create_dir_all(dest_f);
                } else if gg.is_file() {
                    let _copy_f = fs::copy(ent.path(), dest_f);
                }
            }
        }
    }

    /*
    Once we are here, can we re-use any of the earlier logic to just bundle the now mangled workflow as used
    previously for bundling EPI2ME workflows?    
     */
    let dest = PathBuf::from(twome.to_owned().unwrap());
    if dest.exists() {
        eprintln!("twome destination [{:?}] already exists - use `--force`?", dest);
        return;
    }
    bundle::export_cli_run(temp_dir.path.clone(), temp_dir.clone(), dest, &nextflow_stdout.clone().unwrap());

}



pub fn nextflow_manager(list: &bool, nxf_bin: &Option<String>, nxf_work: &Option<String>, runid: &Option<String>, twome: &Option<String>) {
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
        } 

        if twome.is_none() {
            println!("EPI2ME analysis twome archiving requires a --twome identifier (archive to write)");
            return;
        } 

        let wf_analysis = validate_db_entry(runid.as_ref().unwrap().to_string(), localruns.as_ref().unwrap());
        if wf_analysis.is_none() {
            println!("Unable to resolve specified EPI2ME analysis [{}] - check name", runid.as_ref().unwrap());
            return;
        }

        // create a working folder that will be populated
        let tempdir = tempdir::get_tempdir();
        if tempdir.is_some() {
            let temp_dir = tempdir.unwrap();
            

            bundle_cli_run(&temp_dir, wf_analysis.unwrap(), &src_dir, twome);
        
        
        } 
    }
}