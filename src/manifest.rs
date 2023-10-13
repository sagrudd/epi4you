use std::path::PathBuf;

use crate::provenance::Epi2MeProvenance;

static MANIFEST_JSON: &str = "4u_manifest.json";

pub struct FileManifest {
    pub filename: String,
    pub relative_path: String,
    pub size: String,
    pub md5sum: String,
}
impl Default for FileManifest {
    fn default() -> FileManifest {

        FileManifest {
            filename: String::from("undefined"),
            relative_path: String::from("undefined"),
            size: String::from("undefined"),
            md5sum: String::from("undefined"),
        }
    }
}


pub struct Epi2MeManifest {
    pub id: String,
    pub src_path: String,
    pub provenance: Vec<Epi2MeProvenance>,

    pub payload: Vec<FileManifest>,
    pub filecount: u8,
    pub files_size: u8,

    pub signature: String,
}



pub fn get_manifest(source: Option<PathBuf>) -> Option<Epi2MeManifest> {
    if source.is_some() {
        let mut manifest = source.clone().unwrap();
        manifest.push(MANIFEST_JSON);
        if !manifest.exists() {
            // we need to create one
        } else {
            // we should load the manifest
        }
    }
    return None;
}


pub fn load_manifest_from_tarball() -> Option<Epi2MeManifest> {

    return None;
}