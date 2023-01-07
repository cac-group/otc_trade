use cosmwasm_std::{Addr, Coin, DepsMut, Response};

use crate::{
    error::ContractError,
    state::{OFFER, OPEN, PRICE, RECEIVER},
};

use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//Receiver of commission
const COMMISSION: u128 = 1;
const COMMISSION_ADDRESS: &str = "address";
pub fn instantiate(
    deps: DepsMut,
    sender: Addr,
    funds: Vec<Coin>,
    price: Coin,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if funds.is_empty() {
        return Err(ContractError::NoFunds);
    }

    OFFER.save(deps.storage, &funds)?;
    OPEN.save(deps.storage, &true)?;
    RECEIVER.save(deps.storage, &sender)?;
    PRICE.save(deps.storage, &price)?;

    let resp = Response::new()
        .add_attribute("action", "Instantiation")
        .add_attribute("sender", sender.as_str());

    Ok(resp)
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{msg::{OpenResp, ContractResp}, state::{OPEN, PRICE, RECEIVER, OFFER}};

    pub fn isopen(deps: Deps) -> StdResult<OpenResp> {
        let open = OPEN.load(deps.storage)?;
        return Ok(OpenResp { isopen: open });
    }

    pub fn status(deps: Deps) -> StdResult<ContractResp> {
        let open = OPEN.load(deps.storage)?;
        let price = PRICE.load(deps.storage)?;
        let receiver = RECEIVER.load(deps.storage)?;
        let offer = OFFER.load(deps.storage)?;

        return Ok(ContractResp { isopen: open, offer: offer, price: price, receiver: receiver });
    }
}

pub mod exec {
    use cosmwasm_std::{coins, BankMsg, DepsMut, MessageInfo, Response, Env};

    use crate::{
        error::ContractError,
        state::{OPEN, PRICE, RECEIVER, OFFER},
    };

    use super::{COMMISSION, COMMISSION_ADDRESS};

    pub fn buy(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let price = PRICE.load(deps.storage)?;
        if info
            .funds
            .iter()
            .find(|coin| coin.denom == price.denom && coin.amount >= price.amount)
            == None
        {
            return Err(ContractError::OfferFail);
        }

        let payment = info
            .funds
            .iter()
            .find(|coin| coin.denom == price.denom && coin.amount >= price.amount)
            .unwrap();

        let commission_amount = payment.amount.u128() * COMMISSION / 10000;
        let amount_without_commission = payment.amount.u128() - commission_amount;
        let commission_msg = BankMsg::Send {
            to_address: COMMISSION_ADDRESS.to_string(),
            amount: coins(commission_amount, payment.clone().denom),
        };

        let receiver = RECEIVER.load(deps.storage)?;
        let to_owner_msg = BankMsg::Send {
            to_address: receiver.to_string(),
            amount: coins(amount_without_commission, payment.clone().denom),
        };

        let offer = OFFER.load(deps.storage)?;
        let to_user_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: offer,
        };

        let open = OPEN.load(deps.storage)?;
        if open == false {
            return Err(ContractError::ContractClosed);
        }
        OPEN.save(deps.storage, &false)?;

        let resp = Response::new()
            .add_message(commission_msg)
            .add_message(to_owner_msg)
            .add_message(to_user_msg)
            .add_attribute("action", "Buying and closing OTC deal")
            .add_attribute("buyer", info.sender.as_str());

        Ok(resp)
    }

    pub fn close(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {

        let open = OPEN.load(deps.storage)?;
        if open == false {
            return Err(ContractError::ContractClosed);
        }

        let receiver = RECEIVER.load(deps.storage)?;

        if info.sender != receiver {
            return Err(ContractError::NotOwner { owner: receiver.to_string() });
        }

        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: receiver.to_string(),
            amount: balance,
        };
        
        OPEN.save(deps.storage, &false)?;

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "Cancelling OTC deal")
            .add_attribute("buyer", info.sender.as_str());

        Ok(resp)

    }
}
