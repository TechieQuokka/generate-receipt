mod cli;
mod compute;
mod data;
mod document;
mod output;
mod render;
mod skin;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    let receipt_data = data::loader::load(&args.data)?;
    let skin_source = std::fs::read_to_string(&args.skin)?;
    let skin_ast = skin::parser::parse(&skin_source)?;

    let computed = compute::aggregates::compute(&receipt_data, &args);
    let footer = document::build::build_footer_value(&args.footer_text);

    let mut root = serde_json::to_value(&receipt_data)?;
    root["computed"] = computed;
    root["footer"] = footer;

    let ctx = document::build::EvalContext {
        root,
        vars: Default::default(),
    };

    let doc = document::build::build_document(&skin_ast, &ctx)?;
    let bytes = render::escpos::render(&doc, args.cpl, args.margin)?;

    if let Some(out_path) = &args.out {
        output::file::save(&bytes, out_path)?;
        println!("Saved {} bytes to {}", bytes.len(), out_path);
    }

    if args.send {
        output::sender::send(&bytes)?;
        println!("Sent {} bytes to escpresso (localhost:9100)", bytes.len());
    }

    if args.out.is_none() && !args.send {
        println!("Generated {} bytes (use --out <path> or --send to output)", bytes.len());
    }

    Ok(())
}
