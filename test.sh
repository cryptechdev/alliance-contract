#!/bin/bash

# This test suite requires the following:
# - A running node with the wasm and alliance modules enabled
# - an alliance created
# - wasm-cli and jq installed


DAEMON_NAME=noriad
DENOM=ucrd

VAL=$($DAEMON_NAME q staking validators --output json | jq '.validators[0].operator_address' | sed 's/\"//g')

cargo run-script optimize
echo -e "\n### Storing code ###\n"
wasm store -n alliance -s ./artifacts/alliance.wasm
cargo schema

echo -e "\n### Importing schema ###\n"
wasm im schema -n alliance ./schema/ -s

echo -e "\n### Instantiating contract ###\n"
wasm in -n alliance -a noria1ad4w3ldcqwxg0uk3uzd6k2rnx8y84cmtf7qklg -p '{}' -s

echo -e "\n### Testing MsgDelegate ###\n"
wasm tx -a "&alliance" -p '{"delegate":{"validator_address":"'$VAL'","amount":{"denom":"'$DENOM'","amount":"10"}}}' --coins 10$DENOM -s

echo -e "\n### Testing MsgUndelegate ###\n"
wasm tx -a "&alliance" -p '{"undelegate":{"validator_address":"'$VAL'","amount":{"denom":"'$DENOM'","amount":"5"}}}' -s

echo -e "\n### Testing MsgClaimRewards ###\n"
wasm tx -a "&alliance" -p '{"claim_delegation_rewards":{"validator_address":"'$VAL'","denom":"'$DENOM'"}}' -s

ADDR=$(wasm c info -n alliance address)

echo -e "\n\n### QUERIES ###\n\n"
wasm q -a "&alliance" -p '{"params":{}}'
wasm q -a "&alliance" -p '{"alliance":{"denom":"'$DENOM'"}}'
wasm q -a "&alliance" -p '{"alliances":{}}'
wasm q -a "&alliance" -p '{"alliances_delegations":{}}'
wasm q -a "&alliance" -p '{"alliances_delegations":{"pagination":{"limit":1}}}'
wasm q -a "&alliance" -p '{"alliances_delegations":{"pagination":{"limit":1,"key":"IAsG7cuchMrrxrLxKKeg6Ob4Qpl/ihgR//JLQulyJvcxFKf5EaIO6VeswWBGkYYsGzcSBMpyBXVjcmQA"}}}'
wasm q -a "&alliance" -p '{"alliances_delegation_by_validator":{"delegator_addr":"'$ADDR'","validator_addr":"'$VAL'"}}'
wasm q -a "&alliance" -p '{"delegation":{"delegator_addr":"'$ADDR'","validator_addr":"'$VAL'","denom":"'$DENOM'"}}'
wasm q -a "&alliance" -p '{"delegation_rewards":{"delegator_addr":"'$ADDR'","validator_addr":"'$VAL'","denom":"'$DENOM'"}}'
wasm q -a "&alliance" -p '{"validator":{"validator_addr":"'$VAL'"}}'
wasm q -a "&alliance" -p '{"validators":{}}'
