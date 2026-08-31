#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub enum SizeMode {
    Normal,
    Double,
}

#[derive(Debug, Clone)]
pub enum SkinNode {
    /// Literal text line, may contain {{placeholder}} tokens.
    Text(String),
    Align(Alignment),
    Bold(bool),
    Size(SizeMode),
    Underline(bool),
    Divider,
    Blank,
    /// Path to a value resolving to an image file path.
    Image(String),
    /// Path to a value resolving to QR content string.
    Qr(String),
    /// Two-column single line: name on the left, value on the right,
    /// joined by a dot leader filling the remaining paper width.
    Row {
        left: String,
        right: String,
    },
    Cut,
    If {
        condition: String,
        then_branch: Vec<SkinNode>,
        else_branch: Vec<SkinNode>,
    },
    Foreach {
        list: String,
        var: String,
        body: Vec<SkinNode>,
    },
    Match {
        value: String,
        cases: Vec<(String, Vec<SkinNode>)>,
    },
}

pub type Skin = Vec<SkinNode>;
