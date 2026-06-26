// fd/vespera.rs
//
// FileDesc für VesperaOS.
//
// Massiv simpler als sys/fd/unix.rs, weil:
//   - keine pread/pwrite/readv/writev/preadv/pwritev-Syscalls vorhanden
//     -> *_at()-Varianten sind unsupported(), *_vectored() fallen auf den
//        generischen io::default_*_vectored-Loop zurück
//   - kein fork()+exec() -> kein implizites FD-Vererbungsproblem, das
//     CLOEXEC lösen müsste. Vererbung läuft bei uns ausschließlich explizit
//     über spawn_config (stdin_handle/stdout_handle/stderr_handle), nie
//     implizit über "alles was offen ist, außer CLOEXEC". set_cloexec() ist
//     daher ein reines No-Op, kein echtes Sicherheitsloch.
//   - Handles sind u64-IDs aus der kernelseitigen HandleTable, kein
//     POSIX-RawFd -> kein AsRawFd/FromRawFd/IntoRawFd-Trait-Gewirr nötig,
//     stattdessen schlichte as_raw_handle()/from_raw_handle()-Methoden.

use crate::cmp;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, Read};
use crate::sys::pal::c;
use crate::sys::unsupported;

#[derive(Debug)]
#[unstable(feature = "vespera_platform", issue = "none")]
pub struct FileDesc(c::HANDLE);

impl FileDesc {
    #[inline]
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.duplicate()
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe {
            c::read(self.0, buf.as_mut_ptr() as *mut core::ffi::c_void, buf.len())
        };
        if ret < 0 { Err(io::Error::from_raw_os_error(-ret as i32)) } else { Ok(ret as usize) }
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        // Kein readv()-Syscall vorhanden -- generischer sequentieller
        // Fallback (liest in den ersten nicht-leeren Buffer, wie von
        // io::default_read_vectored vorgesehen).
        io::default_read_vectored(|b| self.read(b), bufs)
    }

    #[inline]
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn is_read_vectored(&self) -> bool {
        false
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn read_at(&self, _buf: &mut [u8], _offset: u64) -> io::Result<usize> {
        // Kein pread()-Äquivalent -- es gibt nur lseek()+read(), das den
        // Datei-Cursor mutiert. Echtes positionsbasiertes Lesen (ohne
        // Cursor-Seiteneffekt) ist auf VesperaOS aktuell nicht abbildbar.
        unsupported()
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn read_buf(&self, mut cursor: BorrowedCursor<'_, u8>) -> io::Result<()> {
        // SAFETY: cursor.as_mut() liefert cursor.capacity() beschreibbare Bytes.
        let ret = unsafe {
            c::read(
                self.0,
                cursor.as_mut().as_mut_ptr().cast::<core::ffi::c_void>(),
                cursor.capacity(),
            )
        };
        if ret < 0 {
            return Err(io::Error::from_raw_os_error(-ret as i32));
        }
        // SAFETY: ret Bytes wurden tatsächlich in den initialisierten
        // Teil des Buffers geschrieben.
        unsafe {
            cursor.advance(ret as usize);
        }
        Ok(())
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn read_buf_at(&self, _cursor: BorrowedCursor<'_, u8>, _offset: u64) -> io::Result<()> {
        unsupported()
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn read_vectored_at(&self, _bufs: &mut [IoSliceMut<'_>], _offset: u64) -> io::Result<usize> {
        unsupported()
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe {
            c::write(self.0, buf.as_ptr() as *const core::ffi::c_void, buf.len())
        };
        if ret < 0 { Err(io::Error::from_raw_os_error(-ret as i32)) } else { Ok(ret as usize) }
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        io::default_write_vectored(|b| self.write(b), bufs)
    }

    #[inline]
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn is_write_vectored(&self) -> bool {
        false
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn write_at(&self, _buf: &[u8], _offset: u64) -> io::Result<usize> {
        unsupported()
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn write_vectored_at(&self, _bufs: &[IoSlice<'_>], _offset: u64) -> io::Result<usize> {
        unsupported()
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    // set_cloexec ist ein bewusstes No-Op, kein fehlendes Feature: Vespera
    // hat kein fork()+exec(), Handle-Vererbung läuft ausschließlich explizit
    // über spawn_config::{stdin,stdout,stderr}_handle. Es gibt also kein
    // implizites "alle offenen FDs erben außer CLOEXEC"-Verhalten, das hier
    // abgesichert werden müsste.
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn set_cloexec(&self) -> io::Result<()> {
        Ok(())
    }

    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        // Kein fcntl/ioctl-Äquivalent für O_NONBLOCK vorhanden.
        unsupported()
    }

    #[inline]
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn duplicate(&self) -> io::Result<FileDesc> {
        let ret = unsafe { c::sys_dup(self.0, 0, 0, 0, 0, 0) };
        if ret < 0 {
            return Err(io::Error::from_raw_os_error(-ret as i32));
        }
        Ok(FileDesc(ret as c::HANDLE))
    }

    #[inline]
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn as_raw_handle(&self) -> c::HANDLE {
        self.0
    }

    /// Übernimmt Eigentümerschaft an einem bereits offenen Handle.
    ///
    /// # Safety
    /// `handle` muss ein gültiges, exklusiv besessenes Handle sein -- nach
    /// dem Aufruf schließt der Drop-Impl von FileDesc es automatisch.
    #[inline]
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub unsafe fn from_raw_handle(handle: c::HANDLE) -> FileDesc {
        FileDesc(handle)
    }

    /// Gibt das Handle frei, OHNE es zu schließen (Eigentümerschaft geht an
    /// den Aufrufer über). Pendant zu IntoRawFd::into_raw_fd in unix.rs.
    #[inline]
    #[unstable(feature = "vespera_platform", issue = "none")]
    pub fn into_raw_handle(self) -> c::HANDLE {
        let h = self.0;
        core::mem::forget(self);
        h
    }
}

#[unstable(feature = "vespera_platform", issue = "none")]
impl Drop for FileDesc {
    fn drop(&mut self) {
        unsafe {
            let _ = c::sys_close(self.0, 0, 0, 0, 0, 0);
        }
    }
}

#[unstable(feature = "vespera_platform", issue = "none")]
impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    fn read_buf(&mut self, cursor: BorrowedCursor<'_, u8>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
}