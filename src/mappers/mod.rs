mod nrom;
mod mmc1;
mod unrom;
mod mmc3;

use self::nrom::*;
use self::mmc1::*;
use self::unrom::*;
use self::mmc3::*;

pub trait Mapper {
    // TODO
}

// TODO
pub fn create_mapper(mapper: u8) -> Box<dyn Mapper> {
    match mapper {
        0 => Box::new(Nrom::new()),
        1 => Box::new(Mmc1::new()),
        2 => Box::new(Unrom::new()),
        4 => Box::new(Mmc3::new()),
        _ => unimplemented!()
    }
}
