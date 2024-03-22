# NFT-ICA

## Overview

The goal of this project is to create an NFT collection where each NFT in the source chain (Injective testnet) controls an interchain account (ICA) on the counterparty chain (Cosmos Hub Testnet).

The contracts in this repository are already deployed on the Injective Testnet! You can use them to create your own NFT collection that controls interchain accounts on the Cosmos Hub Testnet via the frontend [here]() given that a relayer is running in the background between the two chains.

## How it works

This project consists of three contracts. No contract has to be deployed on the counterparty chain, only on the source chain (Injective Testnet)!
This is possible thanks to the interaction between the CosmWasm ICA Controller Contract and the IBC Protocol.
The contracts are:

- [Cw721 ICA Extension](#cw721-ica-extension)
- [NFT ICA Coordinator](#nft-ica-coordinator)
- [CosmWasm ICA Controller](#cosmwasm-ica-controller)

CosmWasm ICA Controller (developed by me) is the contract that actually controls an account on the counterparty chain on behalf of its owner. Learn more about it [here](https://github.com/srdtrk/cw-ica-controller/). Each interchain account requires a new instance of this contract.
This is an infrastructure that can be used by any other project to create and control interchain accounts.

Cw721 ICA Extension is a wrapper around the cw721-base contract that allows storing the interchain account address for each token.

NFT ICA Coordinator is the contract that associates each NFT in a collection with an interchain account (ICA). It does this by creating a new instance of the CosmWasm ICA Controller contract each time a new NFT is minted. Therefore, this is the contract that the front-end usually interacts with.
This is the contract that is the admin of the interchain accounts, and it only allows the owner of the NFT to control the interchain account that is associated with it.

## Motivation

This project was created for the [Injective x Google Cloud Illuminate Hackathon](https://dorahacks.io/hackathon/illuminate/detail).
Note that this hackathon submission is not a mono-repo. The two other repositories that are a part of this submission are included in this repository as submodules.

- [CosmWasm ICA Controller](#cosmwasm-ica-controller)
- [CosmWasm ICA Controller Frontend](https://github.com/srdtrk/nft-ica-ui)

I initially made the CosmWasm ICA Controller production ready for this hackathon, but I realized that it would be more impactful if I could create a project that uses it. Some use cases for these NFT include:

- Buying and selling accounts on other chains
- Creating a DAO that controls accounts on another chain
- Using accounts as collaterals in lending protocols
- Taking actions on other chains based on events in injective
- If the interchain account qualifies for some airdrop, the NFT owner can put it up for auction before the airdrop and sell it to the highest bidder.

## Contracts

### Cw721 ICA Extension

This repository wraps the cw721-base contract with an extension that allows storing the interchain account address for each token.

CodeId (Injective Testnet): `4457`

Contract Address: `inj18dsyqvg6qrlaygag3wf2xskeq83ys7rtn4a3vl` (deployed by nft-ica-coordinator)

### NFT ICA Coordinator

This contract associates each NFT in a collection with an interchain account (ICA) using the [cw-ica-controller](https://github.com/srdtrk/cw-ica-controller/) contract.

CodeId (Injective Testnet): `4810`

Contract Address: `inj15h68ack7wwakmygxdmmzs9nfru0gljpey768k8` (deployed by test wallet)

Instantiate Message:

```json
{"ica_controller_code_id":4691,"cw721_ica_extension_code_id":4457,"default_chan_init_options": {"connection_id": "connection-219","counterparty_connection_id": "connection-3570"}}
```

### CosmWasm ICA Controller

This contract is not a part of this repository, but it is required for the NFT ICA Coordinator to work. It is a CosmWasm contract that creates controls and controls an interchain account. [Learn more about it here.](https://github.com/srdtrk/cw-ica-controller/).
This contract is instantiated by the NFT ICA Coordinator contract each time a new NFT is minted.
Since this is the contract that actually controls the interchain account, I took testing and documentation extremely seriously!

CodeId (Injective Testnet): `4691` (v0.3-alpha)

## How to use (WIP)

The frontend for this project is [here](). You can simply use the intuitive UI to mint NFTs that control interchain accounts on the Cosmos Hub Testnet. However, you must ensure that a relayer is running in the back.
I'd run the relayer at all times myself, but I don't have the resources to do so (access to stable grpc and rpc nodes for the testnet). So instead, I'll explain how to run the relayer yourself.
Instead, the judges can schedule a time period with me to run the relayer and they can just use the frontend to test the project.

### Running the relayer (WIP)

For this example, we will use the [hermes relayer v1.7.4](https://hermes.informal.systems/). I've already included the config file and the mnemonic for the test wallet in this repository. You can use the following command to run the relayer after installing hermes from their website.
If you have cargo installed, you can simply run:

```bash
cargo install ibc-relayer-cli --version 1.7.4 --bin hermes --locked
```

You can verify the installation by running:

```bash
hermes version
```

Then we copy the config file to the hermes directory:

```bash
cp hermes/config.toml ~/.hermes/
```

Then we add the keys to hermes (these wallets are already funded on the testnet):

```bash
hermes keys add --key-name charlie --chain earth --mnemonic-file hermes/charlie.mnemonic
hermes keys add --key-name damian --chain mars --mnemonic-file hermes/damian.mnemonic
```

Then we start the relayer:

```bash
hermes start
```

Wait for the relayer to scan both chains until you see `Hermes has started`. This will take a few minutes. Then you may use the frontend.
