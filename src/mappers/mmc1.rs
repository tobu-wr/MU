use super::*;

const PRG_RAM_SIZE: usize = 0x2000;
const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7fff;

const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;

const PRG_ROM_BANK_0_START: u16 = 0x8000;
const PRG_ROM_BANK_0_END: u16 = 0xbfff;

const PRG_ROM_BANK_1_START: u16 = 0xc000;
const PRG_ROM_BANK_1_END: u16 = 0xffff;

pub(super) struct Mmc1 {
    prg_ram: [u8; PRG_RAM_SIZE],
    prg_rom: Vec<u8>,
    shift_register: u8,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8
}

impl Mmc1 {
    pub(super) fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_ram: [0; PRG_RAM_SIZE],
            prg_rom: prg_rom.to_vec(),
            shift_register: 0b10000,
            control: 0,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0
        }
    }
}

impl Mapper for Mmc1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => self.prg_ram[(address - PRG_RAM_START) as usize],
            PRG_ROM_BANK_0_START ..= PRG_ROM_BANK_0_END => {
                let mode = (self.control >> 2) & 0b11;
                match mode {
                    0 | 1 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + 0x8000 * (self.prg_bank & 0b11110) as usize],
                    2 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize],
                    3 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + 0x4000 * self.prg_bank as usize],
                    _ => panic!()
                }
            },
            PRG_ROM_BANK_1_START ..= PRG_ROM_BANK_1_END => {
                let mode = (self.control >> 2) & 0b11;
                match mode {
                    0 | 1 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + 0x8000 * (self.prg_bank & 0b11110) as usize],
                    2 => self.prg_rom[(address - PRG_ROM_BANK_1_START) as usize + 0x4000 * self.prg_bank as usize],
                    3 => self.prg_rom[(address - PRG_ROM_BANK_1_START) as usize + self.prg_rom.len() - 0x4000],
                    _ => panic!()
                }
            },
            _ => unimplemented!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => self.prg_ram[(address - PRG_RAM_START) as usize] = value,
            PRG_ROM_START ..= PRG_ROM_END => if (value & 0x80) == 0 {
                if (self.shift_register & 1) == 0 {
                    self.shift_register = ((value & 1) << 4) | (self.shift_register >> 1);
                } else {
                    let value = ((value & 1) << 4) | (self.shift_register >> 1);
                    match address {
                        0x8000 ..= 0x9fff => self.control = value,
                        0xa000 ..= 0xbfff => self.chr_bank_0 = value,
                        0xc000 ..= 0xdfff => self.chr_bank_1 = value,
                        0xe000 ..= 0xffff => self.prg_bank = value,
                        _ => panic!()
                    }
                    self.shift_register = 0b10000;
                }
            } else {
                self.shift_register = 0b10000;
            },
            _ => unimplemented!()
        }
    }
}
