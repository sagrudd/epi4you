use crate::nextflow::nextflow_log_item::NxfLogItem;
use polars::prelude::*;
use std::env;

#[macro_export]
macro_rules! struct_to_dataframe {
    ($input:expr, [$($field:ident),+]) => {
        {
            $(let mut $field = Vec::new();)*

            for e in $input.into_iter() {
                $($field.push(e.$field);)*
            }
            polars_core::df! {
                $(stringify!($field) => $field,)*
            }
        }
    };
}

pub fn nextflow_vec_to_df(vec: Vec<NxfLogItem>) -> DataFrame {
    struct_to_dataframe!(
        vec,
        [
            timestamp,
            duration,
            run_name,
            status,
            revision_id,
            session_id,
            command
        ]
    )
    .unwrap()
}

pub fn print_polars_df(df: &DataFrame) {
    env::set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
    env::set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES", "1");
    env::set_var("POLARS_FMT_MAX_ROWS", "20");

    println!("{:?}", df);
}
