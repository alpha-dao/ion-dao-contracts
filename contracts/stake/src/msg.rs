use cosmwasm_std::{Addr, Order, Uint128};
pub use cw_controllers::ClaimsResponse;
pub use cw_utils::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub denom: String,
    pub unstaking_duration: Option<Duration>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Stake {},
    Unstake {
        amount: Uint128,
    },
    Fund {},
    Claim {},
    UpdateConfig {
        admin: Option<Addr>,
        duration: Option<Duration>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RangeOrder {
    Asc,
    Desc,
}

impl From<RangeOrder> for Order {
    fn from(order: RangeOrder) -> Self {
        match order {
            RangeOrder::Asc => Order::Ascending,
            RangeOrder::Desc => Order::Descending,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // DAO DAO compatability
    VotingPowerAtHeight {
        address: String,
        height: Option<u64>,
    },
    TotalPowerAtHeight {
        height: Option<u64>,
    },
    StakedValue {
        address: String,
    },
    TotalValue {},
    Claims {
        address: String,
    },

    RangeStakers {
        start_at: Option<String>,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    },

    GetConfig {},
    Info {},
    Dao {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VotingPowerAtHeightResponse {
    pub power: Uint128,
    pub height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalPowerAtHeightResponse {
    pub power: Uint128,
    pub height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StakedValueResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TotalValueResponse {
    pub total: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetConfigResponse {
    pub admin: Option<Addr>,
    pub denom: String,
    pub unstaking_duration: Option<Duration>,
}
