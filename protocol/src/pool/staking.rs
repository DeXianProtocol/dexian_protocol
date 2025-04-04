use scrypto::prelude::*;
use common::utils::*;
use common::TO_ZERO;

use std::cmp::min;

#[blueprint]
#[types(FungibleVault)]
#[events(JoinEvent, RebalanceEvent, DseUnstakeEvent)]
mod staking_pool {
    enable_method_auth!{
        roles{
            admin => updatable_by: [];
            protocol_caller => updatable_by: [];
        },
        methods {
            contribute => restrict_to:[protocol_caller];
            redeem => restrict_to:[protocol_caller];

            rebalance => restrict_to:[admin];

            get_redemption_value => PUBLIC;
            get_underlying_token => PUBLIC;
        }
    }

    struct StakingResourePool{
        underlying_token: ResourceAddress,
        staking_unit_res_mgr: FungibleResourceManager,
        lsu_map: HashMap<ComponentAddress, FungibleVault>
    }

    impl StakingResourePool {
        
        pub fn instantiate(
            owner_role: OwnerRole,
            underlying_token: ResourceAddress,
            admin_rule: AccessRule,
            protocol_rule: AccessRule
        ) -> (Global<StakingResourePool>, ResourceAddress) {
            let (address_reservation, address) =
                Runtime::allocate_component_address(StakingResourePool::blueprint_id());

            let staking_unit_res_mgr: FungibleResourceManager = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(init{
                    "pool" => address, locked;
                    "symbol" => "dseXRD", locked;
                    "underlying" => underlying_token, locked;
                    "name" => "DeXian Staking Earning Token ", locked;
                    "icon_url" => "https://dexian.io/images/dse.png", updatable;
                    "info_url" => "https://dexian.io", updatable;

                }))
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(address)));
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(require(global_caller(address)));
                    burner_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let staking_unit_token = staking_unit_res_mgr.address();
            let component = Self {
                lsu_map: HashMap::new(),
                underlying_token,
                staking_unit_res_mgr
            }.instantiate()
            .prepare_to_globalize(owner_role)
            .with_address(address_reservation)
            .metadata(metadata!(init{
                "pool_unit" => staking_unit_token, locked;
                "name" => "DeXian Staking Earning ", locked;
                "icon_url" => "https://dexian.io/images/dse.png", updatable;
                "info_url" => "https://dexian.io", updatable;
                })
            )
            .roles(roles!{
                admin => admin_rule.clone();
                protocol_caller => protocol_rule.clone();
            })
            .globalize();
            
            (component, staking_unit_token)
        }

        pub fn contribute(&mut self, bucket: FungibleBucket, validator_addr: ComponentAddress) -> FungibleBucket{
            #[cfg(feature = "verbose")]
            
            assert_resource(&bucket.resource_address(), &self.underlying_token);
            let (_, _, value_per_unit) = self.get_values();
            let mut validator: Global<Validator> = Global::from(validator_addr);
            let amount = bucket.amount();
            let lsu = validator.stake(bucket);

            let lsu_amount = lsu.amount();
            let join_amount = validator.get_redemption_value(lsu_amount);
            let dse_amount = floor_by_resource(self.staking_unit_res_mgr.address(), join_amount.checked_div(value_per_unit).unwrap());
            let dse_bucket = self.staking_unit_res_mgr.mint(dse_amount);

            let lsu_index = amount / lsu_amount;
            self.put_lsu(&validator_addr, lsu);
            Runtime::emit_event(JoinEvent{
                amount: join_amount,
                validator: validator_addr,
                dse_index: value_per_unit,
                dse_amount: dse_bucket.amount(),
                lsu_index,
                lsu_amount
            });

            dse_bucket
        }

        pub fn rebalance(&mut self, unstake_validator: ComponentAddress, lsu_amount: Decimal, stake_validator_addr: ComponentAddress, stake_bucket: FungibleBucket) -> NonFungibleBucket{
            assert!(self.lsu_map.contains_key(&unstake_validator), "the validator address not exists");
            let lsu_vault = self.lsu_map.get_mut(&unstake_validator).unwrap();
            let mut validator: Global<Validator> = Global::from(unstake_validator);
            let unstake_value = validator.get_redemption_value(lsu_amount);
            let stake_value = stake_bucket.amount();
            assert!(unstake_value.checked_sub(stake_value).unwrap().checked_abs().unwrap() < dec!("1"), "diff exceed 1");
            
            let dust = dec!("0.000001");
            let current_lsu_amount = lsu_vault.amount();
            let diff = current_lsu_amount.checked_sub(lsu_amount).unwrap().checked_abs().unwrap();
            let unstake_bucket =  if diff <= dust {lsu_vault.take_all()} else {lsu_vault.take(lsu_amount)};
            let unstake_lsu_amount = unstake_bucket.amount();
            let claim_nft = validator.unstake(unstake_bucket);
            if diff <= dust{
                self.lsu_map.remove(&unstake_validator);
            }

            let mut stake_validator: Global<Validator> = Global::from(stake_validator_addr.clone());
            let lsu_bucket = stake_validator.stake(stake_bucket);
            let stake_lsu_amount = lsu_bucket.amount();
            self.put_lsu(&stake_validator_addr, lsu_bucket);
            Runtime::emit_event(RebalanceEvent{
                stake_validator: stake_validator_addr,
                stake_amount: stake_value,
                stake_lsu_amount,
                unstake_value,
                unstake_lsu_amount,
                unstake_validator,
            });
            
            claim_nft

        }

        pub fn redeem(&mut self, validators: Vec<ComponentAddress>, bucket: FungibleBucket) -> (Vec<NonFungibleBucket>, Decimal){
            assert_resource(&bucket.resource_address(), &self.staking_unit_res_mgr.address());
            
            let mut nfts: Vec<NonFungibleBucket> = Vec::new();
            let (_, _, value_per_share) = self.get_values();
            let amount = bucket.amount();
            let mut redeem_value = amount.checked_mul(value_per_share).unwrap();
            
            for validator_addr in validators{
                assert!(self.lsu_map.contains_key(&validator_addr), "the validator address not exists");
                
                let lsu_vault = self.lsu_map.get_mut(&validator_addr).unwrap();
                let lsu_amount = lsu_vault.amount();
                let mut validator: Global<Validator> = Global::from(validator_addr.clone());
                let lsu_value = validator.get_redemption_value(lsu_amount);
                let lsu_index = lsu_value.checked_div(lsu_amount).unwrap();
                let unstake_value = min(redeem_value, lsu_value);
                let unstake_lsu_bucket = if unstake_value == lsu_value {lsu_vault.take_all()} else{lsu_vault.take_advanced(unstake_value.checked_div(lsu_index).unwrap(), TO_ZERO)};
                let unstake_lsu_amount = unstake_lsu_bucket.amount();
                Runtime::emit_event(DseUnstakeEvent{
                    validator: validator_addr,
                    unstake_lsu: unstake_lsu_amount,
                    unstake_value
                });

                nfts.push(validator.unstake(unstake_lsu_bucket));
                redeem_value = redeem_value.checked_sub(unstake_value).unwrap();
                if redeem_value <= Decimal::ZERO {
                    break;
                }
            }
            
            assert!(redeem_value == Decimal::ZERO, "Unredeemed balance remaining!");
            self.staking_unit_res_mgr.burn(bucket);
            (nfts, amount.checked_mul(value_per_share).unwrap())
        }

        pub fn get_redemption_value(&self, amount_of_pool_units: Decimal) -> Decimal{
            let(_, _, value_per_unit) = self.get_values();
            amount_of_pool_units.checked_mul(value_per_unit).unwrap()
        }

        pub fn get_underlying_token(&self) -> ResourceAddress{
            self.underlying_token
        }

        fn put_lsu(&mut self, validator_addr: &ComponentAddress, lsu_bucket: FungibleBucket) -> Decimal{
            if self.lsu_map.get(validator_addr).is_some(){
                let v = self.lsu_map.get_mut(validator_addr).unwrap();
                v.put(lsu_bucket);
                v.amount()
            }
            else{
                let lsu_amount = lsu_bucket.amount();
                self.lsu_map.insert(validator_addr.clone(), FungibleVault::with_bucket(lsu_bucket));
                lsu_amount
            }
        }

        fn get_values(&self) -> (Decimal, Decimal, Decimal){
            let total_value = self.sum_current_staked();
            let staking_unit_qty = self.staking_unit_res_mgr.total_supply().unwrap();
            (
                total_value,
                staking_unit_qty,
                if staking_unit_qty.is_zero() {
                    Decimal::ONE
                } else{
                    total_value.checked_div(staking_unit_qty).unwrap()
                }  //value_per_unit
            )
        }

        fn sum_current_staked(&self) -> Decimal {
            self.lsu_map.iter().fold(Decimal::ZERO, |sum, (validator_addr, vault)| {
                let validator: Global<Validator> = Global::from(*validator_addr);
                let latest = validator.get_redemption_value(vault.amount());
                sum.checked_add(latest).unwrap()
            })
        }
    }
}


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct JoinEvent {
    pub amount: Decimal,
    pub validator: ComponentAddress,
    pub lsu_index: Decimal,
    pub lsu_amount: Decimal,
    pub dse_index: Decimal,
    pub dse_amount: Decimal,
}


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct RebalanceEvent {
    pub stake_validator: ComponentAddress,
    pub stake_amount: Decimal,
    pub stake_lsu_amount: Decimal,
    pub unstake_validator: ComponentAddress,
    pub unstake_lsu_amount: Decimal,
    pub unstake_value: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct DseUnstakeEvent {
    pub validator: ComponentAddress,
    pub unstake_lsu: Decimal,
    pub unstake_value: Decimal
}
