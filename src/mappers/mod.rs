mod nrom;
mod mmc1;
mod unrom;
mod mmc3;

use self::nrom::*;
use self::mmc1::*;
use self::unrom::*;
use self::mmc3::*;

pub trait Mapper {
    fn read(&self, u16) -> u8 {
        unimplemented!();
    }

    fn write(&mut self, u16, u8) {
        unimplemented!();
    }
}

pub fn create_mapper(number: u8, prg_rom: &[u8]) -> Box<dyn Mapper> {
    match number {
        0 => Box::new(Nrom::new(prg_rom)),
        1 => Box::new(Mmc1::new(prg_rom)),
        2 => Box::new(Unrom::new(prg_rom)),
        4 => Box::new(Mmc3::new(prg_rom)),
        _ => unimplemented!()
    }
}
