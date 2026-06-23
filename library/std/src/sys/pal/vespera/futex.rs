// sys/pal/vespera/futex.rs

use crate::sync::atomic::Atomic;
use crate::time::Duration;
use crate::sys::pal::c;

pub type Futex = Atomic<Primitive>;
pub type Primitive = u32;
pub type SmallFutex = Atomic<SmallPrimitive>;
pub type SmallPrimitive = u32;

pub fn futex_wait(futex: &Atomic<u32>, expected: u32, timeout: Option<Duration>) -> bool {
    let ts = timeout.map(|d| c::timespec {
        tv_sec:  d.as_secs() as i64,
        tv_nsec: d.subsec_nanos() as i64,
    });

    let ret = unsafe {
        c::futex_wait(
            futex as *const Atomic<u32> as *const u32,
            expected,
            ts.as_ref().map_or(core::ptr::null(), |t| t as *const c::timespec),
        )
    };

    ret != -(c::ETIMEDOUT as i32)
}

pub fn futex_wake(futex: &Atomic<u32>) -> bool {
    unsafe {
        c::futex_wake(futex as *const Atomic<u32> as *mut u32, 1) > 0
    }
}

pub fn futex_wake_all(futex: &Atomic<u32>) {
    unsafe {
        c::futex_wake_all(futex as *const Atomic<u32> as *mut u32);
    }
}