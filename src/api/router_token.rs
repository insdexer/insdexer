use super::{HttpResponseExt, WebData, PAGE_SIZE, router_inscription::insc_list_to_display};
use crate::{
    inscription::{db::*, types::*},
    num_index,
    txn_db::TxnDB,
};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn register(config: &mut web::ServiceConfig) {
    config.service(tokens);
    config.service(token_info);
    config.service(token_holders);
    config.service(token_balance);
    config.service(token_txs);
}

fn token_to_display(token: &InscriptionToken) -> serde_json::Value {
    json!({
        "insc_id": token.insc_id,
        "tick": token.tick,
        "tick_i": token.tick_i,
        "tx": token.tx,
        "from": token.from,
        "blocknumber": token.blocknumber.to_string(),
        "timestamp": token.timestamp.to_string(),
        "holders": token.holders.to_string(),
        "mint_max": token.mint_max.to_string(),
        "mint_limit": token.mint_limit.to_string(),
        "mint_progress": token.mint_progress.to_string(),
        "mint_finished": token.mint_finished,
        "market_volume24h": token.market_volume24h.to_string(),
        "market_txs24h": token.market_txs24h.to_string(),
        "market_cap": token.market_cap.to_string(),
        "market_floor_price": token.market_floor_price.to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct TokensParams {
    page: Option<u64>,
    order_by: Option<String>,
}

#[get("/tokens")]
async fn tokens(info: Query<TokensParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let mut list = db.get_tokens_list();
    let order_type = &info.order_by;

    match order_type.as_deref() {
        Some("holders") => {
            list.sort_unstable_by(|a: &InscriptionToken, b: &InscriptionToken| b.holders.cmp(&a.holders));
        }
        Some("mcap") => {
            list.sort_unstable_by(|a: &InscriptionToken, b: &InscriptionToken| b.market_cap.cmp(&a.market_cap));
            list.retain(|i| i.mint_finished);
        }
        Some("mint") => {
            list.sort_unstable_by(|a: &InscriptionToken, b: &InscriptionToken| {
                let progress_a = a.mint_progress as f64 / a.mint_max as f64;
                let progress_b = b.mint_progress as f64 / b.mint_max as f64;
                if progress_b < progress_a {
                    std::cmp::Ordering::Less
                } else if progress_b == progress_a {
                    std::cmp::Ordering::Equal
                } else {
                    std::cmp::Ordering::Greater
                }
            });
            list.retain(|i| !i.mint_finished);
        }
        _ => {}
    }

    let mut pages = list.chunks(PAGE_SIZE.try_into().unwrap());
    if let Some(page_slice) = pages.nth(page.try_into().unwrap()) {
        let result_list: Vec<serde_json::Value> = page_slice.iter().map(token_to_display).collect();
        HttpResponse::response_data(result_list)
    } else {
        HttpResponse::response_data(Vec::<InscriptionToken>::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenInfoParams {
    tick: String,
}

#[get("/token_info")]
async fn token_info(info: Query<TokenInfoParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let key_tick = make_index_key(KEY_INSC_TOKEN_INDEX_TICK_I, &info.tick);
    let id = db.get_u64(key_tick.as_str());
    if id == 0 {
        HttpResponse::response_error_notfound();
    }

    let key_id = make_index_key(KEY_INSC_TOKEN_INDEX_ID, num_index!(id));
    let result = db.get(key_id.as_bytes()).unwrap();
    let res: Option<InscriptionToken> = serde_json::from_slice(&result.unwrap()).unwrap();
    HttpResponse::response_data(token_to_display(&res.unwrap()))
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenHoldersParams {
    page: Option<u64>,
    tick: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenHoldersResponse {
    address: String,
    balance: String,
}

#[get("/token_holders")]
async fn token_holders(info: Query<TokenHoldersParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let start_key = make_index_key(KEY_INSC_BALANCE_INDEX_TICK_BALANCE_HOLDER, &info.tick) + ":";
    let key_list = db.get_items(
        &start_key,
        &start_key,
        page * PAGE_SIZE,
        PAGE_SIZE,
        rocksdb::Direction::Forward,
    );

    let mut holders: Vec<TokenHoldersResponse> = Vec::new();
    for (key, value) in &key_list {
        let address = String::from_utf8(key[key.len() - 42..].to_vec()).unwrap();
        let balance = u64::from_be_bytes(value.as_slice().try_into().unwrap());
        holders.push(TokenHoldersResponse {
            address,
            balance: balance.to_string(),
        });
    }

    HttpResponse::response_data(holders)
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenBalanceParams {
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenBalanceResponse {
    tick: String,
    balance: String,
    token: serde_json::Value,
}

#[get("/token_balance")]
async fn token_balance(info: Query<TokenBalanceParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let prefix = make_index_key(KEY_INSC_BALANCE_INDEX_HOLDER_TICK, info.address.to_lowercase().as_str());
    let key_list = db.get_items(&prefix, &prefix, 0, 9999, rocksdb::Direction::Forward);
    let mut token_list = Vec::new();
    for (key, value) in &key_list {
        let tick_pos = key.len() - key.iter().rev().position(|&x| x == b':').unwrap();
        let tick = String::from_utf8(key[tick_pos..].to_vec()).unwrap();
        let balance = u64::from_be_bytes(value.as_slice().try_into().unwrap());
        let token = db.get_token(&tick).unwrap();
        token_list.push(TokenBalanceResponse {
            tick,
            balance: balance.to_string(),
            token: token_to_display(&token),
        });
    }
    HttpResponse::response_data(token_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenTxsParams {
    tick: String,
    page: Option<u64>,
}

#[get("/token_txs")]
async fn token_txs(info: Query<TokenTxsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let start_key = make_index_key(KEY_INSC_TOKEN_TRANSFER, info.tick.as_str()) + ":";
    let key_list = db.get_item_keys(
        &start_key,
        &start_key,
        page * PAGE_SIZE,
        PAGE_SIZE,
        rocksdb::Direction::Forward,
    );

    let id_list = db_index2id_desc(key_list);
    let insc_list = db.get_inscriptions_by_id(&id_list);

    HttpResponse::response_data(insc_list_to_display(&db, &insc_list))
}
