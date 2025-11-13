use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    routing::{get, post},
};
use clap::Parser;
use node::config::NodeArgs;
use node::structs::MpcNodeAddresses;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{Instant, interval};
use tower_http::set_header::SetResponseHeaderLayer;

#[derive(Clone, Debug)]
struct AppState {
    node_nr: u32,
    nodes: Arc<Mutex<Vec<MpcNodeAddresses>>>,
}

#[tokio::main]
async fn main() {
    let args = NodeArgs::parse();
    let state = AppState {
        node_nr: args.node_nr,
        nodes: Arc::new(Mutex::new(Vec::new())),
    };
    let app = Router::new()
        .route("/", get(root))
        .route("/nodes", post(set_nodes_addr))
        .route("/health", get(stealthy_healthy))
        .route("/health_logged", get(healthy_logged))
        .route("/info", get(info))
        .route("/hello_from/{node_nr}", get(hello_from))
        .with_state(state)
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        ));

    let listener = tokio::net::TcpListener::bind(args.bind_addr).await.unwrap();
    println!("Starting node on {}", args.bind_addr);
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello World!"
}

async fn info(State(state): State<AppState>) -> (StatusCode, String) {
    // println!("{:?}", state);
    (StatusCode::OK, state.node_nr.to_string())
}

async fn healthy_logged() -> (StatusCode, String) {
    println!("got health check");
    (StatusCode::OK, "healthy".to_string())
}

async fn stealthy_healthy() -> (StatusCode, String) {
    (StatusCode::OK, "healthy".to_string())
}

async fn set_nodes_addr(
    State(state): State<AppState>,
    Json(payload): Json<Vec<MpcNodeAddresses>>,
) -> (StatusCode, String) {
    println!("got node addresses: {:?}", payload);
    let mut nodes = state.nodes.lock().expect("mutex was poisened");
    *nodes = payload;

    nodes.iter().cloned().for_each(|service| {
        tokio::spawn(async move {
            send_interval_to_node(service.address, state.node_nr).await;
        });
    });

    (StatusCode::OK, "".to_string())
}

async fn hello_from(Path(id): Path<u32>) -> StatusCode {
    println!("Got hello from: {id}");
    StatusCode::OK
}

async fn send_interval_to_node(url: String, node_nr: u32) {
    let client = Client::new();

    let mut ticker = interval(Duration::from_secs(10));

    loop {
        ticker.tick().await;
        let started = Instant::now();
        match client
            .get(url.clone() + "/hello_from/" + &node_nr.to_string())
            .send()
            .await
        {
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
