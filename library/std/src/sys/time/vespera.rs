// sys/time/vespera.rs

use crate::sys::AsInner;
use crate::sys::pal::c;
use crate::sys::pal::time::Timespec;
use crate::time::Duration;
use crate::{fmt, io};

pub const UNIX_EPOCH: SystemTime = SystemTime { t: Timespec::zero() };

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime {
    pub(crate) t: Timespec,
}

impl SystemTime {
    pub const MAX: SystemTime = SystemTime { t: Timespec::MAX };
    pub const MIN: SystemTime = SystemTime { t: Timespec::MIN };

    pub fn new(tv_sec: i64, tv_nsec: i64) -> Result<SystemTime, io::Error> {
        Ok(SystemTime { t: Timespec::new(tv_sec, tv_nsec)? })
    }

    pub fn now() -> SystemTime {
        SystemTime { t: Timespec::now(c::CLOCK_REALTIME as c::clockid_t) }
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.t.sub_timespec(&other.t)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime { t: self.t.checked_add_duration(other)? })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime { t: self.t.checked_sub_duration(other)? })
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemTime")
            .field("tv_sec", &self.t.tv_sec)
            .field("tv_nsec", &self.t.tv_nsec)
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    t: Timespec,
}

impl Instant {
    // CLOCK_MONOTONIC_RAW waere ideal (unbeeinflusst von NTP-Anpassungen),
    // aber CLOCK_MONOTONIC ist der konventionelle, breiter unterstuetzte
    // Standard fuer Instant -- entspricht dem Verhalten der meisten
    // Nicht-Apple Unix-Targets in der Referenzimplementierung.
    pub(crate) const CLOCK_ID: c::clockid_t = c::CLOCK_MONOTONIC as c::clockid_t;

    pub fn now() -> Instant {
        Instant { t: Timespec::now(Self::CLOCK_ID) }
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        self.t.sub_timespec(&other.t).ok()
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant { t: self.t.checked_add_duration(other)? })
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant { t: self.t.checked_sub_duration(other)? })
    }

    pub(crate) fn into_timespec(self) -> Timespec {
        self.t
    }
}

impl AsInner<Timespec> for Instant {
    fn as_inner(&self) -> &Timespec {
        &self.t
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instant")
            .field("tv_sec", &self.t.tv_sec)
            .field("tv_nsec", &self.t.tv_nsec)
            .finish()
    }
}