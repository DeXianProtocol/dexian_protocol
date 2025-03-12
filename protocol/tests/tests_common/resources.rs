
#![allow(dead_code)]


use scrypto_test::prelude::*;
use scrypto::prelude::Url;

#[derive(Clone)]
pub struct Resources{
    pub owner_resource: ResourceAddress,
    pub owner_role: OwnerRole,
    pub authority_resource: ResourceAddress,
    pub base_authority_resource: ResourceAddress,
    pub base_resource: ResourceAddress,
}

pub fn create_resources(account: ComponentAddress, ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>) -> Resources{
    let owner_resource = ledger.create_fungible_resource(dec!(9), 0, account);
    let owner_role = OwnerRole::Fixed(rule!(allow_all)); 
    let authority_resource = ledger.create_fungible_resource(dec!(1), 18, account);
    let base_authority_resource = ledger.create_fungible_resource(dec!(1), 18, account);

    // let base_resource = create_base_resource(account, owner_role.clone(), base_authority_resource, ledger);
    let base_resource = ledger.create_fungible_resource(dec!(1), 18, account);
    

    Resources { owner_resource, owner_role, authority_resource, base_authority_resource, base_resource }
}

fn create_base_resource(
    account: ComponentAddress,
    owner_role: OwnerRole, 
    base_authority_resource: ResourceAddress, 
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> ResourceAddress {
    let metadata = metadata!(
        init {
            "name" => "Surge USD", updatable;
            "symbol" => "sUSD", updatable;
            "description" => "Surge wrapped USD.", updatable;
            "icon_url" => Url::of("https://surge.trade/images/susd_token.png"), updatable;
            "info_url" => Url::of("https://surge.trade"), updatable;
        }
    );
    let resource_roles = FungibleResourceRoles {
        mint_roles: mint_roles! {
            minter => rule!(require(base_authority_resource));
            minter_updater => rule!(deny_all);
        },
        burn_roles: burn_roles! {
            burner => rule!(allow_all);
            burner_updater => rule!(deny_all);
        },
        ..Default::default()
    };
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            owner_role, 
            true,
            DIVISIBILITY_MAXIMUM,
            resource_roles, 
            metadata, 
            Some(dec!(100000000000)) // None
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success().new_resource_addresses()[0]
}