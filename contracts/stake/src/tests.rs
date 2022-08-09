use anyhow::Result as AnyResult;
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::{coin, coins, Addr, BankMsg, Coin, Uint128};
use cw_controllers::Claim;
use cw_multi_test::{
    next_block, AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg,
};
use cw_utils::Expiration::AtHeight;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};
use osmo_bindings_test::OsmosisApp;

use crate::msg::{
    ClaimsResponse, Duration, ExecuteMsg, GetConfigResponse, QueryMsg, StakedValueResponse,
    TotalPowerAtHeightResponse, TotalValueResponse, VotingPowerAtHeightResponse,
};
use crate::state::MAX_CLAIMS;
use crate::ContractError;

const DENOM: &str = "denom";
const ADDR_OWNER: &str = "owner";
const ADDR_OWNER2: &str = "owner2";
const ADDR1: &str = "addr0001";
const ADDR2: &str = "addr0002";
const ADDR3: &str = "addr0003";
const ADDR4: &str = "addr0004";

fn mock_app() -> OsmosisApp {
    OsmosisApp::default()
}

fn mock_staking_code() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    Box::new(ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ))
}

fn mock_staking(app: &mut OsmosisApp, unstaking_duration: Option<Duration>) -> Stake {
    let staking_code_id = app.store_code(mock_staking_code());
    let msg = crate::msg::InstantiateMsg {
        admin: Some(Addr::unchecked(ADDR_OWNER)),
        denom: DENOM.to_string(),
        unstaking_duration,
    };
    let address = app
        .instantiate_contract(
            staking_code_id,
            Addr::unchecked(ADDR1),
            &msg,
            &[],
            "staking",
            None,
        )
        .unwrap();

    Stake { address }
}

fn setup_test_case(
    app: &mut OsmosisApp,
    initial_balances: Vec<(&str, u128)>,
    unstaking_duration: Option<Duration>,
) -> Stake {
    for (address, amount) in initial_balances.iter() {
        app.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: address.to_string(),
            amount: coins(*amount, DENOM),
        }))
        .unwrap();
        app.update_block(next_block)
    }

    let staking = mock_staking(app, unstaking_duration);
    app.update_block(next_block);

    staking
}

fn get_balance(app: &OsmosisApp, addr: &str) -> Uint128 {
    app.wrap().query_balance(addr, DENOM).unwrap().amount
}

struct Stake {
    pub address: Addr,
}

impl Stake {
    // ============================ EXECUTIONS

    pub fn stake(
        &self,
        app: &mut OsmosisApp,
        sender: &Addr,
        amount: Coin,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender.clone(),
            self.address.clone(),
            &ExecuteMsg::Stake {},
            &[amount],
        )
    }

    pub fn fund(
        &self,
        app: &mut OsmosisApp,
        sender: &Addr,
        amount: Coin,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender.clone(),
            self.address.clone(),
            &ExecuteMsg::Fund {},
            &[amount],
        )
    }

    pub fn unstake(
        &self,
        app: &mut OsmosisApp,
        sender: &Addr,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender.clone(),
            self.address.clone(),
            &ExecuteMsg::Unstake { amount },
            &[],
        )
    }

    pub fn claim(&self, app: &mut OsmosisApp, sender: &Addr) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender.clone(),
            self.address.clone(),
            &ExecuteMsg::Claim {},
            &[],
        )
    }

    pub fn update_config(
        &self,
        app: &mut OsmosisApp,
        sender: &Addr,
        admin: Option<Addr>,
        duration: Option<Duration>,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender.clone(),
            self.address.clone(),
            &ExecuteMsg::UpdateConfig { admin, duration },
            &[],
        )
    }

    // ============================ QUERIES

    pub fn query_staked_balance_at_height(
        &self,
        app: &OsmosisApp,
        address: impl Into<String>,
        height: Option<u64>,
    ) -> VotingPowerAtHeightResponse {
        app.wrap()
            .query_wasm_smart(
                &self.address,
                &QueryMsg::VotingPowerAtHeight {
                    address: address.into(),
                    height,
                },
            )
            .unwrap()
    }

    pub fn query_total_staked_at_height(
        &self,
        app: &OsmosisApp,
        height: Option<u64>,
    ) -> TotalPowerAtHeightResponse {
        app.wrap()
            .query_wasm_smart(&self.address, &QueryMsg::TotalPowerAtHeight { height })
            .unwrap()
    }

    pub fn query_staked_value(
        &self,
        app: &OsmosisApp,
        address: impl Into<String>,
    ) -> StakedValueResponse {
        app.wrap()
            .query_wasm_smart(
                &self.address,
                &QueryMsg::StakedValue {
                    address: address.into(),
                },
            )
            .unwrap()
    }

    pub fn query_total_value(&self, app: &OsmosisApp) -> TotalValueResponse {
        app.wrap()
            .query_wasm_smart(&self.address, &QueryMsg::TotalValue {})
            .unwrap()
    }

    pub fn query_config(&self, app: &OsmosisApp) -> GetConfigResponse {
        app.wrap()
            .query_wasm_smart(&self.address, &QueryMsg::GetConfig {})
            .unwrap()
    }

    pub fn query_claims(&self, app: &OsmosisApp, address: impl Into<String>) -> ClaimsResponse {
        app.wrap()
            .query_wasm_smart(
                &self.address,
                &QueryMsg::Claims {
                    address: address.into(),
                },
            )
            .unwrap()
    }
}

#[test]
fn test_initialize() {
    let mut app = mock_app();
    let staking = mock_staking(&mut app, None);
    let config = staking.query_config(&app);
    assert_eq!(config.denom, DENOM.to_string());
    assert_eq!(config.admin, Some(Addr::unchecked(ADDR_OWNER)));
    assert_eq!(config.unstaking_duration, None);
}

#[test]
fn test_update_config() {
    let mut app = mock_app();
    let staking = setup_test_case(&mut app, vec![], None);

    // success - happy path
    let info = mock_info(ADDR_OWNER, &[]);
    let _res = staking
        .update_config(
            &mut app,
            &info.sender,
            Some(Addr::unchecked(ADDR_OWNER2)),
            Some(Duration::Height(100)),
        )
        .unwrap();
    assert_eq!(
        staking.query_config(&app),
        GetConfigResponse {
            admin: Some(Addr::unchecked(ADDR_OWNER2)),
            denom: DENOM.to_string(),
            unstaking_duration: Some(Duration::Height(100))
        }
    );

    // success - remove all
    let info = mock_info(ADDR_OWNER2, &[]);
    let _res = staking
        .update_config(&mut app, &info.sender, None, None)
        .unwrap();
    assert_eq!(
        staking.query_config(&app),
        GetConfigResponse {
            admin: None,
            denom: DENOM.to_string(),
            unstaking_duration: None
        }
    );

    // fail
    let info = mock_info(ADDR_OWNER, &[]);
    let _err = staking
        .update_config(&mut app, &info.sender, None, None)
        .unwrap_err();
}

#[test]
fn test_staking() {
    let mut app = mock_app();
    let amount1 = Uint128::from(100u128);
    let initial_balances = vec![(ADDR1, amount1.u128())];
    let staking = setup_test_case(&mut app, initial_balances, None);

    let info = mock_info(ADDR1, &[]);

    // Successful bond
    let amount = Uint128::new(50);
    let _res = staking
        .stake(&mut app, &info.sender, coin(amount.u128(), DENOM))
        .unwrap();
    app.update_block(next_block);
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(50u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(50u128)
    );
    assert_eq!(
        (&app).wrap().query_balance(ADDR1, DENOM).unwrap().amount,
        Uint128::from(50u128)
    );

    // Can't transfer bonded amount
    let msg = BankMsg::Send {
        to_address: ADDR2.to_string(),
        amount: coins(51, DENOM),
    };
    let _err = (&mut app)
        .execute(info.sender.clone(), msg.into())
        .unwrap_err();

    // Sucessful transfer of unbonded amount
    let msg = BankMsg::Send {
        to_address: ADDR2.to_string(),
        amount: coins(20, DENOM),
    };
    let _res = (&mut app).execute(info.sender.clone(), msg.into()).unwrap();

    assert_eq!(get_balance(&app, ADDR1), Uint128::from(30u128));
    assert_eq!(get_balance(&app, ADDR2), Uint128::from(20u128));

    // Addr 2 successful bond
    let info = mock_info(ADDR2, &[]);
    staking
        .stake(&mut app, &info.sender, coin(20, DENOM))
        .unwrap();

    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR2, None)
            .power,
        Uint128::from(20u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(70u128)
    );
    assert_eq!(get_balance(&app, ADDR2), Uint128::zero());

    // Can't unstake more than you have staked
    let info = mock_info(ADDR2, &[]);
    let _err = staking
        .unstake(&mut app, &info.sender, Uint128::new(100))
        .unwrap_err();

    // Successful unstake
    let info = mock_info(ADDR2, &[]);
    let _res = staking
        .unstake(&mut app, &info.sender, Uint128::new(10))
        .unwrap();
    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR2, None)
            .power,
        Uint128::from(10u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(60u128)
    );
    assert_eq!(get_balance(&app, ADDR2), Uint128::from(10u128));

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(50u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(30u128));
}

#[test]
fn text_max_claims() {
    let mut app = mock_app();
    let amount1 = Uint128::from(MAX_CLAIMS + 1);
    let unstaking_blocks = 1u64;
    let initial_balances = vec![(ADDR1, amount1.u128())];
    let staking = setup_test_case(
        &mut app,
        initial_balances,
        Some(Duration::Height(unstaking_blocks)),
    );

    let info = mock_info(ADDR1, &[]);
    staking
        .stake(&mut app, &info.sender, coin(amount1.u128(), DENOM))
        .unwrap();

    // Create the max number of claims
    for _ in 0..MAX_CLAIMS {
        staking
            .unstake(&mut app, &info.sender, Uint128::new(1))
            .unwrap();
    }

    // Additional unstaking attempts ought to fail.
    staking
        .unstake(&mut app, &info.sender, Uint128::new(1))
        .unwrap_err();

    // Clear out the claims list.
    app.update_block(next_block);
    staking.claim(&mut app, &info.sender).unwrap();

    // Unstaking now allowed again.
    staking
        .unstake(&mut app, &info.sender, Uint128::new(1))
        .unwrap();
    app.update_block(next_block);
    staking.claim(&mut app, &info.sender).unwrap();

    assert_eq!(get_balance(&app, ADDR1), amount1);
}

#[test]
fn test_unstaking_with_claims() {
    let mut app = mock_app();
    let amount1 = Uint128::from(100u128);
    let unstaking_blocks = 10u64;
    let initial_balances = vec![(ADDR1, amount1.u128())];
    let staking = setup_test_case(
        &mut app,
        initial_balances,
        Some(Duration::Height(unstaking_blocks)),
    );

    let info = mock_info(ADDR1, &[]);

    // Successful bond
    let _res = staking
        .stake(&mut app, &info.sender, coin(50, DENOM))
        .unwrap();
    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(50u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(50u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(50u128));

    // Unstake
    let info = mock_info(ADDR1, &[]);
    let _res = staking
        .unstake(&mut app, &info.sender, Uint128::new(10))
        .unwrap();
    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(40u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(40u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(50u128));

    // Cannot claim when nothing is available
    let info = mock_info(ADDR1, &[]);
    let _err: ContractError = staking
        .claim(&mut app, &info.sender)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(_err, ContractError::NothingToClaim {});

    // Successful claim
    app.update_block(|b| b.height += unstaking_blocks);
    let info = mock_info(ADDR1, &[]);
    let _res = staking.claim(&mut app, &info.sender).unwrap();
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(40u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(40u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(60u128));

    // Unstake and claim multiple
    let _info = mock_info(ADDR1, &[]);
    let info = mock_info(ADDR1, &[]);
    let _res = staking
        .unstake(&mut app, &info.sender, Uint128::new(5))
        .unwrap();
    app.update_block(next_block);

    let _info = mock_info(ADDR1, &[]);
    let info = mock_info(ADDR1, &[]);
    let _res = staking
        .unstake(&mut app, &info.sender, Uint128::new(5))
        .unwrap();
    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(30u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(30u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(60u128));

    app.update_block(|b| b.height += unstaking_blocks);
    let info = mock_info(ADDR1, &[]);
    let _res = staking.claim(&mut app, &info.sender).unwrap();
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(30u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(30u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(70u128));
}

#[test]
fn multiple_address_staking() {
    let amount1 = Uint128::from(100u128);
    let initial_balances = vec![
        (ADDR1, amount1.u128()),
        (ADDR2, amount1.u128()),
        (ADDR3, amount1.u128()),
        (ADDR4, amount1.u128()),
    ];

    let mut app = mock_app();
    let amount1 = Uint128::from(100u128);
    let unstaking_blocks = 10u64;
    let staking = setup_test_case(
        &mut app,
        initial_balances,
        Some(Duration::Height(unstaking_blocks)),
    );

    for addr in &[ADDR1, ADDR2, ADDR3, ADDR4] {
        let info = mock_info(*addr, &[]);
        // Successful bond
        let _res = staking
            .stake(&mut app, &info.sender, coin(amount1.u128(), DENOM))
            .unwrap();
        app.update_block(next_block);

        assert_eq!(
            staking
                .query_staked_balance_at_height(&app, *addr, None)
                .power,
            amount1
        );
        assert_eq!(get_balance(&app, *addr), Uint128::zero())
    }
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        amount1.checked_mul(Uint128::new(4)).unwrap()
    );
}

#[test]
fn test_auto_compounding_staking() {
    let mut app = mock_app();
    let amount1 = Uint128::from(1000u128);
    let initial_balances = vec![(ADDR1, amount1.u128())];
    let staking = setup_test_case(&mut app, initial_balances, None);

    let info = mock_info(ADDR1, &[]);

    // Successful bond
    let amount = Uint128::new(100);
    staking
        .stake(&mut app, &info.sender, coin(amount.u128(), DENOM))
        .unwrap();
    app.update_block(next_block);
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1.to_string(), None)
            .power,
        Uint128::from(100u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(100u128),
    );
    assert_eq!(
        staking.query_staked_value(&app, ADDR1.to_string()).value,
        Uint128::from(100u128)
    );
    assert_eq!(
        staking.query_total_value(&app).total,
        Uint128::from(100u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(900u128));

    // Add compounding rewards
    let _res = staking
        .fund(&mut app, &info.sender, coin(100, DENOM))
        .unwrap();
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1.to_string(), None)
            .power,
        Uint128::from(100u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(100u128)
    );
    assert_eq!(
        staking.query_staked_value(&app, ADDR1.to_string()).value,
        Uint128::from(200u128)
    );
    assert_eq!(
        staking.query_total_value(&app).total,
        Uint128::from(200u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(800u128));

    // Sucessful transfer of unbonded amount
    let msg = BankMsg::Send {
        to_address: ADDR2.to_string(),
        amount: coins(100, DENOM),
    };
    let _res = (&mut app).execute(info.sender, msg.into()).unwrap();

    assert_eq!(get_balance(&app, ADDR1), Uint128::from(700u128));
    assert_eq!(get_balance(&app, ADDR2), Uint128::from(100u128));

    // Addr 2 successful bond
    let info = mock_info(ADDR2, &[]);
    staking
        .stake(&mut app, &info.sender, coin(100, DENOM))
        .unwrap();

    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR2, None)
            .power,
        Uint128::from(50u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(150u128)
    );
    assert_eq!(
        staking.query_staked_value(&app, ADDR2.to_string()).value,
        Uint128::from(100u128)
    );
    assert_eq!(
        staking.query_total_value(&app).total,
        Uint128::from(300u128)
    );
    assert_eq!(get_balance(&app, ADDR2), Uint128::zero());

    // Can't unstake more than you have staked
    let info = mock_info(ADDR2, &[]);
    let _err = staking
        .unstake(&mut app, &info.sender, Uint128::new(51))
        .unwrap_err();

    // Add compounding rewards
    let _res = staking
        .fund(&mut app, &Addr::unchecked(ADDR1), coin(90, DENOM))
        .unwrap();

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(100u128)
    );
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR2, None)
            .power,
        Uint128::from(50u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(150u128)
    );
    assert_eq!(
        staking.query_staked_value(&app, ADDR1.to_string()).value,
        Uint128::from(260u128)
    );
    assert_eq!(
        staking.query_staked_value(&app, ADDR2.to_string()).value,
        Uint128::from(130u128)
    );
    assert_eq!(
        staking.query_total_value(&app).total,
        Uint128::from(390u128)
    );
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(610u128));

    // Successful unstake
    let info = mock_info(ADDR2, &[]);
    let _res = staking
        .unstake(&mut app, &info.sender, Uint128::new(25))
        .unwrap();
    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR2, None)
            .power,
        Uint128::from(25u128)
    );
    assert_eq!(
        staking.query_total_staked_at_height(&app, None).power,
        Uint128::from(125u128)
    );
    assert_eq!(get_balance(&app, ADDR2), Uint128::from(65u128));
}

#[test]
fn test_simple_unstaking_with_duration() {
    let mut app = mock_app();
    let amount1 = Uint128::from(100u128);
    let initial_balances = vec![(ADDR1, amount1.u128()), (ADDR2, amount1.u128())];
    let staking = setup_test_case(&mut app, initial_balances, Some(Duration::Height(1)));

    // Bond Address 1
    let info = mock_info(ADDR1, &[]);
    let amount = Uint128::new(100);
    staking
        .stake(&mut app, &info.sender, coin(amount.u128(), DENOM))
        .unwrap();

    // Bond Address 2
    let info = mock_info(ADDR2, &[]);
    let amount = Uint128::new(100);
    staking
        .stake(&mut app, &info.sender, coin(amount.u128(), DENOM))
        .unwrap();
    app.update_block(next_block);
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(100u128)
    );
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(100u128)
    );

    // Unstake Addr1
    let info = mock_info(ADDR1, &[]);
    let amount = Uint128::new(100);
    staking.unstake(&mut app, &info.sender, amount).unwrap();

    // Unstake Addr2
    let info = mock_info(ADDR2, &[]);
    let amount = Uint128::new(100);
    staking.unstake(&mut app, &info.sender, amount).unwrap();

    app.update_block(next_block);

    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR1, None)
            .power,
        Uint128::from(0u128)
    );
    assert_eq!(
        staking
            .query_staked_balance_at_height(&app, ADDR2, None)
            .power,
        Uint128::from(0u128)
    );

    // Claim
    assert_eq!(
        staking.query_claims(&app, ADDR1).claims,
        vec![Claim {
            amount: Uint128::new(100),
            release_at: AtHeight(12350)
        }]
    );
    assert_eq!(
        staking.query_claims(&app, ADDR2).claims,
        vec![Claim {
            amount: Uint128::new(100),
            release_at: AtHeight(12350)
        }]
    );

    let info = mock_info(ADDR1, &[]);
    staking.claim(&mut app, &info.sender).unwrap();
    assert_eq!(get_balance(&app, ADDR1), Uint128::from(100u128));

    let info = mock_info(ADDR2, &[]);
    staking.claim(&mut app, &info.sender).unwrap();
    assert_eq!(get_balance(&app, ADDR2), Uint128::from(100u128));
}
