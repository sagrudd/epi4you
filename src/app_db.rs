use rusqlite::{Connection, Result};

use std::path::PathBuf;

#[allow(non_snake_case)]
#[derive(Debug)]
struct Epi2MeAnalysis {
    id: String  ,
    path: String,
    name: String,
    status: String,
    workflowRepo: String,
    workflowUser: String,
    workflowCommit: String,
    workflowVersion: String,
    createdAt: String,
    updatedAt: String,
}

pub fn load_db(path: PathBuf) -> Result<(), rusqlite::Error> {


    let lookup = String::from("SELECT id, path, name, status, workflowRepo, workflowUser, workflowCommit, workflowVersion, createdAt, updatedAt FROM bs");

    let connection = Connection::open(path)?;

    let mut stmt = connection.prepare(&lookup)?;
    let analysis_iter = stmt.query_map([], |row| {
        Ok(Epi2MeAnalysis {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            status: row.get(3)?,
            workflowRepo: row.get(4)?,
            workflowUser: row.get(5)?,
            workflowCommit: row.get(6)?,
            workflowVersion: row.get(7)?,
            createdAt: row.get(8)?,
            updatedAt: row.get(9)?,
        })
    })?;

    for person in analysis_iter {
        let my_person = person.unwrap();
        println!("Found analysis result {:?} || {:?}", &my_person.id, &my_person.name);
    }

    Ok(())
}