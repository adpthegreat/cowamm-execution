use std::sync::LazyLock;
use ethcontract::{Address, H256, H160, U256, Bytes};
use {
    cow_amm::helper::Amm,
    interactions::{join_pool::JoinPoolInteraction, exit_pool::ExitPoolInteraction},
    contracts::{BCowPool, CowAmmLegacyHelper}
};


fn main() {
    //Settlement encoding example 
    //join pool and exit pool encoding examples 
     let exit_pool_interaction = ExitPoolInteraction {
            b_cow_pool: BCowPool,
            pool_amount_in: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
            min_amounts_out: vec![
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
            ],
    };

    let encoded_exit = exit_pool_interaction.encode_exit();

    let join_pool_interaction = JoinPoolInteraction {
            b_cow_pool: BCowPool,
            pool_amount_out: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
            max_amounts_in: vec![
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
            ],
    };

    let encoded_join = join_pool_interaction.encode_join();

    // let address = Address::from("0x9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1");

    // let amm = Amm::new(address, HELPER_BYTECODE); private?

    // let _ = amm.template_order()

    //post order to api example 
}
