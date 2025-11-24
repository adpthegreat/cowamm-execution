use {
    derive_more::Debug as DeriveDebug,
    ethcontract::{
        transaction::TransactionBuilder,
        Address,
        Bytes,
    },
    hex_literal::hex,
    model::order::OrderUid,
    num::BigUint,
    number::serialization::HexOrDecimalU256,
    primitive_types::{H160, H256, U256},
    serde::{de, Deserialize, Deserializer, Serialize, Serializer},
    serde_with::{serde_as, DisplayFromStr},
    std::{
        collections::{HashMap, HashSet},
        fmt::{self, Debug, Display},
        str::FromStr,
    },
    web3::Transport,
};

#[serde_as]
#[derive(Eq, PartialEq, Clone,Default, Deserialize, Serialize, DeriveDebug)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedAmounts {
    #[debug("{}", format_args!("{sell}"))]
    #[serde_as(as = "DisplayFromStr")]
    pub sell: BigUint,
    #[debug("{}", format_args!("{buy}"))]
    #[serde_as(as = "DisplayFromStr")]
    pub buy: BigUint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionPrices {
    pub clearing_prices: HashMap<Address, String>, 
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionOrderStatus {
    pub r#type: String,
    pub value: Vec<SolverStatus>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverStatus {
    pub solver: String,
    pub executed_amounts: Option<ExecutedAmounts>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePriceResponse {
    pub price: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalSurplus {
    pub total_surplus: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitionAuction {
    pub orders: Vec<OrderUid>,
    pub prices: AuctionPrices,
}

#[serde_as]
#[derive(Clone, Default, Serialize, Deserialize, DeriveDebug)]
#[serde(rename_all = "camelCase")]
pub struct SolverSettlement {
    pub ranking: f64,
    #[serde_as(as = "DisplayFromStr")]
    #[debug("{}", format_args!("{solver_address}"))]
    pub solver_address: BigUint,
    #[serde_as(as = "DisplayFromStr")]
    #[debug("{}", format_args!("{score}"))]
    pub score: BigUint,
    #[debug("{}", format_args!("{reference_score}"))]
    #[serde_as(as = "DisplayFromStr")]
    pub reference_score: BigUint,
    pub tx_hash: H256,
    #[serde_as(as = "HashMap<DisplayFromStr, DisplayFromStr>")]
    pub clearing_prices: HashMap<Address, String>,
    pub orders: Vec<SolverOrder>,
    pub is_winner: bool,
    pub filtered_out: bool,
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Default, Serialize, Deserialize, DeriveDebug)]
#[serde(rename_all = "camelCase")]
pub struct SolverOrder {
    pub id: OrderUid,
    #[debug("{}", format_args!("{sell_amount}"))]
    #[serde_as(as = "DisplayFromStr")]
    pub sell_amount: BigUint,
    #[debug("{}", format_args!("{buy_amount}"))]
    #[serde_as(as = "DisplayFromStr")]
    pub buy_amount: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolverCompetitionResponse {
    pub auction_id: u64,
    pub auction_start_block: u64,
    pub transaction_hashes: Vec<H256>,
    pub reference_scores: HashMap<String, String>,
    pub auction: CompetitionAuction,
    pub solutions: Vec<SolverSettlement>,
}
