use std::path::PathBuf;

use app_db::dbmanager;
use clap::{Parser, Subcommand, ArgAction};
use docker::docker_agent;
use epi2me_db::epi2me_manager;
use manifest::load_manifest_from_tarball;
use workflow::workflow_manager;

mod epi2me_db;
mod json;
mod app_db;
mod nextflow;
mod bundle;
mod manifest;
mod provenance;
mod workflow;
mod epi2me_tar;
mod docker;
mod dataframe;
mod tempdir;


use crate::manifest::{is_manifest_honest, import_resolved_content};

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

    /// containers used by the EPI2ME software
    Docker {
        /// define EPI2ME Desktop analysis
        #[arg(short, long)]
        workflow: Option<String>,

        /// List project linked containers
        #[arg(short, long, action=ArgAction::SetTrue)]
        list: bool,

        /// pull project linked containers
        #[arg(short, long, action=ArgAction::SetTrue)]
        pull: bool,

        /// Export containers into archive
        #[arg(short, long)]
        export: Option<String>,
    },


    /// bioinformatics workflows
    Nextflow {
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
    }
}

#[tokio::main]
async fn main() {
    let cliargs = Args::parse();

    let epi2me_opt = epi2me_db::find_db();
    if epi2me_opt.is_some() {
        let epi2me = epi2me_opt.unwrap();
        let df = app_db::load_db(&epi2me.epi2db_path);
        if df.is_ok() {

            match &cliargs.command {

                Some(Datatypes::Docker { workflow: project, list, pull, export }) => {
                    docker_agent(&epi2me, project, list, pull, export).await;
                },

                Some(Datatypes::Database { list, runid, status, delete, rename, housekeeping, clone }) => {
                    dbmanager(&epi2me.epi2db_path, &df.unwrap(), list, runid, status, delete, rename, housekeeping, clone);
                },

                Some(Datatypes::Workflow { list, workflow, twome, force }) => {
                    workflow_manager(list, workflow, twome, force);
                },

                Some(Datatypes::Nextflow { list, nxf_bin, nxf_work, runid, twome }) => {
                    nextflow::nextflow_manager(list, nxf_bin, nxf_work, runid, twome);
                },

                Some(Datatypes::EPI2ME { list, bundlewf, runid, twome, force }) => {
                    epi2me_manager(&epi2me, &df.unwrap(), list, runid, twome, force, bundlewf);
                },

                Some(Datatypes::Import { twome, force}) => {

                    if twome.is_none() {
                        eprintln!("EPI2ME twome import requires a --twome <file> target to read");
                        return; 
                    } else {
                        let path = PathBuf::from(twome.as_ref().unwrap());
                        let manifest = load_manifest_from_tarball(&path);

                        if manifest.is_some() {

                            let honest = is_manifest_honest(&manifest.unwrap(), &path);
                            if honest.is_none() {
                                eprintln!("this epi4you archive is not trusted - exiting");
                                return;
                            } if honest.is_some() {
                                println!("importing something");
                                import_resolved_content(&honest.unwrap(), force);
                            }

                            

                        } else {
                            eprintln!("This archive may be malformed - cannot continue");
                        }
                    }
                    

                },

                None => {}
            }
        } else {
            println!("db issue?");
        }
    }

    
}
