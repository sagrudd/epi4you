use uuid::Uuid;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct Epi2MeProvenance {
    pub id: String,
    pub action: String,
    pub user: String,
    pub timestamp: String,
}

impl Default for Epi2MeProvenance {
    fn default() -> Epi2MeProvenance {

        Epi2MeProvenance {
            id: Uuid::new_v4().to_string(),
            action: String::from("undefined"),
            user: String::from("unknown"),
            timestamp: Local::now().to_string(),
        }
    }
}

pub fn is_trusted() -> bool {
    return false;
}


pub fn append_provenance(what: String, when: Option<String>, host: Option<String>, path: String) -> Epi2MeProvenance {

    let luser = String::new(); // how do we get current user in platform independent way?

    let lhost: String = String::new(); // how do we get hostname?

    let mut lwhen = String::new();
    if when.is_some() {
        lwhen = when.unwrap();
    } else {
        lwhen = Local::now().to_string();
    }

    return Epi2MeProvenance{
        action: String::from(what),
        timestamp: lwhen,
        ..Default::default()
    }

}


pub fn check_file_checksums() {

}

pub fn check_file_checksum() {

}

pub fn check_manifest_signature() {
    
}