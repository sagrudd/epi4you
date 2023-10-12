use rusqlite::{Connection, Result};
use polars::prelude::*;
use std::env;
use std::path::PathBuf;

#[allow(dead_code)]
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

#[allow(non_snake_case)]
pub fn load_db(path: PathBuf) -> Result<DataFrame, rusqlite::Error> {


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

    let mut nf_run_vec: Vec<Epi2MeAnalysis> = Vec::new();

    for nextflow_run in analysis_iter {
        let my_nextflow_run = nextflow_run.unwrap();
        nf_run_vec.push(my_nextflow_run);
    }

    // and wrangle observations into a dataframe
    let df: DataFrame = struct_to_dataframe!(nf_run_vec, [id,
        path,
        name,
        status,
        workflowRepo,
        workflowUser,
        workflowCommit, workflowVersion, createdAt, updatedAt]).unwrap();

    Ok(df)
}

pub fn validate_db_entry(runid: String, polardb: &DataFrame) -> bool {
    // is runid in name field and unique

    // if this is not unique then list the id options and suggestion to list

    // is runid in id field and unique

    return false;
}



pub fn print_appdb(df: &DataFrame) {
    env::set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
    env::set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES","1");
    let df2 = df!(
        "id" => df.column("id").unwrap(),
        "name" => df.column("name").unwrap(),
        "workflowRepo" => df.column("workflowRepo").unwrap(),
        "createdAt" => df.column("createdAt").unwrap(),
    );

    if df2.is_ok() {
        println!("{:?}", df2.unwrap());
    }
}


macro_rules! struct_to_dataframe {
    ($input:expr, [$($field:ident),+]) => {
        {
            // Extract the field values into separate vectors
            $(let mut $field = Vec::new();)*

            for e in $input.into_iter() {
                $($field.push(e.$field);)*
            }
            df! {
                $(stringify!($field) => $field,)*
            }
        }
    };
}
pub(crate) use struct_to_dataframe;