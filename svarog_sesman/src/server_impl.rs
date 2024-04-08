use std::sync::Arc;

use crossbeam_skiplist::SkipSet;
use dashmap::DashMap;
use erreur::*;
use svarog_grpc::{
    mpc_session_manager_server::MpcSessionManager, Message, SessionConfig, SessionTag, VecMessage,
    Void,
};
use tokio::{
    task::JoinHandle,
    time::{sleep, Duration},
};
use tonic::{Request, Response, Status};

pub fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    epoch.as_secs()
}

#[derive(Clone)]
pub struct Sesman {
    sessions: Sessions,
    history: History,
    span: u64,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SessionExpireAt {
    session_id: String,
    expire_at: u64,
}

type History = Arc<SkipSet<SessionExpireAt>>;
type Messages = DashMap<String, Vec<u8>>;
type Sessions = Arc<DashMap<String, Messages>>;

impl Sesman {
    pub async fn init(
        span: u64, // seconds of session lifespan
    ) -> Resultat<(Self, JoinHandle<()>)> {
        let sessions = Arc::new(DashMap::new());
        let history = Arc::new(SkipSet::new());
        let sesman = Self {
            sessions,
            history,
            span,
        };
        let h = tokio::spawn(sesman.clone().recycle());

        Ok((sesman, h))
    }

    async fn recycle(self) {
        loop {
            let now = now();
            for entry in self.history.iter() {
                if entry.expire_at < now {
                    self.sessions.remove(&entry.session_id);
                    entry.remove();
                } else {
                    break;
                }
            }
            sleep(Duration::from_secs(60)).await;
        }
    }
}

#[tonic::async_trait]
impl MpcSessionManager for Sesman {
    async fn new_session(
        &self,
        request: Request<SessionConfig>,
    ) -> Result<Response<SessionTag>, Status> {
        let mut cfg = request.into_inner();
        if cfg.session_id == "" {
            cfg.session_id = hex::encode(uuid::Uuid::new_v4().as_bytes()).to_lowercase();
        }
        let expire_at = now()
            .checked_add(self.span)
            .ifnone("IntegerOverflow", "... when calculating `expire_at`.")
            .map_err(|e| Status::internal(e.to_string()))?;
        cfg.expire_at = expire_at;

        let cfg_bytes = serde_pickle::to_vec(&cfg, Default::default())
            .catch_()
            .map_err(|e| Status::internal(e.to_string()))?;
        self.sessions.insert(cfg.session_id.clone(), DashMap::new());
        self.history.insert(SessionExpireAt {
            session_id: cfg.session_id.clone(),
            expire_at,
        });
        let session = self.sessions.get(&cfg.session_id).unwrap();
        session.insert("session_config".to_string(), cfg_bytes);

        let tag = SessionTag {
            session_id: cfg.session_id.clone(),
            expire_at,
        };

        Ok(Response::new(tag))
    }

    async fn get_session_config(
        &self,
        request: Request<SessionTag>,
    ) -> Result<Response<SessionConfig>, Status> {
        let sid = request.into_inner().session_id;
        let session = self
            .sessions
            .get(&sid)
            .ifnone_()
            .map_err(|_| Status::not_found(&sid))?;
        let cfg_bytes = session
            .get("session_config")
            .ifnone_()
            .map_err(|_| Status::not_found(&sid))?;
        let cfg: SessionConfig = serde_pickle::from_slice(&cfg_bytes, Default::default())
            .catch_()
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(cfg))
    }

    async fn inbox(&self, req: Request<VecMessage>) -> Result<Response<Void>, Status> {
        let msgs = req.into_inner().values;
        for msg in msgs.iter() {
            let db = self
                .sessions
                .get(&msg.session_id)
                .ifnone_()
                .map_err(|_| Status::not_found(&msg.session_id))?;
            let key = format!("{}-{}-{}-{}", &msg.topic, &msg.src, &msg.dst, &msg.seq);
            let val = msg
                .obj
                .as_ref()
                .ifnone_()
                .map_err(|_| Status::invalid_argument(&key))?;
            db.insert(key, val.clone());
        }
        Ok(Response::new(Void {}))
    }

    async fn outbox(&self, request: Request<VecMessage>) -> Result<Response<VecMessage>, Status> {
        let idxs = request.into_inner().values;
        let mut resp = Vec::new();
        for idx in idxs.iter() {
            let key = format!("{}-{}-{}-{}", &idx.topic, &idx.src, &idx.dst, &idx.seq);
            let obj = loop {
                let db = self
                    .sessions
                    .get(&idx.session_id)
                    .ifnone_()
                    .map_err(|_| Status::not_found(&idx.session_id))?;
                match db.get(&key) {
                    Some(ref_obj) => break ref_obj.value().clone(),
                    None => {
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };
            };
            resp.push(Message {
                session_id: idx.session_id.clone(),
                topic: idx.topic.clone(),
                src: idx.src,
                dst: idx.dst,
                seq: idx.seq,
                obj: Some(obj),
            })
        }

        Ok(Response::new(VecMessage { values: resp }))
    }
}
