//! This module handles the execution logic of the contract.

use cosmwasm_std::{entry_point, Addr, Reply, StdError};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::types::keys::{
    self, CW721_INSTANTIATE_REPLY_ID, CW_ICA_CONTROLLER_INSTANTIATE_REPLY_ID,
};
use crate::types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::state::{ContractState, STATE};
use crate::types::ContractError;

/// Instantiate the contract.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, keys::CONTRACT_NAME, keys::CONTRACT_VERSION)?;

    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;

    let instantiate_submsg =
        instantiate::instantiate_cw721_ica_extension(env, msg.cw721_ica_extension_code_id)?;

    let state = ContractState {
        default_chan_init_options: msg.default_chan_init_options,
        ica_controller_code_id: msg.ica_controller_code_id,
        // TODO: remove this once injective supports instantiate2 (There is already a branch which supports it).
        // Must be filled in by the reply from the cw721-ica-extension contract.
        cw721_ica_extension_address: Addr::unchecked("".to_string()),
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
        ExecuteMsg::UpdateOwnership(action) => execute::update_ownership(deps, env, info, action),
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
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
        QueryMsg::GetContractState {} => to_json_binary(&query::state(deps)?),
        QueryMsg::NftIcaControllerBimap { key } => {
            to_json_binary(&query::nft_ica_controller_bimap(deps, key)?)
        }
        QueryMsg::GetIcaAddress { token_id } => {
            to_json_binary(&query::get_ica_address(deps, token_id)?)
        }
        QueryMsg::GetIcaAddresses { token_ids } => {
            to_json_binary(&query::get_ica_addresses(deps, token_ids)?)
        }
        QueryMsg::GetMintQueue {} => to_json_binary(&query::get_mint_queue(deps)?),
        QueryMsg::GetTransactionHistory {
            token_id,
            page,
            page_size,
        } => to_json_binary(&query::get_transaction_history(
            deps, token_id, page, page_size,
        )?),
        QueryMsg::GetChannelStatus { token_id } => {
            to_json_binary(&query::get_channel_status(deps, token_id)?)
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
    use crate::{types::keys::CW721_INSTANTIATE_REPLY_ID, utils};

    use super::*;

    use cosmwasm_std::{Addr, Api, CosmosMsg, QuerierWrapper, SubMsg, WasmMsg};

    /// Instantiate the cw721-ica-extension contract using the instantiate2 pattern.
    /// Returns the instantiate2 message and the contract address.
    ///
    /// This is ignored since injective doesn't seem to support instantiate2.
    pub fn instantiate2_cw721_ica_extension(
        api: &dyn Api,
        querier: QuerierWrapper,
        env: Env,
        code_id: u64,
        salt: Option<String>,
    ) -> Result<(CosmosMsg, Addr), ContractError> {
        let instantiate_msg = to_json_binary(&cw721_base::InstantiateMsg {
            name: "NFT-ICA".to_string(),
            symbol: "ICA".to_string(),
            minter: env.contract.address.to_string(),
        })?;

        let label = format!("cw721-ica-{}", env.block.height);

        utils::instantiate2_contract(api, querier, env, code_id, salt, label, instantiate_msg)
    }

    /// Instantiate the cw721-ica-extension contract using the submessage pattern.
    /// Returns the instantiate submessage whose reply will contain the new contract address.
    pub fn instantiate_cw721_ica_extension(
        env: Env,
        code_id: u64,
    ) -> Result<SubMsg, ContractError> {
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&cw721_base::InstantiateMsg {
                name: "NFT-ICA".to_string(),
                symbol: "ICA".to_string(),
                minter: env.contract.address.to_string(),
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

    use cosmwasm_std::{Addr, Api, CosmosMsg, QuerierWrapper, SubMsg, WasmMsg};
    use cw721_ica_extension::{helpers::new_cw721_ica_extension_helper, Extension};
    use cw_ica_controller::{
        helpers::CwIcaControllerContract,
        ibc::types::packet::acknowledgement::Data,
        types::{
            callbacks::IcaControllerCallbackMsg,
            msg::{options::ChannelOpenInitOptions, ExecuteMsg as IcaControllerExecuteMsg},
        },
    };
    use cw_storage_plus::Deque;

    use crate::{
        types::{
            keys::CW_ICA_CONTROLLER_INSTANTIATE_REPLY_ID,
            state::{
                channel::ChannelStatus,
                get_tx_history_prefix,
                history::{TransactionRecord, TransactionStatus},
                QueueItem, CHANNEL_STATUS, NFT_ICA_CONTRACT_BI_MAP, NFT_ICA_MAP, NFT_MINT_QUEUE,
                REGISTERED_ICA_ADDRS, TOKEN_COUNTER,
            },
        },
        utils,
    };

    /// Update the ownership of the contract.
    pub fn update_ownership(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        action: cw_ownable::Action,
    ) -> Result<Response, ContractError> {
        cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
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
            state.ica_controller_code_id,
            Some(state.default_chan_init_options),
        )?;

        Ok(Response::new().add_submessage(instantiate_submsg))
    }

    pub fn receive_ica_callback(
        deps: DepsMut,
        info: MessageInfo,
        callback: IcaControllerCallbackMsg,
    ) -> Result<Response, ContractError> {
        if !REGISTERED_ICA_ADDRS.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized);
        };

        match callback {
            IcaControllerCallbackMsg::OnChannelOpenAckCallback { ica_address, .. } => {
                match NFT_ICA_CONTRACT_BI_MAP.may_load(deps.storage, info.sender.as_str())? {
                    Some(token_id) => {
                        if CHANNEL_STATUS.load(deps.storage, &token_id)? == ChannelStatus::Open {
                            return Err(ContractError::ChannelAlreadyOpen);
                        };

                        CHANNEL_STATUS.save(
                            deps.storage,
                            &token_id,
                            &ChannelStatus::Open,
                        )?;

                        Ok(Response::default())
                    }
                    None => {
                        let queue_item = NFT_MINT_QUEUE
                            .pop_back(deps.storage)?
                            .ok_or(ContractError::QueueEmpty)?;

                        let cw721_ica_extension_address =
                            STATE.load(deps.storage)?.cw721_ica_extension_address;

                        NFT_ICA_CONTRACT_BI_MAP.insert(
                            deps.storage,
                            info.sender.as_str(),
                            &queue_item.token_id,
                        )?;

                        NFT_ICA_MAP.save(deps.storage, &queue_item.token_id, &ica_address)?;
                        CHANNEL_STATUS.save(
                            deps.storage,
                            &queue_item.token_id,
                            &ChannelStatus::Open,
                        )?;

                        let msg = cw721_ica_extension::ExecuteMsg::Mint {
                            token_id: queue_item.token_id,
                            owner: queue_item.owner,
                            token_uri: None,
                            extension: Extension {
                                ica_controller_address: info.sender,
                                ica_address,
                            },
                        };

                        let cosmos_msg: CosmosMsg = WasmMsg::Execute {
                            contract_addr: cw721_ica_extension_address.to_string(),
                            msg: to_json_binary(&msg)?,
                            funds: vec![],
                        }
                        .into();

                        Ok(Response::new().add_message(cosmos_msg))
                    }
                }
            }
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
                    let records_store: Deque<TransactionRecord> = Deque::new(&prefix);
                    let mut record = records_store
                        .pop_front(deps.storage)?
                        .ok_or(ContractError::QueueEmpty)?;
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
                    let records_store: Deque<TransactionRecord> = Deque::new(&prefix);
                    let mut record = records_store
                        .pop_front(deps.storage)?
                        .ok_or(ContractError::QueueEmpty)?;
                    record.status = TransactionStatus::Timeout;
                    records_store.push_front(deps.storage, &record)?;

                    CHANNEL_STATUS.save(deps.storage, &token_id, &ChannelStatus::Closed)?;
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
        let cw721_ica_extension = new_cw721_ica_extension_helper(state.cw721_ica_extension_address);
        let owner = cw721_ica_extension
            .owner_of(&deps.querier, &token_id, false)?
            .owner;

        if owner != info.sender {
            return Err(ContractError::Unauthorized);
        };

        let ica_address = Addr::unchecked(NFT_ICA_CONTRACT_BI_MAP.load(deps.storage, &token_id)?);
        // additional hardening check
        if !REGISTERED_ICA_ADDRS.has(deps.storage, &ica_address) {
            return Err(ContractError::Unauthorized);
        };

        // Set channel status to pending if the message is a create channel message.
        if matches!(msg, IcaControllerExecuteMsg::CreateChannel { .. })
            && matches!(
                CHANNEL_STATUS.load(deps.storage, &token_id)?,
                ChannelStatus::Closed
            )
        {
            CHANNEL_STATUS.save(deps.storage, &token_id, &ChannelStatus::Pending)?;
        }

        if let Some(tx_record) = TransactionRecord::from_ica_msg(
            &msg,
            &token_id,
            owner,
            env.block.height,
            env.block.time.nanos(),
        ) {
            let prefix = get_tx_history_prefix(&token_id);
            let records_store: Deque<TransactionRecord> = Deque::new(&prefix);
            records_store.push_front(deps.storage, &tx_record)?;
        }

        let cw_ica_controller = CwIcaControllerContract::new(ica_address);
        let cosmos_msg = cw_ica_controller.call(msg)?;

        Ok(Response::new().add_message(cosmos_msg))
    }

    /// Instantiate the cw721-ica extension contract using the instantiate2 pattern.
    /// Returns the instantiate2 message and the contract address.
    ///
    /// This is ignored since injective doesn't seem to support instantiate2.
    fn instantiate2_cw_ica_controller(
        api: &dyn Api,
        querier: QuerierWrapper,
        env: Env,
        code_id: u64,
        salt: Option<String>,
        channel_open_init_options: Option<ChannelOpenInitOptions>,
    ) -> Result<(CosmosMsg, Addr), ContractError> {
        let instantiate_msg = to_json_binary(&cw_ica_controller::types::msg::InstantiateMsg {
            owner: Some(env.contract.address.to_string()),
            channel_open_init_options,
            send_callbacks_to: Some(env.contract.address.to_string()),
        })?;

        let label = format!("cw-ica-controller-{}", env.block.height);

        utils::instantiate2_contract(api, querier, env, code_id, salt, label, instantiate_msg)
    }

    /// Instantiate the cw721-ica-extension contract using the submessage pattern.
    /// Returns the instantiate submessage whose reply will contain the new contract address.
    pub fn instantiate_cw_ica_controller(
        env: Env,
        code_id: u64,
        channel_open_init_options: Option<ChannelOpenInitOptions>,
    ) -> Result<SubMsg, ContractError> {
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_json_binary(&cw_ica_controller::types::msg::InstantiateMsg {
                owner: Some(env.contract.address.to_string()),
                channel_open_init_options,
                send_callbacks_to: Some(env.contract.address.to_string()),
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
    use super::*;

    use crate::types::{
        msg::query_responses::{
            GetIcaAddressesResponse, GetTransactionHistoryResponse, NftIcaPair,
        },
        state::{
            channel::ChannelStatus, get_tx_history_prefix, history::TransactionRecord, QueueItem,
            CHANNEL_STATUS, NFT_ICA_CONTRACT_BI_MAP, NFT_ICA_MAP, NFT_MINT_QUEUE,
        },
    };

    use cosmwasm_std::StdResult;
    use cw_storage_plus::Deque;

    /// Query the contract state.
    pub fn state(deps: Deps) -> StdResult<ContractState> {
        STATE.load(deps.storage)
    }

    /// Query the ICA NFT ID to ICA ID mapping.
    pub fn nft_ica_controller_bimap(deps: Deps, key: String) -> StdResult<String> {
        NFT_ICA_CONTRACT_BI_MAP.load(deps.storage, &key)
    }

    /// Query the ICA controller address for a given ICA NFT ID.
    pub fn get_ica_address(deps: Deps, token_id: String) -> StdResult<String> {
        NFT_ICA_MAP.load(deps.storage, &token_id)
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
                    let ica_address = NFT_ICA_MAP.load(deps.storage, token_id)?;
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
        let records_store: Deque<TransactionRecord> = Deque::new(&prefix);

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
            total: records_store.len(deps.storage)?,
        })
    }

    pub fn get_channel_status(deps: Deps, token_id: String) -> StdResult<ChannelStatus> {
        CHANNEL_STATUS.load(deps.storage, &token_id)
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
                    cs.cw721_ica_extension_address = addr;
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
