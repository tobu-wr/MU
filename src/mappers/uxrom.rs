use super::*;

const PRG_ROM_BANK_SIZE: usize = 0x4000;

const PRG_ROM_SWITCHABLE_BANK_START: u16 = 0x8000;
const PRG_ROM_SWITCHABLE_BANK_END: u16 = 0xbfff;

const PRG_ROM_LAST_BANK_START: u16 = 0xc000;
const PRG_ROM_LAST_BANK_END: u16 = 0xffff;

pub(super) struct Uxrom {
    prg_rom: Vec<u8>,
    bank: u8
}

impl Uxrom {
    pub(super) fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_rom: prg_rom.to_vec(),
            bank: 0
        }
    }
}

impl Mapper for Uxrom {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x6000 ..= 0x7fff => 0,
            PRG_ROM_SWITCHABLE_BANK_START ..= PRG_ROM_SWITCHABLE_BANK_END => self.prg_rom[(address - PRG_ROM_SWITCHABLE_BANK_START) as usize + PRG_ROM_BANK_SIZE * self.bank as usize],
            PRG_ROM_LAST_BANK_START ..= PRG_ROM_LAST_BANK_END => self.prg_rom[(address - PRG_ROM_LAST_BANK_START) as usize + self.prg_rom.len() - PRG_ROM_BANK_SIZE],
            _ => unimplemented!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000 ..= 0xffff => self.bank = value & 0b1111,
            _ => unimplemented!()
        }
    }
}
