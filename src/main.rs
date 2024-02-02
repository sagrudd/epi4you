use app_db::dbmanager;
use clap::{Parser, Subcommand, ArgAction};
use epi2me_db::epi2me_manager;
use workflow::workflow_manager;

mod app_db;
mod bundle;
mod epi2me_db;
mod epi2me_tar;
mod docker;
mod dataframe;
mod importer;
mod json;
mod manifest;
mod nextflow;
mod provenance;
mod tempdir;
mod workflow;
mod settings;

mod xdocker;
mod ximporter;
mod xmanifest;
mod xnf_parser;
mod xworkflows;

/// Trivial application to package EPI2ME workflows and analysis results
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Datatypes>,

}

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

    /// import .2me format tar archive
    Import {
        /// filepath to the .2me file to import
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
    NextflowRun {
        /// List analyses run using Desktop Client
        #[arg(short, long, action=ArgAction::SetTrue)]
        list: bool,

        /// path to nextflow binary (if not obvious)
        #[arg(short, long, default_value = None)]
        nxf_bin: Option<String>,

        /// path to nextflow work folder
        #[arg(short = 'w', long, default_value = None)]
        nxf_work: Option<String>,

        /// Export EPI2ME analysis by nun_name
        #[arg(short, long)]
        runid: Option<String>,

        /// target twome archive file
        #[arg(short, long)]
        twome: Option<String>,

        /// force overwrite of exising twome archive
        #[arg(short, long, action=ArgAction::SetTrue)]
        force: bool,
    },

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

#[tokio::main]
async fn main() {
    let cliargs = Args::parse();

    let tempdir = tempdir::get_tempdir();
    if tempdir.is_none() {
        eprintln!("error creating tempdir - aborting!");
        return;
    }

    match &cliargs.command {

        Some(Datatypes::Docker { workflow: project, list, pull, twome }) => {
            //let epi2me_opt = epi2me_db::find_db();
            // if epi2me_opt.is_some() {
                //let epi2me = epi2me_opt.unwrap();
                //docker::docker_agent(&tempdir.unwrap(), &epi2me, project, list, pull, twome).await;
                xdocker::docker_agent(&tempdir.unwrap(), project, list, pull, twome).await;
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

        Some(Datatypes::Import { twome, force}) => {
            //import_manager(twome, force).await;
            ximporter::import_coordinator(&tempdir.unwrap().path, twome, force).await;
        },

        None => {}
    }

}
