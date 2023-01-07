use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::{
    execute, instantiate, query, msg::{InstantiateMsg, ContractResp, QueryMsg, ExecMsg, OpenResp}, error::ContractError
};

#[derive(Debug)]
pub struct OTCContract(Addr);

impl OTCContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        funds: Vec<Coin>,
        price: Coin,
    ) -> Result<Self, ContractError> {
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                price: price.clone(),
            },
            &funds,
            label,
            None,
        )
        .map(OTCContract)
        .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn query_status(&self, app: &App) -> StdResult<ContractResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Status {})
    }

    #[track_caller]
    pub fn query_open(&self, app: &App) -> StdResult<OpenResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::IsOpen {})
    }

    #[track_caller]
    pub fn buy(&self, app: &mut App, sender: &Addr, funds: &[Coin]) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Buy {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }
    /*
    #[track_caller]
    pub fn query_highestbid(&self, app: &App) -> StdResult<HighestBidResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::HighestBid {})
    }

    #[track_caller]
    pub fn query_owner(&self, app: &App) -> StdResult<OwnerResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Owner {})
    }

    #[track_caller]
    pub fn bid(&self, app: &mut App, sender: &Addr, funds: &[Coin]) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Bid {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn close(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Close {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn retract(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Retract {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn retract_to(&self, app: &mut App, sender: &Addr, receiver: Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::RetractTo { receiver: receiver }, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }*/
}
