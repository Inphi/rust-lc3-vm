#[cfg(not(windows))]
extern crate mio;

#[cfg(windows)]
extern crate winapi;

extern crate byteorder;

mod instruction;
mod register;
mod trap;
mod vm;

use self::vm::VM;
use std::env;
use std::fs;

fn run() {
    let program = env::args().skip(1).next().unwrap();
    println!("loading program at {}", program);

    let mut file = fs::File::open(&program).unwrap();
    let mut vm = VM::new();
    vm.load_image(&mut file);
    vm.start();

    println!("shutting down");
}

fn main() {
    run();
}
