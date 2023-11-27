package types

import (
	"context"
	"strconv"

	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
)

type CoordinatorContract struct {
	Contract
	Cw721Contract *Cw721Contract
}

func NewCoordinatorContract(contract Contract) *CoordinatorContract {
	return &CoordinatorContract{
		Contract:      contract,
		Cw721Contract: nil,
	}
}

// StoreAndInstantiateNewCoordinatorContract stores the contract code and instantiates a new contract as the caller.
// Returns a new Contract instance.
func StoreAndInstantiateNewCoordinatorContract(
	ctx context.Context, chain *cosmos.CosmosChain,
	callerKeyName, fileName, icaCodeID, cw721CodeID,
	connectionID, counterpartyConnectionID string,
	counterpartyPortID, txEncoding *string,
	extraExecTxArgs ...string,
) (*CoordinatorContract, error) {
	codeId, err := chain.StoreContract(ctx, callerKeyName, fileName)
	if err != nil {
		return nil, err
	}

	cw721CodeId, err := strconv.ParseUint(cw721CodeID, 10, 64)
	if err != nil {
		return nil, err
	}

	icaCodeId, err := strconv.ParseUint(icaCodeID, 10, 64)
	if err != nil {
		return nil, err
	}

	contractAddr, err := chain.InstantiateContract(
		ctx, callerKeyName, codeId,
		newCoordinatorInstantiateMsg(nil, icaCodeId, cw721CodeId, connectionID, counterpartyConnectionID, counterpartyPortID, txEncoding),
		true,
		extraExecTxArgs...,
	)
	if err != nil {
		return nil, err
	}

	contract := Contract{
		Address: contractAddr,
		CodeID:  codeId,
		chain:   chain,
	}

	coordContract := NewCoordinatorContract(contract)
	contractState, err := coordContract.QueryContractState(ctx)
	if err != nil {
		return nil, err
	}

	cw721Contract := Contract{
		Address: contractState.Cw721IcaExtensionAddress,
		CodeID:  cw721CodeID,
		chain:   chain,
	}

	coordContract.Cw721Contract = NewCw721Contract(cw721Contract)

	return coordContract, nil
}

// MintIca mints an ICA-NFT for the caller
func (c *CoordinatorContract) MintIca(ctx context.Context, callerKeyName string, salt *string, extraExecTxArgs ...string) error {
	return c.Execute(ctx, callerKeyName, newCoordinatorMintIcaMsg(salt), extraExecTxArgs...)
}

// QueryContractState queries the contract's state
func (c *CoordinatorContract) QueryContractState(ctx context.Context) (*CoordinatorContractState, error) {
	queryResp := QueryResponse[CoordinatorContractState]{}
	err := c.chain.QueryContract(ctx, c.Address, newGetContractStateQueryMsg(), &queryResp)
	if err != nil {
		return nil, err
	}

	contractState, err := queryResp.GetResp()
	if err != nil {
		return nil, err
	}

	return &contractState, nil
}

func (c *CoordinatorContract) QueryNftIcaBimap(ctx context.Context, key string) (*string, error) {
	queryResp := QueryResponse[string]{}
	err := c.chain.QueryContract(ctx, c.Address, newNftIcaBimapQueryMsg(key), &queryResp)
	if err != nil {
		return nil, err
	}

	ica, err := queryResp.GetResp()
	if err != nil {
		return nil, err
	}

	return &ica, nil
}
