use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use error::ContractError;
use msg::InstantiateMsg;

mod contract;
pub mod msg;
pub mod error;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;
mod state;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    contract::instantiate(deps, info.sender, msg.amount, msg.denom, msg.cw20offer, msg.priceamount, msg.pricedenom, msg.cw20price, env)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use contract::query;
    use msg::QueryMsg::*;

    match msg {
        IsOpen{} => to_binary(&query::isopen(deps)?),
        Status{} => to_binary(&query::status(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: msg::ExecMsg,
) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match msg {
        Buy {} => exec::buy(deps, info),
        Close {} => exec::close(deps, info),
    }
}
