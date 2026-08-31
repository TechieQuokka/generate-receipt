use super::model::{Alignment, DocumentElement, ReceiptDocument, SizeMode};
use crate::skin::model::{
    Alignment as SkinAlignment, SizeMode as SkinSizeMode, Skin, SkinNode,
};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Evaluation context: root data (store/meta/items/computed/footer)
/// plus any loop-bound variables (e.g. "item", "child").
pub struct EvalContext {
    pub root: Value,
    pub vars: HashMap<String, Value>,
}

impl EvalContext {
    fn resolve(&self, path: &str) -> Option<Value> {
        let mut parts = path.split('.');
        let first = parts.next()?;

        let mut current = if let Some(v) = self.vars.get(first) {
            v.clone()
        } else {
            self.root.get(first)?.clone()
        };

        for p in parts {
            current = current.get(p)?.clone();
        }
        Some(current)
    }

    fn is_truthy(&self, path: &str) -> bool {
        match self.resolve(path) {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => b,
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
            Some(Value::Number(n)) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        }
    }

    fn as_string(&self, path: &str) -> String {
        match self.resolve(path) {
            Some(Value::String(s)) => s,
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            _ => String::new(),
        }
    }
}

/// Builds the CLI `--footer-text` entries into a JSON array of
/// {"kind": "text"|"qr", "content": "..."} objects, in the order given.
pub fn build_footer_value(entries: &[String]) -> Value {
    let items: Vec<Value> = entries
        .iter()
        .map(|e| {
            if let Some(rest) = e.strip_prefix("qr:") {
                json!({"kind": "qr", "content": rest})
            } else {
                json!({"kind": "text", "content": e})
            }
        })
        .collect();
    Value::Array(items)
}

struct Style {
    align: Alignment,
    bold: bool,
    size: SizeMode,
    underline: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            align: Alignment::Left,
            bold: false,
            size: SizeMode::Normal,
            underline: false,
        }
    }
}

pub fn build_document(skin: &Skin, ctx: &EvalContext) -> Result<ReceiptDocument> {
    let mut doc = Vec::new();
    let mut style = Style::default();
    walk(skin, ctx, &mut style, &mut doc)?;
    Ok(doc)
}

fn walk(
    nodes: &[SkinNode],
    ctx: &EvalContext,
    style: &mut Style,
    doc: &mut ReceiptDocument,
) -> Result<()> {
    for node in nodes {
        match node {
            SkinNode::Text(t) => {
                let content = interpolate(ctx, t);
                doc.push(DocumentElement::Text {
                    content,
                    align: clone_align(&style.align),
                    bold: style.bold,
                    size: clone_size(&style.size),
                    underline: style.underline,
                });
            }
            SkinNode::Align(a) => {
                style.align = convert_align(a);
                doc.push(DocumentElement::Align(clone_align(&style.align)));
            }
            SkinNode::Bold(b) => style.bold = *b,
            SkinNode::Size(s) => style.size = convert_size(s),
            SkinNode::Underline(u) => style.underline = *u,
            SkinNode::Divider => doc.push(DocumentElement::Divider),
            SkinNode::Blank => doc.push(DocumentElement::Blank),
            SkinNode::Image(path) => {
                let resolved = ctx.as_string(path);
                if !resolved.is_empty() {
                    doc.push(DocumentElement::Image(resolved));
                }
            }
            SkinNode::Qr(content) => {
                let resolved = ctx.as_string(content);
                if !resolved.is_empty() {
                    doc.push(DocumentElement::Qr(resolved));
                }
            }
            SkinNode::Row { left, right } => {
                doc.push(DocumentElement::Row {
                    left: interpolate(ctx, left),
                    right: interpolate(ctx, right),
                    bold: style.bold,
                });
            }
            SkinNode::Cut => doc.push(DocumentElement::Cut),
            SkinNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if ctx.is_truthy(condition) {
                    walk(then_branch, ctx, style, doc)?;
                } else {
                    walk(else_branch, ctx, style, doc)?;
                }
            }
            SkinNode::Foreach { list, var, body } => {
                if let Some(Value::Array(items)) = ctx.resolve(list) {
                    for item in items {
                        let child_ctx = EvalContext {
                            root: ctx.root.clone(),
                            vars: {
                                let mut v = ctx.vars.clone();
                                v.insert(var.clone(), item);
                                v
                            },
                        };
                        walk(body, &child_ctx, style, doc)?;
                    }
                }
            }
            SkinNode::Match { value, cases } => {
                let v = ctx.as_string(value);
                if let Some((_, body)) = cases.iter().find(|(cv, _)| cv == &v) {
                    walk(body, ctx, style, doc)?;
                }
            }
        }
    }
    Ok(())
}

/// Replaces all {{path}} occurrences in a text line with resolved string values.
fn interpolate(ctx: &EvalContext, template: &str) -> String {
    let mut result = String::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("}}") {
            let path = after[..end].trim();
            result.push_str(&ctx.as_string(path));
            rest = &after[end + 2..];
        } else {
            result.push_str("{{");
            rest = after;
            break;
        }
    }
    result.push_str(rest);
    result
}

fn convert_align(a: &SkinAlignment) -> Alignment {
    match a {
        SkinAlignment::Left => Alignment::Left,
        SkinAlignment::Center => Alignment::Center,
        SkinAlignment::Right => Alignment::Right,
    }
}

fn convert_size(s: &SkinSizeMode) -> SizeMode {
    match s {
        SkinSizeMode::Normal => SizeMode::Normal,
        SkinSizeMode::Double => SizeMode::Double,
    }
}

fn clone_align(a: &Alignment) -> Alignment {
    match a {
        Alignment::Left => Alignment::Left,
        Alignment::Center => Alignment::Center,
        Alignment::Right => Alignment::Right,
    }
}

fn clone_size(s: &SizeMode) -> SizeMode {
    match s {
        SizeMode::Normal => SizeMode::Normal,
        SizeMode::Double => SizeMode::Double,
    }
}
