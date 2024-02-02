use std::{fs, path::PathBuf};
use glob::glob;
use polars::prelude::*;
use serde::{Serialize, Deserialize};
use crate::{epi2me_db::Epi2meSetup, xnf_parser, dataframe::print_polars_df};


#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct Workflow {
    pub project: String,
    pub name: String,
    pub version: String,
}

#[derive(Clone)]
pub struct Epi2meWorkflow {
    temp_dir: PathBuf,
    epi2me: Option<Epi2meSetup>,
    workflows: Vec<Workflow>,
}

impl Epi2meWorkflow {
    pub fn new(tempdir: PathBuf, epi2me: Option<Epi2meSetup>) -> Self {
        let mut wf = Epi2meWorkflow {
            temp_dir: tempdir,
            epi2me: epi2me.to_owned(),
            workflows: Vec::new(),
        };
        wf.load_installed_workflows();
        return wf;
    }

    pub fn load_installed_workflows(&mut self) {
        println!("Epi2meWorkflow::load_installed_workflows");
        let e = self.epi2me.clone().unwrap();
        let wf_dir = e.epi2wf_dir;
        let globpat = wf_dir.to_owned().into_os_string().into_string().unwrap();
        let path_pattern = [&globpat, "/*/*"].join("");

        for entry in glob(&path_pattern).expect("Failed to read glob pattern") {
            match entry {
                Ok(mut globpath) => {
                    
                    let mut config_path = globpath.clone();
                    config_path.push("nextflow.config");
                    let globpathstr = globpath.as_os_str().to_str().unwrap();
                    if globpath.is_dir() && !globpathstr.contains(".nextflow") {
                        let workflow = globpath.file_name().unwrap().to_str().unwrap().to_string();
                        globpath.pop();
                        let project = globpath.file_name().unwrap().to_str().unwrap().to_string();
    
                        // extract workflow revision for the linked artifact
                        // this is probably best prepared by parsing the information from the config file?
                        if config_path.exists() {
                            let contents = fs::read_to_string(&config_path).unwrap();
                            let config = xnf_parser::nextflow_parser(&contents);
                            let mut version = String::from("?");
                            let man_version = config.get("manifest.version");
                            if man_version.is_some() {
                                version = String::from(man_version.unwrap());
                            }
    
                            let w = Workflow{
                                project: project,
                                name: workflow,
                                version: String::from(version),
                            };
                            self.workflows.push(w);
    
                        }
                    }
                },
                Err(e) => println!("{:?}", e),
            }
        }
        println!("items == [{}]", self.workflows.len());
    }


    pub fn workflows_df(&self) -> DataFrame {
        let x: Vec<String> = self.workflows.iter().map(|v| String::from(&v.project)).collect();
        let y: Vec<String> = self.workflows.iter().map(|v| String::from(&v.name)).collect();
        let z: Vec<String> = self.workflows.iter().map(|v| String::from(&v.version)).collect();
        let xx: Series = Series::new("project", x);
        let yy = Series::new("name", y);
        let zz = Series::new("version", z);
        DataFrame::new(vec![xx, yy, zz]).unwrap()
    }


    pub fn print(&mut self) {
        print_polars_df(&self.workflows_df());
    }

    pub fn wf_vector(&self) -> Vec<Workflow> {
        return self.workflows.clone();
    }

    fn is_folder_wf_compliant(&self, wffolder: &PathBuf) -> bool {
        let required_files = vec!["main.nf", "nextflow.config"];
        let mut counter = 0;
        let paths = fs::read_dir(wffolder).unwrap();
        for path in paths {
            let fname = &path.unwrap().file_name().to_string_lossy().to_string();
            if required_files.contains(&fname.as_str()) {
                // println!("found {:?}", fname);
                counter += 1;
            }
        }
        if required_files.len() == counter {
            return true;
        }
        return false;
    }

    pub fn glob_path_by_wfname(&self, project: &String, name: &String) -> Option<PathBuf> {

        let mut src = PathBuf::from(self.epi2me.clone().unwrap().epi2wf_dir);
        src.push(&project);
    
        let globpat = src.into_os_string().into_string().unwrap();
        let result = [&globpat, "/*", &name].join("");
        
        let gdata =  glob(&result).expect("Failed to read glob pattern");
        for entry in gdata {
            if entry.is_ok() {
                let entry_item = entry.unwrap();
                // ensure that the folder found is actually a nextflow folder (nanopore flavoured)
                if self.is_folder_wf_compliant(&entry_item) {
                    // println!("folder picked == {:?}", entry_item);
                    return Some(entry_item)
                }
            }
        }
        // we can also assess whether project is a link to e.g. a nextflow based folder elsewhere on CLI
        return None;
    }


}
