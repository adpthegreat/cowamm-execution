use {
    contracts::{BCowPool, GPv2Settlement},
    ethcontract::{Bytes, H256},
    primitive_types::U256,
    crate::{
        shared::http_solver::model::TokenAmount,
        interaction::{EncodedInteraction, Interaction},
    },
};

#[derive(Clone, Debug)]
pub struct ExitPoolInteraction {
    pub settlement: GPv2Settlement,
    pub b_cow_pool: BCowPool,
    pub pool_id: H256,
    pub pool_amount_in: U256,
    pub min_amounts_out: Vec<U256>,
    pub user_data: Bytes<Vec<u8>>,
}

impl ExitPoolInteraction {
    pub fn encode_exit(&self) -> EncodedInteraction {
        let calldata = self.b_cow_pool.exit_pool(
            self.pool_id,
            self.settlement.address(),
            self.settlement.address(),
            self.user_data.clone(),
        ).tx.data.expect("exit_pool should have calldata").0;

        // This assumes user_data is already ABI-encoded ExitPool (with poolAmountIn + minAmountsOut).
        // If not, you'll need to encode using `ethabi::encode`.

        (self.vault.address(), 0.into(), Bytes(calldata))
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
        let vault = dummy_contract!(BCowPool, [0x01; 20]);
        let interaction = JoinPoolInteraction {
            vault: vault.clone(),
            pool_id: H256([0x02; 32]),
            sender: H160([0x03; 20]),
            recipient: H160([0x04; 20]),
            pool_amount_out: U256::from_dec_str("1000000000000000000").unwrap(), // 1e18
            max_amounts_in: vec![
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
                U256::from_dec_str("500000000000000000").unwrap(), // 0.5e18
            ],
            user_data: Bytes::default(),
        };

        let (to, value, data) = interaction.encode();
        assert_eq!(to, vault.address());
        assert_eq!(value, U256::zero());
        assert!(data.0.len() > 4); // basic sanity check
    }
}
