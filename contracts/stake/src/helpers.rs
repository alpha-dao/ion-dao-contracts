use crate::ContractError;
use cosmwasm_std::{StdError, StdResult, Uint128};

pub fn calc_staked_value(balance: Uint128, staked: Uint128, total: Uint128) -> StdResult<Uint128> {
    if balance == Uint128::zero() || staked == Uint128::zero() || total == Uint128::zero() {
        Ok(Uint128::zero())
    } else {
        Ok(staked
            .checked_mul(balance)
            .map_err(StdError::overflow)?
            .checked_div(total)
            .map_err(StdError::divide_by_zero)?)
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
