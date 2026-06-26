#![unstable(feature = "vespera_platform", issue = "none")]

#[path = "../unix/ffi/os_str.rs"]
mod os_str;

#[unstable(feature = "vespera_platform", issue = "none")]
pub use self::os_str::{OsStrExt, OsStringExt};