//! This module handles the execution logic of the contract.

use cosmwasm_std::{entry_point, Addr, Reply, StdError};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::types::keys::{
    self, CW721_INSTANTIATE_REPLY_ID, CW_ICA_CONTROLLER_INSTANTIATE_REPLY_ID,
};
use crate::types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::state::{self, ContractState, STATE};
use crate::types::ContractError;

/// Instantiate the contract.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = msg.owner.unwrap_or(info.sender.to_string());
    let owner = deps.api.addr_validate(&owner)?;

    state::OWNER.save(deps.storage, &owner)?;

    let instantiate_submsg =
        instantiate::instantiate_snip721(env, msg.snip721_code.code_id, msg.snip721_code.code_hash.clone())?;

    let state = ContractState {
        default_chan_init_options: msg.default_chan_init_options,
        ica_controller_code: msg.ica_controller_code,
        // TODO: remove this once injective supports instantiate2 (There is already a branch which supports it).
        // Must be filled in by the reply from the cw721-ica-extension contract.
        snip721_address: Addr::unchecked("".to_string()),
        snip721_code_hash: msg.snip721_code.code_hash,
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_submessage(instantiate_submsg))
}

/// Execute the contract.
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwnership { owner } => execute::update_ownership(deps, info, owner),
        ExecuteMsg::ReceiveIcaCallback(callback) => {
            execute::receive_ica_callback(deps, info, callback)
        }
        ExecuteMsg::MintIca { salt } => execute::mint_ica(deps, env, info, salt),
        ExecuteMsg::ExecuteIcaMsg { token_id, msg } => {
            execute::ica_msg(deps, env, info, token_id, msg)
        }
    }
}

/// Query the contract.
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_binary(&query::owner(deps)?),
        QueryMsg::GetContractState {} => to_binary(&query::state(deps)?),
        QueryMsg::NftIcaControllerBimap { key } => {
            to_binary(&query::nft_ica_controller_bimap(deps, key)?)
        }
        QueryMsg::GetIcaAddress { token_id } => {
            to_binary(&query::get_ica_address(deps, token_id)?)
        }
        QueryMsg::GetIcaAddresses { token_ids } => {
            to_binary(&query::get_ica_addresses(deps, token_ids)?)
        }
        QueryMsg::GetMintQueue {} => to_binary(&query::get_mint_queue(deps)?),
        QueryMsg::GetTransactionHistory {
            token_id,
            page,
            page_size,
        } => to_binary(&query::get_transaction_history(
            deps, token_id, page, page_size,
        )?),
        QueryMsg::GetChannelState { token_id } => {
            to_binary(&query::get_channel_state(deps, token_id)?)
        }
    }
}

/// Reply to a submessage.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        CW721_INSTANTIATE_REPLY_ID => reply::cw721_instantiate(deps, msg),
        CW_ICA_CONTROLLER_INSTANTIATE_REPLY_ID => reply::cw_ica_controller_instantiate(deps, msg),
        id => Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}

mod instantiate {
    use crate::types::keys::CW721_INSTANTIATE_REPLY_ID;

    use super::*;

    use cosmwasm_std::{SubMsg, WasmMsg};

    /// Instantiate the cw721-ica-extension contract using the submessage pattern.
    /// Returns the instantiate submessage whose reply will contain the new contract address.
    pub fn instantiate_snip721(
        env: Env,
        code_id: u64,
        code_hash: String,
    ) -> Result<SubMsg, ContractError> {
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            code_hash,
            msg: to_binary(&snip721_reference_impl::msg::InstantiateMsg {
                name: "NFT-ICA".to_string(),
                symbol: "ICA".to_string(),
                admin: Some(env.contract.address.to_string()),
                entropy: env.block.time.seconds().to_string(),
                royalty_info: None,
                config: Some(snip721_reference_impl::msg::InstantiateConfig{
                    public_owner: Some(true),
                    public_token_supply: Some(true),
                    enable_sealed_metadata: Some(false),
                    unwrapped_metadata_is_private: Some(false),
                    minter_may_update_metadata: Some(true),
                    owner_may_update_metadata: Some(true),
                    enable_burn: Some(false),
                }),
                post_init_callback: None,
            })?,
            label: format!("cw721-ica-{}", env.block.height),
            funds: vec![],
        };

        Ok(SubMsg::reply_on_success(
            instantiate_msg,
            CW721_INSTANTIATE_REPLY_ID,
        ))
    }
}

mod execute {
    use super::*;

    use cosmwasm_std::{Addr, CosmosMsg, SubMsg, WasmMsg};
    use cw_ica_controller::{
        helpers::CwIcaControllerContract,
        ibc::types::packet::acknowledgement::Data,
        types::{
            callbacks::IcaControllerCallbackMsg,
            msg::{options::ChannelOpenInitOptions, CallbackInfo, ExecuteMsg as IcaControllerExecuteMsg},
        },
    };
    use snip721_reference_impl::msg::QueryAnswer;
    use secret_toolkit::storage::DequeStore as Deque;
    use secret_toolkit::serialization::Json;

    use crate::
        types::{
            keys::CW_ICA_CONTROLLER_INSTANTIATE_REPLY_ID,
            state::{
                channel::{ChannelState, ChannelStatus},
                get_tx_history_prefix,
                history::{TransactionRecord, TransactionStatus},
                QueueItem, CHANNEL_STATE, NFT_ICA_CONTRACT_BI_MAP, NFT_ICA_MAP, NFT_MINT_QUEUE,
                REGISTERED_ICA_ADDRS, TOKEN_COUNTER,
            },
        };

    /// Update the ownership of the contract.
    pub fn update_ownership(
        deps: DepsMut,
        info: MessageInfo,
        owner: String,
    ) -> Result<Response, ContractError> {
        state::assert_owner(deps.storage, info.sender)?;
        let owner = deps.api.addr_validate(&owner)?;

        state::OWNER.save(deps.storage, &owner)?;

        Ok(Response::default())
    }

    /// Mint a new ICA for the caller.
    pub fn mint_ica(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        _salt: Option<String>,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let ica_count = TOKEN_COUNTER.may_load(deps.storage)?.unwrap_or_default();

        let queue_item = QueueItem {
            token_id: format!("{}-{}", keys::TOKEN_PREFIX, ica_count),
            owner: info.sender.to_string(),
        };

        NFT_MINT_QUEUE.push_front(deps.storage, &queue_item)?;
        TOKEN_COUNTER.save(deps.storage, &(ica_count + 1))?;

        let instantiate_submsg = instantiate_cw_ica_controller(
            env,
            state.ica_controller_code.code_id,
            state.ica_controller_code.code_hash,
            state.default_chan_init_options,
        )?;

        Ok(Response::new().add_submessage(instantiate_submsg))
    }

    pub fn receive_ica_callback(
        deps: DepsMut,
        info: MessageInfo,
        callback: IcaControllerCallbackMsg,
    ) -> Result<Response, ContractError> {
        if !REGISTERED_ICA_ADDRS.contains(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized);
        };

        match callback {
            IcaControllerCallbackMsg::OnChannelOpenAckCallback {
                ica_address,
                channel,
                ..
            } => match NFT_ICA_CONTRACT_BI_MAP.may_load(deps.storage, info.sender.as_str())? {
                Some(token_id) => {
                    let channel_state = CHANNEL_STATE.get(deps.storage, &token_id).ok_or(StdError::not_found("channel state"))?;
                    if channel_state.status == ChannelStatus::Open {
                        return Err(ContractError::ChannelAlreadyOpen);
                    };

                    CHANNEL_STATE.insert(
                        deps.storage,
                        &token_id,
                        &ChannelState {
                            status: ChannelStatus::Open,
                            channel_id: Some(channel.endpoint.channel_id),
                        },
                    )?;

                    Ok(Response::default())
                }
                None => {
                    let queue_item = NFT_MINT_QUEUE
                        .pop_back(deps.storage)?;

                    let state = STATE.load(deps.storage)?;

                    NFT_ICA_CONTRACT_BI_MAP.insert(
                        deps.storage,
                        info.sender.as_str(),
                        &queue_item.token_id,
                    )?;

                    NFT_ICA_MAP.insert(deps.storage, &queue_item.token_id, &ica_address)?;
                    CHANNEL_STATE.insert(
                        deps.storage,
                        &queue_item.token_id,
                        &ChannelState {
                            status: ChannelStatus::Open,
                            channel_id: Some(channel.endpoint.channel_id),
                        },
                    )?;

                    let msg = snip721_reference_impl::msg::ExecuteMsg::MintNft {
                        token_id: Some(queue_item.token_id),
                        owner: Some(queue_item.owner),
                        public_metadata: None,
                        private_metadata: None,
                        serial_number: None,
                        royalty_info: None,
                        padding: None,
                        transferable: Some(true),
                        memo: None,
                    };

                    let cosmos_msg: CosmosMsg = WasmMsg::Execute {
                        contract_addr: state.snip721_address.to_string(),
                        code_hash: state.snip721_code_hash.to_string(),
                        msg: to_binary(&msg)?,
                        funds: vec![],
                    }
                    .into();

                    Ok(Response::new().add_message(cosmos_msg))
                }
            },
            IcaControllerCallbackMsg::OnAcknowledgementPacketCallback {
                original_packet,
                ica_acknowledgement,
                ..
            } => {
                let maybe_controller = original_packet
                    .src
                    .port_id
                    .strip_prefix(keys::WASM_IBC_PORT_PREFIX);
                if let Some(controller_addr) = maybe_controller {
                    let token_id = NFT_ICA_CONTRACT_BI_MAP.load(deps.storage, controller_addr)?;
                    let prefix = get_tx_history_prefix(&token_id);
                    let records_store: Deque<TransactionRecord, Json> = Deque::new(prefix.as_bytes());
                    let mut record = records_store
                        .pop_front(deps.storage)?;
                    record.status = match ica_acknowledgement {
                        Data::Result(_) => TransactionStatus::Completed,
                        Data::Error(_) => TransactionStatus::Failed,
                    };
                    records_store.push_front(deps.storage, &record)?;
                }

                Ok(Response::default())
            }
            IcaControllerCallbackMsg::OnTimeoutPacketCallback {
                original_packet, ..
            } => {
                let maybe_controller = original_packet
                    .src
                    .port_id
                    .strip_prefix(keys::WASM_IBC_PORT_PREFIX);

                if let Some(controller_addr) = maybe_controller {
                    let token_id = NFT_ICA_CONTRACT_BI_MAP.load(deps.storage, controller_addr)?;

                    let prefix = get_tx_history_prefix(&token_id);
                    let records_store: Deque<TransactionRecord, Json> = Deque::new(prefix.as_bytes());
                    let mut record = records_store
                        .pop_front(deps.storage)?;
                    record.status = TransactionStatus::Timeout;
                    records_store.push_front(deps.storage, &record)?;

                    let mut chan_state = CHANNEL_STATE.get(deps.storage, &token_id).ok_or(StdError::not_found("channel state"))?;

                    // Recreate this without update:
                    //
                    // CHANNEL_STATE.update(deps.storage, &token_id, |maybe_cs| {
                    //     if let Some(mut cs) = maybe_cs {
                    //         cs.status = ChannelStatus::Closed;
                    //         Ok(cs)
                    //     } else {
                    //         Err(ContractError::ChannelStateNotFound)
                    //     }
                    // })?;

                    chan_state.status = ChannelStatus::Closed;
                    CHANNEL_STATE.insert(deps.storage, &token_id, &chan_state)?;
                }

                Ok(Response::default())
            }
        }
    }

    /// Execute a message on the ICA contract if the sender is the owner of the ica token.
    pub fn ica_msg(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
        msg: IcaControllerExecuteMsg,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        // verify that the sender is the owner of the token
        let snip721_owner_query = snip721_reference_impl::msg::QueryMsg::OwnerOf {
            token_id: token_id.clone(), viewer: None, include_expired: None,
        };

        let resp: QueryAnswer = deps.querier.query_wasm_smart(state.snip721_code_hash, state.snip721_address, &snip721_owner_query)?;

        let owner = if let QueryAnswer::OwnerOf { owner, .. } = resp {
            owner
        } else {
            return Err(ContractError::Snip721QueryFailed);
        };

        if owner != info.sender {
            return Err(ContractError::Unauthorized);
        };

        let ica_address = Addr::unchecked(NFT_ICA_CONTRACT_BI_MAP.load(deps.storage, &token_id)?);
        // additional hardening check
        if !REGISTERED_ICA_ADDRS.contains(deps.storage, &ica_address) {
            return Err(ContractError::Unauthorized);
        };

        // Set channel status to pending if the message is a create channel message.
        if matches!(msg, IcaControllerExecuteMsg::CreateChannel { .. })
            && matches!(
                CHANNEL_STATE.get(deps.storage, &token_id).ok_or(StdError::not_found("channel state"))?.status,
                ChannelStatus::Closed
            )
        {
            CHANNEL_STATE.insert(
                deps.storage,
                &token_id,
                &ChannelState {
                    status: ChannelStatus::Pending,
                    channel_id: None,
                },
            )?;
        }

        if let Some(tx_record) = TransactionRecord::from_ica_msg(
            &msg,
            &token_id,
            owner,
            env.block.height,
            env.block.time.nanos(),
        ) {
            let prefix = get_tx_history_prefix(&token_id);
            let records_store: Deque<TransactionRecord, Json> = Deque::new(prefix.as_bytes());
            records_store.push_front(deps.storage, &tx_record)?;
        }

        let cw_ica_controller = CwIcaControllerContract::new(ica_address, state.ica_controller_code.code_hash);
        let cosmos_msg = cw_ica_controller.call(msg)?;

        Ok(Response::new().add_message(cosmos_msg))
    }

    /// Instantiate the cw721-ica-extension contract using the submessage pattern.
    /// Returns the instantiate submessage whose reply will contain the new contract address.
    pub fn instantiate_cw_ica_controller(
        env: Env,
        code_id: u64,
        code_hash: String,
        channel_open_init_options: ChannelOpenInitOptions,
    ) -> Result<SubMsg, ContractError> {
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            code_hash,
            msg: to_binary(&cw_ica_controller::types::msg::InstantiateMsg {
                owner: Some(env.contract.address.to_string()),
                channel_open_init_options,
                send_callbacks_to: Some(
                    CallbackInfo {
                        address: env.contract.address.to_string(),
                        code_hash: env.contract.code_hash,
                    }
                ),
            })?,
            label: format!("cw-ica-controller-{}", env.block.height),
            funds: vec![],
        };

        Ok(SubMsg::reply_on_success(
            instantiate_msg,
            CW_ICA_CONTROLLER_INSTANTIATE_REPLY_ID,
        ))
    }
}

mod query {
    use self::state::OWNER;

    use super::*;

    use crate::types::{
        msg::query_responses::{
            GetIcaAddressesResponse, GetTransactionHistoryResponse, NftIcaPair,
        },
        state::{
            channel::ChannelState, get_tx_history_prefix, history::TransactionRecord, QueueItem,
            CHANNEL_STATE, NFT_ICA_CONTRACT_BI_MAP, NFT_ICA_MAP, NFT_MINT_QUEUE,
        },
    };

    use cosmwasm_std::StdResult;
    use secret_toolkit::{serialization::Json, storage::DequeStore as Deque};

    /// Query the contract state.
    pub fn state(deps: Deps) -> StdResult<ContractState> {
        STATE.load(deps.storage)
    }

    pub fn owner(deps: Deps) -> StdResult<String> {
        OWNER.load(deps.storage).map(|addr| addr.to_string())
    }

    /// Query the ICA NFT ID to ICA ID mapping.
    pub fn nft_ica_controller_bimap(deps: Deps, key: String) -> StdResult<String> {
        NFT_ICA_CONTRACT_BI_MAP.load(deps.storage, key)
    }

    /// Query the ICA controller address for a given ICA NFT ID.
    pub fn get_ica_address(deps: Deps, token_id: String) -> StdResult<String> {
        NFT_ICA_MAP.get(deps.storage, &token_id).ok_or(StdError::not_found("nft-ica mapping"))
    }

    /// Query the ICA controller addresses for a given list of ICA NFT IDs.
    pub fn get_ica_addresses(
        deps: Deps,
        token_ids: Vec<String>,
    ) -> StdResult<GetIcaAddressesResponse> {
        let nft_ica_pairs =
            token_ids
                .iter()
                .try_fold(Vec::new(), |mut acc, token_id| -> StdResult<_> {
                    let ica_address = NFT_ICA_MAP.get(deps.storage, token_id).unwrap();
                    acc.push(NftIcaPair {
                        nft_id: token_id.to_string(),
                        ica_address,
                    });
                    Ok(acc)
                })?;

        Ok(GetIcaAddressesResponse {
            pairs: nft_ica_pairs,
        })
    }

    /// Query the mint queue.
    pub fn get_mint_queue(deps: Deps) -> StdResult<Vec<QueueItem>> {
        NFT_MINT_QUEUE.iter(deps.storage)?.collect()
    }

    /// Query the transaction history for a given NFT ID.
    pub fn get_transaction_history(
        deps: Deps,
        token_id: String,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> StdResult<GetTransactionHistoryResponse> {
        let page = page.unwrap_or(0);
        // using 30 as the default page size
        let page_size = page_size.unwrap_or(30);

        let start = (page * page_size) as usize;
        let end = start + page_size as usize;

        let prefix = get_tx_history_prefix(&token_id);
        let records_store: Deque<TransactionRecord, Json> = Deque::new(prefix.as_bytes());

        let tx_records = records_store
            .iter(deps.storage)?
            .skip(start)
            .take(end)
            .try_fold(Vec::new(), |mut acc, maybe_record| -> StdResult<_> {
                acc.push(maybe_record?);
                Ok(acc)
            });

        Ok(GetTransactionHistoryResponse {
            records: tx_records?,
            total: records_store.get_len(deps.storage)?,
        })
    }

    pub fn get_channel_state(deps: Deps, token_id: String) -> StdResult<ChannelState> {
        CHANNEL_STATE.get(deps.storage, &token_id).ok_or(StdError::not_found("channel state"))
    }
}

mod reply {
    use cosmwasm_std::SubMsgResult;

    use crate::types::state::REGISTERED_ICA_ADDRS;

    use super::*;

    pub fn cw721_instantiate(deps: DepsMut, msg: Reply) -> StdResult<Response> {
        match msg.result {
            SubMsgResult::Ok(reply) => {
                let event = reply
                    .events
                    .iter()
                    .find(|e| {
                        e.ty == "instantiate"
                            || e.ty == "cosmwasm.wasm.v1.EventContractInstantiated"
                    })
                    .ok_or_else(|| StdError::generic_err("instantiate event not found"))?;
                let maybe_address = &event
                    .attributes
                    .iter()
                    .find(|a| a.key == "_contract_address" || a.key == "contract_address")
                    .ok_or_else(|| StdError::generic_err("contract_address attribute not found"))?
                    .value;

                // added this to remove the quotes from the address in injective
                let addr = deps.api.addr_validate(
                    maybe_address
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                        .as_str(),
                )?;

                STATE.update(deps.storage, |mut cs| -> StdResult<_> {
                    cs.snip721_address = addr;
                    Ok(cs)
                })?;

                Ok(Response::new())
            }
            SubMsgResult::Err(err) => Err(StdError::generic_err(err)),
        }
    }

    pub fn cw_ica_controller_instantiate(deps: DepsMut, msg: Reply) -> StdResult<Response> {
        match msg.result {
            SubMsgResult::Ok(reply) => {
                let event = reply
                    .events
                    .iter()
                    .find(|e| {
                        e.ty == "instantiate"
                            || e.ty == "cosmwasm.wasm.v1.EventContractInstantiated"
                    })
                    .ok_or_else(|| StdError::generic_err("instantiate event not found"))?;
                let maybe_address = &event
                    .attributes
                    .iter()
                    .find(|a| a.key == "_contract_address" || a.key == "contract_address")
                    .ok_or_else(|| StdError::generic_err("contract_address attribute not found"))?
                    .value;

                // added this to remove the quotes from the address in injective
                let addr = deps.api.addr_validate(
                    maybe_address
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                        .as_str(),
                )?;

                REGISTERED_ICA_ADDRS.insert(deps.storage, &addr)?;

                // Save res.contract_address
                Ok(Response::new())
            }
            SubMsgResult::Err(err) => Err(StdError::generic_err(err)),
        }
    }
}

#[cfg(test)]
mod tests {}
