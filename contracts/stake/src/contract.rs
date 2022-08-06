#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Env, MessageInfo, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};

use crate::msg::{
    ClaimsResponse, Duration, ExecuteMsg, GetConfigResponse, InstantiateMsg, QueryMsg,
    StakedBalanceAtHeightResponse, StakedValueResponse, TotalStakedAtHeightResponse,
    TotalValueResponse,
};
use crate::state::{Config, BALANCE, CLAIMS, CONFIG, MAX_CLAIMS, STAKED_BALANCES, STAKED_TOTAL};
use crate::ContractError;

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
    _info: MessageInfo,
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
            let received = cw_utils::may_pay(&info, denom.as_str()).unwrap();
            execute_stake(deps, env, &info.sender, received)
        }
        ExecuteMsg::Fund {} => {
            let denom = CONFIG.load(deps.storage)?.denom;
            let received = cw_utils::may_pay(&info, denom.as_str()).unwrap();
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
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::StakedBalanceAtHeight { address, height } => {
            to_binary(&query_staked_balance_at_height(deps, env, address, height)?)
        }
        QueryMsg::TotalStakedAtHeight { height } => {
            to_binary(&query_total_staked_at_height(deps, env, height)?)
        }
        QueryMsg::StakedValue { address } => to_binary(&query_staked_value(deps, env, address)?),
        QueryMsg::TotalValue {} => to_binary(&query_total_value(deps, env)?),
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
    }
}

pub fn query_staked_balance_at_height(
    deps: Deps,
    _env: Env,
    address: String,
    height: Option<u64>,
) -> StdResult<StakedBalanceAtHeightResponse> {
    let address = deps.api.addr_validate(&address)?;
    let height = height.unwrap_or(_env.block.height);
    let balance = STAKED_BALANCES
        .may_load_at_height(deps.storage, &address, height)?
        .unwrap_or_default();
    Ok(StakedBalanceAtHeightResponse { balance, height })
}

pub fn query_total_staked_at_height(
    deps: Deps,
    _env: Env,
    height: Option<u64>,
) -> StdResult<TotalStakedAtHeightResponse> {
    let height = height.unwrap_or(_env.block.height);
    let total = STAKED_TOTAL
        .may_load_at_height(deps.storage, height)?
        .unwrap_or_default();
    Ok(TotalStakedAtHeightResponse { total, height })
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

pub fn query_config(deps: Deps) -> StdResult<GetConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(GetConfigResponse {
        admin: config.admin,
        denom: config.denom,
        unstaking_duration: config.unstaking_duration,
    })
}

pub fn query_claims(deps: Deps, address: String) -> StdResult<ClaimsResponse> {
    CLAIMS.query_claims(deps, &deps.api.addr_validate(&address)?)
}
