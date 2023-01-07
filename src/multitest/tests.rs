use crate::{error::ContractError, msg::{ContractResp, OpenResp}, multitest::contract::OTCContract};
use cosmwasm_std::{coin, coins, Addr, Coin};
use cw_multi_test::{App, Executor};

#[test]
fn instantiate_correctly() {
    let sender = Addr::unchecked("sender");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(10000000, "ujuno"))
            .unwrap();
    });

    let code_id = OTCContract::store_code(&mut app);

    let contract = OTCContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "OTC contract",
        coins(10000000, "ujuno"),
        coin(10000, "uatom"),
    )
    .unwrap();

    let resp = contract.query_status(&app).unwrap();
    assert_eq!(
        resp,
        ContractResp {
            isopen: true,
            offer: coins(10000000, "ujuno"),
            price: coin(10000, "uatom"),
            receiver: sender
        }
    );
}

#[test]
fn instantiate_with_no_funds() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let code_id = OTCContract::store_code(&mut app);

    let resp = OTCContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "OTC contract",
        Vec::new(),
        coin(10000, "uatom"),
    )
    .unwrap_err();

    assert_eq!(resp, ContractError::NoFunds,);
}

#[test]
fn buy_succesfully() {
    let seller = Addr::unchecked("seller");
    let buyer = Addr::unchecked("buyer");
    let mut funds: Vec<Coin> = Vec::new();
    funds.push(coin(10000000, "ujuno"));
    funds.push(coin(100000, "uatom"));
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &seller.clone(), funds)
            .unwrap();
    });

    app.send_tokens(seller.clone(), buyer.clone(), &coins(50000, "uatom"))
        .unwrap();

    let code_id = OTCContract::store_code(&mut app);

    let contract = OTCContract::instantiate(
        &mut app,
        code_id,
        &seller,
        "OTC contract",
        coins(10000000, "ujuno"),
        coin(50000, "uatom"),
    )
    .unwrap();

    contract.buy(&mut app, &buyer, &coins(50000, "uatom")).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(buyer).unwrap(),
        coins(10000000, "ujuno")
    );
}


#[test]
fn buy_not_enough_funds() {
    let seller = Addr::unchecked("seller");
    let buyer = Addr::unchecked("buyer");
    let mut funds: Vec<Coin> = Vec::new();
    funds.push(coin(10000000, "ujuno"));
    funds.push(coin(100000, "uatom"));
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &seller.clone(), funds)
            .unwrap();
    });

    app.send_tokens(seller.clone(), buyer.clone(), &coins(50000, "uatom"))
        .unwrap();

    let code_id = OTCContract::store_code(&mut app);

    let contract = OTCContract::instantiate(
        &mut app,
        code_id,
        &seller,
        "OTC contract",
        coins(10000000, "ujuno"),
        coin(100000, "uatom"),
    )
    .unwrap();

    let resp = contract.buy(&mut app, &buyer, &coins(50000, "uatom")).unwrap_err();

    assert_eq!(
        resp,
        ContractError::OfferFail
    );
}


#[test]
fn buy_succesfully_pay_more() {
    let seller = Addr::unchecked("seller");
    let buyer = Addr::unchecked("buyer");
    let mut funds: Vec<Coin> = Vec::new();
    funds.push(coin(10000000, "ujuno"));
    funds.push(coin(100000, "uatom"));
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &seller.clone(), funds)
            .unwrap();
    });

    app.send_tokens(seller.clone(), buyer.clone(), &coins(50000, "uatom"))
        .unwrap();

    let code_id = OTCContract::store_code(&mut app);

    let contract = OTCContract::instantiate(
        &mut app,
        code_id,
        &seller,
        "OTC contract",
        coins(10000000, "ujuno"),
        coin(10000, "uatom"),
    )
    .unwrap();

    contract.buy(&mut app, &buyer, &coins(50000, "uatom")).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(buyer).unwrap(),
        coins(10000000, "ujuno")
    );
}

#[test]
fn closed_succesfully() {
    let seller = Addr::unchecked("seller");
    let buyer = Addr::unchecked("buyer");
    let mut funds: Vec<Coin> = Vec::new();
    funds.push(coin(10000000, "ujuno"));
    funds.push(coin(100000, "uatom"));
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &seller.clone(), funds)
            .unwrap();
    });

    app.send_tokens(seller.clone(), buyer.clone(), &coins(50000, "uatom"))
        .unwrap();

    let code_id = OTCContract::store_code(&mut app);

    let contract = OTCContract::instantiate(
        &mut app,
        code_id,
        &seller,
        "OTC contract",
        coins(10000000, "ujuno"),
        coin(50000, "uatom"),
    )
    .unwrap();

    contract.buy(&mut app, &buyer, &coins(50000, "uatom")).unwrap();

    let resp = contract.query_open(&app).unwrap();

    assert_eq!(
        resp,
        OpenResp { isopen: false }
    );
}

#[test]
fn is_open() {
    let sender = Addr::unchecked("sender");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(10000000, "ujuno"))
            .unwrap();
    });

    let code_id = OTCContract::store_code(&mut app);

    let contract = OTCContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "OTC contract",
        coins(10000000, "ujuno"),
        coin(10000, "uatom"),
    )
    .unwrap();

    let resp = contract.query_open(&app).unwrap();

    assert_eq!(
        resp,
        OpenResp { isopen: true }
    );
}

#[test]
fn cant_buy_closed() {
    let seller = Addr::unchecked("seller");
    let buyer = Addr::unchecked("buyer");
    let mut funds: Vec<Coin> = Vec::new();
    funds.push(coin(10000000, "ujuno"));
    funds.push(coin(100000, "uatom"));
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &seller.clone(), funds)
            .unwrap();
    });

    app.send_tokens(seller.clone(), buyer.clone(), &coins(100000, "uatom"))
        .unwrap();

    let code_id = OTCContract::store_code(&mut app);

    let contract = OTCContract::instantiate(
        &mut app,
        code_id,
        &seller,
        "OTC contract",
        coins(10000000, "ujuno"),
        coin(50000, "uatom"),
    )
    .unwrap();

    contract.buy(&mut app, &buyer, &coins(50000, "uatom")).unwrap();
    let resp = contract.buy(&mut app, &buyer, &coins(50000, "uatom")).unwrap_err();

    assert_eq!(
        resp,
        ContractError::ContractClosed,
    );
}