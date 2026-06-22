
unsafe extern "C" {
    #[doc = " @brief Terminate the current unit.\n\n @param code Exit code.\n @return This function does not return; halts the unit."]
    pub fn sys_exit(code: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> i64;
}

unsafe extern "C" {
    #[doc = " @brief Write data to a handle.\n\n @param hid Handle ID.\n @param buf Buffer pointer.\n @param count Number of bytes to write.\n @return Number of bytes written, or negative error code."]
    pub fn sys_write(hid: u64, buf_ptr: u64, count: u64, arg1: u64, arg2: u64, arg3: u64) -> i64;
}

unsafe extern "C" {
    #[doc = " @brief Read from a handle into a buffer.\n\n @param hid Handle ID.\n @param buf Buffer pointer.\n @param count Number of bytes to read.\n @return Number of bytes read, or negative error code."]
    pub fn sys_read(hid: u64, buf_ptr: u64, count: u64, arg1: u64, arg2: u64, arg3: u64) -> i64;
}

unsafe extern "C" {
    #[doc = " @brief Allocates a block of memory of the specified size.\n\n The memory is allocated using the kernel's memory mapping syscall (mmap).\n This is a very basic allocator and does not reuse freed blocks or implement\n advanced heap management.\n\n @param size Number of bytes to allocate.\n @return Pointer to allocated memory on success,\n         or NULL if allocation failed."]
    pub fn malloc(size: core::ffi::c_ulong) -> *mut core::ffi::c_void;
}
unsafe extern "C" {
    #[doc = " @brief Releases a block of memory previously allocated by malloc().\n\n The memory is unmapped using the kernel's sys_munmap.\n Since this allocator uses mmap per allocation, the entire block is released.\n\n @param ptr Pointer to the memory block to free. If NULL, no action is taken."]
    pub fn free(ptr: *mut core::ffi::c_void);
}
unsafe extern "C" {
    #[doc = " @brief Reallocates a memory block to a new size.\n\n A new block is allocated with malloc(), the old contents are copied,\n and the old block is freed.\n\n @param ptr Pointer to the memory block to resize (may be NULL).\n @param new_size New size in bytes.\n @return Pointer to the new memory block on success,\n         or NULL if allocation failed (old block is not freed in this case)."]
    pub fn realloc(
        ptr: *mut core::ffi::c_void,
        new_size: core::ffi::c_ulong,
    ) -> *mut core::ffi::c_void;
}

unsafe extern "C" {
    pub fn abort() -> !;
}

unsafe extern "C" {
    pub static mut environ: *mut *mut core::ffi::c_char;
}

unsafe extern "C" {
    #[doc = " @brief Get the value of an environment variable.\n\n Searches the environment list for a variable with the given name and\n returns a pointer to its value (the part after the '=' character).\n The returned pointer points to internal memory and must not be modified.\n\n @param name Variable name (null-terminated string, must not contain '=').\n @return Pointer to the value string, or @c NULL if not found.\n\n @see setenv()\n @see unsetenv()"]
    pub fn getenv(name: *const core::ffi::c_char) -> *mut core::ffi::c_char;
}

unsafe extern "C" {
    #[doc = " @brief Set or modify an environment variable.\n\n Adds a new environment variable or modifies an existing one. The function\n makes internal copies of both the name and value strings.\n\n @param name Variable name (must not contain '=').\n @param value Value to set (null-terminated string).\n @param overwrite If @c 0, existing variables are not modified. If non-zero, overwrite.\n @return @c 0 on success, @c -1 on failure.\n\n @see getenv()\n @see unsetenv()"]
    pub fn setenv(
        name: *const core::ffi::c_char,
        value: *const core::ffi::c_char,
        overwrite: core::ffi::c_int,
    ) -> core::ffi::c_int;
}

unsafe extern "C" {
    #[doc = " @brief Remove an environment variable.\n\n Removes the variable with the given name from the environment.\n Returns @c 0 even if the variable doesn't exist (POSIX behavior).\n\n @param name Variable name (must not contain '=').\n @return @c 0 on success, @c -1 on failure (invalid parameters).\n\n @see getenv()\n @see setenv()"]
    pub fn unsetenv(name: *const core::ffi::c_char) -> core::ffi::c_int;
}

// --- fflags.h: Open-Flags ---
pub const O_RDONLY: u32 = 0;
pub const O_WRONLY: u32 = 1;
pub const O_RDWR: u32 = 2;
pub const O_CREAT: u32 = 64;
pub const O_EXCL: u32 = 128;
pub const O_TRUNC: u32 = 512;
pub const O_APPEND: u32 = 1024;
pub const O_DIRECTORY: u32 = 8192;

pub const SEEK_SET: u32 = 0;
pub const SEEK_CUR: u32 = 1;
pub const SEEK_END: u32 = 2;

// --- stat.h: vespera_stat_t ---
pub const VSTAT_TYPE_UNKNOWN: u32 = 0;
pub const VSTAT_TYPE_FILE: u32 = 1;
pub const VSTAT_TYPE_DIR: u32 = 2;
pub const VSTAT_TYPE_CHARDEV: u32 = 4;
pub const VSTAT_TYPE_BLOCKDEV: u32 = 5;
pub const VSTAT_TYPE_SYMLINK: u32 = 3;

pub const VSTAT_FLAG_READABLE: u32 = 1;
pub const VSTAT_FLAG_WRITABLE: u32 = 2;
pub const VSTAT_FLAG_EXEC: u32 = 4;
pub const VSTAT_FLAG_VIRTUAL: u32 = 8;
pub const VSTAT_FLAG_PERMANENT: u32 = 16;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vespera_stat {
    pub node_type: u8,
    pub _pad0: [u8; 3],
    pub flags: u32,
    pub dev_id: u32,
    pub block_size: u32,
    pub inode_id: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: u32,
    pub mtime: u32,
    pub ctime: u32,
    pub crtime: u32,
    pub mode: u16,
    pub links_count: u16,
    pub uid: u32,
    pub gid: u32,
}

#[doc = "< Unknown entry type (not determined)"]
pub const dirent_type_t_DT_UNKNOWN: dirent_type_t = 0;
#[doc = "< Regular file"]
pub const dirent_type_t_DT_FILE: dirent_type_t = 1;
#[doc = "< Directory"]
pub const dirent_type_t_DT_DIR: dirent_type_t = 2;
#[doc = "< Symbolic link"]
pub const dirent_type_t_DT_SYMLINK: dirent_type_t = 3;
#[doc = "< Character device"]
pub const dirent_type_t_DT_CHARDEV: dirent_type_t = 4;
#[doc = "< Block device"]
pub const dirent_type_t_DT_BLOCKDEV: dirent_type_t = 5;
#[doc = "< Named pipe (FIFO)"]
pub const dirent_type_t_DT_FIFO: dirent_type_t = 6;
#[doc = "< Socket"]
pub const dirent_type_t_DT_SOCKET: dirent_type_t = 7;
#[doc = "< Executable file"]
pub const dirent_type_t_DT_EXEC: dirent_type_t = 8;
#[doc = " @brief Enumeration of possible directory entry types.\n\n This enumeration defines the type of an entry returned by `sys_readdir()`\n or higher-level wrappers (e.g. `readdir()`)."]
pub type dirent_type_t = core::ffi::c_uint;
#[doc = " @brief Structure representing a directory entry.\n\n Contains information about a single directory entry."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dirent_t {
    #[doc = "< Null-terminated entry name (up to 127 characters)"]
    pub name: [core::ffi::c_char; 128usize],
    #[doc = "< Type of the directory entry (see ::dirent_type_t)"]
    pub type_: dirent_type_t,
}

pub type HANDLE = u64;
pub type DIR_HANDLE = HANDLE;

unsafe extern "C" {
    pub fn open(path: *const core::ffi::c_char, flags: core::ffi::c_int) -> HANDLE;
    pub fn close(handle: HANDLE) -> core::ffi::c_int;
    pub fn read(handle: HANDLE, buf: *mut core::ffi::c_void, count: usize) -> isize;
    pub fn write(handle: HANDLE, buf: *const core::ffi::c_void, count: usize) -> isize;
    pub fn lseek(handle: HANDLE, offset: i64, whence: core::ffi::c_int) -> i64;

    pub fn opendir(path: *const core::ffi::c_char) -> DIR_HANDLE;
    pub fn closedir(handle: DIR_HANDLE) -> core::ffi::c_int;
    pub fn readdir(handle: DIR_HANDLE, entry: *mut dirent_t) -> isize;

    pub fn mkdir(path: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn rmdir(path: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn unlink(path: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn rename(
        oldpath: *const core::ffi::c_char,
        newpath: *const core::ffi::c_char,
    ) -> core::ffi::c_int;

    pub fn chdir(path: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn getcwd(buf: *mut core::ffi::c_char, size: usize) -> *mut core::ffi::c_char;

    pub fn is_directory(path: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn is_file(path: *const core::ffi::c_char) -> core::ffi::c_int;
}

unsafe extern "C" {
    pub fn sys_stat(path_ptr: u64, buf_ptr: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64;
}

pub const CLOCK_REALTIME: u32 = 0;
pub const CLOCK_MONOTONIC: u32 = 1;
pub const CLOCK_PROCESS_CPUTIME_ID: u32 = 2;
pub const CLOCK_MONOTONIC_RAW: u32 = 4;
pub const CLOCK_BOOTTIME: u32 = 7;
pub const TIMER_ABSTIME: u32 = 1;

pub type clockid_t = i32;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct timespec_t {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

unsafe extern "C" {
    pub fn clock_gettime(clk_id: clockid_t, ts: *mut timespec_t) -> i64;
    pub fn clock_settime(clk_id: clockid_t, ts: *const timespec_t) -> i64;
    pub fn nanosleep(req: *const timespec_t, rem: *mut timespec_t) -> core::ffi::c_int;
    pub fn clock_nanosleep(
        clk_id: clockid_t,
        flags: core::ffi::c_int,
        req: *const timespec_t,
        rem: *mut timespec_t,
    ) -> core::ffi::c_int;
}

pub const SUCCESS_CODE: i32 = 0;
pub const EPERM: i32 = 1;
pub const ENOENT: i32 = 2;
pub const ESRCH: i32 = 3;
pub const EINTR: i32 = 4;
pub const EIO: i32 = 5;
pub const ENXIO: i32 = 6;
pub const E2BIG: i32 = 7;
pub const ENOEXEC: i32 = 8;
pub const EBADH: i32 = 9;
pub const ECHILD: i32 = 10;
pub const EAGAIN: i32 = 11;
pub const ENOMEM: i32 = 12;
pub const EACCES: i32 = 13;
pub const EFAULT: i32 = 14;
pub const EBUSY: i32 = 16;
pub const EEXIST: i32 = 17;
pub const EXDEV: i32 = 18;
pub const ENODEV: i32 = 19;
pub const ENOTDIR: i32 = 20;
pub const EISDIR: i32 = 21;
pub const EINVAL: i32 = 22;
pub const ENFILE: i32 = 23;
pub const EMFILE: i32 = 24;
pub const ENOTTY: i32 = 25;
pub const ETXTBSY: i32 = 26;
pub const EFBIG: i32 = 27;
pub const ENOSPC: i32 = 28;
pub const ESPIPE: i32 = 29;
pub const EROFS: i32 = 30;
pub const EMLINK: i32 = 31;
pub const EPIPE: i32 = 32;
pub const EDOM: i32 = 33;
pub const ERANGE: i32 = 34;
pub const ENAMETOOLONG: i32 = 36;
pub const ENOLCK: i32 = 37;
pub const ENOSYS: i32 = 38;
pub const ENOTEMPTY: i32 = 39;
pub const ELOOP: i32 = 40;
pub const ENOMSG: i32 = 42;
pub const EOVERFLOW: i32 = 75;
pub const EILSEQ: i32 = 84;
pub const EUNKNOWN: i32 = 1000;
pub const EUNSUPPORTED: i32 = 1001;
pub const EDEADLOCK: i32 = 1002;
pub const EWOULDBLOCK: i32 = 11;

unsafe extern "C" {
    pub fn strerror(err: core::ffi::c_int) -> *const core::ffi::c_char;
}
unsafe extern "C" {
    pub static mut errno: core::ffi::c_int;
}