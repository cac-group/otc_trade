use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Map, Item};

pub const TOTAL_BIDS: Map<Addr, Coin> = Map::new("total_bids");
pub const HIGHEST_BID: Item<Coin> = Item::new("highest_bid");
pub const HIGHEST_BIDDER: Item<Option<Addr>> = Item::new("highest_bidder");
pub const COMMISSION: Item<u128> = Item::new("bidding_commission");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const CLOSED: Item<bool> = Item::new("closed");
pub const WINNER: Item<Option<Addr>> = Item::new("winner");