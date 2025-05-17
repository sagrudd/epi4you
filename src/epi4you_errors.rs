
use std::path::PathBuf;

use serde::Serialize;


#[derive(Debug, Serialize)]
pub enum Epi4youError {

    AdditionalParameterRequired,
    CannotVerifyManifestAuthenticity,
    Epi4youMissingRequired2MEartefact,
    ErrorInUnpackingTarElement,
    FailedToCreateFolder(PathBuf),
    FailedToParseFileContent,
    FileAlreadyExistsUnforcedExecution(PathBuf),
    FileFoundWhenFolderExpected(PathBuf),
    FileSelectionFailedFileNotFound,
    FileSelectionIsAmbiguous,
    FolderFoundWhenFileExpected(PathBuf),
    MalformedCLISetup,
    NextflowAnalysisFolderNotFound,
    RequiredPathMissing(PathBuf),
    SpecifiedNextflowRunNotFound(String),
    UnableToLocateNextflowBinary,
    UnableToResolveManifestObject,
    
}