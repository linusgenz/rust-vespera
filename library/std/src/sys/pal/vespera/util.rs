use crate::ffi::{CStr, CString, OsString, OsStr};
use crate::path::{Path, PathBuf};
use crate::os::vespera::ffi::OsStrExt;
use crate::{io};

pub fn cstr(path: &Path) -> io::Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "path contains a NUL byte"))
}

