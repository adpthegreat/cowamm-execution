https://github.com/cowprotocol/services/blob/7273c526eada6c3f6eb351949048d22a3e7d520d/crates/driver/src/domain/liquidity/balancer/v2/weighted.rs
//Encode a pool join or pool exit as an interaction 
use {
    super::{Fee, Id, ScalingFactor},
    crate::{
        boundary,
        domain::{eth, liquidity},
    },
    itertools::Itertools,
};


#[derive(Clone, Debug)]
pub struct Pool {
    pub vault: eth::ContractAddress,
    pub id: Id,
    pub reserves: Reserves,
    pub fee: Fee,
}

impl Pool {
    /// Encodes a pool swap as an interaction. Returns `Err` if the swap
    /// parameters are invalid for the pool, specifically if the input and
    /// output tokens do not belong to the pool.
    pub fn join_pool(
        &self,
        input: &liquidity::MaxInput,
        output: &liquidity::ExactOutput,
        receiver: &eth::Address,
    ) -> Result<eth::Interaction, liquidity::InvalidSwap> {
        if !self.reserves.has_tokens(&input.0.token, &output.0.token) {
            return Err(liquidity::InvalidSwap);
        }

        Ok(boundary::liquidity::cowamm::to_interaction(
            self, input, output, receiver,
        ))
    }
}

/// Balancer weighted pool reserves.
///
/// This is an ordered collection of tokens with their balance and weights.
#[derive(Clone, Debug)]
pub struct Reserves(Vec<Reserve>);

impl Reserves {
    /// Creates new CowAMM token reserves, returns `Err` if the specified
    /// token reserves are invalid.
    pub fn try_new(reserves: Vec<Reserve>) -> Result<Self, InvalidReserves> {
        if !reserves.iter().map(|r| r.asset.token).all_unique() {
            return Err(InvalidReserves::DuplicateToken);
        }

        Ok(Self(reserves))
    }

    /// Returns `true` if the reserves correspond to the specified tokens.
    fn has_tokens(&self, a: &eth::TokenAddress, b: &eth::TokenAddress) -> bool {
        self.tokens().contains(a) && self.tokens().contains(b)
    }

    /// Returns an iterator over the reserve tokens.
    pub fn tokens(&self) -> impl Iterator<Item = eth::TokenAddress> + '_ {
        self.iter().map(|r| r.asset.token)
    }

    /// Returns an iterator over the reserve assets.
    pub fn iter(&self) -> impl Iterator<Item = Reserve> + '_ {
        self.0.iter().copied()
    }
}

impl IntoIterator for Reserves {
    type IntoIter = <Vec<Reserve> as IntoIterator>::IntoIter;
    type Item = Reserve;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidReserves {
    #[error("CowAMM token reserves; duplicate token address")]
    DuplicateToken,
}

/// CowAMM weighted pool reserve for a single token.
#[derive(Clone, Copy, Debug)]
pub struct Reserve {
    pub asset: eth::Asset,
    pub scale: ScalingFactor,
    pub weight: Weight,
}

/// A CowAMM token weight.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Weight(pub eth::U256);

impl Weight {
    /// Creates a new token weight for the specified raw [`eth::U256`] value.
    /// This method expects a weight represented as `w * 1e18`. That is, a
    /// weight of 1 is created with `Weight::new(U256::exp10(18))`.
    pub fn from_raw(weight: eth::U256) -> Self {
        Self(weight)
    }

    /// Returns the weight as a raw [`eth::U256`] value as it is represented
    /// on-chain.
    pub fn as_raw(&self) -> eth::U256 {
        self.0
    }

    fn base() -> eth::U256 {
        eth::U256::exp10(18)
    }
}
