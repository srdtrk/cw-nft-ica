package types

import (
	"encoding/base64"
	"encoding/json"

	"github.com/cosmos/gogoproto/proto"

	codec "github.com/cosmos/cosmos-sdk/codec"

	icatypes "github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/types"
)

// newCoordinatorInstantiateMsg creates a new InstantiateMsg with channel init options.
func newCoordinatorInstantiateMsg(
	owner *string, icaCodeID uint64, cw721CodeID uint64,
	connectionId string, counterpartyConnectionId string,
	counterpartyPortId *string, txEncoding *string,
) string {
	type InstantiateMsg struct {
		Owner                   *string                `json:"owner,omitempty"`
		IcaControllerCodeID     uint64                 `json:"ica_controller_code_id"`
		Cw721IcaExtensionCodeID uint64                 `json:"cw721_ica_extension_code_id"`
		DefaultChanInitOptions  ChannelOpenInitOptions `json:"default_chan_init_options"`
	}

	channelOpenInitOptions := ChannelOpenInitOptions{
		ConnectionId:             connectionId,
		CounterpartyConnectionId: counterpartyConnectionId,
		CounterpartyPortId:       counterpartyPortId,
		TxEncoding:               txEncoding,
	}

	instantiateMsg := InstantiateMsg{
		Owner:                   owner,
		IcaControllerCodeID:     icaCodeID,
		Cw721IcaExtensionCodeID: cw721CodeID,
		DefaultChanInitOptions:  channelOpenInitOptions,
	}

	jsonBytes, err := json.Marshal(instantiateMsg)
	if err != nil {
		panic(err)
	}

	return string(jsonBytes)
}

// newCoordinatorMintIcaMsg creates a new MintIcaMsg with salt.
func newCoordinatorMintIcaMsg(salt *string) string {
	execMsg := CoordinatorExecuteMsg{
		MintIca: &CoordinatorMintIcaMsg{
			Salt: salt,
		},
	}

	jsonBytes, err := json.Marshal(execMsg)
	if err != nil {
		panic(err)
	}

	return string(jsonBytes)
}

// newCoordinatorIcaCustomMsg creates a new ExecuteIcaMsg with custom messages.
func newCoordinatorIcaCustomMsg(cdc codec.BinaryCodec, tokenID string, msgs []proto.Message, encoding string, memo *string, timeout *uint64) string {
	bz, err := icatypes.SerializeCosmosTxWithEncoding(cdc, msgs, encoding)
	if err != nil {
		panic(err)
	}

	messages := base64.StdEncoding.EncodeToString(bz)

	msg := CoordinatorExecuteMsg{
		ExecuteIcaMsg: &CoordinatorExecuteIcaMsg{
			TokenId: tokenID,
			Msg: &IcaControllerExecuteMsg{
				SendCustomIcaMessagesMsg: &SendCustomIcaMessagesMsg{
					Messages:       messages,
					PacketMemo:     memo,
					TimeoutSeconds: timeout,
				},
			},
		},
	}

	jsonBytes, err := json.Marshal(msg)
	if err != nil {
		panic(err)
	}

	return string(jsonBytes)
}

type CoordinatorExecuteMsg struct {
	MintIca       *CoordinatorMintIcaMsg    `json:"mint_ica,omitempty"`
	ExecuteIcaMsg *CoordinatorExecuteIcaMsg `json:"execute_ica_msg,omitempty"`
}

type CoordinatorMintIcaMsg struct {
	Salt *string `json:"salt,omitempty"`
}

type CoordinatorExecuteIcaMsg struct {
	TokenId string                   `json:"token_id"`
	Msg     *IcaControllerExecuteMsg `json:"msg,omitempty"`
}

type SendCustomIcaMessagesMsg struct {
	Messages       string  `json:"messages"`
	PacketMemo     *string `json:"packet_memo,omitempty"`
	TimeoutSeconds *uint64 `json:"timeout_seconds,omitempty"`
}

type IcaControllerExecuteMsg struct {
	SendCustomIcaMessagesMsg *SendCustomIcaMessagesMsg `json:"send_custom_ica_messages"`
}

func newNftIcaBimapQueryMsg(key string) map[string]interface{} {
	return map[string]interface{}{
		"nft_ica_controller_bimap": map[string]interface{}{
			"key": key,
		},
	}
}

func newGetIcaAddressQueryMsg(tokenID string) map[string]interface{} {
	return map[string]interface{}{
		"get_ica_address": map[string]interface{}{
			"token_id": tokenID,
		},
	}
}
