use cosmwasm_std::{Coin, Addr};
use cw_storage_plus::Item;

pub const OFFER: Item<Coin> = Item::new("offer");
pub const IS_OFFER_CW20: Item<bool> = Item::new("cw20offer");
pub const PRICE: Item<Coin> = Item::new("price");
pub const IS_PRICE_CW20: Item<bool> = Item::new("cw20price");
pub const OPEN: Item<bool> = Item::new("open");
pub const RECEIVER: Item<Addr> = Item::new("receiver");
pub const TIME_CREATION: Item<u64> = Item::new("time_creation");