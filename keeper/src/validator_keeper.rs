pub mod structs;

use scrypto::prelude::*;
use common::{_AUTHORITY_RESOURCE, _BASE_AUTHORITY_RESOURCE, RESERVE_WEEKS, A_WEEK_EPOCHS, EPOCH_OF_YEAR, BABYLON_START_EPOCH};
pub use self::structs::*;

#[blueprint]
#[types(StakeData)]
//#[events(DebugGetApy, DebugGetApy2)]
mod validator_keeper{
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;
    const BASE_AUTHORITY_RESOURCE: ResourceAddress = _BASE_AUTHORITY_RESOURCE;

    enable_function_auth! {
        instantiate => rule!(require(AUTHORITY_RESOURCE));
    }

    enable_method_auth!{
        roles{
            authority => updatable_by: [];
            operator => updatable_by: [authority];
            admin => updatable_by: [authority];
        },
        methods {
            // admin
            fill_validator_staking => restrict_to: [admin];
            log_validator_staking => restrict_to: [admin];
            insert_validator_staking => restrict_to: [admin];

            // public
            get_active_set_apy => PUBLIC;

        }
    }

    struct ValidatorKeeper{
        validator_map: HashMap<ComponentAddress, Vec<StakeData>>,
    }

    impl ValidatorKeeper {
        pub fn instantiate(
            owner_role: OwnerRole
        ) -> Global<ValidatorKeeper>{
            let admin_rule = rule!(require(AUTHORITY_RESOURCE));
            let op_rule = rule!(require(BASE_AUTHORITY_RESOURCE));
            
            let component = Self{
                validator_map: HashMap::new()
            }.instantiate()
            .prepare_to_globalize(owner_role)
            .roles(
                roles!(
                    admin => admin_rule;
                    operator => op_rule;
                    authority => rule!(require(AUTHORITY_RESOURCE));
                )
            )
            .enable_component_royalties(component_royalties! {
                roles {
                    royalty_setter => OWNER;
                    royalty_setter_updater => OWNER;
                    royalty_locker => OWNER;
                    royalty_locker_updater => rule!(deny_all);
                    royalty_claimer => OWNER;
                    royalty_claimer_updater => OWNER;
                },
                init {
                    fill_validator_staking => Free, locked;
                    log_validator_staking => Free, locked;
                    insert_validator_staking => Free, locked;
                    get_active_set_apy => Usd(dec!(0.1)), updatable;
                }
            })
            .globalize();
            
            component
        }

        pub fn fill_validator_staking(&mut self, validator_addr: ComponentAddress, stake_data_vec: Vec<StakeData>){
            self.validator_map.entry(validator_addr).or_insert(stake_data_vec.clone());
            info!("{}: {},{},{}", Runtime::bech32_encode_address(validator_addr), stake_data_vec[0].last_lsu, stake_data_vec[0].last_staked, stake_data_vec[0].epoch_at);
        }

        pub fn insert_validator_staking(&mut self, validator_addr: ComponentAddress, index:usize,  stake_data: StakeData){
            assert!(self.validator_map.contains_key(&validator_addr), "unknown validator");
            self.validator_map.get_mut(&validator_addr).unwrap().insert(index, stake_data);
        }


        pub fn log_validator_staking(&mut self, add_validator_list: Vec<ComponentAddress>, remove_validator_list: Vec<ComponentAddress>) {
            // Remove validators from the map
            remove_validator_list.iter().for_each(|validator_addr| {
                self.validator_map.remove(validator_addr);
            });
        
            // Update staking information for existing validators
            let current_epoch = Runtime::current_epoch().number();
            let current_week_index = Self::get_week_index(current_epoch);
            let mut current_staked = self.validator_map.iter_mut()
            .map(|(validator_addr, vec)| {
                let validator: Global<Validator> = Global::from(validator_addr.clone());
                let last_lsu = validator.total_stake_unit_supply();
                let last_staked = validator.total_stake_xrd_amount();
                let latest = vec.first_mut().unwrap();
                let last_week_index = Self::get_week_index(latest.epoch_at);
                if current_week_index > last_week_index {
                    vec.insert(0, Self::new_stake_data(last_lsu, last_staked, current_epoch));
                    while vec.capacity() > RESERVE_WEEKS {
                        vec.remove(vec.capacity()-1);
                    }
                }
                else{
                    latest.last_lsu = last_lsu;
                    latest.last_staked = last_staked;
                    latest.epoch_at = current_epoch;
                }
                last_staked
            })
            .fold(Decimal::ZERO, |sum, staked| {
                sum.checked_add(staked).unwrap()
            });

            // Add new validators and update their staking information
            add_validator_list.iter().for_each(|add_validator_addr| {
                if !self.validator_map.contains_key(add_validator_addr) {
                    let staked = self.set_validator_staking(add_validator_addr, current_week_index, current_epoch);
                    current_staked = current_staked.checked_add(staked).unwrap();
                }
            });

        }
        

        fn set_validator_staking(&mut self, validator_addr: &ComponentAddress, current_week_index: usize, current_epoch: u64) -> Decimal{
            let validator: Global<Validator> = Global::from(validator_addr.clone());
            let last_lsu = validator.total_stake_unit_supply();
            let last_staked = validator.total_stake_xrd_amount();
            self.validator_map.entry(validator_addr.clone()).and_modify(|vec|{
                let latest = vec.first_mut().unwrap();
                let last_index = Self::get_week_index(latest.epoch_at);
                if current_week_index > last_index {
                    vec.insert(0, Self::new_stake_data(last_lsu, last_staked, current_epoch));
                    while vec.capacity() > RESERVE_WEEKS {
                        vec.remove(vec.capacity()-1);
                    }
                }
                else{
                    latest.last_lsu = last_lsu;
                    latest.last_staked = last_staked;
                    latest.epoch_at = current_epoch;
                } 

            }).or_insert(Vec::from([Self::new_stake_data(last_lsu, last_staked, current_epoch)]));
            
            last_staked
        }

        fn new_stake_data(last_lsu: Decimal, last_staked: Decimal, epoch_at: u64) -> StakeData{
            StakeData{
                epoch_at,
                last_lsu,
                last_staked
            }
        }

        fn get_week_index(epoch_at: u64) -> usize{
            // let index: I192 = Decimal::from(epoch_at - BABYLON_START_EPOCH).checked_div(Decimal::from(A_WEEK_EPOCHS)).unwrap()
            // .checked_ceiling().unwrap().try_into();
            // ().to_usize()
            let elapsed_epoch = epoch_at - BABYLON_START_EPOCH;
            let week_index = elapsed_epoch / A_WEEK_EPOCHS;
            if week_index * A_WEEK_EPOCHS < elapsed_epoch{
                (week_index + 1) as usize
            }
            else{
                week_index as usize
            }
        }

        pub fn get_active_set_apy(&self) -> Decimal {
            let (sum, count) = self.validator_map.iter()
                .filter_map(|(validator_addr, vec)| {
                    let validator: Global<Validator> = Global::from(validator_addr.clone());
                    let last_staked = validator.total_stake_xrd_amount();
                    let last_lsu = validator.total_stake_unit_supply();
                    self.get_validator_apy(validator_addr, vec, last_staked, last_lsu)
                })
                .fold((Decimal::ZERO, Decimal::ZERO), |(sum, count), apy| {
                    (sum + apy, count + Decimal::ONE)
                });
            info!("sum:{}, count:{}", sum, count);
            if count.is_zero() {
                Decimal::ZERO
            } else {
                sum  / count
            }
        }
        

        fn get_validator_apy(&self, _validator_addr: &ComponentAddress, vec: &Vec<StakeData>, last_staked: Decimal, last_lsu: Decimal) -> Option<Decimal> {
            // let latest = vec.first()?;
            // let latest_week_index = Self::get_week_index(latest.epoch_at);
        
            // // The last entry must be within the last week (inclusive).
            // if latest_week_index < current_week_index -1 {
            //     info!("latest_week_index:{}/{}, current_week_index:{}", latest.epoch_at, latest_week_index, current_week_index);
            //     return None;
            // }
            if let Some(previous) = vec.get(1) {
                let current_epoch = Runtime::current_epoch().number();
                let current_lsu_index = last_staked.checked_div(last_lsu)?;
                let previous_index = previous.last_staked.checked_div(previous.last_lsu)?;
                let delta_index = current_lsu_index.checked_sub(previous_index)?;
                let delta_epoch = Decimal::from(current_epoch - previous.epoch_at);

                info!(
                    "latest_index:{}/{}, previous_index:{}/{}, delta_index:{}, delta_epoch:{}/{}",
                    last_staked, last_lsu,
                    previous.last_staked, previous.last_lsu,
                    delta_index,
                    current_epoch, previous.epoch_at
                );
                return Some(
                    (delta_index).checked_mul(Decimal::from(EPOCH_OF_YEAR)).unwrap()
                    .checked_div(delta_epoch).unwrap()
                );
            }
            None
        }

    }
}