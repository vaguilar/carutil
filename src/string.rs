use std::fmt;

use binrw::{BinRead, BinReaderExt};

#[derive(Clone)]
pub struct String4(pub [u8; 4]);

impl BinRead for String4 {
    type Args<'a> = ();
    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let result = reader.read_type(endian)?;
        Ok(String4(result))
    }
}

impl fmt::Debug for String4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(&self.0))
    }
}

#[derive(Clone)]
pub struct String128(pub [u8; 128]);

impl BinRead for String128 {
    type Args<'a> = ();
    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let result = reader.read_type(endian)?;
        Ok(String128(result))
    }
}

impl fmt::Debug for String128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_length = self.0.iter()
            .position(|&c| c == b'\0')
            .unwrap_or(128);
        if let Ok(string) = std::str::from_utf8(&self.0[0..string_length]) {
            return write!(f, "{}", string);
        }
        write!(f, "BAD_STRING")
    }
}