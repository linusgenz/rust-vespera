// sys/pal/vespera/time.rs

use core::mem;
use core::num::niche_types::Nanoseconds;
use core::mem::MaybeUninit;

use crate::io;
use crate::sys::pal::vespera::c;
use crate::time::Duration;

const NSEC_PER_SEC: u64 = 1_000_000_000;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(crate) struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: Nanoseconds,
}

impl Timespec {
    pub const MAX: Timespec = unsafe { Self::new_unchecked(i64::MAX, 1_000_000_000 - 1) };
    pub const MIN: Timespec = unsafe { Self::new_unchecked(i64::MIN, 0) };

    const unsafe fn new_unchecked(tv_sec: i64, tv_nsec: i64) -> Timespec {
        Timespec { tv_sec, tv_nsec: unsafe { Nanoseconds::new_unchecked(tv_nsec as u32) } }
    }

    pub const fn zero() -> Timespec {
        unsafe { Self::new_unchecked(0, 0) }
    }

    pub const fn new(tv_sec: i64, tv_nsec: i64) -> Result<Timespec, io::Error> {
        if tv_nsec >= 0 && tv_nsec < NSEC_PER_SEC as i64 {
            Ok(unsafe { Self::new_unchecked(tv_sec, tv_nsec) })
        } else {
            Err(io::const_error!(io::ErrorKind::InvalidData, "invalid timestamp"))
        }
    }

    pub fn now(clock: c::clockid_t) -> Timespec {
        let mut t = MaybeUninit::<c::timespec>::uninit();
        let ret = unsafe { c::clock_gettime(clock, t.as_mut_ptr()) };

        assert!(ret == 0, "clock_gettime failed: {ret}");
        let t = unsafe { t.assume_init() };
        Timespec::new(t.tv_sec, t.tv_nsec).expect("clock_gettime returned an invalid timestamp")
    }

    pub fn sub_timespec(&self, other: &Timespec) -> Result<Duration, Duration> {
        fn sub_ge_to_unsigned(a: i64, b: i64) -> u64 {
            debug_assert!(a >= b);
            a.wrapping_sub(b).cast_unsigned()
        }

        if self >= other {
            let (secs, nsec) = if self.tv_nsec.as_inner() >= other.tv_nsec.as_inner() {
                (
                    sub_ge_to_unsigned(self.tv_sec, other.tv_sec),
                    self.tv_nsec.as_inner() - other.tv_nsec.as_inner(),
                )
            } else {
                debug_assert!(self.tv_nsec < other.tv_nsec);
                debug_assert!(self.tv_sec > other.tv_sec);
                debug_assert!(self.tv_sec > i64::MIN);
                (
                    sub_ge_to_unsigned(self.tv_sec - 1, other.tv_sec),
                    self.tv_nsec.as_inner() + (NSEC_PER_SEC as u32) - other.tv_nsec.as_inner(),
                )
            };

            Ok(Duration::new(secs, nsec))
        } else {
            match other.sub_timespec(self) {
                Ok(d) => Err(d),
                Err(d) => Ok(d),
            }
        }
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Timespec> {
        let mut secs = self.tv_sec.checked_add_unsigned(other.as_secs())?;

        let mut nsec = other.subsec_nanos() + self.tv_nsec.as_inner();
        if nsec >= NSEC_PER_SEC as u32 {
            nsec -= NSEC_PER_SEC as u32;
            secs = secs.checked_add(1)?;
        }
        Some(unsafe { Timespec::new_unchecked(secs, nsec.into()) })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Timespec> {
        let mut secs = self.tv_sec.checked_sub_unsigned(other.as_secs())?;

        let mut nsec = self.tv_nsec.as_inner() as i32 - other.subsec_nanos() as i32;
        if nsec < 0 {
            nsec += NSEC_PER_SEC as i32;
            secs = secs.checked_sub(1)?;
        }
        Some(unsafe { Timespec::new_unchecked(secs, nsec.into()) })
    }

    pub fn to_timespec(&self) -> Option<c::timespec> {
        Some(c::timespec {
            tv_sec: self.tv_sec,
            tv_nsec: self.tv_nsec.as_inner() as i64,
        })
    }
}