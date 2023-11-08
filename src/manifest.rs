use std::{path::PathBuf, fs::File};
use serde::{Serialize, Deserialize};
use crate::{provenance::{Epi2MeProvenance, append_provenance}, json::wrangle_manifest};

pub static MANIFEST_JSON: &str = "4u_manifest.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct FileManifest {
    pub filename: String,
    pub relative_path: String,
    pub size: u64,
    pub md5sum: String,
}
impl Default for FileManifest {
    fn default() -> FileManifest {

        FileManifest {
            filename: String::from("undefined"),
            relative_path: String::from("undefined"),
            size: 0,
            md5sum: String::from("undefined"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Epi2MEWorkflow {
    pub workflow_name: String,
    pub workflow_user: String,
    pub workflow_version: String,
    pub workflow_commit: String,
    pub workflow_source: String,
    pub files: Vec<FileManifest>,
}

impl Default for Epi2MEWorkflow {
    fn default() -> Epi2MEWorkflow {

        Epi2MEWorkflow {
            workflow_name: String::from("undefined"),
            workflow_user: String::from("undefined"),
            workflow_version: String::from("undefined"),
            workflow_commit: String::from("undefined"),
            workflow_source: String::from("undefined"),
            files: Vec::<FileManifest>::new(),
        }
    }
}


#[derive(Serialize, Deserialize)]
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


#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Epi2MeContent {
    Epi2mePayload(Epi2meDesktopAnalysis),
    Epi2MEWorkflow(Epi2MEWorkflow),
  }


#[derive(Serialize, Deserialize)]
pub struct Epi2MeManifest {
    pub id: String,
    pub src_path: String,
    pub provenance: Vec<Epi2MeProvenance>,
    pub payload: Vec<Epi2MeContent>,
    pub filecount: u8,
    pub files_size: u64,

    pub signature: String,
}
impl Default for Epi2MeManifest {
    fn default() -> Epi2MeManifest {

        Epi2MeManifest {
            id: String::from("undefined"),
            src_path: String::from("undefined"),
            provenance: Vec::<Epi2MeProvenance>::new(),
            payload: Vec::<Epi2MeContent>::new(),
            filecount: 0,
            files_size: 0,
            signature: String::from("undefined"),
        }
    }
}


pub fn get_manifest_path(source: &PathBuf) -> PathBuf {
    let mut manifest = source.clone();
    manifest.push(MANIFEST_JSON);
    return manifest;
} 



pub fn get_manifest(source: &PathBuf) -> Option<Epi2MeManifest> {
        let manifest = get_manifest_path(source);
        if !manifest.exists() {
            // we need to create one
            println!("creating a new manifest");

            let mut man: Epi2MeManifest = Epi2MeManifest{
                ..Default::default()
            };

            let prov = append_provenance(String::from("manifest_created"), None, None, String::from(""));

            let _ = &man.provenance.push(prov);

            wrangle_manifest(&man);

            return Some(man);
        } else {
            // we should load the manifest

            let json_file = File::open(manifest).expect("file not found");

            let epi2me_manifest: Epi2MeManifest =
                serde_json::from_reader(json_file).expect("error while reading json");
            return Some(epi2me_manifest);
        }

}

pub fn touch_manifest(man: &mut Epi2MeManifest) {

    let touch_prov = append_provenance(String::from("manifest_touched"), None, None, String::from(""));
    man.provenance.push(touch_prov);

}



pub fn load_manifest_from_tarball() -> Option<Epi2MeManifest> {

    return None;
}