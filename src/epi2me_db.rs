use std::{path::{PathBuf, Path}, fs::{self, create_dir_all}, env};

use home;
use path_clean::PathClean;
use polars_core::frame::DataFrame;
use ulid::Ulid;
use crate::{json, workflow, app_db, bundle};


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


pub fn get_tempdir() -> Option<PathBuf> {
    let x = find_db();
    if x.is_some() {
        let mut epi4you = x.unwrap().epi4you_path;
        let ulid_str = Ulid::new().to_string();
        epi4you.push(ulid_str);
        let status = create_dir_all(&epi4you);
        if status.is_ok() {
            return Some(epi4you);
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

pub fn epi2me_manager(epi2me: &Epi2meSetup, df: &DataFrame, list: &bool, runid: &Option<String>, twome: &Option<String>, force: &bool, bundlewf: &bool) {
    println!("epi2me.list == {}",*list);
    if *list {
        app_db::print_appdb(&df);
    } else {
        if runid.is_none() {
            println!("EPI2ME analysis twome archiving requires a --runid identifier (name or id)");
            return;
        } else {
            if !app_db::validate_db_entry(&runid.as_ref().unwrap().to_string(), &df) {
                return;
            }
        }

        let runid_str = &runid.as_ref().unwrap().to_string();
        let polardb = df.clone();

        if twome.is_none() {
            println!("EPI2ME twome archiving requires a --twome <file> target to writing to");
            return; 
        } else {
            let pb = PathBuf::from(twome.as_ref().unwrap());
            if pb.exists() {
                if pb.is_file() && !force {
                    println!("twome file specified already exists - either --force or use different name");
                    return;
                } else if pb.is_dir() {
                    println!("twome file is a directory - file is required");
                    return;
                } 
            }    
        }

        let mut bundle_workflow: Option<PathBuf> = None;
        if bundlewf == &true {
            // ensure that a workflow for bundling is intact ...
            bundle_workflow = app_db::validate_qualified_analysis_workflow(
                &runid_str.to_string(), 
                &polardb, &epi2me.epi2wf_dir,
            )
        }

        // if we are here we have a destination and a unique runid - let's sanity check the destination PATH
        // there is some broken logic as described in https://github.com/sagrudd/epi4you/issues/1
        let path = Path::new(twome.as_ref().unwrap());
        let mut absolute_path;
        if path.is_absolute() {
            absolute_path = path.to_path_buf();
        } else {
            absolute_path = env::current_dir().unwrap().join(path);
        }
        absolute_path = absolute_path.clean();
        println!("tar .2me archive to be written to [{:?}]", absolute_path);

        // we have a destination and a unique runid - let's package something ...
        bundle::export_desktop_run(&runid_str, &polardb, Some(absolute_path), bundle_workflow);
    }
}