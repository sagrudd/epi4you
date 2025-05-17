use std::{env, path::PathBuf};

use polars::frame::DataFrame;
use serde::{Deserialize, Serialize};
use url::{Position, Url};
use glob::glob;
use crate::{app_db::{self, validate_db_entry}, dataframe::get_zero_val, epi2me_workflow::clip_relative_path, nextflow_log_parser::NextFlowLogs, xmanifest::{sha256_digest, FileManifest}};



#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Epi2meDesktopAnalysis {
    pub id: String,
    pub path: String,
    pub name: String,
    pub status: String,
    pub workflowRepo: String,
    pub workflowUser: String,
    pub workflowCommit: String,
    pub workflowVersion: String,
    pub createdAt: String,
    pub updatedAt: String,
    pub files: Vec<FileManifest>,
}

/* 
impl Default for Epi2meDesktopAnalysis {
    fn default() -> Epi2meDesktopAnalysis {

        Epi2meDesktopAnalysis {
            id: String::from("undefined"),
            path: String::from("undefined"),
            name: String::from("undefined"),
            status: String::from("undefined"),
            workflowRepo: String::from("undefined"),
            workflowUser: String::from("undefined"),
            workflowCommit: String::from("undefined"),
            workflowVersion: String::from("undefined"),
            createdAt: String::from("undefined"),
            updatedAt: String::from("undefined"),
            files: Vec::<FileManifest>::new(),
        }
    }
}
    */

impl Epi2meDesktopAnalysis {


    pub fn from_run_id(runid: &String, polardb: &DataFrame) -> Self {
        if validate_db_entry(runid, polardb) {
            let stacked = app_db::get_db_id_entry(runid, polardb).unwrap();
            // this is obligate pass due to previous validation
    
            let x = Epi2meDesktopAnalysis { 
                id: get_zero_val(&stacked, &String::from("id")),
                path: get_zero_val(&stacked, &String::from("path")),
                name: get_zero_val(&stacked, &String::from("name")),
                status: get_zero_val(&stacked, &String::from("status")),
                workflowRepo: get_zero_val(&stacked, &String::from("workflowRepo")),
                workflowUser: get_zero_val(&stacked, &String::from("workflowUser")),
                workflowCommit: get_zero_val(&stacked, &String::from("workflowCommit")),
                workflowVersion: get_zero_val(&stacked, &String::from("workflowVersion")),
                createdAt: get_zero_val(&stacked, &String::from("createdAt")),
                updatedAt: get_zero_val(&stacked, &String::from("updatedAt")),
                files: Vec::<FileManifest>::new()
               };
            return x;
        }
        panic!();
    }   
    


    pub fn init(ulid_str: &String, source: &PathBuf, nextflow_stdout: &String, timestamp: &String) -> Self {

            println!("get_analysis_struct_from_cli");
        
            let mut log = PathBuf::from(source);
            log.push("nextflow.stdout");
        
            // println!("{}", nextflow_stdout);

            let nlp = NextFlowLogs::init(nextflow_stdout);
            nlp.test();

            panic!();
        
            let name = nlp.get_value("name");

            let pname = nlp.get_value("repo");
            let project = nlp.get_value("repo");
            let revision = nlp.get_value("repo");
            let version = nlp.get_value("repo");
        
            let x = Epi2meDesktopAnalysis { 
                id: String::from(ulid_str),
                path: String::from(source.to_str().unwrap()),
                name: String::from(name),
                status: String::from("COMPLETED"),
                workflowRepo: pname,
                workflowUser: project,
                workflowCommit: String::from(revision),
                workflowVersion: version,
                createdAt: String::from(timestamp),
                updatedAt: String::from(timestamp),
                files: Vec::<FileManifest>::new(),
            };
        
            println!("{:?}", x);
            return x;
        }
        


        pub fn fish_files(&mut self, source: &PathBuf, local_prefix: &PathBuf) {

            let globpat = &source.clone().into_os_string().into_string().unwrap();
            let result = [&globpat, "/**/*.*"].join("");
        
            // let mut files: Vec<FileManifest> = Vec::new();
        
            println!("fishing for files at [{}]", result);
        
            let _ = env::set_current_dir(&globpat);
        
            for entry in glob(&result).expect("Failed to read glob pattern") {
                if entry.is_ok() {
                    let e = entry.unwrap();
                    let fname =  &e.file_name().and_then(|s| s.to_str());
                    if e.is_file() && !fname.unwrap().contains("4u_manifest.json") { // don't yet package the manifest - it'll be added later
                        let fp = e.as_os_str().to_str().unwrap();
        
                        let mut parent = e.clone();
                        let _ = parent.pop();
        
                        let relative_path = clip_relative_path(&e, &local_prefix);
                        //println!("{}", &fp);
        
                        let checksum = sha256_digest(&fp);
                        
                        //println!("file [{}] with checksum [{}]", &fp, &vv);
                        let file_size = e.metadata().unwrap().len();
        
                        self.files.push(FileManifest {
                            filename: String::from(e.file_name().unwrap().to_os_string().to_str().unwrap()),
                            relative_path: String::from(relative_path.clone().to_string_lossy().to_string()),
                            size: file_size,
                            md5sum: checksum,
                        })
                    }
                }
            }
        }
        
        pub fn get_files(&self) -> Vec::<FileManifest> {
            return self.files.clone();
        }


        pub fn get_files_size(&self) -> u64 {
            let mut size: u64 = 0;
            for file in self.files.clone() {
                size += file.size;
            }
            return size;
        }
        

}

