use std::collections::HashMap;


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