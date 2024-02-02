use std::path::PathBuf;

use crate::xmanifest::Epi2MeContent;



pub async fn import_coordinator(temp_dir: &PathBuf, twome: &Option<String>, force: &bool) {

    if twome.is_none() {
        eprintln!("EPI2ME twome import requires a --twome <file> target to read");
        return; 
    } 
    
    let path = PathBuf::from(twome.as_ref().unwrap());

    let omanifest = crate::xmanifest::Epi2MeManifest::from_tarball(path.clone());
    if omanifest.is_none() {
        eprintln!("Failed to find manifest within [{:?}]", &path);
        return;
    }

    let mut manifest = omanifest.unwrap();
    manifest.print();

    let payload = manifest.is_manifest_honest(temp_dir, &path, force);
    if payload.is_some() {
        println!("Content makes sense ...");
        for x in payload.unwrap().iter() {
            let epi2me_content = x.to_owned();

            match epi2me_content {
                Epi2MeContent::Epi2meWf(epi2me_workflow) => {

                },
                
                Epi2MeContent::Epi2mePayload(desktop_analysis) => {

                },

                Epi2MeContent::Epi2meContainer(epi2me_container) => {
                    let x = crate::xdocker::Epi2meDocker::from_epi2me_container(epi2me_container, temp_dir).await;
                },
                
            }

        }
    }
}