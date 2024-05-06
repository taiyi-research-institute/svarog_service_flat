#![allow(nonstandard_style)]
#![allow(dead_code)]
use std::collections::{HashMap, HashSet};

use rand::{seq::SliceRandom, Rng};
use sha2::digest::crypto_common::rand_core::OsRng;
use svarog_grpc::{Algorithm, Department, Mnemonic, SessionConfig, SignTask};
pub const SESMAN_URL: &str = "http://127.0.0.1:2000";
pub const PEER_URL: &str = "http://127.0.0.1:2001";

pub const players1: [&str; 8] = [
    "Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Gabriel", "Henry",
];
pub const players2: [&str; 13] = [
    "Frank", "Gabriel", "Henry", "Ivan", "Jack", "Karl", "Lucy", "Mike", "Nancy", "Oliver",
    "Peter", "Quincy", "Roger",
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
            tx_hash: hmsg,
        });
    }

    tasks
}

#[derive(Debug, Clone)]
pub struct MockConfigs {
    pub keygen: SessionConfig,
    pub sign: SessionConfig,
    pub reshare: SessionConfig,
    pub sign_after_reshare: SessionConfig,
}

pub fn mockcfg(algo: &Algorithm) -> MockConfigs {
    let mut rng = OsRng;

    let mut players: HashMap<String, HashMap<String, bool>> = HashMap::new();
    for &player in players1.iter() {
        let dept_name = format!("dept-{}", rng.gen::<usize>() % 2 + 1);
        players
            .entry(dept_name)
            .or_insert(HashMap::new())
            .insert(player.to_owned(), true);
    }
    let th = rng.gen_range(1..=players1.len());
    let mut th_dict: HashMap<String, usize> = HashMap::new();
    for (dept_name, dept) in players.iter() {
        let dept_th = rng.gen_range(1..=dept.len());
        th_dict.insert(dept_name.clone(), dept_th);
    }
    let players: HashMap<String, Department> = {
        let mut val = HashMap::new();
        for (dept_name, dept) in players.iter() {
            let dept_obj = Department {
                name: dept_name.clone(),
                players: dept.clone(),
                threshold: th_dict[dept_name] as u64,
            };
            val.insert(dept_name.clone(), dept_obj);
        }
        val
    };

    let keygen = SessionConfig {
        algorithm: Some(algo.clone()),
        sesman_url: SESMAN_URL.to_string(),
        players: players.clone(),
        threshold: th as u64,
        ..Default::default()
    };

    let mut signers: HashMap<String, Department> = HashMap::new();
    let mut signers_flat: HashSet<String> = HashSet::new();
    for (dept_name, dept_obj) in players.iter() {
        let dept_th = dept_obj.threshold as usize;
        let mut dept_signers: HashMap<String, bool> = HashMap::new();
        let mut dept_members: Vec<String> = dept_obj.players.keys().cloned().collect();

        dept_members.shuffle(&mut rng);
        let n = rng.gen_range(dept_th..=dept_obj.players.len());
        for player in dept_members[..n].iter() {
            signers_flat.insert(player.clone());
            dept_signers.insert(player.clone(), true);
        }
        for player in dept_members[n..].iter() {
            dept_signers.insert(player.clone(), false);
        }

        let dept_obj = Department {
            name: dept_name.clone(),
            players: dept_signers,
            threshold: 0,
        };
        signers.insert(dept_name.clone(), dept_obj);
    }
    if signers_flat.len() < th {
        let all_signers: HashSet<String> = players1.iter().map(|x| x.to_string()).collect();
        let mut rest_signers: Vec<String> =
            all_signers.difference(&signers_flat).cloned().collect();
        rest_signers.shuffle(&mut rng);
        let n = th - signers_flat.len();
        let rest_signers: HashSet<String> = rest_signers.into_iter().take(n).collect();

        for (_, dept_obj) in signers.iter_mut() {
            let players: Vec<String> = dept_obj.players.keys().cloned().collect();
            for player in players.iter() {
                if rest_signers.contains(player) {
                    dept_obj.players.insert(player.clone(), true);
                }
            }
        }
    }

    let sign = SessionConfig {
        algorithm: Some(algo.clone()),
        sesman_url: SESMAN_URL.to_string(),
        players: signers.clone(),
        ..Default::default()
    };

    let mut players: HashMap<String, HashMap<String, bool>> = HashMap::new();
    for &player in players2.iter() {
        let dept_name = format!("dept-{}", rng.gen_range(1..=3));
        players
            .entry(dept_name)
            .or_insert(HashMap::new())
            .insert(player.to_owned(), true);
    }
    let th = rng.gen_range(1..=players2.len());
    let mut th_dict: HashMap<String, usize> = HashMap::new();
    for (dept_name, dept) in players.iter() {
        let dept_th = rng.gen_range(1..=dept.len());
        th_dict.insert(dept_name.clone(), dept_th);
    }
    let players: HashMap<String, Department> = {
        let mut val = HashMap::new();
        for (dept_name, dept) in players.iter() {
            let dept_obj = Department {
                name: dept_name.clone(),
                players: dept.clone(),
                threshold: th_dict[dept_name] as u64,
            };
            val.insert(dept_name.clone(), dept_obj);
        }
        val
    };

    let reshare = SessionConfig {
        algorithm: Some(algo.clone()),
        sesman_url: SESMAN_URL.to_string(),
        players: signers.clone(),
        players_reshared: players.clone(),
        threshold: th as u64,
        ..Default::default()
    };

    let mut signers: HashMap<String, Department> = HashMap::new();
    let mut signers_flat: HashSet<String> = HashSet::new();
    for (dept_name, dept_obj) in players.iter() {
        let dept_th = dept_obj.threshold as usize;
        let mut dept_signers: HashMap<String, bool> = HashMap::new();
        let mut dept_members: Vec<String> = dept_obj.players.keys().cloned().collect();

        dept_members.shuffle(&mut rng);
        let n = rng.gen_range(dept_th..=dept_obj.players.len());
        let mut i = 0;
        while i < n {
            let player = dept_members[i].clone();
            signers_flat.insert(player.clone());
            dept_signers.insert(player.clone(), true);
            i += 1;
        }
        while i < dept_members.len() {
            let player = dept_members[i].clone();
            dept_signers.insert(player.clone(), false);
            i += 1;
        }
        let dept_obj = Department {
            name: dept_name.clone(),
            players: dept_signers,
            threshold: 0,
        };
        signers.insert(dept_name.clone(), dept_obj);
    }
    if signers_flat.len() < th {
        let all_signers: HashSet<String> = players1.iter().map(|x| x.to_string()).collect();
        let mut rest_signers: Vec<String> =
            all_signers.difference(&signers_flat).cloned().collect();
        rest_signers.shuffle(&mut rng);
        let n = th - signers_flat.len();
        let rest_signers: HashSet<String> = rest_signers.into_iter().take(n).collect();

        for (_, dept_obj) in signers.iter_mut() {
            let players: Vec<String> = dept_obj.players.keys().cloned().collect();
            for player in players.iter() {
                if rest_signers.contains(player) {
                    dept_obj.players.insert(player.clone(), true);
                }
            }
        }
    }

    let sign_after_reshare = SessionConfig {
        algorithm: Some(algo.clone()),
        sesman_url: SESMAN_URL.to_string(),
        players: signers.clone(),
        ..Default::default()
    };

    MockConfigs {
        keygen,
        sign,
        reshare,
        sign_after_reshare,
    }
}

pub fn mock_mnem() -> Mnemonic {
    Mnemonic {
        words: "park remain person kitchen mule spell knee armed position rail grid ankle"
            .to_owned(),
        password: "".to_owned(),
    }
}
