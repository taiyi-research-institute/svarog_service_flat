//! Sesman client library

use std::collections::{HashMap, HashSet};

use erreur::*;
use mpc_sig_abs::BatchMessenger;
use serde::{de::DeserializeOwned, Serialize};
use svarog_grpc::{
    mpc_session_manager_client::MpcSessionManagerClient, Message, SessionConfig, SessionTag,
    VecMessage,
};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct SvarogChannel {
    sid: String,
    cl: MpcSessionManagerClient<Channel>,
    tx: Vec<Message>,
    rx: HashMap<MessageIndex, Option<Vec<u8>>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MessageIndex {
    topic: String,
    src: usize,
    dst: usize,
    seq: usize,
}

impl SvarogChannel {
    pub fn sid(&self) -> &str {
        &self.sid
    }

    pub async fn new_session(cfg: &SessionConfig, sesman_url: &str) -> Resultat<Self> {
        let mut cl = MpcSessionManagerClient::connect(sesman_url.to_owned())
            .await
            .map_err(|e| format!("{:#?}", e))
            .catch("CannotConnectGrpcServer", sesman_url)?;
        let tag = cl
            .new_session(cfg.clone())
            .await
            .catch("GrpcCallFailed", "MpcSessionManager::NewSession")?
            .into_inner();
        Ok(Self {
            sid: tag.session_id,
            cl,
            tx: Vec::new(),
            rx: HashMap::new(),
        })
    }

    pub async fn use_session(sid: &str, sesman_url: &str) -> Resultat<(Self, SessionConfig)> {
        let mut cl = MpcSessionManagerClient::connect(sesman_url.to_owned())
            .await
            .map_err(|e| format!("{:#?}", e))
            .catch("CannotConnectGrpcServer", sesman_url)?;
        let cfg = cl
            .get_session_config(SessionTag {
                session_id: sid.to_owned(),
                expire_at: 0,
            })
            .await
            .catch("GrpcCallFailed", "MpcSessionManager::GetSessionConfig")?
            .into_inner();
        let _self = Self {
            sid: sid.to_string(),
            cl,
            tx: Vec::new(),
            rx: HashMap::new(),
        };
        Ok((_self, cfg))
    }
}

#[tonic::async_trait]
impl BatchMessenger for SvarogChannel {
    type ErrorType = Box<Erreur>;

    fn register_send<T>(
        &mut self,
        topic: &str,
        src: usize,
        dst: usize,
        seq: usize,
        obj: &T,
    ) -> Resultat<()>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
    {
        let obj = serde_pickle::to_vec(obj, Default::default()).catch_()?;
        let msg = Message {
            session_id: self.sid.to_owned(),
            topic: topic.to_owned(),
            src: src as u64,
            dst: dst as u64,
            seq: seq as u64,
            obj: Some(obj),
        };
        self.tx.push(msg);
        Ok(())
    }

    async fn execute_send(&mut self) -> Resultat<()> {
        let cl = &mut self.cl;
        let req = VecMessage {
            values: self.tx.drain(..).collect(),
        };
        let _ = cl
            .inbox(req)
            .await
            .catch("GrpcCallFailed", "MpcSessionManager::Inbox")?;
        Ok(())
    }

    fn clear_send(&mut self) {
        self.tx.clear();
    }

    fn register_receive(
        &mut self,
        topic: &str,
        src: usize,
        dst: usize,
        seq: usize,
    ) -> Resultat<()> {
        let key = MessageIndex {
            topic: topic.to_owned(),
            src,
            dst,
            seq,
        };
        self.rx.insert(key, None);
        Ok(())
    }

    async fn execute_receive(&mut self) -> Resultat<()> {
        let cl = &mut self.cl;
        let req = self
            .rx
            .iter()
            .map(|(idx, _)| Message {
                session_id: self.sid.clone(),
                topic: idx.topic.clone(),
                src: idx.src as u64,
                dst: idx.dst as u64,
                seq: idx.seq as u64,
                obj: None,
            })
            .collect();
        let req = VecMessage { values: req };
        let resp = cl
            .outbox(req)
            .await
            .catch("GrpcCallFailed", "MpcSessionManager::Outbox")?
            .into_inner();

        let mut key_set: HashSet<MessageIndex> = self.rx.keys().cloned().collect();
        for msg in resp.values.iter() {
            let key = MessageIndex {
                topic: msg.topic.clone(),
                src: msg.src as usize,
                dst: msg.dst as usize,
                seq: msg.seq as usize,
            };
            assert_throw!(key_set.contains(&key), "Message not registered");
            let obj = msg.obj.clone().ifnone("", "Unexpected null message")?;
            key_set.remove(&key);
            self.rx.insert(key, Some(obj)); // update
        }
        assert_throw!(key_set.is_empty(), "Some messages are missing");

        Ok(())
    }

    fn unpack_receive<T>(&mut self, topic: &str, src: usize, dst: usize, seq: usize) -> Resultat<T>
    where
        T: Serialize + DeserializeOwned + Send + Sync,
    {
        let key = MessageIndex {
            topic: topic.to_owned(),
            src,
            dst,
            seq,
        };
        let val = self
            .rx
            .get(&key)
            .ifnone(
                "",
                format!(
                    "MessageIndex {}-{}-{}-{} is not registered.",
                    topic, src, dst, seq
                ),
            )?
            .as_ref()
            .ifnone("", "Unexpected null message")?;

        let obj = serde_pickle::from_slice(val, Default::default()).catch_()?;
        Ok(obj)
    }

    fn clear_receive(&mut self) {
        self.rx.clear();
    }
}
