use std::path::PathBuf;

use clap::{arg, value_parser, ArgAction, ArgMatches, Command};

use crate::{epi4you_errors::Epi4youError, tempdir::TempDir, xmanifest::{self, Epi2MeContent}};



pub const IMPORT2ME: &str = "import";

pub fn get_cli_setup() -> Command {


    let my_command = Command::new(IMPORT2ME)
        .about("import .2me format tar archive")
        .arg(arg!(--twome "twome archive file").action(ArgAction::Set).required(false).value_parser(value_parser!(String)))
        .arg(arg!(--force "force overwrite of exising content").action(ArgAction::SetTrue))
        ;
    return my_command;
}




    


pub async fn process_2me_import_command(args: &ArgMatches, tempdir: &TempDir) -> Result<(), Epi4youError> {
    let twome = args.get_one::<String>("twome").cloned();
    let force = args.get_one::<bool>("force").unwrap().to_owned();

    // ximporter::import_coordinator(&tempdir.path, twome, force).await;

    if twome.is_none() {
        log::error!("EPI2ME twome import requires a --twome <file> target to read");
        return Err(Epi4youError::Epi4youMissingRequired2MEartefact); 
    } 

    let path = PathBuf::from(twome.as_ref().unwrap());
    if !path.exists() {
        return Err(Epi4youError::RequiredPathMissing(path));
    } else if path.is_dir() {
        return Err(Epi4youError::FolderFoundWhenFileExpected(path));
    }

    let mut manifest = xmanifest::Epi2MeManifest::from_tarball(path.clone())?;
    let _payload = manifest.unpack_container_content(&tempdir.path, &path, &force)?;
    manifest.process_container_content(&tempdir.path)?;

    Ok(())
}