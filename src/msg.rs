use cosmwasm_std::{Addr, Coin};
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::QueryResponses;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HighestBidResp)]
    HighestBid {},
    #[returns(OwnerResp)]
    Owner {},
    #[returns(ClosedResp)]
    IsClosed{},
    #[returns(WinnerResp)]
    Winner{},
    #[returns(CurrentBidResp)]
    CurrentBid{ address: Addr },

}

#[cw_serde]
pub enum ExecMsg {
    Bid {},
    Close{},
    Retract{},
    RetractTo { receiver: Addr },
}
#[cw_serde]
pub struct HighestBidResp {
    pub highestbid: Coin,
    pub highestbidder: Option<Addr>,
}

#[cw_serde]
pub struct OwnerResp {
    pub owner: Addr,
}

#[cw_serde]
pub struct WinnerResp {
    pub winner: Addr,
}

#[cw_serde]
pub struct ClosedResp{
    pub isclosed: bool,
}
#[cw_serde]
pub struct CurrentBidResp{
    pub currentbid: Option<Coin>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<Addr>,
}