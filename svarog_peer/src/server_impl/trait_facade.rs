/// 模块职责: 加工算法参数, 调用算法实现.
use std::collections::{BTreeMap, BTreeSet, HashMap};

use erreur::*;
use svarog_grpc::{
    mpc_peer_server::MpcPeer, Keystore, OptionalKeystore, ParamsKeygen, ParamsKeygenMnem,
    ParamsReshare, ParamsSign, SessionConfig, SessionTag, Signature, VecSignature,
};
use svarog_sesman::SvarogChannel;
use tonic::{Request, Response, Status};

use crate::server_impl::biz::{keygen_mnem_frost, keygen_mnem_gg18, reshare_frost, reshare_gg18, sign_frost, sign_gg18};

use super::biz::{keygen_frost, keygen_gg18};

#[derive(Clone)]
pub(crate) struct SvarogPeer;

fn ses_arch(name: &str, names: &HashMap<String, bool>) -> (usize, BTreeSet<usize>) {
    let names: BTreeMap<String, bool> = names.iter().map(|(k, _)| (k.clone(), true)).collect();
    let mut i = 0;
    let mut players = BTreeSet::new();
    for (j, (_name, &att)) in names.iter().enumerate() {
        if att {
            players.insert(j);
            if name == _name {
                i = j + 1;
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
            let keystore = match cfg.algorithm() {
                svarog_grpc::Algorithm::DontCare => {
                    throw!("", "Algorithm not specified");
                }
                svarog_grpc::Algorithm::Gg18Secp256k1 => {
                    keygen_gg18(chan, i, t, players).await.catch_()?
                }
                svarog_grpc::Algorithm::FrostEd25519 => {
                    keygen_frost(chan, i, t, players).await.catch_()?
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
            let keystore = match cfg.algorithm() {
                svarog_grpc::Algorithm::DontCare => {
                    throw!("", "Algorithm not specified");
                }
                svarog_grpc::Algorithm::Gg18Secp256k1 => {
                    keygen_mnem_gg18(chan, i, t, players, params.mnemonic)
                        .await
                        .catch_()?
                }
                svarog_grpc::Algorithm::FrostEd25519 => {
                    keygen_mnem_frost(chan, i, t, players, params.mnemonic)
                        .await
                        .catch_()?
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
            let sigs = match cfg.algorithm() {
                svarog_grpc::Algorithm::DontCare => {
                    throw!("", "Algorithm not specified");
                }
                svarog_grpc::Algorithm::Gg18Secp256k1 => {
                    sign_gg18(chan, keystore, signers, tasks).await.catch_()?
                }
                svarog_grpc::Algorithm::FrostEd25519 => {
                    sign_frost(chan, keystore, signers, tasks).await.catch_()?
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
            if let Some(keystore) = &keystore {
                let i0 = keystore.i as usize;
                assert_throw!(providers.contains(&i0), "provider not in the session");
            }
            let (i, consumers) = ses_arch(&params.member_name, &cfg.players_reshared);
            assert_throw!(
                consumers.len() == cfg.players_reshared.len(),
                "all keygen members should attend"
            );
            let keystore = match cfg.algorithm() {
                svarog_grpc::Algorithm::DontCare => {
                    throw!("", "Algorithm not specified");
                }
                svarog_grpc::Algorithm::Gg18Secp256k1 => {
                    reshare_gg18(chan, keystore, i, t, providers, consumers)
                        .await
                        .catch_()?
                }
                svarog_grpc::Algorithm::FrostEd25519 => {
                    reshare_frost(chan, keystore, i, t, providers, consumers)
                        .await
                        .catch_()?
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
