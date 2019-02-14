#[allow(dead_code)]
pub enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,
    Cond,
    Count,
}

const REG_COUNT: usize = Register::Count as usize;
pub static mut REG: &'static mut [u16] = &mut [0; REG_COUNT];

pub unsafe fn write(reg: Register, val: u16) {
    REG[reg as usize] = val;
}

pub unsafe fn write_raw(reg: u8, val: u16) {
    REG[reg as usize] = val;
}

pub unsafe fn read(reg: Register) -> u16 {
    REG[reg as usize]
}

pub unsafe fn read_raw(reg: u8) -> u16 {
    REG[reg as usize]
}

pub unsafe fn pc() -> u16 {
    REG[Register::PC as usize]
}

pub unsafe fn set_pc(new_pc: u16) {
    REG[Register::PC as usize] = new_pc;
}
