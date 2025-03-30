pub mod structs;

use scrypto::prelude::*;
use common::{_KEEPER_COMPONENT, _AUTHORITY_RESOURCE, KEEPER_PACKAGE};
pub use self::structs::*;


#[blueprint]
#[events(SetParamsEvent)]
mod def_interest_model{

    const KEEPER_COMPONENT: ComponentAddress = _KEEPER_COMPONENT;
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_function_auth! {
        instantiate => rule!(require(AUTHORITY_RESOURCE));
    }

    enable_method_auth!{
        roles{
            authority => updatable_by:[];
            admin => updatable_by: [authority];
        },
        methods {
            //admin
            set_params => restrict_to: [admin];
            
            //public
            get_interest_rate => PUBLIC;
        }
    }

    extern_blueprint! {
        KEEPER_PACKAGE,
        ValidatorKeeper {
            fn get_active_set_apy(&self) -> Decimal;
        }
    }

    
    struct DefInterestModel{
        def_primary: Decimal,
        def_quadratic: Decimal,
        stable_coin_primary: Decimal,
        stable_coin_quadratic: Decimal
    }
    

    impl DefInterestModel {

        pub fn instantiate(
            owner_role: OwnerRole,
            def_primary: Decimal, 
            def_quadratic: Decimal, 
            stable_coin_primary: Decimal, 
            stable_coin_quadratic:Decimal
        ) -> Global<DefInterestModel>{

            Global::<ValidatorKeeper>::try_from(KEEPER_COMPONENT).expect("keeper component not found");
            let admin_rule = rule!(require(AUTHORITY_RESOURCE));
            Self{
                def_primary,
                def_quadratic,
                stable_coin_primary,
                stable_coin_quadratic
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(
                roles!(
                    admin => admin_rule;
                    authority => rule!(require(AUTHORITY_RESOURCE));
                )
            )
            .globalize()
        }

        pub fn get_interest_rate(&self, 
            borrow_ratio: Decimal, 
            _stable_ratio: Decimal,
            _bond_ratio: Decimal,
            model: InterestModel
        ) -> (Decimal, Decimal){
            
            match model{
                InterestModel::Default => {
                    let interest_rate = self.get_default_variable_interest(borrow_ratio);
                    (interest_rate, interest_rate)
                },
                InterestModel::StableCoin => {
                    let interest_rate = self.get_stablecoin_variable_interest(borrow_ratio);
                    (interest_rate, interest_rate)
                },
                InterestModel::XrdStaking => {
                    let interest_rate = self.get_default_variable_interest(borrow_ratio);
                    let validator_apy = Global::<ValidatorKeeper>::from(KEEPER_COMPONENT).get_active_set_apy();
                    info!(
                        "borrow_ratio: {}, stable_ratio:{}, bond_ratio:{}, apy:{}, validator_apy:{}", 
                        borrow_ratio, _stable_ratio, _bond_ratio, interest_rate, validator_apy
                    );
                    (interest_rate, if interest_rate > validator_apy {interest_rate} else {validator_apy})
                }
            }
        }

        pub fn set_params(&mut self, def_primary: Decimal, def_quadratic: Decimal, stable_coin_primary: Decimal, stable_coin_quadratic:Decimal){
            self.def_primary = def_primary;
            self.def_quadratic = def_quadratic;
            self.stable_coin_primary = stable_coin_primary;
            self.stable_coin_quadratic = stable_coin_quadratic;
            Runtime::emit_event(SetParamsEvent{
                def_primary,
                def_quadratic,
                stable_coin_primary,
                stable_coin_quadratic
            });
        }

        fn get_default_variable_interest(&self, borrow_ratio: Decimal) -> Decimal{
            if borrow_ratio > Decimal::ONE {
                // dec!("0.2") + dec!("0.5")
                self.def_primary + self.def_quadratic
            }
            else{
                // 0.2 * r + 0.5 * r**2
                borrow_ratio.checked_mul(self.def_primary).unwrap().checked_add(
                    borrow_ratio.checked_powi(2).unwrap().checked_mul(self.def_quadratic).unwrap()
                ).unwrap()
            }
        }

        fn get_stablecoin_variable_interest(&self, borrow_ratio: Decimal) -> Decimal{
            let r2 = if borrow_ratio > Decimal::ONE { Decimal::ONE} else{ borrow_ratio.checked_powi(2).unwrap()};
            let r4 = r2.checked_powi(2).unwrap();
            let r8 = r2.checked_powi(4).unwrap();
            // dec!("0.55") * x4  + dec!("0.45")* x8
            self.stable_coin_primary.checked_mul(r4).unwrap().checked_add(self.stable_coin_quadratic.checked_mul(r8).unwrap()).unwrap()
        }
    }


}
