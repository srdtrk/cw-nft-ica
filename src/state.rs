//! This module defines the state storage of the Contract.

use cosmwasm_schema::cw_serde;

mod contract {
    use cosmwasm_std::Addr;
    use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;

    use super::*;

    #[cw_serde]
    pub struct ContractState {
        /// The default options for new ICA channels.
        pub default_chan_init_options: ChannelOpenInitOptions,
        /// The code ID of the cw-ica-controller contract.
        pub ica_controller_code_id: u64,
        /// The address of the cw721-ica-extension contract.
        pub ica_extension_addr: Addr,
    }
}
