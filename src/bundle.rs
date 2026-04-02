use std::path::PathBuf;

use crate::epi2me_db::{self};
use crate::epi2me_desktop_analysis::Epi2meDesktopAnalysis;
use crate::epi2me_tar;
use crate::epi2me_workflow::get_relative_path;
use crate::tempdir::TempDir;

use crate::xmanifest::{Epi2MeContent, FileManifest};
use crate::xmanifest::{Epi2MeManifest, MANIFEST_JSON};

pub fn export_cli_run(
    ulidstr: &String,
    source: PathBuf,
    temp_dir: TempDir,
    dest: PathBuf,
    nextflow_stdout: &String,
    timestamp: &String,
    force: &bool,
) {
    let epi2db = epi2me_db::find_db();
    let mut local_prefix = PathBuf::from("/");
    if epi2db.is_some() {
        local_prefix = epi2db.unwrap().epi2path;
    }

    let mut manifest = Epi2MeManifest::new(temp_dir.path.clone());
    let mut all_files: Vec<FileManifest> = Vec::new();

    log::info!("packing [{:?}] into .2me format archive", &source.clone());

    let mut vehicle = Epi2meDesktopAnalysis::init(ulidstr, &source, nextflow_stdout, timestamp);

    /* we need to parse some information here - at least the tuple of user//repo */

    manifest.note_packaged_analysis(
        &vec![
            String::from(&vehicle.workflowUser),
            String::from(&vehicle.workflowRepo),
            String::from(&vehicle.name),
        ]
        .join("/"),
    );

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
    manifest
        .payload
        .push(Epi2MeContent::Epi2mePayload(vehicle.clone()));

    println!("{:?}", &manifest);

    let mut manifest_pb = PathBuf::from(&temp_dir.path);
    manifest_pb.push(MANIFEST_JSON);
    manifest.write(&manifest_pb);

    // tar up the contents specified in the manifest
    if dest.exists() && !*force {
        eprintln!("destination archive already exists - cannot continue without `--force`")
    } else {
        // tar up the contents specified in the manifest
        epi2me_tar::tar(
            None,
            dest,
            &all_files,
            &get_relative_path(&manifest_pb, &local_prefix),
        );
    }
}
