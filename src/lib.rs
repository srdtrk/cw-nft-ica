#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
