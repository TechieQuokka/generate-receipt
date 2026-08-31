use anyhow::{Context, Result};
use std::fs;

pub fn save(bytes: &[u8], path: &str) -> Result<()> {
    fs::write(path, bytes).with_context(|| format!("failed to write output file: {path}"))
}
