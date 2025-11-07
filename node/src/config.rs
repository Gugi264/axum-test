use clap::Parser;
use std::net::SocketAddr;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct NodeArgs {
    /// Bind addr
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:3000")]
    pub bind_addr: SocketAddr,

    #[arg(long, env = "NODE_NR")]
    pub node_nr: u32,
}
