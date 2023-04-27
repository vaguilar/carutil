use binrw::{helpers::count_with, BinRead, VecArgs};
use std::fmt::Debug;

pub mod dynamic_string;

// wrap Vec<u8> to make debugging better
#[derive(Clone, PartialOrd, PartialEq)]
pub struct RawData(pub Vec<u8>);

impl BinRead for RawData {
    type Args<'a> = VecArgs<u8>;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let r = count_with(args.count, u8::read_options)(reader, endian, ())?;
        Ok(RawData(r))
    }
}

impl Debug for RawData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_length = self.0.len();
        if data_length < 10 {
            f.debug_tuple("RawData").field(&self.0).finish()
        } else {
            f.write_str(&format!("[{} bytes]", &self.0.len()))
        }
    }
}

pub fn parse_padded_string(buffer: &[u8]) -> String {
    let (string_length, _) = buffer
        .iter()
        .enumerate()
        .find(|(_, b)| **b == 0)
        .unwrap_or((buffer.len(), &0));
    String::from_utf8_lossy(&buffer[..string_length]).to_string()
}
