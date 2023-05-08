use anyhow::Result;

use clap::arg;
use clap::command;
use clap::CommandFactory;
use clap::Parser;
use clap::Subcommand;

use assetutil::ToAssetUtilHeader;

mod actool;
mod assetutil;
mod bom;
mod common;
mod coregraphics;
mod coreui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// compatible with assetutil cli tool
    Assetutil {
        /// dumps JSON describing the contents of the .car input file
        #[arg(short = 'I', long, value_name = "inputfile")]
        info: Option<String>,
    },
    /// compatible with actool cli tool
    Actool {
        /// By default, actool provides output in the form of an XML property
        /// list. Specifying binary1 will instruct actool to output a binary
        /// property list. Similarly, xml1 specifies an XML property list,
        /// and human-readable-text specifies human readable text.
        #[arg(long, value_name = "format")]
        output_format: Option<String>,

        /// Compiles document and writes the output to the specified directory
        /// path. The name of the CAR file will be Assets.car. The compile
        /// option instructs actool to convert an asset catalog to files
        /// optimized for runtime.
        #[arg(long, value_name = "path")]
        compile: Option<String>,

        /// Specifies the target platform to compile for. This option influences
        /// warnings, validation, and which images are included in the built
        /// product.
        #[arg(long, value_name = "platform_name")]
        platform: Option<String>,

        document: String,
    },
    /// extract images from Assets.car
    Extract {
        /// path to Assets.car
        car_path: String,

        /// path to dump images
        #[arg(short = 'o', long, value_name = "inputfile", default_value = ".")]
        output_path: String,
    },
    /// dumps structs of parsed Assets.car
    Debug {
        /// path to Assets.car
        car_path: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Assetutil { info } => {
            if let Some(car_path) = info {
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
            } else {
                Cli::command().print_help()?;
                Ok(())
            }
        }
        Commands::Actool {
            output_format,
            compile,
            platform,
            document,
        } => {
            if let Some(output_path) = compile {
                actool::compile(&document, &output_path)
            } else {
                Ok(())
            }
        }
        Commands::Extract {
            car_path,
            output_path,
        } => {
            let car = coreui::CarUtilAssetStorage::from(&car_path, false)?;
            let imagedb = car.theme_store.store.imagedb;
            for (_rendition_key, csi_header) in imagedb.iter() {
                let result = csi_header.extract(&output_path);
                if let Err(err) = result {
                    eprintln!("Unable to extract: {}", err);
                } else if let Ok(Some(output_path)) = result {
                    eprintln!("Extracted: {}", output_path);
                }
            }
            Ok(())
        }
        Commands::Debug { car_path } => {
            let car = coreui::CarUtilAssetStorage::from(&car_path, false)?;
            dbg!(car.theme_store.store.header);
            dbg!(car.theme_store.store.extended_metadata);
            dbg!(car.theme_store.store.renditionkeyfmt);
            dbg!(car.theme_store.store.appearancedb);
            dbg!(car.theme_store.store.bitmapkeydb);
            dbg!(car.theme_store.store.facetkeysdb);
            dbg!(car.theme_store.store.imagedb);
            Ok(())
        }
    }
}
