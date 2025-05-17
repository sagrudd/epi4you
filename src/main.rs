use app_db::dbmanager;
use clap::{Arg, ArgAction, Command, Parser, Subcommand};
use create_2me::create_from_cli_run;
use env_logger::Env;
use epi2me_db::epi2me_manager;
use epi4you_errors::Epi4youError;
use importer::import_from_2me;
use workflow::workflow_manager;

mod app_db;
mod bundle;
mod epi2me_db;
mod epi2me_tar;
mod dataframe;
// mod importer;
mod json;
// mod manifest;
mod depme_nextflow;
mod provenance;
mod tempdir;
mod workflow;
mod settings;

mod docker;
mod xmanifest;
mod xnf_parser;
mod xworkflows;

mod epi2me_desktop_analysis;
mod epi2me_workflow;
mod nextflow_log_parser;

////////////////

pub mod epi4you;
pub mod epi4you_errors;

pub mod create_2me {
    pub mod create_from_cli_run;
    pub mod create_from_nf_artefact;
}

pub mod importer {
    pub mod import_from_2me;
}

pub mod nextflow {
    pub mod nextflow_artefact;
    pub mod nextflow_progress;
    pub mod nextflow_toolkit;
    pub mod nextflow_log_item;
    pub mod nextflow_analysis;
}



/* 

#[derive(Subcommand)]
enum Datatypes {

    /// the EPI2ME Desktop applications database of analysis runs
    Database {
        /// List database entries
        #[arg(short, long, action=ArgAction::SetTrue)]
        list: bool,

        /// define EPI2ME Desktop analysis
        #[arg(short = 'r', long)]
        runid: Option<String>,

        /// modify status field
        #[arg(short, long, )]
        status: Option<String>,

        /// drop database entries
        #[arg(short, long, action=ArgAction::SetTrue)]
        delete: bool,

        /// rename EPI2ME Desktop analysis
        #[arg(short = 'n', long)]
        rename: Option<String>,

        /// drop database entries
        #[arg(short = 'k', long, action=ArgAction::SetTrue)]
        housekeeping: bool,

        /// clone an existing EPI2ME Desktop analysis
        #[arg(short, long)]
        clone: Option<String>,
    },

    /// containers used by the EPI2ME software
    Docker {
        /// define EPI2ME Desktop analysis
        #[arg(num_args(0..), short, long)]
        workflow: Vec<String>,

        /// List project linked containers
        #[arg(short, long, action=ArgAction::SetTrue)]
        list: bool,

        /// pull project linked containers
        #[arg(short, long, action=ArgAction::SetTrue)]
        pull: bool,

        /// Export containers into archive
        #[arg(short, long)]
        twome: Option<String>,
    },

    /// EPI2ME workflow results
    EPI2ME {
        /// List analyses run using Desktop Client
        #[arg(short, long, action=ArgAction::SetTrue)]
        list: bool,

        /// bundle containers and workflow in .2me archive
        #[arg(short, long, action=ArgAction::SetTrue)]
        bundlewf: bool,

        /// Export EPI2ME Desktop analysis by ID
        #[arg(num_args(0..), short, long)]
        runid: Vec<String>,

        /// target twome archive file
        #[arg(short, long)]
        twome: Option<String>,

        /// force overwrite of exising twome archive
        #[arg(short, long, action=ArgAction::SetTrue)]
        force: bool,
    },



    /// interaction and bundling of CLI-based nextflow artifacts
    NextflowArtifact {
        /// List artifacts installed by CLI nextflow
        #[arg(short, long, action=ArgAction::SetTrue)]
        list: bool,

        /// workflows to pull and bundle through the nextflow CLI
        #[arg(num_args(0..), short, long)]
        workflow: Vec<String>,

        /// path to nextflow binary (if not obvious)
        #[arg(short, long, default_value = None)]
        nxf_bin: Option<String>,

        /// perform a nextflow pull update if workflow not downloaded
        #[arg(short, long, action=ArgAction::SetTrue)]
        pull: bool,

        /// target twome archive file
        #[arg(short, long)]
        twome: Option<String>,

        /// force overwrite of exising twome archive
        #[arg(short, long, action=ArgAction::SetTrue)]
        force: bool,

        /// bundle accompanying docker containers
        #[arg(short, long, action=ArgAction::SetTrue)]
        docker: bool,
    },


    /// cli exectured nextflow runs


    /// the core nextflow workflows used by the application
    Workflow {
        /// List project linked containers
        #[arg(short, long, action=ArgAction::SetTrue)]
        list: bool,

        /// specify a workflow
        #[arg(num_args(0..), short, long)]
        workflow: Vec<String>,

        /// target twome archive file
        #[arg(short, long)]
        twome: Option<String>,

        /// force overwrite of exising twome archive
        #[arg(short, long, action=ArgAction::SetTrue)]
        force: bool,

    },
}
*/


#[tokio::main]
async fn main() {

    let env = Env::default()
    .filter_or("MY_LOG_LEVEL", "debug")
    .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let tempdir = tempdir::get_tempdir();
    if tempdir.is_err() {
        eprintln!("error creating tempdir - aborting!");
        return;
    }
    let mut temp_dir = tempdir.ok().unwrap();
    temp_dir.keep();

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

    let xargs = app.try_get_matches();
    if xargs.is_ok() {
        let cmdkeys: Vec<&str> = subcmds.iter().map(|x| x.get_name()).collect();

        let status;

        if cmdkeys.contains(&create_from_cli_run::NEXTFLOW_RUN) && xargs.as_ref().ok().unwrap().subcommand_matches(create_from_cli_run::NEXTFLOW_RUN).is_some()  { 
            log::debug!("subcommand [{}] has been called", create_from_cli_run::NEXTFLOW_RUN);
            status = create_from_cli_run::process_clicapture_command(&xargs.ok().unwrap().subcommand_matches(create_from_cli_run::NEXTFLOW_RUN).unwrap(), &mut temp_dir);
        } else if cmdkeys.contains(&import_from_2me::IMPORT2ME) && xargs.as_ref().ok().unwrap().subcommand_matches(import_from_2me::IMPORT2ME).is_some()  { 
            log::debug!("subcommand [{}] has been called", import_from_2me::IMPORT2ME);
            status = import_from_2me::process_2me_import_command(&xargs.ok().unwrap().subcommand_matches(import_from_2me::IMPORT2ME).unwrap(), &mut temp_dir).await;
        } else {
            status = malformed_cli();
        }

    } else {
        println!("{:?}", xargs.as_ref().err().unwrap().print().ok().unwrap());
    }

    /* 

    match &cliargs.command {

        Some(Datatypes::Docker { workflow: project, list, pull, twome }) => {
            //let epi2me_opt = epi2me_db::find_db();
            // if epi2me_opt.is_some() {
                //let epi2me = epi2me_opt.unwrap();
                //docker::docker_agent(&tempdir.unwrap(), &epi2me, project, list, pull, twome).await;
                docker::docker_agent(&tempdir.unwrap(), project, list, pull, twome).await;
            //}
        },

        Some(Datatypes::Database { list, runid, status, delete, rename, housekeeping, clone }) => {
            let epi2me_opt = epi2me_db::find_db();
            if epi2me_opt.is_some() {
                let epi2me = epi2me_opt.unwrap();
                let df = app_db::load_db(&epi2me.epi2db_path);
                if df.is_ok() {
                    dbmanager(&epi2me.epi2db_path, &df.unwrap(), list, runid, status, delete, rename, housekeeping, clone);
                }
            }
        },

        Some(Datatypes::Workflow { list, workflow, twome, force }) => {
            workflow_manager(list, workflow, twome, force);
        },

        Some(Datatypes::NextflowArtifact { list, workflow, nxf_bin, pull, twome, force, docker }) => {
            nextflow::nextflow_artifact_manager(list, workflow, nxf_bin, pull, twome, force, docker);
        },

        Some(Datatypes::NextflowRun { list, nxf_bin, nxf_work, runid, twome, force }) => {
            nextflow::nextflow_run_manager(list, nxf_bin, nxf_work, runid, twome, force);
        },

        Some(Datatypes::EPI2ME { list, bundlewf, runid, twome, force }) => {
            let epi2me_opt = epi2me_db::find_db();
            if epi2me_opt.is_some() {
                let epi2me = epi2me_opt.unwrap();
                let df = app_db::load_db(&epi2me.epi2db_path);
                if df.is_ok() {
                    epi2me_manager(&epi2me, &df.unwrap(), list, runid, twome, force, bundlewf);
                }
            }
        },


        None => {}
    }

    */

}


fn malformed_cli() -> Result<(), Epi4youError> {
    return Err(Epi4youError::MalformedCLISetup);
}