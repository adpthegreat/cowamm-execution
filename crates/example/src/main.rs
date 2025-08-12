use ethcontract::{Address, H256, H160, U256, Bytes};
use {
    cow_amm::helper::Amm,
    interactions::{join_pool::JoinPoolInteraction, exit_pool::ExitPoolInteraction }
};


fn main() {
    //Settlement encoding example 
    //join pool and exit pool encoding examples 
    //  let interaction = ExitPoolInteraction {
    //         b_cow_pool: 
    //         pool_amount_in: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
    //         min_amounts_out: vec![
    //             U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
    //             U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
    //         ],
    //     };
    
    //   let interaction = JoinPoolInteraction {
    //         b_cow_pool:
    //         pool_amount_out: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
    //         max_amounts_in: vec![
    //             U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
    //             U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
    //         ],
    //     };

    // let address = Address::from("0x9bd702e05b9c97e4a4a3e47df1e0fe7a0c26d2f1");
    // let amm = Amm::new(address, );

    //post order to api example 
}
