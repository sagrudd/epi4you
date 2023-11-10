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


fn string_clip(src: String) -> String {
    let mut start = 0 as usize;
    let mut end = src.len();

    let first = src.chars().next().unwrap();
    let last = src.chars().nth(end-1).unwrap();

    match first {
        '\'' => start += 1,
        '\"' => start += 1,
        _ => start += 0,
    };

    match last {
        '\'' => end -= 1,
        '\"' => end -= 1,
        _ => end -= 0,
    };

    return String::from(&src[start..end]);
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

        // println!("{}",s);

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
            // println!("setting cache_key = [{}]", &cache_key);
        } else if String::from(l2).starts_with("]") {
            // println!("closing cache_key = [{}]", &cache_key);
            let merged = cache.join("-");
            let merged_key = vec![key.clone(), vec![cache_key]].concat().join(".");
            nextflow_config.insert(merged_key, merged);
            cache_key = String::from("");
            cache = Vec::new();
        } else if cache_key.len() > 0 {
            // println!("appending cache");
            cache.push(String::from(l2));
        } else if String::from(l2).contains(" = ") {
            // println!("keypair to extract");
            let (field, value) = s.split_at(s.find(" = ").unwrap());
            let val = String::from(&value[2..]);
            let val2 = string_clip(String::from(val.trim()));
            let merged_key = vec![key.clone(), vec![String::from(field.trim())]].concat().join(".");
            nextflow_config.insert(merged_key, String::from(val2));
        } else {
            // println!("{}", l2);
        }
        
    }
    return nextflow_config;

}


fn extract_containers(config: &HashMap<String, String>) -> Vec<String> {
    let mut container_vec: Vec<String> = Vec::new();
    let prefix = String::from("process.");
    let suffix = String::from("container");
    for key in config.keys() {
        if key.starts_with(&prefix) && key.ends_with(&suffix) { 
            let container_str = String::from(config.get(key).unwrap());
            let mut mod_container_str = container_str.clone();
            let re = Regex::new(r"\$\{[^\}]+\}").unwrap(); // 
            
            for matched in re.find_iter(&container_str) {
                let found = matched.as_str();
                let value = config.get(&found[2..found.len()-1]);
                if value.is_some() {
                    mod_container_str = mod_container_str.replace(found, value.unwrap());
                }
            }
            // println!("container == [{}]", mod_container_str);
            container_vec.push(mod_container_str);
        }
    }
    return container_vec;
}



fn identify_containers(pb: &PathBuf) -> Vec<String> {
    let contents = fs::read_to_string(&pb).unwrap();
    // println!("{}", &contents);
    let config = nextflow_parser(&contents);
    return extract_containers(&config);
}

pub fn docker_agent(epi2me: &Epi2meSetup, workflow_opt: &Option<String>, list: &bool) {

    if !workflow_opt.is_some() {
        println!("docker methods require a --workflow pointer to a workflow");
        return;
    }
    let workflow = workflow_opt.as_ref().unwrap().to_string();

    println!("surveying workflow [{:?}]", workflow);
    // println!("data = {:?}", epi2me.epi2path);
    // println!("arch = {:?}", epi2me.arch);
    // println!("home = {:?}", epi2me.epi2wf_dir);

    let mut containers: Vec<String> = Vec::new();
    let mut valid: bool = false;

    let epi2me_installed_wf = glob_path_by_wfname(epi2me, &workflow);
    if epi2me_installed_wf.is_some() {
        let mut pb = epi2me_installed_wf.unwrap().clone();
        pb.push("nextflow.config");
        containers = identify_containers(&pb);
        valid = true;
    }

    if !valid {
        println!("Cannot continue - the --workflow defined cannot be resolved");
    }

    if *list {
        for container in containers {
            println!("{}", container);
        }
    }


}