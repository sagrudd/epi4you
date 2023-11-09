use std::path::PathBuf;

use crate::epi2me_db::Epi2meSetup;


fn get_workflow_version() -> String {
    return "undefined".to_string();
}

fn get_filename(epi2me: &Epi2meSetup, workflow_path: PathBuf) -> String {
    let fname = format!("wf_workflow_{}_{}", epi2me.arch, get_workflow_version());
    return fname;
}


pub fn config2containers(path: PathBuf) {

}


pub fn pullcontainers() {

}


pub fn containers2tar() {

}


pub fn tar2containers() {

}



pub fn docker_agent(epi2me: &Epi2meSetup, projectopt: &Option<String>) {

    if !projectopt.is_some() {
        println!("docker methods require a --project pointer to a workflow");
        return;
    }
    let project = projectopt.as_ref().unwrap().to_string();

    println!("surveying workflow [{:?}]", project);
    println!("data = {:?}", epi2me.epi2path);
    println!("arch = {:?}", epi2me.arch);

}