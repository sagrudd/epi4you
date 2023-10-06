use clap::{Parser, Subcommand};
mod epi2me_db;
mod json;
mod app_db;

/// Trivial application to package EPI2ME workflows and analysis results
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Datatypes>,
}

#[derive(Subcommand)]
enum Datatypes {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },

    /// bioinformatics workflows
    Nextflow {

    },

    /// EPI2ME workflow results
    EPI2ME {
        /// List analyses run using Desktop Client
        #[arg(short, long)]
        list: bool,

        /// List nextflow runs from specified <path>
        #[arg(short, long)]
        nf_path: Option<String>,

        /// Export EPI2ME Desktop analysis by ID
        #[arg(short, long)]
        export: Option<String>,
    },
}

fn main() {
    let _args = Args::parse();
    let db_path = epi2me_db::find_db();
    if db_path.is_some() {
        let df = app_db::load_db(db_path.unwrap());
        if df.is_ok() {
            // can we print this?
            app_db::print_appdb(&df.unwrap());
        }
    }
}
