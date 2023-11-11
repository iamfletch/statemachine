//! Arguments/Config all in one place
//!   - clap for arguments
//!   - serde_yaml for config
//!   - parses config manually

use std::path::PathBuf;
use lazy_static::lazy_static;
use clap::Parser;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use serde_yaml::from_reader;
use serde_stuff::{deserialize_level_filter, serialize_level_filter};

mod serde_stuff;

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
/// global configuration also used for the config file format
pub struct Configuration {
    #[serde(deserialize_with="deserialize_level_filter", serialize_with="serialize_level_filter")]
    pub log_level: LevelFilter,
    pub port: u16
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            log_level: LevelFilter::Info,
            port: 8081
        }
    }
}

/// args parser
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(short, long="config")]
    config_file: Option<PathBuf>
}

impl Configuration {
    /// parses the configuration
    fn new() -> Self {
        // Load arguments to see if --config is passed in
        let args = Args::parse();

        // find the config file
        let config_file = if let Some(config_file) = args.config_file {
            config_file
        } else {
            // todo OS stuff here
            PathBuf::from("config.yml")
        };

        let mut config: Configuration = Configuration::from_file(config_file).unwrap();

        config.log_level = if args.debug {
            LevelFilter::Trace
        } else if args.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        };

        config
    }

    fn from_file(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>>{

        let f = std::fs::File::open(path)?;
        Ok(from_reader(f)?)
    }
}

lazy_static! {
    /// static and available to every module
    pub static ref CONFIG: Configuration = Configuration::new();
}