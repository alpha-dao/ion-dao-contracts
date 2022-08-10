use cosmwasm_std::{Addr, BlockInfo, Decimal, QuerierWrapper, StdError, StdResult, Uint128};
use cw_utils::{Duration, Expiration};
use osmo_bindings::{OsmosisMsg, OsmosisQuery};

use crate::msg::ProposalResponse;
use crate::state::{BlockTime, Proposal, STAKING_CONTRACT};
use crate::ContractError;

/// type aliases
pub type Response = cosmwasm_std::Response<OsmosisMsg>;
pub type SubMsg = cosmwasm_std::SubMsg<OsmosisMsg>;
pub type Deps<'a> = cosmwasm_std::Deps<'a, OsmosisQuery>;
pub type DepsMut<'a> = cosmwasm_std::DepsMut<'a, OsmosisQuery>;

pub fn duration_to_expiry(block: &BlockTime, period: &Duration) -> Expiration {
    match period {
        Duration::Height(height) => Expiration::AtHeight(block.height + height),
        Duration::Time(time) => Expiration::AtTime(block.time.plus_seconds(*time)),
    }
}

pub fn get_total_staked_supply(deps: Deps) -> StdResult<Uint128> {
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;

    // Get total supply
    let total: ion_stake::msg::TotalPowerAtHeightResponse = deps.querier.query_wasm_smart(
        staking_contract,
        &ion_stake::msg::QueryMsg::TotalPowerAtHeight { height: None },
    )?;
    Ok(total.power)
}

pub fn get_staked_balance(deps: Deps, address: Addr) -> StdResult<Uint128> {
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;

    // Get current staked balance
    let res: ion_stake::msg::VotingPowerAtHeightResponse = deps.querier.query_wasm_smart(
        staking_contract,
        &ion_stake::msg::QueryMsg::VotingPowerAtHeight {
            address: address.to_string(),
            height: None,
        },
    )?;
    Ok(res.power)
}

pub fn get_config(deps: Deps) -> StdResult<ion_stake::msg::GetConfigResponse> {
    let staking_contract = STAKING_CONTRACT.load(deps.storage)?;

    let res: ion_stake::msg::GetConfigResponse = deps
        .querier
        .query_wasm_smart(staking_contract, &ion_stake::msg::QueryMsg::GetConfig {})?;

    Ok(res)
}

pub fn get_voting_power_at_height(
    querier: QuerierWrapper<OsmosisQuery>,
    staking_contract: Addr,
    address: Addr,
    height: u64,
) -> StdResult<Uint128> {
    // Get voting power at height
    let balance: ion_stake::msg::VotingPowerAtHeightResponse = querier.query_wasm_smart(
        staking_contract,
        &ion_stake::msg::QueryMsg::VotingPowerAtHeight {
            address: address.into_string(),
            height: Some(height),
        },
    )?;
    Ok(balance.power)
}

pub fn proposal_to_response(
    block: &BlockInfo,
    id: u64,
    prop: Proposal,
) -> ProposalResponse<OsmosisMsg> {
    let status = prop.current_status(block);
    let total_weight = prop.total_weight;
    let total_votes = prop.votes.total();
    let quorum = if total_weight.is_zero() {
        Decimal::zero()
    } else {
        Decimal::from_ratio(total_votes, total_weight)
    };

    ProposalResponse {
        id,

        title: prop.title,
        link: prop.link,
        description: prop.description,
        proposer: prop.proposer,
        msgs: prop.msgs,
        status,

        submitted_at: prop.submitted_at,
        deposit_ends_at: prop.deposit_ends_at,
        vote_starts_at: prop.vote_starts_at,
        vote_ends_at: prop.vote_ends_at,

        votes: prop.votes,
        quorum,
        threshold: prop.threshold,
        total_votes,
        total_weight,
        total_deposit: prop.total_deposit,

        deposit_claimable: prop.deposit_claimable,
    }
}

pub fn get_and_check_limit(limit: Option<u32>, max: u32, default: u32) -> StdResult<u32> {
    match limit {
        Some(l) => {
            if l <= max {
                Ok(l)
            } else {
                Err(StdError::generic_err(
                    ContractError::OversizedRequest {
                        size: l as u64,
                        max: max as u64,
                    }
                    .to_string(),
                ))
            }
        }
        None => Ok(default),
    }
}
