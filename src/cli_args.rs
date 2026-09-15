use clap_derive::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short, long, required = false, default_value = "./")]
    pub target: PathBuf,

    #[arg(short, long, required = false, default_value = "./output")]
    pub output: PathBuf,
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
