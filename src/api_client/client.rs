// module to post orders to the API 
use {
    anyhow::{Context, Result},
    crate::shared::{
        models::order::{OrderClass, OrderKind, OrderStatus, OrderUid, BUY_ETH_ADDRESS},
        number::serialization::HexOrDecimalU256,
        utils::url_utils,
    },
    primitive_types::{H160, U256},
    reqwest::Client,
    serde_with::serde_as,
    std::{
        collections::HashMap,
        time::{Duration, Instant},
    },
    url::Url,
};

#[serde_as]
#[derive(Debug, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Order {
    kind: OrderKind,
    buy_token: H160,
    #[serde_as(as = "HexOrDecimalU256")]
    buy_amount: U256,
    sell_token: H160,
    #[serde_as(as = "HexOrDecimalU256")]
    sell_amount: U256,
    uid: OrderUid,
    partially_fillable: bool,
    #[serde(flatten)]
    class: OrderClass,
    // Some if the order is fetched from api/v1/orders/{uid}
    // None if the order is fetched from api/v1/auction
    #[serde(default)]
    status: Option<OrderStatus>,
}

impl Order {
    fn is_liquidity_order(&self) -> bool {
        matches!(self.class, OrderClass::Liquidity)
    }
}

struct OrderBookApi {
    base: Url,
    client: Client,
}

impl OrderBookApi {
    pub fn new(client: Client, base_url: &str) -> Self {
        Self {
            base: base_url.parse().unwrap(),
            client,
        }
    }

    pub async fn solvable_orders(&self) -> reqwest::Result<Vec<Order>> {
        #[derive(serde::Deserialize)]
        struct Auction {
            orders: Vec<Order>,
        }
        let url = url_utils::join(&self.base, "api/v1/auction");
        let auction: Auction = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(auction.orders)
    }

    pub async fn order(&self, uid: &OrderUid) -> reqwest::Result<Order> {
        let url = url_utils::join(&self.base, &format!("api/v1/orders/{uid}"));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}


//https://github.com/cowprotocol/trading-bot/blob/main/src/make_trade.ts