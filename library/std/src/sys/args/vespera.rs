// sys/args/vespera.rs

use crate::ffi::{CStr, OsString};
use crate::os::vespera::ffi::OsStringExt;
use crate::ptr;
use crate::sync::atomic::{AtomicIsize, AtomicPtr, Ordering};
use crate::{fmt, vec};

static ARGC: AtomicIsize = AtomicIsize::new(0);
static ARGV: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());

pub unsafe fn init(argc: isize, argv: *const *const u8) {
    ARGC.store(argc.max(0), Ordering::Relaxed);
    ARGV.store(argv as *mut *const u8, Ordering::Relaxed);
}

pub fn args() -> Args {
    let argc = ARGC.load(Ordering::Relaxed);
    let argv = ARGV.load(Ordering::Relaxed);

    let mut vec = vec::Vec::with_capacity(argc as usize);

    if !argv.is_null() {
        for i in 0..argc {
            unsafe {
                let ptr = *argv.offset(i) as *const i8;
                if ptr.is_null() {
                    break;
                }
                let bytes = CStr::from_ptr(ptr).to_bytes().to_vec();
                vec.push(OsString::from_vec(bytes));
            }
        }
    }

    Args { iter: vec.into_iter() }
}

pub struct Args {
    iter: vec::IntoIter<OsString>,
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter.as_slice().fmt(f)
    }
}

impl Iterator for Args {
    type Item = OsString;

    fn next(&mut self) -> Option<OsString> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        self.iter.next_back()
    }
}