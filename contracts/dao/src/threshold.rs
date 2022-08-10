use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ContractError;

/// Declares a `quorum` of the total votes that must participate in the election in order
/// for the vote to be considered at all.
/// See `ThresholdResponse.ThresholdQuorum` in the cw3 spec for details.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, JsonSchema, Debug)]
pub struct Threshold {
    pub threshold: Decimal,
    pub quorum: Decimal,
    pub veto_threshold: Decimal,
}

impl Default for Threshold {
    fn default() -> Self {
        Self {
            threshold: Decimal::from_ratio(1u128, 2u128),      // 50%
            quorum: Decimal::from_ratio(1u128, 3u128),         // 33%
            veto_threshold: Decimal::from_ratio(1u128, 3u128), // 33%
        }
    }
}

impl Threshold {
    /// returns error if this is an unreachable value,
    /// given a total weight of all members in the group
    pub fn validate(&self) -> Result<(), ContractError> {
        valid_percentage(&self.threshold)?;
        valid_percentage(&self.quorum)?;
        valid_percentage(&self.veto_threshold)
    }
}

/// Asserts that the 0.0 < percent <= 1.0
fn valid_percentage(percent: &Decimal) -> Result<(), ContractError> {
    if percent.is_zero() {
        Err(ContractError::ZeroThreshold {})
    } else if *percent > Decimal::one() {
        Err(ContractError::UnreachableThreshold {})
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_percentage() {
        // 0 is never a valid percentage
        let err = valid_percentage(&Decimal::zero()).unwrap_err();
        assert_eq!(err.to_string(), ContractError::ZeroThreshold {}.to_string());

        // 100% is
        valid_percentage(&Decimal::one()).unwrap();

        // 101% is not
        let err = valid_percentage(&Decimal::percent(101)).unwrap_err();
        assert_eq!(
            err.to_string(),
            ContractError::UnreachableThreshold {}.to_string()
        );
        // not 100.1%
        let err = valid_percentage(&Decimal::permille(1001)).unwrap_err();
        assert_eq!(
            err.to_string(),
            ContractError::UnreachableThreshold {}.to_string()
        );

        // other values in between 0 and 1 are valid
        valid_percentage(&Decimal::permille(1)).unwrap();
        valid_percentage(&Decimal::percent(17)).unwrap();
        valid_percentage(&Decimal::percent(99)).unwrap();
    }

    #[test]
    fn validate_threshold() {
        // Quorum enforces both valid just enforces valid_percentage (tested above)
        Threshold {
            threshold: Decimal::percent(51),
            quorum: Decimal::percent(40),
            veto_threshold: Decimal::percent(33),
        }
        .validate()
        .unwrap();
        let err = Threshold {
            threshold: Decimal::percent(101),
            quorum: Decimal::percent(40),
            veto_threshold: Decimal::percent(33),
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            ContractError::UnreachableThreshold {}.to_string()
        );
        let err = Threshold {
            threshold: Decimal::percent(51),
            quorum: Decimal::percent(0),
            veto_threshold: Decimal::percent(10),
        }
        .validate()
        .unwrap_err();
        assert_eq!(err.to_string(), ContractError::ZeroThreshold {}.to_string());
    }
}
