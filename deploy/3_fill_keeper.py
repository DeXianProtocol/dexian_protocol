
import asyncio
import datetime
import json
import sys
import radix_engine_toolkit as ret
from os.path import dirname, join, realpath
from os import makedirs, chdir, getenv
from aiohttp import ClientSession, TCPConnector
from subprocess import run
from dotenv import load_dotenv
load_dotenv()

from tools.gateway import Gateway
from tools.accounts import new_account, load_account
from tools.manifests import lock_fee, deposit_all

async def main():
    path = dirname(realpath(__file__))
    chdir(path)

    async with ClientSession(connector=TCPConnector(ssl=False)) as session:
        gateway = Gateway(session)
        network_config = await gateway.network_configuration()
        account_details = load_account(network_config['network_id'])
        if account_details is None:
            account_details = new_account(network_config['network_id'])
        private_key, public_key, account = account_details

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
        
        owner_resource = config_data['OWNER_RESOURCE']
        auth = config_data['AUTHORITY_RESOURCE']
        base_auth = config_data['BASE_AUTHORITY_RESOURCE']
        keeper = config_data['KEEPER_COMPONENT']
        validator = getenv("validator")
        usdc = config_data['USDC_RESOURCE']
        usdt = config_data['USDT_RESOURCE']
        if not validator:
            print("validator=", validator)
            return
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
                Address("{keeper}")
                "fill_validator_staking"
                Address("{validator}")
                Array<Tuple>(
                    Tuple(Decimal("215003609.309850772332662341"), Decimal("206584622.123424084603641842"),92736u64),
                    Tuple(Decimal("214731500.428212108670025763"), Decimal("206543343.377075727242622976"),90720u64),
                    Tuple(Decimal("214467159.512668941770741925"), Decimal("206509526.448205163408901648"),88704u64),
                    Tuple(Decimal("214200724.815748109876580907"), Decimal("206473434.091545913451617567"),86688u64),
                    Tuple(Decimal("213936073.791968149144107703"), Decimal("206439239.798544778167791825"),84672u64)
                )
            ;
        '''
        print(manifest)

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print(intent, status)

        oracle = config_data['ORACLE_COMPONENT']
        pub_key_str = "6d187b0f2e66d74410e92e2dc92a5141a55c241646ce87acbcad4ab413170f9b"
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
                Address("{oracle}")
                "set_verify_public_key"
                "{pub_key_str}"
            ;
        '''
        manifest = f'''
            CALL_METHOD
                Address("{account.as_str()}")
                "lock_fee"
                Decimal("10")
            ;
            CALL_METHOD
                Address("{account.as_str()}")
                "create_proof_of_amount"
                Address("{base_auth}")
                Decimal("1")
            ;
            CALL_METHOD
                Address("{oracle}")
                "set_price_quote_in_xrd"
                Address("{usdt}")
                Decimal("145.25050961")
            ;
            CALL_METHOD
                Address("{oracle}")
                "set_price_quote_in_xrd"
                Address("{usdc}")
                Decimal("145.25050961")
            ;
        '''
        print(manifest)

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print(intent, status)

if __name__ == '__main__':
    asyncio.run(main())

