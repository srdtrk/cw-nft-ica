//! This module contains utilities for the contract.

use cosmwasm_std::{
    instantiate2_address, Addr, Api, Binary, CosmosMsg, Env, QuerierWrapper, WasmMsg,
};

use crate::types::ContractError;

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

/// Contains the storage utilities.
pub mod storage {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{StdResult, Storage};
    use cw_storage_plus::{Map, PrimaryKey};

    /// A set of keys.
    pub struct KeySet<'a, K>(Map<'a, K, NoValue>);

    #[cw_serde]
    struct NoValue {}

    impl<'a, K> KeySet<'a, K>
    where
        K: PrimaryKey<'a>,
    {
        /// Create a new set of keys.
        pub const fn new(namespace: &'a str) -> Self {
            Self(Map::new(namespace))
        }

        /// Insert a new key.
        pub fn insert(&self, store: &mut dyn Storage, key: K) -> StdResult<()> {
            self.0.save(store, key, &NoValue {})
        }

        /// Check if the given key exists.
        pub fn has(&self, store: &dyn Storage, key: K) -> bool {
            self.0.has(store, key)
        }

        /// Remove the given key.
        /// Does not return an error if the key does not exist.
        pub fn remove(&self, store: &mut dyn Storage, key: K) {
            self.0.remove(store, key)
        }
    }

    /// The bi-directional map between ICA addresses and NFT IDs.
    pub struct NftIcaBiMap<'a, 'b>(Map<'a, &'b str, String>);

    impl<'a, 'b> NftIcaBiMap<'a, 'b> {
        /// Create a new bi-directional map between ICA addresses and NFT IDs.
        pub const fn new(namespace: &'a str) -> Self {
            Self(Map::new(namespace))
        }

        /// Insert a new ICA address and NFT ID pair.
        pub fn insert(
            &self,
            store: &mut dyn Storage,
            ica_addr: impl Into<String>,
            nft_id: impl Into<String>,
        ) -> StdResult<()> {
            let ica_addr = ica_addr.into();
            let nft_id = nft_id.into();

            self.0.save(store, &ica_addr, &nft_id)?;
            self.0.save(store, &nft_id, &ica_addr)?;

            Ok(())
        }

        /// Get the value associated with the given key.
        pub fn load(&self, store: &dyn Storage, key: &str) -> StdResult<String> {
            self.0.load(store, key)
        }

        /// Remove the value associated with the given key and vice versa.
        /// Does not return an error if the key does not exist.
        /// Returns an error if there are issues parsing the value associated with the given key.
        pub fn remove(&self, store: &mut dyn Storage, key: &str) -> StdResult<()> {
            self.0.may_load(store, key)?.map(|value| {
                self.0.remove(store, key);
                self.0.remove(store, &value);
            });

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use cosmwasm_std::testing::MockStorage;

        #[test]
        fn test_nft_ica_bi_map() {
            let mut storage = MockStorage::new();

            let nft_ica_bi_map = NftIcaBiMap::new("nft_ica_bi_map");

            nft_ica_bi_map
                .insert(&mut storage, "ica-addr-1", "nft-id-1")
                .unwrap();
            nft_ica_bi_map
                .insert(&mut storage, "ica-addr-2", "nft-id-2")
                .unwrap();

            assert_eq!(
                nft_ica_bi_map.load(&storage, "ica-addr-1").unwrap(),
                "nft-id-1"
            );
            assert_eq!(
                nft_ica_bi_map.load(&storage, "nft-id-1").unwrap(),
                "ica-addr-1"
            );
            assert_eq!(
                nft_ica_bi_map.load(&storage, "ica-addr-2").unwrap(),
                "nft-id-2"
            );
            assert_eq!(
                nft_ica_bi_map.load(&storage, "nft-id-2").unwrap(),
                "ica-addr-2"
            );

            nft_ica_bi_map.remove(&mut storage, "ica-addr-1").unwrap();

            assert!(nft_ica_bi_map.load(&storage, "ica-addr-1").is_err());
            assert!(nft_ica_bi_map.load(&storage, "nft-id-1").is_err());
            assert_eq!(
                nft_ica_bi_map.load(&storage, "ica-addr-2").unwrap(),
                "nft-id-2"
            );
            assert_eq!(
                nft_ica_bi_map.load(&storage, "nft-id-2").unwrap(),
                "ica-addr-2"
            );

            nft_ica_bi_map.remove(&mut storage, "nft-id-2").unwrap();

            assert!(nft_ica_bi_map.load(&storage, "ica-addr-1").is_err());
            assert!(nft_ica_bi_map.load(&storage, "nft-id-1").is_err());
            assert!(nft_ica_bi_map.load(&storage, "ica-addr-2").is_err());
            assert!(nft_ica_bi_map.load(&storage, "nft-id-2").is_err());
        }
    }
}
