//! This module handles the execution logic of the contract.

use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::types::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::types::state::{ContractState, STATE};
use crate::types::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-ica";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Instantiate the contract.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg.owner.unwrap_or(info.sender.to_string());
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;

    let (cosmos_msg, contract_addr) = instantiate::instantiate2_cw721_ica_extension(
        deps.api,
        deps.querier,
        env,
        msg.cw721_ica_extension_code_id,
        msg.salt,
    )?;

    let state = ContractState {
        default_chan_init_options: msg.default_chan_init_options,
        ica_controller_code_id: msg.ica_controller_code_id,
        cw721_ica_extension_address: contract_addr,
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_message(cosmos_msg))
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
        _ => unimplemented!(),
    }
}

/// Query the contract.
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
        _ => unimplemented!(),
    }
}

mod instantiate {
    use crate::utils;

    use super::*;

    use cosmwasm_std::{Addr, Api, CosmosMsg, QuerierWrapper};

    /// Instantiate the cw721-ica extension contract using the instantiate2 pattern.
    /// Returns the instantiate2 message and the contract address.
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
}

mod execute {
    use super::*;

    use cosmwasm_std::{Addr, Api, CosmosMsg, QuerierWrapper, WasmMsg};
    use cw721_ica_extension::Extension;
    use cw_ica_controller::types::{
        callbacks::IcaControllerCallbackMsg, msg::options::ChannelOpenInitOptions,
    };

    use crate::{
        types::state::{NFT_MINT_QUEUE, REGISTERED_ICA_ADDRS},
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
                let queue_item = NFT_MINT_QUEUE
                    .pop_back(deps.storage)?
                    .ok_or(ContractError::QueueEmpty)?;

                let cw721_ica_extension_address =
                    STATE.load(deps.storage)?.cw721_ica_extension_address;

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
            _ => Ok(Response::new()),
        }
    }

    /// Instantiate the cw721-ica extension contract using the instantiate2 pattern.
    /// Returns the instantiate2 message and the contract address.
    fn instantiate2_cw_ica_controller(
        api: &dyn Api,
        querier: QuerierWrapper,
        env: Env,
        code_id: u64,
        salt: Option<String>,
        channel_open_init_options: Option<ChannelOpenInitOptions>,
    ) -> Result<(CosmosMsg, Addr), ContractError> {
        let instantiate_msg = to_json_binary(&cw_ica_controller::types::msg::InstantiateMsg {
            admin: Some(env.contract.address.to_string()),
            channel_open_init_options,
            send_callbacks_to: Some(env.contract.address.to_string()),
        })?;

        let label = format!("cw-ica-controller-{}", env.block.height);

        utils::instantiate2_contract(api, querier, env, code_id, salt, label, instantiate_msg)
    }
}

#[cfg(test)]
mod tests {}
