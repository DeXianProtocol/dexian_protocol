use scrypto::prelude::*;

#[blueprint]
mod faucet_mod {
    enable_method_auth! {
        roles {
            user => updatable_by: [OWNER];
        },
        methods {
            update_mint_amount => restrict_to: [OWNER];
            admin_mint_token => restrict_to: [OWNER];
            free_tokens => restrict_to: [user];
        }
    }

    struct Faucet {
        mint_amounts: HashMap<ResourceAddress, Decimal>,
    }

    impl Faucet {
        pub fn new() -> (Global<Faucet>, FungibleBucket) {
            let (component_reservation, this) = Runtime::allocate_component_address(Faucet::blueprint_id());
            
            let owner_token: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {
                    init {
                        "name" => "FaucetOwner", locked;
                        "symbol" => "FOT", locked;
                    }
                })
                .mint_initial_supply(1)
                .into();

            let owner_role = OwnerRole::Updatable(rule!(require(owner_token.resource_address())));

            
            let usdc_manager = ResourceBuilder::new_fungible(owner_role.clone())
                .divisibility(6)
                .metadata(metadata! {
                    init {
                        "name" => "Wrapped USDC", updatable;
                        "symbol" => "xUSDC", updatable;
                        "description" => "Wrapped USDC", updatable;
                        "icon_url" => Url::of("https://assets.instabridge.io/tokens/icons/xUSDC.png"), updatable;
                        "info_url" => Url::of("https://assets.instabridge.io/tokens/info/xUSDC"), updatable;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(this)));
                    minter_updater => OWNER;
                })
                .recall_roles(recall_roles! {
                    recaller => OWNER;
                    recaller_updater => OWNER;
                })
                .create_with_no_initial_supply();
            let usdt_manager = ResourceBuilder::new_fungible(owner_role.clone())
                .divisibility(6)
                .metadata(metadata! {
                    init {
                        "name" => "Wrapped USDT", updatable;
                        "symbol" => "xUSDT", updatable;
                        "description" => "Wrapped USDT", updatable;
                        "icon_url" => Url::of("https://assets.instabridge.io/tokens/icons/xUSDT.png"), updatable;
                        "info_url" => Url::of("https://assets.instabridge.io/tokens/info/xUSDT"), updatable;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(this)));
                    minter_updater => OWNER;
                })
                .recall_roles(recall_roles! {
                    recaller => OWNER;
                    recaller_updater => OWNER;
                })
                .create_with_no_initial_supply();

            let mint_amounts = hashmap! {
                usdc_manager.address() => dec!(100),
                usdt_manager.address() => dec!(100),
            };

            let faucet = Self {
                mint_amounts
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                user => rule!(allow_all);
            })
            .with_address(component_reservation)
            .globalize();

            (faucet, owner_token)
        }

        pub fn update_mint_amount(&mut self, resource: ResourceAddress, amount: Decimal) {
            self.mint_amounts.insert(resource, amount);
        }

        pub fn admin_mint_token(&mut self, resource: ResourceAddress, amount: Decimal) -> FungibleBucket {
            let manager = FungibleResourceManager::from(resource);
            let token = manager.mint(amount);
            token.into()
        }

        pub fn free_tokens(&mut self) -> Vec<FungibleBucket> {
            let mut tokens = vec![];
            for (&resource, &amount) in self.mint_amounts.iter() {
                if amount.is_positive() {
                    let manager = FungibleResourceManager::from(resource);
                    let token = manager.mint(amount);
                    tokens.push(token);
                }
            }
            tokens
        }
    }
}
