use crate::{epi2me_db, epi2me_desktop_analysis::Epi2meDesktopAnalysis};
use chrono::{DateTime, Local};
use rusqlite::Connection;
use std::{fs, path::PathBuf};
use ulid::Ulid;

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

fn insert_into_db(path: &PathBuf, epi2meitem: &Epi2MeAnalysis) {
    let conn = match Connection::open(path) {
        Ok(conn) => conn,
        Err(_) => {
            println!("fubar creating db connection");
            return;
        }
    };

    let insert = "INSERT into bs (id, path, name, status, workflowRepo, workflowUser, workflowCommit, workflowVersion, createdAt, updatedAt) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";
    let result = conn.execute(
        insert,
        &[
            &epi2meitem.id,
            &epi2meitem.path,
            &epi2meitem.name,
            &epi2meitem.status,
            &epi2meitem.workflowRepo,
            &epi2meitem.workflowUser,
            &epi2meitem.workflowCommit,
            &epi2meitem.workflowVersion,
            &epi2meitem.createdAt,
            &epi2meitem.updatedAt,
        ],
    );

    if result.is_err() {
        println!("failure --- \n{:?}", result.err());
    }
}

fn resync_progress_json(source: &str, ulid: &str, newlid: &str) {
    let file2mod = ["progress.json", "params.json", "launch.json"];
    let paths = fs::read_dir(source).unwrap();
    for path in paths {
        let xpath = path.unwrap().path().clone();
        let fname = xpath.file_name().unwrap().to_string_lossy().to_string();

        if file2mod.contains(&fname.as_str()) {
            let contents = fs::read_to_string(&xpath).unwrap();
            let updated = contents.replace(ulid, newlid);

            let status = fs::write(&xpath, updated);
            if status.is_err() {
                println!("error with writing file - {:?}", status.err());
            }
        }
    }
}

fn epi2me_item_rebrand(epi2meitem: &Epi2MeAnalysis) -> Epi2MeAnalysis {
    let mut epi2meitem_x = epi2meitem.clone();
    epi2meitem_x.id = Ulid::new().to_string();

    let mut dst_dir = epi2me_db::find_db().unwrap().instances_path;
    dst_dir.push(vec![epi2meitem_x.workflowRepo.clone(), epi2meitem_x.id.clone()].join("_"));
    epi2meitem_x.path = dst_dir.into_os_string().into_string().unwrap();

    let local: DateTime<Local> = Local::now();
    epi2meitem_x.updatedAt = local.to_string();

    epi2meitem_x
}

pub fn insert_untarred_desktop_analysis(
    desktop_analysis: &Epi2meDesktopAnalysis,
    temp_dir: &PathBuf,
) {
    log::warn!("insert_untarred_desktop_analysis");

    let e2eitem = desktop_analysis.as_epi2me_analysis();
    let epi2meitem_x = epi2me_item_rebrand(&e2eitem);
    log::info!("new epi2meobj = {:?}", &epi2meitem_x);

    insert_into_db(&epi2me_db::find_db().unwrap().epi2db_path, &epi2meitem_x);

    for file in &desktop_analysis.files {
        let file_to_check = PathBuf::from(temp_dir)
            .join(&file.relative_path)
            .join(PathBuf::from(&file.filename));

        let mut rp = PathBuf::from(&file.relative_path);
        if rp.starts_with("instances")
            || rp.starts_with("import_export_4you")
            || rp.starts_with("tmp")
        {
            if rp.starts_with("instances") {
                rp = PathBuf::from(rp.strip_prefix("instances").unwrap());
                let exp_dir = vec![
                    String::from(&e2eitem.workflowRepo),
                    String::from(&e2eitem.id),
                ]
                .join("_");
                if rp.starts_with(&exp_dir) {
                    rp = PathBuf::from(rp.strip_prefix(exp_dir).unwrap());
                }
            } else {
                let prefix = if rp.starts_with("import_export_4you") {
                    "import_export_4you"
                } else {
                    "tmp"
                };
                rp = PathBuf::from(rp.strip_prefix(prefix).unwrap());
                let mut components = rp.components();
                if let Some(component) = components.next() {
                    let c = component.as_os_str().to_str().unwrap();
                    rp = PathBuf::from(rp.strip_prefix(c).unwrap());
                }
            }
        }

        let dest_file = PathBuf::from(&epi2meitem_x.path)
            .join(&rp)
            .join(PathBuf::from(&file.filename));

        if let Some(parent) = dest_file.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }

        log::debug!("copying file [{:?}]", file_to_check);
        let _ = fs::copy(file_to_check, dest_file);
    }

    resync_progress_json(&epi2meitem_x.path, &e2eitem.id, &epi2meitem_x.id);
}
