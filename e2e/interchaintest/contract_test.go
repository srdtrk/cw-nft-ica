package main

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"

	sdkmath "cosmossdk.io/math"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"

	icatypes "github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"

	"github.com/strangelove-ventures/interchaintest/v7/testutil"

	mysuite "github.com/srdtrk/cw-ica-controller/interchaintest/v2/testsuite"
	"github.com/srdtrk/cw-ica-controller/interchaintest/v2/types"
)

type ContractTestSuite struct {
	mysuite.TestSuite

	Contract    *types.CoordinatorContract
	NftContract *types.Cw721Contract
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

	contractState, err := s.Contract.QueryContractState(ctx)
	s.Require().NoError(err)

	s.NftContract = types.NewCw721Contract(types.NewContract(contractState.Cw721IcaExtensionAddress, cw721CodeId, s.ChainA))
}

func TestWithContractTestSuite(t *testing.T) {
	suite.Run(t, new(ContractTestSuite))
}

func (s *ContractTestSuite) TestMintAndExecute() {
	ctx := context.Background()

	s.SetupContractTestSuite(ctx)
	wasmd, simd := s.ChainA, s.ChainB
	wasmdUser := s.UserA

	firstTokenID := "ica-token-0"

	var icaContract *types.IcaContract
	s.Run("TestMintIca", func() {
		// Mint a new ICA for the user
		err := s.Contract.MintIca(ctx, wasmdUser.KeyName(), nil, "--gas", "500000")
		s.Require().NoError(err)

		// Wait for the channel to get set up
		err = testutil.WaitForBlocks(ctx, 7, s.ChainA, s.ChainB)
		s.Require().NoError(err)

		// Check that the ICA channel was opened:
		status, err := s.Contract.QueryChannelStatus(ctx, firstTokenID)
		s.Require().NoError(err)
		s.Require().Equal("open", status)

		icaContractAddress, err := s.Contract.QueryNftIcaBimap(ctx, firstTokenID)
		s.Require().NoError(err)

		icaContract = types.NewIcaContract(types.NewContract(icaContractAddress, "0", wasmd))

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

		// Check that the NFT was minted
		tokensResp, err := s.NftContract.QueryTokens(ctx, wasmdUser.FormattedAddress())
		s.Require().NoError(err)

		s.Require().Equal(1, len(tokensResp.Tokens))
		s.Require().Equal(firstTokenID, tokensResp.Tokens[0])
	})

	s.Run("TestExecuteCustomIcaMsg", func() {
		// Get ICA address
		icaAddress, err := s.Contract.QueryIcaAddress(ctx, firstTokenID)
		s.Require().NoError(err)

		// Fund the ICA address:
		s.FundAddressChainB(ctx, icaAddress)

		// Send custom ICA messages through the contract:
		// Let's create a governance proposal on simd and deposit some funds to it.
		testProposal := govtypes.TextProposal{
			Title:       "IBC Gov Proposal",
			Description: "tokens for all!",
		}
		protoAny, err := codectypes.NewAnyWithValue(&testProposal)
		s.Require().NoError(err)
		proposalMsg := &govtypes.MsgSubmitProposal{
			Content:        protoAny,
			InitialDeposit: sdk.NewCoins(sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(5000))),
			Proposer:       icaAddress,
		}

		// Create deposit message:
		depositMsg := &govtypes.MsgDeposit{
			ProposalId: 1,
			Depositor:  icaAddress,
			Amount:     sdk.NewCoins(sdk.NewCoin(simd.Config().Denom, sdkmath.NewInt(10000000))),
		}

		// Execute the contract:
		err = s.Contract.ExecuteCustomIcaMsgs(ctx, wasmdUser.KeyName(), firstTokenID, []proto.Message{proposalMsg, depositMsg}, icatypes.EncodingProtobuf, nil, nil, "--gas", "500000")
		s.Require().NoError(err)

		err = testutil.WaitForBlocks(ctx, 5, wasmd, simd)
		s.Require().NoError(err)

		// Check if contract callbacks were executed:
		callbackCounter, err := icaContract.QueryCallbackCounter(ctx)
		s.Require().NoError(err)

		s.Require().Equal(uint64(1), callbackCounter.Success)
		s.Require().Equal(uint64(0), callbackCounter.Error)

		// Check if the proposal was created:
		proposal, err := simd.QueryProposal(ctx, "1")
		s.Require().NoError(err)
		s.Require().Equal(simd.Config().Denom, proposal.TotalDeposit[0].Denom)
		s.Require().Equal(fmt.Sprint(10000000+5000), proposal.TotalDeposit[0].Amount)
		// We do not check title and description of the proposal because this is a legacy proposal.
	})
}

func (s *ContractTestSuite) TestTimeoutAndChannelReopen() {
	ctx := context.Background()

	s.SetupContractTestSuite(ctx)
	wasmd, simd := s.ChainA, s.ChainB
	wasmdUser := s.UserA

	firstTokenID := "ica-token-0"
	firstChannelID := "channel-0"

	// var icaContract *types.IcaContract
	s.Run("MintIca", func() {
		// Mint a new ICA for the user
		err := s.Contract.MintIca(ctx, wasmdUser.KeyName(), nil, "--gas", "500000")
		s.Require().NoError(err)

		// Wait for the channel to get set up
		err = testutil.WaitForBlocks(ctx, 7, s.ChainA, s.ChainB)
		s.Require().NoError(err)

	})

	s.Run("TestTimeout", func() {
		// We will send a message to the host that will timeout after 3 seconds.
		// You cannot use 0 seconds because block timestamp will be greater than the timeout timestamp which is not allowed.
		// Host will not be able to respond to this message in time.

		// Stop the relayer so that the host cannot respond to the message:
		err := s.Relayer.StopRelayer(ctx, s.ExecRep)
		s.Require().NoError(err)

		time.Sleep(5 * time.Second)

		timeout := uint64(3)

		// Execute the contract:
		err = s.Contract.ExecuteCustomIcaMsgs(ctx, wasmdUser.KeyName(), firstTokenID, []proto.Message{}, icatypes.EncodingProtobuf, nil, &timeout, "--gas", "500000")
		s.Require().NoError(err)

		// Wait until timeout:
		err = testutil.WaitForBlocks(ctx, 5, wasmd, simd)
		s.Require().NoError(err)

		err = s.Relayer.StartRelayer(ctx, s.ExecRep)
		s.Require().NoError(err)

		// Wait until timeout acknoledgement is received:
		err = testutil.WaitForBlocks(ctx, 2, wasmd, simd)
		s.Require().NoError(err)

		// Flush to make sure the channel is closed in simd:
		err = s.Relayer.Flush(ctx, s.ExecRep, s.PathName, firstChannelID)
		s.Require().NoError(err)

		err = testutil.WaitForBlocks(ctx, 2, wasmd, simd)
		s.Require().NoError(err)

		// Check if the channel is closed:
		status, err := s.Contract.QueryChannelStatus(ctx, firstTokenID)
		s.Require().NoError(err)
		s.Require().Equal("closed", status)
	})

	s.Run("TestChannelReopen", func() {
		err := s.Contract.ExecuteIcaMsg(
			ctx, wasmdUser.KeyName(), firstTokenID, types.IcaControllerExecuteMsg{CreateChannel: &types.NoValue{}}, "--gas", "500000",
		)
		s.Require().NoError(err)

		status, err := s.Contract.QueryChannelStatus(ctx, firstTokenID)
		s.Require().NoError(err)
		s.Require().Equal("pending", status)

		// Wait until channel is reopened:
		err = testutil.WaitForBlocks(ctx, 10, wasmd, simd)
		s.Require().NoError(err)

		status, err = s.Contract.QueryChannelStatus(ctx, firstTokenID)
		s.Require().NoError(err)
		s.Require().Equal("open", status)
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
