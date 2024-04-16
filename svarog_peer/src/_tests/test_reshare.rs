#![allow(nonstandard_style)]
use std::collections::{BTreeMap, BTreeSet};

use erreur::*;
use svarog_grpc::{
    mpc_peer_client::MpcPeerClient, Algorithm, ParamsKeygen, ParamsReshare, ParamsSign,
};
use tonic::Request;

use crate::mock_data::{
    mock_keygen_config, mock_one_sign_task, mock_reshare_config, mock_sign_config, players1, players2, th1, th2
};

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
        let keystores_old = {
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
                println!("BEGIN {:#?} dummy keygen -- {}", &algo, player);
            }
            let mut keystores = BTreeMap::new();
            for (player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                keystores.insert(player.clone(), resp);
                println!("END {:#?} dummy keygen -- {}", &algo, player);
            }
            keystores
        };
        println!("END {:#?} dummy keygen", &algo);

        let keystores = {
            let (mut cfg, exclusive_consumers) =
                mock_reshare_config(th1, &players1, th2, &players2);
            cfg.sesman_url = sesman_url.to_string();
            cfg.algorithm = algo.into();
            '_print_providers: {
                let mut providers = BTreeSet::new();
                for (player, &att) in cfg.players.iter() {
                    if false == att {
                        continue;
                    }
                    providers.insert(player);
                }
                let mut consumers = BTreeSet::new();
                for (player, _) in cfg.players_reshared.iter() {
                    consumers.insert(player.clone());
                }
                println!("Providers: {:?}", &providers);
                println!("Consumers: {:?}", &consumers);
                println!("Exclusive Consumers: {:?}", &exclusive_consumers);
            }
            let tag = peer
                .new_session(Request::new(cfg.clone()))
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;

            let mut threads = BTreeMap::new();

            // spawn thread for reshare providers
            for (player, &att) in cfg.players.iter() {
                if false == att {
                    continue;
                }
                let keystore = keystores_old.get(player).ifnone_()?;
                let req = Request::new(ParamsReshare {
                    sesman_url: sesman_url.to_owned(),
                    session_id: sid.clone(),
                    member_name: player.clone(),
                    keystore: Some(keystore.clone()),
                });

                let mut peer = peer.clone();
                let future = async move { peer.reshare(req).await };
                let thread = tokio::spawn(future);
                threads.insert(player.clone(), thread);
                println!("BEGIN {:#?} reshare -- {}", &algo, player);
            }

            // spawn threads for reshare consumers not in providers
            for player in exclusive_consumers.iter() {
                let req = Request::new(ParamsReshare {
                    sesman_url: sesman_url.to_owned(),
                    session_id: sid.clone(),
                    member_name: player.to_owned(),
                    keystore: None,
                });

                let mut peer = peer.clone();
                let future = async move { peer.reshare(req).await };
                let thread = tokio::spawn(future);
                threads.insert(player.clone(), thread);
                println!(
                    "BEGIN {:#?} reshare -- {} (exclusive consumer)",
                    &algo, player
                );
            }

            let mut keystores = BTreeMap::new();
            for (player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                keystores.insert(player.clone(), resp);
                println!("END {:#?} reshare -- {}", &algo, player);
            }
            keystores
        };

        let signatures = {
            let mut cfg = mock_sign_config(th2, &players2);
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
                let keystore = keystores.get(player).ifnone_()?.value.as_ref().ifnone_()?;
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
