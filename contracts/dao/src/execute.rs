use std::ops::Add;

use cosmwasm_std::{
    coins, Addr, BankMsg, BlockInfo, Empty, Env, MessageInfo, StdError, StdResult, Storage, Uint128,
};
use cw3::{Status, Vote};
use cw_utils::{may_pay, Expiration};

use crate::helpers::{duration_to_expiry, get_total_staked_supply, get_voting_power_at_height};
use crate::msg::ProposeMsg;
use crate::state::{
    next_id, Ballot, Config, Proposal, Votes, BALLOTS, CONFIG, DAO_PAUSED, DEPOSITS, GOV_TOKEN,
    IDX_DEPOSITS_BY_DEPOSITOR, IDX_PROPS_BY_PROPOSER, IDX_PROPS_BY_STATUS, PROPOSALS,
    STAKING_CONTRACT, TREASURY_TOKENS,
};
use crate::ContractError;

use super::{DepsMut, Response, MAX_LIMIT};

fn check_paused(storage: &dyn Storage, block: &BlockInfo) -> Result<(), ContractError> {
    let paused = DAO_PAUSED.may_load(storage)?;
    if let Some(expiration) = paused {
        if !expiration.is_expired(block) {
            return Err(ContractError::Paused {});
        }
    }

    Ok(())
}

fn check_status(origin_status: &Status, desired_status: Status) -> Result<(), ContractError> {
    if !origin_status.eq(&desired_status) {
        return Err(ContractError::InvalidProposalStatus {
            current: format!("{:?}", origin_status),
            desired: format!("{:?}", desired_status),
        });
    }

    Ok(())
}

fn create_proposal(
    storage: &mut dyn Storage,
    prop_id: u64,
    proposer: &Addr,
    proposal: &Proposal,
) -> StdResult<()> {
    PROPOSALS.save(storage, prop_id, proposal)?;
    IDX_PROPS_BY_STATUS.save(storage, (proposal.status as u8, prop_id), &Empty {})?;
    IDX_PROPS_BY_PROPOSER.save(storage, (proposer.clone(), prop_id), &Empty {})?;

    Ok(())
}

fn create_deposit(
    storage: &mut dyn Storage,
    prop_id: u64,
    depositor: &Addr,
    amount: &Uint128,
) -> StdResult<()> {
    // deposit
    let mut deposit = DEPOSITS
        .may_load(storage, (prop_id, depositor.clone()))?
        .unwrap_or_default();
    if deposit.amount.is_zero() {
        IDX_DEPOSITS_BY_DEPOSITOR.save(storage, (depositor.clone(), prop_id), &Empty {})?;
    }

    deposit.amount = deposit.amount.checked_add(*amount)?;

    DEPOSITS.save(storage, (prop_id, depositor.clone()), &deposit)?;

    Ok(())
}

fn make_deposit_claimable(
    storage: &mut dyn Storage,
    prop_id: u64,
    proposal: &mut Proposal,
) -> StdResult<()> {
    PROPOSALS.update(storage, prop_id, |v| -> StdResult<Proposal> {
        let mut v = v.unwrap();
        v.deposit_claimable = true;
        Ok(v)
    })?;
    proposal.deposit_claimable = true;

    Ok(())
}

fn update_proposal_status(
    storage: &mut dyn Storage,
    prop_id: u64,
    proposal: &mut Proposal,
    desired: Status,
) -> StdResult<()> {
    let before = proposal.status;
    proposal.status = desired;
    PROPOSALS.update(storage, prop_id, |prop| {
        if let Some(mut prop) = prop {
            prop.status = desired;
            Ok(prop)
        } else {
            Err(StdError::not_found("proposal"))
        }
    })?;
    IDX_PROPS_BY_STATUS.remove(storage, (before as u8, prop_id));
    IDX_PROPS_BY_STATUS.save(storage, (desired as u8, prop_id), &Empty {})?;

    Ok(())
}

pub fn propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    propose_msg: ProposeMsg,
) -> Result<Response, ContractError> {
    check_paused(deps.storage, &env.block)?;

    let cfg = CONFIG.load(deps.storage)?;
    let gov_token = GOV_TOKEN.load(deps.storage)?;

    let received = may_pay(&info, gov_token.as_str())?;
    if received < cfg.proposal_min_deposit {
        return Err(ContractError::Unauthorized {});
    }

    // Get total supply
    let total_supply = get_total_staked_supply(deps.as_ref())?;
    if total_supply.is_zero() {
        return Err(ContractError::LackOfStakes {});
    }

    // Create a proposal
    let mut prop = Proposal {
        // payload
        title: propose_msg.title,
        link: propose_msg.link,
        description: propose_msg.description,
        proposer: info.sender.clone(),
        msgs: propose_msg.msgs,
        status: Status::Pending,

        // time
        submitted_at: env.block.clone().into(),
        deposit_ends_at: duration_to_expiry(&env.block.clone().into(), &cfg.deposit_period),
        vote_starts_at: Default::default(),
        vote_ends_at: duration_to_expiry(
            &env.block.clone().into(),
            &cfg.deposit_period.add(cfg.voting_period)?,
        ), // set it to maximum

        // voting
        votes: Votes::default(),
        threshold: cfg.threshold,
        total_weight: total_supply,
        total_deposit: received, // initial deposit = received
        deposit_base_amount: cfg.proposal_deposit,
        deposit_claimable: false,
    };

    let mut resp = Response::new();
    if received >= cfg.proposal_deposit {
        prop.activate_voting_period(env.block.into(), &cfg.voting_period);

        // refund exceeded amount
        let gap = received - cfg.proposal_deposit;
        if gap > Uint128::zero() {
            resp = resp.add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(gap.u128(), gov_token),
            });
        }
    }

    let id = next_id(deps.storage)?;
    create_deposit(deps.storage, id, &info.sender, &received)?;
    create_proposal(deps.storage, id, &info.sender, &prop)?;

    Ok(resp
        .add_attribute("action", "propose")
        .add_attribute("sender", info.sender)
        .add_attribute("status", format!("{:?}", prop.status))
        .add_attribute("deposit", received.to_string())
        .add_attribute("proposal_id", id.to_string()))
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    prop_id: u64,
) -> Result<Response, ContractError> {
    check_paused(deps.storage, &env.block)?;

    let cfg = CONFIG.load(deps.storage)?;
    let gov_token = GOV_TOKEN.load(deps.storage)?;

    let received = may_pay(&info, gov_token.as_str())?;
    if received.is_zero() {
        return Err(ContractError::Unauthorized {});
    }

    let mut resp = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("denom", gov_token.to_string())
        .add_attribute("amount", received.to_string())
        .add_attribute("proposal_id", prop_id.to_string());

    let mut prop = PROPOSALS.load(deps.storage, prop_id)?;
    check_status(&prop.status, Status::Pending)?;
    if prop.deposit_ends_at.is_expired(&env.block) {
        Err(ContractError::Expired {})
    } else {
        create_deposit(deps.storage, prop_id, &info.sender, &received)?;

        prop.total_deposit += received;
        if prop.total_deposit >= cfg.proposal_deposit {
            // open
            update_proposal_status(deps.storage, prop_id, &mut prop, Status::Open)?;
            prop.activate_voting_period(env.block.into(), &cfg.voting_period);
            PROPOSALS.save(deps.storage, prop_id, &prop)?;

            // refund exceeded amount
            let gap = prop.total_deposit - cfg.proposal_deposit;
            if gap > Uint128::zero() {
                resp = resp.add_message(BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: coins(gap.u128(), gov_token),
                });
            }

            Ok(resp.add_attribute("result", "open"))
        } else {
            // pending = prevent default
            PROPOSALS.save(deps.storage, prop_id, &prop)?;
            Ok(resp.add_attribute("result", "pending"))
        }
    }
}

pub fn claim_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    prop_id: u64,
) -> Result<Response, ContractError> {
    check_paused(deps.storage, &env.block)?;

    let prop = PROPOSALS.load(deps.storage, prop_id)?;
    if !prop.deposit_claimable {
        return Err(ContractError::DepositNotClaimable {});
    }

    let mut deposit = DEPOSITS.load(deps.storage, (prop_id, info.sender.clone()))?;
    if deposit.claimed {
        return Err(ContractError::DepositAlreadyClaimed {});
    }
    deposit.claimed = true;

    DEPOSITS.save(deps.storage, (prop_id, info.sender.clone()), &deposit)?;

    let gov_token = GOV_TOKEN.load(deps.storage)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(deposit.amount.u128(), gov_token),
        })
        .add_attribute("action", "claim_deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("proposal_id", prop_id.to_string())
        .add_attribute("amount", deposit.amount))
}

pub fn vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    prop_id: u64,
    vote: Vote,
) -> Result<Response, ContractError> {
    check_paused(deps.storage, &env.block)?;

    // Ensure proposal exists and can be voted on
    let mut prop = PROPOSALS.load(deps.storage, prop_id)?;
    check_status(&prop.status, Status::Open)?;
    if prop.vote_ends_at.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // Get voter balance at proposal start
    let vote_power = get_voting_power_at_height(
        deps.querier,
        STAKING_CONTRACT.load(deps.storage)?,
        info.sender.clone(),
        prop.vote_starts_at.height,
    )?;
    if vote_power.is_zero() {
        return Err(ContractError::Unauthorized {});
    }

    let ballot = BALLOTS.may_load(deps.storage, (prop_id, &info.sender))?;
    if let Some(ballot) = ballot {
        prop.votes.revoke(ballot.vote, ballot.weight);
    }
    prop.votes.submit(vote, vote_power);

    BALLOTS.save(
        deps.storage,
        (prop_id, &info.sender),
        &Ballot {
            weight: vote_power,
            vote,
        },
    )?;
    PROPOSALS.save(deps.storage, prop_id, &prop)?;

    Ok(Response::new()
        .add_attribute("action", "vote")
        .add_attribute("sender", info.sender)
        .add_attribute("vote", format!("{:?}", vote))
        .add_attribute("proposal_id", prop_id.to_string()))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    prop_id: u64,
) -> Result<Response, ContractError> {
    check_paused(deps.storage, &env.block)?;

    let mut prop = PROPOSALS.load(deps.storage, prop_id)?;
    if !prop.vote_ends_at.is_expired(&env.block) {
        return Err(ContractError::NotExpired {});
    }

    check_status(&prop.current_status(&env.block), Status::Passed)?;
    update_proposal_status(deps.storage, prop_id, &mut prop, Status::Executed)?;
    make_deposit_claimable(deps.storage, prop_id, &mut prop)?;
    prop.update_status(&env.block);

    // Dispatch all proposed messages
    Ok(Response::new()
        .add_messages(prop.msgs)
        .add_attribute("action", "execute")
        .add_attribute("sender", info.sender)
        .add_attribute("proposal_id", prop_id.to_string()))
}

pub fn close(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    prop_id: u64,
) -> Result<Response, ContractError> {
    check_paused(deps.storage, &env.block)?;

    let mut prop = PROPOSALS.load(deps.storage, prop_id)?;

    match prop.status {
        // * failed to satisfy minimum deposit amount -> confiscate
        Status::Pending => {
            if !prop.deposit_ends_at.is_expired(&env.block) {
                return Err(ContractError::NotExpired {});
            }
        }
        // * failed to pass vote threshold -> refund
        // * passed veto threshold -> confiscate
        Status::Open => {
            if !prop.vote_ends_at.is_expired(&env.block) {
                return Err(ContractError::NotExpired {});
            }
        }
        _ => {
            return Err(ContractError::InvalidProposalStatus {
                current: format!("{:?}", prop.status),
                desired: "pending | open".to_string(),
            })
        }
    }

    let prev_status = prop.status;
    check_status(&prop.current_status(&env.block), Status::Rejected)?;
    update_proposal_status(deps.storage, prop_id, &mut prop, Status::Rejected)?;
    prop.update_status(&env.block);

    let mut resp = Response::new()
        .add_attribute("action", "close")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("proposal_id", prop_id.to_string());

    if prev_status == Status::Open && !prop.is_vetoed() {
        make_deposit_claimable(deps.storage, prop_id, &mut prop)?;
        resp = resp.add_attribute("result", "refund");
    } else {
        resp = resp.add_attribute("result", "confiscate")
    }

    Ok(resp)
}

pub fn pause_dao(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    expiration: Expiration,
) -> Result<Response, ContractError> {
    // Only contract can call this method
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    DAO_PAUSED.save(deps.storage, &expiration)?;

    Ok(Response::new()
        .add_attribute("action", "pause_dao")
        .add_attribute("expiration", expiration.to_string()))
}

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    update_config_msg: Config,
) -> Result<Response, ContractError> {
    // Only contract can call this method
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    update_config_msg.threshold.validate()?;

    CONFIG.save(deps.storage, &update_config_msg)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("sender", info.sender))
}

pub fn update_staking_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_staking_contract: Addr,
) -> Result<Response, ContractError> {
    // Only contract can call this method
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let new_staking_contract = deps.api.addr_validate(new_staking_contract.as_str())?;

    // Replace the existing staking contract
    STAKING_CONTRACT.save(deps.storage, &new_staking_contract)?;

    Ok(Response::new()
        .add_attribute("action", "update_staking_contract")
        .add_attribute("new_staking_contract", new_staking_contract))
}

pub fn update_token_list(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_add: Vec<String>,
    to_remove: Vec<String>,
) -> Result<Response, ContractError> {
    // Only contract can call this method
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Limit the number of token modifications that can occur in one
    // execution to prevent out of gas issues.
    if to_add.len() + to_remove.len() > MAX_LIMIT as usize {
        return Err(ContractError::OversizedRequest {
            size: (to_add.len() + to_remove.len()) as u64,
            max: MAX_LIMIT as u64,
        });
    }

    for denom in &to_add {
        TREASURY_TOKENS.save(deps.storage, &denom, &Empty {})?;
    }

    for denom in &to_remove {
        TREASURY_TOKENS.remove(deps.storage, &denom);
    }

    Ok(Response::new().add_attribute("action", "update_cw20_token_list"))
}

#[cfg(test)]
mod test {
    use crate::state::Deposit;
    use cosmwasm_std::testing::MockStorage;

    use super::*;

    #[test]
    fn check_paused() {
        let mut storage = MockStorage::new();

        DAO_PAUSED
            .save(&mut storage, &Expiration::AtHeight(10))
            .unwrap();

        super::check_paused(
            &storage,
            &BlockInfo {
                height: 11,
                time: Default::default(),
                chain_id: "mock_chain".to_string(),
            },
        )
        .unwrap();

        let err = super::check_paused(
            &storage,
            &BlockInfo {
                height: 9,
                time: Default::default(),
                chain_id: "mock_chain".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Paused {})
    }

    #[test]
    fn check_proposal_status() {
        let make_prop = |status: Status| Proposal {
            status,
            ..Default::default()
        };

        super::check_status(&make_prop(Status::Pending).status, Status::Pending).unwrap();

        let err =
            super::check_status(&make_prop(Status::Open).status, Status::Pending).unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidProposalStatus {
                current: "Open".to_string(),
                desired: "Pending".to_string()
            }
        )
    }

    #[test]
    fn create_proposal() {
        let mut storage = MockStorage::default();

        let proposer = Addr::unchecked("proposer");
        let proposal = Proposal::default();

        super::create_proposal(&mut storage, 1, &proposer, &proposal).unwrap();

        assert!(PROPOSALS.has(&storage, 1));
        assert!(IDX_PROPS_BY_STATUS.has(&storage, (Status::Pending as u8, 1)));
        assert!(IDX_PROPS_BY_PROPOSER.has(&storage, (proposer.clone(), 1)));
    }

    #[test]
    fn create_deposit() {
        let mut storage = MockStorage::default();

        let proposer = Addr::unchecked("proposer");
        let proposal = Proposal::default();

        super::create_proposal(&mut storage, 1, &proposer, &proposal).unwrap();

        let depositor = Addr::unchecked("depositor");

        // initial
        super::create_deposit(&mut storage, 1, &depositor, &Uint128::from(10u128)).unwrap();
        assert_eq!(
            DEPOSITS.load(&storage, (1, depositor.clone())).unwrap(),
            Deposit {
                amount: Uint128::from(10u128),
                claimed: false
            },
        );
        assert!(IDX_DEPOSITS_BY_DEPOSITOR.has(&storage, (depositor.clone(), 1)));

        super::create_deposit(&mut storage, 1, &depositor, &Uint128::from(10u128)).unwrap();
        assert_eq!(
            DEPOSITS.load(&storage, (1, depositor.clone())).unwrap(),
            Deposit {
                amount: Uint128::from(20u128),
                claimed: false
            },
        );
        assert!(IDX_DEPOSITS_BY_DEPOSITOR.has(&storage, (depositor.clone(), 1)));
    }

    #[test]
    fn make_deposit_claimable() {
        let mut storage = MockStorage::default();

        let proposer = Addr::unchecked("proposer");
        let mut proposal = Proposal {
            proposer: proposer.clone(),
            ..Default::default()
        };

        super::create_proposal(&mut storage, 1, &proposer, &proposal).unwrap();

        assert!(!PROPOSALS.load(&storage, 1).unwrap().deposit_claimable);

        super::make_deposit_claimable(&mut storage, 1, &mut proposal).unwrap();

        assert!(PROPOSALS.load(&storage, 1).unwrap().deposit_claimable);
    }

    #[test]
    fn update_proposal_status() {
        let mut storage = MockStorage::default();

        let proposer = Addr::unchecked("proposer");
        let mut proposal = Proposal {
            proposer: proposer.clone(),
            ..Default::default()
        };

        super::create_proposal(&mut storage, 1, &proposer, &proposal).unwrap();

        proposal.proposer = Addr::unchecked("abuser");
        super::update_proposal_status(&mut storage, 1, &mut proposal, Status::Passed).unwrap();

        assert_eq!(PROPOSALS.load(&storage, 1).unwrap().status, Status::Passed);
        assert_eq!(PROPOSALS.load(&storage, 1).unwrap().proposer, proposer);
    }
}
