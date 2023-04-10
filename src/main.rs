use std::{io::Cursor, fs};

use anyhow::{Result, Context};

use binrw::BinRead;
use car_reader_lib::{AssetCatalog, bom::BOMHeader};
use clap::{Parser, command, arg, CommandFactory};
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
        let mut values = asset_catalog.assets
            .iter()
            .map(|n| serde_json::to_value(n))
            .collect::<Result<Vec<_>, _>>()?;
        // values.sort_by(|a, b| {
        //     let a_type = a.as_object().unwrap().get("AssetType").unwrap().as_str().unwrap();
        //     let a_name = a.as_object().unwrap().get("Name").unwrap().as_str().unwrap();
        //     let b_type = b.as_object().unwrap().get("AssetType").unwrap().as_str().unwrap();
        //     let b_name = b.as_object().unwrap().get("Name").unwrap().as_str().unwrap();
        //     (a_type, a_name).cmp(&(b_type, b_name))
        // });
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
            let name = String::from_utf8(var.name.clone())?;
            println!("{:?}", name);

            let index = var.index as usize;
            let pointer = &bom_header.index_header.pointers[index];
            // pointer.
        }
    } else {
        return Args::command().print_help().context("no args?");
    }
    Ok(())
}
