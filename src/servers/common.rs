use std::io::{Error, ErrorKind};

/// Determines if an std::io::Error results from a broken connection
///
/// # Parameters
///
/// * `error`: error to examine
pub fn is_disconnect(error: &Error) -> bool {
    match error.kind() {
        ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => true,
        _ => false,
    }
}
