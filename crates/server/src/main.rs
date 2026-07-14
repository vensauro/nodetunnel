#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(rust_2018_idioms)]
#![warn(unused_qualifications)]
#![warn(unused_crate_dependencies)]

use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use crate::relay::server::RelayServer;
use crate::udp::paper_interface::PaperInterface;

mod config;
mod udp;
mod relay;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    dotenvy::dotenv().ok();
    let config = config::loader::load_config()?;
    println!("{:?}", config.allowed_versions);
    let addr: SocketAddr = config.udp_bind_address
        .to_socket_addrs()?
        .next()
        .ok_or("Failed to resolve host name")?;

    let transport = PaperInterface::new(addr).await?;

    let mut server = RelayServer::new(transport, config);
    info!("relay server started");
    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                error!("server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("shutdown signal received");
        }
    }

    info!("shutting down server");
    server.cleanup().await;

    Ok(())
}
