pub mod structs;

use scrypto::prelude::*;
use common::{_KEEPER_COMPONENT, KEEPER_PACKAGE};
pub use self::structs::*;


#[blueprint]
// #[events(DebugGetInterestRateEvent)]
mod def_interest_model{

    const KEEPER_COMPONENT: ComponentAddress = _KEEPER_COMPONENT;

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
            def_primary: Decimal, 
            def_quadratic: Decimal, 
            stable_coin_primary: Decimal, 
            stable_coin_quadratic:Decimal
        ) -> Global<DefInterestModel>{

            Global::<ValidatorKeeper>::try_from(KEEPER_COMPONENT).expect("keeper component not found");
            
            Self{
                def_primary,
                def_quadratic,
                stable_coin_primary,
                stable_coin_quadratic
            }.instantiate().prepare_to_globalize(OwnerRole::None).globalize()
        }

        pub fn get_interest_rate(&self, 
            borrow_ratio: Decimal, 
            _stable_ratio: Decimal,
            _bond_ratio: Decimal,
            model: InterestModel
        ) -> (Decimal, Decimal){
            let apy = self.get_variable_interest_rate(borrow_ratio, model);
            let validator_apy = Global::<ValidatorKeeper>::from(KEEPER_COMPONENT).get_active_set_apy();
            info!(
                "borrow_ratio: {}, stable_ratio:{}, bond_ratio:{}, apy:{}, validator_apy:{}", 
                borrow_ratio, _stable_ratio, _bond_ratio, apy, validator_apy
            );
            // Runtime::emit_event(DebugGetInterestRateEvent{
            //     variable_rate: apy,
            //     stable_ratio: _stable_ratio,
            //     borrow_ratio,
            //     validator_apy
            // });
            (apy, if apy > validator_apy {apy} else {validator_apy})
        }

        fn get_variable_interest_rate(&self, borrow_ratio: Decimal, model: InterestModel) -> Decimal{
            match model{
                InterestModel::Default => if borrow_ratio > Decimal::ONE {
                    // dec!("0.2") + dec!("0.5")
                    self.def_primary + self.def_quadratic
                }
                else{
                    // 0.2 * r + 0.5 * r**2
                    borrow_ratio.checked_mul(self.def_primary).unwrap().checked_add(
                        borrow_ratio.checked_powi(2).unwrap().checked_mul(self.def_quadratic).unwrap()
                    ).unwrap()
                },
                InterestModel::StableCoin => {
                    let r2 = if borrow_ratio > Decimal::ONE { Decimal::ONE} else{ borrow_ratio.checked_powi(2).unwrap()};
                    let r4 = r2.checked_powi(2).unwrap();
                    let r8 = r2.checked_powi(4).unwrap();
                    // dec!("0.55") * x4  + dec!("0.45")* x8
                    self.stable_coin_primary.checked_mul(r4).unwrap().checked_add(self.stable_coin_quadratic.checked_mul(r8).unwrap()).unwrap()
                }
            }
        }
    }


}

// #[derive(ScryptoSbor, ScryptoEvent)]
// pub struct DebugGetInterestRateEvent{
//     pub borrow_ratio: Decimal,
//     pub stable_ratio: Decimal,
//     pub variable_rate: Decimal,
//     pub validator_apy: Decimal
// }