use crate::cli::{Cli, TaxMode};
use crate::data::model::ReceiptData;
use serde_json::{json, Value};

pub fn compute(data: &ReceiptData, cli: &Cli) -> Value {
    let sum: f64 = data
        .items
        .iter()
        .map(|i| i.price.unwrap_or(0.0) * i.quantity.unwrap_or(1) as f64)
        .sum();

    let discount_amount = match &cli.discount {
        Some(d) => {
            if let Some(pct) = d.strip_suffix('%') {
                pct.trim().parse::<f64>().unwrap_or(0.0) / 100.0 * sum
            } else {
                d.trim().parse::<f64>().unwrap_or(0.0)
            }
        }
        None => 0.0,
    };

    let base_after_discount = sum - discount_amount;

    let (tax_amount, tax_rate) = match cli.tax {
        Some(rate) => {
            let amount = match cli.tax_mode {
                TaxMode::Exclusive => base_after_discount * rate / 100.0,
                // Inclusive: back-calculate the tax portion already inside the price.
                TaxMode::Inclusive => {
                    base_after_discount - (base_after_discount / (1.0 + rate / 100.0))
                }
            };
            (Some(amount), Some(rate))
        }
        None => (None, None),
    };

    let service_charge_amount = cli
        .service_charge
        .map(|rate| base_after_discount * rate / 100.0);

    let total = match cli.tax_mode {
        TaxMode::Exclusive => {
            base_after_discount + tax_amount.unwrap_or(0.0) + service_charge_amount.unwrap_or(0.0)
        }
        TaxMode::Inclusive => base_after_discount + service_charge_amount.unwrap_or(0.0),
    };

    let average = if !data.items.is_empty() {
        Some(sum / data.items.len() as f64)
    } else {
        None
    };

    json!({
        "sum": if cli.sum { Some(round2(sum)) } else { None },
        "tax": tax_amount.map(round2),
        "tax_rate": tax_rate,
        "tax_mode_inclusive": matches!(cli.tax_mode, TaxMode::Inclusive),
        "discount": if discount_amount > 0.0 { Some(round2(discount_amount)) } else { None },
        "service_charge": service_charge_amount.map(round2),
        "average": if cli.average { average.map(round2) } else { None },
        "item_count": if cli.item_count { Some(data.items.len()) } else { None },
        "total": if cli.total { Some(round2(total)) } else { None },
        "currency": cli.currency,
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
