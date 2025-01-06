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

    /// Preserve file attributes (mode, ownership, timestamps)
    #[arg(long)]
    pub preserve: bool,

    /// Exclude files/directories that match these patterns
    #[arg(long, value_name = "PATTERN", value_delimiter = ',')]
    pub exclude: Option<Vec<String>>,

    /// Hidden test mode with artificial delay (format: test_mode=<type>:<value>)
    /// Example: test_mode=delay:10
    #[arg(long, hide = true)]
    pub test_mode: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TestMode {
    Delay(u64),      // Milliseconds delay
    SpeedLimit(u64), // Bytes per second
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

    pub fn should_exclude(&self, path: &str) -> bool {
        if let Some(patterns) = &self.exclude {
            patterns.iter().any(|pattern| path.contains(pattern))
        } else {
            false
        }
    }
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
