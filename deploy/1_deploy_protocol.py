import qrcode
from typing import Tuple, Dict, List
import io
import radix_engine_toolkit as ret
import asyncio
import datetime
import json
from os.path import dirname, join, realpath
from os import makedirs, chdir, environ
from aiohttp import ClientSession, TCPConnector
from subprocess import run
from dotenv import load_dotenv
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, create_proof_by_fungible_resource, deposit_all, mint_owner_badge, mint_authority, mint_base_authority
from tools.manifests import create_base, mint_protocol_resource, create_keeper_reward, create_lp, create_referral_str, create_recovery_key_str
timestamp = datetime.datetime.now().strftime("%Y%m%d%H")

def clean(name: str) -> None:
    path = join(dirname(dirname(realpath(__file__))), name)
    print(f'Clean: {path}')
    run(['cargo', 'clean'], cwd=path, check=True)

def build(name: str, envs: list, network: str) -> Tuple[bytes, bytes]:
    path = join(dirname(dirname(realpath(__file__))), name)
    print(f'Build: {path}')
    
    env = environ.copy()
    env.update({str(key): str(value) for key, value in envs})
    run(['scrypto', 'build'], env=env, cwd=path, check=True)

    # run(['docker', 'run', 
    #     '-v', f'/root/surge-scrypto/{name}:/src',
    #     '-v', f'/root/surge-scrypto/radixdlt-scrypto:/radixdlt-scrypto',
    #     '-v', f'/root/surge-scrypto/common:/common',
    #     '-v', f'/root/surge-scrypto/oracle:/oracle',
    #     '-v', f'/root/surge-scrypto/config:/config', 
    #     '-v', f'/root/surge-scrypto/account:/account',
    #     '-v', f'/root/surge-scrypto/permission_registry:/permission_registry',
    #     '-v', f'/root/surge-scrypto/pool:/pool',
    #     '-v', f'/root/surge-scrypto/referral_generator:/referral_generator',
    #     ] + 
    # [item for pair in [[f'-e', f'{key}={value}'] for key, value in envs] for item in pair] + 
    # ['radixdlt/scrypto-builder:v1.2.0'],        
    #     check=True
    # )

    code, definition = None, None
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.wasm'), 'rb') as f:
        code = f.read()
    with open(join(path, f'target/wasm32-unknown-unknown/release/{name}.rpd'), 'rb') as f:
        definition = f.read()

    release_path = join(dirname(dirname(realpath(__file__))), 'releases')
    makedirs(release_path, exist_ok=True)
    release_path = join(release_path, timestamp + '_' + network)
    makedirs(release_path, exist_ok=True)

    with open(join(release_path, f'{name}.wasm'), 'wb') as f:
        f.write(code)
    with open(join(release_path, f'{name}.rpd'), 'wb') as f:
        f.write(definition)
    return code, definition

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        oracle_key_0 = 'a5bc3d9296bda1e52f96bf0a65238998877dbddb0703bd37ef1f18a6ffce458a'

        clean('common')
        clean("faucet")
        clean('keeper')
        clean('interest')
        clean('oracle')
        clean('protocol')
        

        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

        print('ACCOUNT:', account.as_str())
        balance = await gateway.get_xrd_balance(account)
        if balance < 10000:
            if network_config['network_name'] == 'stokenet':
                builder = ret.ManifestV1Builder()
                builder = builder.call_method(
                    ret.ManifestBuilderAddress.STATIC(ret.Address(network_config['faucet'])),
                    'lock_fee',
                    [ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal('100'))]
                )
                builder = builder.call_method(
                    ret.ManifestBuilderAddress.STATIC(ret.Address(network_config['faucet'])),
                    'free',
                    []
                )
                builder = deposit_all(builder, account)

                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
            else:
                print('FUND ACCOUNT:', account.as_str())
                qr = qrcode.QRCode()
                qr.add_data(account.as_str())
                f = io.StringIO()
                qr.print_ascii(out=f)
                f.seek(0)
                print(f.read())
            
                while balance < 30000:
                    await asyncio.sleep(5)
                    balance = await gateway.get_xrd_balance(account)

        state_version = await gateway.get_state_version()
        print('STATE_VERSION:', state_version)

        if network_config['network_name'] == 'stokenet':
            config_path = join(path, 'stokenet.config.json')
        elif network_config['network_name'] == 'mainnet':
            config_path = join(path, 'mainnet.config.json')
        else:
            raise ValueError(f'Unsupported network: {network_config["network_name"]}')

        try:
            with open(config_path, 'r') as config_file:
                config_data = json.load(config_file)
        except FileNotFoundError:
            config_data = {}
        envs = [
            ('NETWORK_ID', network_config['network_id']),
        ]

        try:
            if 'OWNER_RESOURCE' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = mint_owner_badge(builder)
                builder = deposit_all(builder, account)

                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['OWNER_RESOURCE'] = addresses[0]

            owner_resource = config_data['OWNER_RESOURCE']
            envs.append(('OWNER_RESOURCE', owner_resource))
            print('OWNER_RESOURCE:', owner_resource)

            owner_amount = '4'
            owner_role = ret.OwnerRole.UPDATABLE(ret.AccessRule.require_amount(ret.Decimal(owner_amount), ret.Address(owner_resource)))
            manifest_owner_role = ret.ManifestBuilderValue.ENUM_VALUE(2, 
                [ret.ManifestBuilderValue.ENUM_VALUE(2, 
                    [ret.ManifestBuilderValue.ENUM_VALUE(0, 
                        [ret.ManifestBuilderValue.ENUM_VALUE(1, 
                            [   
                                ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal(owner_amount)),
                                ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(owner_resource)))
                            ]
                        )]
                    )]
                )]
            )

            if 'AUTHORITY_RESOURCE' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = mint_authority(builder)
                builder = deposit_all(builder, account)
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['AUTHORITY_RESOURCE'] = addresses[0]

            authority_resource = config_data['AUTHORITY_RESOURCE']
            envs.append(('AUTHORITY_RESOURCE', authority_resource))
            print('AUTHORITY_RESOURCE:', authority_resource)

            # base_authority_resource --> admin
            if 'BASE_AUTHORITY_RESOURCE' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = mint_base_authority(builder)
                builder = deposit_all(builder, account)
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['BASE_AUTHORITY_RESOURCE'] = addresses[0]

            base_authority_resource = config_data['BASE_AUTHORITY_RESOURCE']
            envs.append(('BASE_AUTHORITY_RESOURCE', base_authority_resource))
            print('BASE_AUTHORITY_RESOURCE:', base_authority_resource)

            # base_resource = op
            if 'BASE_RESOURCE' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = create_base(builder, owner_role, base_authority_resource)
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['BASE_RESOURCE'] = addresses[0]

            base_resource = config_data['BASE_RESOURCE']
            envs.append(('BASE_RESOURCE', base_resource))
            print('BASE_RESOURCE:', base_resource)

            if network_config['network_name'] == 'stokenet':
                if 'FAUCET_PACKAGE' not in config_data:
                    code, definition = build('faucet', envs, network_config['network_name'])
                    payload, intent = await gateway.build_publish_transaction(
                        account,
                        code,
                        definition,
                        ret.OwnerRole.NONE(),
                        public_key,
                        private_key,
                    )
                    await gateway.submit_transaction(payload)
                    addresses = await gateway.get_new_addresses(intent)
                    config_data['FAUCET_PACKAGE'] = addresses[0]

                faucet_package = config_data['FAUCET_PACKAGE']
                print('FAUCET_PACKAGE:', faucet_package)

                if 'FAUCET_COMPONENT' not in config_data:
                    builder = ret.ManifestV1Builder()
                    builder = lock_fee(builder, account, 100)
                    builder = builder.call_function(
                        ret.ManifestBuilderAddress.STATIC(ret.Address(faucet_package)),
                        'Faucet',
                        'new',
                        []
                    )
                    builder = deposit_all(builder, account)
                    payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                    await gateway.submit_transaction(payload)
                    addresses = await gateway.get_new_addresses(intent)
                    config_data['FAUCET_COMPONENT'] = addresses[0]
                    config_data['FAUCET_OWNER_RESOURCE'] = addresses[1]
                    config_data['USDC_RESOURCE'] = addresses[2]
                    config_data['USDT_RESOURCE'] = addresses[3]

                faucet_component = config_data['FAUCET_COMPONENT']
                faucet_owner_resource = config_data['FAUCET_OWNER_RESOURCE']
                usdc_resource = config_data['USDC_RESOURCE']
                usdt_resource = config_data['USDT_RESOURCE']
                print('FAUCET_COMPONENT:', faucet_component)
                print('FAUCET_OWNER_RESOURCE:', faucet_owner_resource)
                print('USDC_RESOURCE:', usdc_resource)
                print('USDT_RESOURCE:', usdt_resource)

            if 'KEEPER_PACKAGE' not in config_data:
                code, definition = build('keeper', envs, network_config['network_name'])
                payload, intent = await gateway.build_publish_transaction(
                    account,
                    code,
                    definition,
                    owner_role,
                    public_key,
                    private_key,
                )
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['KEEPER_PACKAGE'] = addresses[0]

            keeper_package = config_data['KEEPER_PACKAGE']
            envs.append(('KEEPER_PACKAGE', keeper_package))
            print('KEEPER_PACKAGE:', keeper_package)

            if 'KEEPER_COMPONENT' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = create_proof_by_fungible_resource(builder, account, ret.Address(authority_resource), ret.Decimal("1"))
                builder = builder.call_function(
                    ret.ManifestBuilderAddress.STATIC(ret.Address(keeper_package)),
                    'ValidatorKeeper',
                    'instantiate',
                    [manifest_owner_role]
                )
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['KEEPER_COMPONENT'] = addresses[0]

            keeper_component = config_data['KEEPER_COMPONENT']
            envs.append(('KEEPER_COMPONENT', keeper_component))
            print('KEEPER_COMPONENT:', keeper_component)

            if 'INTEREST_PACKAGE' not in config_data:
                code, definition = build('interest', envs, network_config['network_name'])
                payload, intent = await gateway.build_publish_transaction(
                    account,
                    code,
                    definition,
                    owner_role,
                    public_key,
                    private_key,
                )
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['INTEREST_PACKAGE'] = addresses[0]

            interest_package = config_data['INTEREST_PACKAGE']
            envs.append(('INTEREST_PACKAGE', interest_package))
            print('INTEREST_PACKAGE:', interest_package)

            if 'INTEREST_COMPONENT' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = create_proof_by_fungible_resource(builder, account, ret.Address(authority_resource), ret.Decimal("1"))
                builder = builder.call_function(
                    ret.ManifestBuilderAddress.STATIC(ret.Address(interest_package)),
                    'DefInterestModel',
                    'instantiate',
                    [
                        manifest_owner_role,
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal("0.2")),
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal("0.5")),
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal("0.55")),
                        ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal("0.45"))
                    ]
                )
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['INTEREST_COMPONENT'] = addresses[0]

            interest_component = config_data['INTEREST_COMPONENT']
            envs.append(('INTEREST_COMPONENT', interest_component))
            print('INTEREST_COMPONENT:', interest_component)

            if 'ORACLE_PACKAGE' not in config_data:
                code, definition = build('oracle', envs, network_config['network_name'])
                payload, intent = await gateway.build_publish_transaction(
                    account,
                    code,
                    definition,
                    owner_role,
                    public_key,
                    private_key,
                )
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['ORACLE_PACKAGE'] = addresses[0]

            oracle_package = config_data['ORACLE_PACKAGE']
            envs.append(('ORACLE_PACKAGE', oracle_package))
            print('ORACLE_PACKAGE:', oracle_package)

            if 'ORACLE_COMPONENT' not in config_data:
                oracle_key_bytes_0 = ret.ManifestBuilderValue.STRING_VALUE(oracle_key_0)
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = create_proof_by_fungible_resource(builder, account, ret.Address(authority_resource), ret.Decimal("1"))
                builder = builder.call_function(
                    ret.ManifestBuilderAddress.STATIC(ret.Address(oracle_package)),
                    'PriceOracle',
                    'instantiate',
                    [
                        manifest_owner_role, 
                        oracle_key_bytes_0,
                        ret.ManifestBuilderValue.U64_VALUE(3000)
                    ]
                )
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['ORACLE_COMPONENT'] = addresses[0]

            oracle_component = config_data['ORACLE_COMPONENT']
            envs.append(('ORACLE_COMPONENT', oracle_component))
            print('ORACLE_COMPONENT:', oracle_component)

            if 'PROTOCOL_PACKAGE' not in config_data:
                code, definition = build('protocol', envs, network_config['network_name'])
                payload, intent = await gateway.build_publish_transaction(
                    account,
                    code,
                    definition,
                    owner_role,
                    public_key,
                    private_key,
                )
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['PROTOCOL_PACKAGE'] = addresses[0]

            exchange_package = config_data['PROTOCOL_PACKAGE']
            envs.append(('PROTOCOL_PACKAGE', exchange_package))
            print('PROTOCOL_PACKAGE:', exchange_package)

            if 'EARNING_COMPONENT' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = create_proof_by_fungible_resource(builder, account, ret.Address(authority_resource), ret.Decimal("1"))
                builder = builder.call_function(
                    ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_package)),
                    'StakingEarning',
                    'instantiate',
                    [
                        manifest_owner_role
                    ]
                )
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['STAKING_POOL'] = addresses[1]
                config_data['EARNING_COMPONENT'] = addresses[0]
                config_data['DSE_RESOURCE'] = addresses[2]

            earning_component = config_data['EARNING_COMPONENT']
            staking_pool = config_data['STAKING_POOL']
            envs.append(('EARNING_COMPONENT', earning_component))
            envs.append(('STAKING_POOL', staking_pool))
            print('EARNING_COMPONENT:', earning_component)
            print('STAKING_POOL:', staking_pool)

            if 'CDP_COMPONENT' not in config_data:
                builder = ret.ManifestV1Builder()
                builder = lock_fee(builder, account, 100)
                builder = create_proof_by_fungible_resource(builder, account, ret.Address(authority_resource), ret.Decimal("1"))
                builder = builder.call_function(
                    ret.ManifestBuilderAddress.STATIC(ret.Address(exchange_package)),
                    'CollateralDebtManager',
                    'instantiate',
                    [
                        manifest_owner_role
                    ]
                )
                payload, intent = await gateway.build_transaction(builder, public_key, private_key)
                await gateway.submit_transaction(payload)
                addresses = await gateway.get_new_addresses(intent)
                config_data['CDP_COMPONENT'] = addresses[0]
            cdp_component = config_data['CDP_COMPONENT']
            envs.append(('CDP_COMPONENT', cdp_component))
            print('CDP_COMPONENT:', cdp_component)

            await create_usdx_pool(gateway, config_data, account, public_key, private_key, config_data['USDC_RESOURCE'], "USDC_POOL", "DX_USDC", envs)
            await create_usdx_pool(gateway, config_data, account, public_key, private_key, config_data['USDT_RESOURCE'], "USDT_POOL", "DX_USDT", envs)
            await create_xrd_pool(gateway, config_data, account, public_key, private_key, network_config['xrd'], envs)

            print(f'---------- DEPLOY {network_config["network_name"].upper()} COMPLETE ----------')

            print(f'STATE_VERSION={state_version}')

        #     print(f'OWNER_RESOURCE={owner_resource}')
        #     print(f'AUTHORITY_RESOURCE={authority_resource}')
        #     print(f'BASE_AUTHORITY_RESOURCE={base_authority_resource}')
        #     print(f'BASE_RESOURCE={base_resource}')
        #     print(f'LP_RESOURCE={lp_resource}')
        #     print(f'REFERRAL_RESOURCE={referral_resource}')
        #     print(f'RECOVERY_KEY_RESOURCE={recovery_key_resource}')
        #     print(f'PROTOCOL_RESOURCE={protocol_resource}')
        #     print(f'KEEPER_REWARD_RESOURCE={keeper_reward_resource}')
        #     print(f'FEE_OATH_RESOURCE={fee_oath_resource}')

        #     print(f'TOKEN_WRAPPER_PACKAGE={token_wrapper_package}')
        #     print(f'CONFIG_PACKAGE={config_package}')
        #     print(f'ACCOUNT_PACKAGE={account_package}')
        #     print(f'POOL_PACKAGE={pool_package}')
        #     print(f'REFERRAL_GENERATOR_PACKAGE={referral_generator_package}')
        #     print(f'PERMISSION_REGISTRY_PACKAGE={permission_registry_package}')
        #     print(f'ORACLE_PACKAGE={oracle_package}')
        #     print(f'FEE_DISTRIBUTOR_PACKAGE={fee_distributor_package}')
        #     print(f'FEE_DELEGATOR_PACKAGE={fee_delegator_package}')
        #     print(f'ENV_REGISTRY_PACKAGE={env_registry_package}')
        #     print(f'EXCHANGE_PACKAGE={exchange_package}')

        #     print(f'TOKEN_WRAPPER_COMPONENT={token_wrapper_component}')
        #     print(f'CONFIG_COMPONENT={config_component}')
        #     print(f'POOL_COMPONENT={pool_component}')
        #     print(f'REFERRAL_GENERATOR_COMPONENT={referral_generator_component}')
        #     print(f'PERMISSION_REGISTRY_COMPONENT={permission_registry_component}')
        #     print(f'ORACLE_COMPONENT={oracle_component}')
        #     print(f'FEE_DISTRIBUTOR_COMPONENT={fee_distributor_component}')
        #     print(f'FEE_DELEGATOR_COMPONENT={fee_delegator_component}')
        #     print(f'ENV_REGISTRY_COMPONENT={env_registry_component}')
        #     print(f'EXCHANGE_COMPONENT={exchange_component}')

            print('-------------------------------------')

        except Exception as e:
            import traceback
            print('TRACEBACK:', traceback.format_exc())
        finally:
            release_path = join(dirname(path), 'releases')
            makedirs(release_path, exist_ok=True)
            release_path = join(release_path, timestamp + '_' + network_config['network_name'])
            makedirs(release_path, exist_ok=True)
        
            with open(join(release_path, network_config['network_name'] + '.config.json'), 'w') as config_file:
                json.dump(config_data, config_file, indent=4)
            with open(config_path, 'w') as config_file:
                json.dump(config_data, config_file, indent=4)
            print(f'Config saved')

        # withdraw_account = input("Please enter your address to withdraw: ")
        # balance = await gateway.get_xrd_balance(account)
        # builder = ManifestV1Builder()
        # builder = lock_fee(builder, account, 100)
        # builder = builder.account_withdraw(
        #     account,
        #     Address(owner_resource),
        #     Decimal('9')
        # )
        # builder = builder.account_withdraw(
        #     account,
        #     Address(network_config['xrd']),
        #     Decimal(str(balance - 1))
        # )
        # builder = deposit_all(builder, Address(withdraw_account))

        # payload, intent = await gateway.build_transaction(builder, public_key, private_key)
        # await gateway.submit_transaction(payload)

        # print('WITHDRAW SUBMITTED:', intent)

async def create_usdx_pool(
        gateway: Gateway, config_data: Dict[str, str], 
        account:ret.Address, public_key: ret.PublicKey, private_key: ret.PrivateKey, 
        token:str, env_pool_name:str,  env_dx_name: str, envs: List[Tuple[str, str]]):
    if env_pool_name not in config_data or env_dx_name not in config_data:
        owner = config_data['OWNER_RESOURCE']
        owner_amount = 4
        auth = config_data['AUTHORITY_RESOURCE']
        cdp_mgr = config_data['CDP_COMPONENT']
        manifest = f'''
                CALL_METHOD
                    Address("{account.as_str()}")
                    "lock_fee"
                    Decimal("10")
                ;
                CALL_METHOD
                    Address("{account.as_str()}")
                    "create_proof_of_amount"
                    Address("{auth}")
                    Decimal("1")
                ;
                CALL_METHOD
                    Address("{cdp_mgr}")
                    "new_pool"
                    Enum<2u8>(
                        Enum<2u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Decimal("{owner_amount}"),
                                    Address("{owner}")
                                )
                            )
                        )
                    )
                    18u8
                    Address("{token}")
                    Enum<1u8>()
                    Decimal("0.85")
                    Decimal("0.87")
                    Decimal("0.02")
                    Decimal("0.10")
                    Decimal("0.001")
                    None
                ;
            '''
        #print(manifest)
        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        config_data[env_pool_name] = addresses[0]
        config_data[env_dx_name] = addresses[1]
    pool = config_data[env_pool_name]
    dx = config_data[env_dx_name]
    envs.append((env_dx_name, pool))
    print(f"{env_pool_name}:{pool}")
    print(f"{env_dx_name}:{dx}")

async def create_xrd_pool(
        gateway: Gateway, config_data: Dict[str, str], 
        account:ret.Address, public_key: ret.PublicKey, private_key: ret.PrivateKey,
        xrd: str, envs: List[Tuple[str, str]]
        ):
    if "XRD_POOL" not in config_data or "DX_XRD" not in config_data:
        owner = config_data['OWNER_RESOURCE']
        owner_amount = 4
        auth = config_data['AUTHORITY_RESOURCE']
        cdp_mgr = config_data['CDP_COMPONENT']
        earning = config_data['EARNING_COMPONENT']
        token = xrd
        manifest = f'''
                CALL_METHOD
                    Address("{account.as_str()}")
                    "lock_fee"
                    Decimal("10")
                ;
                CALL_METHOD
                    Address("{account.as_str()}")
                    "create_proof_of_amount"
                    Address("{auth}")
                    Decimal("1")
                ;
                CALL_METHOD
                    Address("{cdp_mgr}")
                    "new_pool"
                    Enum<2u8>(
                        Enum<2u8>(
                            Enum<0u8>(
                                Enum<1u8>(
                                    Decimal("{owner_amount}"),
                                    Address("{owner}")
                                )
                            )
                        )
                    )                    
                    18u8
                    Address("{token}")
                    Enum<0u8>()
                    Decimal("0.85")
                    Decimal("0.87")
                    Decimal("0.02")
                    Decimal("0.10")
                    Decimal("0.001")
                    Some(Address("{earning}"))
                ;
            '''
        #print(manifest)
        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        addresses = await gateway.get_new_addresses(intent)
        config_data["XRD_POOL"] = addresses[0]
        config_data["DX_XRD"] = addresses[1]
    dx_xrd = config_data["DX_XRD"]
    xrd_pool = config_data['XRD_POOL']
    envs.append(('DX_XRD', dx_xrd))
    print('DX_XRD:', dx_xrd)
    print("XRD_POOL:", xrd_pool)

if __name__ == '__main__':
    asyncio.run(main())
