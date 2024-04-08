use std::collections::BTreeSet;

use erreur::*;
use svarog_grpc::{Keystore, SignTask, Signature};
use svarog_sesman::SvarogChannel;

use super::util::*;

async fn keygen_gg18(chan: SvarogChannel, (i, n, t): (usize, usize, usize)) -> Resultat<Keystore> {
    use svarog_algo_flat::gg18::keygen;

    let players: BTreeSet<usize> = (1..=n).collect();
    let keystore = keygen(chan, players, t, i, None).await.catch_()?;
    let keystore = keystore.to_proto().catch_()?;
    Ok(keystore)
}

async fn keygen_frost(chan: SvarogChannel, (i, n, t): (usize, usize, usize)) -> Resultat<Keystore> {
    use svarog_algo_flat::frost::keygen;

    let players: BTreeSet<usize> = (1..=n).collect();
    let sid = chan.sid().to_owned();
    let keystore = keygen(chan, players, t, i, None, sid).await.catch_()?;
    let keystore = keystore.to_proto().catch_()?;
    Ok(keystore)
}

async fn keygen_mnem_gg18(
    chan: SvarogChannel,
    (i, n, t): (usize, usize, usize),
    mnem: Option<(String, String)>,
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::gg18::{keygen_mnem_consumer, keygen_mnem_provider};
    let players: BTreeSet<usize> = (1..=n).collect();

    let provider_thread = if let Some((mnem, pwd)) = mnem {
        let future: _ = keygen_mnem_provider(chan.clone(), players.clone(), mnem, pwd);
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

async fn keygen_mnem_frost(
    chan: SvarogChannel,
    (i, n, t): (usize, usize, usize),
    mnem: Option<(String, String)>,
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::gg18::{keygen_mnem_consumer, keygen_mnem_provider};
    let players: BTreeSet<usize> = (1..=n).collect();

    let provider_thread = if let Some((mnem, pwd)) = mnem {
        let future: _ = keygen_mnem_provider(chan.clone(), players.clone(), mnem, pwd);
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

async fn sign_gg18(
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

async fn sign_frost(
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

async fn reshare_gg18(
    chan: SvarogChannel,
    keystore: Option<Keystore>,
    providers: BTreeSet<usize>,
    (i, n, t): (usize, usize, usize),
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::gg18::{reshare_consumer, reshare_provider, KeystoreEcdsa};
    let consumers: BTreeSet<usize> = (1..=n).collect();

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

async fn reshare_frost(
    chan: SvarogChannel,
    keystore: Option<Keystore>,
    providers: BTreeSet<usize>,
    (i, n, t): (usize, usize, usize),
) -> Resultat<Option<Keystore>> {
    use svarog_algo_flat::frost::{reshare_consumer, reshare_provider, KeystoreSchnorr};
    let consumers: BTreeSet<usize> = (1..=n).collect();

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
