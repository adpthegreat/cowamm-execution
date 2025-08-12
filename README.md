https://docs.propellerheads.xyz/tycho/for-dexs/protocol-integration/execution/code-architecture 
https://hackmd.io/vHBDLooJSkmX-GNCxy26KQ
https://github.com/propeller-heads/tycho-simulation/blob/main/src/evm/protocol/vm/adapter_contract.rs - this is how we'll access the orderFromSellAmount in the CowAMMHelper contract and get the traded output so that we can send the details to the api, through the ABI

> vm > adapter_contract 

> vm > tycho_simulation_contract

// it'll be 
> vm > helper_contract 

> vm tycho_simulation_contract

//forget all the api responses part, we just need the simulation part with revm extracted out 

// then we get the outputs

// then we create an api execution layer to post it to the api 

//the goal is to make it contract agnostic or cowamm helper specific 

https://github.com/propeller-heads/tycho-simulation/blob/main/src/evm/Readme.md 


https://github.com/cowprotocol/services/tree/eb35a3c47898cf4faae24bd138073e6147c1fd54/crates/contracts all the contracts tests Solver , trader, swapper interaction


https://github.com/cowprotocol/services/blob/eb35a3c47898cf4faae24bd138073e6147c1fd54/crates/cow-amm/src/amm.rs - helper contract sdk - old one , make a new one 

https://github.com/cowprotocol/services/blob/40c8526a3627596ff421aff04a20ee1832b6fb0f/crates/shared/src/interaction.rs

 additional things to do 

- deep dive into cow amm services

- create an api sdk for cow amm endpoints in rust, python, golang 

- create rust version of https://github.com/cowprotocol/watch-tower 

https://github.com/cowprotocol/ethcontract-rs/tree/main/ethcontract-generate 

SOLVER FLOW 

//encode and return interactions for join_pool or exit_pool

//return a valid jit order from sell amount using the cow amm helper module 

//solver integrate interactions and probably other interactions to the generated jit order (settlement)

//encode the settlement 

//post it to the api using the api client and get response 

//DRIVER IS ENTRPOINT TO CODE 
//https://github.com/cowprotocol/services/blob/main/crates/driver/README.md


RPC API Interaction code - useful stuff tbh 
https://github.com/cowprotocol/services/blob/c12eddc78a2b923a10d24e6832a87908509eb4a4/crates/autopilot/src/infra/blockchain/mod.rs#L48