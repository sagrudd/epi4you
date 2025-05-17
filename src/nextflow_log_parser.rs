use std::collections::HashMap;

use regex::Regex;


pub struct NextFlowLogs {
    facet: HashMap<String, String>,
}

impl NextFlowLogs {

    pub fn init(log: &str) -> Self {




        let mut facet = HashMap::<String, String>::new();

        let re = Regex::new(r#"^\|+"#).unwrap();

        let lines = log.lines();
        for (ptr, line) in lines.into_iter().enumerate() {
            if ptr <= 30 {
                println!("{line}");
            }
            if line.contains(r"|") {
                println!("{line}");
            } else if line.contains(":") && line.starts_with(" ") {
                let a: Vec::<&str> = line.split(":").collect();
                facet.insert(a[0].trim().into(), a[1].trim().into());
            } 
        }

        return NextFlowLogs {
            facet,
        };

    }

    pub fn get_value(&self, key: &str) -> String {
        return String::from(self.facet.get(key).unwrap());
    }

    pub fn test(&self) {
        for (k, v) in self.facet.iter() {
            println!("{k}\t\t{v}");
        }
    }

}

/*

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
                // println!("!{line}");
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
                } else if line.contains(xxxkey) && pname.len() > 0 && line.contains(&pname) {
                    println!("extracting vers from [{}]", line);
                    let v = line[line.find(&pname).unwrap()+pname.len()..].trim();
                    version = String::from(&v[.. v.find("-").unwrap()]);
                    //println!("{v}");
                }
            }

*/