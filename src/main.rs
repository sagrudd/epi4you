use clap::{Parser, Subcommand};

/// Trivial application to package EPI2ME workflows and analysis results
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Datatypes>,

    /// Name of the person to greet
    #[arg(short, long)]
    name: Option<String>,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
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
    BfxFlows {

    },

    /// EPI2ME analysis results
    EpiRes {

    },
}

fn main() {
    let _args = Args::parse();
    println!("Hello, world!");
}
