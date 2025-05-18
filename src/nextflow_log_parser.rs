use std::collections::HashMap;

use regex::Regex;
use url::{Position, Url};


pub struct NextFlowLogs {
    facet: HashMap<String, String>,
}

impl NextFlowLogs {

    pub fn init(log: &str) -> Self {

        let mut facet = HashMap::<String, String>::new();

        let mut name = "";
        let mut revision = "";
        let revision_key = " - revision: ";
        let url_str_key = r"Launching `";
        let mut project = String::from("");
        let mut pname = String::from("");
        let mut version = String::from("dev");
        let xxxkey = "||||||||||";

        //let re = Regex::new(r#"^\|+"#).unwrap();

        let lines = log.lines();
        for (ptr, line) in lines.into_iter().enumerate() {
            /*if ptr <= 30 {
                println!("{line}");
            }*/
            if line.contains(url_str_key) {
                log::error!("{line}");

                let clipped_line = &line[line.find(url_str_key).unwrap()+url_str_key.len()..];

                name = &clipped_line[clipped_line.find("[").unwrap()+1..clipped_line.find("]").unwrap()];
                revision = &clipped_line[clipped_line.find(revision_key).unwrap()+revision_key.len()..];
                revision = &revision[..revision.find(" ").unwrap()];
                let mut url_str = &line[line.find(url_str_key).unwrap()+url_str_key.len()..];
                url_str = &url_str[..url_str.find("`").unwrap()];
    
                let url = Url::parse(url_str);
                if url.is_ok() {
                    let data_url_payload = &url.unwrap()[Position::AfterHost..][1..];
                    log::debug!("{:?}", &data_url_payload);
    
                    let x = &data_url_payload.split_once('/');
                    if x.is_some() {
                        let (aproject, apname) = x.clone().unwrap();
                        project = String::from(aproject);
                        pname = String::from(apname);
                    }
                }
            //} else if line.contains(":") && line.starts_with(" ") {
            //    let a: Vec::<&str> = line.split(":").collect();
            //    facet.insert(a[0].trim().into(), a[1].trim().into());
            } else if line.contains(xxxkey) && line.contains(&pname) {
                log::error!("extracting vers from [{}]", line);
                let v = line[line.find(&pname).unwrap()+pname.len()..].trim();
                if v.contains("-") {
                    let vstr = &v[.. v.find("-").unwrap()];
                    version = String::from(vstr);
                }
            }
        }

        facet.insert("name".into(), name.into());
        facet.insert("revision".into(), revision.into());
        facet.insert("project".into(), project.into());
        facet.insert("pname".into(), pname.into());
        facet.insert("version".into(), version.into());
        

        return NextFlowLogs {
            facet,
        };

    }

    pub fn get_value(&self, key: &str) -> String {
        return String::from(self.facet.get(key).unwrap());
    }

    pub fn test(&self) {
        for (k, v) in self.facet.iter() {
            log::debug!("{k}\t\t{v}");
        }
    }

}

/*


        
            let lines = nextflow_stdout.split("\n");
            for line in lines {
                // println!("!{line}");
                
                
                
                
            }

*/