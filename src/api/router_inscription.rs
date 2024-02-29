use super::{HttpResponseExt, WebData, PAGE_SIZE};
use crate::{
    inscription::{db::*, types::*},
    num_index,
};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use rocksdb::Direction;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn register(config: &mut web::ServiceConfig) {
    config.service(inscription);
    config.service(recent);
    config.service(transactions);
    config.service(created);
    config.service(index_behind);
}

#[derive(Debug, Serialize, Deserialize)]
struct InscriptionsParams {
    id: Option<u64>,
    tx: Option<String>,
}

#[get("/inscription")]
async fn inscription(query: Query<InscriptionsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let insc = if let Some(insc_id) = query.id {
        db.get_inscription_by_id(insc_id)
    } else if let Some(insc_tx) = &query.tx {
        db.get_inscription_by_tx(insc_tx)
    } else {
        None
    };

    match insc {
        Some(insc) => {
            let mut insc_json = serde_json::to_value(&insc).unwrap();
            if insc.signature.is_some() {
                let holder = db.get_inscription_nft_holder_by_id(insc.id);
                insc_json["owner"] = serde_json::to_value(holder).unwrap();
            }
            HttpResponse::response_data(insc_json)
        }
        None => HttpResponse::response_error_notfound(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RecentParams {
    page: Option<u64>,
}

#[get("/recent")]
async fn recent(info: Query<RecentParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let top_insc_id = db.get_top_inscription_id();
    let start_key = make_index_key(KEY_INSC_INDEX_ID, num_index!(top_insc_id));
    let insc_list = db.get_items(KEY_INSC_INDEX_ID, &start_key, page * PAGE_SIZE, PAGE_SIZE, Direction::Reverse);

    HttpResponse::response_data(db_item_val2json::<Inscription>(insc_list))
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionsParams {
    page: Option<u64>,
    address: String,
}

#[get("/transactions")]
async fn transactions(info: Query<TransactionsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let prefix = make_index_key(KEY_INSC_INDEX_ADDRESS, &info.address);
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2id_desc(key_list);
    let insc_list = db.get_inscriptions_by_id(&id_list);
    HttpResponse::response_data(insc_list)
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatedParams {
    page: Option<u64>,
    address: String,
}

#[get("/created")]
async fn created(info: Query<CreatedParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let prefix = make_index_key(KEY_INSC_INDEX_CREATER, &info.address);
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2id_desc(key_list);
    let mut insc_list = db.get_inscriptions_by_id(&id_list);
    for insc in insc_list.iter_mut() {
        if insc.mime_category == InscriptionMimeCategory::Image {
            insc.mime_data = "".to_string();
        }
    }
    HttpResponse::response_data(insc_list)
}

#[get("/status")]
async fn index_behind(state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let top_insc_id = db.get_top_inscription_id();
    let sync_blocknumber = db.get_sync_blocknumber();

    match db.get_inscription_by_id(top_insc_id) {
        Some(insc) => {
            let current_blocknumber = *state.blocknumber.read().unwrap();
            let insc_blocknumber = insc.blocknumber;
            let result = json!({
                "behind": insc_blocknumber as i64 - current_blocknumber as i64,
                "block_number": current_blocknumber,
                "block_number_index": insc_blocknumber,
                "block_number_sync": sync_blocknumber,
            });
            HttpResponse::response_data(result)
        }
        None => HttpResponse::response_error_notfound(),
    }
}