import {
  Addr,
  Proposal_1,
  RangeOrder,
  Status,
  Votes_1,
  Vote_1,
} from "./shared-types";

export type QueryMsg =
  | GetConfig
  | TokenList
  | TokenBalances
  | Proposal_1
  | Proposals
  | ProposalCount
  | Vote_1
  | Votes_1
  | Deposit
  | Deposits;
export type ProposalsQueryOption =
  | {
      find_by_status: {
        status: Status;
        [k: string]: unknown;
      };
    }
  | {
      find_by_proposer: {
        proposer: Addr;
        [k: string]: unknown;
      };
    }
  | {
      everything: {
        [k: string]: unknown;
      };
    };
export type DepositsQueryOption =
  | {
      find_by_proposal: {
        proposal_id: number;
        start?: string | null;
        [k: string]: unknown;
      };
    }
  | {
      find_by_depositor: {
        depositor: string;
        start?: number | null;
        [k: string]: unknown;
      };
    }
  | {
      everything: {
        /**
         * @minItems 2
         * @maxItems 2
         */
        start?: [number, string] | null;
        [k: string]: unknown;
      };
    };

/**
 * Returns [ConfigResponse]
 *
 * ## Example
 *
 * ```json { "get_config": {} } ```
 */
export interface GetConfig {
  get_config: {
    [k: string]: unknown;
  };
}
/**
 * Queries list of native Tokens associated with the DAO Treasury. Returns [TokenListResponse]
 *
 * ## Example
 *
 * ```json { "token_list": {} } ```
 */
export interface TokenList {
  token_list: {
    [k: string]: unknown;
  };
}
/**
 * Returns [TokenBalancesResponse] All DAO native token Balances
 *
 * ## Example
 *
 * ```json { "token_balances": { "start"?: "uosmo", "limit": 30 | 10, "order": "asc" | "desc" } } ```
 */
export interface TokenBalances {
  token_balances: {
    limit?: number | null;
    order?: RangeOrder | null;
    start?: string | null;
    [k: string]: unknown;
  };
}

/**
 * Returns [ProposalsResponse]
 *
 * ## Example
 *
 * ```json { "proposals": { "query": { "find_by_status": { "status": "pending" | .. | "executed" } | "find_by_proposer": { "proposer": "osmo1deadbeef" } | "everything": {} }, "start"?: 10, "limit": 30 | 10, "order": "asc" | "desc" } } ```
 */
export interface Proposals {
  proposals: {
    limit?: number | null;
    order?: RangeOrder | null;
    query: ProposalsQueryOption;
    start?: number | null;
    [k: string]: unknown;
  };
}
/**
 * Returns the number of proposals in the DAO (u64)
 *
 * ## Example
 *
 * ```json { "proposal_count": {} } ```
 */
export interface ProposalCount {
  proposal_count: {
    [k: string]: unknown;
  };
}

/**
 * Queries single deposit info by proposal id & address of depositor. Returns [DepositResponse]
 *
 * ## Example
 *
 * ```json { "deposit": { "proposal_id": 1, "depositor": "osmo1deadbeef" } } ```
 */
export interface Deposit {
  deposit: {
    depositor: string;
    proposal_id: number;
    [k: string]: unknown;
  };
}
/**
 * Queries multiple deposits info by 1. proposal id 2. depositor address Returns [DepositsResponse]
 *
 * ## Example
 *
 * ```json { "deposits": { "query": { "find_by_proposal": { "proposal_id": 1, "start"?: "osmo1deadbeef" } | "find_by_depositor": { "depositor": "osmo1deadbeef", "start"?: 1 } | "everything": { "start"?: [1, "osmo1deadbeef"] } }, "limit": 30 | 10, "order": "asc" | "desc" } } ```
 */
export interface Deposits {
  deposits: {
    limit?: number | null;
    order?: RangeOrder | null;
    query: DepositsQueryOption;
    [k: string]: unknown;
  };
}
