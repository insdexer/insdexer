use super::{Web3ABILogEvent, Web3LogEvent};
use web3::ethabi::RawLog;

impl Web3LogEvent for web3::types::Log {
    fn match_event(&self, contract: &web3::ethabi::Contract, event_name: &str) -> Option<web3::ethabi::Log> {
        let event = contract.event(event_name).unwrap();
        let raw_log = RawLog {
            topics: self.topics.clone(),
            data: self.data.0.clone(),
        };

        if let Ok(decode_log) = event.parse_log(raw_log) {
            Some(decode_log)
        } else {
            None
        }
    }
}

impl Web3ABILogEvent for web3::ethabi::Log {
    fn get_param(&self, name: &str) -> Option<&web3::ethabi::Token> {
        if let Some(param) = self.params.iter().find(|param| param.name == name) {
            Some(&param.value)
        } else {
            None
        }
    }
}
