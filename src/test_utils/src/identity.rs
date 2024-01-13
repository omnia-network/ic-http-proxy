use candid::Principal;
use ic_agent::{identity::BasicIdentity, Identity};
use ring::signature::Ed25519KeyPair;

pub fn generate_random_principal() -> Principal {
    let rng = ring::rand::SystemRandom::new();
    let key_pair = Ed25519KeyPair::generate_pkcs8(&rng)
        .unwrap()
        .as_ref()
        .to_vec();
    let identity = BasicIdentity::from_key_pair(Ed25519KeyPair::from_pkcs8(&key_pair).unwrap());

    identity.sender().unwrap()
}
