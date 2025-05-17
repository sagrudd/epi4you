

use serde::Deserialize;

use crate::epi4you_errors::Epi4youError;



#[derive(Clone, Deserialize)]
pub struct Row<'a> {
    timestamp: &'a str,
    duration: &'a str,
    run_name: &'a str,
    status: &'a str,
    revision_id: &'a str,
    session_id: &'a str,
    command: &'a str,
}

impl<'a> Row<'a> {

    pub fn get_status(&self) -> String {
        return String::from(self.status);
    }

}


#[derive(Debug, Clone)]
pub struct NxfLogItem {
    pub timestamp: String,
    pub duration: String,
    pub run_name: String,
    pub status: String,
    pub revision_id: String,
    pub session_id: String,
    pub command: String,
}

impl NxfLogItem {

    pub fn init(row: Row) -> Result<NxfLogItem, Epi4youError> {

        return Ok(
            NxfLogItem {
                timestamp: row.timestamp.into(),
                duration: row.duration.into(),
                run_name: row.run_name.into(),
                status: row.status.into(),
                revision_id: row.revision_id.into(),
                session_id: row.session_id.into(),
                command: row.command.into(),
            }
        )
    }

}