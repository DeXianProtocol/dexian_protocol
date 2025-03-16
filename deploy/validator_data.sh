#!/bin/bash
start=$1
step=$2
end=$3

if [ -z "$start" ] || [ -z "$step" ] || [ -z "$end" ]; then
  echo "Usage: $0 <start> <step> <end>"
  exit 1
fi

# lsu="your_lsu_address"
# validator="your_validator_address"
for epoch in $(seq $start $step $end); do
  supply=$(curl "${GATEWAY_URL}/state/entity/details" \
    -H "authority: ${GATEWAY_URL}" \
    -H 'accept: */*' \
    -H 'accept-language: en-US,en;q=0.6' \
    -H 'content-type: application/json' \
    -H 'origin: https://lending.dexian.io' \
    -H 'rdx-app-dapp-definition: account_tdx_2_129th30gyg5w0fh06swecmtg6ddcqfl77qme7ffvqzrgwc7kyelr5tp' \
    -H 'rdx-app-name: DeXian Lending Protocol' \
    -H 'rdx-app-version: 1.0.0' \
    -H 'rdx-client-name: @radixdlt/babylon-gateway-api-sdk' \
    -H 'rdx-client-version: 1.0.1' \
    -H 'referer: https://lending.dexian.io/' \
    -H 'sec-ch-ua: "Chromium";v="118", "Brave";v="118", "Not=A?Brand";v="99"' \
    -H 'sec-ch-ua-mobile: ?0' \
    -H 'sec-ch-ua-platform: "macOS"' \
    -H 'sec-fetch-dest: empty' \
    -H 'sec-fetch-mode: cors' \
    -H 'sec-fetch-site: cross-site' \
    -H 'sec-gpc: 1' \
    -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36' \
    --data-raw "{\"at_ledger_state\":{\"epoch\":$epoch},\"opt_ins\":{\"ancestor_identities\":false,\"component_royalty_vault_balance\":false,\"package_royalty_vault_balance\":false,\"non_fungible_include_nfids\":true,\"explicit_metadata\":[]},\"addresses\":[\"$lsu\"],\"aggregation_level\":\"Vault\"}" \
    --compressed | jq -r '.items[0].details.total_supply')

  stake=$(curl "${GATEWAY_URL}/state/validators/list" \
    -H "authority: ${GATEWAY_URL}" \
    -H 'accept: */*' \
    -H 'accept-language: zh-CN,zh;q=0.8' \
    -H 'content-type: application/json' \
    -H 'origin: https://stokenet-dashboard.radixdlt.com' \
    -H 'rdx-app-dapp-definition: Unknown' \
    -H 'rdx-app-name: Radix Dashboard' \
    -H 'rdx-app-version: Unknown' \
    -H 'rdx-client-name: @radixdlt/babylon-gateway-api-sdk' \
    -H 'rdx-client-version: 1.2.7' \
    -H 'referer: https://stokenet-dashboard.radixdlt.com/' \
    -H 'sec-ch-ua: "Not A(Brand";v="99", "Brave";v="121", "Chromium";v="121"' \
    -H 'sec-ch-ua-mobile: ?0' \
    -H 'sec-ch-ua-platform: "macOS"' \
    -H 'sec-fetch-dest: empty' \
    -H 'sec-fetch-mode: cors' \
    -H 'sec-fetch-site: same-site' \
    -H 'sec-gpc: 1' \
    -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36' \
    --data-raw "{\"cursor\":null, \"at_ledger_state\":{\"epoch\":$epoch}}" \
    --compressed | jq -r ".validators.items[]|select(.address==\"$validator\")|.stake_vault.balance")

  echo "Tuple(Decimal(\"$stake\"), Decimal(\"$supply\"),${epoch}u64)"
done