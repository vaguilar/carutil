use car_reader_lib::{self, parse_bom, bom::BOMTree};
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

        for var in &(*car.vars).vars {
            println!("{} - {:?}", var.index, String::from_utf8(var.name.clone()));
            let index_index = var.index as usize;
            let index = &(*car.index_header).indices[index_index];
            match &(*index.tree) {
                BOMTree::CarHeader { header } => {
                    println!("{:?}", header);
                },
                BOMTree::CarExtendedMetadata { metadata } => {
                    println!("{:?}", metadata.authoring_tool.to_string());
                },
                BOMTree::Tree{unknown0, child, node_size, path_count, unknown3} => {
                    println!("{:?}", unknown0);
                },
                BOMTree::KeyFormat{ version, max_count, tokens_address } => {
                    println!("{:?}", version);
                },
                BOMTree::Unknown{ .. } => {
                    unimplemented!("Unimplemented BOMTree type");
                },
            }
            println!("");
        }
    } else {
        Args::command().print_help().unwrap();
    }
}
