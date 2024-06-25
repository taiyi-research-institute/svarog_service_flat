#![allow(nonstandard_style)]
use std::collections::{BTreeMap, BTreeSet};

use erreur::*;
use mock_data::{mock_mnem, mock_sign_tasks};
use svarog_peer::{btc, new_session, solana};

// 改成通配符引用之后, 会难以检查到底用了哪些符号. 通配符看着优雅, 但是不利于代码审查.
use crate::mock_data::{mock_keygen_config, mock_sign_config, players1, th1};

mod mock_data;
const sesman_url: &str = "http://127.0.0.1:2000";

/// 集成测试普通的keygen, sign
#[tokio::main]
async fn main() -> Resultat<()> {
    test_btc().await.catch_()?;
    test_solana().await.catch_()?;
    Ok(())
}

async fn test_btc() -> Resultat<()> {
    let keystores = {
        let cfg = mock_keygen_config(th1, &players1, sesman_url);
        let sid = new_session(cfg.clone()).await.catch_()?;

        let mut threads = BTreeMap::new();
        '_mnem_provider: {
            let future = btc::biz_keygen_mnem(
                sesman_url.to_owned(),
                sid.clone(),
                "".to_owned(),
                Some(mock_mnem()),
            );
            let thread = tokio::spawn(future);
            threads.insert("".to_owned(), thread);
        }
        for (player, _) in cfg.players.iter() {
            let future =
                btc::biz_keygen_mnem(sesman_url.to_owned(), sid.clone(), player.clone(), None);
            let thread = tokio::spawn(future);
            threads.insert(player.clone(), thread);
        }
        let mut keystores = BTreeMap::new();
        for (player, thread) in threads.iter_mut() {
            let resp = thread.await.catch("Panic", "")?.catch("Exception", "")?;
            if let Some(resp) = resp {
                keystores.insert(player.clone(), resp);
            }
        }
        keystores
    };

    let signatures = {
        let cfg = mock_sign_config(th1, &players1, sesman_url);
        '_print_signers: {
            let mut signers = BTreeSet::new();
            for (player, &att) in cfg.players.iter() {
                if false == att {
                    continue;
                }
                signers.insert(player);
            }
        }
        let sid = new_session(cfg.clone()).await.catch_()?;
        let mut threads = BTreeMap::new();
        for (player, &att) in cfg.players.iter() {
            if false == att {
                continue;
            }
            let keystore = keystores.get(player).ifnone_()?;
            let future = btc::biz_sign(
                sesman_url.to_owned(),
                sid.clone(),
                keystore.clone(),
                mock_sign_tasks(),
            );
            let thread = tokio::spawn(future);
            threads.insert(player, thread);
        }
        let mut signatures = BTreeMap::new();
        for (&player, thread) in threads.iter_mut() {
            let resp = thread.await.catch("Panic", "")?.catch("Exception", "")?;
            signatures.insert(player.clone(), resp);
        }
        signatures
    };

    let mut sig_it = signatures.values();
    let sig0 = sig_it.next().ifnone_()?;
    for sig in sig_it {
        assert_throw!(sig == sig0);
    }

    Ok(())
}

async fn test_solana() -> Resultat<()> {
    let keystores = {
        let cfg = mock_keygen_config(th1, &players1, sesman_url);
        let sid = new_session(cfg.clone()).await.catch_()?;

        let mut threads = BTreeMap::new();
        '_mnem_provider: {
            let future = solana::biz_keygen_mnem(
                sesman_url.to_owned(),
                sid.clone(),
                "".to_owned(),
                Some(mock_mnem()),
            );
            let thread = tokio::spawn(future);
            threads.insert("".to_owned(), thread);
        }
        for (player, _) in cfg.players.iter() {
            let future =
                solana::biz_keygen_mnem(sesman_url.to_owned(), sid.clone(), player.clone(), None);
            let thread = tokio::spawn(future);
            threads.insert(player.clone(), thread);
        }
        let mut keystores = BTreeMap::new();
        for (player, thread) in threads.iter_mut() {
            let resp = thread.await.catch("Panic", "")?.catch("Exception", "")?;
            if let Some(resp) = resp {
                keystores.insert(player.clone(), resp);
            }
        }
        keystores
    };

    let signatures = {
        let cfg = mock_sign_config(th1, &players1, sesman_url);
        '_print_signers: {
            let mut signers = BTreeSet::new();
            for (player, &att) in cfg.players.iter() {
                if false == att {
                    continue;
                }
                signers.insert(player);
            }
        }
        let sid = new_session(cfg.clone()).await.catch_()?;
        let mut threads = BTreeMap::new();
        for (player, &att) in cfg.players.iter() {
            if false == att {
                continue;
            }
            let keystore = keystores.get(player).ifnone_()?;
            let future = solana::biz_sign(
                sesman_url.to_owned(),
                sid.clone(),
                keystore.clone(),
                mock_sign_tasks(),
            );
            let thread = tokio::spawn(future);
            threads.insert(player, thread);
        }
        let mut signatures = BTreeMap::new();
        for (&player, thread) in threads.iter_mut() {
            let resp = thread.await.catch("Panic", "")?.catch("Exception", "")?;
            signatures.insert(player.clone(), resp);
        }
        signatures
    };

    let mut sig_it = signatures.values();
    let sig0 = sig_it.next().ifnone_()?;
    for sig in sig_it {
        assert_throw!(sig == sig0);
    }

    Ok(())
}
