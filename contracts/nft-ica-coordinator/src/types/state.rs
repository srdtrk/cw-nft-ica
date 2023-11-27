//! This module defines the state storage of the Contract.

use cosmwasm_schema::cw_serde;

pub use contract::ContractState;
use cosmwasm_std::Addr;
use cw_storage_plus::{Deque, Item, Map};
pub use mint::QueueItem;

use crate::utils::storage::{KeySet, NftIcaBiMap};

/// The item used to store the state of the IBC application.
pub const STATE: Item<ContractState> = Item::new("state");
/// The map used to map nft token ids to ICA addresses.
pub const NFT_ICA_MAP: Map<&str, String> = Map::new("nft_ica_map");
/// The keyset used to store the registered ICA addresses to accept callbacks from.
pub const REGISTERED_ICA_ADDRS: KeySet<&Addr> = KeySet::new("registered_ica");
/// The item used to store the bi-directional map between cw-ica-controller address and NFT IDs.
pub const NFT_ICA_CONTRACT_BI_MAP: NftIcaBiMap = NftIcaBiMap::new("nft_ica_contract_bi_map");
/// NFT_MINT_QUEUE is the queue of NFT mint requests, waiting for a callback from the ICA controller contract.
pub const NFT_MINT_QUEUE: Deque<mint::QueueItem> = Deque::new("nft_mint_queue");
/// The item used to store the NFT-ICA counter.
pub const TOKEN_COUNTER: Item<u64> = Item::new("ica_nft_counter");

mod contract {
    use super::*;

    use cosmwasm_std::Addr;
    use cw_ica_controller::types::msg::options::ChannelOpenInitOptions;

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

mod mint {
    use super::*;

    /// The item used to store the mint queue.
    #[cw_serde]
    pub struct QueueItem {
        /// The token ID of the NFT.
        pub token_id: String,
        /// The owner of the NFT.
        pub owner: String,
    }
}
