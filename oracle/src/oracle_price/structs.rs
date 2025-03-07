use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone, PartialEq, Debug)]
pub struct QuotePrice {
    pub price: Decimal,
    pub epoch_at: u64
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SetPriceEvent {
    pub res_addr: ResourceAddress,
    pub price: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SetPublicKeyEvent{
    pub pub_key: String
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SetValidityPeriodEvent{
    pub previous: u64,
    pub new_value: u64
}

