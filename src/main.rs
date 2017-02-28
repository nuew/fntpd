#[macro_use]
extern crate log;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate daemonize;
extern crate fern;
extern crate libc;
extern crate toml;

use std::str::FromStr;

#[macro_use]
mod macros;

mod config;
mod daesock;
mod ntp;

const DEFAULT_CONFIG: &'static str = "config.toml";

/// Initialize Logging Subsystem
fn logging(cfg: config::Log) -> Result<(), fern::InitError> {
  let mut output = vec![fern::OutputConfig::stderr()];

  // If specified, log to a file
  if let Some(ref filename) = cfg.file {
    output.push(fern::OutputConfig::file(filename));
  }

  fern::init_global_logger(fern::DispatchConfig {
                             format: Box::new(|msg, level, _location| {
                               format!("{} [{}] {}", level, chrono::Local::now().to_rfc3339(), msg)
                             }),
                             output: output,
                             level: log::LogLevelFilter::from_str(cfg.level.as_ref())
                               .unwrap_or_else(|_| {
        println!("That isn't a valid loglevel. Valid loglevels:{}{}{}{}{}{}",
                 "\n\tOFF",
                 "\n\tERROR",
                 "\n\tWARN",
                 "\n\tINFO",
                 "\n\tDEBUG",
                 "\n\tTRACE");
        std::process::exit(1);
      }),
                           },
                           log::LogLevelFilter::Trace)
}

fn main() {
  // Apply configuration
  let config_file = DEFAULT_CONFIG;
  let cfg = config::Config::read(config_file).unwrap_or_else(|err| {
    println!("{}", err);
    std::process::exit(1);
  });

  // Init logging
  logging(cfg.log).unwrap();

  // daemonize if supported and enabled
  // we get the lock on udp port 123 here, while we're still root
  let socket = if let Some(daemon) = cfg.daemon {
    debug!("Daemonizing");
    daesock::daemonize(daemon, cfg.network).unwrap_or_else(|err| fatal!("{}", err))
  } else {
    if cfg!(unix) && unsafe { libc::geteuid() } == 0 {
      warn!("Running as root without daemonization. This is a bad idea!");
      warn!("Enable daemonization in the configuration by adding a [daemon] section.");
    }
    daesock::get_socket(&cfg.network).unwrap_or_else(|err| fatal!("Couldn't bind to port: {}", err))
  };

  trace!("Bound to {}",
         socket.local_addr().unwrap_or_else(|err| fatal!("{}", err)));
  info!("Now listening for clients...");
  loop {
    ntp::inner_loop(&socket).unwrap()
  }
}
