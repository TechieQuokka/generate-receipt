use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiptData {
    pub store: StoreInfo,
    #[serde(default)]
    pub meta: ReceiptMeta,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreInfo {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub business_number: Option<String>,
    pub owner_name: Option<String>,
    pub website: Option<String>,
    pub logo_text: Option<String>,
    /// Path to a logo image file, resolved relative to the data file's directory.
    pub logo_path: Option<String>,
    /// Content string to encode as a QR code (e.g. store website or menu link).
    pub qr_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ReceiptMeta {
    pub receipt_number: Option<String>,
    pub timestamp: Option<String>,
    pub cashier: Option<String>,
    pub order_type: Option<String>,
    pub table_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    #[default]
    Normal,
    Bundle,
    Discount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: Option<String>,
    pub name: String,
    pub price: Option<f64>,
    pub quantity: Option<u32>,
    #[serde(default)]
    pub item_type: ItemType,
    #[serde(default)]
    pub emphasis: bool,
    #[serde(default)]
    pub strikethrough: bool,
    pub note: Option<String>,
    /// Recursive children: bundle members, item options, nested sub-items, etc.
    pub children: Option<Vec<Item>>,
}
