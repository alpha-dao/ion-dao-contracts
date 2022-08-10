use cosmwasm_std::{Addr, Decimal, Uint128};
use cw2::query_contract_info;
use cw_multi_test::Executor;
use cw_utils::Duration;
use osmo_bindings_test::OsmosisApp;

use crate::msg::{ConfigResponse, GovToken, InstantiateMsg, QueryMsg, TokenListResponse};
use crate::state::Threshold;
use crate::tests::suite::{contract_dao, contract_stake};
use crate::ContractError;

fn prepare() -> (OsmosisApp, u64, u64) {
    let mut app = OsmosisApp::default();

    let dao_code_id = app.store_code(contract_dao());
    let stake_code_id = app.store_code(contract_stake());

    (app, dao_code_id, stake_code_id)
}

enum Stake {
    Code(u64),
    Addr(Addr),
}

fn happy_init_msg(stake: Stake) -> InstantiateMsg {
    InstantiateMsg {
        name: "test_dao".to_string(),
        description: "test_test".to_string(),
        gov_token: match stake {
            Stake::Code(code) => GovToken::Create {
                denom: "utnt".to_string(),
                label: "new_contract".to_string(),
                stake_contract_code_id: code,
                unstaking_duration: Some(Duration::Height(10)),
            },
            Stake::Addr(addr) => GovToken::Reuse {
                stake_contract: addr.to_string(),
            },
        },
        threshold: Threshold {
            threshold: Decimal::percent(50),
            quorum: Decimal::percent(40),
            veto_threshold: Decimal::percent(33),
        },
        voting_period: Duration::Height(20),
        deposit_period: Duration::Height(10),
        proposal_deposit_amount: Uint128::new(100),
        proposal_deposit_min_amount: Uint128::new(10),
    }
}

#[test]
fn should_work_with_new_stake_contract() {
    let (mut app, dao_code_id, stake_code_id) = prepare();

    let maker = Addr::unchecked("maker");
    let init_msg = happy_init_msg(Stake::Code(stake_code_id));
    let dao_addr = app
        .instantiate_contract(dao_code_id, maker, &init_msg, &[], "new_dao", None)
        .unwrap();

    // check config
    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&dao_addr, &QueryMsg::GetConfig {})
        .unwrap();

    assert_eq!(
        query_contract_info(&app, &config.staking_contract)
            .unwrap()
            .contract,
        "crates.io:ion-stake".to_string(),
    );
    assert_eq!(
        query_contract_info(&app, &dao_addr).unwrap().contract,
        "crates.io:ion-dao".to_string(),
    );
    assert_eq!(config.gov_token, "utnt".to_string());

    // check treasury tokens
    let token_list_resp: TokenListResponse = app
        .wrap()
        .query_wasm_smart(&dao_addr, &QueryMsg::TokenList {})
        .unwrap();

    assert_eq!(token_list_resp.token_list, vec!["utnt".to_string()]);
}

#[test]
fn should_work_with_existing_stake_contract() {
    let (mut app, dao_code_id, stake_code_id) = prepare();

    let maker = Addr::unchecked("maker");
    let stake_addr = app
        .instantiate_contract(
            stake_code_id,
            maker.clone(),
            &ion_stake::msg::InstantiateMsg {
                admin: None,
                denom: "utnt".to_string(),
                unstaking_duration: Some(Duration::Height(20)),
            },
            &[],
            "new_stake",
            None,
        )
        .unwrap();

    let dao_init_msg = happy_init_msg(Stake::Addr(stake_addr.clone()));
    let dao_addr = app
        .instantiate_contract(dao_code_id, maker, &dao_init_msg, &[], "new_dao", None)
        .unwrap();

    // check config
    let config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&dao_addr, &QueryMsg::GetConfig {})
        .unwrap();

    assert_eq!(config.staking_contract, stake_addr);
    assert_eq!(config.gov_token, "utnt".to_string());

    // check treasury tokens
    let token_list_resp: TokenListResponse = app
        .wrap()
        .query_wasm_smart(&dao_addr, &QueryMsg::TokenList {})
        .unwrap();

    assert_eq!(token_list_resp.token_list, vec!["utnt".to_string()]);
}

#[test]
fn should_fail_if_threshold_is_invalid() {
    let (mut app, dao_code_id, stake_code_id) = prepare();

    let maker = Addr::unchecked("maker");

    let dao_init_msg = happy_init_msg(Stake::Code(stake_code_id));

    let mut cases: Vec<InstantiateMsg> = vec![];

    let mut t1 = dao_init_msg.clone();
    t1.threshold.veto_threshold = Decimal::percent(101);
    cases.push(t1);

    let mut t2 = dao_init_msg.clone();
    t2.threshold.threshold = Decimal::percent(101);
    cases.push(t2);

    let mut t3 = dao_init_msg.clone();
    t3.threshold.quorum = Decimal::percent(101);
    cases.push(t3);

    for case in cases.iter() {
        let err = app
            .instantiate_contract(dao_code_id, maker.clone(), case, &[], "new_dao", None)
            .unwrap_err();
        assert_eq!(
            ContractError::UnreachableThreshold {},
            err.downcast().unwrap()
        );
    }

    let mut cases: Vec<InstantiateMsg> = vec![];

    let mut t1 = dao_init_msg.clone();
    t1.threshold.veto_threshold = Decimal::percent(0);
    cases.push(t1);

    let mut t2 = dao_init_msg.clone();
    t2.threshold.threshold = Decimal::percent(0);
    cases.push(t2);

    let mut t3 = dao_init_msg;
    t3.threshold.quorum = Decimal::percent(0);
    cases.push(t3);

    for case in cases.iter() {
        let err = app
            .instantiate_contract(dao_code_id, maker.clone(), case, &[], "new_dao", None)
            .unwrap_err();
        assert_eq!(ContractError::ZeroThreshold {}, err.downcast().unwrap());
    }
}

#[test]
fn should_fail_if_period_is_invalid() {
    let (mut app, dao_code_id, stake_code_id) = prepare();

    let maker = Addr::unchecked("maker");

    let cases = vec![
        (Duration::Height(10), Duration::Time(10)),  // fail
        (Duration::Time(10), Duration::Height(10)),  // fail
        (Duration::Height(10), Duration::Height(9)), // fail
        (Duration::Time(10), Duration::Time(9)),     // fail
    ];

    for (deposit, voting) in cases {
        let mut init_msg = happy_init_msg(Stake::Code(stake_code_id));
        init_msg.deposit_period = deposit;
        init_msg.voting_period = voting;

        let err = app
            .instantiate_contract(dao_code_id, maker.clone(), &init_msg, &[], "new_dao", None)
            .unwrap_err();
        assert_eq!(ContractError::InvalidPeriod {}, err.downcast().unwrap());
    }
}
