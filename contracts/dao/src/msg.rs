use std::fmt;

use cosmwasm_std::{Addr, Coin, CosmosMsg, Decimal, Empty, Order, Uint128};
use cw3::{Status, Vote};
use cw_utils::{Duration, Expiration};
use osmo_bindings::OsmosisMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::proposal::{BlockTime, Votes};
use crate::state::Config;
use crate::threshold::Threshold;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
    // The name of the DAO.
    pub name: String,
    // A description of the DAO.
    pub description: String,
    /// Set an existing governance token or launch a new one
    pub gov_token: GovToken,
    /// Voting params configuration
    pub threshold: Threshold,

    pub voting_period: Duration,

    pub deposit_period: Duration,

    /// Deposit required to make a proposal
    pub proposal_deposit_amount: Uint128,
    pub proposal_deposit_min_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GovToken {
    Create {
        denom: String,
        label: String,
        stake_contract_code_id: u64,
        unstaking_duration: Option<Duration>,
    },
    Reuse {
        stake_contract: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposeMsg {
    pub title: String,
    pub link: String,
    pub description: String,
    pub msgs: Vec<CosmosMsg<OsmosisMsg>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteMsg {
    pub proposal_id: u64,
    pub vote: Vote,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Makes a new proposal
    Propose(ProposeMsg),
    Deposit {
        proposal_id: u64,
    },
    ClaimDeposit {
        proposal_id: u64,
    },
    /// Vote on an open proposal
    Vote(VoteMsg),
    /// Execute a passed proposal
    Execute {
        proposal_id: u64,
    },
    /// Close a failed proposal
    Close {
        proposal_id: u64,
    },
    /// Pauses DAO governance (can only be called by DAO contract)
    PauseDAO {
        expiration: Expiration,
    },
    /// Update DAO config (can only be called by DAO contract)
    UpdateConfig(Config),
    /// Updates token list
    UpdateTokenList {
        to_add: Vec<String>,
        to_remove: Vec<String>,
    },
    /// Update Staking Contract (can only be called by DAO contract)
    /// WARNING: this changes the contract controlling voting
    UpdateStakingContract {
        new_staking_contract: Addr,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ProposalsQueryOption {
    FindByStatus { status: Status },
    FindByProposer { proposer: Addr },
    Everything {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DepositsQueryOption {
    FindByProposal {
        proposal_id: u64,
        start: Option<String>,
    },
    FindByDepositor {
        depositor: String,
        start: Option<u64>,
    },
    Everything {
        start: Option<(u64, String)>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// # GetConfig
    ///
    /// Returns [ConfigResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///    "get_config": {}
    /// }
    /// ```
    GetConfig {},

    /// # TokenList
    ///
    /// Queries list of native Tokens associated with the DAO Treasury.
    /// Returns [TokenListResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "token_list": {}
    /// }
    /// ```
    TokenList {},

    /// # TokenBalances
    ///
    /// Returns [TokenBalancesResponse]
    /// All DAO native token Balances
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "token_balances": {
    ///     "start"?: "uosmo",
    ///     "limit": 30 | 10,
    ///     "order": "asc" | "desc"
    ///   }
    /// }
    /// ```
    TokenBalances {
        start: Option<String>,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    },

    /// # Proposal
    ///
    /// Returns [ProposalResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "proposal": {
    ///     "proposal_id": 1
    ///   }
    /// }
    /// ```
    Proposal { proposal_id: u64 },

    /// # Proposals
    ///
    /// Returns [ProposalsResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "proposals": {
    ///     "query": {
    ///       "find_by_status": { "status": "pending" | .. | "executed" }
    ///         | "find_by_proposer": { "proposer": "osmo1deadbeef" }
    ///         | "everything": {}
    ///     },
    ///     "start"?: 10,
    ///     "limit": 30 | 10,
    ///     "order": "asc" | "desc"
    ///   }
    /// }
    /// ```
    Proposals {
        query: ProposalsQueryOption,
        start: Option<u64>,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    },

    /// # ProposalCount
    ///
    /// Returns the number of proposals in the DAO (u64)
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "proposal_count": {}
    /// }
    /// ```
    ProposalCount {},

    /// # Vote
    ///
    /// Returns [VoteResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "vote": {
    ///     "proposal_id": 1,
    ///     "voter": "osmo1deadbeef"
    ///   }
    /// }
    /// ```
    Vote { proposal_id: u64, voter: String },

    /// # Votes
    ///
    /// Returns [VotesResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "votes": {
    ///     "proposal_id": 1,
    ///     "start"?: "osmo1deadbeef",
    ///     "limit": 30 | 10,
    ///     "order": "asc" | "desc"
    ///   }
    /// }
    /// ```
    Votes {
        proposal_id: u64,
        start: Option<String>,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    },

    /// # Deposit
    ///
    /// Queries single deposit info by proposal id & address of depositor.  
    /// Returns [DepositResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "deposit": {
    ///     "proposal_id": 1,
    ///     "depositor": "osmo1deadbeef"
    ///   }
    /// }
    /// ```
    Deposit { proposal_id: u64, depositor: String },

    /// # Deposits
    ///
    /// Queries multiple deposits info by
    /// 1. proposal id
    /// 2. depositor address
    /// Returns [DepositsResponse]
    ///
    /// ## Example
    ///
    /// ```json
    /// {
    ///   "deposits": {
    ///     "query": {
    ///       "find_by_proposal": {
    ///         "proposal_id": 1,
    ///         "start"?: "osmo1deadbeef"
    ///       } |
    ///       "find_by_depositor": {
    ///         "depositor": "osmo1deadbeef",
    ///         "start"?: 1
    ///       } |
    ///       "everything": {
    ///         "start"?: [1, "osmo1deadbeef"]
    ///       }
    ///     },
    ///     "limit": 30 | 10,
    ///     "order": "asc" | "desc"
    ///   }
    /// }
    /// ```
    Deposits {
        query: DepositsQueryOption,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ConfigResponse {
    pub config: Config,
    pub gov_token: String,
    pub staking_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenListResponse {
    pub token_list: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenBalancesResponse {
    pub balances: Vec<Coin>,
}

/// Note, if you are storing custom messages in the proposal,
/// the querier needs to know what possible custom message types
/// those are in order to parse the response
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProposalResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub id: u64,

    // payload
    pub title: String,
    pub link: String,
    pub description: String,
    pub proposer: Addr,
    pub msgs: Vec<CosmosMsg<T>>,
    pub status: Status,

    // time
    pub submitted_at: BlockTime,
    pub deposit_ends_at: Expiration,
    pub vote_starts_at: BlockTime,
    pub vote_ends_at: Expiration,

    // vote
    pub votes: Votes,
    pub quorum: Decimal,
    pub threshold: Threshold,
    pub total_votes: Uint128,
    pub total_weight: Uint128,
    pub total_deposit: Uint128,

    pub deposit_claimable: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProposalsResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub proposals: Vec<ProposalResponse<T>>,
}

/// Returns the vote (opinion as well as weight counted) as well as
/// the address of the voter who submitted it
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VoteInfo {
    pub voter: String,
    pub vote: Vote,
    pub weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VoteResponse {
    pub vote: Option<VoteInfo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VotesResponse {
    pub votes: Vec<VoteInfo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DepositResponse {
    pub proposal_id: u64,
    pub depositor: String,
    pub amount: Uint128,
    pub claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct DepositsResponse {
    pub deposits: Vec<DepositResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MigrateMsg {}

#[cfg(test)]
mod tests {
    use cosmwasm_std::to_vec;

    use super::*;

    #[test]
    fn vote_encoding() {
        let a = Vote::Yes;
        let encoded = to_vec(&a).unwrap();
        let json = String::from_utf8_lossy(&encoded).to_string();
        assert_eq!(r#""yes""#, json.as_str());
    }

    #[test]
    fn vote_encoding_embedded() {
        let msg = ExecuteMsg::Vote(VoteMsg {
            proposal_id: 17,
            vote: Vote::No,
        });
        let encoded = to_vec(&msg).unwrap();
        let json = String::from_utf8_lossy(&encoded).to_string();
        assert_eq!(r#"{"vote":{"proposal_id":17,"vote":"no"}}"#, json.as_str());
    }
}
