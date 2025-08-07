//! Contains models that are shared between the orderbook and the solver.
use {
    hex::{FromHex, FromHexError},
    primitive_types::H160,
    std::{fmt, sync::LazyLock},
    web3::{
        ethabi::{Token, encode},
        signing,
    },
};


#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct DomainSeparator(pub [u8; 32]);

impl std::str::FromStr for DomainSeparator {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(FromHex::from_hex(s)?))
    }
}

impl std::fmt::Debug for DomainSeparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hex = [0u8; 64];
        // Unwrap because we know the length is correct.
        hex::encode_to_slice(self.0, &mut hex).unwrap();
        // Unwrap because we know it is valid utf8.
        f.write_str(std::str::from_utf8(&hex).unwrap())
    }
}

impl DomainSeparator {
    pub fn new(chain_id: u64, contract_address: H160) -> Self {
        /// The EIP-712 domain name used for computing the domain separator.
        static DOMAIN_NAME: LazyLock<[u8; 32]> =
            LazyLock::new(|| signing::keccak256(b"Gnosis Protocol"));

        /// The EIP-712 domain version used for computing the domain separator.
        static DOMAIN_VERSION: LazyLock<[u8; 32]> = LazyLock::new(|| signing::keccak256(b"v2"));

        /// The EIP-712 domain type used computing the domain separator.
        static DOMAIN_TYPE_HASH: LazyLock<[u8; 32]> = LazyLock::new(|| {
            signing::keccak256(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        )
        });
        let abi_encode_string = encode(&[
            Token::Uint((*DOMAIN_TYPE_HASH).into()),
            Token::Uint((*DOMAIN_NAME).into()),
            Token::Uint((*DOMAIN_VERSION).into()),
            Token::Uint(chain_id.into()),
            Token::Address(contract_address),
        ]);

        DomainSeparator(signing::keccak256(abi_encode_string.as_slice()))
    }
}


#[cfg(test)]
mod tests {
    use {
        super::*,
        hex_literal::hex,
        std::{cmp::Ordering, str::FromStr},
    };

    #[test]
    fn domain_separator_from_str() {
        assert!(
            DomainSeparator::from_str(
                "9d7e07ef92761aa9453ae5ff25083a2b19764131b15295d3c7e89f1f1b8c67d9"
            )
            .is_ok()
        );
    }

    #[test]
    fn domain_separator_sepolia() {
        let contract_address: H160 = hex!("9008D19f58AAbD9eD0D60971565AA8510560ab41").into(); // new deployment
        let chain_id: u64 = 11155111;
        let domain_separator_sepolia = DomainSeparator::new(chain_id, contract_address);
        // domain separator is taken from Sepolia deployment at address
        // 0x9008D19f58AAbD9eD0D60971565AA8510560ab41
        // https://sepolia.etherscan.io/address/0x9008d19f58aabd9ed0d60971565aa8510560ab41#readContract#F2
        let expected_domain_separator = DomainSeparator(hex!(
            "daee378bd0eb30ddf479272accf91761e697bc00e067a268f95f1d2732ed230b"
        ));
        assert_eq!(domain_separator_sepolia, expected_domain_separator);
    }
}