use std::{path::PathBuf, collections::HashMap};

use crate::{manifest::{load_manifest_from_tarball, Epi2MeContent, is_manifest_honest, import_resolved_content}, tempdir::{TempDir, get_named_tempdir}};



pub fn import_manager(twome: &Option<String>, force: &bool) {

    if twome.is_none() {
        eprintln!("EPI2ME twome import requires a --twome <file> target to read");
        return; 
    } else {
        let path = PathBuf::from(twome.as_ref().unwrap());
        let manifest = load_manifest_from_tarball(&path);

        if manifest.is_some() {

            let mut content: HashMap<String, TempDir> = HashMap::new();

            let payload = &manifest.as_ref().unwrap().payload;
            for paypay in payload {
                
                match paypay {
                    Epi2MeContent::Epi2meWf(epi2me_workflow) => {
                        let files = &epi2me_workflow.files;
                        for file in files {
                            if !content.contains_key(&file.relative_path) {
                                content.insert(file.relative_path.to_owned(), get_named_tempdir(&file.relative_path).unwrap());
                            }
                        }
                    },
                    
                    Epi2MeContent::Epi2mePayload(desktop_analysis) => {
                        let files = &desktop_analysis.files;
                        for file in files {
                            if !content.contains_key(&file.relative_path) {
                                content.insert(file.relative_path.to_owned(), get_named_tempdir(&file.relative_path).unwrap());
                            }
                        }
                    },

                    Epi2MeContent::Epi2meContainer(epi2me_container) => {
                        println!("importing Epi2meContainer");
                    },
                    
                }

            }

            let honest = is_manifest_honest(&manifest.unwrap(), &path, force);
            if honest.is_none() {
                eprintln!("this epi4you archive is not trusted - exiting");
                return;
            } if honest.is_some() {
                println!("importing something");
                import_resolved_content(&honest.unwrap(), force);
            }

            

        } else {
            eprintln!("This archive may be malformed - cannot continue");
        }
    }
    

    
}