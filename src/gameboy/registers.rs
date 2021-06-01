use std::fmt;

// Flags values = offset into f register
pub enum Flag {
    Z = 7,
    N = 6,
    H = 5,
    C = 4,
}

#[derive(Copy, Clone)]
struct Split {
    pub bot: u8,
    pub top: u8,
}

#[derive(Copy, Clone)]
union Register {
    pub half: Split,
    pub all: u16,
}

#[derive(Copy, Clone)]
pub struct Registers {
    af: Register,
    bc: Register,
    de: Register,
    hl: Register,
    sp: u16,
    pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            af: Register {all: 0u16},
            bc: Register {all: 0u16},
            de: Register {all: 0u16},
            hl: Register {all: 0u16},
            sp: 0u16,
            pc: 0x0100,
        }
    }

    // AF
    pub fn get_a(&self) -> u8 {
        unsafe { self.af.half.top }
    }
    pub fn set_a(&mut self, a: u8) {
        self.af.half.top = a
    }

    pub fn get_f(&self) -> u8 {
        unsafe { self.af.half.bot }
    }
    pub fn set_f(&mut self, f: u8) {
        self.af.half.bot = f
    }

    pub fn get_af(&self) -> u16 {
        unsafe { self.af.all }
    }
    pub fn set_af(&mut self, af: u16) {
        self.af.all = af
    }

    // BC
    pub fn get_b(&self) -> u8 {
        unsafe { self.bc.half.top }
    }
    pub fn set_b(&mut self, b: u8) {
        self.bc.half.top = b
    }

    pub fn get_c(&self) -> u8 {
        unsafe { self.bc.half.bot }
    }
    pub fn set_c(&mut self, c: u8) {
        self.bc.half.bot = c
    }

    pub fn get_bc(&self) -> u16 {
        unsafe { self.bc.all }
    }
    pub fn set_bc(&mut self, bc: u16) {
        self.bc.all = bc
    }

    //DE
    pub fn get_d(&self) -> u8 {
        unsafe { self.de.half.top }
    }
    pub fn set_d(&mut self, d: u8) {
        self.de.half.top = d
    }

    pub fn get_e(&self) -> u8 {
        unsafe { self.de.half.bot }
    }
    pub fn set_e(&mut self, e: u8) {
        self.de.half.bot = e
    }

    pub fn get_de(&self) -> u16 {
        unsafe { self.de.all }
    }
    pub fn set_de(&mut self, de: u16) {
        self.de.all = de
    }

    // HL
    pub fn get_h(&self) -> u8 {
        unsafe { self.hl.half.top }
    }
    pub fn set_h(&mut self, h: u8) {
        self.hl.half.top = h
    }

    pub fn get_l(&self) -> u8 {
        unsafe { self.hl.half.bot }
    }
    pub fn set_l(&mut self, l: u8) {
        self.hl.half.bot = l
    }

    pub fn get_hl(&self) -> u16 {
        unsafe { self.hl.all }
    }
    pub fn set_hl(&mut self, hl: u16) {
        self.hl.all = hl
    }

    // SP
    pub fn get_sp(&self) -> u16 {
        self.sp
    }
    pub fn set_sp(&mut self, sp: u16) {
        self.sp = sp;
    }

    // PC
    pub fn get_pc(&self) -> u16 {
        self.pc
    }
    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    // Flags
    pub fn set_flag(&mut self, flag: Flag, set: bool) {
        let f: u8 = self.get_f();
        let mut flag_set: u8 = if set { 0x1 } else { 0x0 };
        flag_set <<= flag as u8;
        if set {
            self.set_f(f | flag_set);
        } else {
            self.set_f(f & !flag_set);
        }
    }
    pub fn get_flag(&self, flag: Flag) -> bool {
        if (self.get_f() >> (flag as u8)) & 0x1 == 0x1 {
            true
        } else {
            false
        }
    }

}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registers")
         .field("AF", &format_args!("0x{:04X}", unsafe { &self.af.all} ))
         .field("BC", &format_args!("0x{:04X}", unsafe { &self.bc.all} ))
         .field("DE", &format_args!("0x{:04X}", unsafe { &self.de.all} ))
         .field("HL", &format_args!("0x{:04X}", unsafe { &self.hl.all} ))
         .field("SP", &format_args!("0x{:04X}", &self.sp ))
         .field("PC", &format_args!("0x{:04X}", &self.pc ))
         .finish()
    }
}
