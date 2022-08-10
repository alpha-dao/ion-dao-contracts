pub use crate::error::ContractError;

// Settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub mod contract;
mod error;
mod helpers;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;
