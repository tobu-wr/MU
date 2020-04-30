use ppu::*;
use super::memory::*;

pub struct Ppuctrl;
pub struct Ppumask;
pub struct Ppustatus;
pub struct Oamaddr;
pub struct Ppuscroll;
pub struct Ppuaddr;
pub struct Ppudata;

pub trait Register {
    fn name() -> String;

    fn read(_ppu: &mut Ppu, _memory: &Memory) -> u8 {
        println!("[ERROR] [PPU] Read from {}", Self::name());
		std::process::exit(1);
    }

    fn write(_ppu: &mut Ppu, _memory: &mut Memory, _value: u8) {
        println!("[ERROR] [PPU] Write to {}", Self::name());
		std::process::exit(1);
    }

    #[cfg(feature = "log")]
	fn read_debug(_ppu: &Ppu, _memory: &Memory) -> u8 {
        println!("[ERROR] [PPU] Read from {}", Self::name());
		std::process::exit(1);
		/*
			Register::Ppuctrl => self.ppuctrl,
			Register::Ppumask => self.ppumask,
			Register::Ppustatus => self.ppustatus,
			Register::Ppuscroll => self.read16_debug(self.ppuscroll),
			Register::Ppuaddr => self.read16_debug(self.ppuaddr),
			Register::Ppudata => memory.read(self.ppuaddr)
		*/
	}
}

impl Register for Ppuctrl {
    fn name() -> String {
        "PPUCTRL".to_string()
    }

    fn write(ppu: &mut Ppu, _memory: &mut Memory, value: u8) {
        ppu.ppuctrl = value;
		//	TODO: check for NMI
    }
}

impl Register for Ppumask {
    fn name() -> String {
        "PPUMASK".to_string()
    }

    fn write(ppu: &mut Ppu, _memory: &mut Memory, value: u8) {
        ppu.ppumask = value
    }
}

impl Register for Ppustatus {
    fn name() -> String {
        "PPUSTATUS".to_string()
    }

    fn read(ppu: &mut Ppu, _memory: &Memory) -> u8 {
        let value = ppu.ppustatus;
		ppu.ppustatus &= 0x7f;
		ppu.flipflop = false;
		value
    }
}

impl Register for Oamaddr {
    fn name() -> String {
        "OAMADDR".to_string()
    }

    fn write(ppu: &mut Ppu, _memory: &mut Memory, value: u8) {
        ppu.oamaddr = value;
    }
}

impl Register for Ppuscroll {
    fn name() -> String {
        "PPUSCROLL".to_string()
    }

    fn write(ppu: &mut Ppu, _memory: &mut Memory, value: u8) {
        ppu.ppuscroll = write16(ppu, ppu.ppuscroll, value);
    }
}

impl Register for Ppuaddr {
    fn name() -> String {
        "PPUADDR".to_string()
    }

    fn write(ppu: &mut Ppu, _memory: &mut Memory, value: u8) {
        ppu.ppuaddr = write16(ppu, ppu.ppuaddr, value);
    }
}

impl Register for Ppudata {
    fn name() -> String {
        "PPUDATA".to_string()
    }

    fn read(ppu: &mut Ppu, memory: &Memory) -> u8 {
        let value = memory.read(ppu.ppuaddr);
		increment_ppuaddr(ppu);
		value
    }

    fn write(ppu: &mut Ppu, memory: &mut Memory, value: u8) {
        memory.write(ppu.ppuaddr, value);
		increment_ppuaddr(ppu);
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

#[cfg(feature = "log")]
fn read16_debug(ppu: &Ppu, register: u16) -> u8 {
	(if ppu.flipflop {
		register
	} else {
		register >> 8
	}) as _
}
