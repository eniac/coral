use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(group(
            ArgGroup::new("mode")
                .required(true)
                .args(&["commit", "prove", "verify", "e2e"]),
        ))]
pub struct Options {
    #[arg(long, default_value_t = false)]
    pub commit: bool,
    #[arg(long, default_value_t = false)]
    pub prove: bool,
    #[arg(long, default_value_t = false)]
    pub verify: bool,
    #[arg(long, default_value_t = false)]
    pub e2e: bool,
    #[arg(long, value_name = "FILE", help = "Optional name for .cmt file")]
    pub cmt_name: Option<String>,
    #[arg(long, value_name = "FILE", help = "Optional name for .proof file")]
    pub proof_name: Option<String>,
    #[arg(short = 'd', long, value_name = "FILE")]
    #[arg(short = 'd', long, value_name = "FILE")]
    pub doc: String,
    #[arg(short = 'g', long, value_name = "FILE")]
    pub grammar: String,
    #[arg(
        short = 'm',
        long,
        value_name = "FILE",
        help = "Metrics and other output information"
    )]
    pub metrics: Option<PathBuf>,
    #[arg(
        short = 'b',
        long = "batch-size",
        value_name = "USIZE",
        help = "Batch size (override auto select)",
        default_value_t = 1, // auto select
    )]
    pub batch_size: usize,
}
