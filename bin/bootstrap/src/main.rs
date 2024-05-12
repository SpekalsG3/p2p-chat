use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use axum::{Json, Router};
use axum::http::Uri;
use tokio::join;
use tokio::net::TcpListener;
use bootstrap::heartbeat::check_servers_heartbeat;
use bootstrap::routers::get_router;
use bootstrap::types::{ApiResponse, AppState, AppStateRc, NodeConfig};

async fn handle_404(
    path: Uri,
) -> Json<ApiResponse<(), String>> {
    Json(ApiResponse {
        success: false,
        data: None,
        error: Some(format!("Unknown route {}", path)),
    })
}

#[tokio::main]
async fn main() {
    let config = NodeConfig {
        addr: SocketAddr::from_str("127.0.0.1:6000").expect("Failed to parse SocketAddr"),
        network_name: "test".to_string(),
    };

    let app_state: AppStateRc = Arc::new(RwLock::new(AppState {
        network_name: config.network_name,
        servers: HashMap::new(),
    }));

    let heartbeat_task = tokio::spawn(
        check_servers_heartbeat(app_state.clone()),
    );

    let server_task = {
        let app = Router::new()
            .nest("/", get_router())
            .fallback(handle_404)
            .with_state(app_state)
            ;

        let listener = TcpListener::bind(config.addr).await.expect("Failed to bind TcpListener");
        tokio::spawn(async move {
            axum::serve(listener, app).await
        })
    };

    let joined = join!(
        heartbeat_task,
        server_task,
    );
    joined.0.expect("Heartbeat thread panic");
    joined.1.expect("API thread panic").expect("API server failed");
    unreachable!("Threads are not supposed to finish") // unreachable until `ctrl+c` handle is implemented
}
