use serde::Deserialize;
use std::{fs::File, path::PathBuf};

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct SomeDataType {
    workingDirectory: String,
}

pub fn config_json(path_buf: &PathBuf) -> String {
    let json_file = File::open(path_buf).expect("file not found");
    let epi2me_setup: SomeDataType =
        serde_json::from_reader(json_file).expect("error while reading json");

    println!("\tjson parsed [workDir={}]", epi2me_setup.workingDirectory);
    epi2me_setup.workingDirectory
}
