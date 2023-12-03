# NFT-ICA

A collection of contracts that associate each NFT in a collection with an interchain account (ICA) using the [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) contract.
The owner of the NFT becomes the owner of the ICA on the counterparty chain. This does not require any contracts to be deployed on the counterparty chain, learn more about it in [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/).
This project was created for the [Injective x Google Cloud Illuminate Hackathon](https://dorahacks.io/hackathon/illuminate/detail).
Also, I created the production releases of [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) for this hackathon too, as stated in its README.
Which is an infrastructure that can be used by any other project to create and control interchain accounts.

## Contracts

### Cw721 ICA Extension

This repository wraps the cw721-base contract with an extension that allows storing the interchain account address for each token.

CodeId (Injective Testnet): `4457`

Contract Address: `inj1wsvc09g2lmhwv2tg97jnjq545puhs5m0ak0p73` (deployed nft-ica-coordinator)

### NFT ICA Coordinator

This contract associates each NFT in a collection with an interchain account (ICA) using the [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) contract.

CodeId (Injective Testnet): `4458`

Contract Address: `inj1clt4czjmhdejvm6y9jnzqsjplvdnr0vspht2u8` (deployed by my test wallet)

### CosmWasm ICA Controller (`v0.2.0`)

This contract is not a part of this repository, but it is required for the NFT ICA Coordinator to work. It is a CosmWasm contract that creates controls and controls an interchain account. [Learn more about it here.](https://github.com/srdtrk/cw-ica-controller/).
This contract is instantiated by the NFT ICA Coordinator contract each time a new NFT is minted.

CodeId (Injective Testnet): `4459`
