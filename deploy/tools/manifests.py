from radix_engine_toolkit import *
from typing import Tuple

def lock_fee(builder: ManifestV1Builder, account: Address, fee: int) -> ManifestV1Builder:
    return builder.account_lock_fee(account, Decimal(str(fee)))

def create_proof_by_fungible_resource(
        builder: ManifestV1Builder,
        account: Address,
        resource_address: ResourceAddresses,
        amount: Decimal
        ) -> ManifestV1Builder:
    return builder.account_create_proof_of_amount(account, resource_address, amount)

def deposit_all(builder: ManifestV1Builder, account: Address) -> ManifestV1Builder:
    return builder.account_deposit_entire_worktop(account)

def withdraw_to_bucket(builder: ManifestV1Builder, account: Address, resource: Address, amount: Decimal, name: str) -> ManifestV1Builder:
    builder = builder.account_withdraw(account, resource, amount)
    builder = builder.take_from_worktop(resource, amount, ManifestBuilderBucket(name))
    return builder

def mint_owner_badge(builder: ManifestV1Builder) -> ManifestV1Builder:
    resource_roles = FungibleResourceRoles(
        mint_roles=None,
        burn_roles=None,
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata: MetadataModuleConfig = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('dexian protocol gov badge'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('OWN'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('With power comes responsibility.'), True),
            'icon_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://dexian.io/images/owner_token.png'), True),
            'info_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://dexian.io'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=OwnerRole.NONE(),
        track_total_supply=True,
        divisibility=0,
        initial_supply=Decimal('9'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def mint_authority(builder: ManifestV1Builder) -> ManifestV1Builder:
    resource_roles = FungibleResourceRoles(
        mint_roles=None,
        burn_roles=None,
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata: MetadataModuleConfig = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Authority'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('AUTH'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('dexian protocol authority.'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=OwnerRole.NONE(),
        track_total_supply=True,
        divisibility=18,
        initial_supply=Decimal('1'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def mint_base_authority(builder: ManifestV1Builder) -> ManifestV1Builder:
    resource_roles = FungibleResourceRoles(
        mint_roles=None,
        burn_roles=None,
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata: MetadataModuleConfig = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Base Authority'), True),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('BAUTH'), True),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('dexian protocol base authority.'), True),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=OwnerRole.NONE(),
        track_total_supply=True,
        divisibility=18,
        initial_supply=Decimal('1'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def create_base(builder: ManifestV1Builder, owner_role: OwnerRole, authority_resource: str) -> ManifestV1Builder:
    resource_roles: FungibleResourceRoles = FungibleResourceRoles(
        mint_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=AccessRule.deny_all()
        ),
        burn_roles=ResourceManagerRole(
            role=AccessRule.allow_all(), 
            role_updater=AccessRule.deny_all()
        ),
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge USD'), False),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('sUSD'), False),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge wrapped USD.'), False),
            'icon_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade/images/susd_token.png'), False),
            'info_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade'), False),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=owner_role,
        track_total_supply=True,
        divisibility=18,
        initial_supply=None,
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def create_lp(builder: ManifestV1Builder, owner_role: OwnerRole, authority_resource: str) -> ManifestV1Builder:
    resource_roles: FungibleResourceRoles = FungibleResourceRoles(
        mint_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=AccessRule.deny_all()
        ),
        burn_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=AccessRule.deny_all()
        ),
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge LP'), False),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('SLP'), False),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge liquidity pool LP token.'), False),
            'icon_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade/images/surge_lp_token.png'), False),
            'info_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade'), False),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=owner_role,
        track_total_supply=True,
        divisibility=18,
        initial_supply=None,
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def create_referral_str(account: Address, owner_amount: str, owner_resource: str, authority_resource: str) -> str:
    return f'''
CALL_METHOD
    Address("{account.as_str()}")
    "lock_fee"
    Decimal("10")
;
CREATE_NON_FUNGIBLE_RESOURCE
    Enum<2u8>(
        Enum<2u8>(
            Enum<0u8>(
                Enum<1u8>(
                    Decimal("{owner_amount}"),
                    Address("{owner_resource}")
                )
            )
        )
    )
    Enum<3u8>()
    true
    Enum<0u8>(
        Enum<0u8>(
            Tuple(
                Array<Enum>(
                    Enum<14u8>(
                        Array<Enum>(
                            Enum<0u8>(
                                12u8
                            ),
                            Enum<0u8>(
                                12u8
                            ),
                            Enum<0u8>(
                                198u8
                            ),
                            Enum<0u8>(
                                192u8
                            ),
                            Enum<0u8>(
                                192u8
                            ),
                            Enum<0u8>(
                                10u8
                            ),
                            Enum<0u8>(
                                10u8
                            ),
                            Enum<0u8>(
                                192u8
                            ),
                            Enum<0u8>(
                                192u8
                            )
                        )
                    )
                ),
                Array<Tuple>(
                    Tuple(
                        Enum<1u8>(
                            "ReferralData"
                        ),
                        Enum<1u8>(
                            Enum<0u8>(
                                Array<String>(
                                    "name",
                                    "description",
                                    "key_image_url",
                                    "fee_referral",
                                    "fee_rebate",
                                    "referrals",
                                    "max_referrals",
                                    "balance",
                                    "total_rewarded"
                                )
                            )
                        )
                    )
                ),
                Array<Enum>(
                    Enum<0u8>()
                )
            )
        ),
        Enum<1u8>(
            0u64
        ),
        Array<String>(
            "name",
            "description",
            "key_image_url",
            "fee_referral",
            "fee_rebate",
            "referrals",
            "max_referrals",
            "balance",
            "total_rewarded"
        )
    )
    Tuple(
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<2u8>(
                        Enum<0u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Address("{authority_resource}")
                                )
                            )
                        )
                    )
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        ),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<1u8>()
                ),
                Enum<0u8>()
            )
        ),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<1u8>()
                ),
                Enum<0u8>()
            )
        ),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<1u8>()
                ),
                Enum<0u8>()
            )
        ),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<1u8>()
                ),
                Enum<0u8>()
            )
        ),
        Enum<0u8>(),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<2u8>(
                        Enum<0u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Address("{authority_resource}")
                                )
                            )
                        )
                    )
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        )
    )
    Tuple(
        Map<String, Tuple>(
            "name" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "Surge Referral"
                    )
                ),
                false
            ),
            "description" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "Surge referral badge that can grant reduced fees and earn rewards."
                    )
                ),
                false
            ),
            "icon_url" => Tuple(
                Enum<1u8>(
                    Enum<13u8>(
                        "https://surge.trade/images/referral_badge.png"
                    )
                ),
                false
            ),
            "info_url" => Tuple(
                Enum<1u8>(
                    Enum<13u8>(
                        "https://surge.trade"
                    )
                ),
                false
            )
        ),
        Map<String, Enum>()
    )
    Enum<0u8>()
;
'''

def create_recovery_key_str(account: Address, owner_amount: str, owner_resource: str, authority_resource: str) -> str:
    return f'''
CALL_METHOD
    Address("{account.as_str()}")
    "lock_fee"
    Decimal("10")
;
CREATE_NON_FUNGIBLE_RESOURCE
    Enum<2u8>(
        Enum<2u8>(
            Enum<0u8>(
                Enum<1u8>(
                    Decimal("{owner_amount}"),
                    Address("{owner_resource}")
                )
            )
        )
    )
    Enum<3u8>()
    true
    Enum<0u8>(
        Enum<0u8>(
            Tuple(
                Array<Enum>(
                    Enum<14u8>(
                        Array<Enum>(
                            Enum<0u8>(
                                12u8
                            ),
                            Enum<0u8>(
                                12u8
                            ),
                            Enum<0u8>(
                                198u8
                            )
                        )
                    )
                ),
                Array<Tuple>(
                    Tuple(
                        Enum<1u8>(
                            "RecoveryKeyData"
                        ),
                        Enum<1u8>(
                            Enum<0u8>(
                                Array<String>(
                                    "name",
                                    "description",
                                    "key_image_url"
                                )
                            )
                        )
                    )
                ),
                Array<Enum>(
                    Enum<0u8>()
                )
            )
        ),
        Enum<1u8>(
            0u64
        ),
        Array<String>(
            "name",
            "description",
            "key_image_url"
        )
    )
    Tuple(
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<2u8>(
                        Enum<0u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Address("{authority_resource}")
                                )
                            )
                        )
                    )
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        ),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<1u8>(
            Tuple(
                Enum<1u8>(
                    Enum<2u8>(
                        Enum<0u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Address("{authority_resource}")
                                )
                            )
                        )
                    )
                ),
                Enum<1u8>(
                    Enum<1u8>()
                )
            )
        )
    )
    Tuple(
        Map<String, Tuple>(
            "name" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "Surge Recovery Key"
                    )
                ),
                false
            ),
            "description" => Tuple(
                Enum<1u8>(
                    Enum<0u8>(
                        "Surge recovery key that can be used to update permissions for your trading account."
                    )
                ),
                false
            ),
            "icon_url" => Tuple(
                Enum<1u8>(
                    Enum<13u8>(
                        "https://surge.trade/images/recovery_key.png"
                    )
                ),
                false
            ),
            "info_url" => Tuple(
                Enum<1u8>(
                    Enum<13u8>(
                        "https://surge.trade"
                    )
                ),
                false
            )
        ),
        Map<String, Enum>()
    )
    Enum<0u8>()
;
'''

def mint_protocol_resource(builder: ManifestV1Builder, owner_role: OwnerRole) -> ManifestV1Builder:
    resource_roles: FungibleResourceRoles = FungibleResourceRoles(
        mint_roles=ResourceManagerRole(
            role=AccessRule.deny_all(),
            role_updater=None
        ),
        burn_roles=ResourceManagerRole(
            role=AccessRule.allow_all(), 
            role_updater=AccessRule.deny_all()
        ),
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge'), False),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('SRG'), False),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge protocol utility token.'), False),
            'icon_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade/images/surge_token.png'), False),
            'info_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade'), False),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=owner_role,
        track_total_supply=True,
        divisibility=18,
        initial_supply=Decimal('100000000'),
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

def create_keeper_reward(builder: ManifestV1Builder, owner_role: OwnerRole, authority_resource: str) -> ManifestV1Builder:
    resource_roles: FungibleResourceRoles = FungibleResourceRoles(
        mint_roles=ResourceManagerRole(
            role=AccessRule.require(ResourceOrNonFungible.RESOURCE(Address(authority_resource))), 
            role_updater=AccessRule.deny_all()
        ),
        burn_roles=ResourceManagerRole(
            role=AccessRule.allow_all(), 
            role_updater=AccessRule.deny_all()
        ),
        freeze_roles=None,
        recall_roles=None,
        withdraw_roles=None,
        deposit_roles=None,
    )
    metadata = MetadataModuleConfig(
        init={
            'name': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge Keeper Reward'), False),
            'symbol': MetadataInitEntry(MetadataValue.STRING_VALUE('SKR'), False),
            'description': MetadataInitEntry(MetadataValue.STRING_VALUE('Surge keeper reward token.'), False),
            'icon_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade/images/surge_keeper_reward_token.png'), False),
            'info_url': MetadataInitEntry(MetadataValue.URL_VALUE('https://surge.trade'), False),
        },
        roles={},
    )

    return builder.create_fungible_resource_manager(
        owner_role=owner_role,
        track_total_supply=True,
        divisibility=18,
        initial_supply=None,
        resource_roles=resource_roles,
        metadata=metadata,
        address_reservation=None,
    )

