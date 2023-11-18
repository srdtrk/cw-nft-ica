//! # Messages
//!
//! This module defines the messages the ICA controller contract receives.

use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;

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
    pub default_chan_init_options: ChannelOpenInitOptions
}

/// This is the execution message for the contract.
#[cw_serde]
pub enum ExecuteMsg {}

/// This is the query message for the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
