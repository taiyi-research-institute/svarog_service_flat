use erreur::*;

mod server_impl;
use server_impl::*;
use svarog_grpc::mpc_peer_server::MpcPeerServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Resultat<()> {
    // Parse args
    use clap::{value_parser, Arg, ArgAction, Command};
    let matches = Command::new("svarog_peer")
        .arg(
            Arg::new("host")
                .short('h')
                .required(false)
                .default_value("0.0.0.0")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .required(false)
                .default_value("2001")
                .value_parser(value_parser!(u16))
                .action(ArgAction::Set),
        )
        .disable_help_flag(true)
        .get_matches();
    let host: String = matches.get_one::<String>("host").ifnone_()?.to_owned();
    let port: u16 = matches.get_one::<u16>("port").ifnone_()?.to_owned();

    println!("ver: 2024.06.14 16:40");
    println!(
        "svarog_peer will listen on {}:{}",
        &host, port
    );

    Server::builder()
        .add_service(MpcPeerServer::new(SvarogPeer {}))
        .serve(format!("{host}:{port}").parse().unwrap())
        .await
        .catch("GrpcServerIsDown", "MpcPeer")?;

    Ok(())
}
