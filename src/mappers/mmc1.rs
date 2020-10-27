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
    mirroring: u8,
    prg_rom_bank_mode: u8,
    chr_rom_bank_mode: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_rom_bank: u8,
    prg_ram_enable: bool
}

impl Mmc1 {
    pub(super) fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_ram: [0; PRG_RAM_SIZE],
            prg_rom: prg_rom.to_vec(),
            shift_register: 0b10000,
            mirroring: 0,
            prg_rom_bank_mode: 0,
            chr_rom_bank_mode: 0,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_rom_bank: 0,
            prg_ram_enable: false
        }
    }
}

impl Mapper for Mmc1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => if self.prg_ram_enable {
                self.prg_ram[(address - PRG_RAM_START) as usize]
            } else {
                0
            },
            PRG_ROM_BANK_0_START ..= PRG_ROM_BANK_0_END => {
                match self.prg_rom_bank_mode {
                    0 | 1 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + 0x8000 * (self.prg_rom_bank & 0b1110) as usize], // switchable (32 KB bank)
                    2 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize], // fixed to first bank (16 KB bank)
                    3 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + 0x4000 * self.prg_rom_bank as usize], // switchable (16 KB bank)
                    _ => unreachable!()
                }
            },
            PRG_ROM_BANK_1_START ..= PRG_ROM_BANK_1_END => {
                match self.prg_rom_bank_mode {
                    0 | 1 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + 0x8000 * (self.prg_rom_bank & 0b1110) as usize], // switchable (32 KB bank)
                    2 => self.prg_rom[(address - PRG_ROM_BANK_1_START) as usize + 0x4000 * self.prg_rom_bank as usize], // switchable (16 KB bank)
                    3 => self.prg_rom[(address - PRG_ROM_BANK_1_START) as usize + self.prg_rom.len() - 0x4000], // fixed to last bank (16 KB bank)
                    _ => unreachable!()
                }
            },
            _ => unimplemented!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => if self.prg_ram_enable {
                self.prg_ram[(address - PRG_RAM_START) as usize] = value;
            },
            PRG_ROM_START ..= PRG_ROM_END => self.shift_register = if (value & 0x80) == 0 {
                let value = ((value & 1) << 4) | (self.shift_register >> 1);
                if (self.shift_register & 1) == 0 {
                    value
                } else {
                    match address {
                        0x8000 ..= 0x9fff => {
                            self.mirroring = value & 0b11;
                            self.prg_rom_bank_mode = (value >> 2) & 0b11;
                            self.chr_rom_bank_mode = value >> 4;
                        },
                        0xa000 ..= 0xbfff => self.chr_bank_0 = value,
                        0xc000 ..= 0xdfff => self.chr_bank_1 = value,
                        0xe000 ..= 0xffff => {
                            self.prg_rom_bank = value & 0b1111;
                            self.prg_ram_enable = (value >> 4) == 0;
                        },
                        _ => unreachable!()
                    }
                    0b10000
                }
            } else {
                self.prg_rom_bank_mode = 0b11;
                0b10000
            },
            _ => unimplemented!()
        }
    }
}
