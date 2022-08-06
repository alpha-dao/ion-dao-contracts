use std::borrow::{Borrow, BorrowMut};

use anyhow::Result as AnyResult;
use cosmwasm_std::{coins, Addr, CosmosMsg, Decimal, StdResult, Uint128};
use cw3::Vote;
use cw_multi_test::{AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
use cw_utils::{Duration, Expiration};
use osmo_bindings::{OsmosisMsg, OsmosisQuery};
use osmo_bindings_test::OsmosisApp;

use crate::msg::RangeOrder;
use crate::state::Config;

pub const DEFAULT_DEPOSIT_PERIOD: u64 = 10;
pub const DEFAULT_VOTING_PERIOD: u64 = 15;
pub const DEFAULT_MIN_DEPOSIT: u128 = 10;
pub const DEFAULT_QUO_DEPOSIT: u128 = 100;

pub fn contract_dao() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub fn contract_stake() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        ion_stake::contract::execute,
        ion_stake::contract::instantiate,
        ion_stake::contract::query,
    );
    Box::new(contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    owner: Addr,

    funds: Vec<(Addr, Uint128)>,
    props: Vec<crate::msg::ProposeMsg>,
    staked: Vec<(Addr, Uint128)>,

    gov_token: crate::msg::GovToken,
    threshold: crate::threshold::Threshold,
    periods: (Duration, Duration), // voting, deposit
    deposits: (Uint128, Uint128),  // min, quo
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            owner: Addr::unchecked("owner"),

            funds: vec![],
            props: vec![],
            staked: vec![],

            gov_token: crate::msg::GovToken::Create {
                denom: "denom".to_string(),
                label: "label".to_string(),
                stake_contract_code_id: 0,
                unstaking_duration: Some(Duration::Height(10)),
            },
            threshold: crate::threshold::Threshold {
                threshold: Decimal::percent(50),      // 50%
                quorum: Decimal::percent(33),         // 33%
                veto_threshold: Decimal::percent(33), // 33%
            },
            periods: (
                Duration::Height(DEFAULT_VOTING_PERIOD),
                Duration::Height(DEFAULT_DEPOSIT_PERIOD),
            ),
            deposits: (
                Uint128::new(DEFAULT_MIN_DEPOSIT),
                Uint128::new(DEFAULT_QUO_DEPOSIT),
            ),
        }
    }

    pub fn add_proposal(
        mut self,
        title: impl ToString,
        link: impl ToString,
        desc: impl ToString,
        msgs: Vec<CosmosMsg<OsmosisMsg>>,
    ) -> Self {
        self.props.push(crate::msg::ProposeMsg {
            title: title.to_string(),
            link: link.to_string(),
            description: desc.to_string(),
            msgs,
        });
        self
    }

    pub fn with_funds(mut self, funds: Vec<(impl ToString, u128)>) -> Self {
        self.funds = vec![
            self.funds,
            funds
                .iter()
                .map(|(owner, amount)| (Addr::unchecked(owner.to_string()), Uint128::from(*amount)))
                .collect(),
        ]
        .concat();
        self
    }

    pub fn with_staked(mut self, staked: Vec<(impl ToString, u128)>) -> Self {
        self.staked = vec![
            self.staked,
            staked
                .iter()
                .map(|(owner, amount)| (Addr::unchecked(owner.to_string()), Uint128::from(*amount)))
                .collect(),
        ]
        .concat();
        self
    }

    pub fn with_gov_token(mut self, gov_token: crate::msg::GovToken) -> Self {
        self.gov_token = gov_token;
        self
    }

    pub fn with_threshold(mut self, threshold: crate::threshold::Threshold) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_periods(
        mut self,
        voting_period: Option<Duration>,
        deposit_period: Option<Duration>,
    ) -> Self {
        if let Some(v) = voting_period {
            self.periods.0 = v;
        }

        if let Some(v) = deposit_period {
            self.periods.1 = v;
        }

        self
    }

    pub fn with_deposits(
        mut self,
        min_deposit: Option<Uint128>,
        quo_deposit: Option<Uint128>,
    ) -> Self {
        if let Some(v) = min_deposit {
            self.deposits.0 = v;
        }

        if let Some(v) = quo_deposit {
            self.deposits.1 = v;
        }

        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = OsmosisApp::default();

        // store codes
        let stake = contract_stake();
        let stake_id = app.borrow_mut().store_code(stake);

        let dao = contract_dao();
        let dao_id = app.borrow_mut().store_code(dao);

        let gov_token = match self.gov_token {
            crate::msg::GovToken::Create {
                denom,
                label,
                unstaking_duration,
                ..
            } => crate::msg::GovToken::Create {
                denom,
                label,
                stake_contract_code_id: stake_id,
                unstaking_duration,
            },
            _ => self.gov_token,
        };

        let dao_addr = app
            .borrow_mut()
            .instantiate_contract(
                dao_id,
                self.owner.clone(),
                &crate::msg::InstantiateMsg {
                    name: "dao".to_string(),
                    description: "desc".to_string(),
                    gov_token,
                    threshold: self.threshold,
                    voting_period: self.periods.0,
                    deposit_period: self.periods.1,
                    proposal_deposit_amount: self.deposits.1,
                    proposal_deposit_min_amount: self.deposits.0,
                },
                &[],
                "dao",
                None,
            )
            .unwrap();

        let config: crate::msg::ConfigResponse = app
            .borrow()
            .wrap()
            .query_wasm_smart(&dao_addr, &crate::msg::QueryMsg::GetConfig {})
            .unwrap();

        let mut suite = Suite {
            app,
            dao: dao_addr,
            stake: config.staking_contract,
            denom: config.gov_token,
        };

        suite.app().next_block();

        // funds
        for (owner, amount) in self.funds.iter() {
            suite.sudo_mint(owner, *amount).unwrap();
        }

        suite.app().next_block();

        // staked
        for (owner, amount) in self.staked.iter() {
            suite.sudo_mint(owner, *amount).unwrap();
            suite.stake(owner.as_str(), amount.u128()).unwrap();
        }

        suite.app().next_block();

        // proposals
        for propose_msg in self.props {
            suite
                .sudo_mint(self.owner.as_str(), self.deposits.1)
                .unwrap();

            suite
                .propose(
                    self.owner.as_str(),
                    propose_msg.title,
                    propose_msg.link,
                    propose_msg.description,
                    propose_msg.msgs,
                    Some(self.deposits.1.u128()),
                )
                .unwrap();
        }

        suite.app().next_block();

        suite
    }
}

pub struct Suite {
    app: OsmosisApp,
    pub dao: Addr,
    pub stake: Addr,
    pub denom: String,
}

#[allow(dead_code)]
impl Suite {
    pub fn new(app: OsmosisApp, dao: Addr, denom: impl Into<String>) -> Self {
        let mut suite = Self {
            app,
            dao,
            stake: Addr::unchecked(""),
            denom: denom.into(),
        };

        let config = suite.query_config().unwrap();
        suite.stake = config.staking_contract;

        suite
    }

    pub fn app(&mut self) -> &mut OsmosisApp {
        &mut self.app
    }

    pub fn check_balance(&self, owner: impl ToString, amount: u128) -> bool {
        let denom = self.denom.clone();
        let balance = self
            .app
            .wrap()
            .query_balance(owner.to_string(), denom)
            .unwrap();

        balance.amount.u128() == amount
    }

    /***
     * SUDO CONTRACT ACTIONS
     */

    fn sudo_mint(&mut self, owner: impl ToString, amount: Uint128) -> AnyResult<AppResponse> {
        self.app.borrow_mut().sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: owner.to_string(),
            amount: coins(amount.u128(), &self.denom),
        }))
    }

    /***
     * STAKING CONTRACT ACTIONS
     */

    pub fn stake(&mut self, owner: &str, amount: impl Into<u128>) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(owner),
            self.stake.clone(),
            &ion_stake::msg::ExecuteMsg::Stake {},
            coins(amount.into(), &self.denom).as_slice(),
        )
    }

    pub fn unstake(&mut self, owner: &str, amount: impl Into<Uint128>) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(owner),
            self.stake.clone(),
            &ion_stake::msg::ExecuteMsg::Unstake {
                amount: amount.into(),
            },
            &[],
        )
    }

    pub fn fund(&mut self, owner: &str, amount: impl Into<u128>) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(owner),
            self.stake.clone(),
            &ion_stake::msg::ExecuteMsg::Fund {},
            coins(amount.into(), &self.denom).as_slice(),
        )
    }

    pub fn claim(&mut self, owner: &str) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(owner),
            self.stake.clone(),
            &ion_stake::msg::ExecuteMsg::Claim {},
            &[],
        )
    }

    /***
     * DAO CONTRACT ACTIONS
     */

    pub fn propose(
        &mut self,
        proposer: impl ToString,
        title: impl ToString,
        link: impl ToString,
        desc: impl ToString,
        msgs: Vec<CosmosMsg<OsmosisMsg>>,
        deposit: Option<u128>,
    ) -> AnyResult<AppResponse> {
        let funds = deposit
            .map(|amount| coins(amount, &self.denom))
            .unwrap_or_default();

        self.app.borrow_mut().execute_contract(
            Addr::unchecked(proposer.to_string()),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::Propose(crate::msg::ProposeMsg {
                title: title.to_string(),
                link: link.to_string(),
                description: desc.to_string(),
                msgs,
            }),
            funds.as_slice(),
        )
    }

    pub fn deposit(
        &mut self,
        depositor: &str,
        proposal_id: u64,
        amount: Option<u128>,
    ) -> AnyResult<AppResponse> {
        let funds = amount
            .map(|amount| coins(amount, &self.denom))
            .unwrap_or_default();

        self.app.borrow_mut().execute_contract(
            Addr::unchecked(depositor),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::Deposit { proposal_id },
            funds.as_slice(),
        )
    }

    pub fn claim_deposit(&mut self, claimer: &str, proposal_id: u64) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(claimer),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::ClaimDeposit { proposal_id },
            &[],
        )
    }

    pub fn vote(&mut self, voter: &str, proposal_id: u64, option: Vote) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(voter),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::Vote(crate::msg::VoteMsg {
                proposal_id,
                vote: option,
            }),
            &[],
        )
    }

    pub fn execute_proposal(&mut self, executor: &str, proposal_id: u64) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(executor),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::Execute { proposal_id },
            &[],
        )
    }

    pub fn close_proposal(&mut self, closer: &str, proposal_id: u64) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(closer),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::Close { proposal_id },
            &[],
        )
    }

    pub fn pause(&mut self, pauser: &str, expiration: Expiration) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(pauser),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::PauseDAO { expiration },
            &[],
        )
    }

    pub fn update_config(&mut self, updater: &str, config: Config) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(updater),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::UpdateConfig(config),
            &[],
        )
    }

    pub fn update_staking_contract(
        &mut self,
        updater: &str,
        staking: Addr,
    ) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(updater),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::UpdateStakingContract {
                new_staking_contract: staking,
            },
            &[],
        )
    }

    pub fn update_token_list(
        &mut self,
        updater: &str,
        to_add: Vec<String>,
        to_remove: Vec<String>,
    ) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            Addr::unchecked(updater),
            self.dao.clone(),
            &crate::msg::ExecuteMsg::UpdateTokenList { to_add, to_remove },
            &[],
        )
    }

    /***
     * DAO CONTRACT QUERIES
     */

    pub fn query_config(&self) -> StdResult<crate::msg::ConfigResponse> {
        self.app
            .borrow()
            .wrap()
            .query_wasm_smart(&self.dao, &crate::msg::QueryMsg::GetConfig {})
    }

    pub fn query_token_list(&self) -> StdResult<crate::msg::TokenListResponse> {
        self.app
            .borrow()
            .wrap()
            .query_wasm_smart(&self.dao, &crate::msg::QueryMsg::TokenList {})
    }

    pub fn query_token_balances(
        &self,
        start: Option<String>,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    ) -> StdResult<crate::msg::TokenBalancesResponse> {
        self.app.borrow().wrap().query_wasm_smart(
            &self.dao,
            &crate::msg::QueryMsg::TokenBalances {
                start,
                limit,
                order,
            },
        )
    }

    pub fn query_proposal(
        &self,
        proposal_id: u64,
    ) -> StdResult<crate::msg::ProposalResponse<OsmosisMsg>> {
        self.app
            .borrow()
            .wrap()
            .query_wasm_smart(&self.dao, &crate::msg::QueryMsg::Proposal { proposal_id })
    }

    pub fn query_proposals(
        &self,
        query: crate::msg::ProposalsQueryOption,
        start: Option<u64>,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    ) -> StdResult<crate::msg::ProposalsResponse<OsmosisMsg>> {
        self.app.borrow().wrap().query_wasm_smart(
            &self.dao,
            &crate::msg::QueryMsg::Proposals {
                query,
                start,
                limit,
                order,
            },
        )
    }

    pub fn query_proposal_count(&self) -> StdResult<u64> {
        self.app
            .borrow()
            .wrap()
            .query_wasm_smart(&self.dao, &crate::msg::QueryMsg::ProposalCount {})
    }

    pub fn query_vote(&self, proposal_id: u64, voter: &str) -> StdResult<crate::msg::VoteResponse> {
        self.app.borrow().wrap().query_wasm_smart(
            &self.dao,
            &crate::msg::QueryMsg::Vote {
                proposal_id,
                voter: voter.into(),
            },
        )
    }

    pub fn query_votes(
        &self,
        proposal_id: u64,
        start: Option<String>,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    ) -> StdResult<crate::msg::VotesResponse> {
        self.app.borrow().wrap().query_wasm_smart(
            &self.dao,
            &crate::msg::QueryMsg::Votes {
                proposal_id,
                start,
                limit,
                order,
            },
        )
    }

    pub fn query_deposit(
        &self,
        proposal_id: u64,
        depositor: &str,
    ) -> StdResult<crate::msg::DepositResponse> {
        self.app.borrow().wrap().query_wasm_smart(
            &self.dao,
            &crate::msg::QueryMsg::Deposit {
                proposal_id,
                depositor: depositor.to_string(),
            },
        )
    }

    pub fn query_deposits(
        &self,
        query: crate::msg::DepositsQueryOption,
        limit: Option<u32>,
        order: Option<RangeOrder>,
    ) -> StdResult<crate::msg::DepositsResponse> {
        self.app.borrow().wrap().query_wasm_smart(
            &self.dao,
            &crate::msg::QueryMsg::Deposits {
                query,
                limit,
                order,
            },
        )
    }
}
