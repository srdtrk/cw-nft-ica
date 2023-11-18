//! This module handles the execution logic of the contract.

use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ContractState, STATE};

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
        env,
        info,
        msg.cw721_ica_extension_code_id,
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
    use cosmwasm_std::{Addr, Api, CosmosMsg};

    use super::*;

    pub fn instantiate2_cw721_ica_extension(
        _api: &dyn Api,
        _env: Env,
        _info: MessageInfo,
        _code_id: u64,
    ) -> Result<(CosmosMsg, Addr), ContractError> {
        todo!()
    }
}

mod execute {
    use super::*;

    pub fn update_ownership(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        action: cw_ownable::Action,
    ) -> Result<Response, ContractError> {
        cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
        Ok(Response::default())
    }
}

#[cfg(test)]
mod tests {}
