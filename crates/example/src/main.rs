use std::sync::LazyLock;
use ethcontract::{Address, H256, H160, U256, Bytes};
use chrono::Utc;
use ethrpc::http::HttpTransport;
use std::time::Duration;
use std::str::FromStr;
use hex;
use chrono::TimeZone;
use {
    app_data::AppDataHash,
    model::{
        order::{
            BuyTokenDestination, Interactions, Order, OrderClass, OrderCreation,
            OrderCreationAppData, OrderData, OrderKind, OrderMetadata, OrderStatus,
            OrderUid, SellTokenSource,
        },
        interaction::InteractionData,
        signature::{EcdsaSignature, EcdsaSigningScheme}
    },
    num::BigUint,
    cow_amm::helper::Amm,
    interactions::{join_pool::JoinPoolInteraction, exit_pool::ExitPoolInteraction},
    contracts::{contract, BCowPool, BCowHelper},
    api_client::{
        client::OrderBookApi,
        urls::MAINNET_PROD
    }
};
use reqwest::{Client, Url};

use tycho_simulation::evm::protocol;

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
    template.post_interactions.push(
        InteractionData {
            target: encoded_join.0,
            value: encoded_join.1,
            call_data: encoded_join.2.0.into()
        }
    );

    //exit first then swap (JitOrder) superfluous token
    template.pre_interactions.push( 
        InteractionData {
            target: encoded_exit.0,
            value: encoded_exit.1,
            call_data: encoded_exit.2.0.into(),
    });

    let client = Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .unwrap();

    //post order to api 
    let ob_api = OrderBookApi::new(client, MAINNET_PROD);
    
     //https://github.com/cowprotocol/services/blob/d884bbe4db35f6d48f53cfeef856a72d7f50d302/crates/model/src/order.rs#L48
    let signing_scheme = EcdsaSigningScheme::Eip712;

    let order_creation = OrderCreation {
        sell_token: H160::from_low_u64_be(1), // You'll need to set this properly
        buy_token: H160::from_low_u64_be(2), // You'll need to set this properly
        receiver: None,
        sell_amount: U256::from(0),
        buy_amount: U256::from(0),
        valid_to: 0,
        app_data: OrderCreationAppData::Hash {
            hash: AppDataHash([0x44; 32]),
        },
        fee_amount: U256::from(1),
        kind: OrderKind::Sell, // Or Buy, depending on your order
        partially_fillable: false,
        sell_token_balance: SellTokenSource::Erc20,
        buy_token_balance: BuyTokenDestination::Erc20,
        from: Some(H160::from_low_u64_be(1)),
        signature: EcdsaSignature {
            v: 1,
            r: H256::from_str(
                "0200000000000000000000000000000000000000000000000000000000000003",
            )
            .unwrap(),
            s: H256::from_str(
                "0400000000000000000000000000000000000000000000000000000000000005",
            )
            .unwrap(),
        }
        .to_signature(EcdsaSigningScheme::Eip712), // Use the appropriate signing scheme
        quote_id: None,
    };

    let _ = ob_api.create_order(&order_creation).await.unwrap();
}
