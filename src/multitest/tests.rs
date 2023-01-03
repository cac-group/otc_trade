use crate::{
    error::ContractError,
    msg::{HighestBidResp, OwnerResp},
    multitest::contract::BiddingContract,
};
use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::{App, Executor};

#[test]
fn instantiate_with_different_owner() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");
    let owner = Addr::unchecked("owner");

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &Some(owner))
            .unwrap();

    let resp = contract.query_owner(&app).unwrap();
    assert_eq!(
        resp,
        OwnerResp {
            owner: Addr::unchecked("owner")
        }
    );
}

#[test]
fn instantiate_with_no_owner() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    let resp = contract.query_owner(&app).unwrap();
    assert_eq!(
        resp,
        OwnerResp {
            owner: Addr::unchecked("sender")
        }
    );
}

#[test]
fn highestbid_with_nobids() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    let resp = contract.query_highestbid(&app).unwrap();
    assert_eq!(
        resp,
        HighestBidResp {
            highestbid: coin(0, "atom"),
            highestbidder: None,
        }
    );
}

#[test]
fn bid_failed_nofunds() {
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(100000, "atom"))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    let err = contract.bid(&mut app, &sender, &[]).unwrap_err();

    assert_eq!(ContractError::BiddingEmpty {}, err);
}
#[test]
fn successful_first_bid() {
    let sender = Addr::unchecked("sender");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(100000, "atom"))
            .unwrap();
    });

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    let resp = contract.query_highestbid(&app).unwrap();

    assert_eq!(
        resp,
        HighestBidResp {
            highestbid: coin(950, "atom"),
            highestbidder: Some(sender),
        }
    );
}

#[test]
fn second_bid_fails() {
    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    let err = contract
        .bid(&mut app, &sender2, &coins(500, "atom"))
        .unwrap_err();

    assert_eq!(err, ContractError::Biddingfail {},);
}

#[test]
fn second_bid_succeeds() {
    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(1200, "atom"))
        .unwrap();
    let resp = contract.query_highestbid(&app).unwrap();

    assert_eq!(
        resp,
        HighestBidResp {
            highestbid: coin(1140, "atom"),
            highestbidder: Some(sender2),
        }
    );
}

#[test]
fn bidding_accumulates() {
    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(1200, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();
    let resp = contract.query_highestbid(&app).unwrap();

    assert_eq!(
        resp,
        HighestBidResp {
            highestbid: coin(2850, "atom"),
            highestbidder: Some(sender),
        }
    );
}

#[test]
fn commission_is_received() {
    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(1200, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    assert_eq!(
        app.wrap().query_all_balances(receiver).unwrap(),
        coins(210, "atom")
    );
}

#[test]
fn closing_contract_successfully() {
    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(1200, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    contract.close(&mut app, &receiver.clone()).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(receiver).unwrap(),
        coins(3060, "atom")
    );
}

#[test]
fn closing_twice() {
    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(1200, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    contract.close(&mut app, &receiver.clone()).unwrap();
    let err = contract.close(&mut app, &receiver.clone()).unwrap_err();

    assert_eq!(
        err,
        ContractError::ContractClosed
    );
}

#[test]
fn unauthorized_closing() {
    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(1200, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    let err = contract.close(&mut app, &sender).unwrap_err();

    assert_eq!(
        err,
        ContractError::NotOwner { owner: receiver.clone().to_string() }
    );
}

#[test]
fn closing_empty_contract() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let code_id = BiddingContract::store_code(&mut app);

    let contract =
        BiddingContract::instantiate(&mut app, code_id, &sender, "Bidding contract", &None)
            .unwrap();

    let err = contract.close(&mut app, &sender).unwrap_err();

    assert_eq!(
        err,
        ContractError::NoBids
    );
}

#[test]
fn retract_funds() {

    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(2000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(2000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    contract.close(&mut app, &receiver.clone()).unwrap();
    contract.retract(&mut app, &sender2).unwrap();
    assert_eq!(
        app.wrap().query_all_balances(sender2).unwrap(),
        coins(1900, "atom")
    );
}

#[test]
fn retract_funds_to_receiver() {

    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(2000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(2000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    contract.close(&mut app, &receiver.clone()).unwrap();

    let receiver2 = Addr::unchecked("receiver2");
    contract.retract_to(&mut app, &sender2, receiver2.clone()).unwrap();
    assert_eq!(
        app.wrap().query_all_balances(receiver2).unwrap(),
        coins(1900, "atom")
    );
}

#[test]
fn retract_with_no_funds() {

    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(2000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    contract.close(&mut app, &receiver.clone()).unwrap();

    let err = contract.retract(&mut app, &sender).unwrap_err();
    assert_eq!(
        err,
        ContractError::NoBids,
    );
}

#[test]
fn retract_not_closed() {

    let sender = Addr::unchecked("sender");
    let sender2 = Addr::unchecked("sender2");
    let receiver = Addr::unchecked("receiver");
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender.clone(), coins(100000, "atom"))
            .unwrap();
    });

    app.send_tokens(sender.clone(), sender2.clone(), &coins(50000, "atom"))
        .unwrap();

    let code_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidding contract",
        &Some(receiver.clone()),
    )
    .unwrap();

    contract
        .bid(&mut app, &sender, &coins(1000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender2, &coins(2000, "atom"))
        .unwrap();
    contract
        .bid(&mut app, &sender, &coins(2000, "atom"))
        .unwrap();

    let err = contract.retract(&mut app, &sender).unwrap_err();
    assert_eq!(
        err,
        ContractError::ContractNotClosed,
    );
}