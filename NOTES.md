// https://github.com/cowprotocol/trading-bot/tree/main - make a rust version 

// https://github.com/cowprotocol/services/blob/main/crates/orderbook/src/api/post_order.rs#L340 

//We want to return Interactions for solvers to include in their settlement 

// Generate Interaction for joinPool

// Generate Interaction for exitPool 

// Generate Interaction for cowammHelper contract - orderFromSellAmount (for the JIT order)

//https://github.com/cowprotocol/services/blob/40c8526a3627596ff421aff04a20ee1832b6fb0f/crates/solver/src/solver.rs

//Solver integrates in their settlement example -> https://etherscan.io/tx/0x5cdb9a5fd8532f331d80ef5191b39082460c59f61f242ea14b78d3eaaf09fae8 look at the burning LP token in the middle of everything

//Create a module to integrate in your settlement easily

//Create an API Layer to send the request 

//https://github.com/cowprotocol/services/blob/40c8526a3627596ff421aff04a20ee1832b6fb0f/crates/solvers/src/api/routes/solve/dto/solution.rs - this is more of a solver engine api


//generate order based on your amount, post to the api 

//in that order you can add the exit and join pool as pre interactions 

//we can't return the jit order as an interaction we can only return an order 

