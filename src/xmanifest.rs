use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::PathBuf,
};

use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use stringreader::StringReader;
use tar::Archive;

use crate::{
    app_db, epi2me_desktop_analysis::Epi2meDesktopAnalysis, epi2me_workflow::Epi2meWorkflow,
    epi4you_errors::Epi4youError, provenance::Epi2MeProvenance,
};

pub const MANIFEST_JSON: &str = "4u_manifest.json";
pub const UNDEFINED: &str = "undefined";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileManifest {
    pub filename: String,
    pub relative_path: String,
    pub size: u64,
    pub md5sum: String,
}

impl Default for FileManifest {
    fn default() -> FileManifest {
        FileManifest {
            filename: String::from(UNDEFINED),
            relative_path: String::from(UNDEFINED),
            size: 0,
            md5sum: String::from(UNDEFINED),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Epi2meContainer {
    pub workflow: String,
    pub version: String,
    pub architecture: String,
    pub files: Vec<FileManifest>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Epi2MeContent {
    Epi2mePayload(Epi2meDesktopAnalysis),
    Epi2meWf(Epi2meWorkflow),
    Epi2meContainer(Epi2meContainer),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epi2MeManifest {
    pub id: String,
    pub src_path: String,
    pub provenance: Vec<Epi2MeProvenance>,
    pub payload: Vec<Epi2MeContent>,
    pub filecount: u64,
    pub files_size: u64,
    pub signature: String,
}

impl Epi2MeManifest {
    pub fn new(src_path: PathBuf) -> Self {
        let mut man = Epi2MeManifest {
            id: String::from(UNDEFINED),
            src_path: src_path.to_string_lossy().into_owned(),
            provenance: Vec::new(),
            payload: Vec::new(),
            filecount: 0,
            files_size: 0,
            signature: String::from(UNDEFINED),
        };
        man.append_provenance(String::from("manifest_created"), None);
        if let Ok(hostname) = hostname::get() {
            man.append_provenance(
                String::from("hostname"),
                Some(hostname.to_string_lossy().into_owned()),
            );
        }
        man
    }

    pub fn from_tarball(tarball: PathBuf) -> Result<Self, Epi4youError> {
        if !tarball.exists() {
            return Err(Epi4youError::RequiredPathMissing(tarball));
        }

        if tarball.is_dir() {
            return Err(Epi4youError::FolderFoundWhenFileExpected(tarball));
        }

        let file =
            File::open(&tarball).map_err(|_| Epi4youError::FailedToReadPath(tarball.clone()))?;
        let mut archive = Archive::new(file);
        let entries = archive
            .entries()
            .map_err(|_| Epi4youError::CannotVerifyManifestAuthenticity)?;

        for entry in entries {
            let mut file = entry.map_err(|_| Epi4youError::CannotVerifyManifestAuthenticity)?;
            let file_path = file
                .path()
                .map_err(|_| Epi4youError::CannotVerifyManifestAuthenticity)?
                .into_owned();

            let is_manifest = file_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == MANIFEST_JSON)
                .unwrap_or(false);

            if !is_manifest {
                continue;
            }

            let mut buffer = String::new();
            file.read_to_string(&mut buffer)
                .map_err(|_| Epi4youError::FailedToReadPath(file_path.clone()))?;

            let manifest: Epi2MeManifest = serde_json::from_str(&buffer)
                .map_err(|_| Epi4youError::FailedToParseFileContent)?;

            if manifest.is_trusted() {
                return Ok(manifest);
            }

            log::error!("checksum differences - this repository is untrusted");
            return Err(Epi4youError::CannotVerifyManifestAuthenticity);
        }

        Err(Epi4youError::UnableToResolveManifestObject)
    }

    pub fn note_packaged_analysis(&mut self, id: &String) {
        let action = vec![String::from("analysis_bundled"), String::from(id)].join(": ");
        let prov = Epi2MeProvenance::init(action, None);
        self.provenance.push(prov);
    }

    pub fn note_packaged_workflow(&mut self, id: &String) {
        let action = vec![String::from("workflow_bundled"), String::from(id)].join(": ");
        let prov = Epi2MeProvenance::init(action, None);
        self.provenance.push(prov);
    }

    pub fn untar(
        &mut self,
        tarfile: &PathBuf,
        temp_dir: &PathBuf,
    ) -> Result<PathBuf, Epi4youError> {
        log::info!("untar of file [{:?}] into [{:?}]", tarfile, temp_dir);

        let file =
            File::open(tarfile).map_err(|_| Epi4youError::FailedToReadPath(tarfile.clone()))?;
        let mut archive = Archive::new(file);

        for entry in archive
            .entries()
            .map_err(|_| Epi4youError::ErrorInUnpackingTarElement)?
        {
            let mut file = entry.map_err(|_| Epi4youError::ErrorInUnpackingTarElement)?;
            let fp = file
                .path()
                .map_err(|_| Epi4youError::ErrorInUnpackingTarElement)?
                .into_owned();
            log::debug!("unpacking [{:?}] to [{:?}]", fp, temp_dir);

            file.unpack_in(temp_dir)
                .map_err(|_| Epi4youError::ErrorInUnpackingTarElement)?;
        }

        Ok(temp_dir.to_owned())
    }

    pub fn unpack_container_content(
        &mut self,
        temp_dir: &PathBuf,
        twome: &PathBuf,
        _force: &bool,
    ) -> Result<(), Epi4youError> {
        self.untar(twome, temp_dir)?;
        Ok(())
    }

    pub fn process_container_content(&self, temp_dir: &PathBuf) -> Result<(), Epi4youError> {
        for x in &self.payload {
            match x {
                Epi2MeContent::Epi2meWf(epi2me_workflow) => {
                    log::info!("importing Workflow [{}]", epi2me_workflow.name);
                }
                Epi2MeContent::Epi2mePayload(desktop_analysis) => {
                    log::info!("importing DesktopAnalysis [{}]", &desktop_analysis.id);
                    app_db::insert_untarred_desktop_analysis(desktop_analysis, temp_dir);
                }
                Epi2MeContent::Epi2meContainer(epi2me_container) => {
                    log::info!("importing Epi2meContainer [{}]", &epi2me_container.workflow);
                }
            }
        }

        Ok(())
    }

    pub fn is_trusted(&self) -> bool {
        let resignature = self.get_signature();
        log::info!(
            "looking for signature parity [{}]|[{}]",
            self.signature,
            resignature
        );
        self.signature == resignature
    }

    fn append_provenance(&mut self, what: String, value: Option<String>) {
        let prov = Epi2MeProvenance::init(what, value);
        self.provenance.push(prov);
    }

    pub fn get_signature(&self) -> String {
        let mut unsigned = self.clone();
        unsigned.signature = String::from(UNDEFINED);
        sha256_str_digest(serde_json::to_string_pretty(&unsigned).unwrap().as_str())
    }

    fn sign(&mut self) {
        self.signature = self.get_signature();
    }

    pub fn to_string(&mut self) -> String {
        self.sign();
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn print(&mut self) {
        println!("{}", self.to_string());
    }

    pub fn write(&mut self, dest: &PathBuf) {
        println!("writing manifest to path [{:?}]", dest);
        if fs::write(dest, self.to_string()).is_err() {
            println!("Error with writing manifest to file");
        }
    }
}

pub fn sha256_digest(path: &str) -> String {
    let input = File::open(path).unwrap();
    let mut reader = BufReader::new(input);

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    HEXUPPER.encode(context.finish().as_ref())
}

pub fn sha256_str_digest(payload_str: &str) -> String {
    let streader = StringReader::new(payload_str);
    let mut reader = BufReader::new(streader);

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    HEXUPPER.encode(context.finish().as_ref())
}

#[cfg(test)]
mod tests {
    use super::Epi2MeManifest;
    use std::path::PathBuf;

    #[test]
    fn manifest_signature_round_trip_is_trusted() {
        let mut manifest = Epi2MeManifest::new(PathBuf::from("/tmp/test-manifest"));
        manifest.signature = manifest.get_signature();

        assert!(manifest.is_trusted());
    }
}
