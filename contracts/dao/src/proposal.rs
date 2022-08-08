use cosmwasm_std::{Addr, BlockInfo, CosmosMsg, Decimal, Timestamp, Uint128};
use cw3::{Status, Vote};
use cw_utils::{Duration, Expiration};
use osmo_bindings::OsmosisMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::helpers::duration_to_expiry;
use crate::threshold::Threshold;
use crate::ContractError;

// we multiply by this when calculating needed_votes in order to round up properly
// Note: `10u128.pow(9)` fails as "u128::pow` is not yet stable as a const fn"
const PRECISION_FACTOR: u128 = 1_000_000_000;

// weight of votes for each option
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Votes {
    pub yes: Uint128,
    pub no: Uint128,
    pub abstain: Uint128,
    pub veto: Uint128,
}

impl Votes {
    /// sum of all votes
    pub fn total(&self) -> Uint128 {
        self.yes + self.no + self.abstain + self.veto
    }

    /// create it with a yes vote for this much
    pub fn new(init_weight: Uint128) -> Self {
        Votes {
            yes: init_weight,
            no: Uint128::zero(),
            abstain: Uint128::zero(),
            veto: Uint128::zero(),
        }
    }

    pub fn submit(&mut self, vote: Vote, weight: Uint128) -> Result<(), ContractError> {
        match vote {
            Vote::Yes => self.yes = self.yes.checked_add(weight)?,
            Vote::Abstain => self.abstain = self.abstain.checked_add(weight)?,
            Vote::No => self.no = self.no.checked_add(weight)?,
            Vote::Veto => self.veto = self.veto.checked_add(weight)?,
        }
        Ok(())
    }

    pub fn revoke(&mut self, vote: Vote, weight: Uint128) -> Result<(), ContractError> {
        match vote {
            Vote::Yes => self.yes = self.yes.checked_sub(weight)?,
            Vote::No => self.no = self.no.checked_sub(weight)?,
            Vote::Abstain => self.abstain = self.abstain.checked_sub(weight)?,
            Vote::Veto => self.veto = self.veto.checked_sub(weight)?,
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct BlockTime {
    pub height: u64,
    pub time: Timestamp,
}

impl From<BlockInfo> for BlockTime {
    fn from(info: BlockInfo) -> Self {
        Self {
            height: info.height,
            time: info.time,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Proposal {
    /// Proposal title
    pub title: String,
    /// Related link about this proposal
    pub link: String,
    /// Proposal Description
    pub description: String,
    /// Address of proposer
    pub proposer: Addr,
    /// Current status of this proposal
    pub status: Status,
    /// List of messages to execute
    pub msgs: Vec<CosmosMsg<OsmosisMsg>>,

    /// Starting time / height information
    pub submitted_at: BlockTime,
    pub deposit_ends_at: Expiration,
    pub vote_starts_at: BlockTime,
    pub vote_ends_at: Expiration,

    /// Pass requirements
    pub threshold: Threshold,
    /// The total weight when the proposal started (used to calculate percentages)
    pub total_weight: Uint128,
    /// summary of existing votes
    pub votes: Votes,
    /// Amount of the native governance token required for voting
    pub total_deposit: Uint128,
    pub deposit_base_amount: Uint128,
    pub deposit_claimable: bool,
}

impl Default for Proposal {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            link: "".to_string(),
            description: "".to_string(),
            proposer: Addr::unchecked(""),
            status: Status::Pending,
            msgs: vec![],
            submitted_at: Default::default(),
            deposit_ends_at: Default::default(),
            vote_starts_at: Default::default(),
            vote_ends_at: Default::default(),
            threshold: Default::default(),
            total_weight: Default::default(),
            votes: Default::default(),
            total_deposit: Default::default(),
            deposit_base_amount: Default::default(),
            deposit_claimable: false,
        }
    }
}

impl Proposal {
    pub fn activate_voting_period(&mut self, block_time: BlockTime, voting_period: &Duration) {
        self.status = Status::Open;
        self.vote_starts_at = block_time;
        self.vote_ends_at = duration_to_expiry(&self.vote_starts_at, voting_period);
    }

    /// current_status is non-mutable and returns what the status should be.
    /// (designed for queries)
    pub fn current_status(&self, block: &BlockInfo) -> Status {
        let mut status = self.status;

        match status {
            // if pending, check if voting is opened or timed out
            Status::Pending => {
                // check total deposit amount exceeds deposit base amount
                if self.deposit_base_amount <= self.total_deposit {
                    status = Status::Open;
                } else if self.deposit_ends_at.is_expired(block) {
                    // if not and deposit period ended, change proposal status to rejected.
                    status = Status::Rejected;
                }
            }

            // if open, check if voting is passed or timed out
            Status::Open => {
                // check voting period has ended
                if self.vote_ends_at.is_expired(block) {
                    if self.is_passed() {
                        status = Status::Passed;
                    } else {
                        status = Status::Rejected;
                    }
                }
            }
            _ => {} // do nothing
        }

        status
    }

    /// update_status sets the status of the proposal to current_status.
    /// (designed for handler logic)
    pub fn update_status(&mut self, block: &BlockInfo) {
        self.status = self.current_status(block);
    }

    // returns true if this proposal is sure to pass (even before expiration if no future
    // sequence of possible votes can cause it to fail)
    pub fn is_passed(&self) -> bool {
        // we always require the quorum
        if self.votes.total() < votes_needed(self.total_weight, self.threshold.quorum) {
            return false;
        }
        // remove abstain to calculate opinions
        let opinions = self.votes.total() - self.votes.abstain;
        let passed = self.votes.yes >= votes_needed(opinions, self.threshold.threshold);
        let vetoed = self.is_vetoed();

        !vetoed && passed
    }

    // returns true if this proposal vetoed
    pub fn is_vetoed(&self) -> bool {
        self.votes.veto >= votes_needed(self.total_weight, self.threshold.veto_threshold)
    }
}

// this is a helper function so Decimal works with u64 rather than Uint128
// also, we must *round up* here, as we need 8, not 7 votes to reach 50% of 15 total
fn votes_needed(weight: Uint128, percentage: Decimal) -> Uint128 {
    let applied = percentage * Uint128::from(PRECISION_FACTOR * weight.u128());
    // Divide by PRECISION_FACTOR, rounding up to the nearest integer
    Uint128::from((applied.u128() + PRECISION_FACTOR - 1) / PRECISION_FACTOR)
}

#[cfg(test)]
mod test {
    use std::ops::Add;

    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::Env;

    use super::*;

    #[test]
    fn count_votes() {
        let mut votes = Votes::new(Uint128::new(5));
        votes.submit(Vote::No, Uint128::new(10)).unwrap();
        votes.submit(Vote::Veto, Uint128::new(20)).unwrap();
        votes.submit(Vote::Yes, Uint128::new(30)).unwrap();
        votes.submit(Vote::Abstain, Uint128::new(40)).unwrap();

        assert_eq!(votes.total(), Uint128::new(105));
        assert_eq!(votes.yes, Uint128::new(35));
        assert_eq!(votes.no, Uint128::new(10));
        assert_eq!(votes.veto, Uint128::new(20));
        assert_eq!(votes.abstain, Uint128::new(40));
    }

    #[test]
    // we ensure this rounds up (as it calculates needed votes)
    fn votes_needed_rounds_properly() {
        // round up right below 1
        assert_eq!(
            Uint128::new(1),
            votes_needed(Uint128::new(3), Decimal::permille(333))
        );
        // round up right over 1
        assert_eq!(
            Uint128::new(2),
            votes_needed(Uint128::new(3), Decimal::permille(334))
        );
        assert_eq!(
            Uint128::new(11),
            votes_needed(Uint128::new(30), Decimal::permille(334))
        );

        // exact matches don't round
        assert_eq!(
            Uint128::new(17),
            votes_needed(Uint128::new(34), Decimal::percent(50))
        );
        assert_eq!(
            Uint128::new(12),
            votes_needed(Uint128::new(48), Decimal::percent(25))
        );
    }

    mod pending {
        use super::*;

        fn suite(
            env: &Env,
            deposit_base: Uint128,
            total_deposit: Uint128,
            is_expired: bool,
        ) -> Proposal {
            let expires = match is_expired {
                true => Expiration::AtHeight(env.block.height - 5),
                false => Expiration::AtHeight(env.block.height + 100),
            };

            Proposal {
                status: Status::Pending,

                // time
                submitted_at: BlockTime {
                    height: env.block.height - 20,
                    time: Default::default(),
                },
                deposit_ends_at: expires,
                vote_starts_at: BlockTime {
                    height: env.block.height + 10,
                    time: Default::default(),
                },
                vote_ends_at: expires.add(Duration::Height(20)).unwrap(),

                // deposit
                total_deposit,
                deposit_base_amount: deposit_base,

                ..Default::default()
            }
        }

        fn assert_pending(env: &Env, prop: Proposal) {
            assert_eq!(prop.current_status(&env.block), Status::Pending);
        }

        fn assert_opened(env: &Env, prop: Proposal) {
            assert_eq!(prop.current_status(&env.block), Status::Open);
        }

        fn assert_rejected(env: &Env, prop: Proposal) {
            assert_eq!(prop.current_status(&env.block), Status::Rejected);
        }

        #[test]
        fn test() {
            let env = mock_env();
            let deposit_base = Uint128::new(100);

            // deposit < total_deposit & !expired
            assert_opened(&env, suite(&env, deposit_base, Uint128::new(110), false));

            // deposit < total_deposit & expired
            assert_opened(&env, suite(&env, deposit_base, Uint128::new(110), true));

            // deposit = total_deposit & !expired
            assert_opened(&env, suite(&env, deposit_base, Uint128::new(100), false));

            // deposit = total_deposit & expired
            assert_opened(&env, suite(&env, deposit_base, Uint128::new(100), true));

            // deposit > total_deposit & !expired
            assert_pending(&env, suite(&env, deposit_base, Uint128::new(90), false));

            // deposit > total_deposit & expired
            assert_rejected(&env, suite(&env, deposit_base, Uint128::new(90), true));
        }
    }

    mod open {
        use super::*;

        fn suite(
            env: &Env,
            threshold: &Threshold,
            votes: &Votes,
            total_weight: Uint128,
            is_expired: bool,
        ) -> Proposal {
            let expires = match is_expired {
                // voting period
                true => Expiration::AtHeight(env.block.height - 5),
                false => Expiration::AtHeight(env.block.height + 100),
            };

            Proposal {
                status: Status::Open,

                // time
                submitted_at: BlockTime {
                    height: env.block.height - 30,
                    time: Default::default(),
                },
                deposit_ends_at: Expiration::AtHeight(env.block.height - 20),
                vote_starts_at: BlockTime {
                    height: env.block.height - 10,
                    time: Default::default(),
                },
                vote_ends_at: expires,

                // vote
                threshold: threshold.clone(),
                total_weight,
                votes: votes.clone(),

                ..Default::default()
            }
        }

        fn assert_opened(env: &Env, prop: Proposal) {
            assert_eq!(prop.current_status(&env.block), Status::Open);
        }

        fn assert_passed(env: &Env, prop: Proposal) {
            assert!(prop.is_passed());
            assert_eq!(prop.current_status(&env.block), Status::Passed);
        }

        fn assert_rejected(env: &Env, prop: Proposal) {
            assert!(!prop.is_passed());
            assert_eq!(prop.current_status(&env.block), Status::Rejected);
        }

        fn assert_vetoed(env: &Env, prop: Proposal) {
            assert!(!prop.is_passed());
            assert!(prop.is_vetoed());
            assert_eq!(prop.current_status(&env.block), Status::Rejected)
        }

        #[test]
        fn test_in_voting_period() {
            let quorum = Threshold {
                threshold: Decimal::percent(50),
                quorum: Decimal::percent(40),
                veto_threshold: Decimal::percent(33),
            };

            let env = mock_env();

            // !expired & passed
            let votes = Votes {
                yes: Uint128::new(100),
                no: Default::default(),
                abstain: Default::default(),
                veto: Default::default(),
            };
            assert_opened(&env, suite(&env, &quorum, &votes, votes.total(), false));

            // !expired & rejected - threshold
            let votes = Votes {
                yes: Default::default(),
                no: Uint128::new(100),
                abstain: Default::default(),
                veto: Default::default(),
            };
            assert_opened(&env, suite(&env, &quorum, &votes, votes.total(), false));

            // !expired & rejected - vetoed
            let votes = Votes {
                yes: Default::default(),
                no: Default::default(),
                abstain: Default::default(),
                veto: Uint128::new(100),
            };
            assert_opened(&env, suite(&env, &quorum, &votes, votes.total(), false));
        }

        #[test]
        fn test_out_of_voting_period() {
            let quorum = Threshold {
                threshold: Decimal::percent(50),
                quorum: Decimal::percent(40),
                veto_threshold: Decimal::percent(33),
            };

            let env = mock_env();

            // === expired & over quorum (passed)
            // over quorum (40% of 30 = 12), over threshold (7/11 > 50%)
            let pass = Votes {
                yes: Uint128::new(7),
                no: Uint128::new(3),
                abstain: Uint128::new(2),
                veto: Uint128::new(1),
            };
            let weight = Uint128::new(30);
            assert_passed(&env, suite(&env, &quorum, &pass, weight, true));

            // === expired & over quorum & abstain buffer (passed)
            // over quorum, threshold passes if we ignore abstain
            // 17 total votes w/ abstain => 40% quorum of 40 total
            // 6 yes / (6 yes + 4 no + 2 votes) => 50% threshold
            let pass = Votes {
                yes: Uint128::new(6),
                no: Uint128::new(4),
                abstain: Uint128::new(5),
                veto: Uint128::new(2),
            };
            let weight = Uint128::new(40);
            assert_passed(&env, suite(&env, &quorum, &pass, weight, true));

            // === expired & under quorum (rejected)
            // under quorum (40% of 33 = 13.2 > 13)
            let reject = Votes {
                yes: Uint128::new(7),
                no: Uint128::new(3),
                abstain: Uint128::new(2),
                veto: Uint128::new(1),
            };
            let weight = Uint128::new(33);
            assert_rejected(&env, suite(&env, &quorum, &reject, weight, true));

            // === expired & under threshold (rejected)
            // over quorum (40% of 20 = 8)
            // under pass threshold (50% of (6 + 5 + 2) = 6.5 > 6)
            // under veto threshold (33% of 20 = 6.6 > 2)
            let reject = Votes {
                yes: Uint128::new(6),
                no: Uint128::new(5),
                abstain: Uint128::new(2),
                veto: Uint128::new(2),
            };
            let weight = Uint128::new(20);
            assert_rejected(&env, suite(&env, &quorum, &reject, weight, true));

            // === expired & vetoed (rejected)
            // over quorum (40% of 23 = 9.2)
            // over pass threshold (50% of (11 + 2 + 8) = 10.5 < 11)
            // over veto threshold (33% of 23 = 7.59 < 8)
            let reject = Votes {
                yes: Uint128::new(11),
                no: Uint128::new(2),
                abstain: Uint128::new(2),
                veto: Uint128::new(8),
            };
            let weight = Uint128::new(23);
            assert_vetoed(&env, suite(&env, &quorum, &reject, weight, true));
        }

        #[test]
        fn quorum_edge_cases() {
            // when we pass absolute threshold (everyone else voting no, we pass), but still don't hit quorum
            let quorum = Threshold {
                threshold: Decimal::percent(60),
                quorum: Decimal::percent(80),
                veto_threshold: Decimal::percent(33),
            };

            let env = mock_env();

            // try 9 yes, 1 no (out of 15) -> 90% voter threshold, 60% absolute threshold, still no quorum
            // doesn't matter if expired or not
            let missing_voters = Votes {
                yes: Uint128::new(9),
                no: Uint128::new(1),
                abstain: Uint128::new(0),
                veto: Uint128::new(0),
            };
            assert_rejected(
                &env,
                suite(&env, &quorum, &missing_voters, Uint128::new(15), true),
            );

            // 1 less yes, 3 vetos and this passes only when expired
            let wait_til_expired = Votes {
                yes: Uint128::new(8),
                no: Uint128::new(1),
                abstain: Uint128::new(0),
                veto: Uint128::new(3),
            };
            assert_passed(
                &env,
                suite(&env, &quorum, &wait_til_expired, Uint128::new(15), true),
            );

            // 9 yes and 3 nos passes early
            let passes_early = Votes {
                yes: Uint128::new(9),
                no: Uint128::new(3),
                abstain: Uint128::new(0),
                veto: Uint128::new(0),
            };
            assert_passed(
                &env,
                suite(&env, &quorum, &passes_early, Uint128::new(15), true),
            );
        }
    }
}
