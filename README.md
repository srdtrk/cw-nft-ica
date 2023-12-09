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

Contract Address: `inj1yc9vdylvcm6v5my2r3jevpsl3037dsl7ve2n2n` (deployed nft-ica-coordinator)

### NFT ICA Coordinator

This contract associates each NFT in a collection with an interchain account (ICA) using the [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) contract.

CodeId (Injective Testnet): `4670`

Contract Address: `inj1ltfvqjwl3qfhx0a7w4k09t2lvl8wpwc54ylutp` (deployed by my test wallet)

Instantiate Message:

```json
{"ica_controller_code_id":4459,"cw721_ica_extension_code_id":4457,"default_chan_init_options": {"connection_id": "connection-184","counterparty_connection_id": "connection-2963"}}
```

### CosmWasm ICA Controller (`v0.2.0`)

This contract is not a part of this repository, but it is required for the NFT ICA Coordinator to work. It is a CosmWasm contract that creates controls and controls an interchain account. [Learn more about it here.](https://github.com/srdtrk/cw-ica-controller/).
This contract is instantiated by the NFT ICA Coordinator contract each time a new NFT is minted.

CodeId (Injective Testnet): `4459`
