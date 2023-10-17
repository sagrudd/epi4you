use std::path::PathBuf;

use polars_core::prelude::DataFrame;

use crate::{manifest::{load_manifest_from_tarball, get_manifest, Epi2meDesktopAnalysis, Epi2MeContent}, json::wrangle_manifest, app_db};


pub fn export_desktop_run(runid: &String, polardb: &DataFrame, destination: Option<PathBuf>, _bundlewf: Option<PathBuf>) {

    let source = Some(app_db::get_qualified_analysis_path(&runid, polardb));

    if source.is_some() && destination.is_some() {
        println!("packing [{:?}] into .2me format archive", &source.clone().unwrap());

        // identify a manifest file into which details will be written
        let mut manifest = get_manifest(&source).unwrap();

        let zz = package_desktop_analysis(&source.clone().unwrap());

        manifest.payload.push( Epi2MeContent::Epi2mePayload(zz) );

        wrangle_manifest(&manifest);
    }
}


fn package_desktop_analysis(_source: &PathBuf) -> Epi2meDesktopAnalysis {
        // identify what is being packed into the tarball

        let payload_a = Epi2meDesktopAnalysis{
            ..Default::default()
        };

        // identify the files that will be bundled into the archive ...
        let file_list = list_desktop_files();
        for _file in file_list {

        }

        return payload_a;
}


pub fn _import_2me_bundle() {

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

    /*
        for the purposes of signing the archive with a checksum; we need to load in the information in a strictly controlled
        way - just alphabetical should be fine; maintain information on the relative file path and filesize; with this it
        should be trivial to ensure that the integrity of a packaged container is maintained
    
     */


    return Vec::<String>::new();
}