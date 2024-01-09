use std::{path::{PathBuf, Path}, fs::{self}, env};
use home;
use path_clean::PathClean;
use polars_core::frame::DataFrame;
use crate::{json, workflow::{self, Workflow}, app_db, bundle};


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

    let home_dir = home::home_dir();
    if home_dir.is_some() {

        let mut path: Option<PathBuf> = None;

        /* There is a sequence through which the working directory should be sought - presence of a config file should be considered first
        - if there is no config file the location of the default $HOME/epi2melabs should also be allowed?
         */

        let macos = check_os_specific_db_path(&home_dir, "Library/Application Support/EPI2ME/config.json");
        let linux = check_os_specific_db_path(&home_dir, ".config/EPI2ME/config.json");
        let default: Option<PathBuf> = check_os_specific_db_path(&home_dir, "epi2melabs");
        
        if macos.is_some() {
            path = macos;
        } else if linux.is_some() {
            path = linux;
        } else if default.is_some() {
            path = default;
        }

        if path.is_some() {
            let db_path = get_appdb_path(&path.clone().unwrap());
            let mut instances_path = PathBuf::from(&path.clone().unwrap());
            instances_path.push("instances");
            let wf_dir = workflow::get_epi2me_wfdir_path(&path.clone().unwrap());
            let for_you_dir: Option<PathBuf> = get_4you_path(&path.clone().unwrap());

            if db_path.is_some() && wf_dir.is_some() {

                let vehicle = Epi2meSetup {
                    epi2path: path.unwrap(),
                    epi2db_path: PathBuf::from(&db_path.unwrap()),
                    epi2wf_dir: wf_dir.unwrap(),
                    epi4you_path: for_you_dir.unwrap(),
                    instances_path,
                    arch: String::from(std::env::consts::ARCH),
                };

                return Some(vehicle);
            }
        }
    }
    eprintln!("Unable to locate the EPI2ME application database ...");
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


fn check_os_specific_db_path(home: &Option<PathBuf>, os_specific_path: &str) -> Option<PathBuf> {
    let mut pb = home.clone().unwrap();
    pb.push(os_specific_path);
    if pb.exists() && pb.is_file() {
        println!("\tinstallation [{}]", pb.display());
        return extract_epi2me_path(&pb);
    } else if pb.exists() && pb.is_dir() {
        println!("\tinstallation [{}]", pb.display());
        return Some(pb);
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

pub fn epi2me_manager(epi2me: &Epi2meSetup, df: &DataFrame, list: &bool, runids: &Vec<String>, twome: &Option<String>, force: &bool, bundlewf: &bool) {
    println!("epi2me.list == {}",*list);
    if *list {
        app_db::print_appdb(&df);
        return;
    } 
    if runids.len() == 0 {
        println!("EPI2ME analysis twome archiving requires a --runid identifier (name or id)");
        return;
    } 
    if twome.is_none() {
        println!("EPI2ME twome archiving requires a --twome <file> target to writing to");
        return; 
    } 
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
    // let's iterate through the provided runids
    for runid in runids {
        if !app_db::validate_db_entry(&runid, &df) {
            return;
        }
    }

    let mut bundle_wfs: Vec<Workflow> = Vec::new();

    // if we're here the runids supplied are coherent
    for runid_str in runids {
        let polardb = df.clone();

        if bundlewf == &true {
            // ensure that a workflow for bundling is intact ...
            let wf = app_db::validate_qualified_analysis_workflow(
                &runid_str.to_string(), 
                &polardb, &epi2me.epi2path,
            );
            if wf.is_none() {
                eprintln!("This workflow may be an orphan - cannot continue");
                return;
            } 

            // do we need to add this wf to the vector of wfs?
            let wwf = wf.unwrap();
            let mut add_it = true;
            for xwf in &bundle_wfs {
                if xwf.project == wwf.project && xwf.name == wwf.name {
                    add_it = false;
                }
            }
            if add_it {
                println!("workflow instance found at [{:?}]", &wwf);
                bundle_wfs.push(wwf);
            }
        }

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
    
    bundle::export_desktop_run(runids, df, Some(absolute_path), &bundle_wfs);

    
}