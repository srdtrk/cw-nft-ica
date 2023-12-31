use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_ownable::OwnershipError;
use thiserror::Error;

/// ContractError is the error type returned by contract's functions.
#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("error when computing the instantiate2 address: {0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Queue empty")]
    QueueEmpty,

    #[error("Channel already open")]
    ChannelAlreadyOpen,

    #[error("Channel state not found")]
    ChannelStateNotFound,
}
