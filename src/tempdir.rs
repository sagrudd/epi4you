use std::{path::PathBuf, fs::{create_dir_all, self}, fmt};

use ulid::Ulid;

use crate::epi2me_db::find_db;


fn form_tempdir(temp_path: PathBuf) -> Option<TempDir> {
    let tempdir = TempDir{path: PathBuf::from(&temp_path)};
    let status = create_dir_all(temp_path);
    if status.is_ok() {
        println!("using tempdir at [{}]", &tempdir);
        return Some(tempdir);
    }
    eprintln!("unable to create temporary directory ...");
    return None;
}


pub fn get_named_tempdir(temp_subdir: &String) -> Option<TempDir> {
    let mut epi4you = find_db().unwrap().epi4you_path;
    epi4you.push(temp_subdir);
    return form_tempdir(epi4you)
}


pub fn get_tempdir() -> Option<TempDir> {
    let ulid_str = Ulid::new().to_string();
    return get_named_tempdir(&ulid_str);
}

#[derive(Clone)]
pub struct TempDir {
    pub path: PathBuf,
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let str = self.path.as_os_str().to_str().unwrap();
        // if has been cloned then may not exist -- test for this
        if self.path.exists() {
            println!("Dropping TempDir with path `{}`!", str);
            let cleanup = fs::remove_dir_all(&self.path);
            if cleanup.is_err() {
                eprintln!("failed to cleanup temporary directory at [{}]", str);
            } 
        }
    }
}

impl fmt::Display for TempDir {
   // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let str = self.path.as_os_str().to_str().unwrap();
        write!(f, "{}", str)
    }
}