use std::fs;
use std::io::Cursor;

use anyhow::Context;
use anyhow::Result;

use binrw::BinRead;
use car_reader_lib::bom::BOMHeader;
use car_reader_lib::AssetCatalog;
use clap::arg;
use clap::command;
use clap::CommandFactory;
use clap::Parser;
use memmap::Mmap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// dumps JSON describing the contents of the .car input file
    #[arg(short = 'I', long, value_name = "inputfile")]
    info: Option<String>,

    /// dumps structs from .car file
    #[arg(short = 'd', long, value_name = "inputfile")]
    debug: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(car_path) = args.info {
        let asset_catalog = AssetCatalog::try_from(car_path.as_ref())?;
        let header = serde_json::to_value(asset_catalog.header)?;
        let mut values = asset_catalog
            .assets
            .iter()
            .map(|n| serde_json::to_value(n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut result: Vec<serde_json::Value> = vec![header];
        result.append(&mut values);

        let j = serde_json::to_string_pretty(&result)?;
        println!("{}", j);
    } else if let Some(car_path) = args.debug {
        // let asset_catalog = AssetCatalog::try_from(car_path.as_ref())?;
        // dbg!(asset_catalog);
        let file = fs::File::open(car_path.clone())?;
        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {}", car_path)) };
        let mut cursor = Cursor::new(mmap);

        let bom_header = BOMHeader::read(&mut cursor)?;
        for var in &bom_header.vars.vars {
            println!("{:?}", &var.name);

            // let index = var.index as usize;
            // let pointer = &bom_header.index_header.pointers[index];
        }
    } else {
        return Args::command().print_help().context("no args?");
    }
    Ok(())
}
