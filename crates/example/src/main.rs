use std::sync::LazyLock;
use ethcontract::{Address, H256, H160, U256};
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
    interactions::{
        encode_cowamm::encode_cowamm,
        join_pool::JoinPoolInteraction, exit_pool::ExitPoolInteraction
    },
    contracts::{contract, BCowPool, BCowHelper},
    api_client::{
        client::OrderBookApi,
        urls::MAINNET_PROD
    }
};
use reqwest::{Client, Url};

use tycho_simulation::{
    evm::protocol::{
        cowamm::state::CowAMMState
    },
    tycho_common::{
        simulation::protocol_sim::ProtocolSim,
        Bytes,
        models::{
            token::Token,
            Chain,
        }
    },
    foundry_evm::revm::primitives::U256 as AlloyU256,
    evm::protocol,
};

#[tokio::main]
async fn main() {
    //join pool and exit pool encoding examples 
    // https://github.com/propeller-heads/tycho-simulation/blob/main/examples/quickstart/main.rs#L533 - reference 

    //simulate with get_amount_out - doesnt panic or hit limit, then it can execute , user can see amount_out before execution 

    //In execution , recalculate interim swaps needed to fulfil order add as post or pre interaction where necessary, then 

    // https://github.com/propeller-heads/tycho-simulation/blob/main/examples/quickstart/main.rs#L543 - reference

    // Also if you would like to know how much limits you can swap on a pool 

    //using a static CowAMMPoolState for the demonstration, ideally we use tycho indexer to fetch all the CowAMM Pools for a given pair, then 
    //its hooked up to Tycho Simulation, then we simulate, then create order 

    let helper_addr: Address = Address::from_slice(&hex::decode("3FF0041A614A9E6Bf392cbB961C97DA214E9CB31").unwrap());

    let token_in = Token::new(
        &Bytes::from_str("0xDEf1CA1fb7FBcDC777520aa7f396b4E015F497aB").unwrap(),
        "COW",
        18,
        0,
        &[Some(1_547_000_000_000_000_000)], //_000
        Chain::Ethereum, 
        100,
    );

    let token_out = Token::new(
        &Bytes::from_str("7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0").unwrap(), //wstETH
        "wstETH",
        18,
        0,
        &[Some(100_000_000_000_000_000)],
        Chain::Ethereum,
        100,
    );

    let lp_token = Token::new(
            &Bytes::from_str("0x9bd702E05B9c97E4A4a3E47Df1e0fe7A0C26d2F1").unwrap(),
            "BCoW-50CoW-50wstETH",
            18,
            0,
            &[Some(199_999_999_999_999_990)],
            Chain::Ethereum,
            100,
    );

    let amount_in = BigUint::from(1000000000000000000 as usize);

    //https://github.com/adpthegreat/tycho-simulation/blob/add_cowamm_simulation/src/evm/protocol/cowamm/state.rs#L59 - cow_amm state fields 
    //https://github.com/adpthegreat/tycho-simulation/blob/add_cowamm_simulation/src/evm/protocol/cowamm/state.rs#L650
    let pool_state = CowAMMState::new(
            Bytes::from("0x9bd702E05B9c97E4A4a3E47Df1e0fe7A0C26d2F1"),
            Bytes::from(token_in.address.clone()),
            Bytes::from(token_out.address.clone()),
            AlloyU256::from_str("1547000000000000000000").unwrap(), //COW liquidity
            AlloyU256::from_str("100000000000000000").unwrap(), //wstETH liquidity
            Bytes::from("0x9bd702E05B9c97E4A4a3E47Df1e0fe7A0C26d2F1"),
            AlloyU256::from_str("199999999999999999990").unwrap(),
            AlloyU256::from_str("1000000000000000000").unwrap(),
            AlloyU256::from_str("1000000000000000000").unwrap(),
            0,
    );

    let amount_out = pool_state
        .get_amount_out(amount_in.clone(), &token_in, &lp_token)
        .map_err(|e| eprintln!("Error calculating amount out for Pool: {e:?}"))
        .ok();

    println!("Amount out result : {:?}", amount_out);

    let binding = contract!(BCowHelper, helper_addr);

    let amm = Amm::new(helper_addr, &binding).await.unwrap(); 

    //returns a template order
    let template = encode_cowamm(amount_in, token_in.address, token_out.address, &pool_state, &amm).await.unwrap();

    // Get tokens traded by this AMM
    let tokens = amm.traded_tokens();
    println!("Traded tokens: {:?}", tokens);

    let sell_amount = U256::from(1_000_000_000u64);
    //Jit order returned from the amm
    println!("Order: {:?}", template.order);
    println!("Signature: {:?}", template.signature);
    println!("Pre interactions: {:?}", template.pre_interactions);
    println!("Post interactions: {:?}", template.post_interactions);

    //The pre and post interactions are to be encoded into the solvers settlement 

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

    //check if state has changed
    // let new_state = res
    //         .new_state
    //         .as_any()
    //         .downcast_ref::<CowAMMState>()
    //         .unwrap();

}
