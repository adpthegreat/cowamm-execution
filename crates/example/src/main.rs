use std::sync::LazyLock;
use ethcontract::{Address, H256, H160, U256, Bytes};
use ethrpc::http::HttpTransport;
use std::time::Duration;
use hex;

use {
    cow_amm::helper::Amm,
    interactions::{join_pool::JoinPoolInteraction, exit_pool::ExitPoolInteraction},
    contracts::{contract, BCowPool, BCowHelper},
    api_client::{
        client::OrderBookApi,
        urls::MAINNET_PROD
    }
};
use reqwest::{Client, Url};

#[tokio::main]
async fn main() {
    //join pool and exit pool encoding examples 
    let addr: Address = Address::from_slice(&hex::decode("9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1").unwrap());
    let helper_addr: Address = Address::from_slice(&hex::decode("3FF0041A614A9E6Bf392cbB961C97DA214E9CB31").unwrap());

    let sell_token: Address = Address::from_slice(&hex::decode("7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0").unwrap()); //wstETH

     let exit_pool_interaction = ExitPoolInteraction {
            b_cow_pool: contract!(BCowPool, addr.clone()), 
            pool_amount_in: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
            min_amounts_out: vec![
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
            ],
    };

    let encoded_exit = exit_pool_interaction.encode_exit();

    let join_pool_interaction = JoinPoolInteraction {
            b_cow_pool: contract!(BCowPool, addr),
            pool_amount_out: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
            max_amounts_in: vec![
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
            ],
    };

    let encoded_join = join_pool_interaction.encode_join();

    let binding = contract!(BCowHelper, helper_addr);

    let amm = Amm::new(addr, &binding).await.unwrap();

    // Get tokens traded by this AMM
    let tokens = amm.traded_tokens();
    println!("Traded tokens: {:?}", tokens);

    let sell_amount = U256::from(1_000_000_000u64);

    // Get a template order
    let mut template = amm.template_order_from_sell_amount(sell_token, sell_amount).await.unwrap();

    //Jit order returned from the amm
    println!("Order: {:?}", template.order);
    println!("Signature: {:?}", template.signature);
    println!("Pre interactions: {:?}", template.pre_interactions);
    println!("Post interactions: {:?}", template.post_interactions);

    //The pre and post interactions are to be encoded into the solvers settlement 

    //swap in cowamm (JitOrder) then join 
    template.post_interactions.push(encoded_join.into());

    //exit first then swap (JitOrder) superfluous token
    template.pre_interactions.push(encoded_exit.into());

    let client = Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .unwrap();

    //post order to api 
    let ob_api = OrderBookApi::new(client, MAINNET_PROD);
    
}
