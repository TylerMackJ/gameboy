use std::fs::File;
use std::io::Read;

mod registers;
mod window;

use registers::*;
use window::*;

pub struct Gameboy {
    memory: [u8; 0x10000],
    registers: Registers,
    window: SdlWindow,
}

impl Gameboy {
    pub fn new() -> Result<Gameboy, String> {
        Ok(Gameboy {
            memory: [0u8; 0x10000],
            registers: Registers::new(),
            window: SdlWindow::new()?,
        })
    }

    pub fn load_rom(&mut self, rom_name: String) -> Result<(), std::io::Error> {
        // Only works for 32k roms

        let mut rom_file = File::open(rom_name)?;
        let mut bytes = Vec::new();
        rom_file.read_to_end(&mut bytes)?;
        for (i, b) in bytes.into_iter().enumerate() {
            if i < 0x8000 {
                self.memory[i] = b
            } else {
                break;
            }

        }
        Ok(())
    }

    // Write d8 to memory[addr]
    pub fn write(&mut self, addr: u16, d8: u8) {
        self.memory[addr as usize] = d8;
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    // Get u8 at pc location and increment
    pub fn get_at_pc_incr(&mut self) -> u8 {
        let value: u8 = self.read(self.registers.get_pc());
        self.registers.set_pc(&self.registers.get_pc() + 1);
        value
    }

    pub fn get_next_16(&mut self) -> u16 {
        self.get_at_pc_incr() as u16 | ((self.get_at_pc_incr() as u16) << 8)
    }

    pub fn step(&mut self) -> Result<bool, String> {
        static mut STEP_COUNT: u64 = 0;
        static START_PRINT: u64 = 1500000;

        if unsafe { STEP_COUNT } % 0x1FF == 0 {
            if !self.window.event_loop() {
                return Ok(false);
            }
            self.window.display_loop(&self.memory)?;

            let mut ly: u8 = self.read(0xFF44);
            if ly == 153 {
                ly = 0;
            } else {
                ly += 1;
            }
            self.write(0xFF44, ly);
        }

        if cfg!(debug_assertions) && unsafe { STEP_COUNT > START_PRINT } {
            println!("Before {:?}", self.registers);
        }

        let instruction = self.get_at_pc_incr();

        if cfg!(debug_assertions) && unsafe { STEP_COUNT > START_PRINT } {
            println!("Step: {}\nInstruction: 0x{:02X}", unsafe{ STEP_COUNT }, instruction);
        }

        unsafe { STEP_COUNT += 1 };

        match instruction {
            // 0x
            0x00 => {},
            0x01 => self.ld_d16(Reg16::BC),
            0x02 => {
                let bc: u16 = self.registers.get_bc();
                let a: u8 = self.registers.get_a();
                self.write(bc, a);
            }
            0x03 => self.inc_16(Reg16::BC),
            0x04 => self.inc_8(Reg8::B),
            0x05 => self.dec_8(Reg8::B),
            0x06 => self.ld_d8(Reg8::B),
            0x07 => {
                let a: u8 = self.registers.get_a();

                self.registers.set_flag(Flag::C, a & 0x80 == 0x80);
                self.registers.set_a(a << 1);
            }
            0x08 => {
                let sp: u16 = self.registers.get_sp();
                let a16: u16 = self.get_next_16();
                self.write(a16, (sp >> 8) as u8);
                self.write(a16 + 1, sp as u8)
            }

            0x09 => self.add_hl(Reg16::BC),
            0x0a => {
                let bc: u16 = self.registers.get_bc();
                let d8: u8 = self.read(bc);
                self.registers.set_a(d8);
            }
            0x0b => self.dec_16(Reg16::BC),
            0x0c => self.inc_8(Reg8::C),
            0x0d => self.dec_8(Reg8::C),
            0x0e => self.ld_d8(Reg8::C),

            // 1x
            0x10 => self.stop(),
            0x11 => self.ld_d16(Reg16::DE),
            0x12 => {
                let de: u16 = self.registers.get_de();
                let a: u8 = self.registers.get_a();
                self.write(de, a);
            }
            0x13 => self.inc_16(Reg16::DE),
            0x14 => self.inc_8(Reg8::D),
            0x15 => self.dec_8(Reg8::D),
            0x16 => self.ld_d8(Reg8::D),

            0x18 => self.jr(true),
            0x19 => self.add_hl(Reg16::DE),
            0x1a => {
                let de: u16 = self.registers.get_de();
                let d8: u8 = self.read(de);
                self.registers.set_a(d8);
            }
            0x1b => self.dec_16(Reg16::DE),
            0x1c => self.inc_8(Reg8::E),
            0x1d => self.dec_8(Reg8::E),
            0x1e => self.ld_d8(Reg8::E),

            0x1f => {
                let mut a: u8 = self.registers.get_a();
                let carry: bool = self.registers.get_flag(Flag::C);

                self.registers.set_flag(Flag::C, a & 0x01 == 0x01);
                a >>= 1;
                if carry {
                    a += 0x80;
                }
                self.registers.set_a(a);
            }

            // 2x
            0x20 => self.jr(self.registers.get_flag(Flag::Z) == false),
            0x21 => self.ld_d16(Reg16::HL),
            0x22 => {
                let a: u8 = self.registers.get_a();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, a);
                self.registers.set_hl(hl + 1);
            }
            0x23 => self.inc_16(Reg16::HL),
            0x24 => self.inc_8(Reg8::H),
            0x25 => self.dec_8(Reg8::H),
            0x26 => self.ld_d8(Reg8::H),

            0x28 => self.jr(self.registers.get_flag(Flag::Z) == true),
            0x29 => self.add_hl(Reg16::HL),

            0x2b => self.dec_16(Reg16::HL),
            0x2a => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_a(d8);
                self.registers.set_hl(hl + 1);
            }
            0x2c => self.inc_8(Reg8::L),
            0x2d => self.inc_8(Reg8::L),
            0x2e => self.ld_d8(Reg8::L),
            0x2f => self.cpl(),

            // 3x
            0x30 => self.jr(self.registers.get_flag(Flag::C) == false),
            0x31 => self.ld_d16(Reg16::SP),
            0x32 => {
                let a: u8 = self.registers.get_a();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, a);
                self.registers.set_hl(hl - 1);
            }
            0x33 => self.inc_16(Reg16::SP),

            0x36 => {
                let d8: u8 = self.get_at_pc_incr();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, d8)
            }

            0x38 => self.jr(self.registers.get_flag(Flag::C) == true),
            0x39 => self.add_hl(Reg16::SP),
            0x3a => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_a(d8);
                self.registers.set_hl(hl - 1);
            }

            0x3b => self.dec_16(Reg16::SP),
            0x3c => self.inc_8(Reg8::A),
            0x3d => self.dec_8(Reg8::A),
            0x3e => self.ld_d8(Reg8::A),

            // 4x
            0x40 => self.ld_reg8(Reg8::B, Reg8::B),
            0x41 => self.ld_reg8(Reg8::B, Reg8::C),
            0x42 => self.ld_reg8(Reg8::B, Reg8::D),
            0x43 => self.ld_reg8(Reg8::B, Reg8::E),
            0x44 => self.ld_reg8(Reg8::B, Reg8::H),
            0x45 => self.ld_reg8(Reg8::B, Reg8::L),
            0x46 => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_b(d8);
            }
            0x47 => self.ld_reg8(Reg8::B, Reg8::A),
            0x48 => self.ld_reg8(Reg8::C, Reg8::B),
            0x49 => self.ld_reg8(Reg8::C, Reg8::C),
            0x4a => self.ld_reg8(Reg8::C, Reg8::D),
            0x4b => self.ld_reg8(Reg8::C, Reg8::E),
            0x4c => self.ld_reg8(Reg8::C, Reg8::H),
            0x4d => self.ld_reg8(Reg8::C, Reg8::L),
            0x4e => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_c(d8);
            }
            0x4f => self.ld_reg8(Reg8::C, Reg8::A),

            // 5x
            0x50 => self.ld_reg8(Reg8::D, Reg8::B),
            0x51 => self.ld_reg8(Reg8::D, Reg8::C),
            0x52 => self.ld_reg8(Reg8::D, Reg8::D),
            0x53 => self.ld_reg8(Reg8::D, Reg8::E),
            0x54 => self.ld_reg8(Reg8::D, Reg8::H),
            0x55 => self.ld_reg8(Reg8::D, Reg8::L),
            0x56 => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_d(d8);
            }
            0x57 => self.ld_reg8(Reg8::D, Reg8::A),
            0x58 => self.ld_reg8(Reg8::E, Reg8::B),
            0x59 => self.ld_reg8(Reg8::E, Reg8::C),
            0x5a => self.ld_reg8(Reg8::E, Reg8::D),
            0x5b => self.ld_reg8(Reg8::E, Reg8::E),
            0x5c => self.ld_reg8(Reg8::E, Reg8::H),
            0x5d => self.ld_reg8(Reg8::E, Reg8::L),
            0x5e => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_e(d8);
            }
            0x5f => self.ld_reg8(Reg8::E, Reg8::A),

            // 6x
            0x60 => self.ld_reg8(Reg8::H, Reg8::B),
            0x61 => self.ld_reg8(Reg8::H, Reg8::C),
            0x62 => self.ld_reg8(Reg8::H, Reg8::D),
            0x63 => self.ld_reg8(Reg8::H, Reg8::E),
            0x64 => self.ld_reg8(Reg8::H, Reg8::H),
            0x65 => self.ld_reg8(Reg8::H, Reg8::L),
            0x66 => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_h(d8);
            }
            0x67 => self.ld_reg8(Reg8::H, Reg8::A),
            0x68 => self.ld_reg8(Reg8::L, Reg8::B),
            0x69 => self.ld_reg8(Reg8::L, Reg8::C),
            0x6a => self.ld_reg8(Reg8::L, Reg8::D),
            0x6b => self.ld_reg8(Reg8::L, Reg8::E),
            0x6c => self.ld_reg8(Reg8::L, Reg8::H),
            0x6d => self.ld_reg8(Reg8::L, Reg8::L),
            0x6e => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_l(d8);
            }
            0x6f => self.ld_reg8(Reg8::L, Reg8::A),

            // 7x
            0x70 => {
                let b: u8 = self.registers.get_b();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, b);
            }
            0x71 => {
                let c: u8 = self.registers.get_c();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, c);
            }
            0x72 => {
                let d: u8 = self.registers.get_d();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, d);
            }
            0x73 => {
                let e: u8 = self.registers.get_e();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, e);
            }
            0x74 => {
                let h: u8 = self.registers.get_h();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, h);
            }
            0x75 => {
                let l: u8 = self.registers.get_l();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, l);
            }
            0x76 => return Ok(false),
            0x77 => {
                let a: u8 = self.registers.get_a();
                let hl: u16 = self.registers.get_hl();
                self.write(hl, a);
            }
            0x78 => self.ld_reg8(Reg8::A, Reg8::B),
            0x79 => self.ld_reg8(Reg8::A, Reg8::C),
            0x7a => self.ld_reg8(Reg8::A, Reg8::D),
            0x7b => self.ld_reg8(Reg8::A, Reg8::E),
            0x7c => self.ld_reg8(Reg8::A, Reg8::H),
            0x7d => self.ld_reg8(Reg8::A, Reg8::L),
            0x7e => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.registers.set_a(d8);
            }
            0x7f => self.ld_reg8(Reg8::A, Reg8::A),

            // 8x
            0x80 => self.add_a(self.registers.get_b()),
            0x81 => self.add_a(self.registers.get_c()),
            0x82 => self.add_a(self.registers.get_d()),
            0x83 => self.add_a(self.registers.get_e()),
            0x84 => self.add_a(self.registers.get_h()),
            0x85 => self.add_a(self.registers.get_l()),
            0x86 => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.add_a(d8);
            }
            0x87 => self.add_a(self.registers.get_a()),

            // ax
            0xa0 => {
                let b: u8 = self.registers.get_b();
                self.and(b);
            }
            0xa1 => {
                let c: u8 = self.registers.get_c();
                self.and(c);
            }
            0xa2 => {
                let d: u8 = self.registers.get_d();
                self.and(d);
            }
            0xa3 => {
                let e: u8 = self.registers.get_e();
                self.and(e);
            }
            0xa4 => {
                let h: u8 = self.registers.get_h();
                self.and(h);
            }
            0xa5 => {
                let l: u8 = self.registers.get_l();
                self.and(l);
            }
            0xa6 => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.and(d8);
            }
            0xa7 => {
                let a: u8 = self.registers.get_a();
                self.and(a);
            }
            0xa8 => {
                let b: u8 = self.registers.get_b();
                self.xor(b);
            }
            0xa9 => {
                let c: u8 = self.registers.get_c();
                self.xor(c);
            }
            0xaa => {
                let d: u8 = self.registers.get_d();
                self.xor(d);
            }
            0xab => {
                let e: u8 = self.registers.get_e();
                self.xor(e);
            }
            0xac => {
                let h: u8 = self.registers.get_h();
                self.xor(h);
            }
            0xad => {
                let l: u8 = self.registers.get_l();
                self.xor(l);
            }
            0xae => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.xor(d8);
            }
            0xaf => {
                let a: u8 = self.registers.get_a();
                self.xor(a);
            }

            // bx
            0xb0 => {
                let b = self.registers.get_b();
                self.or(b);
            }
            0xb1 => {
                let c: u8 = self.registers.get_c();
                self.or(c);
            }
            0xb2 => {
                let d: u8 = self.registers.get_d();
                self.or(d);
            }
            0xb3 => {
                let e: u8 = self.registers.get_e();
                self.or(e);
            }
            0xb4 => {
                let h: u8 = self.registers.get_h();
                self.or(h);
            }
            0xb5 => {
                let l: u8 = self.registers.get_l();
                self.or(l);
            }
            0xb6 => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.or(d8);
            }
            0xb7 => {
                let a: u8 = self.registers.get_a();
                self.or(a);
            }
            0xb8 => {
                let b: u8 = self.registers.get_b();
                self.cp(b);
            }
            0xb9 => {
                let c: u8 = self.registers.get_c();
                self.cp(c);
            }
            0xba => {
                let d: u8 = self.registers.get_d();
                self.cp(d);
            }
            0xbb => {
                let e: u8 = self.registers.get_e();
                self.cp(e);
            }
            0xbc => {
                let h: u8 = self.registers.get_h();
                self.cp(h);
            }
            0xbd => {
                let l: u8 = self.registers.get_l();
                self.cp(l);
            }
            0xbe => {
                let hl: u16 = self.registers.get_hl();
                let d8: u8 = self.read(hl);
                self.cp(d8);
            }
            0xbf => {
                let a = self.registers.get_a();
                self.cp(a);
            }

            // cx
            0xc0 => self.ret(self.registers.get_flag(Flag::Z) == false),
            0xc1 => self.pop_d16_into(Reg16::BC),
            0xc2 => self.jmp(self.registers.get_flag(Flag::Z) == false),
            0xc3 => self.jmp(true),

            0xc5 => self.push_d16(self.registers.get_bc()),

            0xc7 => self.rst(0x00),
            0xc8 => self.ret(self.registers.get_flag(Flag::Z) == true),
            0xc9 => self.ret(true),
            0xca => self.jmp(self.registers.get_flag(Flag::Z) == true),
            0xcb => {
                let prefixed_instruction: u8 = self.get_at_pc_incr();

                if cfg!(debug_assertions) && unsafe { STEP_COUNT > START_PRINT } {
                    println!("Prefixed Instruction: 0x{:02X}", prefixed_instruction);
                }

                match prefixed_instruction {
                    // 3x
                    0x30 => self.swap(Reg8::B),
                    0x31 => self.swap(Reg8::C),
                    0x32 => self.swap(Reg8::D),
                    0x33 => self.swap(Reg8::E),
                    0x34 => self.swap(Reg8::H),
                    0x35 => self.swap(Reg8::L),
                    0x36 => {
                        let hl: u16 = self.registers.get_hl();
                        let mut r: u8 = self.read(hl);
                        let r_bottom: u8 = r & 0x0F;
                        r >>= 4;
                        r |= r_bottom << 4;
                        self.write(hl, r);
                    }
                    0x37 => self.swap(Reg8::A),

                    // 8x
                    0x80 => self.res(0, Reg8::B),
                    0x81 => self.res(0, Reg8::C),
                    0x82 => self.res(0, Reg8::D),
                    0x83 => self.res(0, Reg8::E),
                    0x84 => self.res(0, Reg8::H),
                    0x85 => self.res(0, Reg8::L),
                    0x86 => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 0);
                        self.write(hl, d8 & mask);
                    }
                    0x87 => self.res(1, Reg8::A),
                    0x88 => self.res(1, Reg8::B),
                    0x89 => self.res(1, Reg8::C),
                    0x8a => self.res(1, Reg8::D),
                    0x8b => self.res(1, Reg8::E),
                    0x8c => self.res(1, Reg8::H),
                    0x8d => self.res(1, Reg8::L),
                    0x8e => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 1);
                        self.write(hl, d8 & mask);
                    }
                    0x8f => self.res(1, Reg8::A),

                    // 9x
                    0x90 => self.res(2, Reg8::B),
                    0x91 => self.res(2, Reg8::C),
                    0x92 => self.res(2, Reg8::D),
                    0x93 => self.res(2, Reg8::E),
                    0x94 => self.res(2, Reg8::H),
                    0x95 => self.res(2, Reg8::L),
                    0x96 => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 2);
                        self.write(hl, d8 & mask);
                    }
                    0x97 => self.res(2, Reg8::A),
                    0x98 => self.res(3, Reg8::B),
                    0x99 => self.res(3, Reg8::C),
                    0x9a => self.res(3, Reg8::D),
                    0x9b => self.res(3, Reg8::E),
                    0x9c => self.res(3, Reg8::H),
                    0x9d => self.res(3, Reg8::L),
                    0x9e => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 3);
                        self.write(hl, d8 & mask);
                    }
                    0x9f => self.res(3, Reg8::A),

                    // ax
                    0xa0 => self.res(4, Reg8::B),
                    0xa1 => self.res(4, Reg8::C),
                    0xa2 => self.res(4, Reg8::D),
                    0xa3 => self.res(4, Reg8::E),
                    0xa4 => self.res(4, Reg8::H),
                    0xa5 => self.res(4, Reg8::L),
                    0xa6 => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 4);
                        self.write(hl, d8 & mask);
                    }
                    0xa7 => self.res(4, Reg8::A),
                    0xa8 => self.res(5, Reg8::B),
                    0xa9 => self.res(5, Reg8::C),
                    0xaa => self.res(5, Reg8::D),
                    0xab => self.res(5, Reg8::E),
                    0xac => self.res(5, Reg8::H),
                    0xad => self.res(5, Reg8::L),
                    0xae => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 5);
                        self.write(hl, d8 & mask);
                    }
                    0xaf => self.res(5, Reg8::A),

                    // bx
                    0xb0 => self.res(6, Reg8::B),
                    0xb1 => self.res(6, Reg8::C),
                    0xb2 => self.res(6, Reg8::D),
                    0xb3 => self.res(6, Reg8::E),
                    0xb4 => self.res(6, Reg8::H),
                    0xb5 => self.res(6, Reg8::L),
                    0xb6 => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 6);
                        self.write(hl, d8 & mask);
                    }
                    0xb7 => self.res(6, Reg8::A),
                    0xb8 => self.res(7, Reg8::B),
                    0xb9 => self.res(7, Reg8::C),
                    0xba => self.res(7, Reg8::D),
                    0xbb => self.res(7, Reg8::E),
                    0xbc => self.res(7, Reg8::H),
                    0xbd => self.res(7, Reg8::L),
                    0xbe => {
                        let hl: u16 = self.registers.get_hl();
                        let d8: u8 = self.read(hl);
                        let mask: u8 = !(0x01 << 7);
                        self.write(hl, d8 & mask);
                    }
                    0xbf => self.res(7, Reg8::A),

                    _ => {
                        println!("0xCB{:02X} Not implemented", prefixed_instruction);
                        return Ok(false);
                    },
                }
            }

            0xcd => self.call(true),

            0xcf => self.rst(0x08),

            // dx
            0xd0 => self.ret(self.registers.get_flag(Flag::C) == false),
            0xd1 => self.pop_d16_into(Reg16::DE),

            0xd2 => self.jmp(self.registers.get_flag(Flag::C) == false),

            0xd5 => self.push_d16(self.registers.get_de()),

            0xd7 => self.rst(0x10),
            0xd8 => self.ret(self.registers.get_flag(Flag::C) == true),

            0xda => self.jmp(self.registers.get_flag(Flag::C) == true),

            0xdf => self.rst(0x18),

            // ex
            0xe0 => {
                // (0xFF00 + n) <= A
                let offset: u8 = self.get_at_pc_incr();
                let a: u8 = self.registers.get_a();
                self.write(0xFF00 + offset as u16, a);
            }
            0xe1 => self.pop_d16_into(Reg16::HL),

            0xe2 => {
                let c: u8 = self.registers.get_c();
                let a: u8 = self.registers.get_a();
                self.write(0xFF00 + c as u16, a);
            }

            0xe5 => self.push_d16(self.registers.get_hl()),
            0xe6 => {
                let d8: u8 = self.get_at_pc_incr();
                self.and(d8);
            }
            0xe7 => self.rst(0x20),

            0xe9 => {
                let hl: u16 = self.registers.get_hl();
                self.registers.set_pc(hl);
            }

            0xea => {
                // (nn) <= A
                // ROM CHECK
                let addr: u16 = self.get_next_16();
                let a: u8 = self.registers.get_a();
                self.write(addr, a);
            }

            0xef => self.rst(0x28),

            // fx
            0xf0 => {
                let a8: u8 = self.get_at_pc_incr();
                let d8: u8 = self.read(0xFF00 + a8 as u16);
                self.registers.set_a(d8);
            }
            0xf1 => self.pop_d16_into(Reg16::AF),
            0xf3 => self.interrupts_enabled(false),

            0xf5 => self.push_d16(self.registers.get_af()),

            0xf7 => self.rst(0x30),

            0xfa => {
                let a16: u16 = self.get_next_16();
                let d8: u8 = self.read(a16);
                self.registers.set_a(d8);
            }
            0xfb => self.interrupts_enabled(true),

            0xfe => {
                let d8: u8 = self.get_at_pc_incr();
                self.cp(d8);
            }
            0xff => self.rst(0x38),

            _ => {
                println!("0x{:02X} Not implemented", instruction);
                return Ok(false);
            },
        }

        if cfg!(debug_assertions) && unsafe { STEP_COUNT > START_PRINT } {
            println!("After {:?}", self.registers);
            println!("");
        }

        Ok(true)
    }

    // ---Generalized instruction implementations---

    // Add A += n
    pub fn add_a(&mut self, n: u8) {
        let a: u8 = self.registers.get_a();
        let value = a.overflowing_add(n);
        self.registers.set_a(value.0);

        self.registers.set_flag(Flag::Z, value.0 == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, (a & 0x0F) + (n & 0x0F) & 0x10 == 0x10);
        self.registers.set_flag(Flag::C, value.1);
    }

    // Add HL += n
    pub fn add_hl(&mut self, reg: Reg16) {
        let n: u16 = self.registers.get_reg_16(reg);
        let hl: u16 = self.registers.get_hl();

        let addition = hl.overflowing_add(n);
        self.registers.set_hl(addition.0);

        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, ((hl & 0x0fff) + (n & 0x0fff)) & 0x1000 == 0x1000);
        self.registers.set_flag(Flag::C, addition.1);
    }

    // And d8 with A => A 
    pub fn and(&mut self, n: u8) {
        let value: u8 = self.registers.get_a() & n;
        self.registers.set_a(value);

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, true);
        self.registers.set_flag(Flag::C, false);
    }

    // Call a16
    pub fn call(&mut self, condition: bool) {
        let a16: u16 = self.get_next_16();
        if condition {
            let pc: u16 = self.registers.get_pc();
            self.push_d16(pc);
            self.registers.set_pc(a16);
        }
    }

    // Compare
    pub fn cp(&mut self, n: u8) {
        let a: u8 = self.registers.get_a();

        self.registers.set_flag(Flag::Z, a == n);
        self.registers.set_flag(Flag::N, true);
        self.registers.set_flag(Flag::H, (a & 0x0F).overflowing_sub(n & 0x0F).1);
        self.registers.set_flag(Flag::C, a < n);
    }

    // Complement A
    pub fn cpl(&mut self) {
        let a: u8 = self.registers.get_a();
        self.registers.set_a(!a);

        self.registers.set_flag(Flag::N, true);
        self.registers.set_flag(Flag::H, true);
    }

    // Decrement an 16bit register
    pub fn dec_16(&mut self, reg: Reg16) {
        let mut value: u16 = self.registers.get_reg_16(reg);
        // Underflow
        if value == 0x00 {
            value = 0xFF;
        } else {
            value -= 1;
        }
        self.registers.set_reg_16(reg, value);
    }

    // Decrement an 8bit register
    pub fn dec_8(&mut self, reg: Reg8) {
        let mut value: u8 = self.registers.get_reg_8(reg);

        self.registers.set_flag(Flag::H, (value & 0x0F).overflowing_sub(1).1);

        // Underflow
        if value == 0x00 {
            value = 0xFF;
        } else {
            value -= 1;
        }
        self.registers.set_reg_8(reg, value);

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, true);
    }

    // Increment an 16bit register
    pub fn inc_16(&mut self, reg: Reg16) {
        let mut value: u16 = self.registers.get_reg_16(reg);
        // Underflow
        if value == 0xFF {
            value = 0x00;
        } else {
            value += 1;
        }
        self.registers.set_reg_16(reg, value);
    }

    // Increment an 8bit register
    pub fn inc_8(&mut self, reg: Reg8) {
        let mut value: u8 = self.registers.get_reg_8(reg);

        self.registers.set_flag(Flag::H, ((value & 0x0F) + 1) & 0x10 == 0x10);

        // Overflow
        if value == 0xFF {
            value = 0x00;
        } else {
            value += 1;
        }
        self.registers.set_reg_8(reg, value);

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
    }

    // Enable and disable interrupts
    pub fn interrupts_enabled(&mut self, enabled: bool) {
        self.write(0xFFFF, enabled as u8);
    }

    // Jump (Un)Conditional
    pub fn jmp(&mut self, condition: bool) {
        let addr: u16 = self.get_next_16();

        if condition{
            self.registers.set_pc(addr);
        }
    }

    // Jump Relative (Un)Conditional
    pub fn jr(&mut self, condition: bool) {
        let offset: u8 = self.get_at_pc_incr();

        if condition {
            self.registers.set_pc((self.registers.get_pc() as i16 + (offset as i8) as i16) as u16);

        }
    }

    // LD reg <- d16
    pub fn ld_d16(&mut self, reg: Reg16) {
        let d16: u16 = self.get_next_16();
        self.registers.set_reg_16(reg, d16);
    }

    // LD reg <- d8
    pub fn ld_d8(&mut self, reg: Reg8) {
        let d8: u8 = self.get_at_pc_incr();
        self.registers.set_reg_8(reg, d8);
    }

    // LD dst <- src
    pub fn ld_reg8(&mut self, dst: Reg8, src: Reg8) {
        let d8: u8 = self.registers.get_reg_8(src);
        self.registers.set_reg_8(dst, d8);
    }

    // OR n with A => A
    pub fn or(&mut self, n: u8) {
        let value: u8 = self.registers.get_a() | n;
        self.registers.set_a(value);

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);
    }

    // Push d16 to the stack
    pub fn push_d16(&mut self, d16: u16) {
        let sp: u16 = self.registers.get_sp();
        self.write(sp - 1, (d16 >> 8) as u8);
        self.write(sp - 2, d16 as u8);
        self.registers.set_sp(sp - 2);
    }

    pub fn pop_d16(&mut self) -> u16 {
        let sp: u16 = self.registers.get_sp();
        let d16: u16 = self.read(sp) as u16 | ((self.read(sp + 1) as u16) << 8);
        self.registers.set_sp(sp + 2);
        d16
    }

    // Pop d16 off the stack into reg
    pub fn pop_d16_into(&mut self, reg: Reg16) {
        let d16: u16 = self.pop_d16();
        self.registers.set_reg_16(reg, d16);
    }

    // Reset bit b in reg
    pub fn res(&mut self, b: u8, reg: Reg8) {
        let mask: u8 = !(0x01 << b);
        let r = self.registers.get_reg_8(reg);
        self.registers.set_reg_8(reg, r & mask);
    }

    // Return
    pub fn ret(&mut self, condition: bool) {
        if condition {
            let a16: u16 = self.pop_d16();
            self.registers.set_pc(a16);
        }
    }

    // Call at offset address
    pub fn rst(&mut self, offset: u8) {
        let pc: u16 = self.registers.get_pc();
        self.push_d16(pc);
        self.registers.set_pc(offset as u16);
    }

    // Stop CPU and LCD until a button is pressed
    pub fn stop(&mut self) {
        // Stop CPU and LCD until a button is pressed
        return;
    }

    // Swap the upper and lower 4 bits
    pub fn swap(&mut self, reg: Reg8) {
        let mut r: u8 = self.registers.get_reg_8(reg);
        let r_bottom: u8 = r & 0x0F;
        r >>= 4;
        r |= r_bottom << 4;
        self.registers.set_reg_8(reg, r);
    }

    // XOR n with A => A
    pub fn xor(&mut self, n: u8) {
        let value: u8 = self.registers.get_a() ^ n;
        self.registers.set_a(value);

        self.registers.set_flag(Flag::Z, value == 0);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, false);
        self.registers.set_flag(Flag::C, false);
    }
}