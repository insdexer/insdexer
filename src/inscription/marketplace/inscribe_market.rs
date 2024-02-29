use super::{
    db::{InscribeMarketDB, InscribeMarketTxn},
    market_abi::MARKET_ABI_JSON,
    types::{MarketOrder, MarketOrderStatus, MarketOrderType},
    MarketPlace, APP_OPER_TOKEN_MARKET_LIST,
};
use crate::{
    config::MARKET_ADDRESS_LIST,
    ethereum::{Web3ABILogEvent, Web3LogEvent},
    inscription::{
        db::InscribeDB,
        inscribe_token::ProcessBlockContextJsonToken,
        trait_json_value::JsonValueTrait,
        types::{InscribeContext, Inscription, InscriptionMimeCategory, TRANSFER_TX_HEX_LENGTH},
    },
};
use log::{debug, info, warn};
use rocksdb::{Transaction, TransactionDB};
use std::vec;

lazy_static! {
    pub static ref CONTRACT_MARKET: web3::ethabi::Contract = web3::ethabi::Contract::load(MARKET_ABI_JSON.as_bytes()).unwrap();
}

impl MarketPlace for InscribeContext {
    fn process_inscribe_invoke(&mut self, insc: &mut Inscription) -> bool {
        assert!(insc.event_logs.len() == 1);

        if let Some(log) = insc.event_logs[0].match_event(&CONTRACT_MARKET, "MarketBuy") {
            self.execute_market_buy(insc, &log)
        } else if let Some(log) = insc.event_logs[0].match_event(&CONTRACT_MARKET, "MarketCancel") {
            self.execute_market_cancel(insc, &log)
        } else if let Some(log) = insc.event_logs[0].match_event(&CONTRACT_MARKET, "MarketSetPrice") {
            self.execute_market_set_price(insc, &log)
        } else {
            false
        }
    }

    fn execute_market_buy(&mut self, insc: &Inscription, log: &web3::ethabi::Log) -> bool {
        let order_id = "0x".to_string() + &log.get_param("orderId").unwrap().to_string();
        let order = match self.db.read().unwrap().market_get_order_by_id(&order_id) {
            Some(order) => order,
            None => {
                warn!("[indexer] market_buy order not found: {} {}", insc.tx_hash, order_id);
                return false;
            }
        };

        match order.order_type {
            MarketOrderType::Token => self.execute_market_buy_token(insc, &order, log),
            MarketOrderType::NFT => self.execute_market_buy_nft(insc, &order, log),
        }
    }

    fn execute_market_buy_token(&mut self, insc: &Inscription, order: &MarketOrder, log: &web3::ethabi::Log) -> bool {
        let total_price = log.get_param("price").unwrap().clone().into_uint().unwrap().as_u128();
        if order.total_price != total_price {
            warn!(
                "[indexer] market_buy price not match: {} {} {} {}",
                insc.tx_hash, order.order_id, order.total_price, total_price
            );
            return false;
        }

        let tick = &order.tick;
        let transfer_amount = order.amount;
        let market_balance = self.get_token_balance(tick, &insc.to);

        assert!(transfer_amount <= market_balance);

        self.token_balance_change_update(tick, &insc.from, transfer_amount as i64);
        self.token_balance_change_update(tick, &insc.to, -(transfer_amount as i64));

        info!(
            "[indexer] market_buy_token: {} {} {} {} {}",
            insc.tx_hash, order.order_id, insc.from, order.tick, order.amount
        );
        true
    }

    fn execute_market_buy_nft(&mut self, insc: &Inscription, order: &MarketOrder, log: &web3::ethabi::Log) -> bool {
        let total_price = log.get_param("price").unwrap().clone().into_uint().unwrap().as_u128();
        if order.total_price != total_price {
            warn!(
                "[indexer] market_buy price not match: {} {} {} {}",
                insc.tx_hash, order.order_id, order.total_price, total_price
            );
            return false;
        }

        let holder = self.get_inscription_holder(order.nft_id);
        assert!(MARKET_ADDRESS_LIST.contains(&holder));

        let mut trans: Vec<(u64, u64, u64)> = vec![(insc.id, order.nft_id, 0)];

        match self.inscriptions_holder.get_mut(&order.nft_id) {
            Some(holder) => *holder = insc.from.clone(),
            None => {
                self.inscriptions_holder.insert(order.nft_id, insc.from.clone());
            }
        }

        self.inscriptions_transfer.append(&mut trans);

        info!(
            "[indexer] market_buy_nft: {} {} {} {}",
            insc.tx_hash, order.order_id, insc.from, order.nft_id
        );
        true
    }

    fn execute_market_cancel(&mut self, insc: &Inscription, log: &web3::ethabi::Log) -> bool {
        let order_id = "0x".to_string() + &log.get_param("orderId").unwrap().to_string();
        let order = match self.db.read().unwrap().market_get_order_by_id(&order_id) {
            Some(order) => order,
            None => {
                warn!("[indexer] market_cancel order not found: {} {}", insc.tx_hash, order_id);
                return false;
            }
        };

        match order.order_type {
            MarketOrderType::Token => self.execute_market_cancel_token(insc, &order),
            MarketOrderType::NFT => self.execute_market_cancel_nft(insc, &order),
        }
    }

    fn execute_market_cancel_token(&mut self, insc: &Inscription, order: &MarketOrder) -> bool {
        let tick = &order.tick;
        let transfer_amount = order.amount;
        let market_balance = self.get_token_balance(tick, &insc.to);

        assert!(transfer_amount <= market_balance);

        self.token_balance_change_update(tick, &insc.from, transfer_amount as i64);
        self.token_balance_change_update(tick, &insc.to, -(transfer_amount as i64));

        info!(
            "[indexer] market_cancel_token: {} {} {} {} {}",
            insc.tx_hash, order.order_id, insc.from, order.tick, order.amount
        );
        true
    }

    fn execute_market_cancel_nft(&mut self, insc: &Inscription, order: &MarketOrder) -> bool {
        let holder = self.get_inscription_holder(order.nft_id);
        assert!(MARKET_ADDRESS_LIST.contains(&holder));

        let mut trans: Vec<(u64, u64, u64)> = vec![(insc.id, order.nft_id, 0)];

        match self.inscriptions_holder.get_mut(&order.nft_id) {
            Some(holder) => *holder = insc.from.clone(),
            None => {
                self.inscriptions_holder.insert(order.nft_id, insc.from.clone());
            }
        }

        self.inscriptions_transfer.append(&mut trans);

        debug!(
            "[indexer] market_cancel_nft: {} {} {} {}",
            insc.tx_hash, order.order_id, insc.from, order.nft_id
        );
        true
    }

    fn execute_market_set_price(&mut self, insc: &Inscription, log: &web3::ethabi::Log) -> bool {
        let order_id = "0x".to_string() + &log.get_param("orderId").unwrap().to_string();
        if self.db.read().unwrap().market_get_order_by_id(&order_id).is_none() {
            warn!("[indexer] market_set_price order not found: {} {}", insc.tx_hash, order_id);
            return false;
        }

        let total_price_u256 = log.get_param("price").unwrap().clone().into_uint().unwrap();
        let total_price: Result<u128, _> = total_price_u256.try_into();
        match total_price {
            Ok(_) => {
                info!("[indexer] market_set_price: {} {}", insc.tx_hash, order_id);
                true
            }
            Err(_) => {
                warn!("[indexer] market_set_price price overflow: {} {}", insc.tx_hash, order_id);
                false
            }
        }
    }

    fn save_market(&self, db: &TransactionDB, txn: &Transaction<TransactionDB>, insc: &Inscription) {
        if !MARKET_ADDRESS_LIST.contains(&insc.to) {
            return;
        }

        if insc.mime_category == InscriptionMimeCategory::Json
            && insc.json["op"].as_str().unwrap() == APP_OPER_TOKEN_MARKET_LIST
        {
            self.save_market_new_order_token(txn, insc);
        } else if insc.mime_category == InscriptionMimeCategory::Transfer {
            self.save_market_new_order_nft(txn, insc);
        } else if insc.mime_category == InscriptionMimeCategory::Invoke {
            if let Some(log) = insc.event_logs[0].match_event(&CONTRACT_MARKET, "MarketSetPrice") {
                self.save_market_set_price(db, txn, insc, &log);
            } else if let Some(log) = insc.event_logs[0].match_event(&CONTRACT_MARKET, "MarketCancel") {
                self.save_market_cancel(db, txn, insc, &log);
            } else if let Some(log) = insc.event_logs[0].match_event(&CONTRACT_MARKET, "MarketBuy") {
                self.save_market_buy(db, txn, insc, &log);
            }
        }
    }

    fn save_market_new_order_token(&self, txn: &Transaction<TransactionDB>, insc: &Inscription) {
        assert!(insc.market_order_id.is_some());

        let tick = insc.json["tick"].as_str().unwrap();
        let amount = insc.json["amt"].parse_u64().unwrap();

        let order = MarketOrder {
            order_type: MarketOrderType::Token,
            order_id: insc.market_order_id.clone().unwrap(),
            from: insc.from.clone(),
            to: insc.to.clone(),
            nft_id: 0,
            nft_tx: "".to_string(),
            tick: tick.to_string(),
            amount,
            total_price: 0,
            unit_price: 0,
            tx: insc.tx_hash.clone(),
            tx_setprice: "".to_string(),
            tx_cancel: "".to_string(),
            tx_close: "".to_string(),
            blocknumber: insc.blocknumber,
            timestamp: insc.timestamp,
            order_status: MarketOrderStatus::Init,
            buyer: "".to_string(),
        };

        txn.market_order_save(&order);
    }

    fn save_market_new_order_nft(&self, txn: &Transaction<TransactionDB>, insc: &Inscription) {
        assert!(insc.event_logs.len() == 1);

        let log = insc.event_logs[0].match_event(&CONTRACT_MARKET, "MarketList").unwrap();
        let order_id = "0x".to_string() + &"0x".to_string() + &log.get_param("orderId").unwrap().to_string();
        let nft_tx = &insc.mime_data[0..TRANSFER_TX_HEX_LENGTH];
        let nft_id = self.db.read().unwrap().get_inscription_id_by_tx(nft_tx);

        let order = MarketOrder {
            order_type: MarketOrderType::NFT,
            order_id,
            from: insc.from.clone(),
            to: insc.to.clone(),
            nft_id,
            nft_tx: nft_tx.to_string(),
            tick: "".to_string(),
            amount: 0,
            total_price: 0,
            unit_price: 0,
            tx: insc.tx_hash.clone(),
            tx_setprice: "".to_string(),
            tx_cancel: "".to_string(),
            tx_close: "".to_string(),
            blocknumber: insc.blocknumber,
            timestamp: insc.timestamp,
            order_status: MarketOrderStatus::Init,
            buyer: "".to_string(),
        };

        txn.market_order_save(&order);
    }

    fn save_market_set_price(
        &self,
        db: &TransactionDB,
        txn: &Transaction<TransactionDB>,
        insc: &Inscription,
        log: &web3::ethabi::Log,
    ) {
        assert!(insc.event_logs.len() == 1);

        let order_id = "0x".to_string() + &log.get_param("orderId").unwrap().to_string();
        let total_price_u256 = log.get_param("price").unwrap().clone().into_uint().unwrap();
        let total_price = total_price_u256.try_into();

        txn.market_order_set_price(db, &insc.tx_hash, &order_id, total_price.unwrap());
    }

    fn save_market_buy(
        &self,
        db: &TransactionDB,
        txn: &Transaction<TransactionDB>,
        insc: &Inscription,
        log: &web3::ethabi::Log,
    ) {
        assert!(insc.event_logs.len() == 1);

        let order_id = "0x".to_string() + &log.get_param("orderId").unwrap().to_string();

        txn.market_order_close(db, &insc.tx_hash, &order_id, &insc.from);
    }

    fn save_market_cancel(
        &self,
        db: &TransactionDB,
        txn: &Transaction<TransactionDB>,
        insc: &Inscription,
        log: &web3::ethabi::Log,
    ) {
        assert!(insc.event_logs.len() == 1);

        let order_id = "0x".to_string() + &log.get_param("orderId").unwrap().to_string();

        txn.market_order_cancel(db, &insc.tx_hash, &order_id);
    }
}