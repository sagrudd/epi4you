use std::collections::HashMap;

use serde::{ser::SerializeMap, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct ProgressItem {
    pub status: String,
    pub tag: String,
    pub total: u16,
    pub complete: u16,
}

#[derive(Debug, Deserialize)]
pub struct ProgressJson {
    // #[serde(rename = ulid_str)]
    pub name: String,
    pub key: HashMap<String, ProgressItem>,
}

impl Serialize for ProgressJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let item_name = self.name.to_owned();
        let mut struct_ser = serializer.serialize_map(Some(1))?;
        struct_ser.serialize_entry(&item_name, &self.key)?;
        struct_ser.end()
    }
}

