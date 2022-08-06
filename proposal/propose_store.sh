#!/bin/bash

#Executable
DAEMON=${DAEMON:-"osmosisd"}
#CHAIN ID
CHAIN_ID=${CHAIN_ID:-"osmo-test-4"}
#Signer
ACCOUNT=${ACCOUNT:-"tester"}
#Keyring
KEYRING=${KEYRING:-"test"}
#Node
NODE=${NODE:-"https://testnet-rpc.osmosis.zone:443"}
#Mnemonic
MNEMONIC=${MNEMONIC:-"satisfy adjust timber high purchase tuition stool faith fine install that you unaware feed domain license impose boss human eager hat rent enjoy dawn"} # first default account of local osmosis :)

#Delete account if already present
$DAEMON keys delete "$ACCOUNT" --keyring-backend "$KEYRING" -y
#Mnemonic
echo "$MNEMONIC" | $DAEMON keys add "$ACCOUNT" --keyring-backend "$KEYRING" --recover
#Account Address
ACCOUNT_ADDRESS=$($DAEMON keys show -a "$ACCOUNT" --keyring-backend "$KEYRING")

#Account Address and Balance
echo "Balance for $ACCOUNT_ADDRESS"
$DAEMON query bank balances "$ACCOUNT_ADDRESS" --node "$NODE" --output json | jq -c

propose () {
  $DAEMON tx gov submit-proposal wasm-store "$1" --title "$2" --description "$3" \
    --run-as "$ACCOUNT_ADDRESS" \
    --from "$ACCOUNT" --keyring-backend "$KEYRING" --chain-id "$CHAIN_ID" -y -b block \
    --gas 20000000 --gas-prices 0.025uosmo --deposit 500000000uosmo --node "$NODE" --output json \
    | jq -c '.logs[0].events[] | select(.type | contains("submit_proposal")) | .attributes[] | select(.key | contains("proposal_id")) | .value | tonumber'
}

#Submit proposal and store wasm binary including a deposit amount

STAKE_PROPOSAL_DESC=$(cat ./proposal/store_stake.md)
STAKE_PROPOSAL=$(propose "./artifacts/ion_stake.wasm" "[ION DAO] Store staking contract's code" "$STAKE_PROPOSAL_DESC")
echo "Proposal for staking contract has been submitted. Prop ID $STAKE_PROPOSAL"
$DAEMON query gov proposal "$STAKE_PROPOSAL" --node "$NODE" --output json | jq '. | .content.wasm_byte_code = "~~ skipped ~~"'

DAO_PROPOSAL_DESC=$(cat ./proposal/store_dao.md)
DAO_PROPOSAL=$(propose "./artifacts/ion_dao.wasm" "[ION DAO] Store DAO contract's code" "$DAO_PROPOSAL_DESC")
echo "Proposal for dao contract has been submitted. Prop ID $DAO_PROPOSAL"
$DAEMON query gov proposal "$DAO_PROPOSAL" --node "$NODE" --output json | jq '. | .content.wasm_byte_code = "~~ skipped ~~"'
