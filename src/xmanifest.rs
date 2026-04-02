//! Manifest primitives for `.2me` archives.
//!
//! Oxford Nanopore positions EPI2ME Desktop as a local analysis environment for
//! nanopore workflows, with workflow delivery and offline distribution handled
//! through its newer 2ME architecture. In `epi4you`, this module is the bridge
//! between that broader EPI2ME packaging idea and the concrete archive format
//! we read and write here.
//!
//! The central job of this file is to describe:
//!
//! - what content goes into a portable archive,
//! - how that content is fingerprinted,
//! - how an archive is verified before import, and
//! - how the unpacked payload is handed back to local EPI2ME-style storage.

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

/// Canonical filename used for the serialized manifest inside a `.2me` tarball.
pub const MANIFEST_JSON: &str = "4u_manifest.json";

/// Shared placeholder used for fields that are intentionally not populated yet.
pub const UNDEFINED: &str = "undefined";

/// Per-file metadata stored in the manifest.
///
/// EPI2ME analyses and workflows are ultimately directory trees on disk. When
/// `epi4you` packages them, each file is recorded with a relative path and a
/// digest so the import side can reason about what was bundled.
///
/// The field name `md5sum` is a historical holdover from earlier code. The
/// implementation currently stores a SHA-256 digest in this slot.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileManifest {
    /// Basename of the file inside its relative directory.
    pub filename: String,
    /// Path relative to the archive root used when reconstructing the tree.
    pub relative_path: String,
    /// Size in bytes at bundle time.
    pub size: u64,
    /// SHA-256 digest captured at bundle time.
    pub md5sum: String,
}

impl Default for FileManifest {
    /// Builds an obviously invalid placeholder entry.
    ///
    /// This is useful when a caller needs a sentinel value rather than an
    /// `Option<FileManifest>`.
    fn default() -> FileManifest {
        FileManifest {
            filename: String::from(UNDEFINED),
            relative_path: String::from(UNDEFINED),
            size: 0,
            md5sum: String::from(UNDEFINED),
        }
    }
}

/// Description of exported workflow containers.
///
/// EPI2ME workflows commonly rely on containerized tools. This struct groups
/// the saved image tar files associated with one workflow revision so they can
/// be carried alongside the workflow or analysis payload.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Epi2meContainer {
    /// Workflow name that the container set belongs to.
    pub workflow: String,
    /// Workflow version or revision string associated with the container set.
    pub version: String,
    /// Target architecture for the exported image set.
    pub architecture: String,
    /// Tar files or other exported artefacts that realize the container set.
    pub files: Vec<FileManifest>,
}

/// Tagged payload variants that may appear inside a `.2me` bundle.
///
/// At the project level, `epi4you` deals with three transportable concepts:
///
/// - a Desktop analysis result,
/// - a workflow installation, and
/// - a container bundle.
///
/// Using a tagged enum keeps the manifest self-describing across import/export.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Epi2MeContent {
    /// A GUI-visible analysis instance suitable for Desktop import.
    Epi2mePayload(Epi2meDesktopAnalysis),
    /// A workflow installation tree.
    Epi2meWf(Epi2meWorkflow),
    /// A set of container artefacts associated with a workflow.
    Epi2meContainer(Epi2meContainer),
}

/// Top-level manifest stored alongside bundle contents.
///
/// The manifest is the archive's inventory, provenance record, and integrity
/// envelope. It tells import code what the archive claims to contain and
/// exposes enough metadata to recreate local EPI2ME-style state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epi2MeManifest {
    /// Manifest identifier. Currently left unset in most flows.
    pub id: String,
    /// Source path that was being packaged when the manifest was created.
    pub src_path: String,
    /// Audit trail for archive creation and subsequent packaging events.
    pub provenance: Vec<Epi2MeProvenance>,
    /// Typed payload records embedded in the archive.
    pub payload: Vec<Epi2MeContent>,
    /// Aggregate count of files referenced by the payload.
    pub filecount: u64,
    /// Aggregate size of referenced files in bytes.
    pub files_size: u64,
    /// Digest over the unsigned manifest payload.
    pub signature: String,
}

impl Epi2MeManifest {
    /// Creates a new manifest seeded with local provenance.
    ///
    /// The provenance entries are intentionally lightweight. They provide enough
    /// context to understand where a bundle came from without depending on an
    /// external EPI2ME service.
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

    /// Reads, deserializes, and verifies the manifest embedded in a tarball.
    ///
    /// Import is intentionally manifest-first: if the archive does not expose a
    /// valid manifest, or if the manifest signature does not match the payload
    /// representation, `epi4you` treats the archive as untrusted and refuses to
    /// continue.
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

    /// Appends provenance describing the packaging of an analysis payload.
    pub fn note_packaged_analysis(&mut self, id: &String) {
        let action = vec![String::from("analysis_bundled"), String::from(id)].join(": ");
        let prov = Epi2MeProvenance::init(action, None);
        self.provenance.push(prov);
    }

    /// Extracts the archive into a temporary working directory.
    ///
    /// Import happens in a scratch directory first so later steps can validate
    /// and reorganize content before it is copied into long-lived locations.
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

    /// Unpacks the raw archive contents in preparation for import processing.
    ///
    /// The `_force` flag is accepted to match the higher-level import API even
    /// though unpacking itself does not yet use it.
    pub fn unpack_container_content(
        &mut self,
        temp_dir: &PathBuf,
        twome: &PathBuf,
        _force: &bool,
    ) -> Result<(), Epi4youError> {
        self.untar(twome, temp_dir)?;
        Ok(())
    }

    /// Dispatches unpacked payloads into the appropriate local import flow.
    ///
    /// Today the most concrete path is desktop analysis import, but the method
    /// is intentionally shaped to become the single content router for all
    /// manifest payload variants.
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

    /// Verifies that the stored signature matches the manifest's current value.
    ///
    /// This is a manifest-level trust check, not a complete per-file
    /// attestation. It answers "has the manifest payload changed?" rather than
    /// "have all extracted files been independently re-hashed?".
    pub fn is_trusted(&self) -> bool {
        let resignature = self.get_signature();
        log::info!(
            "looking for signature parity [{}]|[{}]",
            self.signature,
            resignature
        );
        self.signature == resignature
    }

    /// Adds one provenance event to the manifest history.
    fn append_provenance(&mut self, what: String, value: Option<String>) {
        let prov = Epi2MeProvenance::init(what, value);
        self.provenance.push(prov);
    }

    /// Computes the canonical manifest signature.
    ///
    /// The signature is derived from the serialized manifest with the signature
    /// field blanked out, which keeps signing deterministic.
    pub fn get_signature(&self) -> String {
        let mut unsigned = self.clone();
        unsigned.signature = String::from(UNDEFINED);
        sha256_str_digest(serde_json::to_string_pretty(&unsigned).unwrap().as_str())
    }

    /// Updates [`Self::signature`] to match the manifest's current contents.
    fn sign(&mut self) {
        self.signature = self.get_signature();
    }

    /// Serializes the manifest, signing it first.
    pub fn to_string(&mut self) -> String {
        self.sign();
        serde_json::to_string_pretty(self).unwrap()
    }

    /// Writes the signed manifest to disk.
    pub fn write(&mut self, dest: &PathBuf) {
        println!("writing manifest to path [{:?}]", dest);
        if fs::write(dest, self.to_string()).is_err() {
            println!("Error with writing manifest to file");
        }
    }
}

/// Computes a SHA-256 digest for a file on disk.
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

/// Computes a SHA-256 digest for an in-memory string payload.
///
/// This is used for manifest signing because the signature is based on the
/// serialized JSON representation rather than a file handle.
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
