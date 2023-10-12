use std::path::PathBuf;

use clap::{Parser, Subcommand, ArgAction};

mod epi2me_db;
mod json;
mod app_db;
mod nextflow;
mod bundle;
mod manifest;
mod provenance;


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
        /// path to nextflow binary (if not obvious)
        #[arg(short, long, default_value = None)]
        nxf_bin: Option<String>,

        /// path to nextflow work folder
        #[arg(short = 'w', long, default_value = None)]
        nxf_work: Option<String>,
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

    let db_path = epi2me_db::find_db();
    if db_path.is_some() {
        let df = app_db::load_db(db_path.unwrap());
        if df.is_ok() {

            match &cliargs.command {

                Some(Datatypes::Nextflow { nxf_bin, nxf_work }) => {
                    nextflow::parse_nextflow_folder(nxf_work.clone(), nxf_bin.clone());
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
                            if !app_db::validate_db_entry(runid.as_ref().unwrap().to_string(), df.as_ref().unwrap()) {
                                println!("Unable to resolve specified EPI2ME analysis [{}] - check name", runid.as_ref().unwrap());
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
                    }
                },

                Some(Datatypes::import { twome_path , dryrun}) => {
                    
                },

                None => {}
            }
        }
    }

    
}
