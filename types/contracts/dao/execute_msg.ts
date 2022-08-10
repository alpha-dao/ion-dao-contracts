import {
  Addr,
  Config,
  CosmosMsgFor_OsmosisMsg,
  Expiration,
  Vote,
} from "./shared-types";

export type ExecuteMsg =
  | {
      propose: ProposeMsg;
    }
  | {
      deposit: {
        proposal_id: number;
        [k: string]: unknown;
      };
    }
  | {
      claim_deposit: {
        proposal_id: number;
        [k: string]: unknown;
      };
    }
  | {
      vote: VoteMsg;
    }
  | {
      execute: {
        proposal_id: number;
        [k: string]: unknown;
      };
    }
  | {
      close: {
        proposal_id: number;
        [k: string]: unknown;
      };
    }
  | {
      pause_d_a_o: {
        expiration: Expiration;
        [k: string]: unknown;
      };
    }
  | {
      update_config: Config;
    }
  | {
      update_token_list: {
        to_add: string[];
        to_remove: string[];
        [k: string]: unknown;
      };
    }
  | {
      update_staking_contract: {
        new_staking_contract: Addr;
        [k: string]: unknown;
      };
    };

export interface ProposeMsg {
  description: string;
  link: string;
  msgs: CosmosMsgFor_OsmosisMsg[];
  title: string;
  [k: string]: unknown;
}

export interface VoteMsg {
  proposal_id: number;
  vote: Vote;
  [k: string]: unknown;
}
