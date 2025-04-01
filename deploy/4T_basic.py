import radix_engine_toolkit as ret
import asyncio
import datetime
import json
from typing import Tuple, Dict
from os.path import dirname, join, realpath
from os import makedirs, chdir, getenv
from aiohttp import ClientSession, TCPConnector
from subprocess import run
from dotenv import load_dotenv
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, deposit_all, withdraw_to_bucket

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        
        (private_key, public_key, account) = await makesure_account(network_config, 0)
        (priv1, pub1, account1) = await makesure_account(network_config, 1)
        (priv2, pub2, account2) = await makesure_account(network_config, 2)
        (priv3, pub3, account3) = await makesure_account(network_config, 3)
        if network_config['network_name'] == 'stokenet':
            config_path = join(path, 'stokenet.config.json')
        elif network_config['network_name'] == 'mainnet':
            config_path = join(path, 'mainnet.config.json')
        else:
            raise ValueError(f'Unsupported network: {network_config["network_name"]}')
        
        with open(config_path, 'r') as config_file:
            config_data = json.load(config_file)


        balance = await gateway.get_xrd_balance(account)
        if balance < 1000:
            print('FUND ACCOUNT:', account.as_str())
        while balance < 1000:
            await asyncio.sleep(5)
            balance = await gateway.get_xrd_balance(account)
        
        faucet = config_data['FAUCET_COMPONENT']
        usdc = config_data['USDC_RESOURCE']
        usdt = config_data['USDT_RESOURCE']
        dse = config_data['DSE_RESOURCE']
        xrd = network_config['xrd']
        # await faucet_tokens(gateway, network_config, faucet, priv1, pub1, account1)
        # await faucet_tokens(gateway, network_config, faucet, priv2, pub2, account2)
        # await faucet_tokens(gateway, network_config, faucet, priv3, pub3, account3)

        # await supply(gateway, config_data, account1, pub1, priv1, network_config['xrd'], "4000")
        # await supply(gateway, config_data, account2, pub2, priv2, network_config['xrd'], "4000")
        # await supply(gateway, config_data, account3, pub3, priv3, usdc, "100")
        # await supply(gateway, config_data, account3, pub3, priv3, usdt, "100")

        dx_xrd = config_data['DX_XRD']
        dx_usdc = config_data['DX_USDC']
        validator = getenv("validator")
        amount = "2000"
        # await borrow(gateway, session, network_config['network_name'], config_data, account1, pub1, priv1, dx_xrd, "3000", usdc, "10", "usdc", None)
        # await borrow(gateway, session, network_config['network_name'], config_data, account2, pub2, priv2, dx_xrd, "3000", usdt, "10", "usdt", None)
        # await borrow(gateway, session, network_config['network_name'], config_data, account3, pub3, priv3, dx_usdc, "50", xrd, "3000", "usdc", None)
        await dse_join(gateway, account3, pub3, priv3, config_data, validator, xrd, amount)
        # await dse_redeem(gateway, account3, pub3, priv3, config_data, validator, dse, "100", False)
        # await dse_redeem(gateway, account3, pub3, priv3, config_data, validator, dse, "200", True)
    

async def makesure_account(network_config: Dict[str, str], index=0) -> Tuple[ret.PrivateKey, ret.PrivateKey, ret.Address]:
    account_details = load_account(network_config['network_id'], index)
    if account_details is None:
        return new_account(network_config['network_id'])
    return account_details

async def faucet_tokens(gateway:Gateway, network_config: Dict[str, str], faucet: str, private_key: ret.PrivateKey, public_key: ret.PublicKey, account: ret.Address):
    builder = ret.ManifestV1Builder()
    builder = builder.faucet_lock_fee()
    builder = builder.call_method(
        ret.ManifestBuilderAddress.STATIC(ret.Address(network_config['faucet'])),
        'free',
        []
    )
    builder = builder.call_method(
        ret.ManifestBuilderAddress.STATIC(ret.Address(faucet)),
        "free_tokens",
        []
    )
    builder = deposit_all(builder, account)
    payload, intent = await gateway.build_transaction(builder, public_key, private_key)
    print('Transaction id:', intent)
    await gateway.submit_transaction(payload)
    status = await gateway.get_transaction_status(intent)
    print('Transaction status:', status)

async def supply(gateway: Gateway, config_data: Dict[str, str], account: ret.Address, public_key: ret.PublicKey, private_key: ret.PrivateKey, token: str, amount: str):
    cdp_mgr = config_data['CDP_COMPONENT']
    builder = ret.ManifestV1Builder()
    builder = lock_fee(builder, account, 10)
    builder = withdraw_to_bucket(builder, account, ret.Address(token), ret.Decimal(amount), "bucket1")
    builder = builder.call_method(
        ret.ManifestBuilderAddress.STATIC(ret.Address(cdp_mgr)),
        "supply",
        [ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket("bucket1"))]
    )
    builder = builder.account_deposit_entire_worktop(account)
    payload, intent = await gateway.build_transaction(builder, public_key, private_key)
    print('supply Transaction id:', intent)
    await gateway.submit_transaction(payload)
    status = await gateway.get_transaction_status(intent)
    print('Transaction status:', status)

async def get_price_signature(session: ClientSession, network: str, base: str, quote: str) -> Tuple[str, str, int, str]:
    headers = {
            'Content-Type': 'application/json',
        }
    print(f"https://price.dexian.io/{network}/{base}-{quote}")
    async with session.get(f"https://price.dexian.io/{network}/{base}-{quote}", headers=headers) as response:
        data = await response.json()
        return (
            data['data']['price'],
            data['data']['symbol'].split("/")[1],
            data['data']['timestamp'],
            data['data']['signature'],
            data['data']['epoch_at']
        )

async def dse_join(gateway: Gateway, account: ret.Address, public_key: ret.PublicKey, private_key: ret.PrivateKey, config_data: Tuple[str, str], validator: str, xrd_addr_str: str,  amount: str):
    earning = config_data['EARNING_COMPONENT']
    builder = ret.ManifestV1Builder()
    builder = lock_fee(builder, account, 10)
    builder = withdraw_to_bucket(builder, account, ret.Address(xrd_addr_str), ret.Decimal(amount), "bucket1")
    builder = builder.call_method(
        ret.ManifestBuilderAddress.STATIC(ret.Address(earning)),
        "join",
        [
            ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(validator))),
            ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket("bucket1"))
        ]
    )
    builder = builder.account_deposit_entire_worktop(account)
    payload, intent = await gateway.build_transaction(builder, public_key, private_key)
    print('join Transaction id:', intent)
    await gateway.submit_transaction(payload)
    status = await gateway.get_transaction_status(intent)
    print('Transaction status:', status)


async def dse_redeem(gateway: Gateway, account: ret.Address, public_key: ret.PublicKey, private_key: ret.PrivateKey, config_data: Tuple[str, str], validator: str, dse: str,  amount: str, faster: bool):
    earning = config_data['EARNING_COMPONENT']
    cdp_mgr = config_data['CDP_COMPONENT']
    builder = ret.ManifestV1Builder()
    builder = lock_fee(builder, account, 10)
    builder = withdraw_to_bucket(builder, account, ret.Address(dse), ret.Decimal(amount), "bucket1")
    builder = builder.call_method(
        ret.ManifestBuilderAddress.STATIC(ret.Address(earning)),
        "redeem",
        [
            ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(cdp_mgr))),
            ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestBuilderAddress.STATIC(ret.Address(validator))),
            ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket("bucket1")),
            ret.ManifestBuilderValue.BOOL_VALUE(faster)
        ]
    )
    builder = builder.account_deposit_entire_worktop(account)
    payload, intent = await gateway.build_transaction(builder, public_key, private_key)
    print('dse redeem Transaction id:', intent)
    await gateway.submit_transaction(payload)
    status = await gateway.get_transaction_status(intent)
    print('Transaction status:', status)

async def borrow(gateway: Gateway, session: ClientSession, network_name: str, config_data: Dict[str, str], account: ret.Address, public_key: ret.PublicKey, private_key: ret.PrivateKey,
                dx_token: str, dx_amount: str, borrow_token:str, borrow_amount:str,
                quote:str, _quote: str):
    cdp_mgr = config_data['CDP_COMPONENT']
    (price1, quote1, timestamp1, signature1, epoch1) = await get_price_signature(session, network_name, "xrd", quote)
    if not _quote:
        (price2, quote2, timestamp2, signature2, epoch2) = (None, None, None, None, '')
    else:
        (price2, quote2, timestamp2, signature2, epoch2) = await get_price_signature(session, network_name, "xrd", _quote)
    print("get_price_signature", price1, quote1, timestamp1, signature1, epoch1)
    print("get_price_signature2", price2, quote2, timestamp2, signature2, epoch2)
    manifest = f'''
        CALL_METHOD
            Address("{account.as_str()}")
            "lock_fee"
            Decimal("10")
        ;
        CALL_METHOD
        Address("{account.as_str()}")
        "withdraw"
        Address("{dx_token}")
        Decimal("{dx_amount}")
        ;
        TAKE_FROM_WORKTOP
        Address("{dx_token}")
        Decimal("{dx_amount}")
        Bucket("bucket1")
        ;
        CALL_METHOD
            Address("{cdp_mgr}")
            "borrow_variable"
            Bucket("bucket1")
            Address("{borrow_token}")
            Decimal("{borrow_amount}")
            "{price1}"
            Address("{quote1}")
            {timestamp1}u64
            "{signature1}"
            {price2}
            {quote2}
            {timestamp2}
            {signature2}
        ;
        CALL_METHOD
            Address("{account.as_str()}")
            "deposit_batch"
            Expression("ENTIRE_WORKTOP")
        ;
    '''
    print(manifest)
    payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
    # if _quote:
    #     (price2, quote2, timestamp2, signature2) = await get_price_signature(session, network_name, "xrd", quote)
    #     mv_price2 = ret.ManifestBuilderValue.ENUM_VALUE(1, ret.ManifestValue.STRING_VALUE(price2))
    #     mv_quote2 = ret.ManifestBuilderValue.ENUM_VALUE(1, ret.ManifestValue.ADDRESS_VALUE(quote2))
    #     mv_price2 = ret.ManifestBuilderValue.ENUM_VALUE(1, ret.ManifestValue.U64_VALUE(timestamp2))
    #     mv_price2 = ret.ManifestBuilderValue.ENUM_VALUE(1, ret.ManifestValue.STRING_VALUE(signature2))
    # else:
    #     mv_price2 = ret.ManifestBuilderValue.ENUM_VALUE(0, None)
    #     mv_quote2 = ret.ManifestBuilderValue.ENUM_VALUE(0, None)
    #     mv_timestamp2 = ret.ManifestBuilderValue.ENUM_VALUE(0, None)
    #     mv_signature2 = ret.ManifestBuilderValue.ENUM_VALUE(0, None)

    # print(f"{borrow_token}")
    # builder = ret.ManifestV1Builder()
    # builder = lock_fee(builder, account, 10)
    # builder = withdraw_to_bucket(builder, account, ret.Address(dx_token), ret.Decimal(dx_amount), "bucket1")
    # builder = builder.call_method(
    #     ret.ManifestBuilderAddress.STATIC(ret.Address(cdp_mgr)),
    #     "borrow_variable",
    #     [
    #         ret.ManifestBuilderValue.BUCKET_VALUE(ret.ManifestBuilderBucket("bucket1")),
    #         ret.ManifestBuilderValue.ADDRESS_VALUE(ret.ManifestValue.ADDRESS_VALUE(borrow_token)),
    #         ret.ManifestBuilderValue.DECIMAL_VALUE(ret.Decimal(borrow_amount)),
    #         ret.ManifestBuilderValue.STRING_VALUE(price1),
    #         # ret.ManifestBuilderValue.ADDRESS_VALUE(ret.Address(quote1)),
    #         ret.ManifestBuilderValue.U64_VALUE(timestamp1),
    #         ret.ManifestBuilderValue.STRING_VALUE(signature1),
    #         mv_price2,
    #         mv_quote2,
    #         mv_timestamp2,
    #         mv_signature2
    #     ]
    # )
    # builder = builder.account_deposit_entire_worktop(account)
    # payload, intent = await gateway.build_transaction(builder, public_key, private_key)
    print('borrow variable Transaction id:', intent)
    await gateway.submit_transaction(payload)
    status = await gateway.get_transaction_status(intent)
    print('Transaction status:', status)


if __name__ == '__main__':
    asyncio.run(main())

