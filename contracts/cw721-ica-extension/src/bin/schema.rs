use cosmwasm_schema::write_api;

use cw721_base::msg::QueryMsg as BaseQueryMsg;
use cw721_ica_extension::{ExecuteMsg, Extension, InstantiateMsg};

type QueryMsg = BaseQueryMsg<Extension>;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
