#!/bin/bash

set -e

./scripts/download-pocket-ic.sh

./scripts/build-test-canisters.sh

POCKET_IC_MUTE_SERVER=1 \
  POCKET_IC_BIN="$(pwd)/bin/pocket-ic" \
  TEST_CANISTER_WASM_PATH="$(pwd)/bin/test_canister.wasm" \
  cargo test -p http_over_ws --test integration_tests

POCKET_IC_MUTE_SERVER=1 \
  POCKET_IC_BIN="$(pwd)/bin/pocket-ic" \
  TEST_USER_CANISTER_WASM_PATH="$(pwd)/bin/test_user_canister.wasm" \
  PROXY_CANISTER_WASM_PATH="$(pwd)/bin/proxy_canister.wasm" \
  cargo test -p proxy_canister --test integration_tests
