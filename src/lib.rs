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
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, msg.owner, info.sender)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use contract::query;
    use msg::QueryMsg::*;

    match msg {
        HighestBid {} => to_binary(&query::highestbidvalue(deps)?),
        Owner {} => to_binary(&query::owneraddr(deps)?),
        CurrentBid { address } => to_binary(&query::currentbid(deps, address)?),
        IsClosed{} => to_binary(&query::isclosed(deps)?),
        Winner{} => to_binary(&query::winner(deps)?),
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
        Bid {} => exec::bid(deps, info),
        Close {} => exec::close(deps, info),
        Retract{} => exec::retract(deps, info),
        RetractTo { receiver } => exec::retract_to(deps, info, receiver),
    }
}
