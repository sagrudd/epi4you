use std::{fs::File, path::PathBuf, env};

use tar::Builder;

use crate::{manifest::FileManifest, epi2me_db};


pub fn tar(tarfile: PathBuf, files: &Vec<FileManifest>, manifest: &PathBuf) {

    let tarfile = File::create(tarfile).unwrap();
    let mut a = Builder::new(tarfile);

    let local_prefix = epi2me_db::find_db().unwrap().epi2path;
    let _ = env::set_current_dir(&local_prefix);

    for file in files {

        let mut file_to_tar = PathBuf::from(file.relative_path.clone());
        file_to_tar.push(&file.filename);

        println!("adding file [{}] to tarball", file_to_tar.as_os_str().to_str().unwrap());

        let _ = a.append_path(file_to_tar);
    }

    println!("writing manifest {:?}", manifest);
    let _ = a.append_path(manifest);

}


pub fn _untar() {

}