use cosmwasm_std::{Addr, Coin, Decimal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub peer_code_id: u64,
    pub incremental_donation: Coin,
    pub collective_ratio: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecMsg {
    Join {},
    Leave {},
    Donate {},
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    MemberPeerAddr {
        addr: String,
    },
    MembersList {
        start_after: Option<String>,
        limit: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResp {
    pub incremental_donation: Coin,
    pub collective_ratio: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct MemberPeerAddrResp {
    pub addr: Addr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Member {
    pub addr: Addr,
    pub peer_addr: Addr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct MembersListResp {
    pub members: Vec<Member>,
}
