use scrypto::prelude::*;


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct JoinEvent {
    pub amount: Decimal,
    pub validator: ComponentAddress,
    pub lsu_index: Decimal,
    pub lsu_amount: Decimal,
    pub dse_index: Decimal,
    pub dse_amount: Decimal,
}