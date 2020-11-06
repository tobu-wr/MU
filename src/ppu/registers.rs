use emulator::*;
use ppu::*;

pub struct Ppuctrl;
pub struct Ppumask;
pub struct Ppustatus;
pub struct Oamaddr;
pub struct Oamdata;
pub struct Ppuscroll;
pub struct Ppuaddr;
pub struct Ppudata;
pub struct Oamdma;

impl Ppuctrl {
    pub fn write(ppu: &mut Ppu, value: u8) {
        ppu.ppuctrl = value;
        if (ppu.ppuctrl & ppu.ppustatus & 0x80) != 0 {
            // TODO: trigger NMI
            warn!("NMI not triggered");
        }
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(ppu: &Ppu) -> u8 {
        ppu.ppuctrl
    }
}

impl Ppumask {
    pub fn write(ppu: &mut Ppu, value: u8) {
        ppu.ppumask = value
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(ppu: &Ppu) -> u8 {
        ppu.ppumask
    }
}

impl Ppustatus {
    pub fn read(ppu: &mut Ppu) -> u8 {
        let value = ppu.ppustatus;
        ppu.ppustatus &= 0x7f;
        ppu.flipflop = false;
        value
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(ppu: &Ppu) -> u8 {
        ppu.ppustatus
    }
}

impl Oamaddr {
    pub fn write(ppu: &mut Ppu, value: u8) {
        ppu.oamaddr = value;
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(ppu: &Ppu) -> u8 {
        ppu.oamaddr
    }
}

impl Oamdata {
    pub fn read(ppu: &mut Ppu) -> u8 {
        ppu.oam[ppu.oamaddr as usize]
    }

    pub fn write(ppu: &mut Ppu, value: u8) {
        ppu.oam[ppu.oamaddr as usize] = value;
        ppu.oamaddr = ppu.oamaddr.wrapping_add(1);
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(ppu: &Ppu) -> u8 {
        ppu.oam[ppu.oamaddr as usize]
    }
}

impl Ppuscroll {
    pub fn write(ppu: &mut Ppu, value: u8) {
        if ppu.flipflop {
            ppu.scroll_y = value
        } else {
            ppu.scroll_x = value
        };
        ppu.flipflop = !ppu.flipflop;
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(_ppu: &Ppu) -> u8 {
        0
    }
}

impl Ppuaddr {
    pub fn write(ppu: &mut Ppu, value: u8) {
        ppu.ppuaddr = if ppu.flipflop {
            (ppu.ppuaddr & 0xff00) | (value as u16)
        } else {
            (ppu.ppuaddr & 0x00ff) | ((value as u16) << 8)
        } % 0x4000;
        ppu.flipflop = !ppu.flipflop;
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(ppu: &Ppu) -> u8 {
        read16_debug(ppu, ppu.ppuaddr)
    }
}

impl Ppudata {
    pub fn read(ppu: &mut Ppu) -> u8 {
        let old_value = ppu.ppudata_buffer;
        ppu.ppudata_buffer = ppu.memory.read(ppu.ppuaddr);
        increment_ppuaddr(ppu);
        if ppu.ppuaddr <= 0x3eff {
            old_value
        } else {
            ppu.ppudata_buffer
        }
    }

    pub fn write(ppu: &mut Ppu, value: u8) {
        ppu.memory.write(ppu.ppuaddr, value);
        increment_ppuaddr(ppu);
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug(ppu: &Ppu) -> u8 {
        if ppu.ppuaddr <= 0x3eff {
            ppu.ppudata_buffer
        } else {
            ppu.memory.read(ppu.ppuaddr)
        }
    }
}

impl Oamdma {
    pub fn write(emulator: &mut Emulator, value: u8) {
        let start = (value as usize) << 8;
        let end = start + OAM_SIZE;
        for value in &emulator.ram[start..end] {
            emulator.ppu.oam[emulator.ppu.oamaddr as usize] = *value;
            emulator.ppu.oamaddr = emulator.ppu.oamaddr.wrapping_add(1);
        }
    }

    #[cfg(any(feature = "trace", test))]
    pub fn read_debug() -> u8 {
        0
    }
}

fn increment_ppuaddr(ppu: &mut Ppu) {
    ppu.ppuaddr += if (ppu.ppuctrl & 0x04) == 0 {
        1
    } else {
        32
    };
}

#[cfg(any(feature = "trace", test))]
fn read16_debug(ppu: &Ppu, register: u16) -> u8 {
    (if ppu.flipflop {
        register
    } else {
        register >> 8
    }) as _
}
