use emulator::*;
use ppu::*;

pub fn write_ppuctrl(ppu: &mut Ppu, value: u8) {
    ppu.ppuctrl = value;
    if (ppu.ppuctrl & ppu.ppustatus & 0x80) != 0 {
        // TODO: trigger NMI
        warn!("NMI not triggered");
    }
}

pub fn write_ppumask(ppu: &mut Ppu, value: u8) {
    ppu.ppumask = value
}

pub fn read_ppustatus(ppu: &mut Ppu) -> u8 {
    let value = ppu.ppustatus;
    ppu.ppustatus &= 0x7f;
    ppu.flipflop = false;
    value
}

#[cfg(any(feature = "trace", test))]
pub fn read_ppustatus_debug(ppu: &Ppu) -> u8 {
    ppu.ppustatus
}

pub fn write_oamaddr(ppu: &mut Ppu, value: u8) {
    ppu.oamaddr = value;
}

pub fn read_oamdata(ppu: &mut Ppu) -> u8 {
    ppu.oam[ppu.oamaddr as usize]
}

#[cfg(any(feature = "trace", test))]
pub fn read_oamdata_debug(ppu: &Ppu) -> u8 {
    ppu.oam[ppu.oamaddr as usize]
}

pub fn write_oamdata(ppu: &mut Ppu, value: u8) {
    ppu.oam[ppu.oamaddr as usize] = value;
    ppu.oamaddr = ppu.oamaddr.wrapping_add(1);
}

pub fn write_ppuscroll(ppu: &mut Ppu, value: u8) {
    if ppu.flipflop {
        ppu.scroll_y = value
    } else {
        ppu.scroll_x = value
    };
    ppu.flipflop = !ppu.flipflop;
}

pub fn write_ppuaddr(ppu: &mut Ppu, value: u8) {
    ppu.ppuaddr = if ppu.flipflop {
        (ppu.ppuaddr & 0xff00) | (value as u16)
    } else {
        (ppu.ppuaddr & 0x00ff) | ((value as u16) << 8)
    } % 0x4000;
    ppu.flipflop = !ppu.flipflop;
}

pub fn read_ppudata(ppu: &mut Ppu) -> u8 {
    let old_value = ppu.ppudata_buffer;
    ppu.ppudata_buffer = ppu.memory.read(ppu.ppuaddr);
    increment_ppuaddr(ppu);
    if ppu.ppuaddr <= 0x3eff {
        old_value
    } else {
        ppu.ppudata_buffer
    }
}

#[cfg(any(feature = "trace", test))]
pub fn read_ppudata_debug(ppu: &Ppu) -> u8 {
    if ppu.ppuaddr <= 0x3eff {
        ppu.ppudata_buffer
    } else {
        ppu.memory.read(ppu.ppuaddr)
    }
}

pub fn write_ppudata(ppu: &mut Ppu, value: u8) {
    ppu.memory.write(ppu.ppuaddr, value);
    increment_ppuaddr(ppu);
}

pub fn write_oamdma(emulator: &mut Emulator, value: u8) {
    let start = (value as usize) << 8;
    let end = start + OAM_SIZE;
    for value in &emulator.ram[start..end] {
        emulator.ppu.oam[emulator.ppu.oamaddr as usize] = *value;
        emulator.ppu.oamaddr = emulator.ppu.oamaddr.wrapping_add(1);
    }
}

fn increment_ppuaddr(ppu: &mut Ppu) {
    ppu.ppuaddr += if (ppu.ppuctrl & 0x04) == 0 {
        1
    } else {
        32
    };
}
