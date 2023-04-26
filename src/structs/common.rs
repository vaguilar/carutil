use binrw::{BinRead, VecArgs, helpers::count_with};
use std::fmt::Debug;

// wrap Vec<u8> to make debugging better
#[derive(Clone)]
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