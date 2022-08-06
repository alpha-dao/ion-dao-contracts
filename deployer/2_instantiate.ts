import { join } from "path";

import Config from "./config";
import { InstantiateResult, UploadResult } from "./types";
import { writeFileSync } from "fs";

import { dao, stake } from "../types/contracts";

async function main() {
  const cfg = await Config.new();
  const [account] = await cfg.wallet.getAccounts();

  console.log(account.address);
  console.log(
    await Promise.all([
      cfg.cosmwasm.getBalance(account.address, "uosmo"),
      cfg.cosmwasm.getBalance(account.address, "uion"),
    ])
  );

  const codes = require(join(__dirname, "1_upload")) as UploadResult;
  const preInit = require(join(
    __dirname,
    "2_instantiate"
  )) as InstantiateResult;

  // instantiate
  const { contractAddress: daoAddress, transactionHash } =
    await cfg.cosmwasm.instantiate(
      account.address,
      codes["ion_dao-aarch64"],
      {
        name: "ION DAO",
        description: "DAO of the ION holders",
        gov_token: preInit["ion_stake-aarch64"]
          ? {
              reuse: {
                stake_contract: preInit["ion_stake-aarch64"],
              },
            }
          : {
              create: {
                denom: "uion",
                label: "ION staking contract for governance",
                stake_contract_code_id: codes["ion_stake-aarch64"],
                unstaking_duration: { time: 3600 },
              },
            },
        threshold: { quorum: "0.3", threshold: "0.5", veto_threshold: "0.3" },
        deposit_period: { time: 600 },
        voting_period: { time: 800 },
        proposal_deposit_amount: "100",
        proposal_deposit_min_amount: "50",
      } as dao.InitMsg,
      "ION governance contract",
      "auto"
    );

  console.log({ contractAddress: daoAddress, transactionHash });

  const queryContractSmart = <T, U>(addr: string, input: T): Promise<U> =>
    cfg.cosmwasm.queryContractSmart(addr, input as any);

  const configResp = await queryContractSmart<dao.QueryMsg, dao.ConfigResponse>(
    daoAddress,
    { get_config: {} }
  );

  const stakeAddress = configResp.staking_contract;

  writeFileSync(
    join(__dirname, "2_instantiate.json"),
    JSON.stringify(
      {
        "ion_dao-aarch64": daoAddress,
        "ion_stake-aarch64": stakeAddress,
      } as InstantiateResult,
      null,
      2
    )
  );
}

main()
  .then(() => console.log("Done"))
  .catch(console.error);
