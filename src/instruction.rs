use crate::register;
use crate::register::Register;
use crate::trap;
use crate::trap::TrapInstr;
use crate::vm::VM;

#[allow(dead_code)]
#[repr(u16)]
pub enum Op {
    Br = 0,
    Add,
    Ld,
    St,
    Jsr,
    And,
    Ldr,
    Str,
    Rti,
    Not,
    Ldi,
    Sti,
    Jmp,
    Res,
    Lea,
    Trap,
}

#[repr(u16)]
pub enum CondFlags {
    Pos = 1 << 0,
    Zero = 1 << 1,
    Neg = 1 << 2,
}

fn sign_extend(x: u16, bit_count: i32) -> u16 {
    let z = x >> (bit_count - 1) & 1;
    if z != 0 {
        x | (0xFFFF << bit_count)
    } else {
        x
    }
}

unsafe fn update_flags(r: u8) {
    let x = register::read_raw(r);
    if x == 0 {
        register::write(Register::Cond, CondFlags::Zero as u16);
    } else if x >> 15 != 0 {
        register::write(Register::Cond, CondFlags::Neg as u16);
    } else {
        register::write(Register::Cond, CondFlags::Pos as u16);
    }
}

pub fn add(instr: u16) {
    let r0 = ((instr >> 9) & 0x7) as u8;
    let r1 = ((instr >> 6) & 0x7) as u8;

    let imm_flag = (instr >> 5) & 0x1;

    if imm_flag != 0 {
        let imm5 = sign_extend(instr & 0x1F, 5);
        unsafe {
            register::write_raw(r0, register::read_raw(r1) + imm5);
        };
    } else {
        let r2 = instr & 0x7;
        unsafe {
            register::write_raw(r0, register::read_raw(r1) + r2);
        };
    }

    unsafe {
        update_flags(r0);
    }
}

pub fn and(instr: u16) {
    let r0 = ((instr >> 9) & 0x7) as u8;
    let r1 = ((instr >> 6) & 0x7) as u8;

    let imm_flag = (instr >> 5) & 0x1;

    if imm_flag != 0 {
        let imm5 = sign_extend(instr & 0x1F, 5);
        unsafe {
            register::write_raw(r0, register::read_raw(r1) & imm5);
        };
    } else {
        let r2 = instr & 0x7;
        unsafe {
            register::write_raw(r0, register::read_raw(r1) & r2);
        };
    }

    unsafe {
        update_flags(r0);
    }
}

pub fn br(instr: u16) {
    let cond_flag = (instr >> 9) & 0x7;

    unsafe {
        if (cond_flag & register::read(Register::Cond)) != 0 {
            let pc = register::read(Register::PC);
            let pc_offset = instr & 0x1FF;
            register::write(Register::PC, pc + sign_extend(pc_offset, 9));
        }
    }
}

pub fn jmp(instr: u16) {
    let base_reg = ((instr >> 6) & 0x7) as u8;

    unsafe {
        if base_reg != 0x7 {
            register::set_pc(register::read_raw(base_reg));
        } else {
            // jmp
            register::set_pc(register::read(Register::R7));
        }
    }
}

pub fn jsr(instr: u16) {
    unsafe {
        let pc = register::pc();
        register::write(Register::R7, pc);

        let flag = ((instr >> 11) & 0x1) as u8;
        if flag == 0 {
            let base_reg = ((instr >> 6) & 0x7) as u8;
            register::set_pc(base_reg as u16);
        } else {
            let pc_offset = instr & 0x3FF;
            register::set_pc(pc + sign_extend(pc_offset, 11));
        }
    }
}

pub fn ld(vm: &mut VM, instr: u16) {
    let dr = ((instr >> 0x9) & 0x3) as u8;
    unsafe {
        let pc = register::pc();
        let pc_offset = sign_extend(instr & 0x1FF, 9);
        let val = vm.mem_read(pc + pc_offset);
        register::write_raw(dr, val);
        update_flags(dr);
    }
}

pub fn ldi(vm: &mut VM, instr: u16) {
    let dr = ((instr >> 0x9) & 0x7) as u8;
    let pc_offset = sign_extend(instr & 0x1FF, 9);

    unsafe {
        let pc = register::pc();
        let addr = vm.mem_read(pc + pc_offset);
        register::write_raw(dr, vm.mem_read(addr));
        update_flags(dr);
    }
}

pub fn ldr(vm: &mut VM, instr: u16) {
    let dr = ((instr >> 9) & 0x7) as u8;
    let base_reg = ((instr >> 6) & 0x7) as u8;
    let offset = instr & 0x3F;

    unsafe {
        let base_addr = register::read_raw(base_reg);
        register::write_raw(dr, vm.mem_read(base_addr + sign_extend(offset, 6)));
        update_flags(dr);
    }
}

pub fn lea(instr: u16) {
    let dr = ((instr >> 9) & 0x7) as u8;
    let pc_offset = instr & 0x1FF;

    unsafe {
        register::write_raw(dr, register::pc() + sign_extend(pc_offset, 9));
        update_flags(dr);
    }
}

pub fn not(instr: u16) {
    let sr = ((instr >> 6) & 0x3) as u8;
    let dr = ((instr >> 9) & 0x3) as u8;

    unsafe {
        register::write_raw(dr, !sr as u16);
        update_flags(dr);
    }
}

#[allow(dead_code)]
pub fn ret(_instr: u16) {
    unsafe {
        register::set_pc(register::read(Register::R7));
    }
}

pub fn st(vm: &mut VM, instr: u16) {
    let sr = ((instr >> 9) & 0x3) as u8;
    let pc_offset = instr & 0x1FF;

    unsafe {
        vm.mem_write(
            register::pc() + sign_extend(pc_offset, 9),
            register::read_raw(sr),
        );
    }
}

pub fn sti(vm: &mut VM, instr: u16) {
    let sr = ((instr >> 9) & 0x3) as u8;
    let pc_offset = instr & 0x1FF;

    unsafe {
        let addr = vm.mem_read(register::pc() + sign_extend(pc_offset, 9));
        vm.mem_write(addr, register::read_raw(sr));
    }
}

pub fn str(vm: &mut VM, instr: u16) {
    let base_reg = ((instr >> 6) & 0x3) as u8;
    let sr = ((instr >> 9) & 0x3) as u8;
    let offset = instr & 0x3F;

    unsafe {
        let addr = register::read_raw(base_reg) + sign_extend(offset, 6);
        vm.mem_write(addr, register::read_raw(sr));
    }
}

pub fn trap(vm: &mut VM, instr: u16) -> bool {
    unsafe {
        register::write(Register::R7, register::pc());
        let trap_vector = instr & 0xFF;
        let trap_instr: TrapInstr = ::std::mem::transmute(trap_vector);

        match trap_instr {
            TrapInstr::Puts => {
                trap::puts(vm);
            }
            TrapInstr::GetC => {
                trap::getc(vm);
            }
            TrapInstr::Out => {
                trap::out(vm);
            }
            TrapInstr::In => {
                trap::putsp(vm);
            }
            TrapInstr::PutsP => {
                trap::putsp(vm);
            }
            TrapInstr::Halt => {
                return true;
            }
        }

        register::set_pc(vm.mem_read(trap_vector));
        return false;
    }
}
