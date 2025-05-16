use std::{env, path::PathBuf};

use serde::{Deserialize, Serialize};
use glob::glob;
use crate::{epi2me_db, xmanifest::{sha256_digest, FileManifest}};



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

impl Epi2meWorkflow {


    pub fn init(p: &String, n: &String, v: &String) -> Self {
        return Epi2meWorkflow {
            project: String::from(p),
            name: String::from(n),
            version: String::from(v),
            files: Vec::<FileManifest>::new(),
        };
    }

    // let wf_path = check_defined_wfdir_exists(&local_prefix, &project, &name);

    pub fn check_defined_wfdir_exists(&self, wfdir: &PathBuf) -> Option<PathBuf> {
        let mut x = wfdir.clone();
        x.push(String::from("workflows"));
        x.push(&self.project);
        x.push(&self.name);
        if x.exists() && x.is_dir() {
            println!("\tdefined workflow folder exists at [{}]", x.display());
            return Some(x.clone());
        }
        eprintln!("\tworkflow folder does not exist at [{}]", x.display());
        return None;
    }

    pub fn path_init(wf_path: Option<&PathBuf>, project: &String, name: &String, version: &String) -> Self {
        let local_prefix: PathBuf;
        if wf_path.is_some() {
            local_prefix = wf_path.unwrap().to_owned();
        } else {
           local_prefix = epi2me_db::find_db().unwrap().epi2path;
        }
    
        let mut vehicle = Epi2meWorkflow::init(&project, &name, &version);
      
        println!("{:?}", vehicle);
    
        let wf_path = vehicle.check_defined_wfdir_exists(&local_prefix);
        println!("Mining files from [{:?}]", wf_path);
    
        vehicle.fish_files(&wf_path.unwrap(), &local_prefix);
        return vehicle;
    }


    fn fish_files(&mut self, source: &PathBuf, local_prefix: &PathBuf) {

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



pub fn clip_relative_path(e: &PathBuf, local_prefix: &PathBuf) -> PathBuf {
    let mut relative_path = get_relative_path(e, local_prefix);
    let _ = relative_path.pop();
    return relative_path;
}



pub fn get_relative_path(e: &PathBuf, local_prefix: &PathBuf) -> PathBuf {
    //println!("relativePath {:?} from lp {:?} ...", e, local_prefix);
    PathBuf::from(e.strip_prefix(local_prefix).unwrap())
}


