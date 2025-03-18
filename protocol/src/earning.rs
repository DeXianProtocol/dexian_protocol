use scrypto::prelude::*;
use common::*;
use keeper::UnstakeData;
use crate::pool::staking::staking_pool::StakingResourePool;
use crate::cdp::cdp_mgr::CollateralDebtManager;



#[blueprint]
#[events(NormalRedeemEvent, FasterRedeemEvent, NftFasterRedeemEvent, ClaimXrdEvent)]
mod staking_earning {

    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;
    const BASE_AUTHORITY_RESOURCE: ResourceAddress = _BASE_AUTHORITY_RESOURCE;

    enable_function_auth! {
        instantiate => rule!(require(AUTHORITY_RESOURCE));
    }

    enable_method_auth! {
        roles{
            authority => updatable_by: [];
            admin => updatable_by: [authority];
            operator => updatable_by: [authority];
        },
        methods {            
            join => PUBLIC;
            claim_xrd => PUBLIC;
            redeem => PUBLIC;
        }
    }

    struct StakingEarning{
        staking_pool: Global<StakingResourePool>,
        dse_token: ResourceAddress
    }

    impl StakingEarning{

        pub fn instantiate(
            owner_role: OwnerRole,
        ) -> Global<StakingEarning>{
            let admin_rule = rule!(require(AUTHORITY_RESOURCE));
            let op_rule = rule!(require(BASE_AUTHORITY_RESOURCE));
            let (address_reservation, component_address) = Runtime::allocate_component_address(
                StakingEarning::blueprint_id()
            );
            let caller_rule = rule!(require(global_caller(component_address)));
            let (staking_pool,dse_token) = StakingResourePool::instantiate(owner_role.clone(), XRD, admin_rule.clone(), caller_rule);
            
            let component = Self{
                staking_pool,
                dse_token
            }.instantiate()
            .prepare_to_globalize(owner_role)
            .with_address(address_reservation)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
                admin => admin_rule.clone();
                operator => op_rule.clone();
            })
            .globalize();
            component
        }

        ///
        /// claim xrd with claimNFT
        /// 
        pub fn claim_xrd(&mut self, cdp_mgr: ComponentAddress, claim_nft: NonFungibleBucket) -> (FungibleBucket, Decimal){
            assert!(claim_nft.amount() == Decimal::ONE, "claim_nft cannot be empty");
            let nft_addr = claim_nft.resource_address();
            let mut validator: Global<Validator> = get_validator(nft_addr);
            let validator_addr = validator.address();
            let res_mgr = NonFungibleResourceManager::from(nft_addr);
            let current_epoch = Runtime::current_epoch().number();
            let nft_id = claim_nft.non_fungible_global_id();
            let unstake_data = res_mgr.get_non_fungible_data::<UnstakeData>(nft_id.local_id());
            let claim_amount = unstake_data.claim_amount;
            let claim_epoch = unstake_data.claim_epoch.number();            
            if claim_epoch <= current_epoch {
                let bucket = validator.claim_xrd(claim_nft);
                Runtime::emit_event(ClaimXrdEvent{
                    claim_nft_id: nft_id,
                    validator_addr,
                    claim_amount,
                    claim_epoch,
                    current_epoch
                });
                (bucket, claim_amount)
            }
            else{
                let cdp_mgr: Global<CollateralDebtManager> = Global::<CollateralDebtManager>::from(cdp_mgr);
                let (_, stable_rate, _) = cdp_mgr.get_interest_rate(XRD, unstake_data.claim_amount);
                let remain_epoch = claim_epoch - current_epoch;
                let principal = calc_principal(unstake_data.claim_amount, stable_rate, Decimal::from(EPOCH_OF_YEAR), remain_epoch);
                let borrow_bucket = cdp_mgr.staking_borrow(XRD, principal, claim_nft, unstake_data.claim_amount.checked_sub(principal).unwrap()); 
                let xrd_amount = borrow_bucket.amount();
                Runtime::emit_event(NftFasterRedeemEvent{
                    rate: stable_rate,
                    claim_nft:nft_id,
                    xrd_amount,
                    validator_addr,
                    claim_amount,
                    claim_epoch,
                    current_epoch
                });
                (borrow_bucket, claim_amount)
            }
        }

        pub fn join(&mut self, validator_addr: ComponentAddress, bucket: FungibleBucket) -> FungibleBucket{
            assert!(self.staking_pool.get_underlying_token() == bucket.resource_address(), "the unsupported token!");
            let unit_bucket = self.staking_pool.contribute(bucket, validator_addr);
            unit_bucket
        }

        pub fn redeem(&mut self, cdp_mgr: ComponentAddress, validator_addr: ComponentAddress,  bucket: FungibleBucket, faster: bool) -> Bucket{
            let res_addr = bucket.resource_address();
            let amount = bucket.amount();
            let claim_nft_bucket = if res_addr == self.dse_token {
                self.staking_pool.redeem(validator_addr, bucket)
            }
            else{
                let mut validator: Global<Validator> = Global::from(validator_addr);
                validator.unstake(bucket)
            };
            
            if faster {
                let (xrd_bucket, _) = self.claim_xrd(cdp_mgr, claim_nft_bucket);
                Runtime::emit_event(FasterRedeemEvent{
                    res_addr,
                    amount,
                    validator_addr,
                    xrd_amount: xrd_bucket.amount()
                });
                xrd_bucket.into()
            }
            else{
                let claim_nft_id = claim_nft_bucket.non_fungible_global_id();
                let unstake_data = NonFungibleResourceManager::from(claim_nft_bucket.resource_address()).get_non_fungible_data::<UnstakeData>(claim_nft_id.local_id());
                let claim_amount = unstake_data.claim_amount;
                Runtime::emit_event(NormalRedeemEvent{
                    res_addr,
                    amount,
                    validator_addr,
                    claim_nft_id,
                    claim_amount
                });
                claim_nft_bucket.into()
            }
        }

    }
}


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct NormalRedeemEvent{
/// resource address of LSUs or DSE
    pub res_addr: ResourceAddress,
    pub amount: Decimal,
    pub validator_addr: ComponentAddress,
    pub claim_nft_id: NonFungibleGlobalId,
    pub claim_amount: Decimal
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FasterRedeemEvent{
    pub validator_addr: ComponentAddress,
    pub res_addr: ResourceAddress,
    pub amount: Decimal,
    pub xrd_amount: Decimal
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct NftFasterRedeemEvent{
    pub rate: Decimal,
    pub validator_addr: ComponentAddress,
    pub claim_nft: NonFungibleGlobalId,
    pub claim_amount: Decimal,
    pub claim_epoch: u64,
    pub xrd_amount: Decimal,
    pub current_epoch: u64
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ClaimXrdEvent{
    pub validator_addr: ComponentAddress,
    pub claim_nft_id: NonFungibleGlobalId,
    pub claim_amount: Decimal,
    pub claim_epoch: u64,
    pub current_epoch: u64
}

