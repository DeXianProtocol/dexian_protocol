
use scrypto::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ScryptoSbor)]
pub struct StakeData{
    pub last_lsu: Decimal,
    pub last_staked: Decimal,
    pub epoch_at: u64
}


#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct UnstakeData {
    pub name: String,

    /// An epoch number at (or after) which the pending unstaked XRD may be claimed.
    /// Note: on unstake, it is fixed to be [`ConsensusManagerConfigSubstate.num_unstake_epochs`] away.
    pub claim_epoch: Epoch,

    /// An XRD amount to be claimed.
    pub claim_amount: Decimal,
}