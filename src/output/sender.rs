use anyhow::{Context, Result};
use std::io::Write;
use std::net::TcpStream;

pub fn send(bytes: &[u8]) -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9100")
        .context("failed to connect to escpresso emulator on localhost:9100 (is it running?)")?;
    stream.write_all(bytes)?;
    Ok(())
}
