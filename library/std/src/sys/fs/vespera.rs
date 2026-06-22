// sys/fs/vespera.rs

use crate::ffi::{CStr, CString, OsStr, OsString};
use crate::fmt;
use crate::fs::TryLockError;
use crate::hash::{Hash, Hasher};
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::mem::MaybeUninit;
use crate::os::vespera::ffi::OsStrExt;
use crate::path::{Path, PathBuf};
pub use crate::sys::fs::common::Dir;
use crate::sys::pal::c;
use crate::sys::pal::util::cstr;
use crate::sys::time::SystemTime;
use crate::sys::unsupported;


fn map_err(ret: i64) -> io::Error {
    io::Error::from_raw_os_error(-ret as i32)
}


#[derive(Clone)]
pub struct FileAttr {
    stat: c::vespera_stat,
}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.stat.size
    }

    pub fn perm(&self) -> FilePermissions {
        FilePermissions { mode: self.stat.mode }
    }

    pub fn file_type(&self) -> FileType {
        FileType { node_type: self.stat.node_type }
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        SystemTime::new(self.stat.mtime as i64, 0)
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        SystemTime::new(self.stat.atime as i64, 0)
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        SystemTime::new(self.stat.crtime as i64, 0)
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FilePermissions {
    mode: u16,
}

const WRITE_BITS: u16 = 0o222;

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.mode & WRITE_BITS == 0
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            self.mode &= !WRITE_BITS;
        } else {
            self.mode |= 0o200; // Owner-Write-Bit setzen
        }
        // TODO: es gibt aktuell keine C-API, um geaenderte mode-Bits
        // synchron zurueckzuschreiben (sys_chmod ist als Syscall
        // vorhanden, aber noch nicht ueber eine hoeherwertige
        // Pfad-Funktion angebunden). set_readonly wirkt daher nur
        // lokal auf dieses Objekt, nicht persistent -- siehe set_perm().
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct FileType {
    node_type: u8,
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.node_type as u32 == c::VSTAT_TYPE_DIR
    }

    pub fn is_file(&self) -> bool {
        self.node_type as u32 == c::VSTAT_TYPE_FILE
    }

    pub fn is_symlink(&self) -> bool {
        self.node_type as u32 == c::VSTAT_TYPE_SYMLINK
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes {}

impl FileTimes {
    pub fn set_accessed(&mut self, _t: SystemTime) {}
    pub fn set_modified(&mut self, _t: SystemTime) {}
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }

    fn flags(&self) -> io::Result<core::ffi::c_int> {
        let mut flags: u32 = match (self.read, self.write, self.append) {
            (true, false, false) => c::O_RDONLY,
            (false, true, false) => c::O_WRONLY,
            (true, true, false) => c::O_RDWR,
            (_, _, true) => {
                if self.read {
                    c::O_RDWR | c::O_APPEND
                } else {
                    c::O_WRONLY | c::O_APPEND
                }
            }
            (false, false, false) => {
                return Err(io::const_error!(
                    io::ErrorKind::InvalidInput,
                    "OpenOptions: at least one of read/write/append must be set"
                ));
            }
        };

        if self.create_new {
            flags |= c::O_CREAT | c::O_EXCL;
        } else if self.create {
            flags |= c::O_CREAT;
        }
        if self.truncate {
            flags |= c::O_TRUNC;
        }

        Ok(flags as core::ffi::c_int)
    }
}

pub struct File {
    handle: c::HANDLE,
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        let p = cstr(path)?;
        let flags = opts.flags()?;
        let handle = unsafe { c::open(p.as_ptr(), flags) };
        if handle == u64::MAX {
            return Err(io::Error::last_os_error());
        }
        Ok(File { handle })
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        // Es gibt keine fstat-Variante (Handle-basiert) in der aktuellen
        // API, nur sys_stat ueber Pfad. Bis es eine Handle-Variante gibt,
        // ist file_attr() fuer offene Handles ohne bekannten Pfad nicht
        // implementierbar.
        unsupported()
    }

    pub fn fsync(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn lock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn lock_shared(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn try_lock(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(io::const_error!(
            io::ErrorKind::Unsupported,
            "file locking not supported on VesperaOS"
        )))
    }

    pub fn try_lock_shared(&self) -> Result<(), TryLockError> {
        Err(TryLockError::Error(io::const_error!(
            io::ErrorKind::Unsupported,
            "file locking not supported on VesperaOS"
        )))
    }

    pub fn unlock(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        unsupported()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            c::read(self.handle, buf.as_mut_ptr() as *mut core::ffi::c_void, buf.len())
        };
        if ret < 0 { Err(map_err(ret as i64)) } else { Ok(ret as usize) }
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_buf(&self, mut cursor: BorrowedCursor<'_, u8>) -> io::Result<()> {
        let ret = unsafe {
            let buf = cursor.as_mut();
            let ptr = buf.as_mut_ptr() as *mut core::ffi::c_void;

            c::read(self.handle, ptr, buf.len())
        };

        if ret < 0 {
            return Err(map_err(ret as i64));
        }

        let n = ret as usize;
        unsafe {
            cursor.advance(n);
        }

        Ok(())
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret =
            unsafe { c::write(self.handle, buf.as_ptr() as *const core::ffi::c_void, buf.len()) };
        if ret < 0 { Err(map_err(ret as i64)) } else { Ok(ret as usize) }
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        crate::io::default_write_vectored(|b| self.write(b), bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Start(off) => (off as i64, c::SEEK_SET),
            SeekFrom::End(off) => (off, c::SEEK_END),
            SeekFrom::Current(off) => (off, c::SEEK_CUR),
        };
        let ret = unsafe { c::lseek(self.handle, offset, whence as core::ffi::c_int) };
        if ret < 0 { Err(map_err(ret)) } else { Ok(ret as u64) }
    }

    pub fn size(&self) -> Option<io::Result<u64>> {
        None
    }

    pub fn tell(&self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }

    pub fn duplicate(&self) -> io::Result<File> {
        unsupported()
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        unsupported()
    }

    pub fn set_times(&self, _times: FileTimes) -> io::Result<()> {
        unsupported()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            c::close(self.handle);
        }
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").field("handle", &self.handle).finish()
    }
}

#[derive(Debug)]
pub struct DirBuilder {}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, p: &Path) -> io::Result<()> {
        let path = cstr(p)?;
        let ret = unsafe { c::mkdir(path.as_ptr()) };
        if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }
}

pub struct ReadDir {
    handle: c::DIR_HANDLE,
    dir_path: PathBuf,
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadDir").field("dir_path", &self.dir_path).finish()
    }
}

impl Drop for ReadDir {
    fn drop(&mut self) {
        unsafe {
            c::closedir(self.handle);
        }
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        let mut entry = MaybeUninit::<c::dirent_t>::uninit();
        let ret = unsafe { c::readdir(self.handle, entry.as_mut_ptr()) };
        if ret == 0 {
            // end
            return None;
        }
        if ret < 0 {
            return Some(Err(map_err(ret as i64)));
        }

        let entry = unsafe { entry.assume_init() };
        let name_bytes = unsafe { CStr::from_ptr(entry.name.as_ptr()) }.to_bytes();
        let os_str = OsStr::from_bytes(name_bytes);
        let file_name = os_str.to_os_string();

        Some(Ok(DirEntry {
            dir_path: self.dir_path.clone(),
            file_name,
            node_type: entry.type_,
        }))
    }
}

pub struct DirEntry {
    dir_path: PathBuf,
    file_name: OsString,
    node_type: u32,
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.dir_path.join(&self.file_name)
    }

    pub fn file_name(&self) -> OsString {
        self.file_name.clone()
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        stat(&self.path())
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(FileType { node_type: self.node_type as u8 })
    }
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    let path = cstr(p)?;
    let handle = unsafe { c::opendir(path.as_ptr()) };
    if handle == u64::MAX {
        return Err(io::Error::last_os_error());
    }
    Ok(ReadDir { handle, dir_path: p.to_path_buf() })
}

pub fn unlink(p: &Path) -> io::Result<()> {
    let path = cstr(p)?;
    let ret = unsafe { c::unlink(path.as_ptr()) };
    if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

pub fn rename(old: &Path, new: &Path) -> io::Result<()> {
    let old_c = cstr(old)?;
    let new_c = cstr(new)?;
    let ret = unsafe { c::rename(old_c.as_ptr(), new_c.as_ptr()) };
    if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

pub fn set_perm(_p: &Path, _perm: FilePermissions) -> io::Result<()> {
    // TODO: ueber sys_chmod anbinden, sobald eine Pfad-basierte
    // hoeherwertige Funktion dafuer existiert.
    unsupported()
}

pub fn set_times(_p: &Path, _times: FileTimes) -> io::Result<()> {
    unsupported()
}

pub fn set_times_nofollow(_p: &Path, _times: FileTimes) -> io::Result<()> {
    unsupported()
}

pub fn rmdir(p: &Path) -> io::Result<()> {
    let path = cstr(p)?;
    let ret = unsafe { c::rmdir(path.as_ptr()) };
    if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
}

pub fn remove_dir_all(path: &Path) -> io::Result<()> {
    // Keine native rekursive Loesch-API vorhanden -- generische
    // std-Hilfsfunktion nutzen, die readdir/unlink/rmdir manuell
    // rekursiv aufruft (verfuegbar ueber sys::fs::common, falls
    // vorhanden) oder selbst rekursiv implementieren.
    let attr = stat(path)?;
    if !attr.file_type().is_dir() {
        return unlink(path);
    }
    for entry in readdir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            remove_dir_all(&entry_path)?;
        } else {
            unlink(&entry_path)?;
        }
    }
    rmdir(path)
}

pub fn exists(path: &Path) -> io::Result<bool> {
    match stat(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    // Keine native readlink-Syscall/Funktion in der aktuellen API
    // vorhanden.
    unsupported()
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    unsupported()
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(p: &Path) -> io::Result<FileAttr> {
    let path = cstr(p)?;
    let mut stat_buf = MaybeUninit::<c::vespera_stat>::uninit();
    let ret = unsafe {
        c::sys_stat(
            path.as_ptr().addr() as u64,
            stat_buf.as_mut_ptr().addr() as u64,
            0,
            0,
            0,
            0,
        )
    };
    if ret < 0 {
        return Err(map_err(ret));
    }
    Ok(FileAttr { stat: unsafe { stat_buf.assume_init() } })
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    stat(p)
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    unsupported()
}

pub fn copy(from: &Path, to: &Path) -> io::Result<u64> {
    let mut reader = File::open(from, &{
        let mut o = OpenOptions::new();
        o.read(true);
        o
    })?;
    let mut writer = File::open(to, &{
        let mut o = OpenOptions::new();
        o.write(true);
        o.create(true);
        o.truncate(true);
        o
    })?;

    let mut buf = [0u8; 8192];
    let mut total = 0u64;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write(&buf[..n])?;
        total += n as u64;
    }
    Ok(total)
}