# Basic example (Rust)

This example contains a [backend canister](./src/basic_backend/) written in Rust that sends HTTP requests via the [Proxy canister](../../../src/proxy_canister/) to external APIs.

## Usage

You can interact with the canister from [its Candid UI interface](https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=ittvb-6qaaa-aaaao-a3agq-cai).

The main method is the [`http_request_via_proxy`](./src/basic_backend/src/lib.rs#L29-L52), which can be invoked to make the HTTP request. The parameters are:

- `req`: the HTTP request parameters (url, method, headers, body).
- `timeout_ms`: the request timeout in milliseconds (optional).

  You can try setting this parameter to a value between _5000_ and _60000_ milliseconds and make a GET request to the [httpbin.org](https://httpbin.org/)'s [delay endpoint](https://httpbin.org/#/Dynamic_data/get_delay__delay_) with a bigger delay value to see the request expiring.

- `with_callback`: set it to `true` to receive the response back from the Proxy canister, `false` otherwise.

  When set to `true`, the Proxy canister will call the [`http_request_callback`](./src/basic_backend/src/lib.rs#L55-L64) canister method, which will register the response result in the local state.

The `http_request_via_proxy` method will return the response id, which can be used to read the response result from the canister state. In order to read the request result, you can pass the response id as an argument to the [`get_http_result_by_id`](./src/basic_backend/src/lib.rs#L72-L74) method.

Use the [`get_http_results`](./src/basic_backend/src/lib.rs#L67-L69) method to retrieve all the responses' results.

## Deploy the canister on MAINNET

Make sure you have [jq](https://jqlang.github.io/jq) installed.

**Remove** the [canister_ids.json](./canister_ids.json) file from this folder and then run the following command:

```bash
./deploy.sh
```

Note: the script will attempt to get the ID of the HTTP Proxy Canister from the [canister_ids.json](../../../canister_ids.json) file, and pass it as the init argument to the `basic_backend` canister.
