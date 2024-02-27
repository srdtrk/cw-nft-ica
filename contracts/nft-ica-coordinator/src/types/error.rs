use cosmwasm_std::StdError;
use thiserror::Error;

/// ContractError is the error type returned by contract's functions.
#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Queue empty")]
    QueueEmpty,

    #[error("Channel already open")]
    ChannelAlreadyOpen,

    #[error("Channel state not found")]
    ChannelStateNotFound,

    #[error("Snip721 query failed")]
    Snip721QueryFailed,
}
