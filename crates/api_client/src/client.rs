use {
    anyhow::{Context, Result},
    shared::{
        models::{
            auction::Auction,
            order::{OrderClass, OrderKind, OrderStatus, OrderUid, CancellationPayload, OrderCancellations, OrderCreation, BUY_ETH_ADDRESS},
            quote::{OrderQuoteRequest, OrderQuoteResponse},
            trade::{Trade},
            competition::{CompetitionOrderStatus, NativePriceResponse, TotalSurplus, CompetitionAuction, SolverCompetitionResponse}
        },
        app_data::app_data_hash::{AppDataHash, AppDataDocument},
        number::serialization::HexOrDecimalU256,
        utils::url_utils,
    },
    primitive_types::{H160, U256, H256},
    serde::{Deserialize, Deserializer, Serialize, Serializer, de},
    reqwest::Client,
    hex_literal::hex,
    serde_with::serde_as,
    serde_json,
    ethcontract::{Address},
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

    //post an order
    pub async fn create_order(&self, order: &OrderCreation) -> reqwest::Result<OrderUid> {
        let url = url_utils::join(&self.base, &format!("api/v1/orders"));
        self.client
            .post(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    //delete an order
    pub async fn cancel_order(&self, uid: &OrderUid, cancellation: &CancellationPayload) -> reqwest::Result<()> {
        let url = url_utils::join(&self.base, &format!("api/v1/orders/{uid}"));
        self.client
            .delete(url)
            .json(cancellation)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
    //delete an order
    pub async fn cancel_orders(&self, cancellations: &OrderCancellations) -> reqwest::Result<()> {
        let url = url_utils::join(&self.base, &format!("api/v1/orders"));
        self.client
            .delete(url)
            .json(cancellations)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_order(&self, uid: &OrderUid) -> reqwest::Result<Order> {
        let url = url_utils::join(&self.base, &format!("api/v1/orders/{uid}"));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    //get the status of an order 
    pub async fn get_order_status(&self, uid: &OrderUid) -> reqwest::Result<CompetitionOrderStatus> {
        let url = url_utils::join(&self.base, &format!("api/v1/orders/{uid}/status"));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

   // Transaction endpoints
    pub async fn get_orders_by_tx(&self, tx_hash: &H256) -> reqwest::Result<Vec<Order>> {
        let url = url_utils::join(&self.base, &format!("api/v1/transactions/{:x}/orders", tx_hash));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // Trades endpoints
    pub async fn get_trades_by_owner(&self, owner: &Address) -> reqwest::Result<Vec<Trade>> {
        let url = url_utils::join(&self.base, "api/v1/trades");
        self.client
            .get(url)
            .query(&[("owner", format!("{:x}", owner))])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_trades_by_order(&self, order_uid: &OrderUid) -> reqwest::Result<Vec<Trade>> {
        let url = url_utils::join(&self.base, "api/v1/trades");
        self.client
            .get(url)
            .query(&[("orderUid", order_uid.to_string())])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // Auction endpoints
    pub async fn get_auction(&self) -> reqwest::Result<Auction> {
        let url = url_utils::join(&self.base, "api/v1/auction");
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // Account endpoints
    pub async fn get_user_orders(&self, owner: &Address, offset: Option<u64>, limit: Option<u64>) -> reqwest::Result<Vec<Order>> {
        let url = url_utils::join(&self.base, &format!("api/v1/account/{:x}/orders", owner));
        let mut query = Vec::new();
        if let Some(offset) = offset {
            query.push(("offset", offset.to_string()));
        }
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        
        self.client
            .get(url)
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // Token endpoints
    pub async fn get_native_price(&self, token: &Address) -> reqwest::Result<NativePriceResponse> {
        let url = url_utils::join(&self.base, &format!("api/v1/token/{:x}/native_price", token));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // Quote endpoints
    pub async fn get_quote(&self, request: &OrderQuoteRequest) -> reqwest::Result<OrderQuoteResponse> {
        let url = url_utils::join(&self.base, "api/v1/quote");
        self.client
            .post(url)
            .json(request)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
 
    // Solver competition endpoints (v2)
    pub async fn get_solver_competition_v2(&self, auction_id: u64) -> reqwest::Result<SolverCompetitionResponse> {
        let url = url_utils::join(&self.base, &format!("api/v2/solver_competition/{auction_id}"));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_solver_competition_by_tx_v2(&self, tx_hash: &H256) -> reqwest::Result<SolverCompetitionResponse> {
        let url = url_utils::join(&self.base, &format!("api/v2/solver_competition/by_tx_hash/{:x}", tx_hash));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn get_latest_solver_competition_v2(&self) -> reqwest::Result<SolverCompetitionResponse> {
        let url = url_utils::join(&self.base, "api/v2/solver_competition/latest");
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // Version endpoint
    pub async fn get_version(&self) -> reqwest::Result<String> {
        let url = url_utils::join(&self.base, "api/v1/version");
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    // App data endpoints
    pub async fn get_app_data(&self, app_data_hash: &AppDataHash) -> reqwest::Result<AppDataDocument> {
        let url = url_utils::join(&self.base, &format!("api/v1/app_data/{}", serde_json::to_string(app_data_hash).unwrap()));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn register_app_data(&self, app_data_hash: &AppDataHash, app_data: &AppDataDocument) -> reqwest::Result<AppDataHash> {
        let url = url_utils::join(&self.base, &format!("api/v1/app_data/{}", serde_json::to_string(app_data_hash).unwrap()));
        self.client
            .put(url)
            .json(app_data)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    pub async fn register_app_data_auto(&self, app_data: &AppDataDocument) -> reqwest::Result<AppDataHash> {
        let url = url_utils::join(&self.base, "api/v1/app_data");
        self.client
            .put(url)
            .json(app_data)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    // User endpoints
    pub async fn get_user_total_surplus(&self, address: &Address) -> reqwest::Result<TotalSurplus> {
        let url = url_utils::join(&self.base, &format!("api/v1/users/{:x}/total_surplus", address));
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

