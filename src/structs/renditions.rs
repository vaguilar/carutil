use binrw::BinRead;

#[derive(Debug, BinRead)]
enum CUIRendition {
    CUIRawDataRendition {
        #[br(magic = b"RAWD")]
        tag: u32,
        version: u32,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: Vec<u8>,
    },
}
