// thread/vespera.rs

use crate::io;
use crate::mem::ManuallyDrop;
use crate::num::NonZero;
use crate::ffi::{CStr, c_void};
use crate::thread::{ThreadInit};
use crate::time::Duration;
use crate::sys::pal::c;

pub const DEFAULT_MIN_STACK_SIZE: usize = 2 * 1024 * 1024; // 2 MiB

pub struct Thread {
    id: c::UnitID,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    pub unsafe fn new(stack: usize, init: Box<ThreadInit>) -> io::Result<Thread> {
        extern "C" fn thread_start(arg: u64) {
            unsafe {
                let init = Box::from_raw(core::ptr::with_exposed_provenance_mut::<ThreadInit>(arg as usize));
                let rust_start = init.init();
                rust_start();
            }
        }

        let data = Box::into_raw(init);

        let realm_id = current_realm_id();

        let entry: u64 = unsafe { core::mem::transmute(thread_start as *const ()) };

        let unit_id = unsafe {
            c::spawn_unit(
                realm_id,
                entry,
                data.expose_provenance() as u64,
                stack as u64,
            )
        };

        if (unit_id as i64) < 0 {
            drop(unsafe { Box::from_raw(data) });
            return Err(io::Error::from_raw_os_error(-(unit_id as i64) as i32));
        }

        Ok(Thread { id: unit_id })
    }

    pub fn join(self) {
        let id = ManuallyDrop::new(self).id;
        let mut exit_code: i64 = 0;
        let ret = unsafe { c::join_unit(id, &mut exit_code) };
        assert!(ret == 0, "join_unit failed: {} tried to get unit id {:?}", io::Error::from_raw_os_error(-ret as i32), id);
    }

    pub fn id(&self) -> c::UnitID {
        self.id
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        // Wenn join() nicht aufgerufen wurde, detachen wir die Unit.
        // VesperaOS hat kein explizites detach-syscall — eine Unit die
        // niemand joined wird beim Realm-Teardown automatisch aufgeräumt.
        // Hier nichts zu tun; kein Leak.
    }
}

pub fn available_parallelism() -> io::Result<NonZero<usize>> {
    unsafe {
        let path = b"/dev/cpuinfo\0";

        let handle = c::open(path.as_ptr() as *const _, 0);
        if (handle as isize) < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut info = core::mem::MaybeUninit::<c::cpu_info>::uninit();

        let ret = c::read(
            handle,
            info.as_mut_ptr() as *mut c_void,
            core::mem::size_of::<c::cpu_info>(),
        );

        c::close(handle);

        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        if ret as usize != core::mem::size_of::<c::cpu_info>() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "short read"));
        }

        let info = info.assume_init();

        let cores = info.cores as usize;

        NonZero::new(cores)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "zero cores"))
    }
}

pub fn current_os_id() -> Option<u64> {
    Some(unsafe { c::get_unit_id() })
}

pub fn yield_now() {
    unsafe { c::sched_yield(); }
}

pub fn set_name(_name: &CStr) {
    // TODO: sys_set_unit_name Syscall implementieren wenn gewünscht

}

pub fn sleep(dur: Duration) {
    use crate::sys::pal::c::timespec;

    let ts = timespec {
        tv_sec:  dur.as_secs() as i64,
        tv_nsec: dur.subsec_nanos() as i64,
    };

    unsafe {
        c::nanosleep(&ts, core::ptr::null_mut());
    }
}

fn current_realm_id() -> c::RealmID {
    unsafe { c::get_realm_id() }
}