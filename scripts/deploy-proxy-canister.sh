#!/bin/bash

set -e

# build the canister before disconnecting clients, so that the deployment is faster
echo -e "\nBuilding canister..."
dfx build proxy_canister --check

echo -e "\nDisconnecting all proxy clients..."
dfx canister call --ic proxy_canister disconnect_all_proxies

echo -e "\nDeploying canister..."
dfx deploy --ic proxy_canister

# just log the status (controllers, balance, etc.)
echo -e "\nFetching canister status..."
dfx canister status proxy_canister --ic

echo -e "\nDone!"
