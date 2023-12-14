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

Contract Address: `inj1t5vs28cd3e5r0flwd3d8hlj7ypkk8x0rjajt6q` (deployed by nft-ica-coordinator)

### NFT ICA Coordinator

This contract associates each NFT in a collection with an interchain account (ICA) using the [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) contract.

CodeId (Injective Testnet): `4738` (new query)

Contract Address: `inj1t6kw77tc5vagcyatl0gd02veqae9ydeaq0s0qm` (deployed by test wallet)

Instantiate Message:

```json
{"ica_controller_code_id":4691,"cw721_ica_extension_code_id":4457,"default_chan_init_options": {"connection_id": "connection-184","counterparty_connection_id": "connection-2963"}}
```

### CosmWasm ICA Controller

This contract is not a part of this repository, but it is required for the NFT ICA Coordinator to work. It is a CosmWasm contract that creates controls and controls an interchain account. [Learn more about it here.](https://github.com/srdtrk/cw-ica-controller/).
This contract is instantiated by the NFT ICA Coordinator contract each time a new NFT is minted.

CodeId (Injective Testnet): `4459` (v0.2), `4691` (v0.3-alpha)
