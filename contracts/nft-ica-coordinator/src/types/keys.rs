//! This module contains key constants used by the contract.

/// The name of the contract for cw2.
pub const CONTRACT_NAME: &str = "crates.io:nft-ica";
/// The version of the contract for cw2.
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The prefix used for the NFT-ICA token.
pub const TOKEN_PREFIX: &str = "ica-token";

/// The prefix used by `x/wasm` for IBC ports.
pub const WASM_IBC_PORT_PREFIX: &str = "wasm.";

/// The reply ID used when instantiating the cw721-ica-extension contract.
pub const CW721_INSTANTIATE_REPLY_ID: u64 = 1;

/// The reply ID used when instantiating the cw-ica-controller contract.
pub const CW_ICA_CONTROLLER_INSTANTIATE_REPLY_ID: u64 = 2;
