use std::{path::PathBuf, fs::File, io::Read};
use tar::Archive;
use serde::{Serialize, Deserialize};
use crate::{provenance::{Epi2MeProvenance, append_provenance}, json::{wrangle_manifest, get_manifest_str}, bundle::{sha256_str_digest, sha256_digest}, epi2me_tar::untar, app_db::insert_untarred_desktop_analysis, workflow::insert_untarred_workflow};

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
            project: String::from("undefined"),
            name: String::from("undefined"),
            version: String::from("undefined"),
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
pub struct Epi2MeManifest {
    pub id: String,
    pub src_path: String,
    pub provenance: Vec<Epi2MeProvenance>,
    pub payload: Vec<Epi2MeContent>,
    pub filecount: u64,
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


fn _get_manifest_path(source: &PathBuf) -> PathBuf {
    let mut manifest = source.clone();
    manifest.push(MANIFEST_JSON);
    return manifest;
} 



pub fn get_manifest(_source: &PathBuf) -> Option<Epi2MeManifest> {

    let mut man: Epi2MeManifest = Epi2MeManifest{
        ..Default::default()
    };
    let prov = append_provenance(String::from("manifest_created"), None, None, String::from(""));
    let _ = &man.provenance.push(prov);
    wrangle_manifest(&man);
    return Some(man);

        /*let manifest = get_manifest_path(source);
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
        }  */

}

pub fn _touch_manifest(man: &mut Epi2MeManifest) {
    let touch_prov = append_provenance(String::from("manifest_touched"), None, None, String::from(""));
    man.provenance.push(touch_prov);
}

pub fn manifest_note_packaged_analysis(man: &mut Epi2MeManifest, id: &String) {
    let action = vec![String::from("analysis_bundled"), String::from(id)].join(": ");
    let pack = append_provenance(action, None, None, String::from(""));
    man.provenance.push(pack);
}

pub fn manifest_note_packaged_workflow(man: &mut Epi2MeManifest, id: &String) {
    let action = vec![String::from("workflow_bundled"), String::from(id)].join(": ");
    let pack = append_provenance(action, None, None, String::from(""));
    man.provenance.push(pack);
}

pub fn file_manifest_size(files: &Vec<FileManifest>) -> u64 {
    let mut size: u64 = 0;
    for file in files {
        size += file.size;
    }
    return size;
}


pub fn load_manifest_from_tarball(twome: &PathBuf) -> Option<Epi2MeManifest> {

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


fn validate_manifest_files(file_manifest: &Vec<FileManifest>) -> Option<bool> {
    for file in file_manifest {
        println!("file [{:?}]", file);
        let file_to_check = PathBuf::from(&file.relative_path).join(PathBuf::from(&file.filename));
        if !file_to_check.exists() {
            eprintln!("error - file [{:?}] is missing", file_to_check);
            return None;
        }
        let digest = sha256_digest(&file_to_check.to_str().unwrap());
        if !(&digest == &file.md5sum) {
            eprintln!(" error checksum inconsistency - {digest}");
            return None;
        }
    }
    return Some(true);
}


pub fn is_manifest_honest(manifest: &Epi2MeManifest, twome: &PathBuf, _force: &bool) -> Option<Vec<Epi2MeContent>> {

    let mut successful_content: Vec<Epi2MeContent> = Vec::new();

    let mut lman = manifest.clone();
    let signature = String::from(&manifest.signature);
    println!("expecting manifest checksum [{}]", signature);
    lman.signature = String::from("undefined");
    let resignature = sha256_str_digest(get_manifest_str(&lman).as_str());
    println!("observed manifest checksum  [{}]", resignature);

    if signature != resignature {
        eprintln!("There is inconsistency in the checksums - cannot trust this content!");
        return None;
    }

    // if we are here - there is parity of md5sum - let's unpack the archive and check each of the files ...
    let untar_status = untar(twome);
    if untar_status.is_some() {
        println!("tarfile successfully unpacked - sanity checking the packed files ...");

        for cfile in &manifest.payload {
            //let is_desktop_payload = matches!(cfile, Epi2MeContent::Epi2mePayload { .. });
            //let is_epi2me_workflow = matches!(cfile, Epi2MeContent::Epi2meWf { .. });
            //println!("Epi2mePayload :: {:?}", is_desktop_payload);
            //println!("Epi2meWorkflow :: {:?}", is_epi2me_workflow);

            match cfile {
                Epi2MeContent::Epi2meWf(epi2me_workflow) => {
                     println!("Epi2MEWorkflow");
                     let x = validate_manifest_files(&epi2me_workflow.files);
                     if x.is_none() {
                        eprintln!("failed to validate the workflow manifest files - quitting");
                        return None;
                     }
                     successful_content.push(cfile.clone());
                },
                
                Epi2MeContent::Epi2mePayload(desktop_analysis) => {
                    println!("Epi2MEWorkflow");
                    let x = validate_manifest_files(&desktop_analysis.files);
                    if x.is_none() {
                        eprintln!("failed to validate the analysis manifest files - quitting");
                        return None;
                    }
                    // if we are here then the manifest specified files are present and coherent; we're good to go ...
                    successful_content.push(cfile.clone());
                },

                Epi2MeContent::Epi2meContainer(epi2me_container) => {
                    println!("Epi2meContainer");
                    let x = validate_manifest_files(&epi2me_container.files);
                    if x.is_none() {
                        eprintln!("failed to validate the analysis manifest files - quitting");
                        return None;
                    }
                    // if we are here then the manifest specified files are present and coherent; we're good to go ...
                    successful_content.push(cfile.clone());
                }
                
            }
        }
        return Some(successful_content);
    }
    return None;
}


pub async fn import_resolved_content(content: &Vec<Epi2MeContent>, force: &bool) {
    println!("import_resolved_content");
    for cfile in content {
        println!("cfile instance ...");
        
        match cfile {
            Epi2MeContent::Epi2meWf(epi2me_workflow) => {
                println!("importing Workflow [{}]", epi2me_workflow.name);
                insert_untarred_workflow(epi2me_workflow, force);
            },
            
            Epi2MeContent::Epi2mePayload(desktop_analysis) => {
                println!("importing DesktopAnalysis[{}]", &desktop_analysis.id);
                insert_untarred_desktop_analysis(desktop_analysis);
            },


            Epi2MeContent::Epi2meContainer(epi2me_container) => {
                println!("deprecated Epi2meContainer");
            },
        }
    }
}