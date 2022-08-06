use std::convert::TryInto;

use crate::ContractError;
use cosmwasm_std::{Addr, Empty, StdError, StdResult, Storage, Uint128};
use cw3::Vote;
use cw_storage_plus::{Item, Map};
use cw_utils::{Duration, Expiration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use crate::proposal::{BlockTime, Proposal, Votes};
pub use crate::threshold::Threshold;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub threshold: Threshold,
    pub voting_period: Duration,
    pub deposit_period: Duration,
    pub proposal_deposit: Uint128,
    pub proposal_min_deposit: Uint128,
}

impl Config {
    pub fn validate(&self) -> Result<(), ContractError> {
        match (self.voting_period, self.deposit_period) {
            (Duration::Height(voting_period_height), Duration::Height(deposit_period_height)) => {
                if voting_period_height < deposit_period_height {
                    Err(ContractError::InvalidPeriod {})
                } else {
                    Ok(())
                }
            }
            (Duration::Time(voting_period_time), Duration::Time(deposit_period_time)) => {
                if voting_period_time < deposit_period_time {
                    Err(ContractError::InvalidPeriod {})
                } else {
                    Ok(())
                }
            }
            _ => Err(ContractError::InvalidPeriod {}),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Deposit {
    pub amount: Uint128,
    pub claimed: bool,
}

// we cast a ballot with our chosen vote and a given weight
// stored under the key that voted
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Ballot {
    pub weight: Uint128,
    pub vote: Vote,
}

// Unique items
pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");
pub const DAO_PAUSED: Item<Expiration> = Item::new("dao_paused");

// Total weight and voters are queried from this contract
pub const STAKING_CONTRACT: Item<Addr> = Item::new("staking_contract");

// Address of the token used for staking
pub const GOV_TOKEN: Item<String> = Item::new("gov_token");

// Stores staking contract CODE ID and Unbonding time for use in a reply
pub const STAKING_CONTRACT_CODE_ID: Item<u64> = Item::new("staking_contract_code_id");
pub const STAKING_CONTRACT_UNSTAKING_DURATION: Item<Option<Duration>> =
    Item::new("staking_contract_unstaking_duration");

// Multiple-item map
pub const BALLOTS: Map<(u64, &Addr), Ballot> = Map::new("votes"); // proposal_id => user_address => Ballot
pub const DEPOSITS: Map<(u64, &Addr), Deposit> = Map::new("deposits");
pub const IDX_DEPOSITS_BY_DEPOSITOR: Map<(&Addr, u64), Empty> =
    Map::new("idx_deposits_by_depositor");
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
pub const IDX_PROPS_BY_STATUS: Map<(u8, u64), Empty> = Map::new("idx_props_by_status");
pub const IDX_PROPS_BY_PROPOSER: Map<(&Addr, u64), Empty> = Map::new("idx_props_by_proposer");
pub const TREASURY_TOKENS: Map<&str, Empty> = Map::new("treasury_tokens"); // token_denom => Empty

pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = PROPOSAL_COUNT.may_load(store)?.unwrap_or_default() + 1;
    PROPOSAL_COUNT.save(store, &id)?;
    Ok(id)
}

pub fn parse_id(data: &[u8]) -> StdResult<u64> {
    match data[0..8].try_into() {
        Ok(bytes) => Ok(u64::from_be_bytes(bytes)),
        Err(_) => Err(StdError::generic_err(
            "Corrupted data found. 8 byte expected.",
        )),
    }
}
