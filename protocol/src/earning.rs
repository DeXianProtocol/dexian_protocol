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
        /// Claims XRD using claim NFTs.
        /// claims matured XRD, and accumulates unmatured claims for instant redemption.
        ///
        /// # Arguments
        /// * `cdp_mgr`: The ComponentAddress of the CollateralDebtManager.
        /// * `claim_nfts`: A vector of NonFungibleBuckets representing claim NFTs.
        ///
        /// # Returns
        ///
        /// A FungibleBucket containing the claimed XRD.
        pub fn claim_xrd(&mut self, cdp_mgr: ComponentAddress, claim_nfts: Vec<NonFungibleBucket>) -> FungibleBucket {

            let mut xrd_bucket = FungibleBucket::new(XRD);

            let mut unmatured_claim_nfts: Vec<NonFungibleBucket> = Vec::new();
            let mut interests: Vec<Decimal> = Vec::new();
            let mut total_claim_amount = Decimal::ZERO;
            let mut total_interest_amount = Decimal::ZERO;
            let current_epoch = Runtime::current_epoch().number();
            let cdp_mgr: Global<CollateralDebtManager> = Global::<CollateralDebtManager>::from(cdp_mgr);
            
            for claim_nft in claim_nfts {
                let nft_addr = claim_nft.resource_address();
                let mut validator: Global<Validator> = get_validator(nft_addr);
                let nft_res_mgr = NonFungibleResourceManager::from(nft_addr);
                
                let nft_id = claim_nft.non_fungible_global_id();
                let unstake_data = nft_res_mgr.get_non_fungible_data::<UnstakeData>(nft_id.local_id());
                let claim_epoch = unstake_data.claim_epoch.number();
                if claim_epoch <= current_epoch {
                    xrd_bucket.put(validator.claim_xrd(claim_nft));
                }
                else{
                    total_claim_amount = total_claim_amount.checked_add(unstake_data.claim_amount).unwrap();
                    
                    let (_, stable_rate, _) = cdp_mgr.get_interest_rate(XRD, total_claim_amount);
                    let remain_epoch = claim_epoch - current_epoch;
                    let principal = calc_principal(
                        unstake_data.claim_amount,
                        stable_rate, 
                        Decimal::from(EPOCH_OF_YEAR),
                        remain_epoch
                    );
                    let interest = unstake_data.claim_amount.checked_sub(principal).unwrap();

                    unmatured_claim_nfts.push(claim_nft);
                    interests.push(interest);
                    total_interest_amount = total_interest_amount.checked_add(interest).unwrap();
                }
            }

            if total_claim_amount > Decimal::ZERO {
                let borrow_amount = total_claim_amount.checked_sub(total_interest_amount).unwrap();
                xrd_bucket.put(cdp_mgr.staking_borrow(XRD, borrow_amount, unmatured_claim_nfts, interests));
            }
        
            xrd_bucket

            // The following event emission is commented out for now, as it is reserved for future use or debugging purposes.
            // Runtime::emit_event(ClaimXrdEvent{
            //     claim_nft_id: nft_id,
            //     validator_addr,
            //     claim_amount,
            //     claim_epoch,
            //     current_epoch
            // });

            // Runtime::emit_event(NftFasterRedeemEvent{
            //     rate: stable_rate,
            //     claim_nft:nft_id,
            //     xrd_amount,
            //     validator_addr,
            //     claim_amount,
            //     claim_epoch,
            //     current_epoch
            // });
        }

        pub fn join(&mut self, validator_addr: ComponentAddress, bucket: FungibleBucket) -> FungibleBucket{
            assert!(
                self.staking_pool.get_underlying_token() == bucket.resource_address(),
                "Unsupported token type! Expected token: {:?}, but received token: {:?}",
                self.staking_pool.get_underlying_token(),
                bucket.resource_address()
            );
            self.staking_pool.contribute(bucket, validator_addr)
        }

        pub fn redeem(&mut self, cdp_mgr: ComponentAddress, validators: Vec<ComponentAddress>,  bucket: FungibleBucket, faster: bool) -> Vec<Bucket>{
            let res_addr = bucket.resource_address();
            // let amount = bucket.amount();
            let claim_nft_buckets = if res_addr == self.dse_token {
                self.staking_pool.redeem(validators, bucket)
            }
            else{
                let mut validator = utils::get_validator(res_addr.clone());
                let mut nfts: Vec<NonFungibleBucket> = Vec::new();
                nfts.push(validator.unstake(bucket));
                nfts
            };
            
            if faster {
                let xrd_bucket = self.claim_xrd(cdp_mgr, claim_nft_buckets);
                // Runtime::emit_event(FasterRedeemEvent{
                //     res_addr,
                //     amount,
                //     validators: validators,
                //     xrd_amount: xrd_bucket.amount()
                // });
                vec!(xrd_bucket.into())
            }
            else{
                // Runtime::emit_event(NormalRedeemEvent{
                //     claim_amount:dec!("0"),
                //     res_addr,
                //     amount,
                //     validators:validators.clone(),
                // });
                claim_nft_buckets.into_iter().map(|nft_bucket| nft_bucket.into()).collect()
            }
        }
    }
}


#[derive(ScryptoSbor, ScryptoEvent)]
pub struct NormalRedeemEvent{
/// resource address of LSUs or DSE
    pub res_addr: ResourceAddress,
    pub amount: Decimal,
    pub validators: Vec<ComponentAddress>,
    pub claim_amount: Decimal
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FasterRedeemEvent{
    pub validators: Vec<ComponentAddress>,
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

