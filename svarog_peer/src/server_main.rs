use erreur::*;

mod server_impl;
use server_impl::*;
use svarog_grpc::mpc_peer_server::MpcPeerServer;
use tonic::transport::{Identity, Server, ServerTlsConfig};

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
        .arg(Arg::new("https").long("https").action(ArgAction::SetTrue))
        .disable_help_flag(true)
        .get_matches();
    let host: String = matches.get_one::<String>("host").ifnone_()?.to_owned();
    let port: u16 = matches.get_one::<u16>("port").ifnone_()?.to_owned();
    let https: bool = matches.get_flag("https");
    println!("svarog_peer will listen on {}:{}", &host, port);

    // Start server
    let mut server = Server::builder();
    if https {
        let cert = tokio::fs::read_to_string("tls/cert.pem").await.catch_()?;
        let key = tokio::fs::read_to_string("tls/privkey.pem")
            .await
            .catch_()?;
        let ident = Identity::from_pem(cert, key);
        server = server
            .tls_config(ServerTlsConfig::new().identity(ident))
            .catch_()?;
    }
    server
        .add_service(MpcPeerServer::new(SvarogPeer { https }))
        .serve(format!("{host}:{port}").parse().unwrap())
        .await
        .catch("GrpcServerIsDown", "MpcPeer")?;

    Ok(())
}
