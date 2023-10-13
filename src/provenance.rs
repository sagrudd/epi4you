use uuid::Uuid;
use chrono::prelude::*;

pub struct Epi2MeProvenance {
    pub id: String,
    pub action: String,
    pub timestamp: DateTime<Local>,
}

impl Default for Epi2MeProvenance {
    fn default() -> Epi2MeProvenance {

        Epi2MeProvenance {
            id: Uuid::new_v4().to_string(),
            action: String::from("undefined"),
            timestamp: Local::now(),
        }
    }
}

pub fn is_trusted() -> bool {
    return false;
}


pub fn append_provenance(who: String, when: String, host: String, path: String) {

}


pub fn check_file_checksums() {

}

pub fn check_file_checksum() {

}

pub fn check_manifest_signature() {
    
}