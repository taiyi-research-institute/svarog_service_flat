#![allow(nonstandard_style)]
use std::collections::{BTreeMap, BTreeSet};

use erreur::*;
use svarog_grpc::{mpc_peer_client::MpcPeerClient, Algorithm, ParamsKeygen, ParamsSign};
use tonic::Request;

// 改成通配符引用之后, 会难以检查到底用了哪些符号. 通配符看着优雅, 但是不利于代码审查.
use crate::mock_data::{mock_keygen_config, mock_one_sign_task, mock_sign_config, players1, th1};

mod mock_data;
const peer_url: &str = "http://127.0.0.1:9001";
const sesman_url: &str = "http://127.0.0.1:9000";
const algorithms: [Algorithm; 2] = [Algorithm::Gg18Secp256k1, Algorithm::FrostEd25519];

/// 集成测试普通的keygen, sign
#[tokio::main]
async fn main() -> Resultat<()> {
    let mut peer = MpcPeerClient::connect(peer_url).await.catch_()?;

    for algo in algorithms.iter().cloned() {
        println!(" ========== BEGIN Testing {:#?} ========== ", &algo);
        let keystores = {
            let mut cfg = mock_keygen_config(th1, &players1);
            cfg.sesman_url = sesman_url.to_string();
            cfg.algorithm = algo.into();
            let tag = peer
                .new_session(Request::new(cfg.clone()))
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;
            let mut threads = BTreeMap::new();
            for (player, _) in cfg.players.iter() {
                let req = Request::new(ParamsKeygen {
                    sesman_url: sesman_url.to_owned(),
                    session_id: sid.clone(),
                    member_name: player.clone(),
                });

                let mut peer = peer.clone();
                let future = async move { peer.keygen(req).await };
                let thread = tokio::spawn(future);
                threads.insert(player.clone(), thread);
                println!("BEGIN {:#?} keygen -- {}", &algo, player);
            }
            let mut keystores = BTreeMap::new();
            for (player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                keystores.insert(player.clone(), resp);
                println!("END {:#?} keygen -- {}", &algo, player);
            }
            keystores
        };
        println!("END {:#?} keygen", &algo);

        let signatures = {
            let mut cfg = mock_sign_config(th1, &players1);
            cfg.sesman_url = sesman_url.to_string();
            cfg.algorithm = algo.into();
            '_print_signers: {
                let mut signers = BTreeSet::new();
                for (player, &att) in cfg.players.iter() {
                    if false == att {
                        continue;
                    }
                    signers.insert(player);
                }
                println!("Signers: {:?}", &signers);
            }
            let tag = peer
                .new_session(Request::new(cfg.clone()))
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;
            let mut threads = BTreeMap::new();
            for (player, &att) in cfg.players.iter() {
                if false == att {
                    continue;
                }
                let keystore = keystores.get(player).ifnone_()?;
                let req = Request::new(ParamsSign {
                    sesman_url: sesman_url.to_owned(),
                    session_id: sid.clone(),
                    keystore: Some(keystore.clone()),
                    tasks: mock_one_sign_task(),
                });

                let mut peer = peer.clone();
                let future = async move { peer.sign(req).await };
                let thread = tokio::spawn(future);
                threads.insert(player, thread);
                println!("BEGIN {:#?} sign -- {}", &algo, player);
            }
            let mut signatures = BTreeMap::new();
            for (&player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                signatures.insert(player.clone(), resp);
                println!("END {:#?} sign -- {}", &algo, player);
            }
            signatures
        };

        let mut sig_it = signatures.values();
        let sig0 = sig_it.next().ifnone_()?;
        for sig in sig_it {
            assert_throw!(sig == sig0);
        }
        println!(" ========== END Testing {:#?} ========== ", &algo);
    }

    Ok(())
}
