use super::{HttpResponseExt, WebData, PAGE_SIZE};
use crate::inscription::{db::*, types::*};
use actix_web::{get, web, web::Query, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use rocksdb::Direction;
use serde::{Deserialize, Serialize};

pub fn register(config: &mut web::ServiceConfig) {
    config.service(nfts);
    config.service(nft_image);
}

#[derive(Debug, Serialize, Deserialize)]
struct NFTsParams {
    page: Option<u64>,
    address: String,
    exclude_image_data: Option<bool>,
}

#[get("/nfts")]
async fn nfts(info: Query<NFTsParams>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();
    let page = info.page.unwrap_or(1) - 1;
    let prefix = make_index_key(KEY_INSC_NFT_INDEX_HOLDER_ADDRESS, &info.address);
    let key_list = db.get_item_keys(&prefix, &prefix, page * PAGE_SIZE, PAGE_SIZE, Direction::Forward);
    let id_list = db_index2id_desc(key_list);
    let mut insc_list = db.get_inscriptions_by_id(&id_list);

    if info.exclude_image_data.unwrap_or(false) {
        for insc in insc_list.iter_mut() {
            if insc.mime_category == InscriptionMimeCategory::Image {
                insc.mime_data = "".to_string();
            }
        }
    }

    HttpResponse::response_data(insc_list)
}

#[get("/nft/{id}")]
async fn nft_image(id: web::Path<u64>, state: WebData) -> impl Responder {
    let db = state.db.read().unwrap();

    match db.get_inscription_by_id(id.into_inner()) {
        Some(insc) => match insc.mime_category {
            InscriptionMimeCategory::Image => {
                let bytes = general_purpose::STANDARD.decode(&insc.mime_data).unwrap();
                HttpResponse::Ok().content_type(insc.mime_type).body(bytes)
            }
            InscriptionMimeCategory::Text => HttpResponse::Ok().body(insc.mime_data),
            _ => HttpResponse::BadRequest().body("Not an NFT"),
        },
        None => HttpResponse::NotFound().body("Inscription not found"),
    }
}
