use std::path::PathBuf;

use clap::{arg, value_parser, ArgAction, ArgMatches, Command};

use crate::{epi4you_errors::Epi4youError, tempdir::TempDir, xmanifest};

pub const IMPORT2ME: &str = "import";

pub fn get_cli_setup() -> Command {
    let my_command = Command::new(IMPORT2ME)
        .about("import .2me format tar archive")
        .arg(
            arg!(--twome "twome archive file")
                .action(ArgAction::Set)
                .required(false)
                .value_parser(value_parser!(String)),
        )
        .arg(arg!(--force "force overwrite of exising content").action(ArgAction::SetTrue));
    return my_command;
}

pub async fn process_2me_import_command(
    args: &ArgMatches,
    tempdir: &TempDir,
) -> Result<(), Epi4youError> {
    let twome = args.get_one::<String>("twome").cloned();
    let force = args.get_one::<bool>("force").copied().unwrap_or(false);

    // ximporter::import_coordinator(&tempdir.path, twome, force).await;

    let twome = twome.ok_or(Epi4youError::Epi4youMissingRequired2MEartefact)?;

    let path = PathBuf::from(twome);
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
