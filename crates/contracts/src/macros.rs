//replica of dummy_contract, but will use it this one for live examples and dummy_contract! for tests
#[macro_export]
macro_rules! contract {
    ($contract:ty, $addr:expr_2021) => {
        <$contract>::at(&$crate::web3::dummy(), $addr.into())
    };
}

#[macro_export]
macro_rules! dummy_contract {
    ($contract:ty, $addr:expr_2021) => {
        <$contract>::at(&$crate::web3::dummy(), $addr.into())
    };
}

#[macro_export]
macro_rules! bytecode {
    ($contract:ty) => {
        <$contract>::raw_contract().bytecode.to_bytes().unwrap()
    };
}

#[macro_export]
macro_rules! deployed_bytecode {
    ($contract:ty) => {
        <$contract>::raw_contract()
            .deployed_bytecode
            .to_bytes()
            .unwrap()
    };
}