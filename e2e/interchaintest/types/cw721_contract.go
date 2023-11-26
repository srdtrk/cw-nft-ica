package types

import "context"

type Cw721Contract struct {
	Contract
}

func NewCw721Contract(contract Contract) *Cw721Contract {
	return &Cw721Contract{
		Contract: contract,
	}
}

// QueryTokens queries the cw721 contract for the tokens owned by the given address
func (c *Cw721Contract) QueryTokens(ctx context.Context, owner string) (*TokensResponse, error) {
	queryResp := QueryResponse[TokensResponse]{}
	err := c.chain.QueryContract(ctx, c.Address, newTokensQueryMsg(owner), &queryResp)
	if err != nil {
		return nil, err
	}

	resp, err := queryResp.GetResp()
	if err != nil {
		return nil, err
	}

	return &resp, nil
}

// newTokensQueryMsg creates a new TokensQueryMsg.
// This function returns a map[string]interface{} instead of []byte
// because interchaintest uses json.Marshal to convert the map to a string
func newTokensQueryMsg(owner string) map[string]interface{} {
	return map[string]interface{}{
		"tokens": map[string]interface{}{
			"owner": owner,
		},
	}
}

// TokensResponse is the response type for the TokensQueryMsg
type TokensResponse struct {
	Tokens []string `json:"tokens"`
}
