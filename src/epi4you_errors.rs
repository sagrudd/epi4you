
use std::path::PathBuf;

use serde::Serialize;


#[derive(Debug, Serialize)]
pub enum Epi4youError {

    AdditionalParameterRequired,
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
    
}