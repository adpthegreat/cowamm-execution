## Execution layer for tycho CoWAMM module 
This repo contains the execution code for the cow_amm protocol for the tycho integration which covers with the generation of orders from the cowamm helper contract and the encoding of pool joins and exits with the relevant models for all of them

Most of the code here is gotten from the CoWprotocol [services](https://github.com/cowprotocol/services) with was not needed removed, and only the relevant parts
for encoding, order generation and setttlement 

`ethcontract` is used to generate the rust bindings, when `cargo build` is run, the build.rs in the `contracts` crate generates the rust bindings for solidity contact which can be located in `target/debug/build/contracts-<some-magic-number>/out/<contract-name>.rs`

The `api_client` crate contains an api for interacting with the cow protocol order book, fetching orders, posting orders 

The `contracts` crate contains artifacts for the contracts to generate the rust bindings for the relevant contracts 

The `cow_amm` crate contains the necessary rust helper code for the fetching the orders for the amm in the `helper` folder 

The `ethrpc` crate contains relevant utilities for rpc methods for simulation and sending transaction 

The `example` crate contains an example usage for the encoding pool joins and exits to be used as a pre or post interaction in the settlement 

The `interactions` crate contains the necessary encoded interactions for the pool join, pool exit and other relevant enocdings like one for wrapping eth to weth 

The `shared` crate contains modules used by a lot of components like the `models`, `number` crate for working with numbers, `app_data` contains models for app_data (extra information associated with an order) `interaction.rs` contains the `Interaction` trait for the encoded interactions, and the `utils` crate for commonly used utilties 

The `testlib` crate contains utilities for testing used in other crates 