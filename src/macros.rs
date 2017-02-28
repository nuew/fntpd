/// Logs a message at the error level than panics with a message of
/// "fatal error".
///
/// Logging at this level is disabled if the `max_level_off` feature is
/// present.
#[macro_export]
macro_rules! fatal {
  (target: $target:expr, $($arg:tt)*) => ({
    log!(target: $target, ::log::LogLevel::Error, $($arg)*);
    panic!("fatal error");
  });
  ($($arg:tt)*) => ({
    log!(::log::LogLevel::Error, $($arg)*);
    panic!("fatal error");
  });
}
