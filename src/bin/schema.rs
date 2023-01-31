use cosmwasm_schema::write_api;
use otc_trade::msg::{ExecMsg, QueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecMsg,
        query: QueryMsg,
    }
}