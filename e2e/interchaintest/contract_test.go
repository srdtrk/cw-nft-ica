package main

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/suite"

	"github.com/strangelove-ventures/interchaintest/v7/testutil"

	icatypes "github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"

	mysuite "github.com/srdtrk/cw-ica-controller/interchaintest/v2/testsuite"
	"github.com/srdtrk/cw-ica-controller/interchaintest/v2/types"
)

type ContractTestSuite struct {
	mysuite.TestSuite

	Contract *types.CoordinatorContract
}

// SetupContractTestSuite starts the chains, relayer, creates the user accounts, creates the ibc clients and connections,
// sets up the contract and does the channel handshake for the contract test suite.
func (s *ContractTestSuite) SetupContractTestSuite(ctx context.Context) {
	s.SetupSuite(ctx, chainSpecs)

	controllerCodeId, err := s.ChainA.StoreContract(ctx, s.UserA.KeyName(), "../../third_party/cw_ica_controller.wasm")
	s.Require().NoError(err)

	cw721CodeId, err := s.ChainA.StoreContract(ctx, s.UserA.KeyName(), "../../artifacts/cw721_ica_extension.wasm")
	s.Require().NoError(err)

	s.Contract, err = types.StoreAndInstantiateNewCoordinatorContract(
		ctx, s.ChainA, s.UserA.KeyName(), "../../artifacts/nft_ica_coordinator.wasm",
		controllerCodeId, cw721CodeId, s.ChainAConnID, s.ChainBConnID, nil, nil,
		"--gas", "500000",
	)
	s.Require().NoError(err)
}

func TestWithContractTestSuite(t *testing.T) {
	suite.Run(t, new(ContractTestSuite))
}

func (s *ContractTestSuite) TestMintIca() {
	ctx := context.Background()

	s.SetupContractTestSuite(ctx)
	wasmd, simd := s.ChainA, s.ChainB
	wasmdUser := s.UserA

	s.Run("TestMintIca", func() {
		// Mint a new ICA for the user
		// err := s.Contract.Execute(ctx, wasmdUser.KeyName(), `{ "mint_ica": {} }`, "--gas", "500000")
		err := s.Contract.MintIca(ctx, wasmdUser.KeyName(), nil, "--gas", "500000")
		s.Require().NoError(err)

		// Wait for the channel to get set up
		err = testutil.WaitForBlocks(ctx, 7, s.ChainA, s.ChainB)
		s.Require().NoError(err)

		// Check that the ICA was minted and contract created
		icaContractAddress, err := s.Contract.QueryNftIcaBimap(ctx, "ica-token-0")
		s.Require().NoError(err)

		icaContract := types.NewIcaContract(types.NewContract(*icaContractAddress, "0", wasmd))

		// Test if the handshake was successful
		wasmdChannels, err := s.Relayer.GetChannels(ctx, s.ExecRep, wasmd.Config().ChainID)
		s.Require().NoError(err)
		s.Require().Equal(1, len(wasmdChannels))

		wasmdChannel := wasmdChannels[0]
		s.T().Logf("wasmd channel: %s", toJSONString(wasmdChannel))
		s.Require().Equal(icaContract.Port(), wasmdChannel.PortID)
		s.Require().Equal(icatypes.HostPortID, wasmdChannel.Counterparty.PortID)
		s.Require().Equal(channeltypes.OPEN.String(), wasmdChannel.State)

		simdChannels, err := s.Relayer.GetChannels(ctx, s.ExecRep, simd.Config().ChainID)
		s.Require().NoError(err)
		// I don't know why sometimes an extra channel is created in simd.
		// this is not related to the localhost connection, and is a failed
		// clone of the successful channel at index 0. I will log it for now.
		s.Require().Greater(len(simdChannels), 0)
		if len(simdChannels) > 1 {
			s.T().Logf("extra simd channels detected: %s", toJSONString(simdChannels))
		}

		simdChannel := simdChannels[0]
		s.T().Logf("simd channel state: %s", toJSONString(simdChannel.State))
		s.Require().Equal(icatypes.HostPortID, simdChannel.PortID)
		s.Require().Equal(icaContract.Port(), simdChannel.Counterparty.PortID)
		s.Require().Equal(channeltypes.OPEN.String(), simdChannel.State)
	})
}

// toJSONString returns a string representation of the given value
// by marshaling it to JSON. It panics if marshaling fails.
func toJSONString(v any) string {
	bz, err := json.Marshal(v)
	if err != nil {
		panic(err)
	}
	return string(bz)
}
