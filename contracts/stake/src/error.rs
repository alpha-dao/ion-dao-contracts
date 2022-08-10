use cosmwasm_std::{Addr, OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Overflow(#[from] OverflowError),
    #[error("{0}")]
    Payment(#[from] PaymentError),
    #[error("Nothing to claim")]
    NothingToClaim {},
    #[error("Invalid token")]
    InvalidToken { received: Addr, expected: Addr },
    #[error("Unauthorized")]
    Unauthorized { received: Addr, expected: Addr },
    #[error("Too many outstanding claims. Claim some tokens before unstaking more.")]
    TooManyClaims {},
    #[error("No admin configured")]
    NoAdminConfigured {},
    #[error("Request size ({size}) is above limit of ({max})")]
    OversizedRequest { size: u64, max: u64 },
}
