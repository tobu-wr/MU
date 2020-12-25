use super::*;

const PRG_RAM_SIZE: usize = 0x2000;
const PRG_RAM_START: u16 = 0x6000;
const PRG_RAM_END: u16 = 0x7fff;

const PRG_ROM_BANK_SIZE: usize = 0x2000;

const PRG_ROM_BANK_0_START: u16 = 0x8000;
const PRG_ROM_BANK_0_END: u16 = 0x9fff;

const PRG_ROM_BANK_1_START: u16 = 0xa000;
const PRG_ROM_BANK_1_END: u16 = 0xbfff;

const PRG_ROM_BANK_2_START: u16 = 0xc000;
const PRG_ROM_BANK_2_END: u16 = 0xdfff;

const PRG_ROM_BANK_3_START: u16 = 0xe000;
const PRG_ROM_BANK_3_END: u16 = 0xffff;

pub(super) struct Mmc3 {
    prg_ram: [u8; PRG_RAM_SIZE],
    prg_rom: Vec<u8>,
    r: u8,
    prg_rom_bank_mode: u8,
    chr_a12_inversion: u8,
    prg_rom_bank_0: u8,
    prg_rom_bank_1: u8,
    mirroring: u8,
    prg_ram_enable: bool
}

impl Mmc3 {
    pub(super) fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_ram: [0; PRG_RAM_SIZE],
            prg_rom: prg_rom.to_vec(),
            r: 0,
            prg_rom_bank_mode: 0,
            chr_a12_inversion: 0,
            prg_rom_bank_0: 0,
            prg_rom_bank_1: 0,
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
            PRG_ROM_BANK_0_START ..= PRG_ROM_BANK_0_END => match self.prg_rom_bank_mode {
                0 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + PRG_ROM_BANK_SIZE * self.prg_rom_bank_0 as usize], // switchable
                1 => self.prg_rom[(address - PRG_ROM_BANK_0_START) as usize + self.prg_rom.len() - 2 * PRG_ROM_BANK_SIZE], // fixed to second-last bank
                _ => unreachable!()
            },
            PRG_ROM_BANK_1_START ..= PRG_ROM_BANK_1_END => self.prg_rom[(address - PRG_ROM_BANK_1_START) as usize + PRG_ROM_BANK_SIZE * self.prg_rom_bank_1 as usize], // switchable
            PRG_ROM_BANK_2_START ..= PRG_ROM_BANK_2_END => match self.prg_rom_bank_mode {
                0 => self.prg_rom[(address - PRG_ROM_BANK_2_START) as usize + self.prg_rom.len() - 2 * PRG_ROM_BANK_SIZE], // fixed to second-last bank
                1 => self.prg_rom[(address - PRG_ROM_BANK_2_START) as usize + PRG_ROM_BANK_SIZE * self.prg_rom_bank_0 as usize], // switchable
                _ => unreachable!()
            },
            PRG_ROM_BANK_3_START ..= PRG_ROM_BANK_3_END => self.prg_rom[(address - PRG_ROM_BANK_3_START) as usize + self.prg_rom.len() - PRG_ROM_BANK_SIZE], // fixed to last bank
            _ => unimplemented!()
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            PRG_RAM_START ..= PRG_RAM_END => if self.prg_ram_enable {
                self.prg_ram[(address - PRG_RAM_START) as usize] = value;
            },
            PRG_ROM_BANK_0_START ..= PRG_ROM_BANK_0_END => if (value % 2) == 0 {
                self.r = value & 0b111;
                self.prg_rom_bank_mode = (value >> 6) & 1;
                self.chr_a12_inversion = value >> 7;
            } else {
                match self.r {
                    0 => {}, // TODO
                    1 => {}, // TODO
                    2 => {}, // TODO
                    3 => {}, // TODO
                    4 => {}, // TODO
                    5 => {}, // TODO
                    6 => self.prg_rom_bank_0 = value & 0x3f,
                    7 => self.prg_rom_bank_1 = value & 0x3f,
                    _ => unreachable!()
                }
            },
            PRG_ROM_BANK_1_START ..= PRG_ROM_BANK_1_END => if (value % 2) == 0 {
                self.mirroring = value & 1;
            } else {
                self.prg_ram_enable = (value >> 7) == 1;
            },
            PRG_ROM_BANK_2_START ..= PRG_ROM_BANK_2_END => if (value % 2) == 0 {
                // TODO: IRQ latch
            } else {
                // TODO: IRQ reload
            },
            PRG_ROM_BANK_3_START ..= PRG_ROM_BANK_3_END => if (value % 2) == 0 {
                // TODO: IRQ disable
            } else {
                // TODO: IRQ enable
            },
            _ => unimplemented!()
        }
    }
}
