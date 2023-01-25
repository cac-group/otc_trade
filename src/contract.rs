use cosmwasm_std::{
    attr, coin, coins, to_binary, Addr, BankMsg, CosmosMsg, DepsMut, Env, Response, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{IS_OFFER_CW20, IS_PRICE_CW20, OFFER, OPEN, PRICE, RECEIVER, TIME_CREATION},
};

use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//Receivers of commission, first address gets 0.08%, second one gets 0.02%
const COMMISSION_1: u128 = 8;
const COMMISSION_1_ADDRESS: &str = "juno1ep2umj6kn34g2ttjalsc5r9w8pt7sv4xnsvmdx";
const COMMISSION_2: u128 = 2;
const COMMISSION_2_ADDRESS: &str = "juno1wev8ptzj27aueu04wgvvl4gvurax6rj5la09yj";

pub fn instantiate(
    deps: DepsMut,
    sender: Addr,
    offeramount: u128,
    offerdenom: Option<String>,
    cw20offer: Option<Addr>,
    priceamount: u128,
    pricedenom: Option<String>,
    cw20price: Option<Addr>,
    env: Env,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if offeramount == 0 {
        return Err(ContractError::NoFunds);
    }

    if (offerdenom.is_none() && cw20offer.is_none())
        || (pricedenom.is_none() && cw20price.is_none())
        || (offerdenom.is_some() && cw20offer.is_some())
        || (pricedenom.is_some() && cw20price.is_some())
    {
        return Err(ContractError::NotOneAsset);
    }

    let commission1_amount = offeramount * COMMISSION_1 / 100000;
    let commission2_amount = offeramount * COMMISSION_2 / 100000;

    let amount_with_commission = offeramount + commission1_amount + commission2_amount;

    let resp;
    if offerdenom.is_some() {
        //NOT cw20
        let message = CosmosMsg::Bank(BankMsg::Send {
            to_address: env.contract.address.into_string(),
            amount: coins(amount_with_commission, offerdenom.clone().unwrap()),
        });
        OFFER.save(deps.storage, &coin(offeramount, offerdenom.unwrap()))?;
        IS_OFFER_CW20.save(deps.storage, &false)?;
        resp = Response::new()
            .add_message(message)
            .add_attribute("action", "Instantiation")
            .add_attribute("sender", sender.as_str());
    } else {
        //cw20
        let transfer_from = Cw20ExecuteMsg::TransferFrom {
            owner: sender.clone().into(),
            recipient: env.contract.address.clone().into(),
            amount: amount_with_commission.into(),
        };
        let exec_message = WasmMsg::Execute {
            contract_addr: cw20offer.clone().unwrap().into(),
            msg: to_binary(&transfer_from)?,
            funds: vec![],
        };
        OFFER.save(deps.storage, &coin(offeramount, cw20offer.unwrap()))?;
        IS_OFFER_CW20.save(deps.storage, &true)?;
        resp = Response::new()
            .add_message(exec_message)
            .add_attributes(vec![
                attr("action", "Instantiation"),
                attr("sender", sender.as_str()),
            ]);
    }

    if pricedenom.is_some() {
        PRICE.save(deps.storage, &coin(priceamount, pricedenom.unwrap()))?;
        IS_PRICE_CW20.save(deps.storage, &false)?;
    } else {
        PRICE.save(
            deps.storage,
            &coin(priceamount, cw20price.unwrap().to_string()),
        )?;
        IS_PRICE_CW20.save(deps.storage, &true)?;
    }

    OPEN.save(deps.storage, &true)?;
    RECEIVER.save(deps.storage, &sender)?;
    TIME_CREATION.save(deps.storage, &env.block.time.seconds())?;

    Ok(resp)
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{
        msg::{ContractResp, OpenResp},
        state::{OFFER, OPEN, PRICE, RECEIVER, TIME_CREATION},
    };

    pub fn isopen(deps: Deps) -> StdResult<OpenResp> {
        let open = OPEN.load(deps.storage)?;
        return Ok(OpenResp { isopen: open });
    }

    pub fn status(deps: Deps) -> StdResult<ContractResp> {
        let open = OPEN.load(deps.storage)?;
        let price = PRICE.load(deps.storage)?;
        let receiver = RECEIVER.load(deps.storage)?;
        let offer = OFFER.load(deps.storage)?;
        let time = TIME_CREATION.load(deps.storage)?;

        return Ok(ContractResp {
            isopen: open,
            offeramount: offer.amount.u128(),
            offerdenom: offer.denom,
            priceamount: price.amount.u128(),
            pricedenom: price.denom,
            receiver: receiver,
            time: time,
        });
    }
}

pub mod exec {
    use cosmwasm_std::{
        attr, coins, to_binary, BankMsg, CosmosMsg, DepsMut, MessageInfo, Response, WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;

    use crate::{
        error::ContractError,
        state::{IS_OFFER_CW20, IS_PRICE_CW20, OFFER, OPEN, PRICE, RECEIVER},
    };

    use super::{COMMISSION_1, COMMISSION_1_ADDRESS, COMMISSION_2, COMMISSION_2_ADDRESS};

    pub fn buy(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {

        let price = PRICE.load(deps.storage)?;
        let receiver = RECEIVER.load(deps.storage)?;
        let mut resp = Response::new();

        let is_price_cw20 = IS_PRICE_CW20.load(deps.storage)?;
        if !is_price_cw20 {
            let to_owner_message: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.clone().into_string(),
                amount: coins(price.amount.u128(), price.denom),
            });
            resp = resp.add_message(to_owner_message);
        } else {
            let transfer_from = Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.clone().into(),
                recipient: receiver.clone().into_string(),
                amount: price.amount,
            };
            let exec_message = WasmMsg::Execute {
                contract_addr: price.denom,
                msg: to_binary(&transfer_from)?,
                funds: vec![],
            };
            resp = resp.add_message(exec_message);
        }

        let offer = OFFER.load(deps.storage)?;
        let commission1 = offer.amount.u128() * COMMISSION_1 / 100000;
        let commission2 = offer.amount.u128() * COMMISSION_2 / 100000;
        let is_cw20 = IS_OFFER_CW20.load(deps.storage)?;

        if is_cw20 {
            let transfer1 = Cw20ExecuteMsg::Transfer {
                recipient: COMMISSION_1_ADDRESS.into(),
                amount: commission1.into(),
            };

            let execute_msg1 = WasmMsg::Execute {
                contract_addr: offer.denom.clone().into(),
                msg: to_binary(&transfer1)?,
                funds: vec![],
            };

            let transfer2 = Cw20ExecuteMsg::Transfer {
                recipient: COMMISSION_2_ADDRESS.into(),
                amount: commission2.into(),
            };

            let execute_msg2 = WasmMsg::Execute {
                contract_addr: offer.denom.clone().into(),
                msg: to_binary(&transfer2)?,
                funds: vec![],
            };

            let transfer3 = Cw20ExecuteMsg::Transfer {
                recipient: info.sender.clone().into(),
                amount: offer.amount,
            };

            let execute_msg3 = WasmMsg::Execute {
                contract_addr: offer.denom.clone().into(),
                msg: to_binary(&transfer3)?,
                funds: vec![],
            };

            resp = resp
                .add_message(execute_msg1)
                .add_message(execute_msg2)
                .add_message(execute_msg3)
                .add_attributes(vec![
                    attr("action", "buying and closing"),
                    attr("sender", info.sender),
                ]);
        } else {
            let transfer1 = BankMsg::Send {
                to_address: COMMISSION_1_ADDRESS.into(),
                amount: coins(commission1, offer.denom.clone()),
            };

            let transfer2 = BankMsg::Send {
                to_address: COMMISSION_2_ADDRESS.into(),
                amount: coins(commission2, offer.denom.clone()),
            };

            let transfer3 = BankMsg::Send {
                to_address: info.sender.clone().into(),
                amount: coins(offer.amount.u128(), offer.denom.clone()),
            };

            resp = resp
                .add_message(transfer1)
                .add_message(transfer2)
                .add_message(transfer3)
                .add_attribute("action", "Buying and closing OTC Deal")
                .add_attribute("recipient", info.sender.as_str());
        }

        OPEN.save(deps.storage, &false)?;

        Ok(resp)
    }

    pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let open = OPEN.load(deps.storage)?;
        if open == false {
            return Err(ContractError::ContractClosed);
        }

        OPEN.save(deps.storage, &true)?;

        let receiver = RECEIVER.load(deps.storage)?;

        if info.sender != receiver {
            return Err(ContractError::NotOwner {
                owner: receiver.to_string(),
            });
        }

        let resp;
        let offer = OFFER.load(deps.storage)?;
        let commission1 = offer.amount.u128() * COMMISSION_1 / 100000;
        let commission2 = offer.amount.u128() * COMMISSION_2 / 100000;
        let is_cw20 = IS_OFFER_CW20.load(deps.storage)?;

        if is_cw20 {
            let transfer1 = Cw20ExecuteMsg::Transfer {
                recipient: COMMISSION_1_ADDRESS.into(),
                amount: commission1.into(),
            };

            let execute_msg1 = WasmMsg::Execute {
                contract_addr: offer.denom.clone().into(),
                msg: to_binary(&transfer1)?,
                funds: vec![],
            };

            let transfer2 = Cw20ExecuteMsg::Transfer {
                recipient: COMMISSION_2_ADDRESS.into(),
                amount: commission2.into(),
            };

            let execute_msg2 = WasmMsg::Execute {
                contract_addr: offer.denom.clone().into(),
                msg: to_binary(&transfer2)?,
                funds: vec![],
            };

            let transfer3 = Cw20ExecuteMsg::Transfer {
                recipient: info.sender.clone().into(),
                amount: offer.amount,
            };

            let execute_msg3 = WasmMsg::Execute {
                contract_addr: offer.denom.clone().into(),
                msg: to_binary(&transfer3)?,
                funds: vec![],
            };

            resp = Response::new()
                .add_message(execute_msg1)
                .add_message(execute_msg2)
                .add_message(execute_msg3)
                .add_attributes(vec![
                    attr("action", "closing_contract"),
                    attr("sender", info.sender),
                ])
        } else {
            let transfer1 = BankMsg::Send {
                to_address: COMMISSION_1_ADDRESS.into(),
                amount: coins(commission1, offer.denom.clone()),
            };

            let transfer2 = BankMsg::Send {
                to_address: COMMISSION_2_ADDRESS.into(),
                amount: coins(commission2, offer.denom.clone()),
            };

            let transfer3 = BankMsg::Send {
                to_address: info.sender.clone().into(),
                amount: coins(offer.amount.u128(), offer.denom.clone()),
            };

            resp = Response::new()
                .add_message(transfer1)
                .add_message(transfer2)
                .add_message(transfer3)
                .add_attribute("action", "Cancelling OTC deal")
                .add_attribute("recipient", info.sender.as_str());
        }

        Ok(resp)
    }
}
