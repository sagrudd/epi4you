use chrono::{Local, DateTime};
use fs_extra::{copy_items, dir};
use rusqlite::{Connection, Result};
use polars::prelude::*;
use polars::df;
use ulid::Ulid;
use url::Position;
use url::Url;
use std::{env, fs};
use std::path::PathBuf;
use crate::dataframe::analysis_vec_to_df;
use crate::dataframe::filter_df_by_value;
use crate::dataframe::get_zero_val;
use crate::manifest::Epi2meDesktopAnalysis;
use crate::workflow::Workflow;
use crate::{workflow, epi2me_db};


#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct Epi2MeAnalysis {
    pub id: String,
    pub path: String,
    pub name: String,
    pub status: String,
    pub workflowRepo: String,
    pub workflowUser: String,
    pub workflowCommit: String,
    pub workflowVersion: String,
    pub createdAt: String,
    pub updatedAt: String,
}

#[allow(non_snake_case)]
pub fn load_db(path: &PathBuf) -> Result<DataFrame, rusqlite::Error> {


    let lookup = String::from("SELECT id, path, name, status, workflowRepo, workflowUser, workflowCommit, workflowVersion, createdAt, updatedAt FROM bs");

    let connection = Connection::open(path)?;

    let mut stmt = connection.prepare(&lookup)?;
    let analysis_iter = stmt.query_map([], |row| {
        Ok(Epi2MeAnalysis {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            status: row.get(3)?,
            workflowRepo: row.get(4)?,
            workflowUser: row.get(5)?,
            workflowCommit: row.get(6)?,
            workflowVersion: row.get(7)?,
            createdAt: row.get(8)?,
            updatedAt: row.get(9)?,
        })
    })?;

    let mut nf_run_vec: Vec<Epi2MeAnalysis> = Vec::new();

    for nextflow_run in analysis_iter {
        let my_nextflow_run = nextflow_run.unwrap();
        nf_run_vec.push(my_nextflow_run);
    }

    // and wrangle observations into a dataframe
    let df = analysis_vec_to_df(nf_run_vec);
    Ok(df)
}



fn get_db_id_entry(runid: &String, polardb: &DataFrame) -> Result<DataFrame, PolarsError> {
    // is runid in name field and unique
    let df = filter_df_by_value(polardb, &String::from("name"), runid);
    let df2 = filter_df_by_value(polardb, &String::from("id"), runid);
    return df.unwrap().vstack(&df2.unwrap());
}

pub fn get_qualified_analysis_path(runid: &String, polardb: &DataFrame) -> PathBuf {
    let stacked = get_db_id_entry(runid, polardb).unwrap();
    return PathBuf::from(get_zero_val(&stacked, &String::from("path")));
}



pub fn validate_qualified_analysis_workflow(runid: &String, polardb: &DataFrame, epi2wf_dir: &PathBuf) -> Option<Workflow> {
    
    let stacked = get_db_id_entry(runid, polardb).unwrap();
    let wf_proj = get_zero_val(&stacked, &String::from("workflowUser"));
    let wf_repo = get_zero_val(&stacked, &String::from("workflowRepo"));
    let wf_vers = get_zero_val(&stacked, &String::from("workflowVersion"));

    let mut workflow: String = String::new();
    
    workflow.push_str(&wf_proj);
    workflow.push_str(&std::path::MAIN_SEPARATOR.to_string());
    workflow.push_str(&wf_repo);
    println!("repo {}", workflow);

    // let's check that the path exists ...
    let wfdir_exists = workflow::check_defined_wfdir_exists(epi2wf_dir, &wf_proj, &wf_repo);
    if wfdir_exists.is_some() {
        let wf = Workflow{project: wf_proj, name: wf_repo, version: wf_vers};
        return Some(wf);
    }

    return None;
}


pub fn validate_db_entry(runid: &String, polardb: &DataFrame) -> bool {

    let stacked = get_db_id_entry(runid, polardb);
    //println!("{:?}",stacked);

    let row_count = &stacked.as_ref().unwrap().height(); // &stacked.unwrap().height();
    if row_count == &(1 as usize) {
        return true;
    } else if row_count == &(0 as usize) {
        println!("unable to resolve the analysis name; please check available analyses");
        return false;
    } else if row_count > &(1 as usize) {
        println!("supplied id name is ambiguous - please try to refine the analysis identifier");
        print_appdb(&stacked.unwrap().clone());
    }

    return false;
}

fn get_instance_struct(runid: &String, polardb: &DataFrame) -> Option<Epi2MeAnalysis> {
    if validate_db_entry(runid, polardb) {
        let stacked = get_db_id_entry(runid, polardb).unwrap();
        let x = Epi2MeAnalysis { 
            id: get_zero_val(&stacked, &String::from("id")),
            path: get_zero_val(&stacked, &String::from("path")),
            name: get_zero_val(&stacked, &String::from("name")),
            status: get_zero_val(&stacked, &String::from("status")),
            workflowRepo: get_zero_val(&stacked, &String::from("workflowRepo")),
            workflowUser: get_zero_val(&stacked, &String::from("workflowUser")),
            workflowCommit: get_zero_val(&stacked, &String::from("workflowCommit")),
            workflowVersion: get_zero_val(&stacked, &String::from("workflowVersion")),
            createdAt: get_zero_val(&stacked, &String::from("createdAt")),
            updatedAt: get_zero_val(&stacked, &String::from("updatedAt")),
        };
        return Some(x);
    }
    return None;
}

fn get_instance_struct_from_desktop_analysis(danalysis: &Epi2meDesktopAnalysis) -> Option<Epi2MeAnalysis> {
    
    let x = Epi2MeAnalysis { 
        id: String::from(&danalysis.id),
        path: String::from(&danalysis.path),
        name: String::from(&danalysis.name),
        status: String::from(&danalysis.status),
        workflowRepo: String::from(&danalysis.workflowRepo),
        workflowUser: String::from(&danalysis.workflowUser),
        workflowCommit: String::from(&danalysis.workflowCommit),
        workflowVersion: String::from(&danalysis.workflowVersion),
        createdAt: String::from(&danalysis.createdAt),
        updatedAt: String::from(&danalysis.updatedAt),
    };
    return Some(x);
} 


pub fn get_analysis_struct_from_cli(ulid_str: &String, source: &PathBuf, nextflow_stdout: &String, timestamp: &String) -> Option<Epi2meDesktopAnalysis>  {

    let mut log = PathBuf::from(source);
    log.push("nextflow.stdout");

    // println!("{}", nextflow_stdout);

    let mut name = "";
    let mut revision = "";
    let revision_key = " - revision: ";
    let url_str_key = "Launching `";
    let mut project = String::from("");
    let mut pname = String::from("");
    let mut version = String::from("");
    let xxxkey = "||||||||||";

    let lines = nextflow_stdout.split("\n");
    for line in lines {
        if line.starts_with(url_str_key) {
            println!("{line}");

            name = &line[line.find("[").unwrap()+1..line.find("]").unwrap()];
            revision = &line[line.find(revision_key).unwrap()+revision_key.len()..];
            revision = &revision[..revision.find(" ").unwrap()];
            let mut url_str = &line[line.find(url_str_key).unwrap()+url_str_key.len()..];
            url_str = &url_str[..url_str.find("`").unwrap()];

            let url = Url::parse(url_str);
            if url.is_ok() {
                let data_url_payload = &url.unwrap()[Position::AfterHost..][1..];
                println!("{:?}", &data_url_payload);

                let x = &data_url_payload.split_once('/');
                if x.is_some() {
                    let (aproject, apname) = x.clone().unwrap();
                    project = String::from(aproject);
                    pname = String::from(apname);
                }
            }
        } else if line.contains(xxxkey) && line.contains(&pname) {
            println!("extracting vers from [{}]", line);
            let v = line[line.find(&pname).unwrap()+pname.len()..].trim();
            version = String::from(&v[.. v.find("-").unwrap()]);
            //println!("{v}");
        }
    }

    let x = Epi2meDesktopAnalysis { 
        id: String::from(ulid_str),
        path: String::from(source.to_str().unwrap()),
        name: String::from(name),
        status: String::from("COMPLETED"),
        workflowRepo: pname,
        workflowUser: project,
        workflowCommit: String::from(revision),
        workflowVersion: version,
        createdAt: String::from(timestamp),
        updatedAt: String::from(timestamp),
        ..Default::default()
    };

    println!("{:?}", x);
    return Some(x);
}



pub fn get_analysis_struct(runid: &String, polardb: &DataFrame) -> Option<Epi2meDesktopAnalysis> {
    if validate_db_entry(runid, polardb) {
        let stacked = get_db_id_entry(runid, polardb).unwrap();
        // this is obligate pass due to previous validation

        let x = Epi2meDesktopAnalysis { 
            id: get_zero_val(&stacked, &String::from("id")),
            path: get_zero_val(&stacked, &String::from("path")),
            name: get_zero_val(&stacked, &String::from("name")),
            status: get_zero_val(&stacked, &String::from("status")),
            workflowRepo: get_zero_val(&stacked, &String::from("workflowRepo")),
            workflowUser: get_zero_val(&stacked, &String::from("workflowUser")),
            workflowCommit: get_zero_val(&stacked, &String::from("workflowCommit")),
            workflowVersion: get_zero_val(&stacked, &String::from("workflowVersion")),
            createdAt: get_zero_val(&stacked, &String::from("createdAt")),
            updatedAt: get_zero_val(&stacked, &String::from("updatedAt")),
            ..Default::default() };
        return Some(x);
    }

    return None;
}


pub fn print_appdb(df: &DataFrame) {
    env::set_var("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", "1");
    env::set_var("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES","1");
    env::set_var("POLARS_FMT_MAX_ROWS", "20");
    let df2 = df!(
        "id" => df.column("id").unwrap(),
        "name" => df.column("name").unwrap(),
        "workflowRepo" => df.column("workflowRepo").unwrap(),
        "createdAt" => df.column("createdAt").unwrap(),
        "status" => df.column("status").unwrap(),
    );

    if df2.is_ok() {
        println!("{:?}", df2.unwrap());
    }
}


fn field_update(path: &PathBuf, epi2me_instances: &DataFrame, runid_str: &String, key: &str, val: &str) {
    let stacked = get_db_id_entry(runid_str, epi2me_instances).unwrap();
    let z = get_zero_val(&stacked, &String::from("id"));
    println!("using database entry id [{}]", z);

    let connection = Connection::open(&path);
    if connection.is_err() {
        println!("fubar creating db connection");
        return;
    }

    let conn = connection.unwrap();
    let sql = format!("UPDATE bs SET {} = ?1 WHERE id = ?2", key);
    let stmt = conn.prepare(sql.as_str());
    if stmt.is_err() {
        println!("fubar creating STMT");
        return;
    }
    
    let qq = stmt.unwrap().execute(&[val, &z.as_str()]);
    if qq.is_err() {
        println!("fubar with the qq");
        println!("{:?}", qq.err());
    }
}


fn drop_epi2me_instance(path: &PathBuf, epi2me_instances: &DataFrame, runid_str: &String) {
    let stacked = get_db_id_entry(runid_str, epi2me_instances).unwrap();
    let z = get_zero_val(&stacked, &String::from("id"));
    let y = get_zero_val(&stacked, &String::from("path"));
    println!("using database entry id [{}] - files at [{}]", z, y);
    let connection = Connection::open(&path);
    if connection.is_err() {
        println!("fubar creating db connection");
        return;
    }

    let conn = connection.unwrap();
    let qq = conn.execute("DELETE from bs where ID = ?1", &[&z.as_str()]);
    if qq.is_err() {
        println!("Error with delete from table - {:?}", qq.err());
        return;
    }

    let dd = fs::remove_dir_all(y.as_str());
    if dd.is_err() {
        println!("issue with deleting files at [{}]", y.as_str());
    }
    
}

fn insert_into_db(path: &PathBuf, epi2meitem: &Epi2MeAnalysis) {
    let connection = Connection::open(&path);
    if connection.is_err() {
        println!("fubar creating db connection");
        return;
    }

    let conn = connection.unwrap();

    let insert = String::from("INSERT into bs (id, path, name, status, workflowRepo, workflowUser, workflowCommit, workflowVersion, createdAt, updatedAt) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)");
    let result = conn.execute(insert.as_str(), &[&epi2meitem.id,
                                                                                &epi2meitem.path,
                                                                                &epi2meitem.name,
                                                                                &epi2meitem.status,
                                                                                &epi2meitem.workflowRepo,
                                                                                &epi2meitem.workflowUser,
                                                                                &epi2meitem.workflowCommit,
                                                                                &epi2meitem.workflowVersion,
                                                                                &epi2meitem.createdAt,
                                                                                &epi2meitem.updatedAt
    ]);

    if result.is_err() {
        println!("failure --- \n{:?}", result.err());
    }

}


fn get_run_status(id: &String, epi2me_instances: &DataFrame) -> Option<String> {
    let x = validate_db_entry(id, epi2me_instances );
    if !x {
        return None;
    }
    let df2 = filter_df_by_value(epi2me_instances, &String::from("id"), id).unwrap();
    let val = get_zero_val(&df2, &String::from("status"));
    return Some(val);
}


fn housekeeper(epi2me_instances: &DataFrame) {
    // extract all unique workflow ids

    let s = epi2me_instances.column("id").unwrap().clone();
    let chunked_array: Vec<Option<&str>> = s.utf8().unwrap().into_iter().collect();
    for id in chunked_array.iter() {
        let id2 = String::from(id.unwrap());
        
        let runstatus = get_run_status(&id2, epi2me_instances);

        if runstatus.is_some() {
            let runstatus_str = runstatus.as_ref().unwrap();

            let status_terms = vec!["UNKNOWN", "COMPLETED", "STOPPED_BY_USER"];
            // check that the status fits within a sensible predefined vocabulary
            if status_terms.contains(&runstatus.as_ref().unwrap().as_str()) {
                let instancepath = get_qualified_analysis_path(&id2, epi2me_instances);
                println!("id [{}] has status [{:?}] --> {:?}", id2, runstatus_str, instancepath);

                let paths = fs::read_dir(instancepath).unwrap();
                for path in paths {
                    let xpath = path.unwrap().path();
                    if xpath.ends_with("work") && xpath.is_dir() {
                        println!("Name: {}", xpath.display());

                        let dd = fs::remove_dir_all(&xpath);
                        if dd.is_err() {
                            println!("issue with deleting files at [{:?}]", &xpath);
                        }
                    }
                }
            }
        }
    }
}


fn resync_progress_json(source: &String, ulid: &String, newlid: &String) {

    let file2mod = vec!["progress.json", "params.json", "launch.json"];
    let paths = fs::read_dir(source).unwrap();
    for path in paths {
        let xpath = path.unwrap().path().clone();
        let fname = xpath.file_name().unwrap().to_string_lossy().to_string();

        if file2mod.contains(&fname.as_str()) {
            println!("matching {:?}", xpath);

            let contents = fs::read_to_string(&xpath).unwrap();
            let updated = contents.as_str().replace(ulid, newlid);

            let status = fs::write(&xpath, updated);
            if status.is_err() {
                println!("error with writing file - {:?}", status.err());
            }
        }

    }
}


pub fn dbmanager(path: &PathBuf, epi2me_instances: &DataFrame, list: &bool, runid: &Option<String>, status: &Option<String>, delete: &bool, rename: &Option<String>, housekeeping: &bool, clone: &Option<String>) {
    println!("Database functionality called ...");

    if *list {
        println!("Listing databases");
        print_appdb(epi2me_instances);
        return;
    } else if *housekeeping {
        housekeeper(epi2me_instances);
    } else if *delete && runid.is_some() {
        println!("dropping instance from database ....");
        // validate the specified runid - return if nonsense
        let runid_str = &runid.as_ref().unwrap().to_string();
        if !validate_db_entry(&runid_str, epi2me_instances) {
            return;
        }
        drop_epi2me_instance(path, epi2me_instances, runid_str);
    } else if runid.is_some() && status.is_some() {
        println!("updating status ....");
        let runid_str = &runid.as_ref().unwrap().to_string();
        // validate the specified runid - return if nonsense
        if !validate_db_entry(&runid_str, epi2me_instances) {
            return;
        }
        // define collection of allowed terms
        let status_terms = vec!["UNKNOWN", "COMPLETED", "ERROR", "STOPPED_BY_USER", "RUNNING"];
        // check that the status fits within a sensible predefined vocabulary
        if !status_terms.contains(&status.as_ref().unwrap().as_str()) {
            println!("status [{}] is not an allowed term - {:?}", &status.as_ref().unwrap().as_str(), status_terms);
            return;
        }
        field_update(path, epi2me_instances, runid_str, "status", &status.as_ref().unwrap().as_str());

        
    } else if runid.is_some() && rename.is_some() {
        println!("renaming instance ....");
        let runid_str = &runid.as_ref().unwrap().to_string();
        // validate the specified runid - return if nonsense
        if !validate_db_entry(&runid_str, epi2me_instances) {
            return;
        }
        field_update(path, epi2me_instances, runid_str, "name", &rename.as_ref().unwrap().as_str());
    } else if clone.is_some() && runid.is_some() {
        println!("cloning instance ....");
        let runid_str = &runid.as_ref().unwrap().to_string();
        // validate the specified runid - return if nonsense
        if !validate_db_entry(&runid_str, epi2me_instances) {
            return;
        }
        clone_extant_database_entry(runid_str, epi2me_instances, clone, path);
    }

}



fn epi2me_item_rebrand(epi2meitem: &Epi2MeAnalysis, clone: &Option<String>) -> Epi2MeAnalysis {
    let mut epi2meitem_x: Epi2MeAnalysis = epi2meitem.clone();
    epi2meitem_x.id = Ulid::new().to_string();
    if clone.is_some() {
        epi2meitem_x.name = clone.as_ref().unwrap().to_string();
    }
    // create a new path for the analysis
    let mut dst_dir = epi2me_db::find_db().unwrap().instances_path;
    dst_dir.push(vec![epi2meitem_x.workflowRepo.clone(), epi2meitem_x.id.clone()].join("_"));
    let dest_str = dst_dir.into_os_string().into_string().unwrap();
    epi2meitem_x.path = dest_str.clone();

    // we can (or should) also update the timestamps since this is a retouch ...
    // e.g. 2023-11-09 07:20:43.492 +00:00
    let local: DateTime<Local> = Local::now();
    println!("NOW == {:?}", local);
    // epi2meitem_x.createdAt = local.to_string();
    epi2meitem_x.updatedAt = local.to_string();

    return epi2meitem_x;
}

fn clone_extant_database_entry(runid_str: &String, epi2me_instances: &DataFrame, clone: &Option<String>, path: &PathBuf) {
    let epi2meitem = get_instance_struct(runid_str, epi2me_instances);
    if epi2meitem.is_some() {
        let e2eitem = epi2meitem.unwrap();
        let mut src_dir = epi2me_db::find_db().unwrap().instances_path;
        src_dir.push(vec![String::from(&e2eitem.workflowRepo), String::from(&e2eitem.id)].join("_"));

        let epi2meitem_x = epi2me_item_rebrand(&e2eitem, clone);
        println!("new epi2meobj = {:?}", &epi2meitem_x);

        insert_into_db(&path, &epi2meitem_x);
        
        // and copy across the accompanying files ...
        println!("copying files from source path [{:?}]", src_dir);
        let mut from_paths: Vec<String> = Vec::new();
        let paths = fs::read_dir(src_dir).unwrap();
        for path in paths {
            let xpath = path.unwrap().path().clone();
            let zz = xpath.into_os_string().into_string().unwrap();
            println!("munging file [{:?}]", zz);
            from_paths.push(zz);
        }
        let dest = epi2meitem_x.path.clone();
        let mkdir = fs::create_dir(&dest);
        println!("dest = {:?} = {:?}", dest, mkdir);
        let opts = dir::CopyOptions::new();
        let cp = copy_items(&from_paths, &dest, &opts);
        println!("state = {:?}", cp);

        resync_progress_json(&dest, runid_str, &epi2meitem_x.id.clone());
        
    }
}


pub fn insert_untarred_desktop_analysis(desktop_analysis: &Epi2meDesktopAnalysis) {

    let epi2meitem = get_instance_struct_from_desktop_analysis(desktop_analysis);
    if epi2meitem.is_some() {
        let e2eitem = epi2meitem.unwrap();
        let src_dir = epi2me_db::find_db().unwrap().epi4you_path;
        // src_dir.push("instances");
        // src_dir.push(vec![String::from(&e2eitem.workflowRepo) ,String::from(&e2eitem.id)].join("_"));

        let clone: Option<String> = None; // keep the name already used
        let epi2meitem_x = epi2me_item_rebrand(&e2eitem, &clone);
        println!("new epi2meobj = {:?}", &epi2meitem_x);
        
        insert_into_db(&epi2me_db::find_db().unwrap().epi2db_path, &epi2meitem_x);

        // and copy across the accompanying files ...
        for file in &desktop_analysis.files {
            let file_to_check = PathBuf::from(&src_dir).join(&file.relative_path).join(PathBuf::from(&file.filename));
            
            let mut rp = PathBuf::from(&file.relative_path);
            if rp.starts_with("instances") || rp.starts_with("import_export_4you") {
                if rp.starts_with("instances") {
                    rp = PathBuf::from(rp.strip_prefix("instances").unwrap());
                    let exp_dir = vec![String::from(&e2eitem.workflowRepo), String::from(&e2eitem.id)].join("_");
                    if rp.starts_with(&exp_dir) {
                        rp = PathBuf::from(rp.strip_prefix(exp_dir).unwrap());
                    }
                } else if rp.starts_with("import_export_4you") {
                    rp = PathBuf::from(rp.strip_prefix("import_export_4you").unwrap());
                    // there is a presumption here that the first path element is just a ulid string from packaging cli nextflow run
                    // -- just clip it  
                    let mut components = rp.components();
                    let c = components.next().unwrap().as_os_str().to_str().unwrap();
                    rp = PathBuf::from(rp.strip_prefix(c).unwrap());
                }
            } 

            let dest_file = PathBuf::from(&epi2meitem_x.path).join(&rp).join(PathBuf::from(&file.filename));

            // ensure that directories have been created 
            if dest_file.parent().is_some() && !dest_file.parent().unwrap().exists() {
                let _ = fs::create_dir_all(dest_file.parent().unwrap());
            }
            println!("copying file [{:?}]", file_to_check);
            let _ = fs::copy(file_to_check, dest_file);
        }

        // and manually add the manifest file ....



        resync_progress_json(&epi2meitem_x.path, &e2eitem.id, &epi2meitem_x.id.clone());
    }
}

