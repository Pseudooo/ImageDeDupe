use clap_derive::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(
        short,
        long,
        required = false,
        default_value = "./",
        help = "The target directory to read images from"
    )]
    pub target: PathBuf,

    #[arg(
        short,
        long,
        required = false,
        default_value = "./output",
        help = "The output directory to write groupings too"
    )]
    pub output: PathBuf,

    #[arg(
        short,
        long,
        required = false,
        default_value = "8",
        help = "The hamming distance between two image hashes to be identified as duplicates"
    )]
    pub distance: i32,
}

impl CliArgs {
    pub fn validate(&self) -> Result<(), String> {
        if !self.target.exists() {
            return Err("Target path doesn't exist".to_string());
        }

        if !self.target.is_dir() {
            return Err("Target is not a directory, can't deduplicate a single file???".to_string());
        }

        if self.output.exists() && !self.output.is_dir() {
            return Err("Output path must be a directory".to_string());
        }

        Ok(())
    }
}
