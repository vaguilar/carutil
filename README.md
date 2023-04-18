# car-reader
A cross-platform clone of the Mac `assetutil` tool.

## Installation
```
git clone https://github.com/vaguilar/car-reader.git
cd car-reader
cargo build
```

## Usage
```
cargo run -- --info ./path/to/Assets.car
```

## Options
```
Usage: car_reader [OPTIONS]

Options:
  -I, --info <inputfile>   dumps JSON describing the contents of the .car input file
  -d, --debug <inputfile>  dumps structs from .car file
  -h, --help               Print help
  -V, --version            Print version
```