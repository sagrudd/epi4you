use crate::json;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Epi2meSetup {
    pub epi2path: PathBuf,
    pub epi2db_path: PathBuf,
    pub epi2wf_dir: PathBuf,
    pub epi4you_path: PathBuf,
    pub instances_path: PathBuf,
    pub arch: String,
}

pub fn find_db() -> Option<Epi2meSetup> {
    println!("locating the EPI2ME app.db");

    let home_dir = home_dir()?;
    let mut path: Option<PathBuf> = None;

    let macos =
        check_os_specific_db_path(&home_dir, "Library/Application Support/EPI2ME/config.json");
    let linux = check_os_specific_db_path(&home_dir, ".config/EPI2ME/config.json");
    let default = check_os_specific_db_path(&home_dir, "epi2melabs");

    if macos.is_some() {
        path = macos;
    } else if linux.is_some() {
        path = linux;
    } else if default.is_some() {
        path = default;
    }

    let path = path?;
    let db_path = get_appdb_path(&path)?;
    let mut instances_path = path.clone();
    instances_path.push("instances");
    let wf_dir = get_epi2me_wfdir_path(&path)?;
    let for_you_dir = get_4you_path(&path)?;

    Some(Epi2meSetup {
        epi2path: path,
        epi2db_path: db_path,
        epi2wf_dir: wf_dir,
        epi4you_path: for_you_dir,
        instances_path,
        arch: String::from(std::env::consts::ARCH),
    })
}

fn get_4you_path(app_db_path: &PathBuf) -> Option<PathBuf> {
    let mut x = app_db_path.clone();
    x.push("import_export_4you");

    if x.exists() {
        println!("\t4you folder exists at [{}]", x.display());
        return Some(x);
    }

    match fs::create_dir(&x) {
        Ok(_) => {
            println!("\t4you folder created at [{}]", x.display());
            Some(x)
        }
        Err(_) => {
            eprintln!("\tErr - failed to create folder at [{}]", x.display());
            None
        }
    }
}

fn check_os_specific_db_path(home: &PathBuf, os_specific_path: &str) -> Option<PathBuf> {
    let mut pb = home.clone();
    pb.push(os_specific_path);
    if pb.exists() && pb.is_file() {
        println!("\tinstallation [{}]", pb.display());
        extract_epi2me_path(&pb)
    } else if pb.exists() && pb.is_dir() {
        println!("\tinstallation [{}]", pb.display());
        Some(pb)
    } else {
        None
    }
}

fn extract_epi2me_path(pb: &PathBuf) -> Option<PathBuf> {
    let app_db_path = PathBuf::from(json::config_json(pb));
    if app_db_path.exists() && app_db_path.is_dir() {
        Some(app_db_path)
    } else {
        None
    }
}

fn get_appdb_path(app_db_path: &PathBuf) -> Option<PathBuf> {
    let mut x = app_db_path.clone();
    x.push("app.db");
    if x.exists() {
        println!("\tapp.db exists at [{}]", x.display());
        Some(x)
    } else {
        None
    }
}

fn get_epi2me_wfdir_path(app_db_path: &PathBuf) -> Option<PathBuf> {
    let mut x = app_db_path.clone();
    x.push("workflows");
    if x.exists() {
        println!("\tworkflows folder exists at [{}]", x.display());
        Some(x)
    } else {
        None
    }
}
