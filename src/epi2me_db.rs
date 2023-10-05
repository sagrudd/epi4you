use std::path::PathBuf;

use home;
use crate::json;

pub fn find_db() -> Option<PathBuf> {
    println!("locating the EPI2ME app.db");

    let home_dir = home::home_dir();
    if home_dir.is_some() {
        let mut pb = home_dir.unwrap();
        pb.push("Library/Application Support/EPI2ME/config.json");
        if pb.exists() {
            println!("\tmacOS installation [{}]", pb.display());

            let mut app_db_path = PathBuf::from(json::config_json(&pb));
            app_db_path.push("app.db");
            
            if app_db_path.exists() {
                println!("\tapp.db exists at [{}]", app_db_path.display());
                return Some(app_db_path);
            }
        }
    }

    return None;
}