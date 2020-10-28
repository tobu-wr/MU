use super::*;

const PRG_RAM_SIZE: usize = 0x2000;
const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7fff;

pub(super) struct Mmc3 {
    prg_ram: [u8; PRG_RAM_SIZE],
    prg_rom: Vec<u8>,
    prg_rom_bank_mode: u8,
    mirroring: u8,
    prg_ram_enable: bool
}

impl Mmc3 {
    pub(super) fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_ram: [0; PRG_RAM_SIZE],
            prg_rom: prg_rom.to_vec(),
            prg_rom_bank_mode: 0,
            mirroring: 0,
            prg_ram_enable: false
        }
    }
}

impl Mapper for Mmc3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => if self.prg_ram_enable {
                self.prg_ram[(address - PRG_RAM_START) as usize]
            } else {
                0
            },
            _ => unimplemented!()
        }
    }
}
