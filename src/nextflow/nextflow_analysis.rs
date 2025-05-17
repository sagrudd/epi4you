use std::{collections::HashMap, fs, path::PathBuf};

use crate::epi4you_errors::Epi4youError;
use glob::glob;
use super::{nextflow_log_item::NxfLogItem, nextflow_progress::{ProgressItem, ProgressJson}};


pub struct NextflowAnalysis {
    wf_analysis: NxfLogItem,
    src_dir: PathBuf,
    folder: PathBuf,
}


impl NextflowAnalysis {

    pub fn init(wf_analysis: NxfLogItem, analysis_folder: PathBuf) -> Result<Self, Epi4youError> {


            let command = &wf_analysis.command;
            log::info!("processing command [{:?}]", &command);
        
            let hooks = vec![" --out_dir "];
            let nextfield = " -";
            for hook in hooks {
                if command.contains(&hook) {
                    log::debug!("filtering on [{}]", &hook);
                    let mut index = command.find(&hook).unwrap();
                    index += &hook.len();
                    let mut substr = command[index..].trim();
                    log::debug!("substr == {}", substr);
                    // let's further filter on the next parameter ...
                    if substr.contains(nextfield) {
                        index = substr.find(nextfield).unwrap();
                        substr = &substr[..index].trim();
                        log::debug!("substr == {}", substr);
                    }
        
                    let mut test_dir = analysis_folder.to_owned();
                    test_dir.push(substr);
        
                    if test_dir.exists() && test_dir.is_dir() {
                        log::info!("we have a candidate folder at [{:?}]", test_dir);

                        let analysis = NextflowAnalysis {
                            wf_analysis,
                            src_dir: analysis_folder,
                            folder: test_dir,
                        };
                        return Ok(analysis);
                    } 
                }
            }
            return Err(Epi4youError::NextflowAnalysisFolderNotFound);
        


    }

    pub fn get_analysis_dir(&self) -> PathBuf {
        return self.folder.clone();
    }

    pub fn locate_nextflow_log(&self, tmp_dir: &PathBuf) -> Result<String, Epi4youError> {


            log::info!("locating nextflow logs ...");
        
        
            let mut candidate_logs: Vec<String> = Vec::new();
            let mut candidate_pbs: Vec<PathBuf> = Vec::new();
        
            let mut glob_fish_str: String = String::from(self.src_dir.clone().into_os_string().to_str().unwrap());
            glob_fish_str.push_str(&std::path::MAIN_SEPARATOR.to_string());
            glob_fish_str.push_str(".nextflow.log*");
        
        
            for entry in glob(&glob_fish_str).expect("Failed to read glob pattern") {
                if entry.is_ok() {
                    let cand_logfile = entry.unwrap();
                    //println!("scanning file {:?}", cand_logfile);
        
                    let log = get_matched_nexflow_log(&cand_logfile, &self.wf_analysis.run_name);
                    if log.is_some() {
                        candidate_logs.push(log.unwrap());
                        candidate_pbs.push(cand_logfile);
                    }
                }
            }
        
            if candidate_logs.len() > 1 {
                log::error!("log file selection is ambiguous - more than one match");
                return Err(Epi4youError::FileSelectionIsAmbiguous);
            } else if candidate_logs.len() == 1 {
                // copy the logfile to the new working directory ...
                let mut target = tmp_dir.clone();
                target.push("nextflow.log");
                let _ = fs::copy(candidate_pbs.get(0).unwrap(), &target);
                log::info!("populating nextflow.log to [{:?}]", target);
                return Ok(String::from(candidate_logs.get(0).unwrap()));
            } else {
                log::error!("failed to locate appropriately tagged logfile - have you been housekeeping?");
                return Err(Epi4youError::FileSelectionFailedFileNotFound);
            }

        

    }



pub fn extract_log_stdout(&self, nf_log: &str, tmp_dir: &PathBuf) -> Result<String, Epi4youError> {
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
        return Ok(cache);
    }

    return Err(Epi4youError::FailedToParseFileContent);
}



pub fn prepare_progress_json(&self, nextflow_stdout: &String, temp_dir: &PathBuf, ulid_str: &String) -> Result<PathBuf, Epi4youError> {

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

    let mut process_counter: HashMap<String, u16> = HashMap::new();
    let mut bfx_process: Vec<String> = Vec::new();

    let subproc = "Submitted process >";
    let lines = nextflow_stdout.split("\n");
    for mut line in lines {
        if line.starts_with("[") && line.contains(subproc) {
            let idx = line.find(subproc).unwrap() + subproc.len();
            line = &line[idx..].trim();

            if line.contains(" (") {
                line = &line[..line.find(" (").unwrap()].trim()
            }

            if process_counter.contains_key(line) {
                process_counter.insert(line.to_owned(), process_counter.get(line).unwrap() + 1);
            } else {
                process_counter.insert(line.to_owned(), 1);
                bfx_process.push(line.to_owned());
            }
        }
    }

    for key in bfx_process {
        let val = process_counter.get(&key).unwrap();
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
            return Ok(target);
        }
    }
    return Err(Epi4youError::FailedToParseFileContent);
}




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
