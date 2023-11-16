use std::{path::PathBuf, fs::File, io::Read};
use tar::Archive;
use serde::{Serialize, Deserialize};
use crate::{provenance::{Epi2MeProvenance, append_provenance}, json::{wrangle_manifest, get_manifest_str}, bundle::sha256_str_digest};

pub static MANIFEST_JSON: &str = "4u_manifest.json";

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
            filename: String::from("undefined"),
            relative_path: String::from("undefined"),
            size: 0,
            md5sum: String::from("undefined"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
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
#[derive(Clone)]
#[derive(Debug)]
#[serde(tag = "type")]
pub enum Epi2MeContent {
    Epi2mePayload(Epi2meDesktopAnalysis),
    Epi2MEWorkflow(Epi2MEWorkflow),
  }


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
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





pub fn load_manifest_from_tarball(twome: PathBuf) -> Option<Epi2MeManifest> {

    let mut ar = Archive::new(File::open(twome).unwrap());

    for (_i, file) in ar.entries().unwrap().enumerate() {
        let mut file = file.unwrap();
        
        let file_path = file.path();
        if file_path.is_ok() {
            let ufilepath = file_path.unwrap().into_owned();
            println!("\t\tobserving file {:?}", &ufilepath);
            let fname =  &ufilepath.file_name().and_then(|s| s.to_str());
            if fname.is_some() && fname.unwrap().contains(MANIFEST_JSON) {
                println!("this is the manifest ...");

                let mut buffer = String::new();

                let manifest = file.read_to_string(&mut buffer);
                if manifest.is_ok() {
                    // println!("{buffer}");
                    let epi2me_manifest: Epi2MeManifest = serde_json::from_str(&buffer).expect("error while reading json");
                    return Some(epi2me_manifest);
                }
            }
        }
    }

    return None;
}


pub fn is_manifest_honest(manifest: &Epi2MeManifest) -> bool {
    let mut lman = manifest.clone();
    let signature = String::from(&manifest.signature);
    println!("expecting manifest checksum [{}]", signature);
    lman.signature = String::from("undefined");
    let resignature = sha256_str_digest(get_manifest_str(&lman).as_str());
    println!("observed manifest checksum  [{}]", resignature);

    if signature == resignature {
        return true;
    }

    return false;
}