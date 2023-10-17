use std::{fs::File, path::PathBuf};

use tar::Builder;

use crate::manifest::FileManifest;


pub fn tar(tarfile: PathBuf, files: Vec<FileManifest>) {

    let tarfile = File::create(tarfile).unwrap();
    let mut a = Builder::new(tarfile);

    for file in files {
        let _ = a.append_path(file.filename);
    }

    

}


pub fn _untar() {

}