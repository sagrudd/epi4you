use std::{path::PathBuf, fs, collections::HashMap};
use crate::{epi2me_db::Epi2meSetup, workflow::glob_path_by_wfname};
use regex::Regex;
use docker_api::{Docker, Result};
use docker_api::opts::PullOpts;
use futures::StreamExt;



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
        } else if String::from(l2).ends_with("[") && cache_key == String::from("") {  // collapse nested
            let (field, _value) = s.split_at(s.find(" = ").unwrap());
            cache_key = String::from(field.trim());
            // println!("setting cache_key = [{}]", &cache_key);
        } else if String::from(l2).starts_with("]") && String::from(l2).ends_with("]") && cache_key != String::from("") { // collapse nexted // TODO: this should be rethought
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
    let mut contents = fs::read_to_string(&pb).unwrap();
    contents = contents.replace(" { ", " {\n");
    contents = contents.replace("}\n", " \n}\n");
    // println!("{}", &contents);
    let config = nextflow_parser(&contents);
    return extract_containers(&config);
}

pub fn new_docker() -> Result<Docker> {
    Ok(Docker::unix("/var/run/docker.sock"))
}


async fn pull_container(container: &String) {
    let docker = new_docker();
        
    if docker.is_ok() {
        let opts = PullOpts::builder().image(container).build();

        let images = docker.unwrap().images();
        let mut stream = images.pull(&opts);

        while let Some(pull_result) = stream.next().await {
            match pull_result {
                Ok(output) => {
                    let x = format!("{output:?}");

                    let ostatus = omatch(r#"status: "[^"]+""#, &x);
                    if ostatus.is_some() {
                        let status = ostatus.unwrap();
                        if status == "Extracting" {
                            let oprogress = omatch(r#"progress: [^\)]+\)"#, &x);
                            if oprogress.is_some() {
                                println!("{}", oprogress.unwrap());
                            }
                        } else if status == "Downloading" {
                        } else {
                            println!("{status}")
                        }
                    }
                        


                },
                Err(e) => eprintln!("{e}"),
            }
        }
    } else {

        println!("docker failure?");
    }
}


fn unsome(s: &str) -> String {
    if s.starts_with(r#"Some(""#) {
        return String::from(&s[6..s.len()-2]);
    }
    return String::from(s);
}


fn unlabel(s: &str, label: &str) -> String {
    let mut unlabelled = String::from(s.replace(label, "").trim());
    while unlabelled.starts_with(r#"""#) {
        unlabelled = String::from(&unlabelled[1..unlabelled.len()-1]);
    }
    if unlabelled.starts_with("Some(") {
        return unsome(&unlabelled);
    }
    return String::from(unlabelled);
}


fn omatch(key: &str, txt: &str) -> Option<String> {
    //println!("looking for || {}", key);
    let re_status = Regex::new(key).unwrap();
    let lmatch = re_status.find(&txt);
    if lmatch.is_some() {
        let matched = lmatch.unwrap();
        let found = matched.as_str();
        //println!("omatch [{}] --> [{}]", key, found);
        if key.ends_with(r#"""#) || key.ends_with(r#")"#) {
            let olabel = omatch("^[^:]+:", &found);
            if olabel.is_some() {
                return Some(unlabel(found, &olabel.unwrap()));
            }
        }
        return Some(String::from(found));
    }
    println!("returning None!");
    return None;
}

pub async fn docker_agent(epi2me: &Epi2meSetup, workflow_opt: &Option<String>, list: &bool, pull: &bool) {

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
        for container in &containers {
            println!("{}", container);
        }
    }

    if *pull {
            for container in &containers {
                println!("pulling [{}]", container);
                pull_container(container).await;
            }
    }


}