package types

import (
	"context"

	"github.com/cosmos/gogoproto/proto"

	"github.com/strangelove-ventures/interchaintest/v7/chain/cosmos"
)

type IcaContract struct {
	Contract
	IcaAddress string
}

func NewIcaContract(contract Contract) *IcaContract {
	return &IcaContract{
		Contract:   contract,
		IcaAddress: "",
	}
}

func (c *IcaContract) SetIcaAddress(icaAddress string) {
	c.IcaAddress = icaAddress
}

// StoreAndInstantiateNewIcaContract stores the contract code and instantiates a new contract as the caller.
// Returns a new Contract instance.
func StoreAndInstantiateNewIcaContract(
	ctx context.Context, chain *cosmos.CosmosChain,
	callerKeyName, fileName string,
) (*IcaContract, error) {
	codeId, err := chain.StoreContract(ctx, callerKeyName, fileName)
	if err != nil {
		return nil, err
	}

	contractAddr, err := chain.InstantiateContract(ctx, callerKeyName, codeId, newInstantiateMsg(nil), true)
	if err != nil {
		return nil, err
	}

	contract := Contract{
		Address: contractAddr,
		CodeID:  codeId,
		chain:   chain,
	}

	return NewIcaContract(contract), nil
}

func (c *IcaContract) ExecCreateChannel(
	ctx context.Context, callerKeyName string, connectionId string,
	counterpartyConnectionId string, counterpartyPortId *string,
	txEncoding *string, extraExecTxArgs ...string,
) error {
	msg := NewCreateChannelMsg(connectionId, counterpartyConnectionId, counterpartyPortId, txEncoding)
	err := c.Execute(ctx, callerKeyName, msg, extraExecTxArgs...)
	return err
}

// ExecCustomMessages invokes the contract's `CustomIcaMessages` message as the caller
func (c *IcaContract) ExecCustomIcaMessages(
	ctx context.Context, callerKeyName string,
	messages []proto.Message, encoding string,
	memo *string, timeout *uint64,
) error {
	customMsg := newSendCustomIcaMessagesMsg(c.chain.Config().EncodingConfig.Codec, messages, encoding, memo, timeout)
	err := c.Execute(ctx, callerKeyName, customMsg)
	return err
}

// QueryContractState queries the contract's state
func (c *IcaContract) QueryContractState(ctx context.Context) (*IcaContractState, error) {
	queryResp := QueryResponse[IcaContractState]{}
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

// QueryChannelState queries the channel state stored in the contract
func (c *IcaContract) QueryChannelState(ctx context.Context) (*IcaContractChannelState, error) {
	queryResp := QueryResponse[IcaContractChannelState]{}
	err := c.chain.QueryContract(ctx, c.Address, newGetChannelQueryMsg(), &queryResp)
	if err != nil {
		return nil, err
	}

	channelState, err := queryResp.GetResp()
	if err != nil {
		return nil, err
	}

	return &channelState, nil
}

// QueryCallbackCounter queries the callback counter stored in the contract
func (c *IcaContract) QueryCallbackCounter(ctx context.Context) (*IcaContractCallbackCounter, error) {
	queryResp := QueryResponse[IcaContractCallbackCounter]{}
	err := c.chain.QueryContract(ctx, c.Address, newGetCallbackCounterQueryMsg(), &queryResp)
	if err != nil {
		return nil, err
	}

	callbackCounter, err := queryResp.GetResp()
	if err != nil {
		return nil, err
	}

	return &callbackCounter, nil
}
