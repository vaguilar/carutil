use binrw::{helpers, BinResult};
use std::io::{Read, Seek};

// parse strings with dynamic length
pub fn dynamic_length_string_parser<R: Read + Seek>(
    length: usize,
) -> impl Fn(&mut R, binrw::Endian, ()) -> BinResult<String> {
    move |reader, endian, args| {
        let buffer: Vec<u8> = helpers::count(length)(reader, endian, args)?;
        let mut string_length: usize = 0;
        for i in 0..length {
            if buffer[i] == 0 {
                break;
            }
            string_length += 1
        }

        Ok(String::from_utf8_lossy(&buffer[..string_length]).to_string())
    }
}
