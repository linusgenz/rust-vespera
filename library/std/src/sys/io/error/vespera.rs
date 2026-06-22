// sys/io/error/vespera.rs

use crate::io;
use crate::sys::pal::c;

#[inline]
pub fn errno() -> i32 {
    unsafe { c::errno }
}

#[inline]
#[allow(dead_code)]
pub fn set_errno(e: i32) {
    unsafe {
        c::errno = e;
    }
}

#[inline]
pub fn is_interrupted(errno: i32) -> bool {
    errno == c::EINTR
}

pub fn decode_error_kind(errno: i32) -> io::ErrorKind {
    use io::ErrorKind::*;
    match errno {
        c::E2BIG => ArgumentListTooLong,
        c::EBUSY => ResourceBusy,
        c::EDEADLOCK => Deadlock,
        c::EEXIST => AlreadyExists,
        c::EFBIG => FileTooLarge,
        c::EINTR => Interrupted,
        c::EINVAL => InvalidInput,
        c::EISDIR => IsADirectory,
        c::ELOOP => FilesystemLoop,
        c::ENOENT => NotFound,
        c::ENOMEM => OutOfMemory,
        c::ENOSPC => StorageFull,
        c::ENOSYS => Unsupported,
        c::EMLINK => TooManyLinks,
        c::ENAMETOOLONG => InvalidFilename,
        c::ENOTDIR => NotADirectory,
        c::ENOTEMPTY => DirectoryNotEmpty,
        c::EPIPE => BrokenPipe,
        c::EROFS => ReadOnlyFilesystem,
        c::ESPIPE => NotSeekable,
        c::ETXTBSY => ExecutableFileBusy,
        c::EXDEV => CrossesDevices,
        c::EUNSUPPORTED => Unsupported,
        c::EBADH => InvalidInput,
        c::EACCES | c::EPERM => PermissionDenied,
        x if x == c::EAGAIN || x == c::EWOULDBLOCK => WouldBlock,
        _ => Uncategorized,
    }
}

pub fn error_string(errno: i32) -> String {
    unsafe {
        let ptr = c::strerror(errno as core::ffi::c_int);
        if ptr.is_null() {
            return format!("Unknown error {errno}");
        }

        crate::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}
