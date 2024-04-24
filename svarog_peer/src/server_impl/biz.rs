/// 模块职责:
/// 1. 将内部的两种Keystore和Signature转换为相同的gRPC消息格式; 以及反向转换.
/// 2. 为`keygen_mnem`, `reshare`这两个操作开辟线程. 这两个操作都有provider和consumer两个角色.
use std::collections::BTreeSet;

use erreur::*;
use svarog_grpc::{Keystore, Mnemonic, SignTask, Signature};
use svarog_sesman::SvarogChannel;

use super::conversion::*;

pub(crate) async fn keygen_gg18(
    chan: SvarogChannel,
    i: usize,
    t: usize,
    players: BTreeSet<usize>,
) -> Resultat<Keystore> {
    use svarog_algo_flat::gg18::keygen;

    let keystore = keygen(chan, players, t, i, None).await.catch_()?;
    let keystore = keystore.to_proto().catch_()?;
    Ok(keystore)
}

pub(crate) async fn keygen_frost(
    chan: SvarogChannel,
    i: usize,
    t: usize,
    players: BTreeSet<usize>,
) -> Resultat<Keystore> {
    use svarog_algo_flat::frost::keygen;

    let sid = chan.sid().to_owned();
    let keystore = keygen(chan, players, t, i, None, sid).await.catch_()?;
    let keystore = keystore.to_proto().catch_()?;
    Ok(keystore)
}

pub(crate) async fn keygen_mnem_gg18(
    chan: SvarogChannel,
    i: usize,
    t: usize,
    players: BTreeSet<usize>,
    mnem: Option<Mnemonic>,
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::gg18::{keygen_mnem_consumer, keygen_mnem_provider};

    let provider_thread = if let Some(mnem) = mnem {
        let future: _ =
            keygen_mnem_provider(chan.clone(), players.clone(), mnem.words, mnem.password);
        let handle: _ = tokio::spawn(future);
        Some(handle)
    } else {
        None
    };

    let ret = if i > 0 {
        let keystore = keygen_mnem_consumer(chan, players, t, i).await.catch_()?;
        let keystore = keystore.to_proto().catch_()?;
        Some(keystore)
    } else {
        None
    };

    if let Some(handle) = provider_thread {
        handle
            .await
            .catch("ThreadFailed", "due to panic")?
            .catch("ThreadFailed", "due to exception")?;
    }

    Ok(ret)
}

pub(crate) async fn keygen_mnem_frost(
    chan: SvarogChannel,
    i: usize,
    t: usize,
    players: BTreeSet<usize>,
    mnem: Option<Mnemonic>,
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::frost::{keygen_mnem_consumer, keygen_mnem_provider};

    let provider_thread = if let Some(mnem) = mnem {
        let future: _ =
            keygen_mnem_provider(chan.clone(), players.clone(), mnem.words, mnem.password);
        let handle: _ = tokio::spawn(future);
        Some(handle)
    } else {
        None
    };

    let sid = chan.sid().to_owned();

    let ret = if i > 0 {
        let keystore = keygen_mnem_consumer(chan, players, t, i, sid)
            .await
            .catch_()?;
        let keystore = keystore.to_proto().catch_()?;
        Some(keystore)
    } else {
        None
    };

    if let Some(handle) = provider_thread {
        handle
            .await
            .catch("ThreadFailed", "due to panic")?
            .catch("ThreadFailed", "due to exception")?;
    }

    Ok(ret)
}

pub(crate) async fn sign_gg18(
    chan: SvarogChannel,
    keystore: Keystore,
    signers: BTreeSet<usize>,
    tasks: Vec<SignTask>,
) -> Resultat<Vec<Signature>> {
    use svarog_algo_flat::gg18::{sign, sign_batch, KeystoreEcdsa};

    assert_throw!(tasks.len() >= 1);
    assert_throw!(signers.len() >= 1);
    let keystore = KeystoreEcdsa::from_proto(&keystore).catch_()?;
    assert_throw!(signers.contains(&keystore.i));
    let res = if tasks.len() == 1 {
        let task = tasks.into_iter().next().unwrap();
        let (dpath, hmsg) = (task.derivation_path, task.tx_hash);
        let sig = sign(chan, signers, keystore, hmsg, dpath).await.catch_()?;
        let sig = sig.to_proto().catch_()?;
        vec![sig]
    } else {
        let tasks = tasks
            .into_iter()
            .map(|task| (task.tx_hash, task.derivation_path))
            .collect();
        let sigs = sign_batch(chan, signers, keystore, tasks).await.catch_()?;
        let mut res = Vec::new();
        for sig in sigs.into_iter() {
            let sig = sig.to_proto().catch_()?;
            res.push(sig);
        }
        res
    };
    Ok(res)
}

pub(crate) async fn sign_frost(
    chan: SvarogChannel,
    keystore: Keystore,
    signers: BTreeSet<usize>,
    tasks: Vec<SignTask>,
) -> Resultat<Vec<Signature>> {
    use svarog_algo_flat::frost::{sign, sign_batch, KeystoreSchnorr};

    assert_throw!(tasks.len() >= 1);
    assert_throw!(signers.len() >= 1);
    let keystore = KeystoreSchnorr::from_proto(&keystore).catch_()?;
    assert_throw!(signers.contains(&keystore.i));
    let res = if tasks.len() == 1 {
        let task = tasks.into_iter().next().unwrap();
        let (dpath, hmsg) = (task.derivation_path, task.tx_hash);
        let sig = sign(chan, signers, keystore, hmsg, dpath).await.catch_()?;
        let sig = sig.to_proto().catch_()?;
        vec![sig]
    } else {
        let tasks = tasks
            .into_iter()
            .map(|task| (task.tx_hash, task.derivation_path))
            .collect();
        let sigs = sign_batch(chan, signers, keystore, tasks).await.catch_()?;
        let mut res = Vec::new();
        for sig in sigs.into_iter() {
            let sig = sig.to_proto().catch_()?;
            res.push(sig);
        }
        res
    };
    Ok(res)
}

pub(crate) async fn reshare_gg18(
    chan: SvarogChannel,
    keystore: Option<Keystore>,
    i: usize,
    t: usize,
    providers: BTreeSet<usize>,
    consumers: BTreeSet<usize>,
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::gg18::{reshare_consumer, reshare_provider, KeystoreEcdsa};
    let provider_thread = if let Some(keystore) = keystore {
        let keystore = KeystoreEcdsa::from_proto(&keystore).catch_()?;
        let future: _ =
            reshare_provider(chan.clone(), keystore, providers.clone(), consumers.clone());
        let handle: _ = tokio::spawn(future);
        Some(handle)
    } else {
        None
    };

    let ret = if i > 0 {
        // let sid = chan.sid().to_owned();
        let keystore = reshare_consumer(chan, t, i, providers, consumers)
            .await
            .catch_()?;
        let keystore = keystore.to_proto().catch_()?;
        Some(keystore)
    } else {
        None
    };

    if let Some(handle) = provider_thread {
        handle
            .await
            .catch("ThreadFailed", "due to panic")?
            .catch("ThreadFailed", "due to exception")?;
    }

    Ok(ret)
}

pub(crate) async fn reshare_frost(
    chan: SvarogChannel,
    keystore: Option<Keystore>,
    i: usize,
    t: usize,
    providers: BTreeSet<usize>,
    consumers: BTreeSet<usize>,
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::frost::{reshare_consumer, reshare_provider, KeystoreSchnorr};

    let provider_thread = if let Some(keystore) = keystore {
        let keystore = KeystoreSchnorr::from_proto(&keystore).catch_()?;
        let future: _ =
            reshare_provider(chan.clone(), keystore, providers.clone(), consumers.clone());
        let handle: _ = tokio::spawn(future);
        Some(handle)
    } else {
        None
    };

    let ret = if i > 0 {
        let sid = chan.sid().to_owned();
        let keystore = reshare_consumer(chan, t, i, providers, consumers, sid)
            .await
            .catch_()?;
        let keystore = keystore.to_proto().catch_()?;
        Some(keystore)
    } else {
        None
    };

    if let Some(handle) = provider_thread {
        handle
            .await
            .catch("ThreadFailed", "due to panic")?
            .catch("ThreadFailed", "due to exception")?;
    }

    Ok(ret)
}
