use erreur::*;

use clap::{Arg, ArgAction, Command};
use svarog_grpc::{mpc_peer_client::MpcPeerClient, Algorithm, ParamsKeygen, ParamsSign, SignTask};

mod client_impl;
use client_impl::*;

#[tokio::main]
async fn main() -> Resultat<()> {
    let matches = Command::new("svpeer_toyclient")
        .arg(
            Arg::new("algo")
                .short('a')
                .required(false)
                .default_value("")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("member_name")
                .short('n')
                .required(false)
                .default_value("")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("new_session")
                .long("new_session")
                .action(ArgAction::SetTrue),
        )
        .get_matches();
    let mode: String = matches.get_one::<String>("mode").ifnone_()?.to_owned();
    let mode = mode.as_str();
    let algo = matches.get_one::<String>("algo").ifnone_()?.to_owned();
    let member_name: String = matches
        .get_one::<String>("member_name")
        .ifnone_()?
        .to_owned();
    let new_session = matches.get_flag("new_session") && algo != "";
    let not_new_session = member_name != "";
    assert_throw!(new_session ^ not_new_session);
    let sesman_url = "http://127.0.0.1:9000";
    let peer_url = "http://127.0.0.1:9001";

    let algo = match algo.as_str() {
        "gg18" => Algorithm::Gg18Secp256k1,
        "frost" => Algorithm::FrostEd25519,
        "" => Algorithm::None,
        _ => throw!("UnknownAlgorithm", algo),
    };

    let mut cl = MpcPeerClient::connect(peer_url).await.catch_()?;
    match (mode, new_session) {
        ("keygen", true) => {
            let conf = sesconf_keygen(&algo, sesman_url);
            let tag = cl.new_session(conf).await.catch_()?;
            let tag = tag.into_inner();
            println!("SessionTag: {:?}", tag);
        }
        ("keygen", false) => {
            let params = ParamsKeygen {
                sesman_url: sesman_url.to_owned(),
                session_id: SESID_KEYGEN.to_owned(),
                member_name: member_name.clone(),
                mnemonics: None,
            };
            let root_addr = cl.keygen(params).await.catch_()?;
            let root_addr = root_addr.into_inner().value;
            println!("RootAddr: {}", root_addr);
        }
        ("sign", true) => {
            let conf = sesconf_sign(&algo, sesman_url);
            let tag = cl.new_session(conf).await.catch_()?;
            let tag = tag.into_inner();
            println!("SessionTag: {:?}", tag);
        }
        ("sign", false) => {
            let params = ParamsSign {
                sesman_url: sesman_url.to_owned(),
                session_id: SESID_SIGN.to_owned(),
                member_name: member_name.clone(),
                key_name: SESID_KEYGEN.to_owned(),
                tasks: vec![SignTask {
                    derivation_path: "m/1/14/5/14".to_owned(),
                    tx_hash: hex::decode(TX_HASHES[0]).unwrap(),
                }],
            };
            let sigs = cl.sign(params).await.catch_()?;
            let sigs = sigs.into_inner().values;
            for (i, sig) in sigs.iter().enumerate() {
                let i = i + 1;
                println!("Signature #{} {{", i);
                println!("  r: {}", bs58::encode(&sig.r).into_string());
                println!("  s: {}", bs58::encode(&sig.s).into_string());
                println!("  v: {}", sig.v);
                println!("}}");
            }
        }
        ("sign_batch", _) => {
            throw!("NotImplemented", "sign_batch");
        }
        ("reshare", _) => {
            throw!("NotImplemented", "reshare");
        }
        (mode, _) => throw!("UnknownMode", mode),
    }

    Ok(())
}
