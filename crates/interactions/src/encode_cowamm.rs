use {
    anyhow::{Context, Result},
    ethcontract::{Address, U256},
    contracts::BCowPool,
    model::{
        interaction::InteractionData,
        signature::{EcdsaSignature}
    },
    cow_amm::helper::{
        TemplateOrder, Amm
    },
    crate::{
        join_pool::JoinPoolInteraction, 
        exit_pool::ExitPoolInteraction
    },
    num_bigint::BigUint,
    contracts::contract,
    tycho_simulation::{
        tycho_common::Bytes,
        evm::protocol::{
            // u256_num::biguint_to_u256,
            cowamm::state::CowAMMState
        },
        foundry_evm::revm::primitives::U256 as AlloyU256,
    },
};


/// Encodes a CowAMM swap for the three possible cases:
/// 1. Token A -> Token B (normal swap)
/// 2. Token A -> LP Token (join pool: swap + add liquidity)
/// 3. LP Token -> Token A (exit pool: remove liquidity + swap)
///
/// # Arguments
/// * `amount_in` - Amount of token_in to sell/swap
/// * `token_in` - Address of the input token
/// * `token_out` - Address of the output token
/// * `pool_state` - Current state of the CowAMM pool for off-chain calculations
/// * `amm` - AMM helper contract instance for generating orders
///
/// # Returns
/// A `TemplateOrder` with the main order and appropriate pre/post interactions
pub async fn encode_cowamm(
    amount_in: BigUint,
    token_in: Bytes,
    token_out: Bytes,
    pool_state: &CowAMMState,
    amm: &Amm,
) -> Result<TemplateOrder> {
    // Convert BigUint to U256
    let amount_in_u256 = biguint_to_u256(&amount_in); 
    
    // Convert Bytes to Address
    let token_in_addr = bytes_to_address(&token_in)?;
    let token_out_addr = bytes_to_address(&token_out)?;
    
    // Get pool address and LP token address
    let pool_address = bytes_to_address(&pool_state.address)?;
    let lp_token_addr = bytes_to_address(&pool_state.lp_token)?;
    
    // Determine which case we're handling
    let is_lp_in = token_in == pool_state.lp_token;
    let is_lp_out = token_out == pool_state.lp_token;
    
    match (is_lp_in, is_lp_out) {
        // Case 1: Normal Token A -> Token B swap
        (false, false) => {
            encode_normal_swap(
                amount_in_u256,
                token_in_addr,
                token_out_addr,
                amm,
            ).await
        }
        
        // Case 2: Token A -> LP Token (Join Pool)
        // User sells Token A, gets LP tokens
        // Flow: Swap some Token A for Token B -> Join pool with both tokens
        (false, true) => {
            encode_join_pool_swap(
                amount_in_u256,
                token_in_addr,
                pool_address,
                pool_state,
                amm,
            ).await
        }
        
        // Case 3: LP Token -> Token A (Exit Pool)
        // User sells LP tokens, gets Token A
        // Flow: Exit pool (burn LP, receive both tokens) -> Swap Token B for Token A
        (true, false) => {
            encode_exit_pool_swap(
                amount_in_u256,
                token_out_addr,
                pool_address,
                pool_state,
                amm,
            ).await
        }
        
        // Invalid case: LP Token -> LP Token
        (true, true) => {
            anyhow::bail!("Cannot swap LP token for LP token")
        }
    }
}

/// Case 1: Normal token-to-token swap
/// No pre or post interactions needed
async fn encode_normal_swap(
    amount_in: U256,
    token_in: Address,
    token_out: Address,
    amm: &Amm,
) -> Result<TemplateOrder> {
    // Generate template order for the swap
    let template = amm
        .template_order_from_sell_amount(token_in, amount_in)
        .await
        .context("Failed to generate template order for normal swap")?;
    
    Ok(template)
}

/// Case 2: Token A -> LP Token (Join Pool)
/// Flow:
/// 1. Calculate proportional amounts needed for joining
/// 2. Main order: Swap excess Token A for Token B
/// 3. Post-interaction: Join pool with both tokens
async fn encode_join_pool_swap(
    amount_in: U256,
    token_in: Address,
    pool_address: Address,
    pool_state: &CowAMMState,
    amm: &Amm,
) -> Result<TemplateOrder> {
    // Calculate the proportional amounts of both tokens needed to join the pool
    // This represents what we'll get when we "buy" the LP token amount

    let (proportional_token_a, proportional_token_b) = pool_state
        .calc_tokens_out_given_exact_lp_token_in(ethcontract_to_alloy(amount_in))
        .context("Failed to calculate proportional token amounts")?;
    
    // Determine which token we're swapping and which we need to acquire
    let token_a_addr = bytes_to_address(&pool_state.token_a.0)?;
    let token_b_addr = bytes_to_address(&pool_state.token_b.0)?;
    
    let (swap_sell_token, swap_buy_token, swap_amount) = if token_in == token_a_addr {
        // We have Token A, need to swap for Token B
        (token_in, token_b_addr, proportional_token_b)
    } else {
        // We have Token B, need to swap for Token A
        (token_in, token_a_addr, proportional_token_a)
    };
    
    // Generate the main swap order
    // This swaps the excess token to get the proportional amount of the other token
    let mut template = amm
        .template_order_from_sell_amount(swap_sell_token, alloy_to_ethcontract(swap_amount))
        .await
        .context("Failed to generate swap order for join pool")?;
    
    // Create the join pool interaction as a post-interaction
    let join_interaction = JoinPoolInteraction {
        b_cow_pool: contract!(BCowPool, pool_address),
        pool_amount_out: amount_in, // Amount of LP tokens to mint
        max_amounts_in: vec![
            alloy_to_ethcontract(proportional_token_a), 
            alloy_to_ethcontract(proportional_token_b)
        ],
    };
    
    let encoded_join = join_interaction.encode_join();
    
    // Add join pool as post-interaction
    template.post_interactions.push(InteractionData {
        target: encoded_join.0,
        value: encoded_join.1,
        call_data: encoded_join.2.0.into(),
    });
    
    Ok(template)
}

/// Case 3: LP Token -> Token A (Exit Pool)
/// Flow:
/// 1. Pre-interaction: Exit pool (burn LP tokens, receive both Token A and Token B)
/// 2. Main order: Swap Token B for Token A
async fn encode_exit_pool_swap(
    lp_amount_in: U256,
    token_out: Address,
    pool_address: Address,
    pool_state: &CowAMMState,
    amm: &Amm,
) -> Result<TemplateOrder> {
    // Calculate the proportional amounts of both tokens we'll receive from exiting
    let (proportional_token_a, proportional_token_b) = pool_state
        .calc_tokens_out_given_exact_lp_token_in(ethcontract_to_alloy(lp_amount_in))
        .context("Failed to calculate tokens out for exit")?;
    
    // Determine which token to keep and which to swap
    let token_a_addr = bytes_to_address(&pool_state.token_a.0)?;
    let token_b_addr = bytes_to_address(&pool_state.token_b.0)?;
    
    let (swap_sell_token, swap_amount) = if token_out == token_a_addr {
        // We want Token A, so swap Token B
        (token_b_addr, proportional_token_b)
    } else {
        // We want Token B, so swap Token A
        (token_a_addr, proportional_token_a)
    };
    
    // Generate the main swap order
    // This swaps the unwanted token for more of the desired token
    let mut template = amm
        .template_order_from_sell_amount(swap_sell_token, alloy_to_ethcontract(swap_amount))
        .await
        .context("Failed to generate swap order for exit pool")?;
    
    // Create the exit pool interaction as a pre-interaction
    let exit_interaction = ExitPoolInteraction {
        b_cow_pool: contract!(BCowPool, pool_address),
        pool_amount_in: lp_amount_in, // Amount of LP tokens to burn
        min_amounts_out: vec![
            alloy_to_ethcontract(proportional_token_a), 
            alloy_to_ethcontract(proportional_token_b)
        ],
    };
    
    let encoded_exit = exit_interaction.encode_exit();
    
    // Add exit pool as pre-interaction
    template.pre_interactions.push(InteractionData {
        target: encoded_exit.0,
        value: encoded_exit.1,
        call_data: encoded_exit.2.0.into(),
    });
    
    Ok(template)
}

// Helper functions
/// Converts BigUint to U256
fn biguint_to_u256(value: &BigUint) -> U256 {
    let bytes = value.to_bytes_be();
    U256::from_big_endian(&bytes)
}

/// Converts Bytes to Address
fn bytes_to_address(bytes: &Bytes) -> Result<Address> {
    if bytes.len() != 20 {
        anyhow::bail!("Invalid address length: expected 20 bytes, got {}", bytes.len());
    }
    Ok(Address::from_slice(bytes.as_ref()))
}

pub fn alloy_to_ethcontract(value: AlloyU256) -> U256 {
    // Convert to big-endian bytes (32 bytes)
    let bytes = value.to_be_bytes::<32>();
    
    // Create ethcontract U256 from big-endian bytes
    U256::from_big_endian(&bytes)
}

pub fn ethcontract_to_alloy(value: U256) -> AlloyU256 {
    // Convert to big-endian bytes (32 bytes)
    let mut bytes = [0u8; 32];
    value.to_big_endian(&mut bytes);
    
    // Create alloy U256 from big-endian bytes
    AlloyU256::from_be_bytes(bytes)
}