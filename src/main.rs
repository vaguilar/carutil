use car_reader_lib::{self, parse_bom, bom::BOMEntry};
use clap::{Parser, command, arg, CommandFactory};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// dumps a JSON file describing the contents of the inputfile 
   #[arg(short = 'I', long, value_name = "inputfile")]
   info: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(car_path) = args.info {
        println!("{}", car_path);
        let car = parse_bom(&car_path).unwrap();

        for entry in car.var_entries() {
            match &(*entry) {
                BOMEntry::CarHeader { header } => {
                    println!("{:?}", header);
                },
                BOMEntry::CarExtendedMetadata { metadata } => {
                    println!("{:?}", metadata);
                },
                BOMEntry::Tree{tree} => {
                    println!("{:?}", tree);
                },
                BOMEntry::KeyFormat{ key_format } => {
                    println!("{:?}", key_format);
                },
                BOMEntry::Unknown{ .. } => {
                    unimplemented!("Unimplemented BOMEntry type");
                },
            }
            println!("");
        }
    } else {
        Args::command().print_help().unwrap();
    }
}
