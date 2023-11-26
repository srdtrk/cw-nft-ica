package types

import "encoding/json"

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
