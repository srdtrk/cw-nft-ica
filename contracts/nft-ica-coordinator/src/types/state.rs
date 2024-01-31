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
/// tha map used to store channel status for each token id
pub const CHANNEL_STATE: Map<&str, channel::ChannelState> = Map::new("channel_status");

/// The prefix used to store the transaction history.
const TX_HISTORY_PREFIX: &str = "tx_history_";

/// Returns the key used to store the transaction history of the given token ID.
pub fn get_tx_history_prefix(token_id: &str) -> String {
    format!("{}{}", TX_HISTORY_PREFIX, token_id)
}

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

/// This module contains the types used to store the ICA channel state.
pub mod channel {
    use cosmwasm_schema::cw_serde;

    /// ChannelState is the simplified channel state stored for each token.
    #[cw_serde]
    pub struct ChannelState {
        /// The channel status.
        pub status: ChannelStatus,
        /// The channel ID. This is only set if the channel is not pending.
        pub channel_id: Option<String>,
    }

    /// The status of a channel.
    #[cw_serde]
    pub enum ChannelStatus {
        /// The channel is open.
        Open,
        /// The channel is closed.
        Closed,
        /// The channel is in the process of opening.
        Pending,
    }
}

/// This module contains the types used to store the ICA transaction history.
pub mod history {
    use super::*;
    use cosmwasm_std::{CosmosMsg, StakingMsg};
    use cw_ica_controller::types::msg::ExecuteMsg as IcaControllerExecuteMsg;

    /// Represents the status of a transaction.
    #[cw_serde]
    pub enum TransactionStatus {
        /// The transaction is waiting for a callback from the ICA controller contract.
        Pending,
        /// The transaction has been completed.
        Completed,
        /// The transaction has failed.
        Failed,
        /// The transaction has timed out.
        Timeout,
    }

    /// Represents the type of a transaction message.
    #[cw_serde]
    pub enum TransactionMsgType {
        /// The transaction is empty.
        Empty,
        /// The transaction is a custom message.
        Custom,
        /// The transaction is a [`CosmosMsg::Bank`] message.
        Send,
        /// The transaction is a [`CosmosMsg::Ibc`] message.
        Ibc,
        /// The transaction is a [`CosmosMsg::Gov`] message.
        Vote,
        /// The transaction is a [`CosmosMsg::Wasm`] message.
        Wasm,
        /// The transaction is a [`StakingMsg::Delegate`] message.
        Delegate,
        /// The transaction is a [`StakingMsg::Undelegate`] message.
        Undelegate,
        /// The transaction is a [`StakingMsg::Redelegate`] message.
        Redelegate,
        /// The transaction is a [`CosmosMsg::Stargate`] message.
        Stargate,
        /// The transaction is a [`CosmosMsg::Distribution`] message.
        Distribution,
        /// The transaction has more than one message.
        MultiMsg,
        /// The transaction type cannot be determined.
        Unknown,
    }

    /// Represents a transaction record.
    #[cw_serde]
    pub struct TransactionRecord {
        /// The status of the transaction.
        pub status: TransactionStatus,
        /// The token ID of the NFT.
        pub token_id: String,
        /// The owner of the NFT.
        pub owner: String,
        /// The type of the message sent to the ICA controller contract.
        pub msg_type: TransactionMsgType,
        /// The height of the block when the transaction was sent.
        pub block_height: u64,
        /// The timestamp of the block when the transaction was sent in nanoseconds.
        pub timestamp: u64,
    }

    impl TransactionMsgType {
        /// Returns the [`TransactionMsgType`] of the given [`CosmosMsg`].
        pub const fn from_cosmos_msg(msg: &CosmosMsg) -> Self {
            match msg {
                CosmosMsg::Custom(_) => Self::Custom,
                CosmosMsg::Stargate { .. } => Self::Stargate,
                CosmosMsg::Bank(_) => Self::Send,
                CosmosMsg::Staking(StakingMsg::Delegate { .. }) => Self::Delegate,
                CosmosMsg::Staking(StakingMsg::Undelegate { .. }) => Self::Undelegate,
                CosmosMsg::Staking(StakingMsg::Redelegate { .. }) => Self::Redelegate,
                CosmosMsg::Distribution(_) => Self::Distribution,
                CosmosMsg::Gov(_) => Self::Vote,
                CosmosMsg::Wasm(_) => Self::Wasm,
                CosmosMsg::Ibc(_) => Self::Ibc,
                _ => Self::Unknown,
            }
        }
    }

    impl TransactionRecord {
        /// Creates a new [`TransactionRecord`] from the given [`IcaControllerExecuteMsg`].
        pub fn from_ica_msg(
            msg: &IcaControllerExecuteMsg,
            token_id: impl Into<String>,
            owner: impl Into<String>,
            block_height: u64,
            timestamp: u64,
        ) -> Option<Self> {
            let msg_type = match msg {
                IcaControllerExecuteMsg::SendCosmosMsgs { messages, .. } => {
                    if messages.is_empty() {
                        TransactionMsgType::Empty
                    } else if messages.len() == 1 {
                        TransactionMsgType::from_cosmos_msg(&messages[0])
                    } else {
                        TransactionMsgType::MultiMsg
                    }
                }
                IcaControllerExecuteMsg::SendCustomIcaMessages { .. } => TransactionMsgType::Custom,
                _ => return None,
            };

            Some(Self {
                status: TransactionStatus::Pending,
                token_id: token_id.into(),
                owner: owner.into(),
                msg_type,
                block_height,
                timestamp,
            })
        }
    }
}
