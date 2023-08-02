#![doc = include_str!("../README.md")]

#[cfg(target_has_atomic = "ptr")]
mod arc;
#[cfg(target_has_atomic = "ptr")]
mod arc_slice;
#[cfg(target_has_atomic = "ptr")]
mod arc_str;
mod rc;
mod rc_slice;
mod rc_str;
#[cfg(target_has_atomic = "ptr")]
pub use arc_str::*;
pub use rc_slice::*;
pub use rc_str::*;
