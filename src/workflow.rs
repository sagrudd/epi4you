use std::{path::PathBuf, fs};
use glob::glob;
use crate::epi2me_db::Epi2meSetup;


pub fn get_epi2me_wfdir_path(app_db_path: &PathBuf) -> Option<PathBuf> {
    let mut x = app_db_path.clone();

    x.push("workflows");
    if x.exists() {
        println!("\tworkflows folder exists at [{}]", x.display());
        return Some(x.clone());
    }
    return None;
}

pub fn check_defined_wfdir_exists(wfdir: &PathBuf, user: &str, repo: &str) -> Option<PathBuf> {
    let mut x = wfdir.clone();
    x.push(user);
    x.push(repo);
    if x.exists() && x.is_dir() {
        println!("\tdefined workflow folder exists at [{}]", x.display());
        return Some(x.clone());
    }
    return None;
}


fn is_folder_wf_compliant(wffolder: &PathBuf) -> bool {
    let required_files = vec!["main.nf", "nextflow.config"];
    let mut counter = 0;
    let paths = fs::read_dir(wffolder).unwrap();
    for path in paths {
        let fname = &path.unwrap().file_name().to_string_lossy().to_string();
        if required_files.contains(&fname.as_str()) {
            println!("found {:?}", fname);
            counter += 1;
        }
    }
    if required_files.len() == counter {
        return true;
    }

    return false;
}


pub fn glob_path_by_wfname(epi2me: &Epi2meSetup, project: &String) -> Option<PathBuf> {

    let globpat = epi2me.epi2wf_dir.clone().into_os_string().into_string().unwrap();
    let result = [&globpat, "/*/", &project].join("");
    
    let gdata =  glob(&result).expect("Failed to read glob pattern");
    for entry in gdata {
        if entry.is_ok() {
            let entry_item = entry.unwrap();
            // ensure that the folder found is actually a nextflow folder (nanopore flavoured)
            if is_folder_wf_compliant(&entry_item) {
                println!("folder picked == {:?}", entry_item);
                return Some(entry_item)
            }
        }
    }
    // we can also assess whether project is a link to e.g. a nextflow based folder elsewhere on CLI


    return None;
}