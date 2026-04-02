use clap::{Arg, Command};
use create_2me::create_from_cli_run;
use env_logger::Env;
use epi4you_errors::Epi4youError;
use importer::import_from_2me;

mod app_db;
mod bundle;
mod dataframe;
mod epi2me_db;
mod epi2me_tar;
mod json;
mod provenance;
mod tempdir;

mod xmanifest;

mod epi2me_desktop_analysis;
mod epi2me_workflow;
mod nextflow_log_parser;

////////////////

pub mod epi4you;
pub mod epi4you_errors;

pub mod create_2me {
    pub mod create_from_cli_run;
}

pub mod importer {
    pub mod import_from_2me;
}

pub mod nextflow {
    pub mod nextflow_analysis;
    pub mod nextflow_log_item;
    pub mod nextflow_progress;
    pub mod nextflow_toolkit;
}

#[tokio::main]
async fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let mut temp_dir = match tempdir::get_tempdir() {
        Ok(temp_dir) => temp_dir,
        Err(err) => {
            eprintln!("error creating tempdir: {err:?}");
            std::process::exit(1);
        }
    };

    if let Err(err) = run(&mut temp_dir).await {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

async fn run(temp_dir: &mut tempdir::TempDir) -> Result<(), Epi4youError> {
    let use_args: Vec<Arg> = Vec::<Arg>::new();
    let mut subcmds: Vec<Command> = Vec::<Command>::new();

    subcmds.push(create_from_cli_run::get_cli_setup());
    subcmds.push(import_from_2me::get_cli_setup());

    let app = Command::new(epi4you::APPLICATION_NAME)
        .subcommand_required(false)
        .version(epi4you::APPLICATION_VERSION)
        .author(epi4you::APPLICATION_AUTHOR)
        .about(epi4you::APPLICATION_ABOUT)
        .long_about(epi4you::APPLICATION_DESCRIPTION)
        .args(use_args)
        .subcommands(subcmds.clone());

    match app.try_get_matches() {
        Ok(matches) => match matches.subcommand() {
            Some((create_from_cli_run::NEXTFLOW_RUN, sub_matches)) => {
                log::debug!(
                    "subcommand [{}] has been called",
                    create_from_cli_run::NEXTFLOW_RUN
                );
                create_from_cli_run::process_clicapture_command(sub_matches, temp_dir)
            }
            Some((import_from_2me::IMPORT2ME, sub_matches)) => {
                log::debug!(
                    "subcommand [{}] has been called",
                    import_from_2me::IMPORT2ME
                );
                import_from_2me::process_2me_import_command(sub_matches, temp_dir).await
            }
            Some((name, _)) => {
                log::error!("unexpected subcommand [{name}]");
                Err(Epi4youError::MalformedCLISetup)
            }
            None => Ok(()),
        },
        Err(err) => {
            err.print().ok();
            Ok(())
        }
    }
}
