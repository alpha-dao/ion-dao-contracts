## Summary
The **ION DAO** has successfully gone through testnet and is now ready to launch onto the Osmosis mainnet.

Vote **YES** on this proposal if you want the bytecodes of ION Staking contract (as part of ION DAO) to be put onto Osmosis.  
Vote **NO** on this proposal if you do not want the bytecodes of ION Staking contract (as part of ION DAO) to go on chain.

Checksum of the contracts are following.

### Compiled with Beaker
```
# Intermediate
ccecaa8f25a986b43e4d5f7702fb5304957fe8c1ad3599342cc30ff80312c89e  target/wasm32-unknown-unknown/release/ion_dao.wasm
29ec9b4fdf84958aeff5790ef1d0902e44544bf243028eec589a0d85e53b401b  target/wasm32-unknown-unknown/release/ion_stake.wasm

# Optimized
c216dfad229b8b06f75f8adb044926eb0c250441a46873f5b5cf7caa153e55f1  ion_dao.wasm
9306b0ce4aa88ab4e0750bbc42226231c423b8d36ca14da752343a873dbc749a  ion_stake.wasm
```

### Compiled with Docker
```
# Intermediate
ccecaa8f25a986b43e4d5f7702fb5304957fe8c1ad3599342cc30ff80312c89e  target/wasm32-unknown-unknown/release/ion_dao.wasm
29ec9b4fdf84958aeff5790ef1d0902e44544bf243028eec589a0d85e53b401b  target/wasm32-unknown-unknown/release/ion_stake.wasm

# Optimized
c216dfad229b8b06f75f8adb044926eb0c250441a46873f5b5cf7caa153e55f1  ion_dao.wasm
9306b0ce4aa88ab4e0750bbc42226231c423b8d36ca14da752343a873dbc749a  ion_stake.wasm
```

## Context

The initial ION treasury will consist of the 16572 IONs clawed back from never-active accounts from the Osmosis genesis airdrop. In Prop 120, the Osmosis DAO agreed that the Osmosis Community pool should transfer these to the ION treasury when possible and safe. The activation of the ION DAO contracts on Osmosis mainnet will serve as the trigger for this transfer.  
On testnet, about 258 IONs(almost 1.5%) were staked on the test net where it was challenging to receive faucets, 640 suggestions were made for a test, and community members shared 980 ideas and suggestions and bug reports.

Website: [https://ion.wtf](https://ion.wtf)  
Commonwealth Thread: [https://commonwealth.im/osmosis/discussion/4956-proposal-put-ion-dao-contracts-on-chain-on-osmosis](https://commonwealth.im/osmosis/discussion/4956-proposal-put-ion-dao-contracts-on-chain-on-osmosis)

## Contract
The ION DAO contracts were written with reference to similar DAO DAO contracts.

### Voting

These contracts allow ION tokens to be staked for voting power (and potentially for staking rewards, though ION has no inflation).

### Treasury

The contracts also allow for ION voting power to control a treasury by electing wallets to a multi-sig

### CosmWasm contract safety

The similar DAO DAO contracts have been safely used on Juno for a number of months. The ION DAO contracts do not tie into the Osmosis AMM and are unable to interfere with the chain logic. They have been audited by the Manythings Team. Any potential undiscovered smart contract bugs have no way of affecting OSMO because it is not custodied by the contracts. Only ION would be affected.

## Scope of Implementation

### ION Landing Page
* Develop ION’s landing page using 3D modeling and Three.js (React-fiber)
* Provide a wasm-based token balance checking feature

### ION Staking Page
* Develop wasm-based staking, unstaking, and staked balance features
* Develop total ION balance tracking feature (to be converted to governance treasury after contract integration)
* Visualize transaction maximum gas fee
* Unstake ION (14 days but in testnet 30min)

### ION Proposal List Page
* Visualize the registered proposal list
* Visualize the title, description, voting period, and voting current status for each proposal
* Can propose after staking ION. The minimum deposit for submission: is 0.05 ION, Max deposit for proposal open: is 0.5 ION. (But in the testnet, its min deposit: 50uion, max deposit: 100uion)
* Deposit another proposal that is in Pending(=Deposit period) status

### ION Voting Page
* Voting (for 7 days but in testnet 10 min)
* Enable identification of the proposal creator and its description
* Visualize proposal messages (send, execute, burn, etc.)
* The quorum is 50%, and the Threshold is 50%

## About MANYTHINGS
MANYTHINGS is a “Builder-oriented DAO” based in South Korea. At large, we strive to persuade talented Web2.0 builders to join us and pioneer the Web3.0 movement. Various organizations under the MANYTHINGS umbrella, including Alphaworks, are already creating their presence in the NFT space.  
In developing the ION DAO, Alphaworks aims to set Osmosis’s vibrant activity as our milestone. Furthermore, by contributing to the ION ecosystem in the future, we plan to actively collaborate on Osmosis to solidify it as a leading layer 1 protocol.

Team website: [Manythings](https://manythings.xyz/)