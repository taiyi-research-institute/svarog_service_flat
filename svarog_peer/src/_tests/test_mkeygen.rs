#![allow(nonstandard_style)]
use std::collections::BTreeMap;

use erreur::*;
use svarog_grpc::{mpc_peer_client::MpcPeerClient, KeyTag, ParamsKeygenMnem, ParamsSign};
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
        Algorithm {
            curve: Curve::Secp256k1.into(),
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
                    let req = Request::new(ParamsKeygenMnem {
                        sesman_url: cfg.keygen.sesman_url.clone(),
                        session_id: sid.clone(),
                        member_name: player.clone(),
                        mnemonic: None,
                    });

                    let mut peer = peer.clone();
                    let future = async move { peer.keygen_mnem(req).await };
                    let thread = tokio::spawn(future);
                    threads.insert(player.clone(), thread);
                    println!("BEGIN keygen -- {}", player);
                }
            }
            '_mnem_provider: {
                let req = Request::new(ParamsKeygenMnem {
                    sesman_url: cfg.keygen.sesman_url.clone(),
                    session_id: sid.clone(),
                    member_name: "".to_owned(),
                    mnemonic: Some(mock_mnem()),
                });

                let mut peer = peer.clone();
                let future = async move { peer.keygen_mnem(req).await };
                let thread = tokio::spawn(future);
                threads.insert("".to_owned(), thread);
                println!("BEGIN keygen -- {}", "__mnem_provider__");
            }

            let mut key_tags = BTreeMap::new();
            for (player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                key_tags.insert(player.clone(), resp);
                println!("END keygen -- {}", player);
            }
            key_tags
        };

        let signatures = {
            let tag = peer
                .new_session(Request::new(cfg.sign.clone()))
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;
            let mut threads = BTreeMap::new();
            for (_, dept_obj) in cfg.sign.players.iter() {
                for (player, &att) in dept_obj.players.iter() {
                    if false == att {
                        continue;
                    }
                    let key_id = key_tags.get(player).ifnone_()?.key_id.clone();
                    let req = Request::new(ParamsSign {
                        sesman_url: cfg.sign.sesman_url.to_owned(),
                        session_id: sid.clone(),
                        key_id,
                        member_name: player.clone(),
                        tasks: mock_sign_tasks(&algo),
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
