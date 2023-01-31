use cosmwasm_std::{DepsMut, Response};

use crate::error::ContractError;

use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//Receivers of commission, first address gets 0.08%, second one gets 0.02%
const COMMISSION_1: u128 = 8;
const COMMISSION_1_ADDRESS: &str = "juno1ep2umj6kn34g2ttjalsc5r9w8pt7sv4xnsvmdx";
const COMMISSION_2: u128 = 2;
const COMMISSION_2_ADDRESS: &str = "juno1wev8ptzj27aueu04wgvvl4gvurax6rj5la09yj";

pub fn instantiate(deps: DepsMut) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("action", "Instantiation"))
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{
        msg::{ContractResp, OpenResp},
        state::{COMPLETED, OFFER, OPEN, PRICE, RECEIVER, TIME_CREATION},
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
        let completed = COMPLETED.load(deps.storage)?;
        let time = TIME_CREATION.load(deps.storage)?;

        return Ok(ContractResp {
            isopen: open,
            offeramount: offer.amount.u128(),
            offerdenom: offer.denom,
            priceamount: price.amount.u128(),
            pricedenom: price.denom,
            receiver: receiver,
            completed: completed,
            time: time,
        });
    }
}

pub mod exec {
    use cosmwasm_std::{
        coin, coins, to_binary, Addr, BankMsg, DepsMut, Env, MessageInfo, Response, WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;

    use crate::{
        error::ContractError,
        state::{
            COMPLETED, IS_OFFER_CW20, IS_PRICE_CW20, OFFER, OPEN, PRICE, RECEIVER,
            TIME_CREATION,
        },
    };

    use super::{COMMISSION_1, COMMISSION_1_ADDRESS, COMMISSION_2, COMMISSION_2_ADDRESS};

    pub fn open(
        deps: DepsMut,
        info: MessageInfo,
        amount: Option<String>,
        cw20contract: Option<Addr>,
        priceamount: String,
        pricedenom: String,
        iscw20: String,
        env: Env,
    ) -> Result<Response, ContractError> {
        if info.funds.is_empty() && amount.is_none() {
            return Err(ContractError::NoFunds);
        }

        if amount.is_none() && info.funds[0].amount.u128() == 0 {
            return Err(ContractError::NoFunds);
        }

        if !info.funds.is_empty() && amount.is_some() {
            return Err(ContractError::NotOneAsset);
        }

        if amount.is_some() && cw20contract.is_none() || amount.is_none() && cw20contract.is_some()
        {
            return Err(ContractError::NoContract);
        }

        OPEN.save(deps.storage, &true)?;
        let resp;
        if amount.is_none() {
            let commission1_amount = info.funds[0].amount.u128() * COMMISSION_1 / 100000;
            let commission2_amount = info.funds[0].amount.u128() * COMMISSION_2 / 100000;
            let amount_without_commission =
                info.funds[0].amount.u128() - commission1_amount - commission2_amount;
            OFFER.save(
                deps.storage,
                &coin(
                    amount_without_commission.into(),
                    info.funds[0].denom.to_string(),
                ),
            )?;
            IS_OFFER_CW20.save(deps.storage, &false)?;

            let commission1_msg = BankMsg::Send {
                to_address: COMMISSION_1_ADDRESS.into(),
                amount: coins(commission1_amount.into(), info.funds[0].denom.to_string()),
            };
            let commission2_msg = BankMsg::Send {
                to_address: COMMISSION_2_ADDRESS.into(),
                amount: coins(commission2_amount.into(), info.funds[0].denom.to_string()),
            };

            resp = Response::new()
                .add_message(commission1_msg)
                .add_message(commission2_msg)
                .add_attribute("action", "Open Trade");
        } else {
            let commission1_amount = amount.clone().unwrap().parse::<u128>().unwrap() * COMMISSION_1 / 100000;
            let commission2_amount = amount.clone().unwrap().parse::<u128>().unwrap() * COMMISSION_2 / 100000;
            let amount_without_commission: u128 =
                amount.unwrap().parse::<u128>().unwrap() - commission1_amount - commission2_amount;

            let transfer1 = Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.clone().into(),
                recipient: COMMISSION_1_ADDRESS.into(),
                amount: commission1_amount.into(),
            };

            let execute_msg1 = WasmMsg::Execute {
                contract_addr: cw20contract.clone().unwrap().into(),
                msg: to_binary(&transfer1)?,
                funds: vec![],
            };

            let transfer2 = Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.clone().into(),
                recipient: COMMISSION_2_ADDRESS.into(),
                amount: commission2_amount.into(),
            };

            let execute_msg2 = WasmMsg::Execute {
                contract_addr: cw20contract.clone().unwrap().into(),
                msg: to_binary(&transfer2)?,
                funds: vec![],
            };

            let transfer3 = Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.clone().into(),
                recipient: env.contract.address.into(),
                amount: amount_without_commission.into(),
            };

            let execute_msg3 = WasmMsg::Execute {
                contract_addr: cw20contract.clone().unwrap().into(),
                msg: to_binary(&transfer3)?,
                funds: vec![],
            };

            OFFER.save(
                deps.storage,
                &coin(amount_without_commission.into(), cw20contract.unwrap()),
            )?;
            IS_OFFER_CW20.save(deps.storage, &true)?;

            resp = Response::new()
                .add_message(execute_msg1)
                .add_message(execute_msg2)
                .add_message(execute_msg3)
                .add_attribute("action", "Open cw20 trade");
        }

        COMPLETED.save(deps.storage, &false)?;
        RECEIVER.save(deps.storage, &info.sender)?;
        TIME_CREATION.save(deps.storage, &env.block.time.seconds())?;
        PRICE.save(deps.storage, &coin(priceamount.parse::<u128>().unwrap(), pricedenom))?;

        if iscw20 == "1" {
            IS_PRICE_CW20.save(deps.storage, &true)?;
        } else {
            IS_PRICE_CW20.save(deps.storage, &false)?;
        }

        Ok(resp)
    }

    pub fn buy(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let price = PRICE.load(deps.storage)?;
        let receiver = RECEIVER.load(deps.storage)?;
        let is_price_cw20 = IS_PRICE_CW20.load(deps.storage)?;
        let offer = OFFER.load(deps.storage)?;
        let is_offer_cw20 = IS_OFFER_CW20.load(deps.storage)?;
        let open = OPEN.load(deps.storage)?;

        if !open {
            return Err(ContractError::ContractClosed);
        }

        let mut resp = Response::new();
        if is_price_cw20 {
            let transfer = Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.clone().into(),
                recipient: receiver.to_string(),
                amount: price.amount,
            };

            let execute_msg = WasmMsg::Execute {
                contract_addr: price.denom,
                msg: to_binary(&transfer)?,
                funds: vec![],
            };
            resp = resp.add_message(execute_msg);
        } else {
            let price_msg = BankMsg::Send {
                to_address: receiver.to_string(),
                amount: coins(price.amount.u128(), price.denom),
            };

            resp = resp.add_message(price_msg);
        }

        if is_offer_cw20 {
            let transfer = Cw20ExecuteMsg::Transfer {
                recipient: info.sender.into_string(),
                amount: offer.amount,
            };

            let execute = WasmMsg::Execute {
                contract_addr: offer.denom.into(),
                msg: to_binary(&transfer)?,
                funds: vec![],
            };

            resp = resp.add_message(execute)
        } else {
            let offer_msg = BankMsg::Send {
                to_address: info.sender.into_string(),
                amount: coins(offer.amount.u128(), offer.denom),
            };
            resp = resp.add_message(offer_msg)
        }

        OPEN.save(deps.storage, &false)?;
        COMPLETED.save(deps.storage, &true)?;

        resp = resp.add_attribute("action", "buy, close trade and mark as completed");

        Ok(resp)
    }

    pub fn close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let open = OPEN.load(deps.storage)?;

        if open == false {
            return Err(ContractError::ContractClosed);
        }

        let receiver = RECEIVER.load(deps.storage)?;

        if info.sender != receiver {
            return Err(ContractError::NotOwner {
                owner: receiver.to_string(),
            });
        }

        let offer = OFFER.load(deps.storage)?;
        let is_offer_cw20 = IS_OFFER_CW20.load(deps.storage)?;
        let resp;

        if is_offer_cw20 {
            let transfer = Cw20ExecuteMsg::Transfer {
                recipient: receiver.to_string(),
                amount: offer.amount,
            };

            let execute = WasmMsg::Execute {
                contract_addr: offer.denom.into(),
                msg: to_binary(&transfer)?,
                funds: vec![],
            };

            resp = Response::new()
                .add_message(execute)
                .add_attribute("action", "Cancelling trade");
        } else {
            let return_msg = BankMsg::Send {
                to_address: receiver.to_string(),
                amount: coins(offer.amount.u128(), offer.denom),
            };
            resp = Response::new()
                .add_message(return_msg)
                .add_attribute("action", "Cancelling trade")
        }

        OPEN.save(deps.storage, &false)?;
        COMPLETED.save(deps.storage, &false)?;

        Ok(resp)
    }
}
