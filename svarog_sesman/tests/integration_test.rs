use erreur::*;
use mpc_sig_abs::*;
use rand::{rngs::OsRng, Rng};
use svarog_grpc::SessionConfig;
use svarog_sesman::SvarogChannel;

#[tokio::test]
async fn test_client() -> Resultat<()> {
    let cfg = mock_sesconf();
    let mut players: Vec<String> = cfg
        .players
        .iter()
        .filter_map(|(name, att)| if *att { Some(name.clone()) } else { None })
        .collect();
    players.sort();

    let i_set: Vec<usize> = (1..=players.len()).collect();
    let messenger = SvarogChannel::new_session(&cfg, "http://127.0.0.1:2000")
        .await
        .catch_()?;
    println!("Session ID: {}", &messenger.sid());

    let mut threads = vec![];
    for i in i_set.iter() {
        let j_set: Vec<usize> = i_set.iter().filter(|j| *j != i).cloned().collect();
        let h = tokio::spawn(thread_body(*i, j_set, messenger.clone()));
        threads.push(h);
    }

    for h in threads {
        h.await
            .catch("ThreadFailedForPanic", "")?
            .catch("ThreadFailedForException", "")?;
    }

    Ok(())
}

async fn thread_body(i: usize, j_set: Vec<usize>, mut chan: SvarogChannel) -> Resultat<()> {
    let mut rng = OsRng;
    let mersenne9 = 1u64 << 61 - 1;
    let obj = rng.gen::<u64>().rem_euclid(mersenne9);
    for j in j_set.iter() {
        chan.register("round1", i, *j, 0, &obj).catch_()?;
    }
    chan.execute().await.catch_()?;
    let mut sum = obj;
    for j in j_set.iter() {
        let obj_j: u64 = chan.unpack_receive("round1", *j, i, 0).catch_()?;
        sum += obj_j;
        sum = sum.rem_euclid(mersenne9);
    }
    chan.clear();

    for j in j_set.iter() {
        chan.register("round2", i, *j, 0, &sum).catch_()?;
    }
    chan.execute().await.catch_()?;
    for j in j_set.iter() {
        let obj: u64 = chan.unpack_receive("round2", *j, i, 0).catch_()?;
        assert_throw!(obj == sum);
    }
    chan.clear();

    Ok(())
}

fn mock_sesconf() -> SessionConfig {
    let mut cfg = SessionConfig::default();
    cfg.players = vec!["fluorine", "chlorine", "bromine", "iodine"]
        .iter()
        .map(|k| (k.to_string(), true))
        .collect();

    cfg
}
