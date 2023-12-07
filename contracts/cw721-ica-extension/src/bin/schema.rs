use cosmwasm_schema::write_api;

use cw721_ica_extension::{ExecuteMsg, InstantiateMsg, Extension};
use cw721_base::msg::QueryMsg as BaseQueryMsg;

type QueryMsg = BaseQueryMsg<Extension>;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
