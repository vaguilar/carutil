use crate::car::CarHeader;
use anyhow::{Result};
use bom::BOMHeader;
use binrw::{BinRead};
use std::{fs, io::Cursor};

mod car;
mod bom;

pub fn parse_bom(file_path: &str) -> Result<BOMHeader> {
    let contents = fs::read(file_path)?;
    let mut cursor = Cursor::new(contents);
    Ok(BOMHeader::read(&mut cursor).unwrap())
}

pub fn parse(file_path: &str) -> Result<CarHeader> {
    let contents = fs::read(file_path)?;
    let mut cursor = Cursor::new(contents);
    Ok(CarHeader::read(&mut cursor).unwrap())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
