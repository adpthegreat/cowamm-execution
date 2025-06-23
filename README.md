method 1 - code that uses revm and the tycho simulation layer to simulate the cow amm helper contract trade generation 

reason is for it to be in sync with every other thing, chatgpt explained we have to , initialize storage for the pool, tokens amm state in general 


method 2 - or better still, we use binding extensions and instantiate it with web3 , then we execute the code and return the trade, and every other thing we need 











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


https://github.com/cowprotocol/trading-bot/tree/main 