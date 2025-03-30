resim reset
result=$(resim new-account)
export admin_account=$(echo $result|grep "Account component address: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')
export admin_account_priv=$(echo $result|grep "Private key:" |awk -F "Private key: " '{print $2}'|awk -F " " '{print $1}')
export admin_account_badge=$(echo $result|grep "Owner badge: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')
export account=$admin_account
result=$(resim new-account)
export p1=$(echo $result|grep "Account component address: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')
export p1_priv=$(echo $result|grep "Private key:" |awk -F "Private key: " '{print $2}'|awk -F " " '{print $1}')
export p1_badge=$(echo $result|grep "Owner badge: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')
result=$(resim new-account)
export p2=$(echo $result|grep "Account component address: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')
export p2_priv=$(echo $result|grep "Private key:" |awk -F "Private key: " '{print $2}'|awk -F " " '{print $1}')
export p2_badge=$(echo $result|grep "Owner badge: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')
result=$(resim new-account)
export p3=$(echo $result|grep "Account component address: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')
export p3_priv=$(echo $result|grep "Private key:" |awk -F "Private key: " '{print $2}'|awk -F " " '{print $1}')
export p3_badge=$(echo $result|grep "Owner badge: "|awk -F ": " '{print $2}'|awk -F " " '{print $1}')

result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/resources.rtm)
export owner=$(echo $result | grep "Resource: " | awk -F "Resource: " '{print $2}' | awk -F " " '{if (NR==1) print $1}')
export auth=$(echo $result | grep "Resource: " | awk -F "Resource: " '{print $2}' | awk -F " " '{if (NR==2) print $1}')
export base_auth=$(echo $result | grep "Resource: " | awk -F "Resource: " '{print $2}' | awk -F " " '{if (NR==3) print $1}')
export base=$(echo $result | grep "Resource: " | awk -F "Resource: " '{print $2}' | awk -F " " '{if (NR==4) print $1}')

export AUTHORITY_RESOURCE=$auth
export BASE_AUTHORITY_RESOURCE=$base_auth
export BASE_RESOURCE=$base

resim set-default-account $admin_account $admin_account_priv $admin_account_badge
cd ../faucet
cargo clean
scrypto build
result=$(resim publish ".")
export pkg=$(echo $result | awk -F ": " '{print $2}')
result=$(resim call-function $pkg "Faucet" "new")
export faucet=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export faucet_owner=$(echo $result | grep "Resource: " | awk -F "Resource: " '{print $2}' | awk -F " " '{if (NR==1) print $1}')
export usdc=$(echo $result | grep "Resource: " | awk -F "Resource: " '{print $2}' | awk -F " " '{if (NR==2) print $1}')
export usdt=$(echo $result | grep "Resource: " | awk -F "Resource: " '{print $2}' | awk -F " " '{if (NR==3) print $1}')


cd ../keeper
resim set-default-account $admin_account $admin_account_priv $admin_account_badge
cargo clean
scrypto build
result=$(resim publish ".")
export pkg=$(echo $result | awk -F ": " '{print $2}')
export KEEPER_PACKAGE=$pkg
export owner_amount=4
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_keeper.rtm)
export keeper=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export KEEPER_COMPONENT=$keeper

cd ../interest
cargo clean
scrypto build
result=$(resim publish ".")
export pkg=$(echo $result | awk -F ": " '{print $2}')
export INTEREST_PACKAGE=$pkg
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_interest.rtm)
export interest=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export INTEREST_COMPONENT=$interest


cd ../oracle
cargo clean
scrypto build
result=$(resim publish ".")
export pkg=$(echo $result | awk -F ": " '{print $2}')
export ORACLE_PACKAGE=$pkg
export pub_key_str="6d187b0f2e66d74410e92e2dc92a5141a55c241646ce87acbcad4ab413170f9b"
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_oracle.rtm)
export oracle=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export ORACLE_COMPONENT=$oracle

cd ../protocol
cargo clean
scrypto build
result=$(resim publish ".")
export pkg=$(echo $result | awk -F ": " '{print $2}')
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_earning.rtm)
export earning=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{if (NR==1) print $1}')
export staking_pool=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{if (NR==2) print $1}')
export dse=$(echo $result | grep "Resource: " | awk -F "Resource: " '{if (NR==1) print $2}')
export EARNING_COMPONENT=$earning
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_cdp_mgr.rtm)
export cdp_mgr=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export CDP_COMPONENT=$cdp_mgr

resim set-default-account $admin_account $admin_account_priv $admin_account_badge
export token=$usdc
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_usdX_pool.rtm)
export usdc_pool=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export dx_usdc=$(echo $result | grep "Resource: " | awk -F "Resource: " '{if (NR==1) print $2}')
export token=$usdt
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_usdX_pool.rtm)
export usdt_pool=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export dx_usdt=$(echo $result | grep "Resource: " | awk -F "Resource: " '{if (NR==1) print $2}')
export xrd="resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3"
result=$(resim run < ../notes/replace_holder.sh ../notes/manifests/new_xrd_pool.rtm)
export xrd_pool=$(echo $result | grep "Component: "| awk -F "Component: " '{print $2}' | awk -F " " '{print $1}')
export dx_xrd=$(echo $result | grep "Resource: " | awk -F "Resource: " '{if (NR==1) print $2}')
resim run < ../notes/replace_holder.sh ../notes/manifests/set_price.rtm

# supply p1
resim set-default-account $p1 $p1_priv $p1_badge
export supply_token=$xrd
export account=$p1
export amount=4000
resim run < ../notes/replace_holder.sh ../notes/manifests/supply.rtm
# supply p2
resim set-default-account $p2 $p2_priv $p2_badge
export supply_token=$xrd
export account=$p2
export amount=4000
resim run < ../notes/replace_holder.sh ../notes/manifests/supply.rtm
resim call-method $faucet "free_tokens"
export supply_token=$usdt
export amount=40
resim run < ../notes/replace_holder.sh ../notes/manifests/supply.rtm
# supply p3
resim set-default-account $p3 $p3_priv $p3_badge
resim call-method $faucet "free_tokens"
export account=$p3
export supply_token=$usdc
export amount=40
resim run < ../notes/replace_holder.sh ../notes/manifests/supply.rtm

resim set-current-epoch 2
resim set-default-account $p1 $p1_priv $p1_badge
export quote="usdt"
export epoch=2
export price1="0.056259787085"
export quote1=$usdt
export timestamp1=1700658816
export signature1=$(python ../deploy/sign-util.py $xrd $quote1 $price1 $epoch $timestamp1)
export price2=None
export quote2=None
export timestamp2=None
export signature2=None
export account=$p1
export dx_token=$dx_xrd
export amount=2000
export borrow_token=$usdt
export borrow_amount=5
resim run < ../notes/replace_holder.sh ../notes/manifests/borrow_variable.rtm


resim set-default-account $p3 $p3_priv $p3_badge
export quote="usdc"
export epoch=2
export price1="0.056259787085"
export quote1=$usdc
export timestamp1=1700658816
export signature1=$(python ../deploy/sign-util.py $xrd $quote1 $price1 $epoch $timestamp1)
export price2=None
export quote2=None
export timestamp2=None
export signature2=None
export account=$p3
export dx_token=$dx_usdc
export amount=20
export borrow_token=$xrd
export borrow_amount=100
resim run < ../notes/replace_holder.sh ../notes/manifests/borrow_variable.rtm


resim set-current-epoch 15122
resim set-default-account $p1 $p1_priv $p1_badge
resim call-method $faucet "free_tokens"
export repay_token=$usdt
export amount=10
export account=$p1
export cdp_id=1
resim run < ../notes/replace_holder.sh ../notes/manifests/repay.rtm


resim set-current-epoch 53051
resim set-default-account $admin_account $admin_account_priv $admin_account_badge
export validator="validator_sim1s0a9c9kwjr3dmw79djvhalyz32sywumacv873yr7ms02v725fydvm0"
resim run < ../notes/replace_holder.sh ../notes/manifests/fill_keeper.rtm

resim set-default-account $p3 $p3_priv $p3_badge
export account=$p3
export amount=5000
resim run < ../notes/replace_holder.sh ../notes/manifests/join.rtm

export amount=100
export faster=true
#export dse="resource_sim1t56vy6c0ulajlqmv6xfy6kjkg5v3j6daslcthuczanhyxqlqzktk8l"
resim run < ../notes/replace_holder.sh ../notes/manifests/redeem_dse.rtm
