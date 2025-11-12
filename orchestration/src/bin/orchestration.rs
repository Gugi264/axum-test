use reqwest::Client;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::{Instant, interval};

use axum::{
    Json, Router,
    http::{HeaderValue, StatusCode, header},
    routing::get,
};
use clap::Parser;
use node::structs::MpcNodeAddresses;
use orchestration::config::Args;
use tower_http::set_header::SetResponseHeaderLayer;
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let services: Vec<MpcNodeAddresses> = args
        .services
        .iter()
        .enumerate()
        .map(|(i, val)| MpcNodeAddresses {
            node_nr: i as u32,
            address: val.to_string(),
        })
        .collect();

    let health_result = services_health_check(&services, Duration::from_secs(10)).await;
    println!("Health check result: {:?}", health_result);
    services.iter().cloned().for_each(|service| {
        tokio::spawn(async move {
            send_interval(service.address).await;
        });
    });

    let _ = send_nodes_addr(services, Duration::from_secs(1)).await;
    let app = Router::new()
        .route("/", get(root))
        // .route("/setNodesAddress", post(set_nodes_addr))
        .route("/health", get(healthy))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        ));

    let listener = tokio::net::TcpListener::bind(args.bind_addr).await.unwrap();
    println!("Starting orchestration server on {}", args.bind_addr);
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello World!"
}

async fn healthy() -> (StatusCode, String) {
    (StatusCode::OK, "healthy".to_string())
}

async fn set_nodes_addr(
    Json(payload): Json<node::structs::MpcNodeAddresses>,
) -> (StatusCode, String) {
    println!("{:?}", payload);
    (StatusCode::OK, "".to_string())
}
pub async fn services_health_check(
    services: &Vec<MpcNodeAddresses>,
    max_wait_time: Duration,
) -> eyre::Result<()> {
    println!("starting health checks");
    let health_checks = services
        .iter()
        .map(|service| health_check(format!("{}/health_logged", service.address)))
        .collect::<JoinSet<_>>();

    tokio::time::timeout(max_wait_time, health_checks.join_all())
        .await
        .map_err(|_| eyre::eyre!("services not healthy in provided time: {max_wait_time:?}"))?;
    Ok(())
}

async fn health_check(health_url: String) {
    loop {
        if reqwest::get(&health_url).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    // tracing::info!("healthy: {health_url}");
}

async fn send_nodes_addr(
    nodes: Vec<MpcNodeAddresses>,
    max_wait_time: Duration,
) -> eyre::Result<()> {
    println!("sending....");
    println!("{:?}", nodes);

    let sending = nodes
        .iter()
        .map(|node| send_single_node_addr(format!("{}/nodes", node.address), nodes.clone()))
        .collect::<JoinSet<_>>();

    tokio::time::timeout(max_wait_time, sending.join_all())
        .await
        .map_err(|_| eyre::eyre!("service post went wrong: {max_wait_time:?}"))?;

    println!("sending addresses");

    Ok(())
}

async fn send_single_node_addr(node_url: String, nodes: Vec<MpcNodeAddresses>) {
    let client = Client::new();
    let res = client.post(&node_url).json(&nodes).send().await;
    match res {
        Ok(a) => println!("success: {:?}", a),
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

async fn send_interval(url: String) {
    let client = Client::new();

    let mut ticker = interval(Duration::from_secs(3));

    loop {
        ticker.tick().await;
        let started = Instant::now();
        match client.get(url.clone() + "/health").send().await {
            Ok(r) => {
                let status = r.status();
                let text = r.text().await.unwrap_or_default();
                // println!(
                //     "[{:?}] Status: {} Body len: {}",
                //     started,
                //     status,
                //     text.len()
                // );
            }
            Err(e) => eprintln!("Request failed: {e}"),
        }
    }
}
