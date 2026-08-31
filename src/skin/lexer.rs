/// Splits skin source into significant lines: trims whitespace,
/// drops blank lines and comment lines (starting with '#').
pub fn tokenize(source: &str) -> Vec<String> {
    source
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}
