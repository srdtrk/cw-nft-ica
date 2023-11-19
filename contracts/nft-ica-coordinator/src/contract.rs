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
    use super::*;

    use cosmwasm_std::{Addr, Api, CosmosMsg, QuerierWrapper, instantiate2_address, WasmMsg};

    pub fn instantiate2_cw721_ica_extension(
        api: &dyn Api,
        querier: QuerierWrapper,
        env: Env,
        code_id: u64,
        salt: Option<String>,
    ) -> Result<(CosmosMsg, Addr), ContractError> {
        let salt = salt.unwrap_or(env.block.time.seconds().to_string());

        let code_info = querier
            .query_wasm_code_info(code_id)?;
        let creator_cannonical = api.addr_canonicalize(env.contract.address.as_str())?;

        let contract_addr = api.addr_humanize(&instantiate2_address(
            &code_info.checksum,
            &creator_cannonical,
            salt.as_bytes(),
        )?)?;

        let instantiate_msg = WasmMsg::Instantiate2 {
            code_id,
            msg: to_json_binary(&cw721_base::InstantiateMsg {
                name: "NFT-ICA".to_string(),
                symbol: "ICA".to_string(),
                minter: env.contract.address.to_string(),
            })?,
            funds: vec![],
            label: format!("cw721-ica-{}", env.block.height),
            admin: Some(env.contract.address.to_string()),
            salt: salt.as_bytes().into(),
        };

        return Ok((instantiate_msg.into(), contract_addr));
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
