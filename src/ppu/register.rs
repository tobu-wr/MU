use ppu::*;

pub struct Ppuctrl;
pub struct Ppumask;
pub struct Ppustatus;
pub struct Oamaddr;
pub struct Oamdata;
pub struct Ppuscroll;
pub struct Ppuaddr;
pub struct Ppudata;

pub trait Register {
    fn name() -> String;

    fn read(_ppu: &mut Ppu) -> u8 {
        error!("Read from {}", Self::name());
        panic!();
    }

    fn write(_ppu: &mut Ppu, _value: u8) {
        error!("Write to {}", Self::name());
        panic!();
    }

    #[cfg(feature = "trace")]
	fn read_debug(_ppu: &Ppu) -> u8 {
        warn!("Read from {}", Self::name());
        0
	}
}

impl Register for Ppuctrl {
    fn name() -> String {
        "PPUCTRL".to_string()
    }

    fn write(ppu: &mut Ppu, value: u8) {
        ppu.ppuctrl = value;
        if (ppu.ppuctrl & ppu.ppustatus & 0x80) != 0 {
            // TODO: trigger NMI
            warn!("NMI not triggered");
        }
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        ppu.ppuctrl
    }
}

impl Register for Ppumask {
    fn name() -> String {
        "PPUMASK".to_string()
    }

    fn write(ppu: &mut Ppu, value: u8) {
        ppu.ppumask = value
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        ppu.ppumask
    }
}

impl Register for Ppustatus {
    fn name() -> String {
        "PPUSTATUS".to_string()
    }

    fn read(ppu: &mut Ppu) -> u8 {
        let value = ppu.ppustatus;
        ppu.ppustatus &= 0x7f;
        ppu.flipflop = false;
        value
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        ppu.ppustatus
    }
}

impl Register for Oamaddr {
    fn name() -> String {
        "OAMADDR".to_string()
    }

    fn write(ppu: &mut Ppu, value: u8) {
        ppu.oamaddr = value;
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        ppu.oamaddr
    }
}

impl Register for Oamdata {
    fn name() -> String {
        "OAMDATA".to_string()
    }

    fn write(ppu: &mut Ppu, value: u8) {
        ppu.oam[ppu.oamaddr as usize] = value;
        ppu.oamaddr = ppu.oamaddr.wrapping_add(1);
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        ppu.oam[ppu.oamaddr as usize]
    }
}

impl Register for Ppuscroll {
    fn name() -> String {
        "PPUSCROLL".to_string()
    }

    fn write(ppu: &mut Ppu, value: u8) {
        ppu.ppuscroll = write16(ppu, ppu.ppuscroll, value);
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        read16_debug(ppu, ppu.ppuscroll)
    }
}

impl Register for Ppuaddr {
    fn name() -> String {
        "PPUADDR".to_string()
    }

    fn write(ppu: &mut Ppu, value: u8) {
        ppu.ppuaddr = write16(ppu, ppu.ppuaddr, value);
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        read16_debug(ppu, ppu.ppuaddr)
    }
}

impl Register for Ppudata {
    fn name() -> String {
        "PPUDATA".to_string()
    }

    fn read(ppu: &mut Ppu) -> u8 {
        let value = ppu.memory.read(ppu.ppuaddr);
        increment_ppuaddr(ppu);
        if ppu.ppuaddr <= 0x3eff {
            let old = ppu.ppudata_buffer;
            ppu.ppudata_buffer = value;
            old
        } else {
            value
        }
    }

    fn write(ppu: &mut Ppu, value: u8) {
        ppu.memory.write(ppu.ppuaddr, value);
        increment_ppuaddr(ppu);
    }

    #[cfg(feature = "trace")]
    fn read_debug(ppu: &Ppu) -> u8 {
        if ppu.ppuaddr <= 0x3eff {
            ppu.ppudata_buffer
        } else {
            ppu.memory.read(ppu.ppuaddr)
        }
    }
}

fn write16(ppu: &mut Ppu, register: u16, value: u8) -> u16 {
    ppu.flipflop = !ppu.flipflop;
    if ppu.flipflop {
        (register & 0x00ff) | ((value as u16) << 8)
    } else {
        (register & 0xff00) | (value as u16)
    }
}

fn increment_ppuaddr(ppu: &mut Ppu) {
	ppu.ppuaddr += if (ppu.ppuctrl & 0x04) == 0 {
        1
	} else {
        32
	};
}

#[cfg(feature = "trace")]
fn read16_debug(ppu: &Ppu, register: u16) -> u8 {
	(if ppu.flipflop {
        register
	} else {
        register >> 8
	}) as _
}
