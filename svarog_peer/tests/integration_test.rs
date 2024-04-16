#![allow(nonstandard_style)]

use std::collections::HashMap;

/// 模块功能: 集成测试keygen, sign, keygen_mnem, reshare
use erreur::*;

use clap::{Arg, ArgAction, Command};
use rand::Rng;
use sha2::digest::crypto_common::rand_core::OsRng;
use svarog_grpc::{
    mpc_peer_client::MpcPeerClient, Algorithm, ParamsKeygen, ParamsSign, SessionConfig, SignTask,
};

const peer_url: &str = "http://127.0.0.1:9001";
const sesman_url: &str = "http://127.0.0.1:9000";
const algorithms: [Algorithm; 2] = [Algorithm::Gg18Secp256k1, Algorithm::FrostEd25519];
const th1: usize = 3;
const th2: usize = 4;
const players1: [&str; 5] = ["Alice", "Bob", "Charlie", "David", "Eve"];
const players2: [&str; 7] = [
    "Frank", "Gabriel", "Henry", "Ivan", "Jack", "Kevin", "Lucas",
];

/// 测试普通的keygen, sign
#[tokio::test]
async fn test_keygen_sign() -> Resultat<()> {
    let peer = MpcPeerClient::connect(peer_url).await.catch_()?;

    for algo in algorithms.iter() {}

    Ok(())
}

/// 测试助记词keygen, 批量sign
#[tokio::test]
async fn test_mkeygen_bsign() -> Resultat<()> {
    let peer = MpcPeerClient::connect(peer_url).await.catch_()?;
    Ok(())
}

/// 测试reshare
#[tokio::test]
async fn test_reshare() -> Resultat<()> {
    let peer = MpcPeerClient::connect(peer_url).await.catch_()?;
    Ok(())
}

pub fn mock_sign_tasks() -> Vec<SignTask> {
    use sha2::{Digest, Sha256};

    let mut tasks = Vec::new();
    let msgs = vec![
        "Je ne veux pas travailler. Je ne veux pas déjeuner. Je veux seulement l'oublier. Et puis je fume.",
        "Mon nom ne vous dit rien. Vous devez ignorer. Que nous sommes voisins. Depuis le mois de mai.",
        "Ma flamme et mon chagrin, mais aussi mes regrets. De ne vous avoir pas, suivi sur le quai.",
    ];
    let dpaths = vec!["m/1/2/3/4", "m/5/6/7", "m/8/9"];
    for (msg, dpath) in msgs.iter().zip(dpaths.iter()) {
        let mut hasher = Sha256::new();
        hasher.update(msg.as_bytes());
        let hmsg = hasher.finalize().to_vec();
        tasks.push(SignTask {
            derivation_path: dpath.to_string(),
            tx_hash: hmsg,
        });
    }

    tasks
}

pub fn mock_keygen_config(th: usize, players: &[&str]) -> SessionConfig {
    let mut config = SessionConfig::default();
    config.threshold = th as u64;
    config.players = players.iter().map(|s| (s.to_string(), true)).collect();
    config
}

pub fn mock_sign_config(th: usize, players: &[&str]) -> SessionConfig {
    let mut rng = OsRng;

    // shuffle players
    let mut config = SessionConfig::default();
    let mut players: Vec<String> = players.iter().map(|s| s.to_string()).collect();
    use rand::seq::SliceRandom;
    players.shuffle(&mut rng);

    // choose players for signing
    let n = players.len();
    let n_att = rng.gen_range(th..=n);

    // fill config.players
    let mut signers = HashMap::new();
    for i in 0..n {
        let name = players[i].clone();
        if i < n_att {
            signers.insert(name, true);
        } else {
            signers.insert(name, false);
        }
    }
    config.players = signers;

    config
}

pub fn mock_reshare_config(
    provider_th: usize,
    providers: &[&str],
    consumer_th: usize,
    consumers: &[&str],
) -> SessionConfig {
    let mut rng = OsRng;
    let mut config = SessionConfig::default();

    use rand::seq::SliceRandom; // provides Vec::shuffle()
    config.players = {
        let mut providers: Vec<String> = providers.iter().map(|s| s.to_string()).collect();
        providers.shuffle(&mut rng);

        let n = providers.len();
        let n_att = rng.gen_range(provider_th..=n);

        let mut signers = HashMap::new();
        for i in 0..n {
            let name = providers[i].clone();
            if i < n_att {
                signers.insert(name, true);
            } else {
                signers.insert(name, false);
            }
        }
        signers
    };

    config.threshold = consumer_th as u64;
    config.players_reshared = consumers.iter().map(|s| (s.to_string(), true)).collect();
    config
}
