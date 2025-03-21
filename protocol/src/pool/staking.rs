use scrypto::prelude::*;
use common::utils::*;


#[blueprint]
#[types(
    FungibleVault
)]
#[events(JoinEvent)]
mod staking_pool {

    enable_method_auth!{
        roles{
            admin => updatable_by: [];
            operator => updatable_by: [];
        },
        methods {
            contribute => restrict_to:[operator, OWNER];
            redeem => restrict_to:[operator, OWNER];

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
            op_rule: AccessRule
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
            // .metadata(metadata! {
            //     // "pool_resources" => vec![underlying_token, staking_unit_token], locked;
            //     "pool_unit" => staking_unit_token, locked;
            //     }
            // )
            .roles(roles!{
                admin => admin_rule.clone();
                operator => op_rule.clone();
            })
            .globalize();
            
            (component, staking_unit_token)
        }

        pub fn contribute(&mut self, bucket: FungibleBucket, validator_addr: ComponentAddress) -> FungibleBucket{
            assert_resource(&bucket.resource_address(), &self.underlying_token);
            let (_, _, value_per_unit) = self.get_values();
            let mut validator: Global<Validator> = Global::from(validator_addr);
            let amount = bucket.amount();
            let lsu = validator.stake(bucket);

            let lsu_amount = lsu.amount();
            let join_amount = validator.get_redemption_value(lsu_amount);
            let unit_amount = floor_by_resource(self.staking_unit_res_mgr.address(), join_amount.checked_div(value_per_unit).unwrap());
            let unit_bucket = self.staking_unit_res_mgr.mint(unit_amount);

            let lsu_index = amount / lsu_amount;
            let _last_lsu = if self.lsu_map.get(&validator_addr).is_some(){
                let v = self.lsu_map.get_mut(&validator_addr).unwrap();
                v.put(lsu);
                v.amount()
            }
            else{
                self.lsu_map.insert(validator_addr.clone(), FungibleVault::with_bucket(lsu));
                lsu_amount
            };
            Runtime::emit_event(JoinEvent{
                amount: join_amount,
                validator: validator_addr,
                dse_index: value_per_unit,
                dse_amount: unit_bucket.amount(),
                lsu_index,
                lsu_amount
            });

            unit_bucket
        }

        pub fn redeem(&mut self, validator_addr: ComponentAddress, bucket: FungibleBucket) -> NonFungibleBucket{
            assert_resource(&bucket.resource_address(), &self.staking_unit_res_mgr.address());
            assert!(self.lsu_map.get(&validator_addr).is_some(), "the validator address not exists");
            let (_, _, value_per_share) = self.get_values();
            let amount = bucket.amount();
            let redeem_value = amount.checked_mul(value_per_share).unwrap();
            
            let lsu = self.lsu_map.get_mut(&validator_addr).unwrap();
            let mut validator: Global<Validator> = Global::from(validator_addr);
            let lsu_amount = lsu.amount();
            let lsu_value = validator.get_redemption_value(lsu_amount);
            
            assert!(redeem_value <= lsu_value, "the target value {} less than expect {}!", lsu_value, redeem_value);
            let lsu_index = lsu_value.checked_div(lsu_amount).unwrap();
            let unstake_lsu_bucket = lsu.take_advanced(redeem_value.checked_div(lsu_index).unwrap(), WithdrawStrategy::Rounded(RoundingMode::ToZero));
            // let unstake_amount = unstake_lsu_bucket.amount();
            let claim_nft = validator.unstake(unstake_lsu_bucket);
            self.staking_unit_res_mgr.burn(bucket);
            claim_nft
            
        }

        pub fn get_redemption_value(&self, amount_of_pool_units: Decimal) -> Decimal{
            let(_, _, value_per_unit) = self.get_values();
            amount_of_pool_units.checked_mul(value_per_unit).unwrap()
        }

        pub fn get_underlying_token(&self) -> ResourceAddress{
            self.underlying_token
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
