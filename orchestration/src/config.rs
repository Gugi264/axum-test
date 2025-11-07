use clap::Parser;
use std::net::SocketAddr;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Bind addr
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:3000")]
    pub bind_addr: SocketAddr,
    /// MPC
    #[arg(long, env = "NODE_SERVICE_ADDR", value_delimiter = ',')]
    pub services: Vec<String>,
}
