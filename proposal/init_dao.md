## Summary
Urgent announcement regarding Proposal 321:

The testnet standard parameter values in the proposal are incorrect due to typo issues. If the proposal is passed as is, the following problems might occur:

1. Short deposit / voting period allows the attacker to rush through the malicious proposal easily.
2. Low proposal deposit makes it easier to spam the DAO.

Therefore, we're submitting a new Proposal #322 with the revised parameters which we agreed with the Osmosis team as below:

Revised parameters
* unstaking period
  * `3600s` -> `1209600s` (= 14 Days)
* quorum
  * `0.3` -> `0.5` (= 50%)
* deposit period
  * `600s` -> `604800s` (= 7 Days)
* voting period
  * `800s` -> `604800s` (= 7 Days)
* proposal deposit
  * `100 uion` -> `500000 uion` (= 0.5 ION)
* minimum proposal deposit
  * `50 uion` -> `50000 uion` (= 0.05 ION)

We would like to request the community to kindly vote NO to Proposal #321, consider #322 as our proposal to instantiate ION DAO and cast your votes accordingly.

Vote **YES** on this proposal if you want to instantiate ION DAO contract on Osmosis with revised parameters.  
Vote **NO** on this proposal if you do not want to.  

## Context

The initial ION treasury will consist of the 16572 IONs clawed back from never-active accounts from the Osmosis genesis airdrop. In Prop 120, the Osmosis DAO agreed that the Osmosis Community pool should transfer these to the ION treasury when possible and safe. The activation of the ION DAO contracts on Osmosis mainnet will serve as the trigger for this transfer.  
About 258 IONs(almost 1.5%) were staked on the testnet where it was challenging to receive faucets, 640 suggestions were made for a test, and community members shared 980 ideas and suggestions and bug reports.  

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
* Develop ION’s landing page using 3D modeling and Three.js (React Fiber)
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
* The quorum is 50%, and the threshold is 50%

## About MANYTHINGS
MANYTHINGS is a “Builder-oriented DAO” based in South Korea. At large, we strive to persuade talented Web2.0 builders to join us and pioneer the Web3.0 movement. Various organizations under the MANYTHINGS umbrella are already creating their presence in the NFT space.  
In developing the ION DAO, manythings aims to set Osmosis’s vibrant activity as our milestone. Furthermore, by contributing to the ION ecosystem in the future, we plan to actively collaborate on Osmosis to solidify it as a leading layer 1 protocol.  

Team website: [Manythings](https://manythings.xyz/)