use std::{fs, io::Cursor, path::PathBuf, process::Command};

use ulid::Ulid;
use walkdir::WalkDir;

use crate::{bundle::{self, anyvalue_to_str}, dataframe::{self, nextflow_vec_to_df}, epi4you_errors::Epi4youError, nextflow::{nextflow_analysis::NextflowAnalysis, nextflow_log_item::{NxfLogItem, Row}}, tempdir::TempDir};




pub struct NextFlowResultFolder {
    folder: PathBuf,
    nxf_bin: PathBuf,
    vec: Vec::<NxfLogItem>,
}

impl NextFlowResultFolder {

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

    return Ok(folder);
    }


    fn get_nextflow_path(nxf_bin: Option<String>) -> Result<PathBuf, Epi4youError> {
        log::info!("getting nextflow path ...");

        let mut nextflow_bin: Option<PathBuf> = None;
        // nxf_bin path

        if nxf_bin.is_some() {
            let x = PathBuf::from(nxf_bin.as_ref().unwrap());
            // does this actually exist?
            if x.exists() && x.is_file() {
                nextflow_bin = Some(x);
            } else if x.exists() && x.is_dir() {
                return Err(Epi4youError::FolderFoundWhenFileExpected(x));
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
                log::debug!("nextflow candidate at [{}]", s);
                let x = PathBuf::from(&s);
                if x.exists() && x.is_file() {
                    // is that enough for now?
                    nextflow_bin = Some(x);
                } else if x.exists() && x.is_dir() {
                    return Err(Epi4youError::FolderFoundWhenFileExpected(x));
                }
            }
        }

        if nextflow_bin.is_some() {
            log::info!("Using nxf_bin found at [{:?}]", &nextflow_bin.clone().unwrap());
        } else {
            log::error!("unable to resolve a functional location for nextflow!");
            return Err(Epi4youError::UnableToLocateNextflowBinary);
        }
        return Ok(nextflow_bin.unwrap());
    }






        fn parse_nextflow_folder(&mut self) -> Result<(), Epi4youError> {

            log::info!("Looking for nxf artifacts at [{}]", &self.folder.to_string_lossy());

            let output = Command::new(String::from(self.nxf_bin.to_string_lossy()))
                .current_dir(String::from(self.folder.to_string_lossy()))
                .arg("log")
                .output()
                .expect("failed to execute process");


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
                    
                    let y: NxfLogItem = NxfLogItem::init(row.clone())?;

                    let ok: &str = "OK";
                    if row.get_status().trim().eq(ok) {
                        self.vec.push(y);
                    }
                }
            }

        return Ok(());


        }



        pub fn list_runs(&self) {
            let df = nextflow_vec_to_df(self.vec.clone());
            dataframe::print_polars_df(&df);
        }



        pub fn verify_cli_entity(&self, runid: String) -> Result<NxfLogItem, Epi4youError> {
            // is runid in name field and unique

            let polardb = nextflow_vec_to_df(self.vec.clone());
        
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
                                    return Ok(entry);
                                }
                            }
                        }
                        idx += 1;
                    }
                    
                }
            }
        
            return Err(Epi4youError::SpecifiedNextflowRunNotFound(runid));
        }
        
        

        pub fn bundle_cli_run(&self, temp_dir: &TempDir, wf_analysis: NxfLogItem, twome: &Option<String>, force: &bool) -> Result<(), Epi4youError> {

            // assign a ULID for this bundle ...
            let ulid_str = Ulid::new().to_string();
        
            // receive the path of the folder to bundle - validate that it exists and is compliant
            let analysis = NextflowAnalysis::init(wf_analysis.clone(), self.folder.clone())?;

        
            // nextflow.log
            /*
             This is named identically; in a CLI run this will be hidden (.nextflow) and will acquire numeric suffix
             to differentiate between different runs
        
             they may be matched on the basis of the provided run_name - this should be unique - esp. in combination with
             the command line used */

            //let nextflow_log = locate_nextflow_log(src_dir, &wf_analysis, &temp_dir.path);
            //if nextflow_log.is_none() {
            //    return;
            //}
            let nextflow_log_str = analysis.locate_nextflow_log(&temp_dir.path)?;
        
            // nextflow.stdout
            /* this can perhaps be parsed from the nextflow.log */
            
            //let nextflow_stdout = extract_log_stdout(nextflow_log.unwrap(), &temp_dir.path);
            //if nextflow_stdout.is_none() {
            //    return;
            //}
            let nextflow_stdout = analysis.extract_log_stdout(&nextflow_log_str, &temp_dir.path)?;
        
        
            // progress.json
            /* this is the summary of tasks that have been run (through the GUI) and the final state and count of completed
            tasks - this is eye candy from the application side; information should be entirely parseable from the nextflow.log */
            
            //let progress_json = prepare_progress_json(&nextflow_stdout.clone().unwrap(), &temp_dir.path, &ulid_str);
            //if progress_json.is_none() {
            //    eprintln!("issue with packaging the workflow progress json");
            //    return;
            //}
            let _progress_json = analysis.prepare_progress_json(&nextflow_stdout, &temp_dir.path, &ulid_str)?;
        
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
            let mkdir = fs::create_dir(outdir.clone());
            if mkdir.is_err() {
                return Err(Epi4youError::FailedToCreateFolder(outdir.clone()));
            }
        
            //let paths = fs::read_dir(analysis_path.unwrap()).unwrap();
            //for path in paths {
            //    let xpath = path.unwrap().path().clone();
            //    println!("handling xpath {:?}", xpath);
            //}
        
            log::info!("TempDir == {}", temp_dir);
            log::info!("AnalysisPath == {:?}", &analysis.get_analysis_dir());
            let mut local_output = temp_dir.path.clone();
            local_output.push("output");
            let _create_d = fs::create_dir(&local_output);
            let ap = &analysis.get_analysis_dir();
            for entry in WalkDir::new(&analysis.get_analysis_dir()) {
                if entry.is_ok() {
                    let ent = entry.unwrap();
                    let core_p = ent.path().strip_prefix(ap);
                    if core_p.is_ok() {
                        let gg = core_p.unwrap();
                        let mut dest_f = local_output.clone();
                        dest_f.push(&gg);
                        // println!("src {:?} -> ", &dest_f);
        
                        if ent.path().is_dir() {
                            let _create_d = fs::create_dir_all(dest_f);
                        } else if ent.path().is_file() {
                            // println!("copying ...");
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
            if dest.exists() && !force {
                log::error!("twome destination [{:?}] already exists - use `--force`?", dest);
                return Err(Epi4youError::FileAlreadyExistsUnforcedExecution(dest));
            }
            bundle::export_cli_run(&ulid_str, temp_dir.path.clone(), temp_dir.clone(), dest, &nextflow_log_str.clone(), &wf_analysis.timestamp, force);
        
            return Ok(());

        }
        
        


      
    }



