use std::path::PathBuf;
use std::fs::File;
use serde::{Serialize, Deserialize};
use serde_json;
extern crate serde;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct SomeDataType {
    workingDirectory: String,
    //globalConfig: String,
    //dockerPath: String,
    //expandInstances: bool,
    //expandWorkflows: bool,
    //localId: String
}

pub fn config_json(path_buf: &PathBuf) -> String {
    let json_file = File::open(path_buf).expect("file not found");

    let epi2me_setup: SomeDataType =
        serde_json::from_reader(json_file).expect("error while reading json");
    
    println!("\tjson parsed [workDir={}]", epi2me_setup.workingDirectory);
    return epi2me_setup.workingDirectory;
}