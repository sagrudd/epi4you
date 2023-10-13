use std::path::PathBuf;


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