use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use candid::{decode_one, encode_args, utils::ArgumentEncoder, CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::provisional::CanisterSettings;
use lazy_static::lazy_static;
use pocket_ic::{PocketIc, PocketIcBuilder, UserError, WasmResult};
use std::fs::File;
use std::io::Read;

lazy_static! {
    pub static ref TEST_ENV: Mutex<TestEnv> = Mutex::new(TestEnv::new());
}

pub fn get_test_env<'a>() -> MutexGuard<'a, TestEnv> {
    TEST_ENV.lock().unwrap()
}

#[derive(Clone)]
pub struct CanisterData {
    pub wasm_module: Vec<u8>,
    pub args: Vec<u8>,
    pub controller: Option<Principal>,
}

pub struct TestEnv {
    pub pic: PocketIc,
    canisters: HashMap<Principal, CanisterData>,
}

impl TestEnv {
    pub fn new() -> Self {
        let pic = PocketIcBuilder::new()
            // NNS subnet needed to retrieve the root key
            .with_nns_subnet()
            .with_application_subnet()
            .build();

        Self {
            pic,
            canisters: HashMap::new(),
        }
    }

    pub fn add_canister(&mut self, data: CanisterData) -> Principal {
        let app_subnet = self.pic.topology().get_app_subnets()[0];
        let canister_id = self.pic.create_canister_on_subnet(
            data.controller,
            Some(CanisterSettings {
                controllers: data.controller.map(|c| vec![c]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
            }),
            app_subnet,
        );
        self.pic.add_cycles(canister_id, 1_000_000_000_000_000);

        self.pic.install_canister(
            canister_id,
            data.wasm_module.clone(),
            data.args.clone(),
            data.controller,
        );

        self.canisters.insert(canister_id, data);

        canister_id
    }

    pub fn get_canisters(&self) -> HashMap<Principal, CanisterData> {
        self.canisters.clone()
    }

    pub fn reset_canister(&self, canister_id: &Principal) {
        self.tick_n(10);

        let data = self.canisters.get(canister_id).unwrap();

        self.pic
            .reinstall_canister(
                *canister_id,
                data.wasm_module.clone(),
                data.args.clone(),
                data.controller,
            )
            .unwrap();
    }

    /// Produce and advance by some blocks to fire eventual timers.
    ///
    /// See https://forum.dfinity.org/t/pocketic-multi-subnet-canister-testing/24901/4.
    pub fn tick_n(&self, n: u8) {
        for _ in 0..n {
            self.pic.tick();
        }
    }

    pub fn advance_canister_time_ms(&self, ms: u64) {
        self.pic.advance_time(Duration::from_millis(ms));
        self.tick_n(100);
    }

    /// # Panics
    /// if the call returns a [WasmResult::Reject].
    pub fn call_canister_method<T, S>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: T,
    ) -> Result<S, UserError>
    where
        T: CandidType + ArgumentEncoder,
        S: CandidType + for<'a> Deserialize<'a>,
    {
        self.pic
            .update_call(
                canister_id,
                caller,
                &method.to_string(),
                encode_args(args).unwrap(),
            )
            .map(|res| match res {
                WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
                _ => panic!("Expected reply"),
            })
    }

    /// # Panics
    /// if [TestEnv::call_canister_method] panics
    /// or the canister returns a [UserError].
    pub fn call_canister_method_with_panic<T, S>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: T,
    ) -> S
    where
        T: CandidType + ArgumentEncoder,
        S: CandidType + for<'a> Deserialize<'a>,
    {
        self.call_canister_method(canister_id, caller, method, args)
            .expect("Failed to call canister")
    }

    /// # Panics
    /// if the call returns a [WasmResult::Reject].
    pub fn query_canister_method<T, S>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: T,
    ) -> Result<S, UserError>
    where
        T: CandidType + ArgumentEncoder,
        S: CandidType + for<'a> Deserialize<'a>,
    {
        self.pic
            .query_call(
                canister_id,
                caller,
                &method.to_string(),
                encode_args(args).unwrap(),
            )
            .map(|res| match res {
                WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
                _ => panic!("Expected reply"),
            })
    }

    /// # Panics
    /// if [TestEnv::query_canister_method] panics
    /// or the canister returns a [UserError].
    pub fn query_canister_method_with_panic<T, S>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: T,
    ) -> S
    where
        T: CandidType + ArgumentEncoder,
        S: CandidType + for<'a> Deserialize<'a>,
    {
        self.query_canister_method(canister_id, caller, method, args)
            .expect("Failed to query canister")
    }
}

pub fn load_canister_wasm_from_path(path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(&path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}
