package types_test

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/srdtrk/cw-ica-controller/interchaintest/v2/types"
)

func TestInstantiateMsg(t *testing.T) {
	t.Parallel()

	msg := types.NewInstantiateMsg(nil)
	require.Equal(t, `{}`, msg)

	admin := "srdtrk"
	msg = types.NewInstantiateMsg(&admin)
	require.Equal(t, `{"admin":"srdtrk"}`, msg)
}

func TestExecuteMsgs(t *testing.T) {
	const testAddress = "srdtrk"

	t.Parallel()

	// Test Coordinator Messgaes
	coordinatorMintIcaMsg := types.NewCoordinatorMintIcaMsg(nil)
	require.Equal(t, `{"mint_ica":{}}`, coordinatorMintIcaMsg)
}

func TestQueries(t *testing.T) {
	t.Parallel()

	channelQueryMsg := types.NewGetChannelQueryMsg()
	msg, err := json.Marshal(channelQueryMsg)
	require.NoError(t, err)
	require.Equal(t, `{"get_channel":{}}`, string(msg))

	contractStateQueryMsg := types.NewGetContractStateQueryMsg()
	msg, err = json.Marshal(contractStateQueryMsg)
	require.NoError(t, err)
	require.Equal(t, `{"get_contract_state":{}}`, string(msg))

	callbackCounterQueryMsg := types.NewGetCallbackCounterQueryMsg()
	msg, err = json.Marshal(callbackCounterQueryMsg)
	require.NoError(t, err)
	require.Equal(t, `{"get_callback_counter":{}}`, string(msg))
}
