use erreur::*;
use svarog_grpc::{
    mpc_peer_server::MpcPeer, ParamsKeygen, ParamsReshare, ParamsSign, SessionConfig, SessionTag,
    VecSignature, Void,
};
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Peer;

impl Peer {
    pub async fn grpc_new_session(&self, cfg: SessionConfig) -> Resultat<SessionTag> {
        let messenger = SvarogChannel::new_session(&cfg, &cfg.sesman_url)
            .await
            .catch_()?;
        Ok(SessionTag {
            session_id: messenger.session_id.clone(),
            expire_at: messenger.expire_at.clone(),
        })
    }

    pub async fn grpc_keygen(&self, params: ParamsKeygen) -> Resultat<Keystore> {
        if params.mnemonics.is_some() {
            throw!("NotImplemented", "keygen from main mnemonics");
        }
        let (chan, cfg) = SvarogChannel::use_session(&params.session_id, &params.sesman_url)
            .await
            .catch_()?;
        let (i, n) = {
            let mut players = BTreeSet::new();
            for (player, &player_attending) in cfg.players.iter() {
                assert_throw!(player_attending);
                players.insert(player.clone());
            }
            assert_throw!(players.len() >= 1);
            assert_throw!(players.contains(&params.member_name));
            let i: usize;
            for (j, player) in players.iter().enumerate() {
                if player == &params.member_name {
                    i = j + 1;
                    break;
                }
            }
            (i, players.len())
        };

        Ok(keystore)
    }

    pub async fn grpc_sign(&self, params: ParamsSign) -> Resultat<Vec<Signature>> {
        assert_throw!(params.tasks.len() >= 1);

        // Prepare sign parameters: messenger, ses_arch, keystore
        let (messenger, cfg) = SvarogChannel::use_session(&params.session_id, &params.sesman_url)
            .await
            .catch_()?;
        let messenger = Arc::new(messenger);
        let (ses_arch, _i_set) = dump_arch(&cfg.groups, &params.member_name, false).catch_()?;
        let path = format!("assets/{}@{}.dat", &params.member_name, &params.key_name);
        let buf = tokio::fs::read(&path)
            .await
            .catch("Cannot read file", path)?;
        let keystore: Box<dyn Any> = match cfg.algorithm() {
            Algorithm::Gg18Secp256k1 => {
                type K = svarog_algo::gg18::keystore::Keystryoshka;
                let inner: K = K::unpickle(&buf).catch_()?;
                Box::new(inner)
            }
            Algorithm::FrostEd25519 => {
                type K = svarog_algo::frost::keystore::Keystryoshka;
                let inner: K = K::unpickle(&buf).catch_()?;
                Box::new(inner)
            }
            _ => {
                throw!("NotImplemented", "unknown algorithm");
            }
        };

        // Call sign.
        let sigs: Vec<Signature> = match cfg.algorithm() {
            Algorithm::Gg18Secp256k1 => {
                type K = svarog_algo::gg18::keystore::Keystryoshka;
                let fn_sign = svarog_algo::gg18::sign;
                let keystore: Box<K> = keystore.downcast().unwrap();

                if params.tasks.len() > 1 {
                    throw!("NotImplemented", "GG18 batch sign");
                } else {
                    // already asserted params.tasks.len() >= 1
                    let messenger = messenger.clone();
                    let ses_arch = ses_arch.clone();
                    let keystore = keystore.as_ref().clone();
                    let hmsg = params.tasks[0].tx_hash.clone();
                    let dpath = params.tasks[0].derivation_path.clone();
                    let sig = fn_sign(messenger, ses_arch, keystore, hmsg.clone(), dpath.clone())
                        .await
                        .catch_()?;
                    let sig = Signature {
                        r: sig.R.to33bytes().to_vec(),
                        s: sig.s.to_bytes().to_vec(),
                        v: sig.v as u32,
                        derivation_path: dpath,
                        tx_hash: hmsg,
                    };
                    vec![sig]
                }
            }
            Algorithm::FrostEd25519 => {
                type K = svarog_algo::frost::keystore::Keystryoshka;
                let fn_sign = svarog_algo::frost::sign;
                let keystore: Box<K> = keystore.downcast().unwrap();

                if params.tasks.len() > 1 {
                    throw!("NotImplemented", "FROST batch sign");
                } else {
                    // already asserted params.tasks.len() >= 1
                    let messenger = messenger.clone();
                    let ses_arch = ses_arch.clone();
                    let keystore = keystore.as_ref().clone();
                    let hmsg = params.tasks[0].tx_hash.clone();
                    let dpath = params.tasks[0].derivation_path.clone();
                    let sig = fn_sign(messenger, ses_arch, keystore, hmsg.clone(), dpath.clone())
                        .await
                        .catch_()?;
                    let sig = Signature {
                        r: sig.R.compress().to_bytes().to_vec(),
                        s: sig.s.to_bytes().to_vec(),
                        v: 0,
                        derivation_path: dpath,
                        tx_hash: hmsg,
                    };
                    vec![sig]
                }
            }
            _ => {
                throw!("NotImplemented", "unknown algorithm");
            }
        };

        Ok(sigs)
    }

    #[allow(unused_variables)]
    pub async fn grpc_reshare(&self, params: ParamsReshare) -> Resultat<()> {
        // Prepare reshare parameters after barrier: messenger, ses_arch, thres_arch, i_set
        let (messenger, cfg) = SvarogChannel::use_session(&params.session_id, &params.sesman_url)
            .await
            .catch_()?;
        let messenger = Arc::new(messenger);
        let (ses_arch, i_set) =
            dump_arch(&cfg.groups_after_reshare, &params.member_name, true).catch_()?;
        let thres_arch = dump_thres_arch(
            &cfg.groups_after_reshare,
            cfg.threshold_after_reshare as usize,
        );
        let thres_arch: BTreeMap<u16, usize> = thres_arch.into_iter().collect();

        // Prepare reshare parameters before barrier: keystore, provider_arch, provider_i_set
        let path = format!("assets/{}@{}.dat", &params.member_name, &params.key_name);
        let buf = tokio::fs::read(&path)
            .await
            .catch("Cannot read file", path)?;
        let keystore: Box<dyn Any> = match cfg.algorithm() {
            Algorithm::Gg18Secp256k1 => {
                type K = svarog_algo::gg18::keystore::Keystryoshka;
                let inner: K = K::unpickle(&buf).catch_()?;
                Box::new(inner)
            }
            Algorithm::FrostEd25519 => {
                type K = svarog_algo::frost::keystore::Keystryoshka;
                let inner: K = K::unpickle(&buf).catch_()?;
                Box::new(inner)
            }
            _ => {
                throw!("NotImplemented", "unknown algorithm");
            }
        };
        let (provider_arch, provider_i_set) =
            dump_arch(&cfg.groups, &params.member_name, false).catch_()?;

        throw!("NotImplemented", "Reshare");
    }
}


#[tonic::async_trait]
impl MpcPeer for Peer {
    async fn new_session(
        &self,
        request: Request<SessionConfig>,
    ) -> Result<Response<SessionTag>, Status> {
        let cfg = request.into_inner();
        let tag = self
            .grpc_new_session(cfg)
            .await
            .catch("GrpcCallFailed", "MpcPeer::NewSession")
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(tag))
    }

    async fn keygen(&self, request: Request<ParamsKeygen>) -> Result<Response<RootAddr>, Status> {
        let params = request.into_inner();
        let root_addr = self
            .grpc_keygen(params)
            .await
            .catch("GrpcCallFailed", "MpcPeer::Keygen")
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(RootAddr { value: root_addr }))
    }

    async fn sign(&self, request: Request<ParamsSign>) -> Result<Response<VecSignature>, Status> {
        let params = request.into_inner();
        let sig_vec = self
            .grpc_sign(params)
            .await
            .catch("GrpcCallFailed", "MpcPeer::Sign")
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(VecSignature { values: sig_vec }))
    }

    async fn reshare(&self, request: Request<ParamsReshare>) -> Result<Response<Void>, Status> {
        let params = request.into_inner();
        self.grpc_reshare(params)
            .await
            .catch("GrpcCallFailed", "MpcSign::Reshare")
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(Void {}))
    }
}
