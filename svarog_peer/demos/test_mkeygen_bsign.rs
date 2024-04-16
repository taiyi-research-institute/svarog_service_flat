#![allow(nonstandard_style)]
use std::collections::BTreeMap;

use erreur::*;
use svarog_grpc::{mpc_peer_client::MpcPeerClient, Algorithm, ParamsKeygen, ParamsKeygenMnem, ParamsSign};

mod mock_data;
use mock_data::*;
use tonic::{IntoRequest, Request};

const peer_url: &str = "http://127.0.0.1:9001";
const sesman_url: &str = "http://127.0.0.1:9000";
const algorithms: [Algorithm; 2] = [Algorithm::Gg18Secp256k1, Algorithm::FrostEd25519];

/// 集成测试普通的keygen, sign
#[tokio::main]
async fn main() -> Resultat<()> {
    let mut peer = MpcPeerClient::connect(peer_url).await.catch_()?;

    for algo in algorithms.iter().cloned() {
        let keystores = {
            let mut cfg = mock_keygen_config(th1, &players1);
            cfg.sesman_url = sesman_url.to_string();
            cfg.algorithm = algo.into();
            let tag = peer
                .new_session(cfg.clone().into_request())
                .await
                .catch_()?
                .into_inner();
            let sid = tag.session_id;
            let mut threads = BTreeMap::new();
            for (player, _) in cfg.players.iter() {
                let player = player.clone();
                let req = Request::new(ParamsKeygenMnem {
                    sesman_url: sesman_url.to_owned(),
                    session_id: sid.clone(),
                    member_name: player.to_owned(),
                    mnemonic: None,
                });
    
                let mut peer = peer.clone();
                let future = async move { peer.keygen_mnem(req).await };
                let thread = tokio::spawn(future);
                threads.insert(player, thread);
            }
            '_mnem_provider: {
                let req = Request::new(ParamsKeygenMnem {
                    sesman_url: sesman_url.to_owned(),
                    session_id: sid.clone(),
                    member_name: "".to_owned(),
                    mnemonic: Some(mock_mnem()),
                });

                let mut peer = peer.clone();
                let future = async move { peer.keygen_mnem(req).await };
                let thread = tokio::spawn(future);
                threads.insert("".to_owned(), thread);
            }
            let mut keystores = BTreeMap::new();
            for (player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                if let Some(resp) = resp.value {
                    keystores.insert(player.clone(), resp);
                }
            }
            keystores
        };

        let signatures = {
            let mut cfg = mock_keygen_config(th1, &players1);
            cfg.sesman_url = sesman_url.to_string();
            cfg.algorithm = algo.into();
            let tag = peer
                .new_session(cfg.clone().into_request())
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
                    tasks: mock_sign_tasks(),
                });
    
                let mut peer = peer.clone();
                let future = async move { peer.sign(req).await };
                let thread = tokio::spawn(future);
                threads.insert(player, thread);
            }
            let mut signatures = BTreeMap::new();
            for (&player, thread) in threads.iter_mut() {
                let resp = thread
                    .await
                    .catch("Panic", "")?
                    .catch("Exception", "")?
                    .into_inner();
                signatures.insert(player.clone(), resp);
            }
            signatures
        };

        let mut sig_it = signatures.values();
        let sig0 = sig_it.next().ifnone_()?;
        for sig in sig_it {
            assert_throw!(sig == sig0);
        }
    }

    Ok(())
}