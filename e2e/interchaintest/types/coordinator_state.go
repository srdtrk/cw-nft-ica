package types

// CoordinatorContractState is used to represent its state in Contract's storage
type CoordinatorContractState struct {
	DefaultChanInitOptions   ChannelOpenInitOptions `json:"default_chan_init_options"`
	IcaControllerCodeID      uint64                 `json:"ica_controller_code_id"`
	Cw721IcaExtensionAddress string                 `json:"cw721_ica_extension_address"`
}

// ChannelState is used to represent the state of a channel in the Coordinator's storage
type ChannelState struct {
	Status    string  `json:"status"`
	ChannelId *string `json:"channel_id,omitempty"`
}
