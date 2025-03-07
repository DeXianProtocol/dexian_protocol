use scrypto::prelude::*;


#[derive(ScryptoSbor, Eq, PartialEq, Debug, Clone)]
pub enum InterestModel {
    Default,
    StableCoin
}