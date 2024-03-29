use std::{fs::File, path::PathBuf, env};

use tar::{Builder, Archive};

use crate::{manifest::FileManifest, epi2me_db};


pub fn tar(wf_path: Option<&PathBuf>, tarfile: PathBuf, files: &Vec<FileManifest>, manifest: &PathBuf) {

    let tarfile = File::create(tarfile).unwrap();
    let mut a = Builder::new(tarfile);

    let epi2db = epi2me_db::find_db();
    let mut local_prefix = PathBuf::from("/");
    if wf_path.is_some() {
        local_prefix = wf_path.unwrap().to_owned();
    } else if epi2db.is_some() {
        local_prefix = epi2db.unwrap().epi2path;
    }
    let _ = env::set_current_dir(&local_prefix);

    for file in files {

        let mut file_to_tar = PathBuf::from(file.relative_path.clone());
        file_to_tar.push(&file.filename);

        println!("adding file [{}] to tarball", file_to_tar.as_os_str().to_str().unwrap());

        let _ = a.append_path(file_to_tar);
    }

    println!("writing manifest {:?}", manifest);
    //let _ = env::set_current_dir(&manifest.parent().unwrap());
    let _ = a.append_path(manifest);


    
}


pub fn untar(tarfile: &PathBuf) -> Option<PathBuf> {
    let local_prefix = epi2me_db::find_db().unwrap().epi4you_path;
    println!("untar of file [{:?}] into [{:?}]", tarfile, local_prefix);
    let _ = env::set_current_dir(&local_prefix);

    let file = File::open(tarfile);
    if file.is_ok() {
        let mut archive = Archive::new(file.unwrap());
        for (_i, file) in archive.entries().unwrap().enumerate() {
            let mut file = file.unwrap();

            let unp = file.unpack_in(&local_prefix);
            if unp.is_err() {
                eprintln!(" error {:?}", unp.err());
                return None;
            }
        }
        return Some(local_prefix);
    }

    return None;
}