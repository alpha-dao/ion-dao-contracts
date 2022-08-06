use crate::msg::{GovToken, RangeOrder};
use crate::state::{Config, Threshold};
use crate::tests::suite::{Suite, SuiteBuilder};

use cosmwasm_std::{coin, Addr, Decimal, Uint128};
use cw3::{Status, Vote};
use cw_utils::Duration;

#[test]
fn test_get_config() {
    let suite = SuiteBuilder::new()
        .with_gov_token(GovToken::Create {
            denom: "testtest".to_string(),
            label: "labellabel".to_string(),
            stake_contract_code_id: 0,
            unstaking_duration: None,
        })
        .with_threshold(Threshold {
            threshold: Decimal::percent(80),
            quorum: Decimal::percent(20),
            veto_threshold: Decimal::percent(99),
        })
        .with_periods(Some(Duration::Height(99)), Some(Duration::Height(10)))
        .with_deposits(Some(Uint128::new(10)), Some(Uint128::new(100)))
        .build();

    let config = suite.query_config().unwrap();

    assert_eq!(config.gov_token, "testtest");
    assert_eq!(config.staking_contract, suite.stake);
    assert_eq!(
        config.config,
        Config {
            name: "dao".to_string(),
            description: "desc".to_string(),
            threshold: Threshold {
                threshold: Decimal::percent(80),
                quorum: Decimal::percent(20),
                veto_threshold: Decimal::percent(99),
            },
            voting_period: Duration::Height(99),
            deposit_period: Duration::Height(10),
            proposal_deposit: Uint128::new(100),
            proposal_min_deposit: Uint128::new(10)
        }
    );
}

#[test]
fn test_token_list() {
    let mut suite = SuiteBuilder::new().build();

    let dao = suite.dao.clone();

    let resp = suite.query_token_list().unwrap();
    assert_eq!(resp.token_list, vec!["denom".to_string()]);

    suite
        .update_token_list(
            dao.as_str(),
            vec!["native-2".to_string(), "native-1".to_string()],
            vec![],
        )
        .unwrap();

    let resp = suite.query_token_list().unwrap();
    assert_eq!(
        resp.token_list,
        vec![
            "denom".to_string(),
            "native-1".to_string(),
            "native-2".to_string(),
        ]
    );
}

#[test]
fn test_token_balances() {
    let mut suite = SuiteBuilder::new()
        .with_funds(vec![("tester0", 10)])
        .build();

    let dao = suite.dao.clone();

    suite
        .update_token_list(
            dao.as_str(),
            vec![
                "native-3".to_string(),
                "native-4".to_string(),
                "native-1".to_string(),
                "native-2".to_string(),
            ],
            vec![],
        )
        .unwrap();

    let resp = suite.query_token_balances(None, None, None).unwrap();
    assert_eq!(
        resp.balances,
        vec![
            coin(0, "denom"),
            coin(0, "native-1"),
            coin(0, "native-2"),
            coin(0, "native-3"),
            coin(0, "native-4"),
        ]
    );
}

mod proposal {
    use super::*;

    use crate::msg::ProposalsQueryOption;

    fn setup_proposal_state(owner: &str, suite: &mut Suite) {
        /***
         * |----------|--------------------------------------|
         * |          |            proposal_status           |
         * | proposer |--------------------------------------|
         * |          | pending | open | rejected | executed |
         * |----------|---------|------|----------|----------|
         * | tester0  |      10 |    9 |        1 |        2 |
         * | tester1  |      12 |   11 |        3 |        4 |
         * | tester2  |      14 |   13 |        5 |        6 |
         * | tester3  |      16 |   15 |        7 |        8 |
         * |----------|--------------------------------------|
         */

        for i in 0..4 {
            let proposer = format!("tester{}", i);

            // REJECTED
            let rejected_prop_id = (i * 2) + 1;
            suite
                .propose(&proposer, "t", "l", "d", vec![], Some(100))
                .unwrap();
            suite.vote(owner, rejected_prop_id, Vote::No).unwrap();
            suite.app().advance_blocks(15);
            suite.close_proposal(owner, rejected_prop_id).unwrap();

            // EXECUTED
            let executed_prop_id = (i * 2) + 2;
            suite
                .propose(&proposer, "t", "l", "d", vec![], Some(100))
                .unwrap();
            suite.vote(owner, executed_prop_id, Vote::Yes).unwrap();
            suite.app().advance_blocks(15);
            suite.execute_proposal(owner, executed_prop_id).unwrap();
        }

        suite.app().advance_blocks(1);

        for i in 0..4 {
            let proposer = format!("tester{}", i);

            // OPEN
            suite
                .propose(&proposer, "t", "l", "d", vec![], Some(100))
                .unwrap();
            // PENDING
            suite
                .propose(&proposer, "t", "l", "d", vec![], Some(10))
                .unwrap();
        }

        suite.app().advance_blocks(1);
    }

    fn pre_setup_proposal_state() -> Suite {
        let mut suite = SuiteBuilder::new()
            .with_funds(
                [0; 4]
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (format!("tester{}", i), 100000000))
                    .collect::<Vec<(String, u128)>>(),
            )
            .with_staked(vec![("owner", 100u128)])
            .build();

        setup_proposal_state("owner", &mut suite);

        suite
    }

    #[test]
    fn test_single_query() {
        let mut builder = SuiteBuilder::new().with_staked(vec![("owner", 100u128)]);
        for i in 1..10 {
            builder = builder.add_proposal(i.to_string(), i.to_string(), i.to_string(), vec![]);
        }

        let suite = builder.build();
        for i in 1..10 {
            let resp = suite.query_proposal(i).unwrap();
            assert_eq!(resp.id, i);
            assert_eq!(resp.title, i.to_string());
            assert_eq!(resp.link, i.to_string());
            assert_eq!(resp.description, i.to_string());
        }
    }

    #[test]
    fn test_multi_query_everything() {
        let suite = pre_setup_proposal_state();

        // everything
        let range_cases = &[
            (None, None, None),
            (Some(10u64), None, None),
            (None, Some(30u32), None),
            (None, None, Some(RangeOrder::Desc)),
        ];
        let expected = &[
            (10, 1u64, 10u64),
            (6, 11u64, 16u64),
            (16, 1u64, 16u64),
            (10, 16u64, 7u64),
        ];
        for i in 0..4 {
            let (start, limit, order) = range_cases.get(i).unwrap();
            let (len, first, last) = expected.get(i).unwrap();

            let resp = suite
                .query_proposals(
                    ProposalsQueryOption::Everything {},
                    *start,
                    *limit,
                    order.clone(),
                )
                .unwrap();
            assert_eq!(resp.proposals.len(), *len as usize);
            assert_eq!(resp.proposals.first().unwrap().id, *first);
            assert_eq!(resp.proposals.last().unwrap().id, *last);
        }
    }

    #[test]
    fn test_multi_query_by_proposer() {
        let suite = pre_setup_proposal_state();

        // find by proposer
        let range_cases = &[
            (None, None, None),
            (Some(8u64), None, None),
            (None, Some(30u32), None),
            (None, None, Some(RangeOrder::Desc)),
        ];
        let expected = &[
            (4, Status::Rejected, Status::Pending),
            (2, Status::Open, Status::Pending),
            (4, Status::Rejected, Status::Pending),
            (4, Status::Pending, Status::Rejected),
        ];
        for i in 0..4 {
            for j in 0..4 {
                let proposer = Addr::unchecked(format!("tester{}", i));
                let (start, limit, order) = range_cases.get(j).unwrap();
                let (len, first, last) = expected.get(j).unwrap();

                let resp = suite
                    .query_proposals(
                        ProposalsQueryOption::FindByProposer {
                            proposer: proposer.clone(),
                        },
                        *start,
                        *limit,
                        order.clone(),
                    )
                    .unwrap();
                assert_eq!(
                    resp.proposals
                        .iter()
                        .map(|x| x.proposer.to_string())
                        .collect::<Vec<String>>(),
                    resp.proposals
                        .iter()
                        .map(|_| proposer.to_string())
                        .collect::<Vec<String>>(),
                );
                assert_eq!(resp.proposals.len(), *len as usize);
                assert_eq!(resp.proposals.first().unwrap().status, *first);
                assert_eq!(resp.proposals.last().unwrap().status, *last);
            }
        }
    }

    #[test]
    fn test_multi_query_by_status() {
        let suite = pre_setup_proposal_state();

        // find by status
        let expected = &[
            [10u64, 12u64, 14u64, 16u64],
            [9u64, 11u64, 13u64, 15u64],
            [1u64, 3u64, 5u64, 7u64],
            [2u64, 4u64, 6u64, 8u64],
        ];
        for (i, status) in vec![
            Status::Pending,
            Status::Open,
            Status::Rejected,
            Status::Executed,
        ]
        .iter()
        .enumerate()
        {
            let ids = expected.get(i).unwrap();
            let assert_list = ids
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, format!("tester{}", i)))
                .collect::<Vec<(u64, String)>>();

            let resp = suite
                .query_proposals(
                    ProposalsQueryOption::FindByStatus { status: *status },
                    None,
                    None,
                    None,
                )
                .unwrap();
            assert!(resp
                .proposals
                .iter()
                .map(|x| (x.id, x.proposer.to_string()))
                .collect::<Vec<(u64, String)>>()
                .eq(&assert_list))
        }
    }

    #[test]
    fn test_query_count() {
        let suite = pre_setup_proposal_state();

        let count = suite.query_proposal_count().unwrap();
        assert_eq!(count, 16);
    }
}

mod vote {
    use super::*;

    fn setup_voting_state(_: &str, suite: &mut Suite) {
        /***
         * |------------|----------------|
         * |            |      vote      |
         * |  voter     |----------------|
         * |  / prop_id | Y | N | A | NV |
         * |------------|---|---|---|----|
         * |    tester0 | 1 | 2 | 3 |  4 |
         * |    tester1 | 2 | 3 | 4 |  1 |
         * |    tester2 | 3 | 4 | 1 |  2 |
         * |    tester3 | 4 | 1 | 2 |  3 |
         * |------------|----------------|
         */

        let votes = vec![Vote::Yes, Vote::No, Vote::Abstain, Vote::Veto];
        for i in 0u64..4 {
            let voter = format!("tester{}", i);
            let id = |x: u64| x % 4 + 1;

            for j in i..(i + 4) {
                let option = votes.get((j - i) as usize).unwrap();
                suite.vote(&voter, id(j), *option).unwrap();
            }
        }
    }

    fn pre_setup_vote_state() -> Suite {
        let mut suite = SuiteBuilder::new()
            .with_staked(
                [0; 4]
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (format!("tester{}", i), 100))
                    .collect::<Vec<(String, u128)>>(),
            )
            .add_proposal("t", "l", "d", vec![]) // 1
            .add_proposal("t", "l", "d", vec![]) // 2
            .add_proposal("t", "l", "d", vec![]) // 3
            .add_proposal("t", "l", "d", vec![]) // 4
            .add_proposal("t", "l", "d", vec![]) // 5
            .build();

        setup_voting_state("owner", &mut suite);

        suite
    }

    #[test]
    fn test_single_query() {
        let suite = pre_setup_vote_state();

        let vote = suite.query_vote(1, "tester0").unwrap().vote.unwrap();
        assert_eq!(vote.vote, Vote::Yes);
        assert_eq!(vote.weight, Uint128::new(100));
        assert_eq!(vote.voter, "tester0");

        assert!(suite.query_vote(5, "tester0").unwrap().vote.is_none());
    }

    #[test]
    fn test_multi_query() {
        let suite = pre_setup_vote_state();

        let votes = &[Vote::Yes, Vote::No, Vote::Abstain, Vote::Veto];
        for i in 0..4 {
            let id = i + 1;
            let options = [0; 4]
                .iter()
                .enumerate()
                .map(|(j, _)| (format!("tester{}", j), (4 + i - j as u64) % 4))
                .map(|(p, v)| (p, *votes.get(v as usize).unwrap()))
                .collect::<Vec<(String, Vote)>>();

            let resp = suite.query_votes(id, None, None, None).unwrap();
            assert!(resp
                .votes
                .iter()
                .map(|x| (x.voter.clone(), x.vote))
                .collect::<Vec<(String, Vote)>>()
                .eq(&options));
        }
    }
}

mod deposit {
    use super::*;

    fn setup_deposit_state(_: &str, suite: &mut Suite) {
        /***
         * |------------|---------------|
         * |            |    prop_id    |
         * |  voter     |---------------|
         * |  / deposit | 1 | 2 | 3 | 4 |
         * |------------|---|---|---|---|
         * |    tester0 | 1 | 2 | 3 | 4 |
         * |    tester1 | 2 | 3 | 4 | 1 |
         * |    tester2 | 3 | 4 | 1 | 2 |
         * |    tester3 | 4 | 1 | 2 | 3 |
         * |------------|---------------|
         */

        for i in 0..4 {
            let depositor = format!("tester{}", i);
            let deposit = |x: u64| x % 4 + 1;

            for j in i..(i + 4) {
                let prop = j - i + 1;
                suite
                    .deposit(&depositor, prop, Some(deposit(j) as u128))
                    .unwrap();
            }
        }
    }

    fn pre_setup_deposit_state() -> Suite {
        let mut suite = SuiteBuilder::new()
            .with_funds(
                [0; 4]
                    .iter()
                    .enumerate()
                    .map(|(i, _)| (format!("tester{}", i), 10))
                    .collect::<Vec<(String, u128)>>(),
            )
            .with_funds(vec![("owner", 40)])
            .with_staked(vec![("owner", 10)])
            .build();
        for _ in 0..4 {
            suite
                .propose("owner", "t", "l", "d", vec![], Some(10))
                .unwrap();
        }

        setup_deposit_state("owner", &mut suite);

        suite
    }

    #[test]
    fn test_single_query() {
        let suite = pre_setup_deposit_state();

        for i in 0..4 {
            for j in 0..4 {
                let depositor = format!("tester{}", j);
                let resp = suite.query_deposit(i + 1, depositor.as_str()).unwrap();
                assert_eq!(resp.amount, Uint128::new(((j + i) % 4 + 1) as u128));
                assert_eq!(resp.depositor, depositor);
                assert_eq!(resp.proposal_id, i + 1);
            }
        }
    }

    // TODO
    // #[test]
    // fn test_multi_query_everything() {
    //     let suite = pre_setup_deposit_state();
    //
    //     let resp = suite
    //         .query_deposits(DepositsQueryOption::Everything { start: None }, None, None)
    //         .unwrap();
    //
    //
    // }
}
