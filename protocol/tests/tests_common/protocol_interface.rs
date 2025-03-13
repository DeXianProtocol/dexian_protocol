#[warn(dead_code)]

use scrypto_test::prelude::*;
use super::*;

pub struct ProtocolInterface{
    pub public_key: Secp256k1PublicKey,
    pub test_account: ComponentAddress,
    pub resources: Resources,
    pub components: Components,
    pub ledger: LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
}

impl ProtocolInterface {
    pub fn new(
        public_key: Secp256k1PublicKey,
        account: ComponentAddress,
        resources: Resources,
        components: Components,
        ledger: LedgerSimulator<NoExtension, InMemorySubstateDatabase>
    ) -> Self{
        Self { public_key, resources, components, ledger, test_account: account }
    }
}