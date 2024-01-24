use std::env;

use polars_core::prelude::*;
use polars::prelude::*;

use crate::{workflow::Workflow, app_db::Epi2MeAnalysis, nextflow::{NxfLogItem, NextflowAssetWorkflow}, docker::{Container, DockerContainer}};


#[macro_export]
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

pub fn workflow_vec_to_df(wfs: Vec<Workflow>) -> DataFrame {
    struct_to_dataframe!(wfs, [project,
        name,
        version]).unwrap()
}


pub fn docker_vec_to_df(wfs: Vec<Container>) -> DataFrame {
    struct_to_dataframe!(wfs, [workflow,
        version,
        dockcont]).unwrap()
}


pub fn dockercontainer_vec_to_df(wfs: Vec<DockerContainer>) -> DataFrame {
    struct_to_dataframe!(wfs, [repository,                          
        tag,                           
        image,
        created,    
        size]).unwrap()
}


#[allow(non_snake_case)]
pub fn analysis_vec_to_df(nf_run_vec: Vec<Epi2MeAnalysis>) -> DataFrame {
    struct_to_dataframe!(nf_run_vec, [id,
        path,
        name,
        status,
        workflowRepo,
        workflowUser,
        workflowCommit, 
        workflowVersion, 
        createdAt, 
        updatedAt]).unwrap()
}

pub fn nextflow_vec_to_df(vec: Vec<NxfLogItem>) -> DataFrame {
    struct_to_dataframe!(vec, [timestamp,
        duration,
        run_name,
        status,
        revision_id,
        session_id,
        command]).unwrap()
}

pub fn nf_wf_vec_to_df(vec: Vec<NextflowAssetWorkflow>) -> DataFrame {
    struct_to_dataframe!(vec, [workflow,
        path,
        version]).unwrap()
}


pub fn print_polars_df(df: &DataFrame) {
    env::set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
    env::set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES","1");
    env::set_var("POLARS_FMT_MAX_ROWS", "20");

    println!("{:?}", df);
}


pub fn filter_df_by_value(df: &DataFrame, column: &String, value: &String) -> Result<DataFrame, PolarsError> {
    return df.clone()
    .lazy()
    .filter(col(column).is_in(lit(Series::from_iter(vec![String::from(value)])))).collect();
}


pub fn get_zero_val(df: &DataFrame, col: &String) -> String {
    let idx = df.find_idx_by_name(col).unwrap();
    let ser = df.select_at_idx(idx).unwrap().clone();
    let chunked_array: Vec<Option<&str>> = ser.utf8().unwrap().into_iter().collect();
    return String::from(chunked_array[0].unwrap());
}

pub fn two_field_filter(df: &DataFrame, c1: &String, c1val: &String, c2: &String, c2val: &String) -> Option<DataFrame> {
    let first_field = filter_df_by_value(&df, c1, c1val);
    if first_field.is_ok() {
        let first_field_df = first_field.unwrap();
        // print_polars_df(&project_df);
        let second_field = filter_df_by_value(&first_field_df, c2, c2val);
        if second_field.is_ok() {
            let second_field_df = second_field.unwrap();
            //print_polars_df(&second_field_df);
            return Some(second_field_df);
        }
    }
    return None;
}