export type QueryMsg =
  | {
      voting_power_at_height: {
        address: string;
        height?: number | null;
        [k: string]: unknown;
      };
    }
  | {
      total_power_at_height: {
        height?: number | null;
        [k: string]: unknown;
      };
    }
  | {
      staked_value: {
        address: string;
        [k: string]: unknown;
      };
    }
  | {
      total_value: {
        [k: string]: unknown;
      };
    }
  | {
      claims: {
        address: string;
        [k: string]: unknown;
      };
    }
  | {
      range_stakers: {
        limit?: number | null;
        order?: RangeOrder | null;
        start_at?: string | null;
        [k: string]: unknown;
      };
    }
  | {
      get_config: {
        [k: string]: unknown;
      };
    }
  | {
      info: {
        [k: string]: unknown;
      };
    }
  | {
      dao: {
        [k: string]: unknown;
      };
    };
export type RangeOrder = "asc" | "desc";
