use clap::Parser;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum TaxMode {
    /// Item prices already include tax; just show the breakdown.
    Inclusive,
    /// Item prices are pre-tax; tax is calculated and added on top.
    Exclusive,
}

#[derive(Parser, Debug)]
#[command(name = "generate-receipt")]
#[command(about = "Generate ESC/POS receipts from data + skin templates")]
pub struct Cli {
    /// Path to the receipt data JSON file
    #[arg(long)]
    pub data: String,

    /// Path to the skin (template) file
    #[arg(long)]
    pub skin: String,

    /// Tax rate in percent (e.g. 10 for 10%)
    #[arg(long)]
    pub tax: Option<f64>,

    /// Tax calculation mode
    #[arg(long, value_enum, default_value_t = TaxMode::Exclusive)]
    pub tax_mode: TaxMode,

    /// Show sum of item prices (pre-tax, pre-discount)
    #[arg(long)]
    pub sum: bool,

    /// Show final total (tax, discount, service charge applied)
    #[arg(long)]
    pub total: bool,

    /// Show average price per item
    #[arg(long)]
    pub average: bool,

    /// Overall discount, e.g. "10%" or "1000" (fixed amount)
    #[arg(long)]
    pub discount: Option<String>,

    /// Service charge in percent
    #[arg(long)]
    pub service_charge: Option<f64>,

    /// Currency code (e.g. KRW, USD)
    #[arg(long, default_value = "KRW")]
    pub currency: String,

    /// Show total item count
    #[arg(long)]
    pub item_count: bool,

    /// Footer lines. Prefix with "qr:" to render as a QR code instead of text.
    /// Can be passed multiple times; rendered in order at the bottom.
    #[arg(long = "footer-text")]
    pub footer_text: Vec<String>,

    /// Output .bin file path (if not set, only sends to emulator)
    #[arg(long)]
    pub out: Option<String>,

    /// Send to escpresso emulator (localhost:9100)
    #[arg(long)]
    pub send: bool,

    /// Characters per line (paper width). 32 for 58mm, 48 for 80mm.
    #[arg(long, default_value_t = 48)]
    pub cpl: usize,

    /// Left/right margin in characters (breathing room from paper edges).
    #[arg(long, default_value_t = 2)]
    pub margin: usize,
}
