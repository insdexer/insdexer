pub trait JsonValueTrait {
    fn parse_u64(&self) -> Option<u64>;
}

impl JsonValueTrait for serde_json::Value {
    fn parse_u64(&self) -> Option<u64> {
        if let Some(value) = self.as_u64() {
            return Some(value);
        }
        if let Some(value) = self.as_str() {
            return value.parse::<u64>().ok();
        }
        
        None
    }
}
