use rusqlite::{Connection, Result};
use polars::prelude::*;
use std::env;
use std::path::PathBuf;
use crate::manifest::Epi2meDesktopAnalysis;
use crate::workflow;


#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Debug)]
struct Epi2MeAnalysis {
    id: String,
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


fn filter_df_by_value(df: &DataFrame, column: &String, value: &String) -> Result<DataFrame, PolarsError> {

    return df.clone()
    .lazy()
    .filter(col(column).is_in(lit(Series::from_iter(vec![String::from(value)])))).collect();

}

fn get_db_id_entry(runid: &String, polardb: &DataFrame) -> Result<DataFrame, PolarsError> {
    // is runid in name field and unique
    let df = filter_df_by_value(polardb, &String::from("name"), runid);
    let df2 = filter_df_by_value(polardb, &String::from("id"), runid);
    return df.unwrap().vstack(&df2.unwrap());
}

fn get_zero_val(df: &DataFrame, col: &String) -> String {
    let idx = df.find_idx_by_name(col).unwrap();
    let ser = df.select_at_idx(idx).unwrap().clone();
    let chunked_array: Vec<Option<&str>> = ser.utf8().unwrap().into_iter().collect();
    return String::from(chunked_array[0].unwrap());
}

pub fn get_qualified_analysis_path(runid: &String, polardb: &DataFrame) -> PathBuf {
    let stacked = get_db_id_entry(runid, polardb).unwrap();
    return PathBuf::from(get_zero_val(&stacked, &String::from("path")));
}


pub fn validate_qualified_analysis_workflow(runid: &String, polardb: &DataFrame, epi2wf_dir: &PathBuf) -> Option<PathBuf> {
    let stacked = get_db_id_entry(runid, polardb).unwrap();

    let workflow_user = get_zero_val(&stacked, &String::from("workflowUser"));
    let workflow_repo = get_zero_val(&stacked, &String::from("workflowRepo"));
    
    let mut workflow: String = String::new();
    
    workflow.push_str(&workflow_user);
    workflow.push_str(&std::path::MAIN_SEPARATOR.to_string());
    workflow.push_str(&workflow_repo);
    
    println!("repo {}", workflow);

    // let's check that the path exists ...
    let wfdir_exists = workflow::check_defined_wfdir_exists(epi2wf_dir, &workflow_user, &workflow_repo);
    if wfdir_exists.is_some() {
        return wfdir_exists;
    }

    return None;
}


pub fn validate_db_entry(runid: &String, polardb: &DataFrame) -> bool {

    let stacked = get_db_id_entry(runid, polardb);
    println!("{:?}",stacked);

    let row_count = &stacked.as_ref().unwrap().height(); // &stacked.unwrap().height();
    if row_count == &(1 as usize) {
        return true;
    } else if row_count == &(0 as usize) {
        println!("unable to resolve the analysis name; please check available analyses");
        return false;
    } else if row_count > &(1 as usize) {
        println!("supplied id name is ambiguous - please try to refine the analysis identifier");
        print_appdb(&stacked.unwrap().clone());
    }

    return false;
}


pub fn get_analysis_struct(runid: &String, polardb: &DataFrame) -> Option<Epi2meDesktopAnalysis> {
    if validate_db_entry(runid, polardb) {
        let stacked = get_db_id_entry(runid, polardb).unwrap();
        // this is obligate pass due to previous validation

        let x = Epi2meDesktopAnalysis { 
            id: get_zero_val(&stacked, &String::from("id")),
            path: get_zero_val(&stacked, &String::from("path")),
            name: get_zero_val(&stacked, &String::from("name")),
            status: get_zero_val(&stacked, &String::from("status")),
            workflowRepo: get_zero_val(&stacked, &String::from("workflowRepo")),
            workflowUser: get_zero_val(&stacked, &String::from("workflowUser")),
            workflowCommit: get_zero_val(&stacked, &String::from("workflowCommit")),
            workflowVersion: get_zero_val(&stacked, &String::from("workflowVersion")),
            createdAt: get_zero_val(&stacked, &String::from("createdAt")),
            updatedAt: get_zero_val(&stacked, &String::from("updatedAt")),
            ..Default::default() };
        return Some(x);
    }

    return None;
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

