#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[cfg(not(feature = "library"))]
pub mod contract;
mod error;
pub mod msg;
pub mod state;
pub mod utils;

pub use crate::error::ContractError;
