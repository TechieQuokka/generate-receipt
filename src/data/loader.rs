use super::model::ReceiptData;
use anyhow::{Context, Result};
use std::fs;

pub fn load(path: &str) -> Result<ReceiptData> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read data file: {}", path))?;
    let data: ReceiptData = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse JSON data file: {}", path))?;
    Ok(data)
}
