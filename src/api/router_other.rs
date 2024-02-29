use super::{HttpResponseExt, WebData};
use crate::inscription::db::*;
use actix_web::{get, web, HttpResponse, Responder};

pub fn register(config: &mut web::ServiceConfig) {
    config.service(collection_trending);
}

#[get("/trending_nft_collection")]
async fn collection_trending(state: WebData) -> impl Responder {
    match state
        .db
        .read()
        .unwrap()
        .get_inscription_by_tx("0x355b9e1d84603a57e06b2d81c0f90d43438959519c987c7c6a03f025b5fc999d")
    {
        Some(insc) => HttpResponse::response_data(insc),
        None => HttpResponse::response_error_notfound(),
    }
}
