use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MarketOrderType {
    #[serde(rename = "0")]
    NFT,
    #[serde(rename = "1")]
    Token,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MarketOrderStatus {
    #[serde(rename = "0")]
    Init,
    #[serde(rename = "1")]
    Open,
    #[serde(rename = "2")]
    Close,
    #[serde(rename = "3")]
    Canceled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOrder {
    pub order_type: MarketOrderType,
    pub order_id: String,
    pub from: String,
    pub to: String,
    pub tick: String,
    pub nft_id: u64,
    pub nft_tx: String,
    pub amount: u64,
    pub total_price: u128,
    pub unit_price: u128,
    pub tx: String,

    pub tx_setprice: String,
    pub tx_cancel: String,
    pub tx_close: String,

    pub blocknumber: u64,
    pub timestamp: u64,

    pub order_status: MarketOrderStatus,

    pub buyer: String,
}
