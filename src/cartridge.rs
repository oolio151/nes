pub enum Mirroring {
    Vertical,
    Horizontal
}

#[allow(non_camel_case_types)]
pub struct iNESHeader{
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
    pub mapper_number: u8,
    pub mirroring: Mirroring,
    pub has_trainer: bool,
    pub battery_backed: bool,
}

pub fn parse_header(bytes: &[u8]) -> Result<iNESHeader, String> {
    // todo
}

pub fn load_rom(bytes: &[u8]) -> Result<Box<dyn Mapper>, String> {
    // todo
}