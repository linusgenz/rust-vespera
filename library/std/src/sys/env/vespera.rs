// sys/env/vespera.rs

use crate::ffi::{CStr, CString, OsStr, OsString};
use crate::os::vespera::ffi::{OsStrExt, OsStringExt};
use crate::sync::Mutex;
use crate::sys::pal::c;
use crate::{fmt, io, vec};

static ENV_LOCK: Mutex<()> = Mutex::new(());

pub struct Env {
    iter: vec::IntoIter<(OsString, OsString)>,
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter.as_slice().fmt(f)
    }
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

pub fn env() -> Env {
    let _guard = ENV_LOCK.lock();
    let mut vec = vec::Vec::new();

    unsafe {
        let mut ptr = c::environ;
        if !ptr.is_null() {
            while !(*ptr).is_null() {
                let entry = CStr::from_ptr(*ptr).to_bytes();
                if let Some(eq) = entry.iter().position(|&b| b == b'=') {
                    let key = OsStr::from_bytes(&entry[..eq]).to_os_string();
                    let value = OsStr::from_bytes(&entry[eq + 1..]).to_os_string();
                    vec.push((key, value));
                }
                ptr = ptr.add(1);
            }
        }
    }

    Env { iter: vec.into_iter() }
}

pub fn getenv(key: &OsStr) -> Option<OsString> {
    let key_c = CString::new(key.as_bytes()).ok()?;
    let _guard = ENV_LOCK.lock();
    unsafe {
        let value_ptr = c::getenv(key_c.as_ptr());
        if value_ptr.is_null() {
            None
        } else {
            let bytes = CStr::from_ptr(value_ptr).to_bytes();
            Some(OsStr::from_bytes(bytes).to_os_string())
        }
    }
}

pub unsafe fn setenv(key: &OsStr, value: &OsStr) -> io::Result<()> {
    let key_c = CString::new(key.as_bytes())
        .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "env key contains a NUL byte"))?;
    let value_c = CString::new(value.as_bytes())
        .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "env value contains a NUL byte"))?;

    let _guard = ENV_LOCK.lock();
    let ret = unsafe { c::setenv(key_c.as_ptr(), value_c.as_ptr(), 1) };
    if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

pub unsafe fn unsetenv(key: &OsStr) -> io::Result<()> {
    let key_c = CString::new(key.as_bytes())
        .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "env key contains a NUL byte"))?;

    let _guard = ENV_LOCK.lock();
    let ret = unsafe { c::unsetenv(key_c.as_ptr()) };
    if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}