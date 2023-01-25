use cosmwasm_std::Addr;
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OpenResp)]
    IsOpen{},
    #[returns(ContractResp)]
    Status{},
}

#[cw_serde]
pub enum ExecMsg {
    Buy {},
    Close {},
}

#[cw_serde]
pub struct OpenResp{
    pub isopen: bool,
}

#[cw_serde]
pub struct ContractResp{
    pub isopen: bool,
    pub offeramount: u128,
    pub offerdenom: String,
    pub priceamount: u128,
    pub pricedenom: String,
    pub receiver: Addr,
    pub time: u64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub amount: u128,
    pub denom: Option<String>,
    pub cw20offer: Option<Addr>,
    pub priceamount: u128,
    pub pricedenom: Option<String>,
    pub cw20price: Option<Addr>,
}