
use std::path::PathBuf;

use polars_core::prelude::*;

use std::fs::remove_dir_all;

use crate::epi2me_db::{self};
use crate::epi2me_desktop_analysis::Epi2meDesktopAnalysis;
use crate::{epi2me_tar, xnf_parser};
use crate::epi2me_workflow::{get_relative_path, Epi2meWorkflow};
use crate::tempdir::{self, TempDir};

use crate::xmanifest::{Epi2MeManifest, MANIFEST_JSON};
use crate::{xmanifest::{Epi2MeContent, FileManifest}, app_db};





pub fn anyvalue_to_str(value: Option<&AnyValue>) -> String {
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






pub fn export_nf_workflow(wf_path: Option<&PathBuf>, source: &DataFrame, twome: &Option<String>, force: &bool) {
    // core path 
    //let core_path = epi2me_db::find_db().unwrap().epi2wf_dir;
    let local_prefix: PathBuf;
    if wf_path.is_some() {
        local_prefix = wf_path.unwrap().to_owned();
    } else {
        local_prefix = epi2me_db::find_db().unwrap().epi2path;
    }

    // create a temporary path for this export exploration
    let temp_dir: TempDir;
    if wf_path.is_some() {
        let mut pb = wf_path.unwrap().to_owned();
        pb.push("manifest");
        let tempdir = tempdir::form_tempdir(pb);
        if tempdir.is_none() {
            return;
        }
        temp_dir = tempdir.unwrap();
    } else {
        let tempdir = tempdir::get_tempdir();
        if tempdir.is_none() {
            return;
        }
        temp_dir = tempdir.unwrap();
    }

    let mut manifest = Epi2MeManifest::new(temp_dir.path.clone());
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

            let vehicle = Epi2meWorkflow::path_init(wf_path, &project, &name, &version);

            let filecount = vehicle.get_files().len();
            let filesize = vehicle.get_files_size();

            all_files.extend(vehicle.files.clone());

            manifest.payload.push(Epi2MeContent::Epi2meWf(vehicle) );
            manifest.filecount += u64::try_from(filecount).unwrap(); 
            manifest.files_size += filesize;  
        }   
    }

    println!("{}", manifest.to_string());

    // add the file manifest to the manifest
    let mut manifest_pb = PathBuf::from(&temp_dir.path);
    manifest_pb.push(MANIFEST_JSON);
    manifest.write(&manifest_pb);

    let dest = PathBuf::from(twome.clone().unwrap());
    /* 
    // as per https://github.com/sagrudd/epi4you/issues/1 - ensure that destination is not in source
    
    let common_prefix = &dest.strip_prefix(&current_prefix);
    if !common_prefix.is_err() {
        eprintln!("Destination is a child of source - this will not work!");
        return;
    }
    */
    if dest.exists() && !*force {
        eprintln!("destination archive already exists - cannot continue without `--force`")
    } else {
        // tar up the contents specified in the manifest
        epi2me_tar::tar(wf_path, dest, &all_files, &get_relative_path(&manifest_pb, &local_prefix));
    }

    // cleanup temporary content ...
    let sanitise = remove_dir_all(&temp_dir.path);
    if sanitise.is_err() {
        eprintln!("failed to cleanup working directory");
    }
}



pub fn export_cli_run(ulidstr: &String, source: PathBuf, temp_dir: TempDir, dest: PathBuf, nextflow_stdout: &String, timestamp: &String, force: &bool) {
    let epi2db = epi2me_db::find_db();
    let mut local_prefix = PathBuf::from("/");
    if epi2db.is_some() {
        local_prefix = epi2db.unwrap().epi2path;
    }
    
    let mut manifest = Epi2MeManifest::new(temp_dir.path.clone());
    let mut all_files: Vec<FileManifest> = Vec::new();

    println!("packing [{:?}] into .2me format archive", &source.clone());
    
    let mut vehicle = Epi2meDesktopAnalysis::init(ulidstr, &source, nextflow_stdout, timestamp);

    /* we need to parse some information here - at least the tuple of user//repo */


        manifest.note_packaged_analysis(
            &vec![String::from(&vehicle.workflowUser), 
                String::from(&vehicle.workflowRepo), String::from(&vehicle.name)].join("/"));

        // as per https://github.com/sagrudd/epi4you/issues/1 - ensure that destination is not in source
        let common_prefix = &dest.strip_prefix(&source);
        if !common_prefix.is_err() {
            eprintln!("Destination is a child of source - this will not work!");
            return;
        }

        
        vehicle.fish_files(&source, &local_prefix);
       
        all_files.extend(vehicle.get_files());
        manifest.filecount += u64::try_from(vehicle.get_files().len()).unwrap();
        manifest.files_size += &vehicle.get_files_size();
        manifest.payload.push( Epi2MeContent::Epi2mePayload(vehicle.clone()) );    


        println!("{:?}", &manifest);
    


    
    let mut manifest_pb = PathBuf::from(&temp_dir.path);
    manifest_pb.push(MANIFEST_JSON);
    manifest.write(&manifest_pb);

    // tar up the contents specified in the manifest
    if dest.exists() && !*force {
        eprintln!("destination archive already exists - cannot continue without `--force`")
    } else {
        // tar up the contents specified in the manifest
        epi2me_tar::tar(None, dest, &all_files, &get_relative_path(&manifest_pb, &local_prefix));
    }

}



pub fn export_desktop_run(wf_path: Option<&PathBuf>, runids: &Vec<String>, polardb: &DataFrame, destination: Option<PathBuf>, bundlewfs: &Vec<Epi2meWorkflow>) {
    let local_prefix = epi2me_db::find_db().unwrap().epi2path;
    if destination.is_none() {
        eprintln!("error with tarball destination ....");
        return;
    }
    let dest = destination.unwrap();
    // create a temporary path for this export exploration
    let tempdir = tempdir::get_tempdir();
    if tempdir.is_none() {
        return;
    }
    let temp_dir = tempdir.unwrap();
    println!("using tempdir at [{}]", &temp_dir);
    let mut manifest = Epi2MeManifest::new(temp_dir.path.clone()); 
    let mut all_files: Vec<FileManifest> = Vec::new();

    for runid in runids {
        let source_opt = Some(app_db::get_qualified_analysis_path(&runid, polardb));

        if source_opt.is_some() {
            let source = source_opt.unwrap();
            println!("packing [{:?}] into .2me format archive", &source.clone());
            let mut vehicle = Epi2meDesktopAnalysis::from_run_id(runid, polardb);

    

                // add some additional comments to the manifest as to what is happening at this moment ...
                //vehicle.
                manifest.note_packaged_analysis(
                    &vec![String::from(&vehicle.workflowUser), 
                        String::from(&vehicle.workflowRepo), String::from(&vehicle.name)].join("/"));

                // as per https://github.com/sagrudd/epi4you/issues/1 - ensure that destination is not in source
                let common_prefix = &dest.strip_prefix(&source);
                if !common_prefix.is_err() {
                    eprintln!("Destination is a child of source - this will not work!");
                    return;
                }

                vehicle.fish_files(&source, &local_prefix);
                all_files.extend(vehicle.files.clone());
    
                manifest.filecount += u64::try_from(vehicle.get_files().len()).unwrap();
                manifest.files_size += &vehicle.get_files_size();
                manifest.payload.push( Epi2MeContent::Epi2mePayload(vehicle) );
            
        }
    }

    // and package the workflows if specified ...
    if bundlewfs.len() > 0 {
        for wf in bundlewfs {
            println!("bundling [{:?}]", wf);

            let wf_vehicle = Epi2meWorkflow::path_init(wf_path, &wf.project, &wf.name, &wf.version);
            manifest.note_packaged_workflow(
                &vec![String::from(&wf.project), 
                    String::from( &wf.name), String::from(&wf.version)].join("/"));
            // println!("{:?}", wf_vehicle);
            all_files.extend(wf_vehicle.files.clone());
            manifest.filecount += u64::try_from(wf_vehicle.get_files().len()).unwrap();
            manifest.files_size += wf_vehicle.get_files_size();
            manifest.payload.push( Epi2MeContent::Epi2meWf(wf_vehicle) );     
        }
    }

    
    let mut manifest_pb = PathBuf::from(&temp_dir.path);
    manifest_pb.push(MANIFEST_JSON);
    manifest.write(&manifest_pb);

    // tar up the contents specified in the manifest
    epi2me_tar::tar(wf_path, dest, &all_files, &get_relative_path(&manifest_pb, &local_prefix));
}



