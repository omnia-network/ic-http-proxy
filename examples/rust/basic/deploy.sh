#!/bin/bash

set -e

PROXY_CANISTER_ID=$(jq .proxy_canister.ic ../../../canister_ids.json)

echo "PROXY_CANISTER_ID: $PROXY_CANISTER_ID"

dfx deploy --ic basic_backend --argument "(principal $PROXY_CANISTER_ID)"

# just to get info about the canister, not needed for deployment
dfx canister --ic status basic_backend
