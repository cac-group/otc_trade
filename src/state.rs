use cosmwasm_std::{Coin, Addr};
use cw_storage_plus::Item;

pub const OFFER: Item<Vec<Coin>> = Item::new("offer");
pub const PRICE: Item<Coin> = Item::new("price");
pub const OPEN: Item<bool> = Item::new("open");
pub const RECEIVER: Item<Addr> = Item::new("receiver");