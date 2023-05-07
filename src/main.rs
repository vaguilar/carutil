use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;

use anyhow::Result;

use binrw::BinRead;
use clap::arg;
use clap::command;
use clap::CommandFactory;
use clap::Parser;
use memmap::Mmap;

use assetutil::ToAssetUtilHeader;

mod assetutil;
mod bom;
mod common;
mod coregraphics;
mod coreui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// dumps JSON describing the contents of the .car input file
    #[arg(short = 'I', long, value_name = "inputfile")]
    info: Option<String>,

    /// dumps structs from .car file
    #[arg(short = 'd', long, value_name = "inputfile")]
    debug: Option<String>,

    /// extract available images from .car file
    #[arg(short = 'e', long, value_name = "inputfile")]
    extract_images: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(car_path) = args.info {
        let car = coreui::CarUtilAssetStorage::from(&car_path, false)?;

        let asset_util_header = serde_json::to_value(car.asset_util_header())?;
        let mut result: Vec<serde_json::Value> = vec![asset_util_header];

        let mut entries =
            assetutil::AssetUtilEntry::entries_from_asset_storage(&car.theme_store.store);
        entries.sort_by(|a, b| {
            (
                a.asset_type.clone(),
                a.name.clone(),
                a.rendition_name.clone(),
            )
                .cmp(&(
                    b.asset_type.clone(),
                    b.name.clone(),
                    b.rendition_name.clone(),
                ))
        });
        for entry in entries {
            let value = serde_json::to_value(entry)?;
            result.push(value);
        }

        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);
        Ok(())
    } else if let Some(car_path) = args.debug {
        // let asset_catalog = AssetCatalog::try_from(car_path.as_ref())?;
        // dbg!(asset_catalog);
        let file = fs::File::open(car_path.clone())?;
        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {}", car_path)) };
        let mut cursor = Cursor::new(mmap);

        let bom_header = bom::Storage::read(&mut cursor)?;
        for var in &bom_header.var_storage.vars {
            println!("{:?}", &var.name);

            // let index = var.index as usize;
            // let pointer = &bom_header.index_header.pointers[index];
        }
        Ok(())
    } else if let Some(car_path) = args.extract_images {
        let car = coreui::CarUtilAssetStorage::from(&car_path, false)?;
        let imagedb = car.theme_store.store.imagedb.unwrap_or_default();
        for (_rendition_key, csi_header) in imagedb.iter() {
            let result = csi_header.extract("/tmp/out/");
            if let Err(err) = result {
                eprintln!("{:?}", err);
            }
        }
        Ok(())
    } else {
        Args::command().print_help()?;
        Ok(())
    }
}
