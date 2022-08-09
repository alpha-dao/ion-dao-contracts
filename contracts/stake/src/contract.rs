#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Env, MessageInfo, Order, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};

use crate::helpers::get_and_check_limit;
use crate::msg::{
    ClaimsResponse, Duration, ExecuteMsg, GetConfigResponse, InstantiateMsg, QueryMsg, RangeOrder,
    StakedValueResponse, TotalPowerAtHeightResponse, TotalValueResponse,
    VotingPowerAtHeightResponse,
};
use crate::state::{
    Config, BALANCE, CLAIMS, CONFIG, DAO, MAX_CLAIMS, STAKED_BALANCES, STAKED_TOTAL,
};
use crate::{ContractError, DEFAULT_LIMIT, MAX_LIMIT};

/// type aliases
pub type Response = cosmwasm_std::Response<OsmosisMsg>;
pub type SubMsg = cosmwasm_std::SubMsg<OsmosisMsg>;
pub type CosmosMsg = cosmwasm_std::CosmosMsg<OsmosisMsg>;
pub type Deps<'a> = cosmwasm_std::Deps<'a, OsmosisQuery>;
pub type DepsMut<'a> = cosmwasm_std::DepsMut<'a, OsmosisQuery>;
pub type QuerierWrapper<'a> = cosmwasm_std::QuerierWrapper<'a, OsmosisQuery>;

const CONTRACT_NAME: &str = "crates.io:ion-stake";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = match msg.admin {
        Some(admin) => Some(deps.api.addr_validate(admin.as_str())?),
        None => None,
    };

    let config = Config {
        admin,
        denom: msg.denom,
        unstaking_duration: msg.unstaking_duration,
    };
    CONFIG.save(deps.storage, &config)?;
    DAO.save(deps.storage, &info.sender)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Stake {} => {
            let denom = CONFIG.load(deps.storage)?.denom;
            let received = cw_utils::may_pay(&info, denom.as_str())?;
            execute_stake(deps, env, &info.sender, received)
        }
        ExecuteMsg::Fund {} => {
            let denom = CONFIG.load(deps.storage)?.denom;
            let received = cw_utils::may_pay(&info, denom.as_str())?;
            execute_fund(deps, env, &info.sender, received)
        }
        ExecuteMsg::Unstake { amount } => execute_unstake(deps, env, info, amount),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::UpdateConfig { admin, duration } => {
            execute_update_config(info, deps, admin, duration)
        }
    }
}

pub fn execute_update_config(
    info: MessageInfo,
    deps: DepsMut,
    new_admin: Option<Addr>,
    duration: Option<Duration>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    match config.admin {
        None => Err(ContractError::NoAdminConfigured {}),
        Some(current_admin) => {
            if info.sender != current_admin {
                return Err(ContractError::Unauthorized {
                    expected: current_admin,
                    received: info.sender,
                });
            }

            config.admin = new_admin;
            config.unstaking_duration = duration;

            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_attribute(
                "admin",
                config
                    .admin
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "None".to_string()),
            ))
        }
    }
}

pub fn execute_stake(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let balance = BALANCE.load(deps.storage).unwrap_or_default();
    let staked_total = STAKED_TOTAL.load(deps.storage).unwrap_or_default();
    let amount_to_stake = if staked_total == Uint128::zero() || balance == Uint128::zero() {
        amount
    } else {
        staked_total
            .checked_mul(amount)
            .map_err(StdError::overflow)?
            .checked_div(balance)
            .map_err(StdError::divide_by_zero)?
    };
    STAKED_BALANCES.update(
        deps.storage,
        sender,
        env.block.height,
        |bal| -> StdResult<Uint128> { Ok(bal.unwrap_or_default().checked_add(amount_to_stake)?) },
    )?;
    STAKED_TOTAL.update(
        deps.storage,
        env.block.height,
        |total| -> StdResult<Uint128> {
            Ok(total.unwrap_or_default().checked_add(amount_to_stake)?)
        },
    )?;
    BALANCE.save(
        deps.storage,
        &balance.checked_add(amount).map_err(StdError::overflow)?,
    )?;

    Ok(Response::new()
        .add_attribute("action", "stake")
        .add_attribute("from", sender)
        .add_attribute("amount", amount))
}

pub fn execute_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let balance = BALANCE.load(deps.storage).unwrap_or_default();
    let staked_total = STAKED_TOTAL.load(deps.storage)?;
    let amount_to_claim = amount
        .checked_mul(balance)
        .map_err(StdError::overflow)?
        .checked_div(staked_total)
        .map_err(StdError::divide_by_zero)?;
    STAKED_BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |bal| -> StdResult<Uint128> { Ok(bal.unwrap_or_default().checked_sub(amount)?) },
    )?;
    STAKED_TOTAL.update(
        deps.storage,
        env.block.height,
        |total| -> StdResult<Uint128> { Ok(total.unwrap_or_default().checked_sub(amount)?) },
    )?;
    BALANCE.save(
        deps.storage,
        &balance
            .checked_sub(amount_to_claim)
            .map_err(StdError::overflow)?,
    )?;
    match config.unstaking_duration {
        None => Ok(Response::new()
            .add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(amount_to_claim.u128(), config.denom),
            })
            .add_attribute("action", "unstake")
            .add_attribute("from", info.sender)
            .add_attribute("amount", amount)
            .add_attribute("claim_duration", "None")),
        Some(duration) => {
            let outstanding_claims = CLAIMS.query_claims(deps.as_ref(), &info.sender)?.claims;
            if outstanding_claims.len() >= MAX_CLAIMS as usize {
                return Err(ContractError::TooManyClaims {});
            }

            CLAIMS.create_claim(
                deps.storage,
                &info.sender,
                amount_to_claim,
                duration.after(&env.block),
            )?;
            Ok(Response::new()
                .add_attribute("action", "unstake")
                .add_attribute("from", info.sender)
                .add_attribute("amount", amount)
                .add_attribute("claim_duration", format!("{}", duration)))
        }
    }
}

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let release = CLAIMS.claim_tokens(deps.storage, &info.sender, &_env.block, None)?;
    if release.is_zero() {
        return Err(ContractError::NothingToClaim {});
    }
    let config = CONFIG.load(deps.storage)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(release.u128(), config.denom),
        })
        .add_attribute("action", "claim")
        .add_attribute("from", info.sender)
        .add_attribute("amount", release))
}

pub fn execute_fund(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let balance = BALANCE.load(deps.storage).unwrap_or_default();
    BALANCE.save(
        deps.storage,
        &balance.checked_add(amount).map_err(StdError::overflow)?,
    )?;
    Ok(Response::new()
        .add_attribute("action", "fund")
        .add_attribute("from", sender)
        .add_attribute("amount", amount))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // DAO DAO compatability
        QueryMsg::VotingPowerAtHeight { address, height } => {
            to_binary(&query_voting_power_at_height(deps, env, address, height)?)
        }
        QueryMsg::TotalPowerAtHeight { height } => {
            to_binary(&query_total_power_at_height(deps, env, height)?)
        }
        QueryMsg::StakedValue { address } => to_binary(&query_staked_value(deps, env, address)?),
        QueryMsg::TotalValue {} => to_binary(&query_total_value(deps, env)?),
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),

        // iterators
        QueryMsg::RangeStakers {
            start_at,
            limit,
            order,
        } => to_binary(&query_range_stakers(deps, start_at, limit, order)?),

        // static
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
        QueryMsg::Dao {} => to_binary(&query_dao(deps)?),
    }
}

pub fn query_voting_power_at_height(
    deps: Deps,
    _env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<VotingPowerAtHeightResponse> {
    let address = deps.api.addr_validate(&address)?;
    let height = height.unwrap_or(_env.block.height);
    let power = STAKED_BALANCES
        .may_load_at_height(deps.storage, &address, height)?
        .unwrap_or_default();
    Ok(VotingPowerAtHeightResponse { power, height })
}

pub fn query_total_power_at_height(
    deps: Deps,
    _env: Env,
    height: Option<u64>,
) -> StdResult<TotalPowerAtHeightResponse> {
    let height = height.unwrap_or(_env.block.height);
    let power = STAKED_TOTAL
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default();
    Ok(TotalPowerAtHeightResponse { power, height })
}

pub fn query_staked_value(
    deps: Deps,
    _env: Env,
    address: String,
) -> StdResult<StakedValueResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCE.load(deps.storage).unwrap_or_default();
    let staked = STAKED_BALANCES
        .load(deps.storage, &address)
        .unwrap_or_default();
    let total = STAKED_TOTAL.load(deps.storage).unwrap_or_default();
    if balance == Uint128::zero() || staked == Uint128::zero() || total == Uint128::zero() {
        Ok(StakedValueResponse {
            value: Uint128::zero(),
        })
    } else {
        let value = staked
            .checked_mul(balance)
            .map_err(StdError::overflow)?
            .checked_div(total)
            .map_err(StdError::divide_by_zero)?;
        Ok(StakedValueResponse { value })
    }
}

pub fn query_total_value(deps: Deps, _env: Env) -> StdResult<TotalValueResponse> {
    let balance = BALANCE.load(deps.storage).unwrap_or_default();
    Ok(TotalValueResponse { total: balance })
}

pub fn query_range_stakers(
    deps: Deps,
    start: Option<String>,
    limit: Option<u32>,
    order: Option<RangeOrder>,
) -> StdResult<Vec<(Addr, StakedValueResponse)>> {
    let limit = get_and_check_limit(limit, MAX_LIMIT, DEFAULT_LIMIT)? as usize;
    let order = order.unwrap_or(RangeOrder::Asc).into();
    let start = maybe_addr(deps.api, start)?;
    let (min, max) = match order {
        Order::Ascending => (start.as_ref().map(Bound::exclusive), None),
        Order::Descending => (None, start.as_ref().map(Bound::exclusive)),
    };

    let balance = BALANCE.load(deps.storage).unwrap_or_default();
    let total = STAKED_TOTAL.load(deps.storage).unwrap_or_default();

    let resp: StdResult<Vec<(Addr, StakedValueResponse)>> = STAKED_BALANCES
        .range(deps.storage, min, max, order)
        .take(limit)
        .map(|item| -> StdResult<(Addr, StakedValueResponse)> {
            let (staker, staked) = item?;

            match balance == Uint128::zero()
                || staked == Uint128::zero()
                || total == Uint128::zero()
            {
                true => Ok((
                    staker,
                    StakedValueResponse {
                        value: Uint128::zero(),
                    },
                )),
                false => Ok((
                    staker,
                    StakedValueResponse {
                        value: staked
                            .checked_mul(balance)
                            .map_err(StdError::overflow)?
                            .checked_div(total)
                            .map_err(StdError::divide_by_zero)?,
                    },
                )),
            }
        })
        .collect();

    resp
}

pub fn query_config(deps: Deps) -> StdResult<GetConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(GetConfigResponse {
        admin: config.admin,
        denom: config.denom,
        unstaking_duration: config.unstaking_duration,
    })
}

pub fn query_info(deps: Deps) -> StdResult<cw2::ContractVersion> {
    cw2::get_contract_version(deps.storage)
}

pub fn query_dao(deps: Deps) -> StdResult<Addr> {
    DAO.load(deps.storage)
}

pub fn query_claims(deps: Deps, address: String) -> StdResult<ClaimsResponse> {
    CLAIMS.query_claims(deps, &deps.api.addr_validate(&address)?)
}
