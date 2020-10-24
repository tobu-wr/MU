use super::*;

const PRG_RAM_SIZE: usize = 0x2000;
const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7fff;

const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;

pub(super) struct Nrom {
    prg_ram: [u8; PRG_RAM_SIZE],
    prg_rom: Vec<u8>
}

impl Nrom {
    pub(super) fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_ram: [0; PRG_RAM_SIZE],
            prg_rom: prg_rom.to_vec()
        }
    }
}

impl Mapper for Nrom {
    fn read(&self, address: u16) -> u8 {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => self.prg_ram[(address - PRG_RAM_START) as usize],
            PRG_ROM_START ..= PRG_ROM_END => self.prg_rom[((address - PRG_ROM_START) as usize) % self.prg_rom.len()],
            _ => unimplemented!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => self.prg_ram[(address - PRG_RAM_START) as usize] = value,
            PRG_ROM_START ..= PRG_ROM_END => {}, // ignore writes to PRG ROM
            _ => unimplemented!()
        }
    }
}
