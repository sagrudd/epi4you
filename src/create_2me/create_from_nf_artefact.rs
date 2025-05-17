use clap::ArgMatches;

use crate::{epi4you_errors::Epi4youError, tempdir::TempDir};



pub fn process_nf_artefact_command(args: &ArgMatches, tempdir: &TempDir) -> Result<(), Epi4youError> {

/* 

    let nextflow_bin = get_nextflow_path(nxf_bin.clone());
    if nextflow_bin.is_some() {
        if *list {
            let extant_artifacts = list_installed_nextflow_artifacts(nextflow_bin.as_ref().unwrap());
            if extant_artifacts.as_ref().is_some() {
                print_polars_df(&extant_artifacts.unwrap());
            }
            return;
        } else {

            if workflow.len() == 0 {
                eprintln!("\trequires a `--workflow` parameter to specify workflow of interest");
                return;
            }

            if twome.is_none() {
                eprintln!("\trequires a `--twome` parameter to specify target archive");
                return;
            }

            let tempdir = tempdir::get_tempdir();
            if tempdir.is_none() {
                eprintln!("error creating tempdir - aborting!");
                return;
            }

            let temp_dir = tempdir.unwrap();
            let mut wfs: Vec<Workflow> = Vec::new();

            let artifacts = get_local_artifacts(&nextflow_bin.as_ref().unwrap());

            let workflows: Vec<String>;
            if workflow.into_iter().nth(0).unwrap().to_owned() == String::from("all") {
                workflows = list_available_workflows();
            } else {
                workflows = workflow.into_iter().map(|v|v.to_owned()).collect();
            }
            
            for workflow_candidate in workflows {
                println!("checking [{}]", &workflow_candidate);

                let asset_opt = get_workflow_entity(&workflow_candidate, &artifacts);
                let asset: NextflowAssetWorkflow;

                if asset_opt.is_some() {
                    asset = asset_opt.unwrap();
                } else {
                    // None - likely due to not existing ...
                    if *pull {
                        let asset_o = nextflow_workflow_pull(nextflow_bin.as_ref().unwrap(), &workflow_candidate);
                        if asset_o.is_some() {
                            asset = asset_o.unwrap();
                        } else {
                            eprintln!("issue with workflow pull - aborting");
                            return;
                        }
                    } else {
                        eprintln!("workflow [{}] has not been installed - consider `--pull` - aborting", &workflow_candidate);
                        return;
                    }
                }
                println!("\tversion [{}] at [{}]", asset.version, asset.path);
                // clone files into a temporary directory

                let mut local_output = temp_dir.path.clone();
                local_output.push("workflows");
                local_output.push(&workflow_candidate);
                let _create_d = fs::create_dir(&local_output);
                let ap = &asset.path;
                for entry in WalkDir::new(ap) {
                    if entry.is_ok() {
                        let ent = entry.unwrap();
                        let core_p = ent.path().strip_prefix(ap);
                        if core_p.is_ok() {
                            let gg = core_p.unwrap();
                            let mut dest_f = local_output.clone();
                            dest_f.push(&gg);
                            // println!("src {:?} -> ", &dest_f);

                            if ent.path().is_dir() {
                                let _create_d = fs::create_dir_all(dest_f);
                            } else if ent.path().is_file() {
                                // println!("copying ...");
                                let _copy_f = fs::copy(ent.path(), dest_f);
                            }
                        }
                    }
                }

                let split = &workflow_candidate.split_once("/");
                let (project, name) = split.unwrap();
                let w = Workflow { project: String::from(project), name: String::from(name), version: asset.version};
                wfs.push(w);

                /*
                if *docker {
                    let _x = extract_containers(&asset.config);
                    for container in _x {
                        println!("we have a [{}] container .... ", container);
                    }
                }
                */
            }

            // we need a dataframe for the items that we'll inject ...
            let df = workflow_vec_to_df(wfs);
            print_polars_df(&df);
            export_nf_workflow(Some(&temp_dir.path), &df, twome, force);
            

        }
    }

*/

    return Ok(());
}