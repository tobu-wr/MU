use super::*;

const PRG_ROM_BANK_SIZE: usize = 0x8000;

const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;

pub(super) struct Axrom {
    prg_rom: Vec<u8>,
    prg_rom_bank: u8
}

impl Axrom {
    pub(super) fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_rom: prg_rom.to_vec(),
            prg_rom_bank: 0
        }
    }
}

impl Mapper for Axrom {
    fn read(&self, address: u16) -> u8 {
        match address {
            PRG_ROM_START ..= PRG_ROM_END => self.prg_rom[(address - PRG_ROM_START) as usize + PRG_ROM_BANK_SIZE * self.prg_rom_bank as usize], // switchable
            _ => unimplemented!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            PRG_ROM_START ..= PRG_ROM_END => self.prg_rom_bank = value & 0b111,
            _ => unimplemented!()
        }
    }
}
