


#[macro_export]
macro_rules! contract {
    ($contract:ty, $addr:expr_2021) => {
        <$contract>::at(&web3::Web3::new(web3::transports::Http::new("https://ethereum-rpc.publicnode.com").unwrap()), $addr.into())
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