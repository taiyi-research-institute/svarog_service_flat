#![allow(nonstandard_style)]
use std::collections::BTreeMap;

use erreur::*;
use svarog_grpc::{
    mpc_peer_client::MpcPeerClient, KeyTag, ParamsKeygen, ParamsReshare, ParamsSign,
};
use tonic::Request;

mod mock_data;
use mock_data::*;

/// 集成测试普通的keygen, sign
#[tokio::main]
async fn main() -> Resultat<()> {
    let mut peer = MpcPeerClient::connect(PEER_URL).await.catch_()?;

    use svarog_grpc::{Algorithm, Curve, Scheme};
    let algorithms = [
        Algorithm {
            curve: Curve::Secp256k1.into(),
            scheme: Scheme::ElGamal.into(),
        },
        Algorithm {
            curve: Curve::Ed25519.into(),
            scheme: Scheme::Schnorr.into(),
        },
    ];

    for algo in algorithms.iter().cloned() {
        let cfg = mockcfg(&algo);

        println!(
            " ========== BEGIN Testing {:?}-{:?} ========== ",
            algo.scheme(),
            algo.curve()
        );
        let key_tags: BTreeMap<String, KeyTag> = {
            let tag = peer
                .new_session(Request::new(cfg.keygen.clone()))
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;
            let mut threads = BTreeMap::new();
            for (_, dept_obj) in cfg.keygen.players.iter() {
                for (player, _) in dept_obj.players.iter() {
                    let req = Request::new(ParamsKeygen {
                        sesman_url: cfg.keygen.sesman_url.clone(),
                        session_id: sid.clone(),
                        member_name: player.clone(),
                    });

                    let mut peer = peer.clone();
                    let future = async move { peer.keygen(req).await };
                    let thread = tokio::spawn(future);
                    threads.insert(player.clone(), thread);
                    println!("BEGIN dummy keygen -- {}", player);
                }
            }

            let mut key_tags = BTreeMap::new();
            for (player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                key_tags.insert(player.clone(), resp);
                println!("END dummy keygen -- {}", player);
            }
            key_tags
        };

        let key_tags = {
            let tag = peer
                .new_session(Request::new(cfg.reshare.clone()))
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;

            let mut threads = BTreeMap::new();

            // spawn thread for reshare providers
            for (_, dept_obj) in cfg.reshare.players.iter() {
                for (player, &att) in dept_obj.players.iter() {
                    if false == att {
                        continue;
                    }
                    let key_id = key_tags.get(player).ifnone_()?.key_id.clone();
                    let req = Request::new(ParamsReshare {
                        sesman_url: cfg.reshare.sesman_url.clone(),
                        session_id: sid.clone(),
                        member_name: player.clone(),
                        key_id,
                    });

                    let mut peer = peer.clone();
                    let future = async move { peer.reshare(req).await };
                    let thread = tokio::spawn(future);
                    threads.insert(player.clone(), thread);
                    println!("BEGIN reshare -- {}", player);
                }
            }

            // spawn threads for reshare consumers not in providers
            for (_, dept_obj) in cfg.reshare.players_reshared.iter() {
                for (player, _) in dept_obj.players.iter() {
                    if threads.contains_key(player) {
                        continue;
                    }
                    let req = Request::new(ParamsReshare {
                        sesman_url: cfg.reshare.sesman_url.clone(),
                        session_id: sid.clone(),
                        member_name: player.clone(),
                        key_id: "".to_owned(),
                    });

                    let mut peer = peer.clone();
                    let future = async move { peer.reshare(req).await };
                    let thread = tokio::spawn(future);
                    threads.insert(player.clone(), thread);
                    println!("BEGIN reshare -- {} (exclusive consumer)", player);
                }
            }

            let mut keystores = BTreeMap::new();
            for (player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                keystores.insert(player.clone(), resp);
                println!("END reshare -- {}", player);
            }
            keystores
        };

        let signatures = {
            let tag = peer
                .new_session(Request::new(cfg.sign_after_reshare.clone()))
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;
            let mut threads = BTreeMap::new();
            for (_, dept_obj) in cfg.sign_after_reshare.players.iter() {
                for (player, &att) in dept_obj.players.iter() {
                    if false == att {
                        continue;
                    }
                    let key_id = key_tags.get(player).ifnone_()?.key_id.clone();
                    let req = Request::new(ParamsSign {
                        sesman_url: cfg.sign_after_reshare.sesman_url.to_owned(),
                        session_id: sid.clone(),
                        key_id,
                        member_name: player.clone(),
                        tasks: mock_sign_tasks(),
                    });

                    let mut peer = peer.clone();
                    let future = async move { peer.sign(req).await };
                    let thread = tokio::spawn(future);
                    threads.insert(player, thread);
                    println!("BEGIN sign -- {}", player);
                }
            }

            let mut signatures = BTreeMap::new();
            for (&player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                signatures.insert(player.clone(), resp);
                println!("END sign -- {}", player);
            }
            signatures
        };

        let mut sig_it = signatures.values();
        let sig0 = sig_it.next().ifnone_()?;
        for sig in sig_it {
            assert_throw!(sig == sig0);
        }
        println!(
            " ========== END Testing {:?}-{:?} ========== ",
            algo.scheme(),
            algo.curve()
        );
    }

    Ok(())
}
