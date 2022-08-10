#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Empty, Env, MessageInfo, Reply, StdResult, WasmMsg};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::helpers::get_config;
use crate::msg::{ExecuteMsg, GovToken, InstantiateMsg, MigrateMsg, QueryMsg, VoteMsg};
use crate::state::{Config, CONFIG, GOV_TOKEN, PROPOSAL_COUNT, STAKING_CONTRACT, TREASURY_TOKENS};
use crate::{Deps, DepsMut, Response, SubMsg};

// Version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:ion-dao";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Reply IDs
const INSTANTIATE_STAKING_CONTRACT_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.threshold.validate()?;

    let cfg = Config {
        name: msg.name,
        description: msg.description,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        deposit_period: msg.deposit_period,
        proposal_deposit: msg.proposal_deposit_amount,
        proposal_min_deposit: msg.proposal_deposit_min_amount,
    };
    cfg.validate()?;

    CONFIG.save(deps.storage, &cfg)?;
    PROPOSAL_COUNT.save(deps.storage, &0)?;

    match msg.gov_token {
        GovToken::Create {
            denom,
            label,
            stake_contract_code_id,
            unstaking_duration,
        } => {
            // Add native token to map of TREASURY TOKENS
            TREASURY_TOKENS.save(deps.storage, &denom, &Empty {})?;

            // Save gov token
            GOV_TOKEN.save(deps.storage, &denom)?;

            // Instantiate staking contract with DAO as admin
            Ok(Response::new().add_submessage(SubMsg::reply_on_success(
                WasmMsg::Instantiate {
                    code_id: stake_contract_code_id,
                    funds: vec![],
                    admin: Some(env.contract.address.to_string()),
                    label,
                    msg: to_binary(&ion_stake::msg::InstantiateMsg {
                        admin: Some(env.contract.address),
                        denom,
                        unstaking_duration,
                    })?,
                },
                INSTANTIATE_STAKING_CONTRACT_REPLY_ID,
            )))
        }

        GovToken::Reuse { stake_contract } => {
            let addr = deps.api.addr_validate(stake_contract.as_str())?;
            STAKING_CONTRACT.save(deps.storage, &addr)?;

            let staking_config = get_config(deps.as_ref())?;
            // Add native token to map of TREASURY TOKENS
            TREASURY_TOKENS.save(deps.storage, &staking_config.denom, &Empty {})?;

            // Save gov token
            GOV_TOKEN.save(deps.storage, &staking_config.denom)?;

            Ok(Response::new())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use crate::execute;
    use crate::msg::ExecuteMsg::*;

    match msg {
        Propose(propose_msg) => execute::propose(deps, env, info, propose_msg),
        Deposit { proposal_id } => execute::deposit(deps, env, info, proposal_id),
        ClaimDeposit { proposal_id } => execute::claim_deposit(deps, env, info, proposal_id),
        Vote(VoteMsg { proposal_id, vote }) => execute::vote(deps, env, info, proposal_id, vote),
        Execute { proposal_id } => execute::execute(deps, env, info, proposal_id),
        Close { proposal_id } => execute::close(deps, env, info, proposal_id),
        PauseDAO { expiration } => execute::pause_dao(deps, env, info, expiration),
        UpdateConfig(config) => execute::update_config(deps, env, info, config),
        UpdateTokenList { to_add, to_remove } => {
            execute::update_token_list(deps, env, info, to_add, to_remove)
        }
        UpdateStakingContract {
            new_staking_contract,
        } => execute::update_staking_contract(deps, env, info, new_staking_contract),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use crate::msg::QueryMsg::*;
    use crate::query;

    match msg {
        GetConfig {} => to_binary(&query::config(deps)?),
        TokenList {} => to_binary(&query::token_list(deps)?),
        TokenBalances {
            start,
            limit,
            order,
        } => to_binary(&query::token_balances(deps, env, start, limit, order)?),

        Proposal { proposal_id } => to_binary(&query::proposal(deps, env, proposal_id)?),
        Proposals {
            query,
            start,
            limit,
            order,
        } => to_binary(&query::proposals(deps, env, query, start, limit, order)?),
        ProposalCount {} => to_binary(&query::proposal_count(deps)?),

        Vote { proposal_id, voter } => to_binary(&query::vote(deps, proposal_id, voter)?),
        Votes {
            proposal_id,
            start,
            limit,
            order,
        } => to_binary(&query::votes(deps, proposal_id, start, limit, order)?),

        Deposit {
            proposal_id,
            depositor,
        } => to_binary(&query::deposit(deps, proposal_id, depositor)?),
        Deposits {
            query,
            limit,
            order,
        } => to_binary(&query::deposits(deps, query, limit, order)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_STAKING_CONTRACT_REPLY_ID => {
            let res = parse_reply_instantiate_data(msg);
            match res {
                Ok(res) => {
                    // Validate contract address
                    let staking_contract_addr = deps.api.addr_validate(&res.contract_address)?;

                    // Save gov token
                    STAKING_CONTRACT.save(deps.storage, &staking_contract_addr)?;

                    Ok(Response::new())
                }
                Err(_) => Err(ContractError::InstantiateGovTokenError {}),
            }
        }
        _ => Err(ContractError::UnknownReplyId { id: msg.id }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // No state migrations performed, just returned a Response
    Ok(Response::default())
}
