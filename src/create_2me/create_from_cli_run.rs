use std::{env, path::PathBuf};

use clap::{arg, value_parser, ArgAction, ArgMatches, Command};

use crate::{
    epi4you_errors::Epi4youError, nextflow::nextflow_toolkit::NextFlowResultFolder,
    tempdir::TempDir,
};

pub const NEXTFLOW_RUN: &str = "nextflow-run";

pub fn get_cli_setup() -> Command {
    let my_command = Command::new(NEXTFLOW_RUN)
        .about("create 2me from nextflow cli results")
        .arg(arg!(--list "List analyses run using Nextflow CLI").action(ArgAction::SetTrue))
        .arg(
            arg!(--nxf_bin "path to nextflow binary (if not obvious)")
                .action(ArgAction::Set)
                .required(false)
                .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(--nxf_work "path to nextflow work folder")
                .action(ArgAction::Set)
                .required(false)
                .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(--runid "export EPI2ME analysis by run_name")
                .action(ArgAction::Set)
                .required(false)
                .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(--twome "twome archive file")
                .action(ArgAction::Set)
                .required(false)
                .value_parser(value_parser!(String)),
        )
        .arg(arg!(--force "force overwrite of exising twome archive").action(ArgAction::SetTrue));
    return my_command;
}

pub fn process_clicapture_command(
    args: &ArgMatches,
    tempdir: &TempDir,
) -> Result<(), Epi4youError> {
    let nxf_bin = args.get_one::<String>("nxf_bin").cloned();
    let nxf_work = args.get_one::<String>("nxf_work").cloned();
    let runid = args.get_one::<String>("runid").cloned();
    let twome = args.get_one::<String>("twome").cloned();
    let list = args.get_one::<bool>("list").copied().unwrap_or(false);
    let force = args.get_one::<bool>("force").copied().unwrap_or(false);

    let mut nxf_workdir = nxf_work.clone();
    if nxf_workdir.is_none() {
        let cwd = env::current_dir()
            .map_err(|_| Epi4youError::RequiredPathMissing(PathBuf::from(".")))?;
        nxf_workdir = Some(cwd.to_string_lossy().into_owned());
        log::info!(
            "Setting nextflow workdir to cwd [{:?}]",
            nxf_workdir.clone().unwrap()
        );
    }

    let nextflow_run_folder =
        NextFlowResultFolder::init(PathBuf::from(nxf_workdir.clone().unwrap()), nxf_bin)?;

    if list {
        nextflow_run_folder.list_runs();
    } else {
        let runid = runid.ok_or(Epi4youError::AdditionalParameterRequired)?;
        let twome = twome.ok_or(Epi4youError::AdditionalParameterRequired)?;
        let wf_analysis = nextflow_run_folder.verify_cli_entity(runid)?;
        nextflow_run_folder.bundle_cli_run(tempdir, wf_analysis, &twome, &force)?;
    }

    Ok(())
}
