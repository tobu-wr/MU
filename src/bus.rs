use emulator::*;
use mappers::*;
use ppu::*;
use apu::*;
use joypad::*;

pub struct Bus<'a> {
    ram: &'a mut [u8; RAM_SIZE],
    mapper: &'a mut Option<Box<dyn Mapper>>,
    ppu: &'a mut Ppu,
	apu: &'a mut Apu,
    joypad: &'a mut Joypad
}

impl<'a> Bus<'a> {
    pub fn new(emulator: &'a mut Emulator) -> Self {
        Self {
            ram: &mut emulator.ram,
            mapper: &mut emulator.mapper,
            ppu: &mut emulator.ppu,
            apu: &mut emulator.apu,
            joypad: &mut emulator.joypad
        }
    }
}
