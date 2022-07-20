use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub peer_code_id: u64,
    pub incremental_donation: Coin,
    pub collective_ratio: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");

// Maps `donations-peer` contract address to its owner address
pub const MEMBERS: Map<Addr, Addr> = Map::new("members");

pub const PENDING_INSTANTIATION: Item<Addr> = Item::new("pending_instantiation");
