#!/bin/bash

if [[ -z "$DAO_CONTRACT_CODE_ID" ]]
then
  echo "You should specify DAO_CONTRACT_CODE_ID environment variable"
  exit 1
fi

if [[ -z "$STAKE_CONTRACT_CODE_ID" ]]
then
  echo "You should specify STAKE_CONTRACT_CODE_ID environment variable"
  exit 1
fi

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
  $DAEMON tx gov submit-proposal instantiate-contract "$1" "$2" --label "ION governance contract" --no-admin --title "$3" --description "$4" \
    --run-as "$ACCOUNT_ADDRESS" \
    --from "$ACCOUNT" --keyring-backend "$KEYRING" --chain-id "$CHAIN_ID" -y -b block \
    --gas 20000000 --gas-prices 0.025uosmo --deposit 1600000000uosmo --node "$NODE" --output json
}

##Submit proposal and store wasm binary including a deposit amount
DAO_CONTRACT_INIT_MSG=$(cat ./proposal/init_dao.json | jq -c '.gov_token.create.stake_contract_code_id = '"$STAKE_CONTRACT_CODE_ID")
DAO_PROPOSAL_DESC=$(cat ./proposal/init_dao.md)
DAO_PROPOSAL=$(propose "$DAO_CONTRACT_CODE_ID" "$DAO_CONTRACT_INIT_MSG" "[ION DAO] Initialize" "$DAO_PROPOSAL_DESC")
echo "Proposal for dao contract has been submitted. Prop ID $DAO_PROPOSAL"
$DAEMON query gov proposal "$DAO_PROPOSAL" --node "$NODE" --output json | jq '. | .content.wasm_byte_code = "~~ skipped ~~"'
