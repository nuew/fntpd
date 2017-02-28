use daemonize::{Daemonize, DaemonizeError};
use std::{error, fmt, io};
use std::net::UdpSocket;
use super::config::{Daemon, Network};

#[derive(Debug)]
/// Combined error type for daemonization and socket errors
pub enum DaeSockError {
  Io(io::Error),
  Daemonize(DaemonizeError),
}

impl fmt::Display for DaeSockError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      DaeSockError::Io(ref err) => write!(f, "Couldn't bind to port: {}", err),
      DaeSockError::Daemonize(ref err) => write!(f, "Couldn't daemonize: {}", err),
    }
  }
}

impl error::Error for DaeSockError {
  fn description(&self) -> &str {
    match *self {
      DaeSockError::Io(ref err) => err.description(),
      DaeSockError::Daemonize(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&error::Error> {
    match *self {
      DaeSockError::Io(ref err) => Some(err),
      DaeSockError::Daemonize(ref err) => Some(err),
    }
  }
}

impl From<io::Error> for DaeSockError {
  fn from(err: io::Error) -> DaeSockError {
    DaeSockError::Io(err)
  }
}

impl From<DaemonizeError> for DaeSockError {
  fn from(err: DaemonizeError) -> DaeSockError {
    DaeSockError::Daemonize(err)
  }
}

pub fn get_socket(network_cfg: &Network) -> io::Result<UdpSocket> {
  let ref ip = network_cfg.ip;
  let port = network_cfg.port.unwrap_or(super::ntp::PORT);

  info!("Binding to {}:{}.", ip, port);
  UdpSocket::bind((ip.as_ref(), port))
}

#[cfg(unix)]
/// Daemonize and run `get_socket()`.
pub fn daemonize(daemon_cfg: Daemon, network_cfg: Network) -> Result<UdpSocket, DaeSockError> {
  let mut daemonize = Daemonize::new().privileged_action(move || get_socket(&network_cfg));

  if let Some(pid_file) = daemon_cfg.pid_file {
    debug!("Setting pid file as {}", pid_file);
    daemonize = daemonize.pid_file(pid_file).chown_pid_file(true);
  }

  if let Some(working_directory) = daemon_cfg.working_directory {
    debug!("Setting cwd as {}", working_directory);
    daemonize = daemonize.working_directory(working_directory);
  }

  if let Some(user) = daemon_cfg.user {
    debug!("Dropping user to {}", user);
    daemonize = daemonize.user(user.as_ref());
  } else if let Some(user) = daemon_cfg.user_id {
    debug!("Dropping to user #{}", user);
    daemonize = daemonize.user(user);
  } else {
    debug!("Dropping user to nobody");
    daemonize = daemonize.user("nobody");
  }

  if let Some(group) = daemon_cfg.group {
    debug!("Dropping group to {}", group);
    daemonize = daemonize.group(group.as_ref());
  } else if let Some(group) = daemon_cfg.group_id {
    debug!("Dropping to group #{}", group);
    daemonize = daemonize.group(group);
  } else {
    debug!("Dropping group to nobody");
    daemonize = daemonize.group("nobody");
  }

  if let Some(umask) = daemon_cfg.umask {
    debug!("Setting umask to {:o}", umask);
    daemonize = daemonize.umask(umask);
  }

  debug!("Preforming daemonization...");
  Ok(daemonize.start()??)
}

#[cfg(not(unix))]
/// An alias for `get_socket()`.
pub fn daemonize(_: &Daemon, network_cfg: &Network) -> Result<UdpSocket, DaeSockError> {
  warn!("Not daemonizing, as platform doesn't support it!");
  Ok(get_socket(network_cfg)?)
}
