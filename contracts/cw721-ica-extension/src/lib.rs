#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty};
pub use cw721_base::{ContractError, InstantiateMsg, MinterResponse};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw721-ica-extension";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the ICA extension data that is stored with each token
#[cw_serde]
pub struct Extension {
    /// The ICA controller contract's address
    pub ica_controller_address: Addr,
    /// The ICA address in the counterparty chain.
    pub ica_address: String,
}

/// This is a wrapper around the [`cw721_base::Cw721Contract`] that adds the ICA extension
pub type Cw721IcaExtensionContract<'a> =
    cw721_base::Cw721Contract<'a, Extension, Empty, Empty, Empty>;
/// This is the execute message that this contract supports
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
/// This is the query message that this contract supports
pub type QueryMsg = cw721_base::QueryMsg<Empty>;

/// This module contains the entry points for the contract
#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    // This makes a conscious choice on the various generics used by the contract
    /// This is the instantiate entry point for the contract
    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        Cw721IcaExtensionContract::default().instantiate(deps.branch(), env, info, msg)
    }

    /// This is the execute entry point for the contract
    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Cw721IcaExtensionContract::default().execute(deps, env, info, msg)
    }

    /// This is the query entry point for the contract
    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721IcaExtensionContract::default().query(deps, env, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use cw721::Cw721Query;

    const CREATOR: &str = "creator";

    /// Make sure cw2 version info is properly initialized during instantiation,
    /// and NOT overwritten by the base contract.
    #[test]
    fn proper_cw2_initialization() {
        let mut deps = mock_dependencies();

        entry::instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("larry", &[]),
            InstantiateMsg {
                name: "".into(),
                symbol: "".into(),
                minter: "larry".into(),
            },
        )
        .unwrap();

        let version = cw2::get_contract_version(deps.as_ref().storage).unwrap();
        assert_eq!(version.contract, CONTRACT_NAME);
        assert_ne!(version.contract, cw721_base::CONTRACT_NAME);

        assert!(cw_ownable::is_owner(&deps.storage, &Addr::unchecked("larry")).unwrap())
    }

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies();
        let contract = Cw721IcaExtensionContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "Enterprise";
        let token_uri = Some("https://starships.example.com/Starship/Enterprise.json".into());
        let extension = Extension {
            ica_controller_address: Addr::unchecked("0x1234567890123456789012345678901234567890"),
            ica_address: "0x1234567890123456789012345678901234567890".into(),
        };
        let exec_msg = ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: token_uri.clone(),
            extension: extension.clone(),
        };
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.token_uri, token_uri);
        assert_eq!(res.extension, extension);
    }
}
