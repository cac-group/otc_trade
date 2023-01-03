use cosmwasm_std::{coin, Addr, DepsMut, Response, StdResult};

use crate::state::{CLOSED, COMMISSION, HIGHEST_BID, HIGHEST_BIDDER, OWNER, WINNER};

use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, owner: Option<Addr>, sender: Addr) -> StdResult<Response> {

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    HIGHEST_BID.save(deps.storage, &coin(0, "atom"))?;
    HIGHEST_BIDDER.save(deps.storage, &None)?;
    CLOSED.save(deps.storage, &false)?;
    WINNER.save(deps.storage, &None)?;
    match owner {
        None => OWNER.save(deps.storage, &sender)?,
        Some(addr) => OWNER.save(deps.storage, &addr)?,
    }

    let commission = 5;
    COMMISSION.save(deps.storage, &commission)?;

    let resp = Response::new()
        .add_attribute("action", "Instantiation")
        .add_attribute("sender", sender.as_str())
        .add_attribute("commission", commission.to_string());

    Ok(resp)
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult, Addr};

    use crate::{
        msg::{HighestBidResp, OwnerResp, CurrentBidResp, ClosedResp, WinnerResp},
        state::{HIGHEST_BID, HIGHEST_BIDDER, OWNER, TOTAL_BIDS, CLOSED, WINNER},
    };

    pub fn highestbidvalue(deps: Deps) -> StdResult<HighestBidResp> {
        let highestbid = HIGHEST_BID.load(deps.storage)?;
        let highestbidder = HIGHEST_BIDDER.load(deps.storage)?;
        Ok(HighestBidResp {
            highestbid: highestbid,
            highestbidder: highestbidder,
        })
    }

    pub fn owneraddr(deps: Deps) -> StdResult<OwnerResp> {
        let owner = OWNER.load(deps.storage)?;
        Ok(OwnerResp { owner })
    }

    pub fn currentbid(deps: Deps, address: Addr) -> StdResult<CurrentBidResp> {
        let current_bid = TOTAL_BIDS.may_load(deps.storage, address)?;

        Ok(CurrentBidResp { currentbid: current_bid})
    }

    pub fn isclosed(deps: Deps) -> StdResult<ClosedResp> {
        let closed = CLOSED.load(deps.storage)?;
        Ok(ClosedResp { isclosed: closed })
    }

    pub fn winner(deps: Deps) -> StdResult<WinnerResp> {
        let winner = WINNER.load(deps.storage)?;
        Ok(WinnerResp { winner: winner.unwrap() })
    }
}

pub mod exec {

    use cosmwasm_std::{coin, coins, Addr, BankMsg, DepsMut, MessageInfo, Response, StdResult};

    use crate::{
        error::ContractError,
        state::{CLOSED, COMMISSION, HIGHEST_BID, HIGHEST_BIDDER, OWNER, TOTAL_BIDS, WINNER},
    };

    pub fn bid(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let closed = CLOSED.load(deps.storage)?;
        if closed == true {
            return Err(ContractError::ContractClosed);
        }

        if info.funds.iter().find(|coin| coin.denom == "atom") == None {
            return Err(ContractError::BiddingEmpty {});
        }

        let new_bid = info
            .funds
            .iter()
            .find(|coin| coin.denom == "atom")
            .unwrap()
            .amount
            .u128();
        let commission = COMMISSION.load(deps.storage)?;
        let owner = OWNER.load(deps.storage)?;

        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: coins(new_bid * commission / 100, "atom"),
        };

        let bid_without_commission = coin(new_bid - new_bid * commission / 100, "atom");

        let mut current_bid = TOTAL_BIDS.may_load(deps.storage, info.clone().sender)?;
        if current_bid == None {
            current_bid = Some(coin(0, "atom"));
        }

        let highest_bid = HIGHEST_BID.load(deps.storage)?;
        let new_highest_bid = current_bid.unwrap().amount + bid_without_commission.amount;

        if new_highest_bid < highest_bid.amount {
            return Err(ContractError::Biddingfail {});
        }

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "Bidding")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("new_highest_bid", new_highest_bid.to_string());

        HIGHEST_BID.save(deps.storage, &coin(new_highest_bid.into(), "atom"))?;
        HIGHEST_BIDDER.save(deps.storage, &Some(info.clone().sender))?;
        TOTAL_BIDS.update(deps.storage, info.clone().sender, |_| -> StdResult<_> {
            Ok(coin(new_highest_bid.into(), "atom"))
        })?;

        Ok(resp)
    }

    pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let closed = CLOSED.load(deps.storage)?;
        if closed == true {
            return Err(ContractError::ContractClosed);
        }
        let highestbidder = HIGHEST_BIDDER.load(deps.storage)?;

        if highestbidder == None {
            return Err(ContractError::NoBids);
        }

        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(ContractError::NotOwner {
                owner: owner.to_string(),
            });
        }

        let highest_bid = HIGHEST_BID.load(deps.storage)?;

        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: coins(highest_bid.amount.into(), highest_bid.denom),
        };

        WINNER.save(deps.storage, &highestbidder)?;
        CLOSED.save(deps.storage, &true)?;

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "Closing")
            .add_attribute("owner", info.sender.as_str())
            .add_attribute("winner", highestbidder.clone().unwrap().to_string())
            .add_attribute("bid", highest_bid.amount.to_string());

        TOTAL_BIDS.remove(deps.storage, highestbidder.unwrap());

        Ok(resp)
    }

    pub fn retract(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let closed = CLOSED.load(deps.storage)?;
        if closed == false {
            return Err(ContractError::ContractNotClosed);
        }

        let current_bid = TOTAL_BIDS.may_load(deps.storage, info.clone().sender)?;
        if current_bid == None{
            return Err(ContractError::NoBids);
        }

        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(
                current_bid.clone().unwrap().amount.into(),
                current_bid.clone().unwrap().denom,
            ),
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "Retracting")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("amount", current_bid.unwrap().amount);

        TOTAL_BIDS.remove(deps.storage, info.sender);

        Ok(resp)
    }

    pub fn retract_to(
        deps: DepsMut,
        info: MessageInfo,
        receiver: Addr,
    ) -> Result<Response, ContractError> {
        let closed = CLOSED.load(deps.storage)?;
        if closed == false {
            return Err(ContractError::ContractNotClosed);
        }

        let current_bid = TOTAL_BIDS.may_load(deps.storage, info.clone().sender)?;
        if current_bid == None{
            return Err(ContractError::NoBids);
        }

        let bank_msg = BankMsg::Send {
            to_address: receiver.to_string(),
            amount: coins(
                current_bid.clone().unwrap().amount.into(),
                current_bid.clone().unwrap().denom,
            ),
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "Retracting")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("amount", current_bid.unwrap().amount);

        TOTAL_BIDS.remove(deps.storage, info.sender);

        Ok(resp)
    }
}
