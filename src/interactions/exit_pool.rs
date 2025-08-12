use {
    ethcontract::{Bytes, H256},
    primitive_types::U256,
    crate::{
        contracts::{BCowPool},
        shared::{
            interaction::{EncodedInteraction, Interaction},
            http_solver::model::TokenAmount
        }
    },
};

#[derive(Clone, Debug)]
pub struct ExitPoolInteraction {
    pub b_cow_pool: BCowPool,
    pub pool_amount_in: U256,
    pub min_amounts_out: Vec<U256>,
}

impl ExitPoolInteraction {
    pub fn encode_exit(&self) -> EncodedInteraction {
        let calldata = self.b_cow_pool.exit_pool(
            self.pool_amount_in,
            self.min_amounts_out,
        ).tx.data.expect("exit_pool should have calldata").0;

        // This assumes user_data is already ABI-encoded ExitPool (with poolAmountIn + minAmountsOut).
        // If not, you'll need to encode using `ethabi::encode`.

        (self.b_cow_pool.address(), 0.into(), Bytes(calldata))
    }
}

impl Interaction for ExitPoolInteraction {
    fn encode(&self) -> EncodedInteraction {
        self.encode_exit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::dummy_contract;
    use primitive_types::{H160};

    #[test]
    fn encode_join_pool() {
        let interaction = ExitPoolInteraction {
            pool_amount_in: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
            max_amounts_out: vec![
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
            ],
        };

        let (to, value, data) = interaction.encode();
        assert_eq!(to, vault.address());
        assert_eq!(value, U256::zero());
        assert!(data.0.len() > 4); // basic sanity check
    }
}
