export type Vote = "yes" | "no" | "abstain" | "veto";
/**
 * Returns [VoteResponse]
 *
 * ## Example
 *
 * ```json { "vote": { "proposal_id": 1, "voter": "osmo1deadbeef" } } ```
 */
export interface Vote_1 {
  vote: {
    proposal_id: number;
    voter: string;
    [k: string]: unknown;
  };
}
/**
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that the full u128 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(), 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 */
export type Uint128 = string;
/**
 * A point in time in nanosecond precision.
 *
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 *
 * ## Examples
 *
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);
 *
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 */
export type Timestamp = Uint64;
/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u64` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 *
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 */
export type Uint64 = string;
export interface BlockTime {
  [k: string]: unknown;
  height: number;
  time: Timestamp;
}
/**
 * Duration is a delta of time. You can add it to a BlockInfo or Expiration to move that further in the future. Note that an height-based Duration and a time-based Expiration cannot be combined
 */
export type Duration =
  | {
      height: number;
    }
  | {
      time: number;
    };
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
/**
 * A human readable address.
 *
 * In Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no assumptions should be made other than being UTF-8 encoded and of reasonable length.
 *
 * This type represents a validated address. It can be created in the following ways 1. Use `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from JSON. This must only be done from JSON that was validated before such as a contract's state. `Addr` must not be used in messages sent by the user because this would result in unvalidated instances.
 *
 * This type is immutable. If you really need to mutate it (Really? Are you sure?), create a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String` instance.
 */
export type Addr = string;
export interface Config {
  [k: string]: unknown;
  deposit_period: Duration;
  description: string;
  name: string;
  proposal_deposit: Uint128;
  proposal_min_deposit: Uint128;
  threshold: Threshold;
  voting_period: Duration;
}
/**
 * Declares a `quorum` of the total votes that must participate in the election in order for the vote to be considered at all. See `ThresholdResponse.ThresholdQuorum` in the cw3 spec for details.
 */
export interface Threshold {
  [k: string]: unknown;
  quorum: Decimal;
  threshold: Decimal;
  veto_threshold: Decimal;
}
export interface DepositResponse {
  [k: string]: unknown;
  amount: Uint128;
  claimed: boolean;
  depositor: string;
  proposal_id: number;
}
export type CosmosMsgFor_OsmosisMsg =
  | {
      bank: BankMsg;
    }
  | {
      custom: OsmosisMsg;
    }
  | {
      staking: StakingMsg;
    }
  | {
      distribution: DistributionMsg;
    }
  | {
      stargate: {
        type_url: string;
        value: Binary;
        [k: string]: unknown;
      };
    }
  | {
      ibc: IbcMsg;
    }
  | {
      wasm: WasmMsg;
    }
  | {
      gov: GovMsg;
    };
/**
 * The message types of the bank module.
 *
 * See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto
 */
export type BankMsg =
  | {
      send: {
        amount: Coin[];
        to_address: string;
        [k: string]: unknown;
      };
    }
  | {
      burn: {
        amount: Coin[];
        [k: string]: unknown;
      };
    };
/**
 * A number of Custom messages that can call into the Osmosis bindings
 */
export type OsmosisMsg = {
  swap: {
    amount: SwapAmountWithLimit;
    first: Swap;
    route: Step[];
    [k: string]: unknown;
  };
};
export type SwapAmountWithLimit =
  | {
      exact_in: {
        input: Uint128;
        min_output: Uint128;
        [k: string]: unknown;
      };
    }
  | {
      exact_out: {
        max_input: Uint128;
        output: Uint128;
        [k: string]: unknown;
      };
    };
/**
 * The message types of the staking module.
 *
 * See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto
 */
export type StakingMsg =
  | {
      delegate: {
        amount: Coin;
        validator: string;
        [k: string]: unknown;
      };
    }
  | {
      undelegate: {
        amount: Coin;
        validator: string;
        [k: string]: unknown;
      };
    }
  | {
      redelegate: {
        amount: Coin;
        dst_validator: string;
        src_validator: string;
        [k: string]: unknown;
      };
    };
/**
 * The message types of the distribution module.
 *
 * See https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto
 */
export type DistributionMsg =
  | {
      set_withdraw_address: {
        /**
         * The `withdraw_address`
         */
        address: string;
        [k: string]: unknown;
      };
    }
  | {
      withdraw_delegator_reward: {
        /**
         * The `validator_address`
         */
        validator: string;
        [k: string]: unknown;
      };
    };
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
 */
export type Binary = string;
/**
 * These are messages in the IBC lifecycle. Only usable by IBC-enabled contracts (contracts that directly speak the IBC protocol via 6 entry points)
 */
export type IbcMsg =
  | {
      transfer: {
        /**
         * packet data only supports one coin https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
         */
        amount: Coin;
        /**
         * exisiting channel to send the tokens over
         */
        channel_id: string;
        /**
         * when packet times out, measured on remote chain
         */
        timeout: IbcTimeout;
        /**
         * address on the remote chain to receive these tokens
         */
        to_address: string;
        [k: string]: unknown;
      };
    }
  | {
      send_packet: {
        channel_id: string;
        data: Binary;
        /**
         * when packet times out, measured on remote chain
         */
        timeout: IbcTimeout;
        [k: string]: unknown;
      };
    }
  | {
      close_channel: {
        channel_id: string;
        [k: string]: unknown;
      };
    };
/**
 * The message types of the wasm module.
 *
 * See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
 */
export type WasmMsg =
  | {
      execute: {
        contract_addr: string;
        funds: Coin[];
        /**
         * msg is the json-encoded ExecuteMsg struct (as raw Binary)
         */
        msg: Binary;
        [k: string]: unknown;
      };
    }
  | {
      instantiate: {
        admin?: string | null;
        code_id: number;
        funds: Coin[];
        /**
         * A human-readbale label for the contract
         */
        label: string;
        /**
         * msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
         */
        msg: Binary;
        [k: string]: unknown;
      };
    }
  | {
      migrate: {
        contract_addr: string;
        /**
         * msg is the json-encoded MigrateMsg struct that will be passed to the new code
         */
        msg: Binary;
        /**
         * the code_id of the new logic to place in the given contract
         */
        new_code_id: number;
        [k: string]: unknown;
      };
    }
  | {
      update_admin: {
        admin: string;
        contract_addr: string;
        [k: string]: unknown;
      };
    }
  | {
      clear_admin: {
        contract_addr: string;
        [k: string]: unknown;
      };
    };
export type GovMsg = {
  vote: {
    proposal_id: number;
    vote: VoteOption;
    [k: string]: unknown;
  };
};
export type VoteOption = "yes" | "no" | "abstain" | "no_with_veto";
/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration =
  | {
      at_height: number;
    }
  | {
      at_time: Timestamp;
    }
  | {
      never: {
        [k: string]: unknown;
      };
    };
export type Denom =
  | {
      native: string;
    }
  | {
      cw20: Addr;
    };
export interface Coin {
  [k: string]: unknown;
  amount: Uint128;
  denom: string;
}
export interface Swap {
  [k: string]: unknown;
  denom_in: string;
  denom_out: string;
  pool_id: number;
}
export interface Step {
  [k: string]: unknown;
  denom_out: string;
  pool_id: number;
}
/**
 * In IBC each package must set at least one type of timeout: the timestamp or the block height. Using this rather complex enum instead of two timeout fields we ensure that at least one timeout is set.
 */
export interface IbcTimeout {
  [k: string]: unknown;
  block?: IbcTimeoutBlock | null;
  timestamp?: Timestamp | null;
}
/**
 * IBCTimeoutHeight Height is a monotonically increasing data type that can be compared against another Height for the purposes of updating and freezing clients. Ordering is (revision_number, timeout_height)
 */
export interface IbcTimeoutBlock {
  [k: string]: unknown;
  /**
   * block height after which the packet times out. the height within the given revision
   */
  height: number;
  /**
   * the version that the client is currently on (eg. after reseting the chain this could increment 1 as height drops to 0)
   */
  revision: number;
}
export type CosmosMsgFor_Empty =
  | {
      bank: BankMsg;
    }
  | {
      custom: Empty;
    }
  | {
      staking: StakingMsg;
    }
  | {
      distribution: DistributionMsg;
    }
  | {
      stargate: {
        type_url: string;
        value: Binary;
        [k: string]: unknown;
      };
    }
  | {
      ibc: IbcMsg;
    }
  | {
      wasm: WasmMsg;
    }
  | {
      gov: GovMsg;
    };
export type Status = "pending" | "open" | "rejected" | "passed" | "executed";
/**
 * An empty struct that serves as a placeholder in different places, such as contracts that don't set a custom message.
 *
 * It is designed to be expressable in correct JSON and JSON Schema but contains no meaningful data. Previously we used enums without cases, but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
 */
export interface Empty {
  [k: string]: unknown;
}
export interface Votes {
  [k: string]: unknown;
  abstain: Uint128;
  no: Uint128;
  veto: Uint128;
  yes: Uint128;
}
/**
 * Returns [VotesResponse]
 *
 * ## Example
 *
 * ```json { "votes": { "proposal_id": 1, "start"?: "osmo1deadbeef", "limit": 30 | 10, "order": "asc" | "desc" } } ```
 */
export interface Votes_1 {
  votes: {
    limit?: number | null;
    order?: RangeOrder | null;
    proposal_id: number;
    start?: string | null;
    [k: string]: unknown;
  };
}
export interface Proposal {
  [k: string]: unknown;
  deposit_base_amount: Uint128;
  deposit_claimable: boolean;
  deposit_ends_at: Expiration;
  /**
   * Proposal Description
   */
  description: string;
  /**
   * Related link about this proposal
   */
  link: string;
  /**
   * List of messages to execute
   */
  msgs: CosmosMsgFor_OsmosisMsg[];
  /**
   * Address of proposer
   */
  proposer: Addr;
  /**
   * Current status of this proposal
   */
  status: Status;
  /**
   * Starting time / height information
   */
  submitted_at: BlockTime;
  /**
   * Pass requirements
   */
  threshold: Threshold;
  /**
   * Proposal title
   */
  title: string;
  /**
   * Amount of the native governance token required for voting
   */
  total_deposit: Uint128;
  /**
   * The total weight when the proposal started (used to calculate percentages)
   */
  total_weight: Uint128;
  vote_ends_at: Expiration;
  vote_starts_at: BlockTime;
  /**
   * summary of existing votes
   */
  votes: Votes;
}
/**
 * Returns [ProposalResponse]
 *
 * ## Example
 *
 * ```json { "proposal": { "proposal_id": 1 } } ```
 */
export interface Proposal_1 {
  proposal: {
    proposal_id: number;
    [k: string]: unknown;
  };
}
export type RangeOrder = "asc" | "desc";
/**
 * Returns the vote (opinion as well as weight counted) as well as the address of the voter who submitted it
 */
export interface VoteInfo {
  [k: string]: unknown;
  vote: Vote;
  voter: string;
  weight: Uint128;
}
