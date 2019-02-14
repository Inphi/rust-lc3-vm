use crate::register;
use crate::register::Register;
use crate::vm::VM;
use std::io::{self, Read, Write};

#[allow(dead_code)]
#[repr(u16)]
pub enum TrapInstr {
    GetC = 0x20,
    Out,
    Puts,
    In,
    PutsP,
    Halt,
}

pub fn puts(vm: &VM) {
    unsafe {
        let offset = register::read(Register::R0);
        let iter = vm
            .mem
            .iter()
            .skip(offset as usize)
            .take_while(|x| **x != ('\0' as u16));

        for c in iter {
            print!("{}", *c as u8);
        }
        io::stdout().flush().unwrap();
    }
}

pub fn getc(_vm: &VM) {
    let mut buf: [u8; 1] = [0; 1];
    io::stdin().read_exact(&mut buf).unwrap();
    unsafe { register::write(Register::R0, buf[0] as u16) };
}

pub fn out(_vm: &VM) {
    let ch = unsafe {
        let r0 = register::read(Register::R0);
        (r0 & 0x00FF) as u8
    };
    print!("{}", ch);
    io::stdout().flush().unwrap();
}

pub fn in_t(vm: &VM) {
    print!("{}", vm.prompt);
    let mut buf: [u8; 1] = [0; 1];
    io::stdin().read_exact(&mut buf).unwrap();
    unsafe { register::write(Register::R0, buf[0] as u16) };

    print!("{}", buf[0]);
    io::stdout().flush().unwrap();
}

pub fn putsp(vm: &VM) {
    let addr = unsafe { register::read(Register::R0) };
    let iter = vm
        .mem
        .iter()
        .skip(addr as usize)
        .take_while(|x| (**x >> 8) == ('\0' as u16));

    for x in iter {
        print!("{}{}", *x as u8, (*x & 0xFF00) as u8);
    }
    io::stdout().flush().unwrap();
}
