import { MsgSubmitProposalEncodeObject } from "@cosmjs/stargate";
import { join } from "path";
import { cosmwasm, cosmos } from "osmojs";
import { readFileSync } from "fs";

import Config from "./config";

async function main() {
  const cfg = await Config.new();
  const [account] = await cfg.wallet.getAccounts();

  const basePath = join(__dirname, "../artifacts");

  const encodedProposalMsg = cosmwasm.wasm.v1.StoreCodeProposal.encode({
    title: "Test Proposal",
    description: "Test Description",
    instantiatePermission: {
      address: "",
      permission: cosmwasm.wasm.v1.AccessType.ACCESS_TYPE_NOBODY,
    },
    runAs: account.address,
    wasmByteCode: readFileSync(join(basePath, "ion_stake.wasm")),
  }).finish();

  const proposeResp = await cfg.stargate.signAndBroadcast(
    account.address,
    [
      cosmos.gov.v1beta1.MessageComposer.withTypeUrl.submitProposal({
        content: {
          typeUrl: "/cosmwasm.wasm.v1.StoreCodeProposal",
          value: encodedProposalMsg,
        },
        // initialDeposit: [{ amount: `${500 * 1e6}`, denom: "uosmo" }],
        initialDeposit: [{ amount: `${1 * 1e6}`, denom: "uosmo" }],
        proposer: account.address,
      }),
    ] as [MsgSubmitProposalEncodeObject],
    "auto"
  );

  console.log(proposeResp.transactionHash);
}

main()
  .then(() => console.log("Done"))
  .catch(console.error);
