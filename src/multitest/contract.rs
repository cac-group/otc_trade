use cosmwasm_std::{Addr, StdResult};
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
        offeramount: u128,
        offerdenom: Option<String>,
        offercw20: Option<Addr>,
        priceamount: u128,
        pricedenom: Option<String>,
        pricecw20: Option<Addr>
    ) -> Result<Self, ContractError> {
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                amount: offeramount,
                denom: offerdenom,
                cw20offer: offercw20,
                priceamount: priceamount,
                pricedenom: pricedenom,
                cw20price: pricecw20,
            },
            &[],
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
    pub fn buy(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Buy {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }
}
