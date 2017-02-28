use std::{error, fmt, io};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use toml;

#[derive(Debug)]
/// Combined error type for configuration errors.
pub enum ConfigError {
  Io(io::Error),
  Parse(toml::de::Error),
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ConfigError::Io(ref err) => write!(f, "Couldn't read config: {}", err),
      ConfigError::Parse(ref err) => write!(f, "Couldn't parse config: {}", err),
    }
  }
}

impl error::Error for ConfigError {
  fn description(&self) -> &str {
    match *self {
      ConfigError::Io(ref err) => err.description(),
      ConfigError::Parse(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&error::Error> {
    match *self {
      ConfigError::Io(ref err) => Some(err),
      ConfigError::Parse(ref err) => Some(err),
    }
  }
}

impl From<io::Error> for ConfigError {
  fn from(err: io::Error) -> ConfigError {
    ConfigError::Io(err)
  }
}

impl From<toml::de::Error> for ConfigError {
  fn from(err: toml::de::Error) -> ConfigError {
    ConfigError::Parse(err)
  }
}

#[derive(Debug, Deserialize)]
/// Configuration relating to the network.
pub struct Network {
  pub ip: String,
  pub ip6: String,
  pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
/// Configuration relating to the logging subsystem,
pub struct Log {
  pub level: String,
  pub file: Option<String>,
  pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
/// Configuration relating to daemonization.
pub struct Daemon {
  pub pid_file: Option<String>,
  pub working_directory: Option<String>,
  pub user: Option<String>,
  pub user_id: Option<u32>,
  pub group: Option<String>,
  pub group_id: Option<u32>,
  pub umask: Option<u32>,
}

#[derive(Debug, Deserialize)]
/// General configuration superstructure.
pub struct Config {
  /// The constant date to reply with.
  pub date: toml::value::Datetime,
  /// Configuration relating to the network.
  pub network: Network,
  /// Configuration relating to the logging subsystem,
  pub log: Log,
  /// Configuration relating to daemonization.
  pub daemon: Option<Daemon>,
}

impl Config {
  /// Read configuration from a file.
  pub fn read<P: AsRef<Path> + fmt::Display>(filename: P) -> Result<Config, ConfigError> {
    let mut config_text = String::new();
    File::open(filename)?.read_to_string(&mut config_text)?;
    Ok(toml::from_str(config_text.as_ref())?)
  }
}
