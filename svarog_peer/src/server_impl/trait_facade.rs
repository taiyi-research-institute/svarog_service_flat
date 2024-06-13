/// 模块职责:
/// 1. 将player名称转换为数字id
/// 2. 将Keystore和Signature转换为内部数据结构, 以及反向转换.
/// 3. 调用算法实现.
use std::collections::{BTreeMap, BTreeSet, HashMap};

use erreur::*;
use svarog_grpc::{
    mpc_peer_server::MpcPeer, Curve, Keystore, OptionalKeystore, ParamsKeygen, ParamsKeygenMnem,
    ParamsReshare, ParamsSign, Scheme, SessionConfig, SessionTag, Signature, VecSignature,
};
use svarog_sesman::SvarogChannel;
use tonic::{Request, Response, Status};

use super::biz::{
    keygen_frost, keygen_gg18, keygen_mnem_frost, keygen_mnem_gg18, reshare_frost, reshare_gg18,
    sign_frost, sign_gg18,
};

#[derive(Clone, Copy)]
pub(crate) struct SvarogPeer;

fn ses_arch(name: &str, names: &HashMap<String, bool>) -> (usize, BTreeSet<usize>) {
    let names: BTreeMap<String, bool> = names.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let mut i = 0;
    let mut players = BTreeSet::new();
    for (j, (_name, &att)) in names.iter().enumerate() {
        if att {
            let j = j + 1;
            players.insert(j);
            if name == _name {
                i = j;
            }
        }
    }
    (i, players)
}

#[tonic::async_trait]
impl MpcPeer for SvarogPeer {
    async fn new_session(
        &self,
        request: Request<SessionConfig>,
    ) -> Result<Response<SessionTag>, Status> {
        let cfg = request.into_inner();
        println!("{}", &cfg.sesman_url);
        let chan = SvarogChannel::new_session(&cfg, &cfg.sesman_url)
            .await
            .catch_()
            .map_err(|e| Status::internal(e.to_string()))?;
        let tag = SessionTag {
            session_id: chan.sid().to_owned(),
            expire_at: chan.expire_at(),
        };
        Ok(Response::new(tag))
    }

    async fn keygen(&self, request: Request<ParamsKeygen>) -> Result<Response<Keystore>, Status> {
        async fn foo(request: Request<ParamsKeygen>) -> Resultat<Keystore> {
            let params = request.into_inner();
            let (chan, cfg) = SvarogChannel::use_session(&params.session_id, &params.sesman_url)
                .await
                .catch_()?;
            let t = cfg.threshold as usize;
            let (i, players) = ses_arch(&params.member_name, &cfg.players);
            assert_throw!(
                players.len() == cfg.players.len(),
                "all keygen members should attend"
            );
            let algo = cfg.algorithm.ifnone_()?;
            let curve = algo.curve();
            let scheme = algo.scheme();
            let keystore = match (curve, scheme) {
                (Curve::Secp256k1, Scheme::ElGamal) => {
                    keygen_gg18(chan, i, t, players).await.catch_()?
                }
                (Curve::Ed25519, Scheme::Schnorr) => {
                    keygen_frost(chan, i, t, players).await.catch_()?
                }
                _ => {
                    let msg = format!("Combination of curve {:?} and scheme {:?}.", curve, scheme);
                    throw!("NotImplemented", msg);
                }
            };
            Ok(keystore)
        }
        let keystore = foo(request)
            .await
            .catch_()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(keystore))
    }

    async fn keygen_mnem(
        &self,
        request: Request<ParamsKeygenMnem>,
    ) -> Result<Response<OptionalKeystore>, Status> {
        async fn foo(request: Request<ParamsKeygenMnem>) -> Resultat<Option<Keystore>> {
            let params = request.into_inner();
            let (chan, cfg) = SvarogChannel::use_session(&params.session_id, &params.sesman_url)
                .await
                .catch_()?;
            let t = cfg.threshold as usize;
            let (i, players) = ses_arch(&params.member_name, &cfg.players);
            assert_throw!(
                players.len() == cfg.players.len(),
                "all keygen members should attend"
            );
            let algo = cfg.algorithm.ifnone_()?;
            let curve = algo.curve();
            let scheme = algo.scheme();
            let keystore = match (curve, scheme) {
                (Curve::Secp256k1, Scheme::ElGamal) => {
                    keygen_mnem_gg18(chan, i, t, players, params.mnemonic)
                        .await
                        .catch_()?
                }
                (Curve::Ed25519, Scheme::Schnorr) => {
                    keygen_mnem_frost(chan, i, t, players, params.mnemonic)
                        .await
                        .catch_()?
                }
                _ => {
                    let msg = format!("Combination of curve {:?} and scheme {:?}.", curve, scheme);
                    throw!("NotImplemented", msg);
                }
            };
            Ok(keystore)
        }
        let keystore = foo(request)
            .await
            .catch_()
            .map_err(|e| Status::internal(e.to_string()))?;
        let keystore = OptionalKeystore { value: keystore };
        Ok(Response::new(keystore))
    }

    async fn sign(&self, request: Request<ParamsSign>) -> Result<Response<VecSignature>, Status> {
        async fn foo(request: Request<ParamsSign>) -> Resultat<Vec<Signature>> {
            let params = request.into_inner();
            let (chan, cfg) = SvarogChannel::use_session(&params.session_id, &params.sesman_url)
                .await
                .catch_()?;
            let (_, signers) = ses_arch("", &cfg.players);
            let keystore = params.keystore.ifnone_()?;
            let i = keystore.i as usize;
            assert_throw!(signers.contains(&i), "signer not in the session");
            let tasks = params.tasks;
            let algo = cfg.algorithm.ifnone_()?;
            let curve = algo.curve();
            let scheme = algo.scheme();
            let sigs = match (curve, scheme) {
                (Curve::Secp256k1, Scheme::ElGamal) => {
                    sign_gg18(chan, keystore, signers, tasks).await.catch_()?
                }
                (Curve::Ed25519, Scheme::Schnorr) => {
                    sign_frost(chan, keystore, signers, tasks).await.catch_()?
                }
                _ => {
                    let msg = format!("Combination of curve {:?} and scheme {:?}.", curve, scheme);
                    throw!("NotImplemented", msg);
                }
            };
            Ok(sigs)
        }
        let sigs = foo(request)
            .await
            .catch_()
            .map_err(|e| Status::internal(e.to_string()))?;
        let sigs = VecSignature { values: sigs };
        Ok(Response::new(sigs))
    }

    async fn reshare(
        &self,
        request: Request<ParamsReshare>,
    ) -> Result<Response<OptionalKeystore>, Status> {
        async fn foo(request: Request<ParamsReshare>) -> Resultat<Option<Keystore>> {
            let params = request.into_inner();
            let (chan, cfg) = SvarogChannel::use_session(&params.session_id, &params.sesman_url)
                .await
                .catch_()?;
            let keystore = params.keystore;
            let t = cfg.threshold as usize;
            let (_, providers) = ses_arch("", &cfg.players);
            println!("reshare providers: {:?}", &providers);
            if let Some(keystore) = &keystore {
                let i0 = keystore.i as usize;
                println!(
                    "reshare provider: Player {}, ID {}",
                    &params.member_name, i0
                );
                assert_throw!(providers.contains(&i0), "provider not in the session");
            }
            let (i, consumers) = ses_arch(&params.member_name, &cfg.players_reshared);
            println!("reshare consumers: {:?}", &consumers);
            if i > 0 {
                println!("reshare consumer: Player {}, ID {}", &params.member_name, i);
            }
            assert_throw!(
                consumers.len() == cfg.players_reshared.len(),
                "all keygen members should attend"
            );
            let algo = cfg.algorithm.ifnone_()?;
            let curve = algo.curve();
            let scheme = algo.scheme();
            let keystore = match (curve, scheme) {
                (Curve::Secp256k1, Scheme::ElGamal) => {
                    reshare_gg18(chan, keystore, i, t, providers, consumers)
                        .await
                        .catch_()?
                }
                (Curve::Ed25519, Scheme::Schnorr) => {
                    reshare_frost(chan, keystore, i, t, providers, consumers)
                        .await
                        .catch_()?
                }
                _ => {
                    let msg = format!("Combination of curve {:?} and scheme {:?}.", curve, scheme);
                    throw!("NotImplemented", msg);
                }
            };
            Ok(keystore)
        }

        let keystore = foo(request)
            .await
            .catch_()
            .map_err(|e| Status::internal(e.to_string()))?;
        let keystore = OptionalKeystore { value: keystore };
        Ok(Response::new(keystore))
    }
}
