use std::fs::File;
use std::io::Read;

mod registers;
mod window;

use registers::*;
use window::*;

pub struct Gameboy {
    memory: [u8; 0x10000],
    registers: Registers,
    pub window: SdlWindow,
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

    // Debug / Messy
    #[cfg(DebugAssertions)]
    pub fn print_memory(&self) {
        for i in 0x0000..0x5000 {
            print!("{:02X} ", self.memory[i]);
            if i % 0x20 == 0x1f {
                println!("");
            }
        }
    }

    // Get u8 at pc location and increment
    pub fn get_at_pc_incr(&mut self) -> u8 {
        let value: u8 = self.memory[self.registers.get_pc() as usize];
        self.registers.set_pc(&self.registers.get_pc() + 1);
        value
    }

    pub fn get_next_16(&mut self) -> u16 {
        self.get_at_pc_incr() as u16 | ((self.get_at_pc_incr() as u16) << 8)
    }

    pub fn step(&mut self) -> bool {
        if !self.window.event_loop() {
            return false;
        }
        self.window.display_loop();

        if cfg!(debug_assertions) {
            println!("Before {:?}", self.registers);
        }

        let instruction = self.get_at_pc_incr();
        static mut STEP_COUNT: u64 = 0;

        if cfg!(debug_assertions) {
            println!("Step: {}\nInstruction: 0x{:02X}", unsafe{ STEP_COUNT }, instruction);
        }

        unsafe { STEP_COUNT += 1 };

        match instruction {
            // 0x
            0x00 => {},
            0x01 => self.ld_d16(Reg16::BC),

            0x04 => self.inc_8(Reg8::B),
            0x05 => self.dec_8(Reg8::B),
            0x06 => {
                let d8: u8 = self.get_at_pc_incr();
                self.registers.set_c(d8);
            }
            0x07 => {
                let mut a: u8 = self.registers.get_a();

                self.registers.set_flag(Flag::C, a & 0x80 == 0x80);
                self.registers.set_a(a << 1);
            }
            0x08 => {
                let sp: u16 = self.registers.get_sp();
                let a16: u16 = self.get_next_16();
                self.memory[a16 as usize] = (sp >> 8) as u8;
                self.memory[a16 as usize + 1] = sp as u8;
            }

            0x09 => self.add_hl(Reg16::BC),

            0x0c => self.inc_8(Reg8::C),
            0x0d => self.dec_8(Reg8::C),
            0x0e => {
                let d8: u8 = self.get_at_pc_incr();
                self.registers.set_c(d8);
            }

            // 1x
            0x10 => self.stop(),
            0x11 => self.ld_d16(Reg16::DE),

            0x14 => self.inc_8(Reg8::D),
            0x15 => self.dec_8(Reg8::D),
            0x16 => {
                let d8: u8 = self.get_at_pc_incr();
                self.registers.set_d(d8);
            }

            0x18 => self.jr(true),
            0x19 => self.add_hl(Reg16::DE),

            0x1c => self.inc_8(Reg8::E),
            0x1d => self.dec_8(Reg8::E),

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

            0x24 => self.inc_8(Reg8::H),
            0x25 => self.dec_8(Reg8::H),

            0x28 => self.jr(self.registers.get_flag(Flag::Z) == true),
            0x29 => self.add_hl(Reg16::HL),

            0x2c => self.inc_8(Reg8::L),
            0x2d => self.inc_8(Reg8::L),

            // 3x
            0x31 => self.ld_d16(Reg16::SP),
            0x32 => {
                let a: u8 = self.registers.get_a();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = a;
                self.registers.set_hl(hl - 1);
            }

            0x39 => self.add_hl(Reg16::SP),

            0x3c => self.inc_8(Reg8::A),
            0x3d => self.dec_8(Reg8::A),

            0x3e => {
                let d8: u8 = self.get_at_pc_incr();
                self.registers.set_a(d8);
            }

            // 7x
            0x70 => {
                let b: u8 = self.registers.get_b();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = b;
            }
            0x71 => {
                let c: u8 = self.registers.get_c();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = c;
            }
            0x72 => {
                let d: u8 = self.registers.get_d();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = d;
            }
            0x73 => {
                let e: u8 = self.registers.get_e();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = e;
            }
            0x74 => {
                let h: u8 = self.registers.get_h();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = h;
            }
            0x75 => {
                let l: u8 = self.registers.get_l();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = l;
            }
            0x77 => {
                let a: u8 = self.registers.get_a();
                let hl: u16 = self.registers.get_hl();
                self.memory[hl as usize] = a;
            }

            0x7b => {
                let e: u8 = self.registers.get_e();
                self.registers.set_a(e);
            }

            // ax
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
                let d8: u8 = self.memory[hl as usize];
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
                let d8: u8 = self.memory[hl as usize];
                self.cp(d8);
            }
            0xbf => {
                let a = self.registers.get_a();
                self.cp(a);
            }

            // cx
            0xc2 => self.jmp(self.registers.get_flag(Flag::Z) == false),
            0xc3 => self.jmp(true),

            0xca => self.jmp(self.registers.get_flag(Flag::Z) == true),

            // dx
            0xd2 => self.jmp(self.registers.get_flag(Flag::C) == false),
            0xda => self.jmp(self.registers.get_flag(Flag::C) == true),

            // ex
            0xe0 => {
                // (0xFF00 + n) <= A
                let offset = self.get_at_pc_incr();
                let a: u8 = self.registers.get_a();
                self.memory[0xFF00 + offset as usize] = a;
            }

            0xea => {
                // (nn) <= A
                // ROM CHECK
                let addr: u16 = self.get_next_16();
                let a: u8 = self.registers.get_a();
                self.memory[addr as usize] = a;
            }

            // fx
            0xf3 => self.interrupts_enabled(false),

            0xfe => {
                let d8: u8 = self.get_at_pc_incr();
                self.cp(d8);
            },

            _ => {
                println!("0x{:02X} Not implemented", instruction);
                return false;
            },
        }

        if cfg!(debug_assertions) {
            println!("After {:?}", self.registers);
            println!("");
        }

        true
    }

    // ---Generalized instruction implementations---

    // Add HL += n
    pub fn add_hl(&mut self, reg: Reg16) {
        let n: u16 = self.registers.get_reg_16(reg);
        let hl: u16 = self.registers.get_hl();

        let addition = hl.overflowing_add(n);
        self.registers.set_hl(addition.0);

        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, (hl & 0x0fff).overflowing_add(n & 0x0fff).1);
        self.registers.set_flag(Flag::C, addition.1);
    }

    // Compare
    pub fn cp(&mut self, n: u8) {
        let a: u8 = self.registers.get_a();

        self.registers.set_flag(Flag::Z, a == n);
        self.registers.set_flag(Flag::N, true);
        self.registers.set_flag(Flag::H, (a & 0x0F).overflowing_sub(n & 0x0F).1);
        self.registers.set_flag(Flag::C, a < n);
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

    // Increment an 8bit register
    pub fn inc_8(&mut self, reg: Reg8) {
        let mut value: u8 = self.registers.get_reg_8(reg);

        self.registers.set_flag(Flag::H, (value & 0x0F).overflowing_add(1).1);

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
        self.memory[0xffff] = enabled as u8;
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
            self.registers.set_pc(self.registers.get_pc() + offset as u16 - 1);  // -1 added for pc change to get relative values
        }
    }

    // LD reg <- d16
    pub fn ld_d16(&mut self, reg: Reg16) {
        let d16: u16 = self.get_next_16();
        self.registers.set_reg_16(reg, d16);
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

    pub fn stop(&mut self) {
        // Stop CPU and LCD until a button is pressed
        return;
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