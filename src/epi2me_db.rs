use std::path::PathBuf;

use home;
use crate::json;

pub fn find_db() -> Option<PathBuf> {
    println!("locating the EPI2ME app.db");

    let home_dir = home::home_dir();
    if home_dir.is_some() {

        let macos = check_os_specific_db_path(&home_dir, "Library/Application Support/EPI2ME/config.json", "macOS");
        if macos.is_some() {
            return macos;
        }

        let linux = check_os_specific_db_path(&home_dir, ".config/EPI2ME/config.json", "Linux");
        if linux.is_some() {
            return linux;
        }


    }

    return None;
}


fn check_os_specific_db_path(home: &Option<PathBuf>, os_specific_path: &str, os_label: &str) -> Option<PathBuf> {
    let mut pb = home.clone().unwrap();
    pb.push(os_specific_path);
    if pb.exists() {
        println!("\t{} installation [{}]", os_label, pb.display());
        return extract(&pb);
    }
    return None;
}

fn extract(pb: &PathBuf) -> Option<PathBuf> {
    let mut app_db_path = PathBuf::from(json::config_json(&pb));
    app_db_path.push("app.db");
    
    if app_db_path.exists() {
        println!("\tapp.db exists at [{}]", app_db_path.display());
        return Some(app_db_path);
    }
    return None;
}