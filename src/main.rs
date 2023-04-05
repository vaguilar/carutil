use anyhow::{Result, Context};

use car_reader_lib::AssetCatalog;
use clap::{Parser, command, arg, CommandFactory};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// dumps a JSON file describing the contents of the inputfile 
   #[arg(short = 'I', long, value_name = "inputfile")]
   info: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(car_path) = args.info {
        let asset_catalog = AssetCatalog::try_from(car_path.as_ref())?;
        let header = serde_json::to_value(asset_catalog.header)?;
        let mut values = asset_catalog.assets
            .iter()
            .map(|n| serde_json::to_value(n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut result: Vec<serde_json::Value> = vec![header];
        result.append(&mut values);

        let j = serde_json::to_string_pretty(&result)?;
        println!("{}", j);
    } else {
        return Args::command().print_help().context("no args?");
    }
    Ok(())
}
