use std::path::PathBuf;

use clap::{Parser, Subcommand, ArgAction};

mod epi2me_db;
mod json;
mod app_db;
mod nextflow;
mod bundle;
mod manifest;
mod provenance;
mod workflow;


/// Trivial application to package EPI2ME workflows and analysis results
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Datatypes>,

}

#[derive(Subcommand)]
enum Datatypes {

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
    import {
        /// filepath to the .2me file to import
        #[arg(short, long)]
        twome_path: Option<String>,

        /// dryrun - validate and log import tasks without writing
        #[arg(short, long, action=ArgAction::SetTrue)]
        dryrun: bool,
    }
}

fn main() {
    let cliargs = Args::parse();

    let epi2me_opt = epi2me_db::find_db();
    if epi2me_opt.is_some() {
        let epi2me = epi2me_opt.unwrap();
        let df = app_db::load_db(epi2me.epi2db_path);
        if df.is_ok() {

            match &cliargs.command {

                Some(Datatypes::Nextflow { list, nxf_bin, nxf_work, runid }) => {
                    let localruns = nextflow::parse_nextflow_folder(nxf_work.clone(), nxf_bin.clone());
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
                                &runid.as_ref().unwrap().to_string(), 
                                df.as_ref().unwrap(), &epi2me.epi2wf_dir,
                            )
                        }

                        // if we are here we have a destination and a unique runid - let's package something ...
                        let src = Some(app_db::get_qualified_analysis_path(&runid.as_ref().unwrap().to_string(), df.as_ref().unwrap()));
                        let dest = Some(PathBuf::from(twome.as_ref().unwrap()));
                        
                        bundle::export_desktop_run(src, dest, bundle_workflow);
                    }
                },

                Some(Datatypes::import { twome_path , dryrun}) => {
                    
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
                },

                None => {}
            }
        } else {
            println!("db issue?");
        }
    }

    
}
