use clap::{value_parser, Arg, ArgAction, Command};
use erreur::*;
use svarog_grpc::mpc_session_manager_server::{
    MpcSessionManagerServer, // server struct
};
use tonic::transport::Server;

mod server_impl;
pub use server_impl::*;

#[tokio::main]
async fn main() -> Resultat<()> {
    // Parse args
    let matches = Command::new("svarog_sesman")
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
                .default_value("9000")
                .value_parser(value_parser!(u16))
                .action(ArgAction::Set),
        )
        .disable_help_flag(true)
        .get_matches();
    let host: String = matches.get_one::<String>("host").ifnone_()?.to_owned();
    let port: u16 = matches.get_one::<u16>("port").ifnone_()?.to_owned();
    println!("svarog_sesman will listen on {}:{}", &host, port);

    // Init service
    let (sesman, recycle_task_handle) = Sesman::init(1200).await.catch_()?;

    // Start server
    Server::builder()
        .add_service(MpcSessionManagerServer::new(sesman))
        .serve(format!("{host}:{port}").parse().unwrap())
        .await
        .catch("GrpcServerIsDown", "MpcSessionManager")?;

    recycle_task_handle.abort();

    Ok(())
}
