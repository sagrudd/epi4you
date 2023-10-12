use std::path::PathBuf;

use crate::provenance::Epi2MeProvenance;



struct FileManifest {
    filename: String,
    relative_path: String,
    size: String,
    md5sum: String,
}

pub struct Epi2MeManifest {
    id: String,
    src_path: String,
    provenance: Vec<Epi2MeProvenance>,

    payload: Vec<FileManifest>,
    filecount: u8,
    files_size: u8,

    signature: String,
}


pub fn manifest_exists() -> bool {
    return false;
}

pub fn get_manifest(source: Option<PathBuf>) -> Option<Epi2MeManifest> {
    if source.is_some() {

        if !manifest_exists() {

        }
    }
    return None;
}


pub fn load_manifest_from_tarball() -> Option<Epi2MeManifest> {

    return None;
}