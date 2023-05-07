# carutil
A cross-platform tool to read and extract images from Assets.car files. Also provides a clone of the Mac `assetutil` tool.

## Installation
```
git clone https://github.com/vaguilar/carutil.git
cd carutil
cargo build
```

## Usage
Output info like `assetutil`:
```
cargo run -- assetutil --info ./path/to/Assets.car
```

Extract images to a destination:
```
cargo run -- extract --output-path /tmp ./path/to/Assets.car
```

Dump structs from Assets.car to stdout for debugging:
```
cargo run -- debug ./path/to/Assets.car
```

## Commands 
```
Usage: carutil [OPTIONS]

Commands:
  assetutil  compatible with assetutil cli tool
  extract    extract images from Assets.car
  debug      dumps structs of parsed Assets.car
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```