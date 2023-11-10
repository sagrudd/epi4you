use std::{path::PathBuf, fs, collections::HashMap};
use crate::{epi2me_db::Epi2meSetup, workflow::glob_path_by_wfname};
use regex::Regex;

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



fn nextflow_parser(contents: &String) -> HashMap<String, String> {

    let mut key: Vec<String> = Vec::new();
    let mut cache: Vec<String> = Vec::new();
    let mut cache_key: String = String::from("");

    let mut nextflow_config: HashMap<String, String> = HashMap::new();

    let lines = contents.lines();
    for line in lines {
        let l2 = line.trim();
        let s = String::from(l2);

        println!("{}",s);

        /*let idx = s.rfind("//");
        if idx.is_some() {
            l2 = &s[..idx.unwrap()].trim();
        }*/

        if String::from(l2).starts_with("//") {
            // skip it ...
        } else if String::from(l2).len() == 0 {
            // skip it ...
        } else if String::from(l2).ends_with("{") {
            let open_key = l2.replace(" {", "");
            // println!("-> handling a chunk start -- [{}]", open_key);
            key.push(open_key);
        } else if String::from(l2).starts_with("}") {
            // let close_key = &key[key.len()-1];
            // println!("!! closing chunk -- [{}]", close_key);
            key.pop();
        } else if String::from(l2).ends_with("[") {
            let (field, _value) = s.split_at(s.find(" = ").unwrap());
            cache_key = String::from(field.trim());
            println!("setting cache_key = [{}]", &cache_key);
        } else if String::from(l2).starts_with("]") {
            println!("closing cache_key = [{}]", &cache_key);
            let merged = cache.join("-");
            let merged_key = vec![key.clone(), vec![cache_key]].concat().join(".");
            nextflow_config.insert(merged_key, merged);
            cache_key = String::from("");
            cache = Vec::new();
        } else if cache_key.len() > 0 {
            println!("appending cache");
            cache.push(String::from(l2));
        } else if String::from(l2).contains(" = ") {
            println!("keypair to extract");
            let (field, value) = s.split_at(s.find(" = ").unwrap());
            let val = String::from(&value[2..]);
            let val2 = String::from(val.trim());
            let merged_key = vec![key.clone(), vec![String::from(field.trim())]].concat().join(".");
            nextflow_config.insert(merged_key, String::from(val2));
        } else {
            // println!("{}", l2);
        }
        
    }

    /*
    for key in nextflow_config.keys() {
        let val = nextflow_config.get(key);
        println!("config k={} v={}", key, val.unwrap());
    }
    */

    return nextflow_config;

}


fn extract_containers(config: &HashMap<String, String>) {
    let prefix = String::from("process");
    let suffix = String::from("container");
    for key in config.keys() {
        if key.starts_with(&prefix) && key.ends_with(&suffix) { 
            let container_str = config.get(key).unwrap();

            let re = Regex::new(r"\$\{[^\}]+\}").unwrap();
            let bb = re.captures(container_str);
            if bb.is_some() {
                let bb2 = bb.unwrap();
                let found = bb2.get(0).unwrap().as_str();
                let found2 = &found[2..found.len()-1];
                println!("something ... {:?}", found2);


            }

            println!("container [{:?}]", container_str.as_str());
        }
    }
}



fn identify_containers(pb: &PathBuf) {
    let contents = fs::read_to_string(&pb).unwrap();
    // println!("{}", &contents);
    let config = nextflow_parser(&contents);
    let containers = extract_containers(&config);
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
    println!("home = {:?}", epi2me.epi2wf_dir);

    let xx = glob_path_by_wfname(epi2me, &project);
    if xx.is_some() {
        println!("exploring folder for config specified containers ...");
        let mut pb = xx.unwrap().clone();
        pb.push("nextflow.config");

        identify_containers(&pb);
    }

}