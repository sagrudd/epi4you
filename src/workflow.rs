use std::{path::PathBuf, fs};
use glob::glob;
use polars_core::prelude::DataFrame;
use serde::{Serialize, Deserialize};
use crate::{epi2me_db::{Epi2meSetup, self}, docker::nextflow_parser, dataframe::{workflow_vec_to_df, print_polars_df, two_field_filter}, bundle::export_nf_workflow, manifest::Epi2meWorkflow};


#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct Workflow {
    pub project: String,
    pub name: String,
    pub version: String,
}

pub fn get_workflow_struct(p: &String, n: &String, v: &String) -> Epi2meWorkflow {
    return Epi2meWorkflow {
        project: String::from(p),
        name: String::from(n),
        version: String::from(v),
        ..Default::default()
    };
}

pub fn get_epi2me_wfdir_path(app_db_path: &PathBuf) -> Option<PathBuf> {
    let mut x = app_db_path.clone();

    x.push("workflows");
    if x.exists() {
        println!("\tworkflows folder exists at [{}]", x.display());
        return Some(x.clone());
    }
    return None;
}

pub fn check_defined_wfdir_exists(wfdir: &PathBuf, user: &str, repo: &str) -> Option<PathBuf> {
    let mut x = wfdir.clone();
    x.push(String::from("workflows"));
    x.push(user);
    x.push(repo);
    if x.exists() && x.is_dir() {
        println!("\tdefined workflow folder exists at [{}]", x.display());
        return Some(x.clone());
    }
    eprintln!("\tworkflow folder does not exist at [{}]", x.display());
    return None;
}


fn is_folder_wf_compliant(wffolder: &PathBuf) -> bool {
    let required_files = vec!["main.nf", "nextflow.config"];
    let mut counter = 0;
    let paths = fs::read_dir(wffolder).unwrap();
    for path in paths {
        let fname = &path.unwrap().file_name().to_string_lossy().to_string();
        if required_files.contains(&fname.as_str()) {
            // println!("found {:?}", fname);
            counter += 1;
        }
    }
    if required_files.len() == counter {
        return true;
    }

    return false;
}


pub fn glob_path_by_wfname(epi2me: &Epi2meSetup, project: &String) -> Option<PathBuf> {

    let globpat = epi2me.epi2wf_dir.clone().into_os_string().into_string().unwrap();
    let result = [&globpat, "/*/", &project].join("");
    
    let gdata =  glob(&result).expect("Failed to read glob pattern");
    for entry in gdata {
        if entry.is_ok() {
            let entry_item = entry.unwrap();
            // ensure that the folder found is actually a nextflow folder (nanopore flavoured)
            if is_folder_wf_compliant(&entry_item) {
                // println!("folder picked == {:?}", entry_item);
                return Some(entry_item)
            }
        }
    }
    // we can also assess whether project is a link to e.g. a nextflow based folder elsewhere on CLI
    return None;
}


fn workflows_to_polars(path: &PathBuf) -> Option<DataFrame> {
    println!("\tparsing workflows from path [{:?}]", path);

    let globpat = &path.clone().into_os_string().into_string().unwrap();
    let path_pattern = [&globpat, "/*/*"].join("");
    let mut wfs: Vec<Workflow> = Vec::new();

    for entry in glob(&path_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(mut globpath) => {
                
                let mut config_path = globpath.clone();
                config_path.push("nextflow.config");
                let globpathstr = globpath.as_os_str().to_str().unwrap();
                if globpath.is_dir() && !globpathstr.contains(".nextflow") {
                    let workflow = globpath.file_name().unwrap().to_str().unwrap().to_string();
                    globpath.pop();
                    let project = globpath.file_name().unwrap().to_str().unwrap().to_string();

                    // extract workflow revision for the linked artifact
                    // this is probably best prepared by parsing the information from the config file?
                    if config_path.exists() {
                        let contents = fs::read_to_string(&config_path).unwrap();
                        let config = nextflow_parser(&contents);
                        let mut version = String::from("?");
                        let man_version = config.get("manifest.version");
                        if man_version.is_some() {
                            version = String::from(man_version.unwrap());
                        }

                        let w = Workflow{
                            project: project,
                            name: workflow,
                            version: String::from(version),
                        };
                        wfs.push(w);

                    }
                }
            },
            Err(e) => println!("{:?}", e),
        }
    }

    let df = workflow_vec_to_df(wfs);
    return Some(df);

    /* 
    let myd: DataFrame = struct_to_dataframe!(wfs, [project,
        name,
        version]).unwrap();
    */

}


pub fn insert_untarred_workflow(epi2me_workflow: &Epi2meWorkflow, force: &bool) {

    let e4u_path = epi2me_db::find_db().unwrap().epi4you_path;

    let mut dest_dir = epi2me_db::find_db().unwrap().epi2wf_dir;
    dest_dir.push(&epi2me_workflow.project);
    dest_dir.push(&epi2me_workflow.name);
    println!("workflow_path resolved as {:?}", dest_dir);

    if dest_dir.exists() {
        if dest_dir.is_dir() {
            if *force {
                // nuke the already existing directory - this is with vengeance
            } else {
                eprintln!("The workflow directory [{:?}] already exists - consider `--force`", dest_dir);
                return;
            }
        } else if dest_dir.is_file() {
            eprintln!("The workflow directory [{:?}] already exists as a file - nonsense", dest_dir);
            return;
        }
    }

    for file in &epi2me_workflow.files {
        let file_to_check = PathBuf::from(&e4u_path).join(&file.relative_path).join(PathBuf::from(&file.filename));

        let target = PathBuf::from(&file.relative_path);
        let mut t2: PathBuf = target.iter().skip(3).collect();
        t2.push(&file.filename);

        let mut t3 = dest_dir.clone();
        t3.push(t2);

        if t3.parent().is_some() && !t3.parent().unwrap().exists() {
            let _ = fs::create_dir_all(t3.parent().unwrap());
        }
        println!("copying file [{:?}]", file_to_check);
        let _ = fs::copy(file_to_check, t3);
    }

}


pub fn workflow_manager(list: &bool, workflow: &Vec<String>, twome: &Option<String>, force: &bool) {
    
    let src_dir = epi2me_db::find_db().unwrap().epi2wf_dir;
    let df = workflows_to_polars(&src_dir);
    let df2 = df.as_ref().unwrap();
    let mut picked = DataFrame::default();

    if *list {
        println!("Listing installed bioinformatics workflows from [{:?}]", &src_dir);
        if df.as_ref().is_some() {
            print_polars_df(&df.unwrap());
        }
        return;
    }
    if workflow.len() == 0 {
        eprintln!("The workflow option requires a `--workflow` parameter to specify workflow of interest");
        return;
    }

    if twome.is_none() {
        eprintln!("A `--twome` parameter is required to define output tar archive");
        return;
    }

    for workflow_id in workflow {
        println!("processing workflow [{}]", &workflow_id);
        
        // checking if project / name exist in the df
        let split = &workflow_id.split_once("/");
        if split.is_none() {
            eprintln!("workflow [{:?}] could not be split - requires a '/' delimiter", &workflow_id);
            return;
        }
        let (project, name) = split.unwrap();
        // filter on project and name
        let filtered_df = two_field_filter(&df2, &String::from("project"), &String::from(project), &String::from("name"), &String::from(name)); 
        if filtered_df.is_none() {
            eprintln!("unexpected failure - failed to find appropriate workflow installation");
            return;
        }
        let filtered = filtered_df.unwrap();
        let height = filtered.height();
        if height == 0 {
            eprintln!("failed to resolve specified workflow installation [{}]", &workflow_id);
            return;
        } else if height > 1 { // can this even happen?
            eprintln!("specified workflow installation is ambiguous [{}]", &workflow_id);
            return;
        }
        // print_polars_df(&filtered);
        if picked.is_empty() {
            picked = filtered;
        } else {
            let repicked = picked.vstack(&filtered);
            if repicked.is_ok() {
                picked = repicked.unwrap();
            }
        }
    }
    print_polars_df(&picked);

    // and now export into an archive ...
    export_nf_workflow(None, &picked, twome, force);

}