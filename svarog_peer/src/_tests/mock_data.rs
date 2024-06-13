#![allow(nonstandard_style)]
#![allow(dead_code)]
use std::collections::{BTreeSet, HashMap};

use rand::Rng;
use sha2::digest::crypto_common::rand_core::OsRng;
use svarog_grpc::{Mnemonic, SessionConfig, SignTask};

pub const th1: usize = 3;
pub const th2: usize = 4;
pub const players1: [&str; 5] = ["Alice", "Bob", "Charlie", "David", "Eve"];
pub const players2: [&str; 7] = [
    "Charlie", "David", "Eve", "Frank", "Gabriel", "Henry", "Ivan",
];

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
            tx_data: hmsg,
        });
    }

    tasks
}

pub fn mock_one_sign_task() -> Vec<SignTask> {
    vec![mock_sign_tasks().pop().unwrap()]
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
) -> (SessionConfig, BTreeSet<String>) {
    let mut config = SessionConfig::default();

    let _config = mock_sign_config(provider_th, providers);
    config.players = _config.players;

    let _config = mock_keygen_config(consumer_th, consumers);
    config.threshold = consumer_th as u64;
    config.players_reshared = _config.players;

    let provider_set: BTreeSet<String> = {
        let mut res = BTreeSet::new();
        for (player, &att) in config.players.iter() {
            if att {
                res.insert(player.clone());
            }
        }
        res
    };
    let mut remain_set: BTreeSet<String> =
        config.players_reshared.keys().map(|s| s.clone()).collect();
    remain_set = remain_set.difference(&provider_set).cloned().collect();
    (config, remain_set)
}

pub fn mock_mnem() -> Mnemonic {
    Mnemonic {
        words: "park remain person kitchen mule spell knee armed position rail grid ankle"
            .to_owned(),
        password: "".to_owned(),
    }
}
