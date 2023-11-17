use std::{path::PathBuf, fs};

use home;
use crate::{json, workflow};


pub struct Epi2meSetup {
    pub epi2os: String,
    pub epi2path: PathBuf,
    pub epi2db_path: PathBuf,
    pub epi2wf_dir: PathBuf,
    pub epi4you_path: PathBuf,
    pub instances_path: PathBuf,
    pub arch: String,
}

pub fn get_platformstr() -> String {
    return String::from(std::env::consts::ARCH);
}


pub fn find_db() -> Option<Epi2meSetup> {
    println!("locating the EPI2ME app.db");

    let home_dir = home::home_dir();
    if home_dir.is_some() {

        let mut os: Option<String> = None;
        let mut path: Option<PathBuf> = None;

        let macos = check_os_specific_db_path(&home_dir, "Library/Application Support/EPI2ME/config.json", "macOS");
        let linux = check_os_specific_db_path(&home_dir, ".config/EPI2ME/config.json", "Linux");
        
        if macos.is_some() {
            os = Some(String::from("macOS"));
            path = macos;
        } else if linux.is_some() {
            os = Some(String::from("linux"));
            path = linux;
        }

        if path.is_some() {
            let db_path = get_appdb_path(&path.clone().unwrap());
            let mut instances_path = PathBuf::from(&path.clone().unwrap());
            instances_path.push("instances");
            let wf_dir = workflow::get_epi2me_wfdir_path(&path.clone().unwrap());
            let for_you_dir: Option<PathBuf> = get_4you_path(&path.clone().unwrap());

            if db_path.is_some() && wf_dir.is_some() {

                let vehicle = Epi2meSetup {
                    epi2os: os.unwrap(),
                    epi2path: path.unwrap(),
                    epi2db_path: PathBuf::from(&db_path.unwrap()),
                    epi2wf_dir: wf_dir.unwrap(),
                    epi4you_path: for_you_dir.unwrap(),
                    instances_path: instances_path,
                    arch: get_platformstr(),
                };

                return Some(vehicle);
            }
        }
    }

    return None;
}


fn get_4you_path(app_db_path: &PathBuf) -> Option<PathBuf> {
    let mut x = app_db_path.clone();

    x.push("import_export_4you");
    if x.exists() {
        println!("\t4you folder exists at [{}]", x.display());
        return Some(x.clone());
    } else {
        let create = fs::create_dir(&x);
        if create.is_ok() {
            println!("\t4you folder created at [{}]", x.display());
            return Some(x.clone());
        }
        eprintln!("\tErr - failed to create folder at [{}]", x.display());
    }
    return None;
}


fn check_os_specific_db_path(home: &Option<PathBuf>, os_specific_path: &str, os_label: &str) -> Option<PathBuf> {
    let mut pb = home.clone().unwrap();
    pb.push(os_specific_path);
    if pb.exists() {
        println!("\t{} installation [{}]", os_label, pb.display());
        return extract_epi2me_path(&pb);
    }
    return None;
}

fn extract_epi2me_path(pb: &PathBuf) -> Option<PathBuf> {
    let app_db_path = PathBuf::from(json::config_json(&pb));
    if app_db_path.exists() && app_db_path.is_dir() {
        return Some(app_db_path);
    }
    return None;
}

fn get_appdb_path(app_db_path: &PathBuf) -> Option<PathBuf> {
    let mut x = app_db_path.clone();

    x.push("app.db");
    if x.exists() {
        println!("\tapp.db exists at [{}]", x.display());
        return Some(x.clone());
    }
    return None;
}

