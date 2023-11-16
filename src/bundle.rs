
use std::{env, path::PathBuf};

use glob::glob;

use polars_core::prelude::DataFrame;

use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use stringreader::StringReader;
use std::fs::File;
use std::io::{BufReader, Read};


use crate::epi2me_db;
use crate::json::{get_manifest_str, write_manifest_str};
use crate::manifest::{MANIFEST_JSON, Epi2MeManifest, touch_manifest};
use crate::{manifest::{load_manifest_from_tarball, get_manifest, Epi2MeContent, FileManifest}, app_db, epi2me_tar};


fn sha256_digest(path: &str) -> String {

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

    HEXUPPER.encode(context.finish().as_ref())
}


pub fn sha256_str_digest(payload_str: &str) -> String {

    let streader = StringReader::new(payload_str);
    let mut reader = BufReader::new(streader);

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    HEXUPPER.encode(context.finish().as_ref())
}


pub fn export_desktop_run(runid: &String, polardb: &DataFrame, destination: Option<PathBuf>, _bundlewf: Option<PathBuf>) {

    let source_opt = Some(app_db::get_qualified_analysis_path(&runid, polardb));
    let local_prefix = epi2me_db::find_db().unwrap().epi2path;

    if source_opt.is_some() && destination.is_some() {
        let source = source_opt.unwrap();
        println!("packing [{:?}] into .2me format archive", &source.clone());

        // identify a manifest file into which details will be written
        let mut manifest = get_manifest(&source).unwrap();

        let zz = app_db::get_analysis_struct(runid, polardb);

        if zz.is_some() {

            // we need two paths here - the manifest is either new or being reused ...
            if is_nascent_manifest(&manifest) {
                // prepare_new_manifest();
                println!("This is a nascent manifest");
            } else {
                // otherwise this EPI2ME object has already been packaged / unpackaged ...
                println!("This manifest is being reused");
                // add some history to the manifest to note that it is being repacked ...

                touch_manifest(&mut manifest)
            }

            let mut vehicle = zz.unwrap();

            // load the files into the Epi2meDesktopAnalysis struct
            //let mut files = Vec::<FileManifest>::new();

            let globpat = &source.clone().into_os_string().into_string().unwrap();
            let result = [&globpat, "/**/*.*"].join("");

            println!("fishing for files at [{}]", result);

            let _ = env::set_current_dir(&globpat);

            let mut files_size: u64 = 0;

            for entry in glob(&result).expect("Failed to read glob pattern") {
                if entry.is_ok() {
                    let e = entry.unwrap();
                    let fname =  &e.file_name().and_then(|s| s.to_str());
                    if e.is_file() && !fname.unwrap().contains("4u_manifest.json") { // don't yet package the manifest - it'll be added later
                        let fp = e.as_os_str().to_str().unwrap();

                        let mut parent = e.clone();
                        let _ = parent.pop();


                        let relative_path = clip_relative_path(&e, &local_prefix);
                        //println!("{}", &fp);

                        let checksum = sha256_digest(&fp);
                        
                        //println!("file [{}] with checksum [{}]", &fp, &vv);
                        let file_size = e.metadata().unwrap().len();
                        files_size += file_size;

                        vehicle.files.push(FileManifest {
                            filename: String::from(e.file_name().unwrap().to_os_string().to_str().unwrap()),
                            relative_path: String::from(relative_path.clone().to_string_lossy().to_string()),
                            size: file_size,
                            md5sum: checksum,
                        })
                    }
                }
            }

            let filecount = vehicle.files.len();
            let filecontext = vehicle.files.clone();

            manifest.payload.push( Epi2MeContent::Epi2mePayload(vehicle) );
            manifest.filecount = u8::try_from(filecount).unwrap(); 
            manifest.files_size = files_size;  

            println!("{:?}", get_manifest_str(&manifest).as_str());

            let manifest_signature = sha256_str_digest(get_manifest_str(&manifest).as_str());
            manifest.signature = manifest_signature;
            
            // add the file manifest to the manifest

            let mut manifest_pb = source.clone();
            manifest_pb.push(MANIFEST_JSON);
            write_manifest_str(&manifest, &manifest_pb);

            // as per https://github.com/sagrudd/epi4you/issues/1 - ensure that destination is not in source
            let dest = destination.unwrap();
            let common_prefix = &dest.strip_prefix(source);
            if !common_prefix.is_err() {
                eprintln!("Destination is a child of source - this will not work!");
                return;
            }

            // tar up the contents specified in the manifest
            epi2me_tar::tar(dest, &filecontext, &get_relative_path(&manifest_pb, &local_prefix));
        }
    }
}


fn is_nascent_manifest(manifest: &Epi2MeManifest) -> bool {
    // if the manifest has a non-default checksum then it is unlikely to be new
    if manifest.signature == String::from("undefined") {
        return true;
    }
    return false;
}


fn clip_relative_path(e: &PathBuf, local_prefix: &PathBuf) -> PathBuf {
    let mut relative_path = get_relative_path(e, local_prefix);
    let _ = relative_path.pop();
    return relative_path;
}

fn get_relative_path(e: &PathBuf, local_prefix: &PathBuf) -> PathBuf {
    PathBuf::from(e.strip_prefix(local_prefix).unwrap())
}




pub fn _import_2me_bundle(twome: PathBuf) {

    // load manifest from tarball
    let manifest = load_manifest_from_tarball(twome);
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