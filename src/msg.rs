use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OpenResp)]
    IsOpen {},
    #[returns(ContractResp)]
    Status {},
}

#[cw_serde]
pub enum ExecMsg {
    Open {
        amount: Option<String>,
        cw20contract: Option<Addr>,
        priceamount: String,
        pricedenom: String,
        iscw20: String,
    },
    Buy {},
    Close {},
}

#[cw_serde]
pub struct InstantiateMsg {
}

#[cw_serde]
pub struct OpenResp {
    pub isopen: bool,
}

#[cw_serde]
pub struct ContractResp {
    pub isopen: bool,
    pub offeramount: u128,
    pub offerdenom: String,
    pub priceamount: u128,
    pub pricedenom: String,
    pub receiver: Addr,
    pub completed: bool,
    pub time: u64,
}
