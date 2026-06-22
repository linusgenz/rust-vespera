use crate::ffi::{OsStr, OsString};
use crate::os::vespera::ffi::{OsStrExt, OsStringExt};
use crate::marker::PhantomData;
use crate::path::{self, PathBuf};
use crate::sys::unsupported;
use crate::sys::pal::util::cstr;
use crate::{fmt, io};
use crate::sys::pal::c;

pub fn getcwd() -> io::Result<PathBuf> {
    let mut buf = vec![0u8; 1024];

    let res_ptr = unsafe {
        c::getcwd(buf.as_mut_ptr() as *mut core::ffi::c_char, buf.len())
    };

    if res_ptr.is_null() {
        return Err(io::Error::last_os_error());
    }

    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    let clean_bytes = &buf[..len];

    Ok(PathBuf::from(OsString::from(OsStr::from_bytes(clean_bytes))))
}

pub fn chdir(p: &path::Path) -> io::Result<()> {
    let path = cstr(p)?;

    let ret = unsafe { c::chdir(path.as_ptr()) };

    if ret == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

pub struct SplitPaths<'a> {
    iter: core::slice::Split<'a, u8, fn(&u8) -> bool>,
}

pub fn split_paths(unparsed: &OsStr) -> SplitPaths<'_> {
    fn is_colon(b: &u8) -> bool {
        *b == b':'
    }

    SplitPaths {
        iter: unparsed.as_bytes().split(is_colon),
    }
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        let bytes = self.iter.next()?;

        if bytes.is_empty() {
            return Some(PathBuf::from("."));
        }

        let os_str = OsStr::from_bytes(bytes);
        Some(PathBuf::from(os_str))
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    let mut joined = Vec::new();

    for (i, path) in paths.enumerate() {
        let path = path.as_ref();
        let bytes = path.as_bytes();
        if bytes.contains(&b':') {
            return Err(JoinPathsError);
        }
        if i > 0 {
            joined.push(b':');
        }
        joined.extend_from_slice(bytes);
    }

    Ok(OsString::from_vec(joined))
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "path segment contains separator ':'".fmt(f)
    }
}

impl crate::error::Error for JoinPathsError {}

pub fn current_exe() -> io::Result<PathBuf> {
    let mut args = crate::env::args_os();
    let argv_0 = args.next().ok_or_else(|| {
        io::const_error!(io::ErrorKind::NotFound, "argv[0] empty")
    })?;

    let path = PathBuf::from(argv_0);

    if path.is_absolute() {
        Ok(path)
    } else {
        let mut cwd = getcwd()?;
        cwd.push(path);
        Ok(cwd)
    }
}

pub fn temp_dir() -> PathBuf {
    PathBuf::from("/tmp")
}

pub fn home_dir() -> Option<PathBuf> {
    crate::env::var_os("HOME").map(PathBuf::from)
}