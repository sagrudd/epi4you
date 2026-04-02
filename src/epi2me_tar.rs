use std::{fs::File, path::PathBuf};

use tar::Builder;

use crate::{epi2me_db, xmanifest::FileManifest};

pub fn tar(
    wf_path: Option<&PathBuf>,
    tarfile: PathBuf,
    files: &Vec<FileManifest>,
    manifest: &PathBuf,
) {
    let tarfile = File::create(tarfile).unwrap();
    let mut a = Builder::new(tarfile);

    let mut local_prefix = PathBuf::from("/");
    if wf_path.is_some() {
        local_prefix = wf_path.unwrap().to_owned();
    } else if let Some(epi2db) = epi2me_db::find_db() {
        local_prefix = epi2db.epi2path;
    }

    for file in files {
        let mut file_to_tar = local_prefix.join(&file.relative_path);
        file_to_tar.push(&file.filename);

        println!(
            "adding file [{}] to tarball",
            file_to_tar.as_os_str().to_str().unwrap()
        );

        let _ = a.append_path(file_to_tar);
    }

    println!("writing manifest {:?}", manifest);
    let _ = a.append_path(manifest);
}

/*
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

    */
