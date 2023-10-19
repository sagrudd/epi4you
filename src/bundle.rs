
use std::{env, path::PathBuf};

use glob::glob;

use polars_core::prelude::DataFrame;

use data_encoding::HEXUPPER;
use ring::digest::{Context, Digest, SHA256};
use std::fs::File;
use std::io::{BufReader, Read};


use crate::epi2me_db;
use crate::{manifest::{load_manifest_from_tarball, get_manifest, Epi2MeContent, FileManifest}, json::wrangle_manifest, app_db, epi2me_tar};


fn sha256_digest(path: &str) -> Option<Digest> {

    let input = File::open(path).unwrap();
    let mut reader = BufReader::new(input);

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    return Some(context.finish());
}


pub fn export_desktop_run(runid: &String, polardb: &DataFrame, destination: Option<PathBuf>, _bundlewf: Option<PathBuf>) {

    let source = Some(app_db::get_qualified_analysis_path(&runid, polardb));

    if source.is_some() && destination.is_some() {
        println!("packing [{:?}] into .2me format archive", &source.clone().unwrap());

        // identify a manifest file into which details will be written
        let mut manifest = get_manifest(&source).unwrap();

        let zz = app_db::get_analysis_struct(runid, polardb);

        if zz.is_some() {

            let mut vehicle = zz.unwrap();

            // load the files into the Epi2meDesktopAnalysis struct
            //let mut files = Vec::<FileManifest>::new();

            let globpat = &source.unwrap().into_os_string().into_string().unwrap();
            let result = [&globpat, "/**/*.*"].join("");

            println!("fishing for files at [{}]", result);

            let _ = env::set_current_dir(&globpat);

            for entry in glob(&result).expect("Failed to read glob pattern") {
                if entry.is_ok() {
                    let e = entry.unwrap();
                    if e.is_file() {
                        let fp = e.as_os_str().to_str().unwrap();

                        let mut parent = e.clone();
                        let _ = parent.pop();

                        let local_prefix = epi2me_db::find_db().unwrap().epi2path;

                        let mut relative_path = PathBuf::from(e.strip_prefix(local_prefix).unwrap());
                        let _ = relative_path.pop();

                    //println!("{}", &fp);

                    let checksum = sha256_digest(&fp).unwrap();
                    let vv = HEXUPPER.encode(checksum.as_ref());
                    //println!("file [{}] with checksum [{}]", &fp, &vv);

                    vehicle.files.push(FileManifest {
                        filename: String::from(e.file_name().unwrap().to_os_string().to_str().unwrap()),
                        relative_path: String::from(relative_path.clone().to_string_lossy().to_string()),
                        size: e.metadata().unwrap().len(),
                        md5sum: vv,
                    })
                }
            }
            }

            epi2me_tar::tar(destination.unwrap(), &vehicle.files);
            manifest.payload.push( Epi2MeContent::Epi2mePayload(vehicle) );

            //wrangle_manifest(&manifest);

            
        }
    }
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




fn _list_desktop_files() -> Vec<String> {

    /*
        for the purposes of signing the archive with a checksum; we need to load in the information in a strictly controlled
        way - just alphabetical should be fine; maintain information on the relative file path and filesize; with this it
        should be trivial to ensure that the integrity of a packaged container is maintained
    
     */


    return Vec::<String>::new();
}