# NFT-ICA

A collection of contracts that associate each NFT in a collection with an interchain account (ICA) using the [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) contract.

## Contracts

### Cw721 ICA Extension

This repository wraps the cw721-base contract with an extension that allows storing the interchain account address for each token.

### NFT ICA Coordinator

This contract associates each NFT in a collection with an interchain account (ICA) using the [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) contract.

### CosmWasm ICA Controller

This contract is not a part of this repository, but it is required for the NFT ICA Coordinator to work. It is a CosmWasm contract that creates controls and controls an interchain account. [Learn more about it here.](https://github.com/srdtrk/cw-ica-controller/)
