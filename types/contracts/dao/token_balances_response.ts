import { Coin } from "./shared-types";

export interface TokenBalancesResponse {
  balances: Coin[];
  [k: string]: unknown;
}
