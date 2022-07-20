use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct State {
    pub donators: u64,
    pub incremental_donation: Coin,
    pub collective_ratio: Decimal,
    pub manager: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const OWNER: Item<Addr> = Item::new("owner");
