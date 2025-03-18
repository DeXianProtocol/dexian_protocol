
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
                    Tuple(Decimal("112237180.296872849296766891"), Decimal("106945526.373340375151046984"),86688u64),
                    Tuple(Decimal("112098789.798285268715788744"), Decimal("106941612.989392488139496828"),84672u64),
                    Tuple(Decimal("111959502.729679940898983504"), Decimal("106937614.480396442958605832"),82656u64),
                    Tuple(Decimal("111812504.140792595817896675"), Decimal("106927142.751198134517340198"),80640u64),
                    Tuple(Decimal("111671430.762427438906729348"), Decimal("106922321.81834615851120144"),78624u64),
                    Tuple(Decimal("111531123.578410529527003172"), Decimal("106918421.298990062333924856"),76608u64),
                    Tuple(Decimal("111391249.978008627097998059"), Decimal("106913368.20277604878239515"),74592u64),
                    Tuple(Decimal("111217613.872691674663641253"), Decimal("106879479.885389093107931943"),72576u64),
                    Tuple(Decimal("111069369.324371512654697253"), Decimal("106869097.447947881260897466"),70560u64),
                    Tuple(Decimal("108415946.518229418502963481"), Decimal("104444698.30595915458410914"),68544u64),
                    Tuple(Decimal("108277396.093980704102555695"), Decimal("104440978.133015624626436059"),66528u64),
                )
            ;
        '''
        print(manifest)

        payload, intent = await gateway.build_transaction_str(manifest, public_key, private_key)
        await gateway.submit_transaction(payload)
        status = await gateway.get_transaction_status(intent)
        print(intent, status)

        oracle = config_data['ORACLE_COMPONENT']
        # pub_key_str = "d7feb0f5c5c1f587be6b651e3244da1b053e1aa3147c3219aa1aa1f6265e57a0"
        # manifest = f'''
        #     CALL_METHOD
        #         Address("{account.as_str()}")
        #         "lock_fee"
        #         Decimal("10")
        #     ;
        #     CALL_METHOD
        #         Address("{account.as_str()}")
        #         "create_proof_of_amount"
        #         Address("{auth}")
        #         Decimal("1")
        #     ;

        #     CALL_METHOD
        #         Address("{oracle}")
        #         "set_verify_public_key"
        #         "{pub_key_str}"
        #     ;
        # '''
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

