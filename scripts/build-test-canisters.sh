#!/bin/bash

set -e

cargo build --locked --target wasm32-unknown-unknown --release --package test_canister

cargo build --locked --target wasm32-unknown-unknown --release --package test_user_canister
cargo build --locked --target wasm32-unknown-unknown --release --package proxy_canister
