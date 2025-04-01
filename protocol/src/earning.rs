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
            let mut matured_nft_cnt: usize = 0;
            let mut matured_claim_amount = Decimal::ZERO;
            let mut unmatured_claim_nfts: Vec<NonFungibleBucket> = Vec::new();
            let mut interests: Vec<Decimal> = Vec::new();
            let mut unmatured_claim_amount = Decimal::ZERO;
            let mut unmatured_interest_amount = Decimal::ZERO;
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
                    matured_nft_cnt += 1;
                    matured_claim_amount = matured_claim_amount.checked_add(unstake_data.claim_amount).unwrap();
                }
                else{
                    unmatured_claim_amount = unmatured_claim_amount.checked_add(unstake_data.claim_amount).unwrap();
                    
                    let (_, stable_rate, _) = cdp_mgr.get_interest_rate(XRD, unmatured_claim_amount);
                    let remain_epoch = claim_epoch - current_epoch;
                    let principal = calc_principal(
                        unstake_data.claim_amount,
                        stable_rate, 
                        Decimal::from(EPOCH_OF_YEAR),
                        remain_epoch
                    );
                    
                    let interest = unstake_data.claim_amount.checked_sub(principal).unwrap();
                    interests.push(interest);
                    unmatured_interest_amount = unmatured_interest_amount.checked_add(interest).unwrap();
                    unmatured_claim_nfts.push(claim_nft);
                }
            }

            if unmatured_claim_amount > Decimal::ZERO {
                let claim_nft_cnt = unmatured_claim_nfts.len();
                let borrow_amount = unmatured_claim_amount.checked_sub(unmatured_interest_amount).unwrap();
                xrd_bucket.put(cdp_mgr.staking_borrow(XRD, borrow_amount, unmatured_claim_nfts, interests));
                Runtime::emit_event(NftFasterRedeemEvent{
                    claim_amount: unmatured_claim_amount,
                    claim_nfts: claim_nft_cnt,
                    xrd_amount: borrow_amount,
                    current_epoch
                });
            }

            if matured_claim_amount > Decimal::ZERO {
                Runtime::emit_event(ClaimXrdEvent{
                    claim_amount: matured_claim_amount,
                    claim_nfts: matured_nft_cnt,
                    current_epoch
                })
            }
        
            xrd_bucket            
        }

        /// Joins a validator with a contribution of fungible tokens.
        ///
        /// This function allows a user to contribute fungible tokens to a validator
        /// through the staking pool. It asserts that the provided bucket contains the
        /// expected underlying token type before contributing.
        ///
        /// # Arguments
        ///
        /// * `validator_addr`: The ComponentAddress of the validator to join.
        /// * `bucket`: The FungibleBucket containing the tokens to contribute.
        ///
        /// # Returns
        ///
        /// A FungibleBucket representing the user's contribution to the validator.
        pub fn join(&mut self, validator_addr: ComponentAddress, bucket: FungibleBucket) -> FungibleBucket{
            assert!(
                self.staking_pool.get_underlying_token() == bucket.resource_address(),
                "Unsupported token type! Expected token: {:?}, but received token: {:?}",
                self.staking_pool.get_underlying_token(),
                bucket.resource_address()
            );
            self.staking_pool.contribute(bucket, validator_addr)
        }

        /// Redeems staking units and claims corresponding rewards.
        ///
        /// This function handles the redemption of staking units from either the staking pool
        /// or individual validators, and processes the resulting claim NFTs. It supports
        /// two modes: 'faster' and 'normal', each with different handling of the claim NFTs.
        ///
        /// # Arguments
        ///
        /// * `cdp_mgr`: The ComponentAddress of the CollateralDebtManager.
        /// * `validators`: A vector of ComponentAddresses for the validators to redeem from.
        /// * `bucket`: The FungibleBucket containing the staking units to redeem.
        /// * `faster`: A boolean flag indicating whether to use the faster redemption mode.
        ///
        /// # Returns
        ///
        /// A vector of Buckets, either containing the claimed XRD in 'faster' mode,
        /// or the claim NFTs in 'normal' mode.
        pub fn redeem(&mut self, cdp_mgr: ComponentAddress, validators: Vec<ComponentAddress>,  bucket: FungibleBucket, faster: bool) -> Vec<Bucket>{
            let res_addr = bucket.resource_address();
            let amount = bucket.amount();
            let (claim_nft_buckets, xrd_amount) = if res_addr == self.dse_token {
                // Redeem from the staking pool if the resource is the DSE token.
                self.staking_pool.redeem(validators, bucket)
            }
            else{
                // Redeem from an individual validator if the resource is not the DSE token.
                let mut validator = get_validator(res_addr.clone());
                let claim_nft = validator.unstake(bucket);
                let nft_id = claim_nft.non_fungible_global_id();
                let nft_res_mgr = NonFungibleResourceManager::from(claim_nft.resource_address());
                let unstake_data = nft_res_mgr.get_non_fungible_data::<UnstakeData>(nft_id.local_id());
                (vec![claim_nft], unstake_data.claim_amount)
            };
            
            if faster {
                let xrd_bucket = self.claim_xrd(cdp_mgr, claim_nft_buckets);
                Runtime::emit_event(FasterRedeemEvent{
                    res_addr,
                    amount,
                    xrd_amount: xrd_bucket.amount()
                });
                vec![xrd_bucket.into()]
            }
            else{
                Runtime::emit_event(NormalRedeemEvent{
                    claim_amount:xrd_amount,
                    res_addr,
                    amount
                });
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
    pub claim_amount: Decimal
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct FasterRedeemEvent{
    pub res_addr: ResourceAddress,
    pub amount: Decimal,
    pub xrd_amount: Decimal
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct NftFasterRedeemEvent{
    pub claim_amount: Decimal,
    pub xrd_amount: Decimal,
    pub claim_nfts: usize,
    pub current_epoch: u64
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ClaimXrdEvent{
    pub claim_nfts: usize,
    pub claim_amount: Decimal,
    pub current_epoch: u64
}

