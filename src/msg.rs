use cosmwasm_std::Addr;
use cosmwasm_std::Coin;
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
    pub offer: Vec<Coin>,
    pub price: Coin,
    pub receiver: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub price: Coin,
}