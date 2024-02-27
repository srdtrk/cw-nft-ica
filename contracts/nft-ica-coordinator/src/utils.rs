//! This module contains utilities for the contract.

/// Contains the storage utilities.
pub mod storage {
    use cosmwasm_std::{StdError, StdResult, Storage};
    use secret_toolkit::storage::Keymap;
    use secret_toolkit::serialization::Bincode2;

    /// The bi-directional map between ICA addresses and NFT IDs.
    pub struct NftIcaBiMap<'a>(Keymap<'a, String, String, Bincode2>);

    impl<'a> NftIcaBiMap<'a> {
        /// Create a new bi-directional map between ICA addresses and NFT IDs.
        pub const fn new(namespace: &'a [u8]) -> Self {
            Self(Keymap::new(namespace))
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

            self.0.insert(store, &ica_addr, &nft_id)?;
            self.0.insert(store, &nft_id, &ica_addr)?;

            Ok(())
        }

        /// Get the value associated with the given key.
        pub fn load(&self, store: &dyn Storage, key: impl Into<String>) -> StdResult<String> {
            self.0.get(store, &key.into()).ok_or(StdError::not_found("nft_ica_bi_map"))
        }

        /// Get the value associated with the given key if the key is present.
        pub fn may_load(&self, store: &dyn Storage, key: impl Into<String>) -> StdResult<Option<String>> {
            Ok(self.0.get(store, &key.into()))
        }

        /// Remove the value associated with the given key and vice versa.
        /// Does not return an error if the key does not exist.
        /// Returns an error if there are issues parsing the value associated with the given key.
        pub fn remove(&self, store: &mut dyn Storage, key: impl Into<String>) -> StdResult<()> {
            let key = key.into();
            if let Some(value) = self.may_load(store, &key)? {
                self.0.remove(store, &key)?;
                self.0.remove(store, &value)?;
            }

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

            let nft_ica_bi_map = NftIcaBiMap::new(b"nft_ica_bi_map");

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
