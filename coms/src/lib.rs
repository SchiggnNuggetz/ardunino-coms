#![expect(clippy::multiple_crate_versions, clippy::cargo_common_metadata)]
#![cfg_attr(not(feature = "std"), no_std)]
pub mod encryption;
#[cfg(feature = "std")]
pub mod receiver;
