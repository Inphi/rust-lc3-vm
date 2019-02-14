use self::register::Register;
use crate::instruction;
use crate::instruction::Op;
use crate::register;
use mio::*;
use std::io;
use std::io::Read;
use std::time::Duration;
use byteorder::{BigEndian, ReadBytesExt};

const MEM_SIZ: usize = 1 << 16;
const REG_COUNT: usize = Register::Count as usize;
const PC_START: u16 = 0x3000;

const KBSR: u16 = 0xFE00; // memory-mapped keyboard status register
const KBDR: u16 = 0xFE02; // memory-mapped keyboard data register
const DPSR: u16 = 0xFE04; // memory-mapped display status register
const DPDR: u16 = 0xFE06; // memory-mapped display data register
const MCCR: u16 = 0xFFFE; // memory-mapped machine control register

const STDIN: Token = Token(0);

pub struct VM {
    pub mem: [u16; MEM_SIZ],
    pub reg: [u16; REG_COUNT],
    pub prompt: &'static str,
}

impl VM {
    pub fn new() -> VM {
        VM {
            mem: [0; MEM_SIZ],
            reg: [0; REG_COUNT],
            prompt: "_",
        }
    }

    pub unsafe fn mem_read(&mut self, addr: u16) -> u16 {
        if addr == KBSR as u16 {
            if self.check_key() {
                self.mem[KBSR as usize] = 1 << 15;
                self.mem[KBDR as usize] = self.getchar() as u16;
            } else {
                self.mem[KBSR as usize] = 0;
            }
        }

        self.mem[addr as usize]
    }

    pub unsafe fn mem_write(&mut self, val: u16, addr: u16) {
        self.mem[addr as usize] = val;
    }

    #[cfg(target_family = "unix")]
    fn getchar(&self) -> u8 {
        unimplemented!();
    }

    #[cfg(target_family = "windows")]
    fn getchar(&self) -> u8 {}

    #[cfg(target_family = "unix")]
    fn check_key(&self) -> bool {
        // TODO: use libc instead of mio
        use mio::unix::EventedFd;

        let stdin = 0;
        let poll = Poll::new().unwrap();
        let ev_fd = EventedFd(&stdin);
        poll.register(&ev_fd, STDIN, Ready::readable(), PollOpt::edge())
            .unwrap();
        let mut events = Events::with_capacity(1);

        poll.poll(&mut events, Some(Duration::new(0, 0))).unwrap();
        for event in &events {
            if event.readiness().is_readable() {
                return true;
            }
        }
        return false;
    }

    #[cfg(target_family = "windows")]
    fn check_key(&self) -> bool {
        use std::mem;
        use std::ptr::null_mut;
        use winapi::shared::minwindef::*;
        use winapi::um::consoleapi::ReadConsoleInputA;
        use winapi::um::errhandlingapi::*;
        use winapi::um::handleapi::*;
        use winapi::um::processenv::*;
        use winapi::um::winbase::*;
        use winapi::um::wincon::*;

        unsafe {
            let handle = CreateFileA(
                b"CONIN$\0".as_ptr() as *const i8,
                GENERIC_READ,
                FILE_SHARE_READ,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            );
            if handle == INVALID_HANDLE_VALUE {
                panic!("invalid stdin handle error_code={}", GetLastError());
            }
            if handle.is_null() {
                panic("null handle");
            }

            let mut record: INPUT_RECORD = mem::zeroed();
            let mut nr_read: DWORD = mem::zeroed();
            let val = ReadConsoleInputA(
                handle,
                &mut record as PINPUT_RECORD,
                1,
                &mut nr_read as LPDWORD,
            );

            if val == 0 {
                panic!("error peeking console input");
            }
            assert_eq!(val, 1);

            record.EventType == KEY_EVENT && record.Event.KeyEvent().bKeyDown == 1
        }
    }

    pub fn read(&self, r: Register) -> u16 {
        self.reg[r as usize]
    }

    pub fn rwrite(&mut self, r: Register, val: u16) {
        self.reg[r as usize] = val
    }

    pub fn load_image<T: Read>(&mut self, image: &mut T) {
        let origin = image.read_u16::<BigEndian>().unwrap();

        for x in self.mem.iter_mut().skip(origin as usize) {
            let val = match image.read_u16::<BigEndian>() {
                Ok(val) => val,
                Err(err) => {
                    if err.kind() == io::ErrorKind::UnexpectedEof {
                        return;
                    } else {
                        panic!(err);
                    }
                }
            };
            *x = val;
        }
    }

    pub fn start(&mut self) {
        unsafe {
            register::set_pc(PC_START);
        }

        loop {
            // fetch and incr pc
            let instr = unsafe {
                let pc = self.reg[Register::PC as usize];
                let instr = self.mem_read(pc);
                self.reg[Register::PC as usize] += 1;
                instr
            };

            let op: Op = unsafe { ::std::mem::transmute(instr >> 12) };

            match op {
                Op::Add => {
                    instruction::add(instr);
                }
                Op::And => {
                    instruction::and(instr);
                }
                Op::Not => {
                    instruction::not(instr);
                }
                Op::Br => {
                    instruction::br(instr);
                }
                Op::Jmp => {
                    instruction::jmp(instr);
                }
                Op::Jsr => {
                    instruction::jsr(instr);
                }
                Op::Ld => {
                    instruction::ld(self, instr);
                }
                Op::Ldi => {
                    instruction::ldi(self, instr);
                }
                Op::Ldr => {
                    instruction::ldr(self, instr);
                }
                Op::Lea => {
                    instruction::lea(instr);
                }
                Op::St => {
                    instruction::st(self, instr);
                }
                Op::Sti => {
                    instruction::sti(self, instr);
                }
                Op::Str => {
                    instruction::str(self, instr);
                }
                Op::Trap => {
                    let halt = instruction::trap(self, instr);
                    if halt {
                        break;
                    }
                }
                Op::Res | Op::Rti => panic!("bad opcode"),
            }
        }

        println!("good bye");
    }
}
