use std::path::{PathBuf, Path};

use app_db::dbmanager;
use clap::{Parser, Subcommand, ArgAction};
use docker::docker_agent;
use manifest::load_manifest_from_tarball;
use path_clean::PathClean;

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

use std::env;

/// Trivial application to package EPI2ME workflows and analysis results
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Datatypes>,

}

#[derive(Subcommand)]
enum Datatypes {

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
        #[arg(short, long)]
        runid: Option<String>,

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

        /// dryrun - validate and log import tasks without writing
        #[arg(short, long, action=ArgAction::SetTrue)]
        dryrun: bool,
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

                Some(Datatypes::Nextflow { list, nxf_bin, nxf_work, runid }) => {

                    let mut nxf_workdir = nxf_work.clone();
                    if nxf_workdir.is_none() {
                        nxf_workdir = Some(env::current_dir().unwrap().to_string_lossy().into_owned());
                        println!("Setting nextflow workdir to cwd [{:?}]", nxf_workdir.clone().unwrap());
                    }

                    let localruns = nextflow::parse_nextflow_folder(nxf_workdir.clone(), nxf_bin.clone());
                    if localruns.is_none() {
                        println!("No local nextflow run folders found at specified path");
                        return;
                    }

                    if *list {
                        nextflow::print_nxf_log(&localruns.unwrap());
                        // todo - how do we print out dataframe with a more considered number of columns?
                    } else {
                        if runid.is_none() {
                            println!("EPI2ME analysis twome archiving requires a --runid identifier (run_name)");
                            return;
                        } else {
                            if !nextflow::validate_db_entry(runid.as_ref().unwrap().to_string(), localruns.as_ref().unwrap()) {
                                println!("Unable to resolve specified EPI2ME analysis [{}] - check name", runid.as_ref().unwrap());
                                return;
                            }
                        }
                    }
                },

                Some(Datatypes::EPI2ME { list, bundlewf, runid, twome, force }) => {
                    println!("epi2me.list == {}",*list);
                    if *list {
                        app_db::print_appdb(&df.unwrap());
                    } else {
                        if runid.is_none() {
                            println!("EPI2ME analysis twome archiving requires a --runid identifier (name or id)");
                            return;
                        } else {
                            if !app_db::validate_db_entry(&runid.as_ref().unwrap().to_string(), df.as_ref().unwrap()) {
                                return;
                            }
                        }

                        let runid_str = &runid.as_ref().unwrap().to_string();
                        let polardb = df.as_ref().unwrap();

                        if twome.is_none() {
                            println!("EPI2ME twome archiving requires a --twome <file> target to writing to");
                            return; 
                        } else {
                            let pb = PathBuf::from(twome.as_ref().unwrap());
                            if pb.exists() {
                                if pb.is_file() && !force {
                                    println!("twome file specified already exists - either --force or use different name");
                                    return;
                                } else if pb.is_dir() {
                                    println!("twome file is a directory - file is required");
                                    return;
                                } 
                            }    
                        }

                        let mut bundle_workflow: Option<PathBuf> = None;
                        if bundlewf == &true {
                            // ensure that a workflow for bundling is intact ...
                            bundle_workflow = app_db::validate_qualified_analysis_workflow(
                                &runid_str.to_string(), 
                                polardb, &epi2me.epi2wf_dir,
                            )
                        }

                        // if we are here we have a destination and a unique runid - let's sanity check the destination PATH
                        // there is some broken logic as described in https://github.com/sagrudd/epi4you/issues/1
                        let path = Path::new(twome.as_ref().unwrap());
                        let mut absolute_path;
                        if path.is_absolute() {
                            absolute_path = path.to_path_buf();
                        } else {
                            absolute_path = env::current_dir().unwrap().join(path);
                        }
                        absolute_path = absolute_path.clean();
                        println!("tar .2me archive to be written to [{:?}]", absolute_path);

                        // we have a destination and a unique runid - let's package something ...
                        bundle::export_desktop_run(&runid_str, polardb, Some(absolute_path), bundle_workflow);
                    }
                },

                Some(Datatypes::Import { twome: _ , dryrun: _}) => {
                    
                    let manifest = load_manifest_from_tarball();
                    if manifest.is_some() {

                        println!("importing something");

                    // validate that the twome file is signed and contains a manifest

                    // create temporary (auto delete) folder to unpack twome archive into

                    // for each of the workflows provided within the twome archive

                        // is the archive trusted

                        // does the corresponding workflow already exist on the system

                            // if it does is it an ==offline== existing installation that is older than the twome

                            // has force been specified

                        // deploy package 

                        // are there linked docker containers?

                            // is docker reachable through API calls?

                    // cleanup any residual temp folder content


                    }

                },

                None => {}
            }
        } else {
            println!("db issue?");
        }
    }

    
}
