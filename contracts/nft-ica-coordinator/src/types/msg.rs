//! # Messages
//!
//! This module defines the messages the ICA controller contract receives.

use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ica_controller::types::{
    callbacks::IcaControllerCallbackMsg,
    msg::{options::ChannelOpenInitOptions, ExecuteMsg as IcaControllerExecuteMsg},
};

/// This is the instantiation message for the contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// The owner of the contract. If not set, the sender of the
    /// instantiation message is the owner.
    #[serde(default)]
    pub owner: Option<String>,
    /// The code ID of the ICA controller contract.
    pub ica_controller_code_id: u64,
    /// The code ID of the Cw721 ICA extension contract.
    pub cw721_ica_extension_code_id: u64,
    /// The default channel open init options for interchain accounts.
    pub default_chan_init_options: ChannelOpenInitOptions,
    /// The optional salt used to generate the cw721 ICA extension
    /// contract address.
    #[serde(default)]
    pub salt: Option<String>,
}

/// This is the execution message for the contract.
#[cw_ownable::cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// MintIca creates a new ICA for the caller.
    /// The NFT is minted after a callback from the ICA controller contract.
    MintIca {
        /// The optional salt used to generate the cw721 ICA extension
        /// contract address.
        #[serde(default)]
        salt: Option<String>,
    },
    /// ReceiveIcaCallback is the message sent by the ICA controller contract
    /// on packet and channel lifecycle events.
    ReceiveIcaCallback(IcaControllerCallbackMsg),
    /// ExecuteIcaMsg allows the owner of the ICA NFT to send a custom message.
    /// This is directly forwarded to the ICA controller contract after authorization.
    ExecuteIcaMsg {
        /// The token ID of the ICA NFT.
        token_id: String,
        /// The custom message to send to the ICA controller contract.
        msg: IcaControllerExecuteMsg,
    },
}

/// This is the query message for the contract.
#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// GetContractState returns the contact's state.
    #[returns(crate::types::state::ContractState)]
    GetContractState {},
    /// NftIcaBimap queries the ICA NFT ID to ICA ID mapping.
    #[returns(String)]
    NftIcaBimap {
        /// The token ID or ICA address to query.
        key: String
    },
}