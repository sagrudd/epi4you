use std::{path::PathBuf, collections::HashMap, fs};
use polars::prelude::*;
use polars_core::{frame::DataFrame, series::Series};
use regex::Regex;
use serde::{Serialize, Deserialize};

use crate::{tempdir::TempDir, epi2me_db::{Epi2meSetup, self}, xworkflows::Epi2meWorkflow};


#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct Container {
    pub workflow: String,
    pub version: String,
    pub dockcont: String,
    
}

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct DockerContainer {
    pub repository: String,                          
    pub tag: String,                           
    pub image: String,
    pub created: String,    
    pub size: String,
}

#[derive(Clone)]
struct Epi2meDocker {
    temp_dir: PathBuf,
    epi2me: Option<Epi2meSetup>,
    containers: Vec<Container>
}

impl Epi2meDocker {
    fn new(tempdir: PathBuf) -> Self {
        Epi2meDocker {
            temp_dir: tempdir,
            epi2me: epi2me_db::find_db(),
            containers: Vec::new(),
        }    
    }

    fn extract_containers(&self, config: &HashMap<String, String>) -> Vec<String> {
        let mut container_vec: Vec<String> = Vec::new();
        let prefix = String::from("process.");
        let suffix = String::from("container");
        for key in config.keys() {
            if key.starts_with(&prefix) && key.ends_with(&suffix) { 
                let container_str = String::from(config.get(key).unwrap());
                let mut mod_container_str = container_str.clone();
                let re = Regex::new(r"\$\{[^\}]+\}").unwrap(); // 
                
                for matched in re.find_iter(&container_str) {
                    let found = matched.as_str();
                    let value = config.get(&found[2..found.len()-1]);
                    if value.is_some() {
                        mod_container_str = mod_container_str.replace(found, value.unwrap());
                    }
                }
                // println!("container == [{}]", mod_container_str);
                container_vec.push(mod_container_str);
            }
        }
        return container_vec;
    }

    fn identify_containers(&self, pb: &PathBuf) -> (HashMap<String, String>, Vec<String>) {
        let contents = fs::read_to_string(&pb).unwrap();
    
        let config = crate::xnf_parser::nextflow_parser(&contents);
        return (config.clone(), self.extract_containers(&config.clone()));
    }

    fn populate(&mut self) {
        println!("docker::populate");
        let e = self.epi2me.clone().unwrap();
        let td = self.temp_dir.clone();
        //let src_dir = &x.epi2wf_dir;
        //let wfs = list_installed_workflows(&src_dir);
        
        let mut workflows = crate::xworkflows::Epi2meWorkflow::new(td, Some(e));

        for workflow in workflows.wf_vector() {
            let workflow_id = vec![String::from(&workflow.project), String::from(&workflow.name)].join("/");
            println!("{:?}", workflow_id);
            let containers: Vec<String>;
            let _nf_config: HashMap<String, String>;

            let epi2me_installed_wf = workflows.glob_path_by_wfname(&workflow.project, &workflow.name);
            if epi2me_installed_wf.is_some() {
                let mut pb = epi2me_installed_wf.unwrap().clone();
                pb.push("nextflow.config");
                (_nf_config, containers) = self.identify_containers(&pb);
                for container in containers {
                    // println!("container == {}", container);
                    let container = Container{
                        workflow: String::from(&workflow_id), 
                        version:String::from(&workflow.version), 
                        dockcont:container,
                    };
                    self.containers.push(container);
                }
            } else {
                eprintln!("Unable to parse containers from [{}]", workflow_id);
            }
        }
    }


    pub fn containers_df(&self) -> DataFrame {
        let x: Vec<String> = self.containers.iter().map(|v| String::from(&v.workflow)).collect();
        let y: Vec<String> = self.containers.iter().map(|v| String::from(&v.version)).collect();
        let z: Vec<String> = self.containers.iter().map(|v| String::from(&v.dockcont)).collect();
        let xx: Series = Series::new("workflow", x);
        let yy = Series::new("version", y);
        let zz = Series::new("dockcont", z);
        DataFrame::new(vec![xx, yy, zz]).unwrap()
    }


    pub fn print(&mut self) {
        crate::dataframe::print_polars_df(&self.containers_df());
    }


}





pub fn docker_agent(tempdir: &TempDir, workflows: &Vec<String>, list: &bool, pull: &bool, twome: &Option<String>) {
    println!("--docker_agent");
    let mut edocker = Epi2meDocker::new(tempdir.path.clone());
    edocker.populate();

    if *list {
        println!("Make container vec pretty ...");
        edocker.print();
        return;
    }


}
