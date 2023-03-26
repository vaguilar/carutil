use car_reader_lib::{self, parse_bom};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let car_path = &args[1];
    println!("{}", car_path);
    let car = parse_bom(car_path).unwrap();

    // for index in &(*car.index_header).indices {
    //     println!("{:?}", index);
    // }
    // println!("{:?}", car.index_length);
    // println!("{:?}", car._trailer_len);
    for var in &(*car.vars).vars {
        println!("{} - {:?}", var.index, String::from_utf8(var.name.clone()));
        let index_index = var.index as usize;
        let index = &(*car.index_header).indices[index_index];
        // println!("{:?}", index);
        println!("{:?}", String::from_utf8(index.tree.tree.to_vec()));
        println!("{:?}", index.tree);
        println!("");
    }
}
