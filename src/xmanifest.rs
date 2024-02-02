use std::{path::PathBuf, fs::{self, File}, env};
use tar::Builder;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

use crate::{bundle::sha256_str_digest, epi2me_db::Epi2meSetup};


pub static MANIFEST_JSON: &str = "4u_manifest.json";
pub static UNDEFINED: &str = "undefined";

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct FileManifest {
    pub filename: String,
    pub relative_path: String,
    pub size: u64,
    pub md5sum: String,
}

impl Default for FileManifest {
    fn default() -> FileManifest {

        FileManifest {
            filename: String::from(UNDEFINED),
            relative_path: String::from(UNDEFINED),
            size: 0,
            md5sum: String::from(UNDEFINED),
        }
    }
}



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

impl Default for Epi2meDesktopAnalysis {
    fn default() -> Epi2meDesktopAnalysis {

        Epi2meDesktopAnalysis {
            id: String::from(UNDEFINED),
            path: String::from(UNDEFINED),
            name: String::from(UNDEFINED),
            status: String::from(UNDEFINED),
            workflowRepo: String::from(UNDEFINED),
            workflowUser: String::from(UNDEFINED),
            workflowCommit: String::from(UNDEFINED),
            workflowVersion: String::from(UNDEFINED),
            createdAt: String::from(UNDEFINED),
            updatedAt: String::from(UNDEFINED),
            files: Vec::<FileManifest>::new(),
        }
    }
}


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Epi2meContainer {
    pub workflow: String,
    pub version: String,
    pub architecture: String,
    pub files: Vec<FileManifest>,
}


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Epi2meWorkflow {
    pub project: String,
    pub name: String,
    pub version: String,
    pub files: Vec<FileManifest>,
}

impl Default for Epi2meWorkflow {
    fn default() -> Epi2meWorkflow {

        Epi2meWorkflow {
            project: String::from(UNDEFINED),
            name: String::from(UNDEFINED),
            version: String::from(UNDEFINED),
            files: Vec::<FileManifest>::new(),
        }
    }
}


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
#[serde(tag = "type")]
pub enum Epi2MeContent {
    Epi2mePayload(Epi2meDesktopAnalysis),
    Epi2meWf(Epi2meWorkflow),
    Epi2meContainer(Epi2meContainer),
  }


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Epi2MeProvenance {
    pub id: String,
    pub action: String,
    pub value: Option<String>,
    pub user: String,
    pub timestamp: String,
}

impl Default for Epi2MeProvenance {
    fn default() -> Epi2MeProvenance {

        Epi2MeProvenance {
            id: Uuid::new_v4().to_string(),
            action: String::from(UNDEFINED),
            value: None,
            user: whoami::username(),
            timestamp: Local::now().to_string(),
        }
    }
}



#[derive(Clone, Serialize, Deserialize)]
pub struct Epi2MeManifest {
    pub id: String,
    pub src_path: String,
    pub provenance: Vec<Epi2MeProvenance>,
    pub payload: Vec<Epi2MeContent>,
    pub filecount: u64,
    pub files_size: u64,
    pub signature: String,
}

impl Epi2MeManifest {
    pub fn new(src_path: PathBuf) -> Self {
        let mut man = Epi2MeManifest {
            id: String::from(UNDEFINED),
            src_path: src_path.as_os_str().to_str().unwrap().to_string(),
            provenance: Vec::<Epi2MeProvenance>::new(),
            payload: Vec::<Epi2MeContent>::new(),
            filecount: 0,
            files_size: 0,
            signature: String::from(UNDEFINED),
        };   
        man.append_provenance(String::from("manifest_created"), None);
        man.append_provenance(String::from("hostname"), Some(hostname::get().unwrap().to_string_lossy().to_string()));
        return man; 
    }

    fn append_provenance(&mut self, what: String, value: Option<String>) {
        let when = Local::now().to_string();
        let prov = Epi2MeProvenance{
            action: String::from(what),
            value,
            timestamp: when,
            ..Default::default()
        };
        self.provenance.push(prov);
    
    }

    pub fn get_signature(&mut self) -> String {
        let mut unsigned = self.clone();
        unsigned.signature = String::from(UNDEFINED);
        sha256_str_digest(serde_json::to_string_pretty(&unsigned).unwrap().as_str())
    }

    fn sign(&mut self) {
        let signature = self.get_signature();
        self.signature = signature;
    }

    fn to_string(&mut self) -> String {
        self.sign();
        serde_json::to_string_pretty(&self).unwrap()
    }

    pub fn print(&mut self) {
        println!("{}", self.to_string());
    }

    pub fn write(&mut self, dest: &PathBuf) {
        println!("writing manifest to path [{:?}]", dest);
        let x = fs::write(dest, self.to_string());
        if x.is_err() {
            println!("Error with writing manifest to file");
        }
    }

    pub fn tar(&mut self, epi2me: &Option<Epi2meSetup>, manifest: &PathBuf, twome: &PathBuf) {
        let tarfile = File::create(twome).unwrap();
        let mut a = Builder::new(tarfile);

        let local_prefix = PathBuf::from(self.src_path.clone());
        let _ = env::set_current_dir(&local_prefix);

        println!("<+> {:?}", manifest);
        let _ = a.append_path(manifest);
        
        for x in &self.payload.clone() {
            match x {
                Epi2MeContent::Epi2meWf(epi2me_workflow) => {
                    println!("importing Workflow [{}]", epi2me_workflow.name);
                    //insert_untarred_workflow(epi2me_workflow, force);
                },
                
                Epi2MeContent::Epi2mePayload(desktop_analysis) => {
                    println!("importing DesktopAnalysis[{}]", &desktop_analysis.id);
                    //insert_untarred_desktop_analysis(desktop_analysis);
                },
    
    
                Epi2MeContent::Epi2meContainer(epi2me_container) => {
                    println!("importing Epi2meContainer");
                    for file in epi2me_container.files.iter() {
                        let mut pp = PathBuf::from(file.relative_path.clone());
                        pp.push(file.filename.clone());
                        if pp.exists() {
                            println!("<+> {:?}", pp);
                            let _ = a.append_path(pp);
                        } else {
                            eprintln!("<?> {:?}", pp);
                        }
                    }
                },
            }
        }

    } 

}