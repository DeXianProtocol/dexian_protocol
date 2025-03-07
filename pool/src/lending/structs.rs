use scrypto::prelude::*;
use common::List;

#[derive(ScryptoSbor)]
pub struct FixedEpochBond {
    pub epoch_at: u64,
    pub interest: Decimal,
    pub global_id_list: List<NonFungibleGlobalId>
}
