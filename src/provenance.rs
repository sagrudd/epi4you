use uuid::Uuid;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Epi2MeProvenance {
    pub id: String,
    pub action: String,
    pub value: Option<String>,
    pub user: String,
    pub timestamp: String,
}


impl Epi2MeProvenance {
    pub fn init(what: String, value: Option<String>) -> Self {

        let lwhen = Local::now().to_string();

        Epi2MeProvenance {
            id: Uuid::new_v4().to_string(),
            action: String::from(what),
            value: value,
            user: whoami::username(),
            timestamp: lwhen,
        }
    }
}