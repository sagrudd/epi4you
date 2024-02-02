use std::{path::PathBuf, collections::HashMap, fs::{self, File}, process::Command, io::Write};
use docker_api::{Docker, opts::{PullOpts, TagOpts}, Image};
use futures::{StreamExt, TryStreamExt};
use polars::prelude::*;
use polars_core::{frame::DataFrame, series::Series};
use regex::Regex;
use serde::{Serialize, Deserialize};

use crate::{tempdir::TempDir, epi2me_db::{Epi2meSetup, self}, xmanifest::{FileManifest, Epi2meContainer, Epi2MeContent}, bundle::get_relative_path};


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

#[derive(Clone)]
pub struct Epi2meDocker {
    temp_dir: PathBuf,
    epi2me: Option<Epi2meSetup>,
    containers: Vec<Container>
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

impl Epi2meDocker {
    fn new(tempdir: PathBuf) -> Self {
        Epi2meDocker {
            temp_dir: tempdir,
            epi2me: epi2me_db::find_db(),
            containers: Vec::new(),
        }    
    }

    pub async fn from_epi2me_container(epi2me_container: Epi2meContainer, temp_dir: &PathBuf) -> Option<Self> {
        let basis = Epi2meDocker {
            temp_dir: temp_dir.to_owned(),
            epi2me: epi2me_db::find_db(),
            containers: Vec::new(),
        };

        if !epi2me_container.architecture.eq(&String::from(std::env::consts::ARCH)) {
            eprintln!("there is a mismatch with arch - archive is [{:?}]", epi2me_container.architecture);
            return None;
        }
        let docker = basis.new_docker();
        for f in epi2me_container.files {
            let mut fb = temp_dir.to_owned();
            fb.push(f.relative_path);
            fb.push(f.filename);
            if fb.exists() {
                println!("file [{:?}] being imported", fb);

                let images = docker.images();
                let f = File::open(fb).expect("Unable to open file");
                let reader = Box::from(f);
                let mut stream = images.import(reader);
    
                while let Some(import_result) = stream.next().await {
                    match import_result {
                        Ok(output) => println!("{output:?}"),
                        Err(e) => eprintln!("Error: {e}"),
                    }
                }
            }
        }
        return Some(basis); 
    }


    fn extract_containers(&self, config: &HashMap<String, String>) -> Vec<String> {
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

    fn identify_containers(&self, pb: &PathBuf) -> (HashMap<String, String>, Vec<String>) {
        let contents = fs::read_to_string(&pb).unwrap();
    
        let config = crate::xnf_parser::nextflow_parser(&contents);
        return (config.clone(), self.extract_containers(&config.clone()));
    }

    fn populate(&mut self) {
        println!("docker::populate");
        let e = self.epi2me.clone().unwrap();
        let td = self.temp_dir.clone();
        
        let workflows = crate::xworkflows::Epi2meWorkflow::new(td, Some(e));

        for workflow in workflows.wf_vector() {
            let workflow_id = vec![String::from(&workflow.project), String::from(&workflow.name)].join("/");
            // println!("{:?}", workflow_id);
            let containers: Vec<String>;
            let _nf_config: HashMap<String, String>;

            let epi2me_installed_wf = workflows.glob_path_by_wfname(&workflow.project, &workflow.name);
            if epi2me_installed_wf.is_some() {
                let mut pb = epi2me_installed_wf.unwrap().clone();
                pb.push("nextflow.config");
                (_nf_config, containers) = self.identify_containers(&pb);
                for container in containers {
                    // println!("container == {}", container);
                    let container = Container{
                        workflow: String::from(&workflow_id), 
                        version:String::from(&workflow.version), 
                        dockcont:container,
                    };
                    self.containers.push(container);
                }
            } else {
                eprintln!("Unable to parse containers from [{}]", workflow_id);
            }
        }
    }


    pub fn containers_df(&self) -> DataFrame {
        let x: Vec<String> = self.containers.iter().map(|v| String::from(&v.workflow)).collect();
        let y: Vec<String> = self.containers.iter().map(|v| String::from(&v.version)).collect();
        let z: Vec<String> = self.containers.iter().map(|v| String::from(&v.dockcont)).collect();
        let xx: Series = Series::new("workflow", x);
        let yy = Series::new("version", y);
        let zz = Series::new("dockcont", z);
        DataFrame::new(vec![xx, yy, zz]).unwrap()
    }

    fn get_workflow_containers(&self, workflow_name: &str) -> Vec<Container> {
        let mut containers: Vec<Container> = Vec::new();
        for container in self.containers.iter() {
            if workflow_name.to_string().eq(&container.workflow) {
                containers.push(container.clone());
            }
        }
        return containers;
    }

    fn get_docker_context_socket(&self) -> String {
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


    fn new_docker(&self) -> Docker {
        // we should parse connection strings via -  `docker context ls`
        let socket = self.get_docker_context_socket();
        //Ok(Docker::unix("/var/run/docker.sock"))
        Docker::unix(socket)
    }


    

    fn omatch(&self, key: &str, txt: &str) -> Option<String> {
        //println!("looking for || {}", key);
        let re_status = Regex::new(key).unwrap();
        let lmatch = re_status.find(&txt);
        if lmatch.is_some() {
            let matched = lmatch.unwrap();
            let found = matched.as_str();
            //println!("omatch [{}] --> [{}]", key, found);
            if key.ends_with(r#"""#) || key.ends_with(r#")"#) {
                let olabel = self.omatch("^[^:]+:", &found);
                if olabel.is_some() {
                    return Some(unlabel(found, &olabel.unwrap()));
                }
            }
            return Some(String::from(found));
        }
        println!("returning None!");
        return None;
    }

    async fn retag_image(&self, docker: &Docker, installed: &String, requested: &String) {
        println!("retagging image [{installed}] -> [{requested}]");    
        let x = requested.split_once(":");
        if x.is_some() {
            let (repo, tag) = x.unwrap();
            let tag_opts = TagOpts::builder().repo(repo).tag(tag).build();
            let image = Image::new(docker.clone(), installed);
            let status = image.tag(&tag_opts).await;
            if status.is_err() {
                eprintln!("Error: {:?}", status.err());
            } 
        }
    }

    async fn pull_container(&self, container: &Container) {
        let docker = self.new_docker();
        println!("pulling container [{}]", container.dockcont);
        let opts = PullOpts::builder().image(container.dockcont.to_string()).build();

        let images = docker.images();
        let mut stream = images.pull(&opts);

        while let Some(pull_result) = stream.next().await {
            // println!("<?>");
            match pull_result {
                Ok(output) => {
                    let x = format!("{output:?}");

                    let ostatus = self.omatch(r#"status: "[^"]+""#, &x);
                    if ostatus.is_some() {
                        let status = ostatus.unwrap();
                        if status == "Extracting" {
                            let oprogress = self.omatch(r#"progress: [^\)]+\)"#, &x);
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
                                self.retag_image(&docker, &installed_img, &container.dockcont.to_string()).await;
                            } else if status.starts_with(up2da_image) {
                                let installed_img = status.replace(up2da_image, "");
                                self.retag_image(&docker, &installed_img, &container.dockcont.to_string()).await;
                            }
                        }
                    }
                },
                Err(e) => eprintln!("oops {e}"),
            }
        }

    }


    async fn export_container(&self, container: &Container, p: &PathBuf) -> Option<FileManifest> {

        let docker = self.new_docker();
        let epi2 = self.epi2me.as_ref().unwrap();
        let local_prefix = &epi2.epi2path;
            
        println!("exporting [{}]", &container.dockcont);

        let mut write_path = p.clone();
        let mut tar_file = String::from(&container.dockcont);
        tar_file = tar_file.replace("/", "-");
        tar_file = tar_file.replace(":", "-");
        tar_file = format!("{}.tar", &tar_file);
        write_path.push(&tar_file);

        println!("writing to file [{}]", write_path.display());

        let images = &docker.images();
        let image = images.get(&container.dockcont);
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
                return None;
            } else {

                let relative_path = crate::bundle::clip_relative_path(&write_path, &local_prefix);
                let file_size = &write_path.metadata().unwrap().len();
                let checksum = crate::bundle::sha256_digest(&write_path.as_os_str().to_str().unwrap());

                let man = FileManifest{filename: tar_file,
                    relative_path: String::from(relative_path.clone().to_string_lossy().to_string()),
                    size: *file_size,
                    md5sum: checksum
                };

                return Some(man);
            }
        } else {
            eprintln!("file is fubar\n{:?}", file.err());
            return None;
        }


        
    }


    pub async fn save_workflow_containers(&self, workflow_name: &str, pull: &bool) -> Option<Epi2meContainer> {
        let containers = self.get_workflow_containers(workflow_name);
        let mut version = "";
        let mut files: Vec<FileManifest> = Vec::new();
        for container in containers.iter() {
            if *pull {
                self.pull_container(&container).await;
            }
            let mut dest = self.temp_dir.clone();
            dest.push("containers");
            version = container.version.as_str();
            let arch = String::from(std::env::consts::ARCH);
            let folder = vec![container.workflow.to_string(), container.version.to_string(), String::from(&arch)].join(".");
            println!("creating object = {folder}");
            dest.push(folder);
            if !dest.exists() {
                let state = fs::create_dir_all(&dest);
                if state.is_err() {
                    eprintln!("failed to create folder {:?}", dest);
                    return None;
                }
            }

            let file = self.export_container(&container.clone(), &dest).await;
            if file.is_some() {
                files.push(file.unwrap());
            }
        }
        //return files;  


        let e: Epi2meContainer = Epi2meContainer{ 
            workflow: workflow_name.to_string(),
            version: String::from(version),
            architecture: String::from(std::env::consts::ARCH),
            files: files
        };
        return Some(e);
    }


    pub fn print(&mut self) {
        crate::dataframe::print_polars_df(&self.containers_df());
    }

    pub fn describes_workflow(&self, workflow: &String) -> bool {
        return self.containers.iter().any(|x| workflow.to_string().eq(&x.workflow));
    }


}





pub async fn docker_agent(tempdir: &TempDir, workflows: &Vec<String>, list: &bool, pull: &bool, twome: &Option<String>) {
    println!("--docker_agent");
    let mut edocker = Epi2meDocker::new(tempdir.path.to_path_buf());
    edocker.populate();

    if *list {
        println!("Make container vec pretty ...");
        edocker.print();
        return;
    }

    if workflows.len() == 0 {
        println!("docker methods require a --workflow pointer to a workflow");
        return;
    }

    if twome.is_none() {
        eprintln!("requires a --twome path - please try again");
        return;
    }

    // sanity check the specified workflows ...
    for workflow in workflows.iter() {
        if !edocker.describes_workflow(workflow) {
            eprintln!("specified workflow [{}] not found", workflow);
            return;
        }
    }

    let e2 = edocker.clone().epi2me.unwrap();

    let mut manifest = crate::xmanifest::Epi2MeManifest::new(e2.epi2path.clone());
    let mut all_files: Vec<FileManifest> = Vec::new();
    
    // export the files ...
    for workflow in workflows.iter() {
        println!("surveying workflow [{:?}]", workflow);
        let epiconts = edocker.save_workflow_containers(workflow, pull).await;

        if epiconts.is_some() {
            let e = epiconts.unwrap();
            all_files.extend(e.files.clone());
            manifest.filecount += u64::try_from(e.files.len()).unwrap();
            let f: u64 = e.files.iter().map(|x| x.size).sum();
            manifest.files_size += f;
            manifest.payload.push( Epi2MeContent::Epi2meContainer(e.clone()) ); 
        }
    }

    manifest.print();
    let mut manifest_pb = PathBuf::from(&tempdir.path);
    manifest_pb.push(crate::xmanifest::MANIFEST_JSON);
    manifest.write(&manifest_pb);
    manifest.tar( 
        &get_relative_path(&manifest_pb, &e2.epi2path), 
        &PathBuf::from(twome.clone().unwrap())
    );

}
