// sys/stdio/vespera.rs

use crate::io;
use crate::sys::pal::c;

const HANDLE_TYPE_DEVICE: u64 = 0x7000000000000000;
const HANDLE_STDIN: u64 = HANDLE_TYPE_DEVICE | 0;
const HANDLE_STDOUT: u64 = HANDLE_TYPE_DEVICE | 1;
const HANDLE_STDERR: u64 = HANDLE_TYPE_DEVICE | 2;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            c::sys_read(HANDLE_STDIN, buf.as_mut_ptr().addr() as u64, buf.len() as u64, 0, 0, 0)
        };
        if ret < 0 {
            Err(io::Error::from_raw_os_error(-ret as i32))
        } else {
            Ok(ret as usize)
        }
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe {
            c::sys_write(HANDLE_STDOUT, buf.as_ptr().addr() as u64, buf.len() as u64, 0, 0, 0)
        };
        if ret < 0 {
            Err(io::Error::from_raw_os_error(-ret as i32))
        } else {
            Ok(ret as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe {
            c::sys_write(HANDLE_STDERR, buf.as_ptr().addr() as u64, buf.len() as u64, 0, 0, 0)
        };
        if ret < 0 {
            Err(io::Error::from_raw_os_error(-ret as i32))
        } else {
            Ok(ret as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = 8 * 1024;

pub fn is_ebadf(_err: &io::Error) -> bool {
    false
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}