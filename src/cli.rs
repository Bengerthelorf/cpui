use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Source file or directory
    #[arg(value_name = "SOURCE")]
    pub source: PathBuf,

    /// Destination file or directory
    #[arg(value_name = "DESTINATION")]
    pub destination: PathBuf,

    /// Recursively copy directories
    #[arg(short, long)]
    pub recursive: bool,

    /// Hidden test mode with artificial delay (format: test_mode=<type>:<value>)
    /// Example: test_mode=delay:10
    #[arg(long, hide = true)]
    pub test_mode: Option<String>,
}

#[derive(Debug, Clone)] // 添加 Clone trait
pub enum TestMode {
    Delay(u64),          // Milliseconds delay
    SpeedLimit(u64),     // Bytes per second
    None,
}

impl Cli {
    pub fn get_test_mode(&self) -> TestMode {
        if let Some(test_mode) = self.test_mode.as_ref() {
            let parts: Vec<&str> = test_mode.split(':').collect();
            if parts.len() == 2 {
                match (parts[0], parts[1].parse::<u64>()) {
                    ("delay", Ok(ms)) => TestMode::Delay(ms),
                    ("speed_limit", Ok(bps)) => TestMode::SpeedLimit(bps),
                    _ => TestMode::None,
                }
            } else {
                TestMode::None
            }
        } else {
            TestMode::None
        }
    }
}

pub fn parse_args() -> Cli {
    Cli::parse()
}