use std::fs::File;
use std::io::Read;

mod registers;
use registers::*;

pub struct Gameboy {
    memory: [u8; 0x10000],
    pub registers: Registers,
}

impl Gameboy {
    pub fn new() -> Gameboy {
        Gameboy {
            memory: [0u8; 0x10000],
            registers: Registers::new(),
        }
    }

    pub fn load_rom(&mut self, rom_name: String) -> Result<(), std::io::Error> {
        let mut rom_file = File::open(rom_name)?;
        let mut bytes = Vec::new();
        rom_file.read_to_end(&mut bytes)?;
        for (i, b) in bytes.into_iter().enumerate() {
            if i < 0x4000 {
                self.memory[i] = b
            } else {
                break;
            }

        }
        Ok(())
    }

    // Debug / Messy
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

    pub fn step(&mut self) -> bool {
        if cfg!(debug_assertions) {
            println!("Before {:?}", self.registers);
        }

        let instruction = self.get_at_pc_incr();
        static mut step_count: u64 = 0;

        if cfg!(debug_assertions) {
            println!("Step: {}\nInstruction: 0x{:02X}", unsafe{ step_count }, instruction);
        }

        unsafe { step_count += 1 };

        match instruction {
            0x00 => {},

            0x18 => self.jr(true),

            0x28 => self.jr(self.registers.get_flag(Flag::Z) == true),

            0x3e => {
                let d8: u8 = self.get_at_pc_incr();
                self.registers.set_a(d8);
            }

            0xc3 => self.jmp(),

            0xaf => {
                let a: u8 = self.registers.get_a();
                self.xor(a);
            }

            0xe0 => {
                // (0xFF00 + n) <= A
                let offset = self.get_at_pc_incr();
                let a: u8 = self.registers.get_a();
                self.memory[0xFF00 + offset as usize] = a;
            }

            0xea => {
                // (nn) <= A
                // ROM CHECK
                let addr: u16 = self.get_at_pc_incr() as u16 | ((self.get_at_pc_incr() as u16) << 8);
                let a: u8 = self.registers.get_a();
                self.memory[addr as usize] = a;
            }

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

    // Compare
    pub fn cp(&mut self, n: u8) {
        let a: u8 = self.registers.get_a();

        self.registers.set_flag(Flag::Z, a == n);
        self.registers.set_flag(Flag::N, true);
        self.registers.set_flag(Flag::H, (a & 0x0F).checked_sub(n & 0x0F) == None);
        self.registers.set_flag(Flag::C, a < n);
    }

    // Enable and disable interrupts
    pub fn interrupts_enabled(&mut self, enabled: bool) {
        self.memory[0xffff] = enabled as u8;
    }

    // Jump Unconditional
    pub fn jmp(&mut self) {
        let addr: u16 = self.get_at_pc_incr() as u16 | ((self.get_at_pc_incr() as u16) << 8);
        self.registers.set_pc(addr);
    }

    // Jump Relative Conditional
    pub fn jr(&mut self, condition: bool) {
        let offset: u8 = self.get_at_pc_incr();

        if condition {
            self.registers.set_pc(self.registers.get_pc() + offset as u16 - 1);  // -1 added for pc change to get relative values
        }
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