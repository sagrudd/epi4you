use std::path::PathBuf;

use crate::manifest::{load_manifest_from_tarball, get_manifest};


pub fn export_desktop_run(source: Option<PathBuf>, destination: Option<PathBuf>) {

    if source.is_some() && destination.is_some() {
        println!("packing [{:?}] into .2me format archive", source.clone().unwrap());

        // identify a manifest file into which details will be written
        let manifest = get_manifest(source);

        // identify the files that will be bundled into the archive ...
        let file_list = list_desktop_files();
        for file in file_list {

        }

    }
}


pub fn export_nextflow_run() {

}


pub fn import_2me_bundle() {

    // load manifest from tarball
    let manifest = load_manifest_from_tarball();
    if manifest.is_none() {
        println!("unable to extract EPI2ME manifest from tarball");
        return;
    }

    // check that workflow name and id are unique

    // check expected size of archive ... is there sufficient disk space

    // unpack the files into a temporary directory

    // are the target files appropriately signed?

    // are there any target files that there shouldn't be?

    // are there any missing files?

    // is there a signature and can we trust the authenticity of the dataset?

    // move temporarary directory into EPI2ME desktop directory

    // make app.db update
    
}




fn list_desktop_files() -> Vec<String> {
    let mut xx: Vec<String> = Vec::new();


    return xx;
}