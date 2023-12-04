
use std::{env, path::PathBuf};

use glob::glob;
use polars_core::prelude::*;

use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use stringreader::StringReader;
use std::fs::{File, remove_dir_all};
use std::io::{BufReader, Read};

use crate::epi2me_db::{self, get_tempdir};
use crate::json::{get_manifest_str, write_manifest_str};
use crate::manifest::{MANIFEST_JSON, Epi2MeManifest, touch_manifest, Epi2meWorkflow, file_manifest_size, manifest_note_packaged_analysis, manifest_note_packaged_workflow};
use crate::workflow::{self, check_defined_wfdir_exists, Workflow};
use crate::{manifest::{get_manifest, Epi2MeContent, FileManifest}, app_db, epi2me_tar};


pub fn sha256_digest(path: &str) -> String {

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


fn anyvalue_to_str(value: Option<&AnyValue>) -> String {
    if value.is_some() {
        let vstr = value.unwrap().to_string();
        if vstr.starts_with("\"") && vstr.ends_with("\"") {
            let mut chars = vstr.chars();
            chars.next();
            chars.next_back();
            return String::from(chars.as_str());
        }
        return vstr;
    }
    return String::from("Err!");
}

pub fn get_workflow_vehicle(project: &String, name: &String, version: &String) -> Epi2meWorkflow {
    let local_prefix = epi2me_db::find_db().unwrap().epi2path;
    let mut vehicle = workflow::get_workflow_struct(&project, &name, &version);
            
    println!("{:?}", vehicle);

    let wf_path = check_defined_wfdir_exists(&local_prefix, &project, &name);
    println!("Mining files from [{:?}]", wf_path);

    vehicle.files = fish_files(&wf_path.unwrap(), &local_prefix);
    return vehicle;
}





pub fn export_nf_workflow(source: &DataFrame, twome: &Option<String>, force: &bool) {
    // core path 
    //let core_path = epi2me_db::find_db().unwrap().epi2wf_dir;
    let local_prefix = epi2me_db::find_db().unwrap().epi2path;

    // create a temporary path for this export exploration
    let tempdir = get_tempdir();
    if tempdir.is_none() {
        return;
    }
    let temp_dir = tempdir.unwrap();
    println!("using tempdir at [{:?}]", &temp_dir);
    let mut manifest = get_manifest(&temp_dir).unwrap();
    let mut all_files: Vec<FileManifest> = Vec::new();

    for idx in 0..source.height() {

        let single_row = source.get(idx);
        if single_row.is_some() {
            let unwrapped_row = single_row.unwrap();
            let project = anyvalue_to_str(unwrapped_row.get(0));
            let name = anyvalue_to_str(unwrapped_row.get(1));
            let version = anyvalue_to_str(unwrapped_row.get(2));

            let merged = vec![String::from(&project), String::from(&name)].join("/");
            println!("We have some data {}", merged);

            let vehicle = get_workflow_vehicle(&project, &name, &version);

            let filecount = vehicle.files.len();
            let filesize = file_manifest_size(&vehicle.files);

            all_files.extend(vehicle.files.clone());

            manifest.payload.push( Epi2MeContent::Epi2meWf(vehicle) );
            manifest.filecount += u64::try_from(filecount).unwrap(); 
            manifest.files_size += filesize;  
        }   
    }

    println!("{}", get_manifest_str(&manifest).as_str());

    let manifest_signature = sha256_str_digest(get_manifest_str(&manifest).as_str());
    manifest.signature = manifest_signature;

    // add the file manifest to the manifest
    let mut manifest_pb = temp_dir.clone();
    manifest_pb.push(MANIFEST_JSON);
    write_manifest_str(&manifest, &manifest_pb);

    // as per https://github.com/sagrudd/epi4you/issues/1 - ensure that destination is not in source
    let dest = PathBuf::from(twome.clone().unwrap());
    let common_prefix = &dest.strip_prefix(&local_prefix);
    if !common_prefix.is_err() {
        eprintln!("Destination is a child of source - this will not work!");
        return;
    }

    if dest.exists() && !*force {
        eprintln!("destination archive already exists - cannot continue without `--force`")
    } else {
        // tar up the contents specified in the manifest
        epi2me_tar::tar(dest, &all_files, &get_relative_path(&manifest_pb, &local_prefix));
    }

    // cleanup temporary content ...
    let sanitise = remove_dir_all(&temp_dir);
    if sanitise.is_err() {
        eprintln!("failed to cleanup working directory");
    }
}


fn fish_files(source: &PathBuf, local_prefix: &PathBuf) -> Vec<FileManifest> {

    let globpat = &source.clone().into_os_string().into_string().unwrap();
    let result = [&globpat, "/**/*.*"].join("");

    let mut files: Vec<FileManifest> = Vec::new();

    println!("fishing for files at [{}]", result);

    let _ = env::set_current_dir(&globpat);

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

                files.push(FileManifest {
                    filename: String::from(e.file_name().unwrap().to_os_string().to_str().unwrap()),
                    relative_path: String::from(relative_path.clone().to_string_lossy().to_string()),
                    size: file_size,
                    md5sum: checksum,
                })
            }
        }
    }
    return files;
}


pub fn export_desktop_run(runids: &Vec<String>, polardb: &DataFrame, destination: Option<PathBuf>, bundlewfs: &Vec<Workflow>) {
    let local_prefix = epi2me_db::find_db().unwrap().epi2path;
    if destination.is_none() {
        eprintln!("error with tarball destination ....");
        return;
    }
    let dest = destination.unwrap();
    // create a temporary path for this export exploration
    let tempdir = get_tempdir();
    if tempdir.is_none() {
        return;
    }
    let temp_dir = tempdir.unwrap();
    println!("using tempdir at [{:?}]", &temp_dir);
    let mut manifest = get_manifest(&temp_dir).unwrap();
    let mut all_files: Vec<FileManifest> = Vec::new();

    for runid in runids {
        let source_opt = Some(app_db::get_qualified_analysis_path(&runid, polardb));

        if source_opt.is_some() {
            let source = source_opt.unwrap();
            println!("packing [{:?}] into .2me format archive", &source.clone());
            let zz = app_db::get_analysis_struct(runid, polardb);
            if zz.is_some() {
                let mut vehicle = zz.unwrap();

                // add some additional comments to the manifest as to what is happening at this moment ...
                //vehicle.
                manifest_note_packaged_analysis(&mut manifest, 
                    &vec![String::from(&vehicle.workflowUser), 
                        String::from(&vehicle.workflowRepo), String::from(&vehicle.name)].join("/"));

                // as per https://github.com/sagrudd/epi4you/issues/1 - ensure that destination is not in source
                let common_prefix = &dest.strip_prefix(&source);
                if !common_prefix.is_err() {
                    eprintln!("Destination is a child of source - this will not work!");
                    return;
                }

                vehicle.files = fish_files(&source, &local_prefix);
                all_files.extend(vehicle.files.clone());
    
                manifest.filecount += u64::try_from(vehicle.files.len()).unwrap();
                manifest.files_size += file_manifest_size(&vehicle.files);
                manifest.payload.push( Epi2MeContent::Epi2mePayload(vehicle) );
            }
        }
    }

    // and package the workflows if specified ...
    if bundlewfs.len() > 0 {
        for wf in bundlewfs {
            println!("bundling [{:?}]", wf);

            let wf_vehicle = get_workflow_vehicle(&wf.project, &wf.name, &wf.version);
            manifest_note_packaged_workflow(&mut manifest, 
                &vec![String::from(&wf.project), 
                    String::from( &wf.name), String::from(&wf.version)].join("/"));
            // println!("{:?}", wf_vehicle);
            all_files.extend(wf_vehicle.files.clone());
            manifest.filecount += u64::try_from(wf_vehicle.files.len()).unwrap();
            manifest.files_size += file_manifest_size(&wf_vehicle.files);
            manifest.payload.push( Epi2MeContent::Epi2meWf(wf_vehicle) );     
        }
    }

    let manifest_signature = sha256_str_digest(get_manifest_str(&manifest).as_str());
    manifest.signature = manifest_signature;
    
    let mut manifest_pb = temp_dir.clone();
    manifest_pb.push(MANIFEST_JSON);
    write_manifest_str(&manifest, &manifest_pb);

    // tar up the contents specified in the manifest
    epi2me_tar::tar(dest, &all_files, &get_relative_path(&manifest_pb, &local_prefix));
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


