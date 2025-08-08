use {
    contracts::BCowPool,
    ethcontract::{Bytes, H256, Address},
    primitive_types::U256,
    crate::{
        shared::{
            http_solver::model::TokenAmount,
            interaction::{EncodedInteraction, Interaction},
        },
    },
};

#[derive(Clone, Debug)]
pub struct JoinPoolInteraction {
    pub b_cow_pool: BCowPool,
    pub pool_id: H256,
    pub sender: Address,
    pub recipient: Address,
    pub pool_amount_out: U256,
    pub max_amounts_in: Vec<U256>,
    pub user_data: Bytes<Vec<u8>>,
}

impl JoinPoolInteraction {
    pub fn encode_join(&self) -> EncodedInteraction {
        let method = self.b_cow_pool.join_pool(
            (
                Bytes(self.pool_id.0),
                self.sender,
                self.recipient,
                (
                    self.pool_amount_out,
                    self.max_amounts_in.clone(),
                    self.user_data.clone(),
                ),
            )
        );

        let calldata = method.tx.data.expect("no calldata").0;

        (self.vault.address(), 0.into(), Bytes(calldata))
    }
}

impl Interaction for JoinPoolInteraction {
    fn encode(&self) -> EncodedInteraction {
        self.encode_join()
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
        assert_eq!(to, pool);
        assert_eq!(value, U256::zero());
        assert!(data.0.len() > 4); // basic sanity check
    }
}
