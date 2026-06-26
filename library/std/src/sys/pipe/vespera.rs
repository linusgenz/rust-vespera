// pipe/vespera.rs

use crate::io;
use crate::sys::fd::FileDesc;
use crate::sys::pal::c;

pub type Pipe = FileDesc;

/// Creates a unidirectional pipe using `sys_pipe`.
///
/// Returns `(read_end, write_end)`. Both ends share a
/// `Channel` (ring buffer, currently 64 KiB in size) at the kernel level—the syscall
/// itself creates two handles on the same channel (one with `CAP_READ`,
/// one with `CAP_WRITE`):
///
/// ```text
/// i64 sys_pipe(u64 arg0, ...) {
///     auto* hdls = reinterpret_cast<i64*>(arg0);
///     // hdls[0] = read handle, hdls[1] = write handle
/// }
/// ```

#[inline]
pub fn pipe() -> io::Result<(Pipe, Pipe)> {
    let mut handles: [i64; 2] = [0; 2];
    let ret = unsafe { c::sys_pipe(handles.as_mut_ptr().addr() as u64, 0, 0, 0, 0, 0) };
    if ret < 0 {
        return Err(io::Error::from_raw_os_error(-ret as i32));
    }

    unsafe {
        let read_end = FileDesc::from_raw_handle(handles[0] as c::HANDLE);
        let write_end = FileDesc::from_raw_handle(handles[1] as c::HANDLE);
        Ok((read_end, write_end))
    }
}
