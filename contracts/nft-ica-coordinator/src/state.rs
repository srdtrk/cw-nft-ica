//! This module defines the state storage of the Contract.

use cosmwasm_schema::cw_serde;

pub use contract::ContractState;
use cw_storage_plus::Item;

/// The item used to store the state of the IBC application.
pub const STATE: Item<ContractState> = Item::new("state");

mod contract {
    use cosmwasm_std::Addr;
    use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;

    use super::*;

    /// The state of the contract.
    #[cw_serde]
    pub struct ContractState {
        /// The default options for new ICA channels.
        pub default_chan_init_options: ChannelOpenInitOptions,
        /// The code ID of the cw-ica-controller contract.
        pub ica_controller_code_id: u64,
        /// The address of the cw721-ica-extension contract.
        pub cw721_ica_extension_address: Addr,
    }
}
