use binrw::BinRead;
use binrw::BinWrite;
use std::fmt::Debug;

#[derive(BinRead, BinWrite)]
#[brw(little)]
pub struct Key {
    pub raw: [u16; 11],
}

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "BitmapKey {{ {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {} }}",
            self.raw[0],
            self.raw[1],
            self.raw[2],
            self.raw[3],
            self.raw[4],
            self.raw[5],
            self.raw[6],
            self.raw[7],
            self.raw[8],
            self.raw[9],
            self.raw[10],
        ))
    }
}
