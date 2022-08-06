use cosmwasm_std::{OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Failed to instantiate governance token")]
    InstantiateGovTokenError {},

    #[error("Initial governance token balances must not be empty")]
    InitialBalancesError {},

    #[error("Required threshold cannot be zero")]
    ZeroThreshold {},

    #[error("Not possible to reach required (passing) threshold")]
    UnreachableThreshold {},

    #[error("Invalid voting / deposit period")]
    InvalidPeriod {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Proposal is not open")]
    NotOpen {},

    #[error("Proposal voting period has expired")]
    Expired {},

    #[error("Proposal must expire before you can close it")]
    NotExpired {},

    #[error("Wrong expiration option")]
    WrongExpiration {},

    #[error("Already voted on this proposal")]
    AlreadyVoted {},

    #[error("Invalid proposal status. current: {current}, desired: {desired}")]
    InvalidProposalStatus { current: String, desired: String },

    #[error("Total staked amount is too low")]
    LackOfStakes {},

    #[error("Cannot deposit to non-pended proposals")]
    WrongDepositStatus {},

    #[error("Cannot execute completed or unpassed proposals")]
    WrongExecuteStatus {},

    #[error("Cannot close completed or passed proposals")]
    WrongCloseStatus {},

    #[error("Deposit not claimable")]
    DepositNotClaimable {},

    #[error("Deposit already claimed")]
    DepositAlreadyClaimed {},

    #[error("Got a submessage reply with unknown id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Request size ({size}) is above limit of ({max})")]
    OversizedRequest { size: u64, max: u64 },

    #[error("DAO is paused")]
    Paused {},
}
