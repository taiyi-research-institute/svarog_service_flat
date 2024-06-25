use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Mnemonics {
    pub phrases: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SignTask {
    pub message: Vec<u8>,
    pub bip32_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8,
}

pub use svarog_algo::elgamal_secp256k1::KeystoreElgamal;
pub use svarog_algo::schnorr_ed25519::KeystoreSchnorr;
pub use svarog_grpc::SessionConfig;
