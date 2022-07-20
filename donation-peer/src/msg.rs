use cosmwasm_std::{Addr, Coin, Decimal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ManagerExec {
    Donate {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub owner: String,
    pub incremental_donation: Coin,
    pub collective_ratio: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecMsg {
    Donate {},
    Withdraw {},
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Owner {},
    Manager {},
    PendingDonations { denom: Option<String> },
    Config {},
    Donators {},
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OwnerResp {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ManagerResp {
    pub manager: Addr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PendingDonationsResp {
    pub donations: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResp {
    pub incremental_donation: Coin,
    pub collective_ratio: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DonatorsResp {
    pub donators: u64,
}
