#[path = "../unsupported/common.rs"]
mod unsupported_common;

pub mod c;
pub mod util;
pub mod time;

pub unsafe fn init(argc: isize, argv: *const *const u8, _sigpipe: u8) {
    unsafe {
        crate::sys::args::init(argc, argv);
    }
}

pub unsafe fn cleanup() {}

pub fn abort_internal() -> ! {
    unsafe { c::abort() }
}