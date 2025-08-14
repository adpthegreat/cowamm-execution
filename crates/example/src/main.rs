use std::sync::LazyLock;
use ethcontract::{Address, H256, H160, U256, Bytes};
use hex;
use {
    cow_amm::helper::Amm,
    interactions::{join_pool::JoinPoolInteraction, exit_pool::ExitPoolInteraction},
    contracts::{contract, BCowPool, BCowHelper},
};

[tokio::main]
async fn main() {
    //join pool and exit pool encoding examples 
    let addr: Address = Address::from_slice(&hex::decode("0x9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1").unwrap());
    let helper_addr: Address = Address::from_slice(&hex::decode("0x3FF0041A614A9E6Bf392cbB961C97DA214E9CB31").unwrap());

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

    let amm = Amm::new(addr, &binding);

    // Get tokens traded by this AMM
    let tokens = amm.traded_tokens();
    println!("Traded tokens: {:?}", tokens);

    sell_amount
    let sell_amount = U256::from(1_000_000_000u64);

    // Get a template order
    let template = amm.template_order_from_sell_amount(sell_amount).await?;

    // You can now use template.order, template.signature, etc.
    println!("Order: {:?}", template.order);
    println!("Signature: {:?}", template.signature);
    println!("Pre interactions: {:?}", template.pre_interactions);
    println!("Post interactions: {:?}", template.post_interactions);

    //swap in cowamm (JitOrder) then join 

    //exit first then swap superfluous (JitOrder)

    //post order to api 

}
