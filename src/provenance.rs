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
            user: whoami::username(),
            timestamp: Local::now().to_string(),
        }
    }
}



pub fn append_provenance(what: String, when: Option<String>, _host: Option<String>, _path: String) -> Epi2MeProvenance {

    let _luser = String::new(); // how do we get current user in platform independent way?

    let _lhost: String = String::new(); // how do we get hostname?

    let mut lwhen = Local::now().to_string();
    if when.is_some() {
        lwhen = when.unwrap();
    } 

    return Epi2MeProvenance{
        action: String::from(what),
        timestamp: lwhen,
        ..Default::default()
    }

}
