use crate::constants;
use std::io::Write;
use std::os::unix::net::UnixStream;

pub fn try_communicate() -> anyhow::Result<bool> {
    match UnixStream::connect(constants::UNIX_SOCKET_PATH) {
        Ok(mut stream) => {
            stream.write_all("new_window".as_bytes())?;
            Ok(true)
        }
        Err(_) => {
            crate::application::new_backend();
            Ok(true)
        }
    }
}
