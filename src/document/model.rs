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
pub enum DocumentElement {
    Text {
        content: String,
        align: Alignment,
        bold: bool,
        size: SizeMode,
        underline: bool,
    },
    Divider,
    Blank,
    /// Resolved file path to an image to embed.
    Image(String),
    /// Resolved string content to encode as a QR code.
    Qr(String),
    /// Two-column single line: left label, right value, dot-leader filled
    /// between them to span the full printable width.
    Row {
        left: String,
        right: String,
        bold: bool,
    },
    Cut,
    /// Explicit alignment change. Emitted for every @align directive so the
    /// printer's justification state stays in sync even when it isn't
    /// immediately followed by a Text element (e.g. before a Divider, Image,
    /// or Qr element).
    Align(Alignment),
}

pub type ReceiptDocument = Vec<DocumentElement>;
