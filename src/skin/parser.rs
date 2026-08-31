use super::lexer::tokenize;
use super::model::*;
use anyhow::{bail, Result};

pub fn parse(source: &str) -> Result<Skin> {
    let lines = tokenize(source);
    let mut cursor = 0usize;
    parse_block(&lines, &mut cursor, None)
}

/// Parses lines into nodes until a terminator line is hit (or EOF if None).
/// Terminators ending with ':' are matched by prefix (e.g. "@case:"),
/// others are matched exactly (e.g. "@endif").
/// The terminator line itself is NOT consumed; the caller decides what to do with it.
fn parse_block(
    lines: &[String],
    cursor: &mut usize,
    terminators: Option<&[&str]>,
) -> Result<Vec<SkinNode>> {
    let mut nodes = Vec::new();

    while *cursor < lines.len() {
        let line = lines[*cursor].as_str();

        if let Some(terms) = terminators {
            if is_terminator(line, terms) {
                return Ok(nodes);
            }
        }

        if let Some(rest) = line.strip_prefix("@align:") {
            let a = match rest {
                "left" => Alignment::Left,
                "center" => Alignment::Center,
                "right" => Alignment::Right,
                other => bail!("unknown alignment: {other}"),
            };
            nodes.push(SkinNode::Align(a));
            *cursor += 1;
        } else if let Some(rest) = line.strip_prefix("@bold:") {
            nodes.push(SkinNode::Bold(rest == "on"));
            *cursor += 1;
        } else if let Some(rest) = line.strip_prefix("@size:") {
            let s = match rest {
                "normal" => SizeMode::Normal,
                "double" | "double_width" | "double_height" => SizeMode::Double,
                other => bail!("unknown size: {other}"),
            };
            nodes.push(SkinNode::Size(s));
            *cursor += 1;
        } else if let Some(rest) = line.strip_prefix("@underline:") {
            nodes.push(SkinNode::Underline(rest == "on"));
            *cursor += 1;
        } else if line == "@divider" {
            nodes.push(SkinNode::Divider);
            *cursor += 1;
        } else if line == "@blank" {
            nodes.push(SkinNode::Blank);
            *cursor += 1;
        } else if let Some(rest) = line.strip_prefix("@image:") {
            nodes.push(SkinNode::Image(unwrap_placeholder(rest)));
            *cursor += 1;
        } else if let Some(rest) = line.strip_prefix("@qr:") {
            nodes.push(SkinNode::Qr(unwrap_placeholder(rest)));
            *cursor += 1;
        } else if let Some(rest) = line.strip_prefix("@row:") {
            let (left, right) = split_row(rest)?;
            nodes.push(SkinNode::Row { left, right });
            *cursor += 1;
        } else if line == "@cut" {
            nodes.push(SkinNode::Cut);
            *cursor += 1;
        } else if let Some(rest) = line.strip_prefix("@if:") {
            let condition = rest.to_string();
            *cursor += 1;
            let then_branch = parse_block(lines, cursor, Some(&["@else", "@endif"]))?;

            let mut else_branch = Vec::new();
            if *cursor < lines.len() && lines[*cursor] == "@else" {
                *cursor += 1;
                else_branch = parse_block(lines, cursor, Some(&["@endif"]))?;
            }

            if *cursor >= lines.len() || lines[*cursor] != "@endif" {
                bail!("expected @endif to close @if:{condition}");
            }
            *cursor += 1;

            nodes.push(SkinNode::If {
                condition,
                then_branch,
                else_branch,
            });
        } else if let Some(rest) = line.strip_prefix("@foreach:") {
            let (list, var) = split_foreach(rest)?;
            *cursor += 1;
            let body = parse_block(lines, cursor, Some(&["@endforeach"]))?;

            if *cursor >= lines.len() || lines[*cursor] != "@endforeach" {
                bail!("expected @endforeach to close @foreach:{rest}");
            }
            *cursor += 1;

            nodes.push(SkinNode::Foreach { list, var, body });
        } else if let Some(rest) = line.strip_prefix("@match:") {
            let value = rest.to_string();
            *cursor += 1;

            let mut cases = Vec::new();
            while *cursor < lines.len() && lines[*cursor].starts_with("@case:") {
                let case_value = lines[*cursor]
                    .strip_prefix("@case:")
                    .unwrap()
                    .trim()
                    .to_string();
                *cursor += 1;
                let case_body = parse_block(lines, cursor, Some(&["@case:", "@endmatch"]))?;
                cases.push((case_value, case_body));
            }

            if *cursor >= lines.len() || lines[*cursor] != "@endmatch" {
                bail!("expected @endmatch to close @match:{value}");
            }
            *cursor += 1;

            nodes.push(SkinNode::Match { value, cases });
        } else {
            nodes.push(SkinNode::Text(line.to_string()));
            *cursor += 1;
        }
    }

    Ok(nodes)
}

fn is_terminator(line: &str, terminators: &[&str]) -> bool {
    terminators.iter().any(|t| {
        if t.ends_with(':') {
            line.starts_with(t)
        } else {
            line == *t
        }
    })
}

fn unwrap_placeholder(s: &str) -> String {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("{{").and_then(|s| s.strip_suffix("}}")) {
        inner.trim().to_string()
    } else {
        s.to_string()
    }
}

fn split_foreach(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.split(" as ").collect();
    if parts.len() != 2 {
        bail!("invalid @foreach syntax: expected '<list> as <var>', got '{s}'");
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

fn split_row(s: &str) -> Result<(String, String)> {
    match s.split_once('|') {
        Some((l, r)) => Ok((l.trim().to_string(), r.trim().to_string())),
        None => bail!("invalid @row syntax: expected '<left>|<right>', got '{s}'"),
    }
}
