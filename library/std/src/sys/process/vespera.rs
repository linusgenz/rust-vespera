// process/vespera.rs
//
// VesperaOS-Implementierung von std::process.
//
// Architektur-Unterschied zu unix.rs: kein fork()+exec() als zwei getrennte
// Syscalls. spawn_realm() erzeugt Realm + lädt das ELF-Image + startet die
// erste Unit in EINEM synchronen Aufruf und gibt Fehler direkt zurück.
// Das eliminiert den ganzen CLOEXEC-Pipe-Tanz aus unix.rs (dort nötig, um
// einen exec()-Fehler vom Kind zurück zum Parent zu melden, weil fork() und
// exec() zwei getrennte Prozess-Zustände sind) -- bei uns gibt es diesen
// Zwischenzustand schlicht nicht.

use super::env::{CommandEnv, CommandEnvs, CommandResolvedEnvs};
pub use crate::ffi::OsString as EnvKey;
use crate::ffi::{CString, OsStr, OsString};
use crate::num::NonZero;
use crate::path::Path;
use crate::process::StdioPipes;
use crate::sys::fd::FileDesc;
use crate::sys::fs::File;
use crate::sys::pal::c;
use crate::sys::pipe::{self, Pipe};
use crate::{fmt, io};

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

pub struct Command {
    program: OsString,
    args: Vec<OsString>,
    env: CommandEnv,

    cwd: Option<OsString>,
    uid: Option<u32>,
    gid: Option<u32>,

    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
}

#[derive(Debug)]
pub enum Stdio {
    Inherit,
    Null,
    MakePipe,
    ParentStdout,
    ParentStderr,
    Pipe(ChildPipe),
    #[allow(dead_code)] // existiert nur für den Debug-Impl, wie in unsupported.rs
    InheritFile(File),
}

impl Command {
    pub fn new(program: &OsStr) -> Command {
        Command {
            program: program.to_owned(),
            args: vec![program.to_owned()],
            env: Default::default(),
            cwd: None,
            uid: None,
            gid: None,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn arg(&mut self, arg: &OsStr) {
        self.args.push(arg.to_owned());
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, dir: &OsStr) {
        self.cwd = Some(dir.to_owned());
    }

    pub fn uid(&mut self, uid: u32) {
        self.uid = Some(uid);
    }

    pub fn gid(&mut self, gid: u32) {
        self.gid = Some(gid);
    }

    pub fn stdin(&mut self, stdin: Stdio) {
        self.stdin = Some(stdin);
    }

    pub fn stdout(&mut self, stdout: Stdio) {
        self.stdout = Some(stdout);
    }

    pub fn stderr(&mut self, stderr: Stdio) {
        self.stderr = Some(stderr);
    }

    pub fn get_program(&self) -> &OsStr {
        &self.program
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        let mut iter = self.args.iter();
        iter.next();
        CommandArgs { iter }
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    pub fn get_env_clear(&self) -> bool {
        self.env.does_clear()
    }

    pub fn get_resolved_envs(&self) -> CommandResolvedEnvs {
        CommandResolvedEnvs::new(self.env.capture())
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        self.cwd.as_ref().map(|cs| Path::new(cs))
    }

    fn saw_nul(&self) -> bool {
        self.program.as_encoded_bytes().contains(&0)
            || self.args.iter().any(|a| a.as_encoded_bytes().contains(&0))
    }

    fn cstring_args(&self) -> io::Result<(CString, Vec<CString>)> {
        let to_cstring = |s: &OsStr| {
            CString::new(s.as_encoded_bytes())
                .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "nul byte found in provided data"))
        };
        let program = to_cstring(&self.program)?;
        let mut args = Vec::with_capacity(self.args.len());
        for a in &self.args {
            args.push(to_cstring(a)?);
        }
        Ok((program, args))
    }

    fn capture_envp(&mut self) -> io::Result<Option<Vec<CString>>> {
        let env = self.env.capture();
        if self.env.is_unchanged() {
            return Ok(None);
        }
        let mut result = Vec::with_capacity(env.len());
        for (k, v) in env.iter() {
            let mut entry = OsString::from(k);
            entry.push("=");
            entry.push(v);
            result.push(
                CString::new(entry.as_encoded_bytes())
                    .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "nul byte found in provided data"))?,
            );
        }
        Ok(Some(result))
    }

    fn resolve_stdio(slot: Option<Stdio>) -> io::Result<(u64, Option<Pipe>, Option<Pipe>)> {
        match slot {
            None | Some(Stdio::Inherit) => Ok((0, None, None)),

            Some(Stdio::Null) => {
                let mut opts = crate::sys::fs::OpenOptions::new();
                opts.read(true);
                opts.write(true);
                let f = File::open(Path::new("/dev/null"), &opts)?;
                let handle = f.as_raw_handle() as u64;
                Ok((handle, None, Some(unsafe { Pipe::from_raw_handle(f.into_raw_handle()) })))
            }

            Some(Stdio::MakePipe) => {
                let (ours, theirs) = pipe::pipe()?;
                let handle = theirs.as_raw_handle() as u64;
                Ok((handle, Some(ours), Some(theirs)))
            }

            Some(Stdio::Pipe(child_pipe)) => {
                let handle = child_pipe.as_raw_handle() as u64;
                Ok((handle, None, Some(child_pipe)))
            }

            Some(Stdio::ParentStdout) => Ok((c::HANDLE_STDOUT, None, None)),
            Some(Stdio::ParentStderr) => Ok((c::HANDLE_STDERR, None, None)),

            Some(Stdio::InheritFile(f)) => {
                let handle = f.as_raw_handle() as u64;
                Ok((handle, None, Some(unsafe { Pipe::from_raw_handle(f.into_raw_handle()) })))
            }
        }
    }

    pub fn spawn(
        &mut self,
        default: Stdio,
        needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        if self.saw_nul() {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "nul byte found in provided data",
            ));
        }

        let (program, args) = self.cstring_args()?;

        let mut argv_ptrs: Vec<*mut core::ffi::c_char> =
            args.iter().map(|s| s.as_ptr() as *mut _).collect();
        argv_ptrs.push(core::ptr::null_mut());

        let envp = self.capture_envp()?;
        let mut envp_ptrs: Vec<*mut core::ffi::c_char> = Vec::new();
        let envp_arg = if let Some(envp) = &envp {
            envp_ptrs = envp.iter().map(|s| s.as_ptr() as *mut _).collect();
            envp_ptrs.push(core::ptr::null_mut());
            envp_ptrs.as_ptr()
        } else {
            core::ptr::null()
        };

        let stdin_default = if needs_stdin { Stdio::MakePipe } else { Stdio::Inherit };

        let (stdin_h, our_stdin, stdin_parent_copy) = Self::resolve_stdio(self.stdin.take())?;
        let (stdout_h, our_stdout, stdout_parent_copy) = Self::resolve_stdio(self.stdout.take())?;
        let (stderr_h, our_stderr, stderr_parent_copy) = Self::resolve_stdio(self.stderr.take())?;

        let cwd_cstr = self
            .cwd
            .as_ref()
            .map(|c| CString::new(c.as_encoded_bytes()))
            .transpose()
            .map_err(|_| io::const_error!(io::ErrorKind::InvalidInput, "nul byte found in provided data"))?;

        let mut cfg = c::spawn_config {
            stdin_handle: stdin_h,
            stdout_handle: stdout_h,
            stderr_handle: stderr_h,
            bg_realm: 0,
            realm_name: core::ptr::null_mut(),
            uid: self.uid.unwrap_or(0),
            gid: self.gid.unwrap_or(0),
            home: cwd_cstr.as_ref().map(|c| c.as_ptr() as *mut _).unwrap_or(core::ptr::null_mut()),
        };

        let rid: c::RealmID = unsafe {
            c::spawn_realm(program.as_ptr(), argv_ptrs.as_ptr(), envp_arg, &mut cfg)
        };


        if (rid as i64) < 0 {
            return Err(io::Error::from_raw_os_error(-(rid as i64) as i32));
        }

        if (rid as i64) < 0 {
            return Err(io::Error::from_raw_os_error(-(rid as i64) as i32));
        }

        let pipes = StdioPipes { stdin: our_stdin, stdout: our_stdout, stderr: our_stderr };

        Ok((Process { rid, status: None }, pipes))
    }

    pub fn exec(&mut self, default: Stdio) -> io::Error {
        match self.spawn(default, false) {
            Ok((mut child, _)) => match child.wait() {
                Ok(status) => crate::process::exit(status.code().unwrap_or(1)),
                Err(e) => e,
            },
            Err(e) => e,
        }
    }
}

impl From<ChildPipe> for Stdio {
    fn from(pipe: ChildPipe) -> Stdio {
        Stdio::Pipe(pipe)
    }
}

impl From<io::Stdout> for Stdio {
    fn from(_: io::Stdout) -> Stdio {
        Stdio::ParentStdout
    }
}

impl From<io::Stderr> for Stdio {
    fn from(_: io::Stderr) -> Stdio {
        Stdio::ParentStderr
    }
}

impl From<File> for Stdio {
    fn from(file: File) -> Stdio {
        Stdio::InheritFile(file)
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let mut debug_command = f.debug_struct("Command");
            debug_command.field("program", &self.program).field("args", &self.args);
            if !self.env.is_unchanged() {
                debug_command.field("env", &self.env);
            }
            if self.cwd.is_some() {
                debug_command.field("cwd", &self.cwd);
            }
            if self.stdin.is_some() {
                debug_command.field("stdin", &self.stdin);
            }
            if self.stdout.is_some() {
                debug_command.field("stdout", &self.stdout);
            }
            if self.stderr.is_some() {
                debug_command.field("stderr", &self.stderr);
            }
            debug_command.finish()
        } else {
            if let Some(ref cwd) = self.cwd {
                write!(f, "cd {cwd:?} && ")?;
            }
            if self.env.does_clear() {
                write!(f, "env -i ")?;
            } else {
                let mut any_removed = false;
                for (key, value_opt) in self.get_envs() {
                    if value_opt.is_none() {
                        if !any_removed {
                            write!(f, "env ")?;
                            any_removed = true;
                        }
                        write!(f, "-u {} ", key.to_string_lossy())?;
                    }
                }
            }
            for (key, value_opt) in self.get_envs() {
                if let Some(value) = value_opt {
                    write!(f, "{}={value:?} ", key.to_string_lossy())?;
                }
            }
            if self.program != self.args[0] {
                write!(f, "[{:?}] ", self.program)?;
            }
            write!(f, "{:?}", self.args[0])?;
            for arg in &self.args[1..] {
                write!(f, " {:?}", arg)?;
            }
            Ok(())
        }
    }
}

pub struct CommandArgs<'a> {
    iter: crate::slice::Iter<'a, OsString>,
}

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        self.iter.next().map(|os| &**os)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for CommandArgs<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<'a> fmt::Debug for CommandArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.clone()).finish()
    }
}

pub struct Process {
    rid: c::RealmID,

    status: Option<ExitStatus>,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.rid as u32
    }

    pub fn kill(&mut self) -> io::Result<()> {
        // VesperaOS exponiert noch keine eigenen Signal-Konstanten über die
        // FFI -- POSIX-Standardwert verwendet.
        const SIGKILL: i32 = 9;
        self.send_signal(SIGKILL)
    }

    pub(crate) fn send_signal(&mut self, signal: i32) -> io::Result<()> {
        if self.status.is_some() {
            return Ok(());
        }
        let ret = unsafe { c::kill(self.rid as core::ffi::c_int, signal) };
        if ret == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        let mut raw_status: core::ffi::c_int = 0;
        let ret = unsafe { c::wait_realm(self.rid, &mut raw_status, c::WAIT_FLAG_NONE) };
        if ret < 0 {
            return Err(io::Error::from_raw_os_error(-ret));
        }
        let status = ExitStatus::new(raw_status);
        self.status = Some(status);
        Ok(status)
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(status) = self.status {
            return Ok(Some(status));
        }
        let mut raw_status: core::ffi::c_int = 0;
        let ret = unsafe { c::wait_realm(self.rid, &mut raw_status, c::WAIT_FLAG_NOHANG) };
        if ret < 0 {
            return Err(io::Error::from_raw_os_error(-ret));
        }
        if ret == 0 {
            return Ok(None);
        }
        let status = ExitStatus::new(raw_status);
        self.status = Some(status);
        Ok(Some(status))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct ExitStatus(core::ffi::c_int);

impl ExitStatus {
    pub fn new(status: core::ffi::c_int) -> ExitStatus {
        ExitStatus(status)
    }

    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        match NonZero::try_from(self.0) {
            Ok(failure) => Err(ExitStatusError(failure)),
            Err(_) => Ok(()),
        }
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0)
    }

    pub fn signal(&self) -> Option<i32> {
        None
    }
}

impl fmt::Debug for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("vespera_exit_status").field(&self.0).finish()
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit status: {}", self.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ExitStatusError(NonZero<core::ffi::c_int>);

impl fmt::Debug for ExitStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("vespera_exit_status_error").field(&self.0).finish()
    }
}

impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        ExitStatus(self.0.into())
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZero<i32>> {
        NonZero::new(self.0.get() as i32)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitCode(u8);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(0);
    pub const FAILURE: ExitCode = ExitCode(1);

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        Self(code)
    }
}

pub type ChildPipe = crate::sys::pipe::Pipe;

pub fn read_output(
    out: ChildPipe,
    stdout: &mut Vec<u8>,
    err: ChildPipe,
    stderr: &mut Vec<u8>,
) -> io::Result<()> {
    out.read_to_end(stdout)?;
    err.read_to_end(stderr)?;
    Ok(())
}

pub fn getpid() -> u32 {
    unsafe { c::get_realm_id() as u32 }
}