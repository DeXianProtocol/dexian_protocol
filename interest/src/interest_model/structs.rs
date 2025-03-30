use scrypto::prelude::*;


#[derive(ScryptoSbor, Eq, PartialEq, Debug, Clone)]
pub enum InterestModel {
    Default,
    StableCoin,
    XrdStaking
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct SetParamsEvent{
    pub def_primary: Decimal,
    pub def_quadratic: Decimal,
    pub stable_coin_primary: Decimal,
    pub stable_coin_quadratic: Decimal
}
