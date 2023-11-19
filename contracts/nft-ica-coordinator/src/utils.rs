//! This module contains utilities for the contract.

use cosmwasm_std::{
    instantiate2_address, Addr, Api, Binary, CosmosMsg, Env, QuerierWrapper, WasmMsg,
};

use crate::ContractError;

/// Instantiate a contract using the instantiate2 pattern.
/// Returns the instantiate2 message and the contract address.
pub fn instantiate2_contract(
    api: &dyn Api,
    querier: QuerierWrapper,
    env: Env,
    code_id: u64,
    salt: Option<String>,
    label: impl Into<String>,
    instantiate_msg: Binary,
) -> Result<(CosmosMsg, Addr), ContractError> {
    let salt = salt.unwrap_or(env.block.time.seconds().to_string());

    let code_info = querier.query_wasm_code_info(code_id)?;
    let creator_cannonical = api.addr_canonicalize(env.contract.address.as_str())?;

    let contract_addr = api.addr_humanize(&instantiate2_address(
        &code_info.checksum,
        &creator_cannonical,
        salt.as_bytes(),
    )?)?;

    let instantiate_msg = WasmMsg::Instantiate2 {
        code_id,
        msg: instantiate_msg,
        funds: vec![],
        label: label.into(),
        admin: Some(env.contract.address.to_string()),
        salt: salt.as_bytes().into(),
    };

    return Ok((instantiate_msg.into(), contract_addr));
}
