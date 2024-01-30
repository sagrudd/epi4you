use std::process::Command;
use std::{path::PathBuf, fs, collections::HashMap};
use crate::bundle::{clip_relative_path, sha256_digest};
use crate::dataframe::{docker_vec_to_df, print_polars_df, dockercontainer_vec_to_df};
use crate::epi2me_db;
use crate::manifest::{get_manifest, FileManifest, Epi2meContainer, file_manifest_size, Epi2MeContent};
use crate::tempdir::TempDir;
use crate::workflow::list_installed_workflows;
use crate::{epi2me_db::Epi2meSetup, workflow::glob_path_by_wfname};
use regex::Regex;
use docker_api::{Docker, Result};
use docker_api::opts::PullOpts;
use docker_api::api::Image;
use docker_api::opts::TagOpts;
use futures::{StreamExt, TryStreamExt};
use serde::{Serialize, Deserialize};
use std::io::Write;



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

pub fn nextflow_parser(xcontents: &String) -> HashMap<String, String> {
    let mut contents = String::from(xcontents);

    contents = contents.replace(" { ", " {\n");
    contents = contents.replace("}\n", " \n}\n");

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


pub fn extract_containers(config: &HashMap<String, String>) -> Vec<String> {
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



fn identify_containers(pb: &PathBuf) -> (HashMap<String, String>, Vec<String>) {
    let contents = fs::read_to_string(&pb).unwrap();

    let config = nextflow_parser(&contents);
    return (config.clone(), extract_containers(&config.clone()));
}


fn get_docker_context_socket() -> String {
    let output = Command::new("docker")
        .arg("context")
        .arg("ls")
        .output()
        .expect("failed to execute process");

    let s = String::from_utf8_lossy(&output.stdout).into_owned();
    let trimmed_s = s.trim();

    let lines = trimmed_s.lines();
    for line in lines {
        if !line.trim().starts_with("NAME") {
            // println!("{line}");
            let re = Regex::new(r"\s{2,50}").unwrap(); // 
            let mut mod_container_str = String::from(line.trim()).replace("\t", "");
            for matched in re.find_iter(&String::from(line)) {
                let found = matched.as_str();
                mod_container_str = mod_container_str.replacen(found, "|", 1);
            }
            // println!("{}", mod_container_str);
            let split_str: Vec<&str> = mod_container_str.split("|").collect();
            let socket_name = split_str.get(0).unwrap().to_owned();
            let socket = split_str.get(3).unwrap().to_owned();
            // println!("socket [{}] == <{}>", socket_name, socket);
            if socket_name.ends_with("*") {
                return String::from(socket).replace("unix://", "");
            }
        }
    }
    return String::from("/var/run/docker.sock");
}


pub fn new_docker() -> Result<Docker> {

    // we should parse connection strings via -  `docker context ls`
    let socket = get_docker_context_socket();

    //Ok(Docker::unix("/var/run/docker.sock"))
    Ok(Docker::unix(socket))
}


async fn retag_image(installed: &String, requested: &String) {

    println!("retagging image [{installed}] -> [{requested}]");

    let docker = new_docker();
    if docker.is_ok() {

        let x = requested.split_once(":");
        if x.is_some() {
            let (repo, tag) = x.unwrap();
            let tag_opts = TagOpts::builder().repo(repo).tag(tag).build();
            let image = Image::new(docker.unwrap(), installed);
            let status = image.tag(&tag_opts).await;
            if status.is_err() {
                eprintln!("Error: {:?}", status.err());
            } 
        }
    }
}


async fn pull_container(container: &String) {
    let docker = new_docker();

    println!("pulling container [{}]", container);

    if docker.is_ok() {
        let opts = PullOpts::builder().image(container).build();

        let images = docker.unwrap().images();
        let mut stream = images.pull(&opts);

        while let Some(pull_result) = stream.next().await {
            // println!("<?>");
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
                            println!("{status}");

                            // can we capture the name of the container that has been pulled?
                            // Status: Downloaded newer image for ontresearch/prokka:latest
                            // Status: Downloaded newer image for ontresearch/wf-bacterial-genomes:latest
                            // Status: Image is up to date for ontresearch/wf-common:latest

                            let newer_image = "Status: Downloaded newer image for ";
                            let up2da_image = "Status: Image is up to date for ";
                            if status.starts_with(newer_image) {
                                let installed_img = status.replace(newer_image, "");
                                retag_image(&installed_img, container).await;
                            } else if status.starts_with(up2da_image) {
                                let installed_img = status.replace(up2da_image, "");
                                retag_image(&installed_img, container).await;
                            }
                        }
                    }
                },
                Err(e) => eprintln!("oops {e}"),
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

async fn export_containers(containers: &Vec<String>, p: &PathBuf) -> Vec<FileManifest> {

    let docker = new_docker();
    let local_prefix = epi2me_db::find_db().unwrap().epi2path;
    let mut files: Vec<FileManifest> = Vec::new(); 
        
    if docker.is_ok() {
        println!("docker is OK");
        for container in containers {
            println!("exporting [{}]", &container);

            let mut write_path = p.clone();
            let mut tar_file = String::from(container);
            tar_file = tar_file.replace("/", "-");
            tar_file = tar_file.replace(":", "-");
            write_path.push(format!("{}.tar", &tar_file));

            println!("writing to file [{}]", write_path.display());

            let images = docker.clone().unwrap().images();
            let image = images.get(container);
            let export_stream = image.export();
            let export_data = export_stream.try_concat().await.expect("image archive");

            let file = fs::OpenOptions::new()
            .create(true) // To create a new file
            .write(true)
            .open(&write_path);
        
            if file.is_ok() {
                println!("file is OK!");
                let xxx = file.unwrap().write_all(&export_data);
                if xxx.is_err() {
                    eprintln!("{:?}", xxx.err());
                } else {

                    let relative_path = clip_relative_path(&write_path, &local_prefix);
                    let file_size = &write_path.metadata().unwrap().len();
                    let checksum = sha256_digest(&write_path.as_os_str().to_str().unwrap());

                    files.push( FileManifest{filename: tar_file,
                        relative_path: String::from(relative_path.clone().to_string_lossy().to_string()),
                        size: *file_size,
                        md5sum: checksum
                    } );
                }
            } else {
                eprintln!("file is fubar\n{:?}", file.err());
            }
        }


    }   
    return files;
}




#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct Container {
    pub workflow: String,
    pub version: String,
    pub dockcont: String,
    
}

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
pub struct DockerContainer {
    pub repository: String,                          
    pub tag: String,                           
    pub image: String,
    pub created: String,    
    pub size: String,
}

fn load_installed_docker_artifacts() {
    let mut containers: Vec<DockerContainer> = Vec::new();
    let output = Command::new("docker")
    .arg("images")
    .output()
    .expect("failed to execute process");

    let s = String::from_utf8_lossy(&output.stdout).into_owned();
    let trimmed_s = s.trim();

    let lines = trimmed_s.lines();
        for line in lines {
            if !line.starts_with("REPOSITORY") {
            let mut mod_container_str = String::from(line);
            let re = Regex::new(r"\s\s+").unwrap(); // 
                
            for matched in re.find_iter(&String::from(line)) {
                let found = matched.as_str();
                mod_container_str = mod_container_str.replace(found, "|");
            }
            let split_str: Vec<&str> = mod_container_str.split("|").collect();

            //println!("{mod_container_str}");
            let db = DockerContainer {
                repository: String::from(split_str.get(0).unwrap().to_owned()),                          
                tag: String::from(split_str.get(1).unwrap().to_owned()),                           
                image: String::from(split_str.get(2).unwrap().to_owned()),
                created: String::from(split_str.get(3).unwrap().to_owned()),    
                size: String::from(split_str.get(4).unwrap().to_owned())
            };
            containers.push(db);
            
        }
    }

    let dbdf = dockercontainer_vec_to_df(containers.clone());
    print_polars_df(&dbdf);
}



fn load_container_contexts(epi2me: &Epi2meSetup) -> Option<Vec<Container>> {
    println!("Loading container contexts ...");

    let mut docker_content: Vec<Container> = Vec::new();

    // load all EPI2ME installed workflows ...
    let src_dir = epi2me_db::find_db().unwrap().epi2wf_dir;
    let wfs = list_installed_workflows(&src_dir);

    load_installed_docker_artifacts();

    for workflow in wfs {
        //println!("{:?}", workflow);
        let workflow_id = vec![String::from(&workflow.project), String::from(&workflow.name)].join("/");
  
        let containers: Vec<String>;
        let _nf_config: HashMap<String, String>;

        let epi2me_installed_wf = glob_path_by_wfname(epi2me, &workflow.project, &workflow.name);
        if epi2me_installed_wf.is_some() {
            let mut pb = epi2me_installed_wf.unwrap().clone();
            pb.push("nextflow.config");
            (_nf_config, containers) = identify_containers(&pb);
            for container in containers {
                // println!("container == {}", container);
                docker_content.push( Container{workflow: String::from(&workflow_id), 
                    version:String::from(&workflow.version), 
                    dockcont:container, } )
            }
        } else {
            eprintln!("Unable to parse containers from [{}]", workflow_id);
            return None;
        }
    }
    return Some(docker_content);
}


pub async fn docker_agent(tempdir: &TempDir, epi2me: &Epi2meSetup, workflows: &Vec<String>, list: &bool, pull: &bool, twome: &Option<String>) {

    if !workflows.len() == 0 {
        println!("docker methods require a --workflow pointer to a workflow");
        return;
    }

    let docker_content = load_container_contexts(epi2me);
    if docker_content.is_none() {
        return;
    }

    if *list {
        println!("Make container vec pretty ...");
        let df = docker_vec_to_df(docker_content.clone().unwrap());
        print_polars_df(&df);
        return;
    }

    if twome.is_none() {
        eprintln!("requires a --twome path - please try again");
        return;
    }

    // sanity check the specified workflows ...
    for workflow in workflows {
        let mut found: bool = false;
        for containerc in docker_content.clone().unwrap() {
            if workflow.to_string().eq(&containerc.workflow) {
                found = true;
            }
        }
        if !found {
            eprintln!("specified workflow [{}] not found", workflow);
            return;
        }
    }

    // create a manifest
    let mut manifest = get_manifest(&tempdir.path).unwrap();
    let mut all_files: Vec<FileManifest> = Vec::new();

    // export the files ...
    for workflow in workflows {
        println!("surveying workflow [{:?}]", workflow);

        let mut containers: Vec<String> = Vec::new();
        let mut version: String = String::from("undefined");
        for containerc in docker_content.clone().unwrap() {
            if workflow.to_string().eq(&containerc.workflow) {
                let cont = String::from(&containerc.dockcont);
                version = containerc.version.clone();
                println!("{cont}");
                containers.push(cont.clone());
                if *pull {
                    pull_container(&cont).await;
                }
            }
        }
        let export_path = tempdir.path.clone();
        let mut p = PathBuf::from(&export_path);
        p.push("containers");
        let _ = fs::create_dir_all(&p);
        if !p.exists() {
            eprintln!("export path [{:?}] does not exist", p);
            return;
        } else if p.is_file() {
            eprintln!("export path [{:?}] is a file; folder required", p);
            return;
        } else {
            let arch = String::from(std::env::consts::ARCH);
            let folder = vec![String::from(workflow), String::from(&version), String::from(&arch)].join(".");
            println!("creating object = {folder}");
            p.push(folder);

            if !p.exists() {
                let state = fs::create_dir_all(&p);
                if state.is_err() {
                    eprintln!("failed to create folder {:?}", p);
                    return;
                }
            }
            let files = export_containers(&containers, &p).await;
            // println!("files == [{:?}]", files);

            let e: Epi2meContainer = Epi2meContainer{ 
                workflow: workflow.to_owned(),
                version: String::from(&version),
                architecture: String::from(&arch),
                files: files
            };
            // println!("e == {:?}", e);
            all_files.extend(e.files.clone());
            manifest.filecount += u64::try_from(e.files.len()).unwrap();
            manifest.files_size += file_manifest_size(&e.files);
            manifest.payload.push( Epi2MeContent::Epi2meContainer(e.clone()) ); 

            // and prepare the bundle ...
        }

    }

    println!("manifest -> {:?}", manifest);

}