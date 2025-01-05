use clap::Parser;

/// Application arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The source directory to scan for PBOs
    pub source_dir: std::path::PathBuf,
}