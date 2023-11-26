package main

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/suite"

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

	controllerCodeId, err := s.ChainA.StoreContract(ctx, s.UserA.KeyName(), "../../artifacts/cw_ica_controller.wasm")
	s.Require().NoError(err)

	cw721CodeId, err := s.ChainA.StoreContract(ctx, s.UserA.KeyName(), "../../artifacts/cw721_ica_extension.wasm")
	s.Require().NoError(err)

	s.Contract, err = types.StoreAndInstantiateNewCoordinatorContract(
		ctx, s.ChainA, s.UserA.KeyName(), "../../artifacts/nft_ica_coordinator.wasm",
		controllerCodeId, cw721CodeId, s.ChainAConnID, s.ChainBConnID, nil, nil,
		"--gas", "500000",
	)
	s.Require().NoError(err)

	// // Wait for the channel to get set up
	// err = testutil.WaitForBlocks(ctx, 5, s.ChainA, s.ChainB)
	// s.Require().NoError(err)
	//
	// contractState, err := s.Contract.QueryContractState(ctx)
	// s.Require().NoError(err)
	//
	// s.IcaAddress = contractState.IcaInfo.IcaAddress
	// s.Contract.SetIcaAddress(s.IcaAddress)
}

func TestWithContractTestSuite(t *testing.T) {
	suite.Run(t, new(ContractTestSuite))
}

func (s *ContractTestSuite) TestContractInstantiate() {
	ctx := context.Background()

	s.SetupContractTestSuite(ctx)
	// wasmd, simd := s.ChainA, s.ChainB
	// wasmdUser := s.UserA

	s.Run("TestInstantiate", func() {
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
